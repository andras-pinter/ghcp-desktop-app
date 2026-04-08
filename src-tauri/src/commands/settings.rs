//! Settings commands: get/update settings, DB management, export.
//! Also includes draft commands for crash recovery.

use crate::db::queries;
use crate::state::AppState;
use tauri::{AppHandle, Manager};

// ── Settings ────────────────────────────────────────────────────

/// Get a single setting value by key.
#[tauri::command]
pub fn get_setting(app: AppHandle, key: String) -> Result<Option<String>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_setting(&db, &key).map_err(|e| e.to_string())
}

/// Set a setting value (insert or replace).
#[tauri::command]
pub fn update_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_setting(&db, &key, &value).map_err(|e| e.to_string())
}

/// Get the database file size in bytes.
#[tauri::command]
pub fn get_db_size(app: AppHandle) -> Result<u64, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = app_data_dir.join("chuck.db");
    queries::get_db_size(&db_path).map_err(|e| e.to_string())
}

/// Delete conversations older than the given number of days. Returns count deleted.
#[tauri::command]
pub fn delete_old_conversations(app: AppHandle, older_than_days: u32) -> Result<usize, String> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(older_than_days));
    let before_date = cutoff.to_rfc3339();
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_old_conversations(&db, &before_date).map_err(|e| e.to_string())
}

/// Export a single conversation as JSON string.
#[tauri::command]
pub fn export_conversation_json(app: AppHandle, id: String) -> Result<String, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let export = queries::get_conversation_for_export(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Conversation {} not found", id))?;
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

/// Export a single conversation as Markdown string.
#[tauri::command]
pub fn export_conversation_markdown(app: AppHandle, id: String) -> Result<String, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let export = queries::get_conversation_for_export(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Conversation {} not found", id))?;
    Ok(queries::conversation_to_markdown(&export))
}

/// Export all conversations as JSON string.
#[tauri::command]
pub fn export_all_conversations_json(app: AppHandle) -> Result<String, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let exports = queries::get_all_conversations_for_export(&db).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&exports).map_err(|e| e.to_string())
}

/// Export all conversations as a single Markdown string.
#[tauri::command]
pub fn export_all_conversations_markdown(app: AppHandle) -> Result<String, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let exports = queries::get_all_conversations_for_export(&db).map_err(|e| e.to_string())?;
    let mut result = String::new();
    for export in &exports {
        result.push_str(&queries::conversation_to_markdown(export));
        result.push('\n');
    }
    Ok(result)
}

/// Write text content to a user-chosen file path (for conversation export).
#[tauri::command]
pub fn save_export_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("Failed to write file: {}", e))
}

// ── Drafts ──────────────────────────────────────────────────────

/// Save (or update) a draft for a conversation.
#[tauri::command]
pub fn save_draft(app: AppHandle, conversation_id: String, content: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::save_draft(&db, &conversation_id, &content).map_err(|e| e.to_string())
}

/// Get the draft for a conversation, if any.
#[tauri::command]
pub fn get_draft(
    app: AppHandle,
    conversation_id: String,
) -> Result<Option<queries::Draft>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_draft(&db, &conversation_id).map_err(|e| e.to_string())
}

/// Delete the draft for a conversation.
#[tauri::command]
pub fn delete_draft(app: AppHandle, conversation_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_draft(&db, &conversation_id).map_err(|e| e.to_string())
}
