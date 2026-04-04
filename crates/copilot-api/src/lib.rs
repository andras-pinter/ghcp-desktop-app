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

/// Build the User-Agent header value used in all API requests.
pub fn user_agent() -> String {
    format!("Chuck/{APP_VERSION} (GitHub Copilot Desktop Client)")
}

pub use auth::{AuthError, DeviceFlowAuth, GITHUB_CLIENT_ID};
pub use client::{ClientError, CopilotClient, StreamEvent};
pub use keychain::KeychainError;
pub use types::{
    AuthState, ChatMessage, ChatRequest, ChatResponse, CopilotTokenResponse, DeviceCodeResponse,
    GitHubUser, MessageRole, Model, ModelsResponse, OAuthTokenResponse, StreamingChatResponse,
    StreamingChoice, StreamingDelta,
};
