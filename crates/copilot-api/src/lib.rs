//! GitHub Copilot API client library.
//!
//! Provides OAuth device flow authentication, SSE streaming chat completions,
//! model discovery, and secure token storage. Zero Tauri dependency — usable
//! from any Rust project.

pub mod auth;
pub mod client;
pub mod keychain;
pub mod types;

/// Application version derived from Cargo.toml at compile time.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Editor version sent in API requests.
/// The Copilot API validates this header against known editors.
/// Since we authenticate using the VS Code OAuth client ID, we must send
/// a compatible editor identifier.
const EDITOR_VERSION: &str = "vscode/1.100.0";
const EDITOR_PLUGIN_VERSION: &str = "copilot-chat/0.26.0";

/// Build the User-Agent header value used in all API requests.
pub fn user_agent() -> String {
    format!("Chuck/{APP_VERSION} (GitHub Copilot Desktop Client)")
}

/// Editor version header value.
pub fn editor_version() -> &'static str {
    EDITOR_VERSION
}

/// Editor plugin version header value.
pub fn editor_plugin_version() -> &'static str {
    EDITOR_PLUGIN_VERSION
}

pub use auth::{AuthError, DeviceFlowAuth, GITHUB_CLIENT_ID};
pub use client::{ClientError, CopilotClient, StreamEvent};
pub use keychain::KeychainError;
pub use types::{
    AuthState, ChatMessage, ChatRequest, ChatResponse, CopilotTokenResponse, DeviceCodeInfo,
    DeviceCodeResponse, GitHubUser, MessageRole, Model, ModelsResponse, OAuthTokenResponse,
    StreamingChatResponse, StreamingChoice, StreamingDelta,
};
