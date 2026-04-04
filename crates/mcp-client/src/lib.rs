//! MCP protocol client library.
//!
//! Provides MCP server connection management, tool discovery, and tool
//! invocation over HTTP and stdio transports. Zero Tauri dependency.

pub mod catalog;
pub mod client;
pub mod registry;
pub mod types;
