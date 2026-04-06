//! MCP server connection management.
//!
//! Wraps the `rmcp` SDK to provide connection lifecycle, tool discovery,
//! and tool invocation for a single MCP server.

use std::net::IpAddr;

use rmcp::model::{CallToolRequestParams, Tool};
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::child_process::TokioChildProcess;
#[cfg(feature = "http")]
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::ServiceExt;
use tokio::process::Command;
use url::Url;

use crate::types::{
    McpClientError, McpServerConfig, McpServerStatus, McpToolContent, McpToolInfo, McpToolResult,
    McpTransport,
};

/// Maximum total size of MCP tool response content (1 MB).
const MAX_RESPONSE_SIZE: usize = 1_000_000;

/// Allowed image MIME types for MCP tool responses.
const ALLOWED_IMAGE_MIMES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/svg+xml",
];

/// A live connection to a single MCP server.
pub struct McpConnection {
    config: McpServerConfig,
    status: McpServerStatus,
    error: Option<String>,
    client: Option<RunningService<RoleClient, ()>>,
    tools: Vec<McpToolInfo>,
}

impl McpConnection {
    /// Create a new (disconnected) connection for the given server config.
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            status: McpServerStatus::Disconnected,
            error: None,
            client: None,
            tools: Vec::new(),
        }
    }

    /// Establish a connection to the MCP server.
    pub async fn connect(&mut self) -> Result<(), McpClientError> {
        self.status = McpServerStatus::Connecting;
        self.error = None;

        match self.config.transport {
            McpTransport::Stdio => self.connect_stdio().await,
            McpTransport::Http => self.connect_http().await,
        }
    }

    /// Connect via stdio transport (launches a child process).
    ///
    /// # Security
    /// - Binary path must be absolute
    /// - Binary must exist on disk
    /// - Args JSON must be a valid string array if provided
    async fn connect_stdio(&mut self) -> Result<(), McpClientError> {
        let binary = self.config.binary_path.as_deref().ok_or_else(|| {
            McpClientError::InvalidConfig("stdio transport requires binary_path".to_string())
        })?;

        // Validate binary path is absolute
        let binary_path = std::path::Path::new(binary);
        if !binary_path.is_absolute() {
            let msg = "binary_path must be an absolute path".to_string();
            self.status = McpServerStatus::Error;
            self.error = Some(msg.clone());
            return Err(McpClientError::InvalidConfig(msg));
        }

        // Validate binary exists
        if !binary_path.exists() {
            let msg = format!("binary not found: {binary}");
            self.status = McpServerStatus::Error;
            self.error = Some(msg.clone());
            return Err(McpClientError::InvalidConfig(msg));
        }

        log::info!(
            "Launching stdio MCP server '{}' (binary: {})",
            self.config.name,
            binary
        );

        let mut cmd = Command::new(binary);
        if let Some(args_json) = &self.config.args {
            let args = serde_json::from_str::<Vec<String>>(args_json).map_err(|e| {
                let msg = format!("invalid args JSON (must be string array): {e}");
                self.status = McpServerStatus::Error;
                self.error = Some(msg.clone());
                McpClientError::InvalidConfig(msg)
            })?;
            cmd.args(&args);
        }

        let transport = TokioChildProcess::new(cmd).map_err(|e| {
            log::error!("Stdio transport error for '{}': {e}", self.config.id);
            let msg = "Failed to spawn MCP server process".to_string();
            self.status = McpServerStatus::Error;
            self.error = Some(msg.clone());
            McpClientError::Transport(msg)
        })?;

        let client = ().serve(transport).await.map_err(|e| {
            log::error!("MCP handshake error for '{}': {e}", self.config.id);
            let msg = "MCP handshake failed".to_string();
            self.status = McpServerStatus::Error;
            self.error = Some(msg.clone());
            McpClientError::Connection(msg)
        })?;

        self.client = Some(client);
        self.discover_tools().await?;
        self.status = McpServerStatus::Connected;
        Ok(())
    }

    /// Connect via HTTP transport.
    ///
    /// # Security
    /// - Only HTTPS URLs allowed (HTTP only for localhost)
    /// - Private/internal IP ranges blocked (SSRF protection)
    async fn connect_http(&mut self) -> Result<(), McpClientError> {
        let url_str = self.config.url.as_deref().ok_or_else(|| {
            McpClientError::InvalidConfig("HTTP transport requires url".to_string())
        })?;

        // Parse and validate URL
        let parsed = Url::parse(url_str).map_err(|e| {
            let msg = format!("Invalid URL: {e}");
            self.status = McpServerStatus::Error;
            self.error = Some(msg.clone());
            McpClientError::InvalidConfig(msg)
        })?;

        // Only allow http/https schemes
        match parsed.scheme() {
            "https" => {}
            "http" => {
                let host = parsed.host_str().unwrap_or("");
                if !matches!(host, "localhost" | "127.0.0.1" | "[::1]") {
                    let msg = "HTTP (non-TLS) only allowed for localhost; use HTTPS".to_string();
                    self.status = McpServerStatus::Error;
                    self.error = Some(msg.clone());
                    return Err(McpClientError::InvalidConfig(msg));
                }
            }
            other => {
                let msg = format!("Unsupported URL scheme: {other}; only HTTPS allowed");
                self.status = McpServerStatus::Error;
                self.error = Some(msg.clone());
                return Err(McpClientError::InvalidConfig(msg));
            }
        }

        // Block private/internal IP ranges (SSRF protection)
        if let Some(host) = parsed.host_str() {
            if let Ok(ip) = host.parse::<IpAddr>() {
                if is_private_ip(&ip) {
                    let msg = "Cannot connect to private/internal IP addresses".to_string();
                    self.status = McpServerStatus::Error;
                    self.error = Some(msg.clone());
                    return Err(McpClientError::InvalidConfig(msg));
                }
            }
        }

        #[cfg(feature = "http")]
        {
            let transport = StreamableHttpClientTransport::from_uri(url_str);

            let client: RunningService<RoleClient, ()> = ().serve(transport).await.map_err(|e| {
                log::error!("HTTP connection error for '{}': {e}", self.config.id);
                let msg = "MCP HTTP connection failed".to_string();
                self.status = McpServerStatus::Error;
                self.error = Some(msg.clone());
                McpClientError::Connection(msg)
            })?;

            self.client = Some(client);
            self.discover_tools().await?;
            self.status = McpServerStatus::Connected;
            Ok(())
        }

        #[cfg(not(feature = "http"))]
        {
            let _ = url_str;
            let msg = "HTTP transport not enabled in this build".to_string();
            self.status = McpServerStatus::Error;
            self.error = Some(msg.clone());
            Err(McpClientError::Transport(msg))
        }
    }

    /// Discover tools from the connected server and cache them.
    async fn discover_tools(&mut self) -> Result<(), McpClientError> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| McpClientError::NotConnected(self.config.id.clone()))?;

        let tools: Vec<Tool> = client
            .list_all_tools()
            .await
            .map_err(|e| McpClientError::ToolInvocation(format!("failed to list tools: {e}")))?;

        self.tools = tools.into_iter().map(convert_tool).collect();
        Ok(())
    }

    /// Disconnect from the server and clean up resources.
    pub async fn disconnect(&mut self) {
        if let Some(client) = self.client.take() {
            let _ = client.cancel().await;
        }
        self.tools.clear();
        self.status = McpServerStatus::Disconnected;
        self.error = None;
    }

    /// List discovered tools.
    pub fn tools(&self) -> &[McpToolInfo] {
        &self.tools
    }

    /// Get the current connection status.
    pub fn status(&self) -> &McpServerStatus {
        &self.status
    }

    /// Get the error message (if any).
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Get the server configuration.
    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Update the server configuration (does not reconnect).
    pub fn set_config(&mut self, config: McpServerConfig) {
        self.config = config;
    }

    /// Invoke a tool by name with the given arguments.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<McpToolResult, McpClientError> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| McpClientError::NotConnected(self.config.id.clone()))?;

        let mut params = CallToolRequestParams::new(tool_name.to_string());
        if let Some(serde_json::Value::Object(map)) = arguments {
            params = params.with_arguments(map);
        }

        let result = client
            .call_tool(params)
            .await
            .map_err(|e| McpClientError::ToolInvocation(format!("tool call failed: {e}")))?;

        Ok(convert_call_result(result))
    }
}

// ── Conversion helpers ──────────────────────────────────────────

/// Convert an rmcp `Tool` to our `McpToolInfo`.
fn convert_tool(tool: Tool) -> McpToolInfo {
    McpToolInfo {
        name: tool.name.to_string(),
        description: tool.description.map(|d| d.to_string()),
        input_schema: serde_json::to_value(&tool.input_schema).unwrap_or_default(),
    }
}

/// Convert an rmcp `CallToolResult` to our `McpToolResult`.
///
/// Enforces a 1 MB total payload size limit and validates image MIME types.
fn convert_call_result(result: rmcp::model::CallToolResult) -> McpToolResult {
    let mut total_size: usize = 0;
    let content = result
        .content
        .into_iter()
        .filter_map(|c| {
            use rmcp::model::RawContent;
            match c.raw {
                RawContent::Text(t) => {
                    total_size = total_size.saturating_add(t.text.len());
                    if total_size > MAX_RESPONSE_SIZE {
                        log::warn!("MCP tool response exceeded size limit, truncating");
                        return None;
                    }
                    Some(McpToolContent::Text { text: t.text })
                }
                RawContent::Image(img) => {
                    // Validate MIME type
                    if !ALLOWED_IMAGE_MIMES.contains(&img.mime_type.as_str()) {
                        log::warn!(
                            "MCP tool returned disallowed image MIME type: {}",
                            img.mime_type
                        );
                        return None;
                    }
                    total_size = total_size.saturating_add(img.data.len());
                    if total_size > MAX_RESPONSE_SIZE {
                        log::warn!("MCP tool response exceeded size limit, truncating");
                        return None;
                    }
                    Some(McpToolContent::Image {
                        data: img.data,
                        mime_type: img.mime_type,
                    })
                }
                _ => None,
            }
        })
        .collect();

    McpToolResult {
        content,
        is_error: result.is_error.unwrap_or(false),
    }
}

// ── Security helpers ────────────────────────────────────────────

/// Check if an IP address is private, loopback, link-local, or a cloud metadata endpoint.
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()            // 127.0.0.0/8
                || v4.is_private()       // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                || v4.is_link_local()    // 169.254.0.0/16
                || v4.is_broadcast()     // 255.255.255.255
                || v4.is_unspecified()   // 0.0.0.0
                || v4.octets() == [169, 254, 169, 254] // cloud metadata
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_ip_detection() {
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"169.254.169.254".parse().unwrap()));
        assert!(is_private_ip(&"169.254.1.1".parse().unwrap()));
        assert!(is_private_ip(&"0.0.0.0".parse().unwrap()));
        assert!(is_private_ip(&"::1".parse().unwrap()));

        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse().unwrap()));
        assert!(!is_private_ip(&"203.0.113.1".parse().unwrap()));
    }
}
