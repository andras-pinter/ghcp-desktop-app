//! Tauri command handlers — IPC bridge between frontend and backend.

pub mod agents;
pub mod auth;
pub mod chat;
pub mod conversations;
pub mod mcp;
pub mod models;
pub mod projects;
pub mod settings;
pub mod skills;
pub mod web_research;

/// Temporary greeting command for verifying IPC works.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Chuck.", name)
}

/// Return basic application info.
#[tauri::command]
pub fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "Chuck",
        "version": env!("CARGO_PKG_VERSION"),
    })
}

/// Log a message from the frontend to the Rust console.
#[tauri::command]
pub fn log_frontend(level: &str, message: &str) {
    match level {
        "error" => log::error!("[frontend] {message}"),
        "warn" => log::warn!("[frontend] {message}"),
        "debug" => log::debug!("[frontend] {message}"),
        _ => log::info!("[frontend] {message}"),
    }
}
