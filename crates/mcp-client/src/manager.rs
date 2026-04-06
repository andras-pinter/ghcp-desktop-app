//! MCP connection manager.
//!
//! Manages multiple MCP server connections concurrently behind an
//! `Arc<RwLock<>>` for thread-safe access from Tauri commands.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::client::McpConnection;
use crate::types::{
    McpClientError, McpConnectionInfo, McpServerConfig, McpToolInfo, McpToolResult,
};

/// Manages the lifecycle of multiple MCP server connections.
#[derive(Clone)]
pub struct McpManager {
    connections: Arc<RwLock<HashMap<String, McpConnection>>>,
}

impl McpManager {
    /// Create a new (empty) manager.
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a server (creates a disconnected entry).
    pub async fn register_server(&self, config: McpServerConfig) {
        let id = config.id.clone();
        let conn = McpConnection::new(config);
        self.connections.write().await.insert(id, conn);
    }

    /// Remove a server (disconnects first if needed).
    pub async fn remove_server(&self, server_id: &str) -> Result<(), McpClientError> {
        let mut map = self.connections.write().await;
        if let Some(mut conn) = map.remove(server_id) {
            conn.disconnect().await;
        }
        Ok(())
    }

    /// Connect to a registered server.
    pub async fn connect_server(&self, server_id: &str) -> Result<(), McpClientError> {
        let mut map = self.connections.write().await;
        let conn = map
            .get_mut(server_id)
            .ok_or_else(|| McpClientError::ServerNotFound(server_id.to_string()))?;
        conn.connect().await
    }

    /// Disconnect from a server.
    pub async fn disconnect_server(&self, server_id: &str) -> Result<(), McpClientError> {
        let mut map = self.connections.write().await;
        if let Some(conn) = map.get_mut(server_id) {
            conn.disconnect().await;
        }
        Ok(())
    }

    /// Test a connection by connecting and immediately disconnecting.
    /// Returns Ok(tool_count) on success, Err on failure.
    pub async fn test_connection(&self, config: &McpServerConfig) -> Result<usize, McpClientError> {
        let mut conn = McpConnection::new(config.clone());
        conn.connect().await?;
        let count = conn.tools().len();
        conn.disconnect().await;
        Ok(count)
    }

    /// Get connection info for all registered servers.
    pub async fn get_all_connections(&self) -> Vec<McpConnectionInfo> {
        let map = self.connections.read().await;
        map.values().map(connection_to_info).collect()
    }

    /// Get connection info for a specific server.
    pub async fn get_connection(
        &self,
        server_id: &str,
    ) -> Result<McpConnectionInfo, McpClientError> {
        let map = self.connections.read().await;
        let conn = map
            .get(server_id)
            .ok_or_else(|| McpClientError::ServerNotFound(server_id.to_string()))?;
        Ok(connection_to_info(conn))
    }

    /// Get tools for a specific server (with full tool details).
    pub async fn get_tools(&self, server_id: &str) -> Result<Vec<McpToolInfo>, McpClientError> {
        let map = self.connections.read().await;
        let conn = map
            .get(server_id)
            .ok_or_else(|| McpClientError::ServerNotFound(server_id.to_string()))?;
        Ok(conn.tools().to_vec())
    }

    /// Invoke a tool on a specific server.
    pub async fn invoke_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<McpToolResult, McpClientError> {
        // Clone the connection reference outside the lock to avoid holding
        // the read guard across the async network call.
        let map = self.connections.read().await;
        let conn = map
            .get(server_id)
            .ok_or_else(|| McpClientError::ServerNotFound(server_id.to_string()))?;
        // call_tool takes &self, and McpConnection is behind the RwLock,
        // so we need to call it while we have the lock. But the lock is
        // only a read lock, so other reads can proceed concurrently.
        // This is acceptable — true per-connection locking would require
        // Arc<RwLock<McpConnection>> per entry.
        conn.call_tool(tool_name, arguments).await
    }

    /// Update the configuration of a registered server (does NOT reconnect).
    pub async fn update_config(
        &self,
        server_id: &str,
        config: McpServerConfig,
    ) -> Result<(), McpClientError> {
        let mut map = self.connections.write().await;
        let conn = map
            .get_mut(server_id)
            .ok_or_else(|| McpClientError::ServerNotFound(server_id.to_string()))?;
        conn.set_config(config);
        Ok(())
    }

    /// Connect all enabled servers (non-blocking; errors logged per-server).
    ///
    /// Connects each server individually, only holding the write lock briefly
    /// for registration to avoid blocking other MCP operations.
    pub async fn connect_enabled_servers(&self, configs: Vec<McpServerConfig>) {
        for config in configs {
            if !config.enabled {
                continue;
            }
            let id = config.id.clone();
            let mut conn = McpConnection::new(config);
            if let Err(e) = conn.connect().await {
                log::warn!("Failed to auto-connect MCP server {id}: {e}");
            }
            self.connections.write().await.insert(id, conn);
        }
    }

    /// Disconnect all servers and clear the map.
    pub async fn shutdown(&self) {
        let mut map = self.connections.write().await;
        for (_, mut conn) in map.drain() {
            conn.disconnect().await;
        }
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a connection to its info representation (no tool details).
fn connection_to_info(conn: &McpConnection) -> McpConnectionInfo {
    McpConnectionInfo {
        config: conn.config().clone(),
        status: conn.status().clone(),
        error: conn.error().map(String::from),
        tool_count: conn.tools().len(),
        tools: None,
    }
}
