//! Chuck — Tauri application setup and plugin registration.

mod commands;
mod db;
mod state;

use state::AppState;
use tauri::Manager;

/// Run the Tauri application.
pub fn run(force_logout: bool) {
    env_logger::init();

    if force_logout {
        log::info!("--logout flag detected, clearing stored tokens");
        let _ = copilot_api::auth::DeviceFlowAuth::clear_tokens();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_state = AppState::new(app.handle())?;

            // Load enabled MCP servers for auto-connect
            let mcp_configs = {
                let conn = app_state.db.lock().map_err(|e| e.to_string())?;
                let rows =
                    db::queries::get_enabled_mcp_servers(&conn).map_err(|e| e.to_string())?;
                rows.iter()
                    .map(|row| mcp_client::McpServerConfig {
                        id: row.id.clone(),
                        name: row.name.clone(),
                        transport: row
                            .transport
                            .parse()
                            .unwrap_or(mcp_client::types::McpTransport::Http),
                        url: row.url.clone(),
                        binary_path: row.binary_path.clone(),
                        args: row.args.clone(),
                        auth_header: row.auth_header.clone(),
                        from_catalog: row.from_catalog,
                        enabled: row.enabled,
                    })
                    .collect::<Vec<_>>()
            };

            let mcp_manager = app_state.mcp.clone();
            app.manage(app_state);

            // Auto-connect enabled MCP servers in background (non-blocking)
            if !mcp_configs.is_empty() {
                tauri::async_runtime::spawn(async move {
                    log::info!(
                        "Auto-connecting {} enabled MCP server(s)...",
                        mcp_configs.len()
                    );
                    mcp_manager.connect_enabled_servers(mcp_configs).await;
                });
            }

            log::info!("Chuck initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::log_frontend,
            commands::auth::authenticate,
            commands::auth::poll_auth_token,
            commands::auth::logout,
            commands::auth::get_auth_state,
            commands::chat::send_message,
            commands::chat::stop_streaming,
            commands::models::get_models,
            commands::conversations::get_conversations,
            commands::conversations::get_conversation,
            commands::conversations::create_conversation,
            commands::conversations::update_conversation,
            commands::conversations::delete_conversation,
            commands::conversations::get_messages,
            commands::conversations::create_message,
            commands::conversations::update_message_content,
            commands::conversations::delete_messages_after,
            commands::settings::get_setting,
            commands::settings::update_setting,
            commands::settings::get_db_size,
            commands::settings::save_draft,
            commands::settings::get_draft,
            commands::settings::delete_draft,
            commands::web_research::web_search,
            commands::web_research::fetch_url,
            commands::mcp::get_mcp_servers,
            commands::mcp::add_mcp_server,
            commands::mcp::update_mcp_server,
            commands::mcp::remove_mcp_server,
            commands::mcp::connect_mcp_server,
            commands::mcp::disconnect_mcp_server,
            commands::mcp::test_mcp_connection,
            commands::mcp::get_mcp_tools,
            commands::mcp::invoke_mcp_tool,
            commands::mcp::get_mcp_catalog,
            commands::mcp::fetch_mcp_registry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Chuck");
}
