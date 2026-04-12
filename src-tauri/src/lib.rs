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
use tauri_plugin_store::StoreExt;

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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_state = AppState::new(app.handle())?;

            // Load enabled MCP servers for auto-connect (with keychain auth + binary approval)
            let mcp_configs = {
                let conn = app_state.db.lock().map_err(|e| e.to_string())?;
                let rows =
                    db::queries::get_enabled_mcp_servers(&conn).map_err(|e| e.to_string())?;
                rows.iter()
                    .filter_map(|row| {
                        let config = commands::mcp::row_to_config(row);
                        // Skip unapproved stdio servers
                        if config.transport == mcp_client::types::McpTransport::Stdio {
                            if let Some(ref binary) = config.binary_path {
                                match db::queries::is_binary_approved(&conn, binary) {
                                    Ok(true) => {}
                                    Ok(false) => {
                                        log::warn!(
                                            "Skipping unapproved stdio MCP server '{}' (binary: {})",
                                            config.id,
                                            binary
                                        );
                                        return None;
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to check binary approval for '{}': {e}",
                                            config.id
                                        );
                                        return None;
                                    }
                                }
                            }
                        }
                        Some(config)
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

            let icon = match app.default_window_icon().cloned() {
                Some(icon) => icon,
                None => {
                    log::warn!("No default window icon set — tray icon will be empty");
                    tauri::image::Image::new(&[], 0, 0)
                }
            };

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
                        save_window_state(app);
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
                let app_handle_dd = app.handle().clone();
                win.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            save_window_state(&app_handle_dd);
                            api.prevent_close();
                            if let Err(e) = win_clone.hide() {
                                log::debug!("Failed to hide window on close: {e}");
                            }
                        }
                        tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop {
                            paths, ..
                        }) => {
                            // Register dropped file paths server-side so that
                            // read_dropped_files can validate them. This avoids
                            // exposing an IPC command that the webview could call
                            // with arbitrary paths.
                            if let Some(state) = app_handle_dd.try_state::<AppState>() {
                                if let Ok(mut allowed) = state.allowed_file_paths.lock() {
                                    for path in paths {
                                        if let Some(s) = path.to_str() {
                                            allowed.insert(s.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                });
            }

            // Restore saved window position/size from previous session
            restore_window_state(app.handle());

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
            commands::mcp::test_mcp_connection_config,
            commands::mcp::get_mcp_tools,
            commands::mcp::invoke_mcp_tool,
            commands::mcp::fetch_mcp_registry,
            commands::mcp::approve_mcp_binary,
            commands::mcp::is_mcp_binary_approved,
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
            commands::sources::get_git_sources,
            commands::sources::get_git_source,
            commands::sources::create_git_source,
            commands::sources::update_git_source,
            commands::sources::delete_git_source,
            commands::sources::sync_git_source,
            commands::sources::import_source_items,
            commands::sources::sync_all_sources,
            commands::sources::get_source_items,
            commands::sources::search_catalog,
            commands::sources::install_catalog_item,
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
    if let Err(e) = app.show() {
        log::debug!("Failed to show app: {e}");
    }
    if let Some(win) = app.get_webview_window("main") {
        if let Err(e) = win.show() {
            log::debug!("Failed to show main window: {e}");
        }
        if let Err(e) = win.set_focus() {
            log::debug!("Failed to focus main window: {e}");
        }
    }
}

const WINDOW_STATE_STORE: &str = "window-state.json";

/// Save the main window's position, size, and maximized state to the store.
fn save_window_state(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let store = match app.store(WINDOW_STATE_STORE) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to open window state store: {e}");
            return;
        }
    };

    let maximized = win.is_maximized().unwrap_or(false);
    store.set("maximized", serde_json::json!(maximized));

    // Only save position/size when not maximized — the OS manages maximized geometry.
    if !maximized {
        if let Ok(pos) = win.outer_position() {
            store.set("x", serde_json::json!(pos.x));
            store.set("y", serde_json::json!(pos.y));
        }
        if let Ok(size) = win.outer_size() {
            store.set("width", serde_json::json!(size.width));
            store.set("height", serde_json::json!(size.height));
        }
    }
}

/// Restore the main window's position, size, and maximized state from the store.
fn restore_window_state(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let store = match app.store(WINDOW_STATE_STORE) {
        Ok(s) => s,
        Err(_) => return,
    };

    // If no saved state yet, leave the window at its tauri.conf.json defaults.
    if store.keys().is_empty() {
        return;
    }

    if store.get("maximized").and_then(|v| v.as_bool()) == Some(true) {
        let _ = win.maximize();
        return;
    }

    // Restore size first so that position is relative to the correct dimensions.
    let width = store
        .get("width")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok());
    let height = store
        .get("height")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok());
    if let (Some(w), Some(h)) = (width, height) {
        if w >= 400 && h >= 300 && w <= 10000 && h <= 10000 {
            let _ = win.set_size(tauri::PhysicalSize::new(w, h));
        }
    }

    // Restore position — validate it falls on a connected monitor.
    let x = store
        .get("x")
        .and_then(|v| v.as_i64())
        .and_then(|v| i32::try_from(v).ok());
    let y = store
        .get("y")
        .and_then(|v| v.as_i64())
        .and_then(|v| i32::try_from(v).ok());
    if let (Some(x), Some(y)) = (x, y) {
        if is_position_visible(&win, x, y) {
            let _ = win.set_position(tauri::PhysicalPosition::new(x, y));
        }
    }
}

/// Check if a window position is at least partially visible on any connected monitor.
///
/// Handles the case where a saved position refers to a now-disconnected external monitor.
fn is_position_visible(win: &tauri::WebviewWindow, x: i32, y: i32) -> bool {
    let monitors = match win.available_monitors() {
        Ok(m) => m,
        Err(_) => return true,
    };
    if monitors.is_empty() {
        return true;
    }

    // Allow some tolerance so the title bar remains reachable even if the
    // window is partially off-screen.
    let margin = 200;
    for monitor in &monitors {
        let mp = monitor.position();
        let ms = monitor.size();
        let mx = mp.x;
        let my = mp.y;
        let mw = i32::try_from(ms.width).unwrap_or(i32::MAX);
        let mh = i32::try_from(ms.height).unwrap_or(i32::MAX);

        if x >= mx - margin && x < mx + mw + margin && y >= my - margin && y < my + mh + margin {
            return true;
        }
    }

    false
}
