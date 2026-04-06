//! MCP commands: server management + tool invocation.

use mcp_client::types::{McpConnectionInfo, McpServerConfig, McpServerStatus, McpTransport};
use mcp_client::{McpToolInfo, McpToolResult};
use tauri::State;

use crate::db::queries::{self, McpServerRow};
use crate::state::AppState;

// ── Helpers ─────────────────────────────────────────────────────

/// Convert a DB row to an `McpServerConfig`.
fn row_to_config(row: &McpServerRow) -> McpServerConfig {
    McpServerConfig {
        id: row.id.clone(),
        name: row.name.clone(),
        transport: row.transport.parse::<McpTransport>().unwrap_or_else(|_| {
            log::warn!(
                "Invalid transport '{}' for MCP server '{}', defaulting to HTTP",
                row.transport,
                row.id
            );
            McpTransport::Http
        }),
        url: row.url.clone(),
        binary_path: row.binary_path.clone(),
        args: row.args.clone(),
        auth_header: row.auth_header.clone(),
        from_catalog: row.from_catalog,
        enabled: row.enabled,
    }
}

/// Convert an `McpServerConfig` to a DB row (with timestamps).
fn config_to_row(config: &McpServerConfig, now: &str) -> McpServerRow {
    McpServerRow {
        id: config.id.clone(),
        name: config.name.clone(),
        transport: config.transport.to_string(),
        url: config.url.clone(),
        binary_path: config.binary_path.clone(),
        args: config.args.clone(),
        auth_header: config.auth_header.clone(),
        from_catalog: config.from_catalog,
        enabled: config.enabled,
        created_at: now.to_string(),
        updated_at: now.to_string(),
    }
}

/// ISO 8601 timestamp helper.
fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Validate an `McpServerConfig` before persisting to DB.
///
/// The connection logic (`connect_stdio`, `connect_http`) performs its own
/// validation at connect-time, but we must also reject obviously invalid
/// configs before they reach the database — a compromised webview could
/// bypass the frontend form and send arbitrary IPC payloads.
fn validate_config(config: &McpServerConfig) -> Result<(), String> {
    if config.name.trim().is_empty() {
        return Err("Server name is required".to_string());
    }

    match config.transport {
        McpTransport::Http => {
            let url_str = config.url.as_deref().unwrap_or("");
            if url_str.is_empty() {
                return Err("HTTP transport requires a URL".to_string());
            }
            let parsed = tauri::Url::parse(url_str).map_err(|e| format!("Invalid URL: {e}"))?;
            match parsed.scheme() {
                "https" => {}
                "http" => {
                    let host = parsed.host_str().unwrap_or("");
                    if !matches!(host, "localhost" | "127.0.0.1" | "::1") {
                        return Err(
                            "HTTP (non-TLS) only allowed for localhost; use HTTPS".to_string()
                        );
                    }
                }
                other => return Err(format!("Unsupported URL scheme: {other}")),
            }
        }
        McpTransport::Stdio => {
            let binary = config.binary_path.as_deref().unwrap_or("");
            if binary.is_empty() {
                return Err("Stdio transport requires a binary path".to_string());
            }
            // Block path traversal components
            if binary.contains("..") {
                return Err("Binary path must not contain '..' components".to_string());
            }
            // If it looks like a path (contains separator), must be absolute
            if (binary.contains('/') || binary.contains('\\'))
                && !std::path::Path::new(binary).is_absolute()
            {
                return Err("Binary path must be absolute or a bare command name".to_string());
            }
            // Validate args JSON if present
            if let Some(args) = &config.args {
                serde_json::from_str::<Vec<String>>(args)
                    .map_err(|e| format!("Invalid args (must be JSON string array): {e}"))?;
            }
        }
    }

    Ok(())
}

/// Redact sensitive fields from a config before sending to the frontend.
fn redact_config(config: McpServerConfig) -> McpServerConfig {
    McpServerConfig {
        auth_header: config.auth_header.as_ref().map(|_| "••••••••".to_string()),
        ..config
    }
}

// ── Commands ────────────────────────────────────────────────────

/// List all configured MCP servers with their connection status.
#[tauri::command]
pub async fn get_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpConnectionInfo>, String> {
    // Get servers from DB
    let db_servers = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        queries::get_mcp_servers(&conn).map_err(|e| e.to_string())?
    };

    // Get live connection status from manager
    let live = state.mcp.get_all_connections().await;

    // Merge: DB is source of truth for config, manager for status
    let mut result = Vec::new();
    for row in &db_servers {
        let config = redact_config(row_to_config(row));
        if let Some(live_info) = live.iter().find(|c| c.config.id == row.id) {
            result.push(McpConnectionInfo {
                config,
                status: live_info.status.clone(),
                error: live_info.error.clone(),
                tool_count: live_info.tool_count,
                tools: None,
            });
        } else {
            result.push(McpConnectionInfo {
                config,
                status: McpServerStatus::Disconnected,
                error: None,
                tool_count: 0,
                tools: None,
            });
        }
    }

    Ok(result)
}

/// Add a new MCP server.
#[tauri::command]
pub async fn add_mcp_server(
    state: State<'_, AppState>,
    config: McpServerConfig,
) -> Result<McpConnectionInfo, String> {
    validate_config(&config)?;

    let now = now_iso();
    let row = config_to_row(&config, &now);

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        queries::insert_mcp_server(&conn, &row).map_err(|e| e.to_string())?;
    }

    // Register in manager (disconnected)
    state.mcp.register_server(config.clone()).await;

    Ok(McpConnectionInfo {
        config: redact_config(config),
        status: McpServerStatus::Disconnected,
        error: None,
        tool_count: 0,
        tools: None,
    })
}

/// Update an existing MCP server configuration.
#[tauri::command]
pub async fn update_mcp_server(
    state: State<'_, AppState>,
    config: McpServerConfig,
) -> Result<(), String> {
    validate_config(&config)?;

    let now = now_iso();
    let mut row = config_to_row(&config, &now);

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        // Preserve original created_at
        if let Some(existing) =
            queries::get_mcp_server(&conn, &config.id).map_err(|e| e.to_string())?
        {
            row.created_at = existing.created_at;
        }
        queries::update_mcp_server(&conn, &row).map_err(|e| e.to_string())?;
    }

    // Update in manager
    let id = config.id.clone();
    let _ = state.mcp.update_config(&id, config).await;

    Ok(())
}

/// Remove an MCP server.
#[tauri::command]
pub async fn remove_mcp_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<(), String> {
    // Remove from DB first — if this fails, manager state is untouched
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        queries::delete_mcp_server(&conn, &server_id).map_err(|e| e.to_string())?;
    }

    // Disconnect + remove from manager
    let _ = state.mcp.remove_server(&server_id).await;

    Ok(())
}

/// Connect to an MCP server.
#[tauri::command]
pub async fn connect_mcp_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<McpConnectionInfo, String> {
    // Ensure server is registered in manager
    let config = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let row = queries::get_mcp_server(&conn, &server_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Server not found: {server_id}"))?;
        row_to_config(&row)
    };
    state.mcp.register_server(config).await;

    state
        .mcp
        .connect_server(&server_id)
        .await
        .map_err(|e| e.to_string())?;

    state
        .mcp
        .get_connection(&server_id)
        .await
        .map_err(|e| e.to_string())
}

/// Disconnect from an MCP server.
#[tauri::command]
pub async fn disconnect_mcp_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<(), String> {
    state
        .mcp
        .disconnect_server(&server_id)
        .await
        .map_err(|e| e.to_string())
}

/// Test an MCP connection (connect + list tools + disconnect).
#[tauri::command]
pub async fn test_mcp_connection(
    state: State<'_, AppState>,
    config: McpServerConfig,
) -> Result<usize, String> {
    validate_config(&config)?;
    state
        .mcp
        .test_connection(&config)
        .await
        .map_err(|e| e.to_string())
}

/// Get tools discovered from a connected MCP server.
#[tauri::command]
pub async fn get_mcp_tools(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<Vec<McpToolInfo>, String> {
    state
        .mcp
        .get_tools(&server_id)
        .await
        .map_err(|e| e.to_string())
}

/// Invoke an MCP tool on a connected server.
#[tauri::command]
pub async fn invoke_mcp_tool(
    state: State<'_, AppState>,
    server_id: String,
    tool_name: String,
    arguments: Option<serde_json::Value>,
) -> Result<McpToolResult, String> {
    state
        .mcp
        .invoke_tool(&server_id, &tool_name, arguments)
        .await
        .map_err(|e| e.to_string())
}

/// Fetch MCP servers from the official MCP Registry.
///
/// If `query` is provided, performs a server-side search by name.
/// If `cursor` is provided, fetches the next page.
/// Returns a page of results with an optional cursor for the next page.
#[tauri::command]
pub async fn fetch_mcp_registry(
    query: Option<String>,
    cursor: Option<String>,
) -> Result<mcp_client::RegistryPage, String> {
    mcp_client::fetch_registry(query.as_deref(), cursor.as_deref())
        .await
        .map_err(|e| e.to_string())
}
