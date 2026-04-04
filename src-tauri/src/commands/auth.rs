//! Auth commands: authenticate, logout, get_auth_state.

use crate::state::AppState;
use copilot_api::auth::DeviceFlowAuth;
use copilot_api::types::{AuthState, DeviceCodeResponse, GitHubUser};
use tauri::{AppHandle, Emitter, Manager};

/// Initiate the OAuth device flow. Returns the device code info for the UI.
#[tauri::command]
pub async fn authenticate(app: AppHandle) -> Result<DeviceCodeResponse, String> {
    let state = app.state::<AppState>();
    let auth = state.copilot.auth();

    let device_code = auth
        .request_device_code()
        .await
        .map_err(|e| e.to_string())?;

    Ok(device_code)
}

/// Poll for the OAuth token after the user has entered the device code.
/// Returns the GitHub user on success, or an error string indicating
/// "authorization_pending", "slow_down", "expired", or a fatal error.
#[tauri::command]
pub async fn poll_auth_token(app: AppHandle, device_code: String) -> Result<GitHubUser, String> {
    let state = app.state::<AppState>();
    let auth = state.copilot.auth();

    // Poll once for the GitHub OAuth token
    let token_resp = auth
        .poll_for_token(&device_code)
        .await
        .map_err(|e| e.to_string())?;

    // Exchange for Copilot token
    let copilot_resp = auth
        .exchange_for_copilot_token(&token_resp.access_token)
        .await
        .map_err(|e| e.to_string())?;

    // Store tokens in keychain
    DeviceFlowAuth::store_tokens(&token_resp.access_token, &copilot_resp)
        .map_err(|e| e.to_string())?;

    // Fetch user info
    let user = auth
        .get_github_user(&token_resp.access_token)
        .await
        .map_err(|e| e.to_string())?;

    // Emit auth state change
    let _ = app.emit("auth-state-changed", true);

    Ok(user)
}

/// Sign out: clear all tokens from keychain.
#[tauri::command]
pub async fn logout(app: AppHandle) -> Result<(), String> {
    DeviceFlowAuth::clear_tokens().map_err(|e| e.to_string())?;
    let _ = app.emit("auth-state-changed", false);
    Ok(())
}

/// Check current auth state — are we authenticated?
#[tauri::command]
pub async fn get_auth_state(app: AppHandle) -> Result<AuthState, String> {
    // Check if we have a valid Copilot token
    let has_token = DeviceFlowAuth::is_copilot_token_valid();

    if has_token {
        // Try to load user info
        match DeviceFlowAuth::load_github_token() {
            Ok(github_token) => {
                let state = app.state::<AppState>();
                match state.copilot.auth().get_github_user(&github_token).await {
                    Ok(user) => Ok(AuthState {
                        authenticated: true,
                        user: Some(user),
                    }),
                    Err(_) => {
                        // Token might be valid for Copilot but user fetch failed
                        Ok(AuthState {
                            authenticated: true,
                            user: None,
                        })
                    }
                }
            }
            Err(_) => Ok(AuthState {
                authenticated: false,
                user: None,
            }),
        }
    } else {
        // Try refreshing
        let state = app.state::<AppState>();
        match state.copilot.auth().ensure_copilot_token().await {
            Ok(_) => {
                // Refresh worked — we're authenticated
                match DeviceFlowAuth::load_github_token() {
                    Ok(github_token) => {
                        match state.copilot.auth().get_github_user(&github_token).await {
                            Ok(user) => Ok(AuthState {
                                authenticated: true,
                                user: Some(user),
                            }),
                            Err(_) => Ok(AuthState {
                                authenticated: true,
                                user: None,
                            }),
                        }
                    }
                    Err(_) => Ok(AuthState {
                        authenticated: false,
                        user: None,
                    }),
                }
            }
            Err(_) => Ok(AuthState {
                authenticated: false,
                user: None,
            }),
        }
    }
}
