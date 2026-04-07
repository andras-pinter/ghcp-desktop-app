//! Conversation commands: CRUD for conversations + messages.

use crate::db::queries;
use crate::state::AppState;
use tauri::{AppHandle, Manager};

/// List all conversations for the sidebar.
#[tauri::command]
pub fn get_conversations(app: AppHandle) -> Result<Vec<queries::Conversation>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_conversations(&db).map_err(|e| e.to_string())
}

/// Get a single conversation by ID.
#[tauri::command]
pub fn get_conversation(
    app: AppHandle,
    id: String,
) -> Result<Option<queries::Conversation>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_conversation(&db, &id).map_err(|e| e.to_string())
}

/// Create a new conversation.
#[tauri::command]
pub fn create_conversation(
    app: AppHandle,
    id: String,
    title: Option<String>,
    agent_id: Option<String>,
    project_id: Option<String>,
    model: Option<String>,
) -> Result<queries::Conversation, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::create_conversation(
        &db,
        &id,
        title.as_deref(),
        agent_id.as_deref(),
        project_id.as_deref(),
        model.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Update conversation fields (title, favourite, model, project).
#[tauri::command]
pub fn update_conversation(
    app: AppHandle,
    id: String,
    title: Option<String>,
    is_favourite: Option<bool>,
    model: Option<String>,
    project_id: Option<Option<String>>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let pid = project_id.as_ref().map(|p| p.as_deref());
    queries::update_conversation(
        &db,
        &id,
        title.as_deref(),
        is_favourite,
        model.as_deref(),
        pid,
    )
    .map_err(|e| e.to_string())
}

/// Delete a conversation (messages + drafts cascade).
#[tauri::command]
pub fn delete_conversation(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_conversation(&db, &id).map_err(|e| e.to_string())
}

/// Get all messages for a conversation.
#[tauri::command]
pub fn get_messages(
    app: AppHandle,
    conversation_id: String,
) -> Result<Vec<queries::Message>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_messages(&db, &conversation_id).map_err(|e| e.to_string())
}

/// Save a message (insert into DB).
#[tauri::command]
pub fn create_message(app: AppHandle, message: queries::Message) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::create_message(&db, &message).map_err(|e| e.to_string())?;
    queries::touch_conversation(&db, &message.conversation_id).map_err(|e| e.to_string())
}

/// Update a message's content (after streaming completes or user edits).
#[tauri::command]
pub fn update_message_content(
    app: AppHandle,
    id: String,
    content: String,
    thinking_content: Option<String>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_message_content(&db, &id, &content, thinking_content.as_deref())
        .map_err(|e| e.to_string())
}

/// Delete all messages after a given sort_order (for message editing).
#[tauri::command]
pub fn delete_messages_after(
    app: AppHandle,
    conversation_id: String,
    after_sort_order: i64,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_messages_after(&db, &conversation_id, after_sort_order)
        .map_err(|e| e.to_string())
}
