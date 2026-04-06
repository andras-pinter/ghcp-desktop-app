//! MCP protocol client library.
//!
//! Provides MCP server connection management, tool discovery, and tool
//! invocation over HTTP and stdio transports. Zero Tauri dependency.
//!
//! Built on top of the official `rmcp` SDK.

pub mod client;
pub mod manager;
pub mod registry;
pub mod types;

pub use client::McpConnection;
pub use manager::McpManager;
pub use registry::{fetch_registry, RegistryPackage, RegistryPage, RegistryRemote, RegistryServer};
pub use types::{
    McpClientError, McpConnectionInfo, McpServerConfig, McpServerStatus, McpToolInfo, McpToolResult,
};
