//! Tauri command handlers — IPC bridge between frontend and backend.

mod agents;
mod auth;
mod chat;
mod conversations;
mod mcp;
mod models;
mod projects;
mod settings;
mod skills;
mod web_research;

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
