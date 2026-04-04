//! GitHub Copilot API client library.
//!
//! Provides OAuth device flow authentication, SSE streaming chat completions,
//! model discovery, and secure token storage. Zero Tauri dependency — usable
//! from any Rust project.

pub mod auth;
pub mod client;
pub mod keychain;
pub mod types;

pub use auth::DeviceFlowAuth;
pub use client::CopilotClient;
pub use types::{ChatMessage, ChatRequest, ChatResponse, MessageRole, Model};
