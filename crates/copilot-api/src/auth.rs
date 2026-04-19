//! OAuth device flow authentication for GitHub Copilot.
//!
//! Flow:
//! 1. `request_device_code()` → user sees code, visits verification URL
//! 2. `poll_for_token()` → polls until user authorizes or timeout
//! 3. `exchange_for_copilot_token()` → exchanges GitHub token for Copilot JWT
//! 4. Copilot token auto-refreshed before expiry
//!
//! All auth credentials are stored as a single consolidated JSON blob in the
//! OS keychain under one key (`auth_credentials`). This reduces macOS Keychain
//! password prompts from 4× to 1× after app updates (unsigned binary signature
//! changes trigger per-entry prompts).

use crate::keychain;
use crate::types::{
    CopilotTokenResponse, DeviceCodeResponse, GitHubUser, OAuthErrorResponse, OAuthTokenResponse,
};
use crate::user_agent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// GitHub OAuth app client ID for Copilot (same as VS Code uses).
pub const GITHUB_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";

/// Refresh the Copilot JWT this many seconds before it actually expires,
/// so requests don't fail due to clock skew or network latency.
const TOKEN_EXPIRY_BUFFER_SECS: i64 = 300;

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_COPILOT_TOKEN_URL: &str = "https://api.github.com/copilot_internal/v2/token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

/// Default Copilot API base URL (used when token response has no endpoints).
pub const DEFAULT_COPILOT_API_BASE: &str = "https://api.individual.githubcopilot.com";

/// Single consolidated keychain key for all auth credentials.
const KEY_AUTH_CREDENTIALS: &str = "auth_credentials";

/// Legacy keychain keys (pre-consolidation) — used only for migration.
const LEGACY_KEY_GITHUB_TOKEN: &str = "github_oauth_token";
const LEGACY_KEY_COPILOT_TOKEN: &str = "copilot_token";
const LEGACY_KEY_COPILOT_EXPIRES: &str = "copilot_token_expires_at";
const LEGACY_KEY_COPILOT_API_BASE: &str = "copilot_api_base";

/// All auth credentials stored as a single keychain entry.
///
/// Serialized to JSON before storage. This is intentionally `Serialize` because
/// the JSON never leaves the keychain — it is stored in and loaded from the OS
/// secure credential store only. The individual token values are never sent to
/// the frontend.
#[derive(Serialize, Deserialize)]
struct StoredCredentials {
    github_token: String,
    copilot_token: String,
    copilot_expires_at: i64,
    copilot_api_base: String,
}

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
    #[error("Copilot token exchange failed (HTTP {status})")]
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
            .header("User-Agent", user_agent())
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            log::error!("Copilot token exchange failed: {status}");
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
            .header("User-Agent", user_agent())
            .send()
            .await?
            .error_for_status()?
            .json::<GitHubUser>()
            .await?;

        Ok(user)
    }

    /// Store auth tokens in the OS keychain as a single consolidated entry.
    pub fn store_tokens(
        github_token: &str,
        copilot_resp: &CopilotTokenResponse,
    ) -> Result<(), AuthError> {
        let api_base = copilot_resp
            .endpoints
            .api
            .as_deref()
            .unwrap_or(DEFAULT_COPILOT_API_BASE);

        let creds = StoredCredentials {
            github_token: github_token.to_string(),
            copilot_token: copilot_resp.token.clone(),
            copilot_expires_at: copilot_resp.expires_at,
            copilot_api_base: api_base.to_string(),
        };

        let json = serde_json::to_string(&creds)
            .map_err(|e| keychain::KeychainError::Store(e.to_string()))?;
        keychain::store(KEY_AUTH_CREDENTIALS, &json)?;
        Ok(())
    }

    /// Load stored credentials from the keychain.
    ///
    /// Transparently migrates from legacy per-key storage on first access.
    fn load_credentials() -> Result<StoredCredentials, AuthError> {
        // Try consolidated key first
        match keychain::retrieve(KEY_AUTH_CREDENTIALS) {
            Ok(json) => {
                let creds: StoredCredentials = serde_json::from_str(&json)
                    .map_err(|e| keychain::KeychainError::Retrieve(e.to_string()))?;
                return Ok(creds);
            }
            Err(keychain::KeychainError::NotFound(_)) => {
                // Fall through to legacy migration
            }
            Err(e) => return Err(AuthError::Keychain(e)),
        }

        // Migrate from legacy individual keys
        let github_token = keychain::retrieve(LEGACY_KEY_GITHUB_TOKEN)?;
        let copilot_token = keychain::retrieve(LEGACY_KEY_COPILOT_TOKEN)?;
        let expires_str = keychain::retrieve(LEGACY_KEY_COPILOT_EXPIRES)?;
        let copilot_expires_at = expires_str.parse::<i64>().unwrap_or_else(|_| {
            log::warn!(
                "Corrupted copilot token expiration in legacy keychain, treating as expired"
            );
            0
        });
        let copilot_api_base = keychain::retrieve(LEGACY_KEY_COPILOT_API_BASE)
            .unwrap_or_else(|_| DEFAULT_COPILOT_API_BASE.to_string());

        let creds = StoredCredentials {
            github_token,
            copilot_token,
            copilot_expires_at,
            copilot_api_base,
        };

        // Store consolidated and clean up legacy keys
        let json = serde_json::to_string(&creds)
            .map_err(|e| keychain::KeychainError::Store(e.to_string()))?;
        keychain::store(KEY_AUTH_CREDENTIALS, &json)?;
        Self::delete_legacy_keys();

        log::info!("Migrated auth credentials from legacy keychain entries to consolidated entry");
        Ok(creds)
    }

    /// Delete legacy individual keychain entries (best-effort, ignores errors).
    fn delete_legacy_keys() {
        for key in [
            LEGACY_KEY_GITHUB_TOKEN,
            LEGACY_KEY_COPILOT_TOKEN,
            LEGACY_KEY_COPILOT_EXPIRES,
            LEGACY_KEY_COPILOT_API_BASE,
        ] {
            let _ = keychain::delete(key);
        }
    }

    /// Load the stored GitHub OAuth token from the keychain.
    pub fn load_github_token() -> Result<String, AuthError> {
        Ok(Self::load_credentials()?.github_token)
    }

    /// Load the stored Copilot API token from the keychain.
    ///
    /// Returns `(token, expires_at_unix, api_base_url)`.
    pub fn load_copilot_token() -> Result<(String, i64, String), AuthError> {
        let creds = Self::load_credentials()?;
        Ok((
            creds.copilot_token,
            creds.copilot_expires_at,
            creds.copilot_api_base,
        ))
    }

    /// Check if the stored Copilot token is still valid
    /// (with [`TOKEN_EXPIRY_BUFFER_SECS`] before actual expiry).
    pub fn is_copilot_token_valid() -> bool {
        match Self::load_copilot_token() {
            Ok((_, expires_at, _)) => {
                let now = chrono::Utc::now().timestamp();
                expires_at > now + TOKEN_EXPIRY_BUFFER_SECS
            }
            Err(_) => false,
        }
    }

    /// Refresh the Copilot token if expired. Returns the current valid token.
    pub async fn ensure_copilot_token(&self) -> Result<(String, String), AuthError> {
        // Single keychain read — check validity and extract token in one pass
        if let Ok((token, expires_at, api_base)) = Self::load_copilot_token() {
            let now = chrono::Utc::now().timestamp();
            if expires_at > now + TOKEN_EXPIRY_BUFFER_SECS {
                return Ok((token, api_base));
            }
        }

        // Token expired or missing — refresh using stored GitHub token
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
        // Delete consolidated entry
        match keychain::delete(KEY_AUTH_CREDENTIALS) {
            Ok(()) | Err(keychain::KeychainError::NotFound(_)) => {}
            Err(e) => return Err(AuthError::Keychain(e)),
        }
        // Also clean up any remaining legacy entries
        Self::delete_legacy_keys();
        Ok(())
    }
}

impl Default for DeviceFlowAuth {
    fn default() -> Self {
        Self::new()
    }
}
