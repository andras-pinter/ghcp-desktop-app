//! Settings commands: get/update settings, DB management.
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
