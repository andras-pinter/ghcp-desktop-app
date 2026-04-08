//! Chuck — Tauri application setup and plugin registration.

mod commands;
mod db;
pub mod registry;
pub mod skillmd;
mod state;
pub mod text_extract;

use state::AppState;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
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

            // ── System tray ──────────────────────────────────────
            let new_chat = MenuItemBuilder::with_id("new_chat", "New Chat").build(app)?;
            let show = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .item(&new_chat)
                .item(&show)
                .item(&quit)
                .build()?;

            let icon = app
                .default_window_icon()
                .cloned()
                .expect("default window icon must be set");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(false)
                .tooltip("Chuck")
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "new_chat" => {
                        show_and_focus(app);
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.emit("tray-new-chat", ());
                        }
                    }
                    "show" => {
                        show_and_focus(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        show_and_focus(tray.app_handle());
                    }
                })
                .build(app)?;

            // Intercept window close → hide to tray instead of quitting
            if let Some(win) = app.get_webview_window("main") {
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
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
            commands::chat::generate_title,
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
            commands::settings::delete_old_conversations,
            commands::settings::export_conversation_json,
            commands::settings::export_conversation_markdown,
            commands::settings::export_all_conversations_json,
            commands::settings::export_all_conversations_markdown,
            commands::settings::save_export_file,
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
            commands::mcp::fetch_mcp_registry,
            commands::agents::get_agents,
            commands::agents::get_agent,
            commands::agents::create_agent,
            commands::agents::update_agent,
            commands::agents::delete_agent,
            commands::agents::set_agent_skills,
            commands::agents::set_agent_mcp_connections,
            commands::agents::install_agent_from_registry,
            commands::agents::import_agent_from_git,
            commands::agents::fetch_git_agents,
            commands::skills::get_skills,
            commands::skills::create_skill,
            commands::skills::update_skill,
            commands::skills::delete_skill,
            commands::skills::toggle_skill,
            commands::skills::search_registry,
            commands::skills::install_from_registry,
            commands::skills::fetch_git_skills,
            commands::skills::import_git_skill,
            commands::projects::get_projects,
            commands::projects::get_project,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::projects::get_project_files,
            commands::projects::add_project_file,
            commands::projects::get_project_file_content,
            commands::projects::remove_project_file,
            commands::projects::get_project_conversations,
            commands::projects::pick_file_for_upload,
            commands::projects::pick_file_for_chat,
            commands::projects::extract_file_text,
            commands::projects::read_dropped_files,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Chuck")
        .run(|app_handle, event| {
            // macOS: reopen the main window when the dock icon is clicked
            if let tauri::RunEvent::Reopen {
                has_visible_windows,
                ..
            } = event
            {
                if !has_visible_windows {
                    show_and_focus(app_handle);
                }
            }
        });
}

/// Show the main window and activate the app (macOS needs app-level activation).
fn show_and_focus(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.show();
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}
