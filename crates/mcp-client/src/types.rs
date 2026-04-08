//! MCP client types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Errors ──────────────────────────────────────────────────────

/// Errors from MCP client operations.
#[derive(Debug, Error)]
pub enum McpClientError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Tool invocation failed: {0}")]
    ToolInvocation(String),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Server not connected: {0}")]
    NotConnected(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

// ── Server configuration ────────────────────────────────────────

/// Transport type for an MCP server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    Http,
    Stdio,
}

impl std::fmt::Display for McpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::Stdio => write!(f, "stdio"),
        }
    }
}

impl std::str::FromStr for McpTransport {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "http" => Ok(Self::Http),
            "stdio" => Ok(Self::Stdio),
            other => Err(format!("unknown transport: {other}")),
        }
    }
}

/// Configuration for an MCP server (mirrors SQLite `mcp_servers` table).
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub transport: McpTransport,
    /// URL for HTTP transport.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Binary path for stdio transport.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
    /// Arguments for stdio transport (JSON array).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,
    /// Optional auth header for HTTP transport.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// Legacy field — kept for DB compatibility. Always false for new entries.
    #[serde(default)]
    pub from_catalog: bool,
    /// Whether this server is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl std::fmt::Debug for McpServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServerConfig")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("transport", &self.transport)
            .field("url", &self.url)
            .field("binary_path", &self.binary_path)
            .field("args", &self.args)
            .field(
                "auth_header",
                &self.auth_header.as_ref().map(|_| "••••••••"),
            )
            .field("from_catalog", &self.from_catalog)
            .field("enabled", &self.enabled)
            .finish()
    }
}

fn default_true() -> bool {
    true
}

// ── Connection status ───────────────────────────────────────────

/// Connection status of an MCP server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl std::fmt::Display for McpServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting"),
            Self::Connected => write!(f, "connected"),
            Self::Error => write!(f, "error"),
        }
    }
}

// ── Tool info ───────────────────────────────────────────────────

/// Information about a tool discovered from an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    /// Tool name (unique within a server).
    pub name: String,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the tool's input parameters.
    pub input_schema: serde_json::Value,
}

/// Result of an MCP tool invocation (serializable for frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolResult {
    /// The text content returned by the tool.
    pub content: Vec<McpToolContent>,
    /// Whether this result represents an error.
    #[serde(default)]
    pub is_error: bool,
}

/// A single content block from a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpToolContent {
    Text { text: String },
    Image { data: String, mime_type: String },
}

// ── Connection info (full state for frontend) ───────────────────

/// Full connection information for an MCP server (sent to frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConnectionInfo {
    /// Server configuration.
    pub config: McpServerConfig,
    /// Current connection status.
    pub status: McpServerStatus,
    /// Error message (if status is Error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Number of tools discovered.
    pub tool_count: usize,
    /// Discovered tools (populated on request).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<McpToolInfo>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_roundtrip() {
        assert_eq!("http".parse::<McpTransport>().unwrap(), McpTransport::Http);
        assert_eq!(
            "stdio".parse::<McpTransport>().unwrap(),
            McpTransport::Stdio
        );
        assert!("invalid".parse::<McpTransport>().is_err());
    }

    #[test]
    fn server_config_serialization() {
        let config = McpServerConfig {
            id: "test".to_string(),
            name: "Test Server".to_string(),
            transport: McpTransport::Http,
            url: Some("https://example.com/mcp".to_string()),
            binary_path: None,
            args: None,
            auth_header: None,
            from_catalog: false,
            enabled: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"transport\":\"http\""));
        assert!(json.contains("\"fromCatalog\":false"));

        let back: McpServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Test Server");
    }

    #[test]
    fn tool_result_serialization() {
        let result = McpToolResult {
            content: vec![McpToolContent::Text {
                text: "hello".to_string(),
            }],
            is_error: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"type\":\"text\""));
    }

    #[test]
    fn connection_info_serialization() {
        let info = McpConnectionInfo {
            config: McpServerConfig {
                id: "s1".to_string(),
                name: "GitHub".to_string(),
                transport: McpTransport::Http,
                url: Some("https://api.github.com/mcp".to_string()),
                binary_path: None,
                args: None,
                auth_header: None,
                from_catalog: true,
                enabled: true,
            },
            status: McpServerStatus::Connected,
            error: None,
            tool_count: 12,
            tools: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"status\":\"connected\""));
        assert!(json.contains("\"toolCount\":12"));
    }
}
