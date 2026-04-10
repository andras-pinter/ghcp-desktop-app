//! MCP commands: server management + tool invocation.

use std::path::{Component, Path};

use mcp_client::types::{McpConnectionInfo, McpServerConfig, McpServerStatus, McpTransport};
use mcp_client::{McpToolInfo, McpToolResult};
use tauri::State;

use crate::db::queries::{self, McpServerRow};
use crate::state::AppState;

// ── Keychain helpers for MCP auth ───────────────────────────────

/// Keychain key for an MCP server's auth header.
fn mcp_auth_key(server_id: &str) -> String {
    format!("mcp_auth_{server_id}")
}

/// Store an MCP auth header in the OS keychain.
fn store_mcp_auth(server_id: &str, auth_header: &str) {
    if let Err(e) = copilot_api::keychain::store(&mcp_auth_key(server_id), auth_header) {
        log::warn!("Failed to store MCP auth for {server_id}: {e}");
    }
}

/// Retrieve an MCP auth header from the OS keychain.
fn retrieve_mcp_auth(server_id: &str) -> Option<String> {
    copilot_api::keychain::retrieve(&mcp_auth_key(server_id)).ok()
}

/// Delete an MCP auth header from the OS keychain.
fn delete_mcp_auth(server_id: &str) {
    if let Err(e) = copilot_api::keychain::delete(&mcp_auth_key(server_id)) {
        log::warn!("Failed to delete MCP auth for {server_id}: {e}");
    }
}

// ── Helpers ─────────────────────────────────────────────────────

/// Convert a DB row to an `McpServerConfig`, loading auth from keychain.
pub(crate) fn row_to_config(row: &McpServerRow) -> McpServerConfig {
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
        // Load auth from keychain, falling back to DB for migration
        auth_header: retrieve_mcp_auth(&row.id).or_else(|| row.auth_header.clone()),
        from_catalog: row.from_catalog,
        enabled: row.enabled,
    }
}

/// Check if a stdio server's binary has been approved by the user.
///
/// Returns `Ok(())` if the server is HTTP or if the binary is approved.
/// Returns `Err("BINARY_NOT_APPROVED:{path}")` if approval is required.
fn check_binary_approval(state: &AppState, config: &McpServerConfig) -> Result<(), String> {
    if config.transport == McpTransport::Stdio {
        if let Some(binary) = &config.binary_path {
            let approved = {
                let conn = state.db.lock().map_err(|e| e.to_string())?;
                queries::is_binary_approved(&conn, binary).map_err(|e| e.to_string())?
            };
            if !approved {
                return Err(format!("BINARY_NOT_APPROVED:{binary}"));
            }
        }
    }
    Ok(())
}

/// Convert an `McpServerConfig` to a DB row (with timestamps).
/// Auth header is stored in the keychain, not in the DB row.
fn config_to_row(config: &McpServerConfig, now: &str) -> McpServerRow {
    McpServerRow {
        id: config.id.clone(),
        name: config.name.clone(),
        transport: config.transport.to_string(),
        url: config.url.clone(),
        binary_path: config.binary_path.clone(),
        args: config.args.clone(),
        auth_header: None, // Never store auth in SQLite
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
    // Validate server_id format — used as keychain key suffix and DB primary key.
    // Reject anything that could cause path traversal in debug keychain or key collisions.
    if config.id.is_empty() || config.id.len() > 128 {
        return Err("Server ID must be 1-128 characters".to_string());
    }
    if !config
        .id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Server ID must contain only alphanumeric characters, hyphens, and underscores"
                .to_string(),
        );
    }

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
            // Block path traversal components using proper path parsing
            if Path::new(binary)
                .components()
                .any(|c| matches!(c, Component::ParentDir))
            {
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

    // Store auth header in keychain (not SQLite)
    if let Some(auth) = &config.auth_header {
        store_mcp_auth(&config.id, auth);
    }

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

    // Sentinel value used when frontend returns the redacted placeholder.
    const REDACTED: &str = "••••••••";

    // Update auth header in keychain — but skip if it's the redacted sentinel
    // (the user didn't change the auth, so we keep the existing keychain value).
    if let Some(auth) = &config.auth_header {
        if auth != REDACTED {
            store_mcp_auth(&config.id, auth);
        }
        // else: unchanged redacted value → keep existing keychain entry
    } else {
        delete_mcp_auth(&config.id);
    }

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
    state
        .mcp
        .update_config(&id, config)
        .await
        .map_err(|e| e.to_string())?;

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

    // Clean up auth from keychain
    delete_mcp_auth(&server_id);

    // Disconnect + remove from manager
    let _ = state.mcp.remove_server(&server_id).await;

    Ok(())
}

/// Redact sensitive fields from connection info before returning to the frontend.
fn redact_connection_info(mut info: McpConnectionInfo) -> McpConnectionInfo {
    info.config = redact_config(info.config);
    info
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

    // Re-validate config loaded from DB (guards against corruption or legacy data)
    validate_config(&config)?;

    // Enforce stdio binary approval before connecting
    check_binary_approval(&state, &config)?;

    state.mcp.register_server(config).await;

    state
        .mcp
        .connect_server(&server_id)
        .await
        .map_err(|e| e.to_string())?;

    let info = state
        .mcp
        .get_connection(&server_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(redact_connection_info(info))
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
///
/// Accepts a `server_id` and loads the real config from DB + keychain
/// (so it uses the actual auth header, not a redacted placeholder).
#[tauri::command]
pub async fn test_mcp_connection(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<usize, String> {
    // Load real config from DB + keychain (same as connect_mcp_server)
    let config = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let row = queries::get_mcp_server(&conn, &server_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Server not found: {server_id}"))?;
        row_to_config(&row)
    };

    validate_config(&config)?;

    // Enforce stdio binary approval before testing (same as connect)
    check_binary_approval(&state, &config)?;

    state
        .mcp
        .test_connection(&config)
        .await
        .map_err(|e| e.to_string())
}

/// Test an MCP server connection using a raw config (for unsaved servers).
///
/// Used by the "Add Server" form where the server hasn't been persisted to DB yet.
/// The auth_header passed here is the real value (not redacted), since the user
/// just typed it in the form.
#[tauri::command]
pub async fn test_mcp_connection_config(
    state: State<'_, AppState>,
    config: McpServerConfig,
) -> Result<usize, String> {
    validate_config(&config)?;
    check_binary_approval(&state, &config)?;

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

/// Approve an MCP stdio binary for execution.
///
/// Shows a native OS confirmation dialog (cannot be forged by the webview)
/// before persisting approval. This is the security gate for MCP stdio.
#[tauri::command]
pub async fn approve_mcp_binary(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    binary_path: String,
) -> Result<(), String> {
    // Validate the path (same rules as stdio config)
    if binary_path.trim().is_empty() {
        return Err("Binary path is required".to_string());
    }
    if Path::new(&binary_path)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err("Binary path must not contain '..' components".to_string());
    }
    if (binary_path.contains('/') || binary_path.contains('\\'))
        && !std::path::Path::new(&binary_path).is_absolute()
    {
        return Err("Binary path must be absolute or a bare command name".to_string());
    }

    // Native OS confirmation dialog — cannot be spoofed by webview JS
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    let confirmed = app
        .dialog()
        .message(format!(
            "Allow MCP server to run \"{}\"?\n\nThis binary will be executed on your system. Only approve binaries you trust.",
            binary_path
        ))
        .title("MCP Binary Approval")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Allow".into(),
            "Deny".into(),
        ))
        .blocking_show();

    if !confirmed {
        return Err("User denied binary approval".to_string());
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::approve_binary(&conn, &binary_path).map_err(|e| e.to_string())
}

/// Check if an MCP stdio binary is approved.
#[tauri::command]
pub fn is_mcp_binary_approved(
    state: State<'_, AppState>,
    binary_path: String,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::is_binary_approved(&conn, &binary_path).map_err(|e| e.to_string())
}
