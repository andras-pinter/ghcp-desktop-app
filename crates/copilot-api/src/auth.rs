//! OAuth device flow authentication for GitHub Copilot.
//!
//! Flow:
//! 1. `request_device_code()` → user sees code, visits verification URL
//! 2. `poll_for_token()` → polls until user authorizes or timeout
//! 3. `exchange_for_copilot_token()` → exchanges GitHub token for Copilot JWT
//! 4. Copilot token auto-refreshed before expiry

use crate::keychain;
use crate::types::{
    CopilotTokenResponse, DeviceCodeResponse, GitHubUser, OAuthErrorResponse, OAuthTokenResponse,
};
use thiserror::Error;

/// GitHub OAuth app client ID for Copilot (same as VS Code uses).
pub const GITHUB_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_COPILOT_TOKEN_URL: &str = "https://api.github.com/copilot_internal/v2/token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

/// Default Copilot API base URL (used when token response has no endpoints).
pub const DEFAULT_COPILOT_API_BASE: &str = "https://api.individual.githubcopilot.com";

/// Keychain keys.
const KEY_GITHUB_TOKEN: &str = "github_oauth_token";
const KEY_COPILOT_TOKEN: &str = "copilot_token";
const KEY_COPILOT_EXPIRES: &str = "copilot_token_expires_at";
const KEY_COPILOT_API_BASE: &str = "copilot_api_base";

/// Errors that can occur during authentication.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Authorization pending — user has not entered the code yet")]
    AuthorizationPending,
    #[error("Polling too fast — slow down")]
    SlowDown,
    #[error("Device code expired — restart the flow")]
    ExpiredToken,
    #[error("User denied access")]
    AccessDenied,
    #[error("OAuth error: {0}")]
    OAuthError(String),
    #[error("Copilot token exchange failed: {status} {body}")]
    CopilotTokenExchange { status: u16, body: String },
    #[error("Keychain error: {0}")]
    Keychain(#[from] keychain::KeychainError),
    #[error("No GitHub token stored — user must authenticate first")]
    NotAuthenticated,
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Handles the GitHub OAuth device flow for Copilot authentication.
pub struct DeviceFlowAuth {
    client: reqwest::Client,
    client_id: String,
}

impl DeviceFlowAuth {
    /// Create a new device flow auth handler with the default Copilot client ID.
    pub fn new() -> Self {
        Self::with_client_id(GITHUB_CLIENT_ID.to_string())
    }

    /// Create with a custom client ID (useful for testing).
    pub fn with_client_id(client_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            client_id,
        }
    }

    /// Step 1: Request a device code from GitHub.
    ///
    /// Returns the device code response containing the user-facing code
    /// and the verification URL.
    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse, AuthError> {
        let resp = self
            .client
            .post(GITHUB_DEVICE_CODE_URL)
            .header("Accept", "application/json")
            .form(&[("client_id", self.client_id.as_str()), ("scope", "copilot")])
            .send()
            .await?
            .error_for_status()?
            .json::<DeviceCodeResponse>()
            .await?;

        Ok(resp)
    }

    /// Step 2: Poll once for the OAuth token.
    ///
    /// Call this in a loop with the interval from [`DeviceCodeResponse`].
    /// Returns `Ok(token_response)` on success, or an [`AuthError`]
    /// indicating the caller should retry, slow down, or abort.
    pub async fn poll_for_token(&self, device_code: &str) -> Result<OAuthTokenResponse, AuthError> {
        let resp = self
            .client
            .post(GITHUB_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?;

        let body = resp.text().await?;

        // Try parsing as success first
        if let Ok(token) = serde_json::from_str::<OAuthTokenResponse>(&body) {
            if !token.access_token.is_empty() {
                return Ok(token);
            }
        }

        // Parse as error
        let err: OAuthErrorResponse = serde_json::from_str(&body)?;
        match err.error.as_str() {
            "authorization_pending" => Err(AuthError::AuthorizationPending),
            "slow_down" => Err(AuthError::SlowDown),
            "expired_token" => Err(AuthError::ExpiredToken),
            "access_denied" => Err(AuthError::AccessDenied),
            other => Err(AuthError::OAuthError(
                err.error_description.unwrap_or_else(|| other.to_string()),
            )),
        }
    }

    /// Step 3: Exchange the GitHub OAuth token for a short-lived Copilot API token.
    pub async fn exchange_for_copilot_token(
        &self,
        github_token: &str,
    ) -> Result<CopilotTokenResponse, AuthError> {
        let resp = self
            .client
            .get(GITHUB_COPILOT_TOKEN_URL)
            .header("Authorization", format!("token {github_token}"))
            .header("Accept", "application/json")
            .header("User-Agent", "Chuck/0.1.0 (GitHub Copilot Desktop Client)")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::CopilotTokenExchange { status, body });
        }

        let token_resp = resp.json::<CopilotTokenResponse>().await?;
        Ok(token_resp)
    }

    /// Fetch the authenticated GitHub user's profile.
    pub async fn get_github_user(&self, github_token: &str) -> Result<GitHubUser, AuthError> {
        let user = self
            .client
            .get(GITHUB_USER_URL)
            .header("Authorization", format!("token {github_token}"))
            .header("Accept", "application/json")
            .header("User-Agent", "Chuck/0.1.0 (GitHub Copilot Desktop Client)")
            .send()
            .await?
            .error_for_status()?
            .json::<GitHubUser>()
            .await?;

        Ok(user)
    }

    /// Store auth tokens in the OS keychain after successful authentication.
    pub fn store_tokens(
        github_token: &str,
        copilot_resp: &CopilotTokenResponse,
    ) -> Result<(), AuthError> {
        keychain::store(KEY_GITHUB_TOKEN, github_token)?;
        keychain::store(KEY_COPILOT_TOKEN, &copilot_resp.token)?;
        keychain::store(KEY_COPILOT_EXPIRES, &copilot_resp.expires_at.to_string())?;

        let api_base = copilot_resp
            .endpoints
            .api
            .as_deref()
            .unwrap_or(DEFAULT_COPILOT_API_BASE);
        keychain::store(KEY_COPILOT_API_BASE, api_base)?;

        Ok(())
    }

    /// Load the stored GitHub OAuth token from the keychain.
    pub fn load_github_token() -> Result<String, AuthError> {
        keychain::retrieve(KEY_GITHUB_TOKEN).map_err(AuthError::from)
    }

    /// Load the stored Copilot API token from the keychain.
    ///
    /// Returns `(token, expires_at_unix, api_base_url)`.
    pub fn load_copilot_token() -> Result<(String, i64, String), AuthError> {
        let token = keychain::retrieve(KEY_COPILOT_TOKEN)?;
        let expires_str = keychain::retrieve(KEY_COPILOT_EXPIRES)?;
        let expires_at = expires_str.parse::<i64>().unwrap_or(0);
        let api_base = keychain::retrieve(KEY_COPILOT_API_BASE)
            .unwrap_or_else(|_| DEFAULT_COPILOT_API_BASE.to_string());

        Ok((token, expires_at, api_base))
    }

    /// Check if the stored Copilot token is still valid (with 5-minute buffer).
    pub fn is_copilot_token_valid() -> bool {
        match Self::load_copilot_token() {
            Ok((_, expires_at, _)) => {
                let now = chrono::Utc::now().timestamp();
                expires_at > now + 300 // 5-minute buffer
            }
            Err(_) => false,
        }
    }

    /// Refresh the Copilot token if expired. Returns the current valid token.
    pub async fn ensure_copilot_token(&self) -> Result<(String, String), AuthError> {
        if Self::is_copilot_token_valid() {
            let (token, _, api_base) = Self::load_copilot_token()?;
            return Ok((token, api_base));
        }

        // Token expired — refresh using stored GitHub token
        let github_token = Self::load_github_token().map_err(|_| AuthError::NotAuthenticated)?;
        let copilot_resp = self.exchange_for_copilot_token(&github_token).await?;

        let api_base = copilot_resp
            .endpoints
            .api
            .clone()
            .unwrap_or_else(|| DEFAULT_COPILOT_API_BASE.to_string());

        Self::store_tokens(&github_token, &copilot_resp)?;

        Ok((copilot_resp.token, api_base))
    }

    /// Clear all stored tokens from the keychain (logout).
    pub fn clear_tokens() -> Result<(), AuthError> {
        // Ignore NotFound errors — token might not exist
        let _ = keychain::delete(KEY_GITHUB_TOKEN);
        let _ = keychain::delete(KEY_COPILOT_TOKEN);
        let _ = keychain::delete(KEY_COPILOT_EXPIRES);
        let _ = keychain::delete(KEY_COPILOT_API_BASE);
        Ok(())
    }
}

impl Default for DeviceFlowAuth {
    fn default() -> Self {
        Self::new()
    }
}
