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
            app.manage(app_state);

            // Open devtools in debug builds
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            log::info!("Chuck initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Chuck");
}
