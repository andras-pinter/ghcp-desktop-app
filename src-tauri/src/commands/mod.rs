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
pub mod sources;
pub mod web_research;

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
    // Cap length to prevent log flooding from the frontend
    let msg = if message.len() > 1024 {
        &message[..1024]
    } else {
        message
    };
    match level {
        "error" => log::error!("[frontend] {}", msg),
        "warn" => log::warn!("[frontend] {}", msg),
        "debug" => log::debug!("[frontend] {}", msg),
        _ => log::info!("[frontend] {}", msg),
    }
}
