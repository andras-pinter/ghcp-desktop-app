//! Copilot Desktop — Tauri application setup and plugin registration.

mod commands;
mod db;
mod state;

use state::AppState;
use tauri::Manager;

/// Run the Tauri application.
pub fn run() {
    env_logger::init();

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
            log::info!("Copilot Desktop initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Copilot Desktop");
}
