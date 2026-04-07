//! Agent commands: CRUD for agent personas.

use crate::db::queries;
use crate::state::AppState;
use tauri::{AppHandle, Manager};

/// List all agents (default agent first).
#[tauri::command]
pub fn get_agents(app: AppHandle) -> Result<Vec<queries::Agent>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_agents(&db).map_err(|e| e.to_string())
}

/// Get a single agent by ID.
#[tauri::command]
pub fn get_agent(app: AppHandle, id: String) -> Result<Option<queries::Agent>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_agent(&db, &id).map_err(|e| e.to_string())
}

/// Create a new agent.
#[tauri::command]
pub fn create_agent(
    app: AppHandle,
    name: String,
    avatar: Option<String>,
    system_prompt: String,
    source_url: Option<String>,
    source_type: Option<String>,
) -> Result<queries::Agent, String> {
    if name.trim().is_empty() {
        return Err("Agent name cannot be empty".to_string());
    }
    if system_prompt.trim().is_empty() {
        return Err("System prompt cannot be empty".to_string());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::create_agent(
        &db,
        &id,
        name.trim(),
        avatar.as_deref(),
        system_prompt.trim(),
        source_url.as_deref(),
        &source_type.unwrap_or_else(|| "local".to_string()),
    )
    .map_err(|e| e.to_string())
}

/// Update an existing agent (cannot modify the default agent).
#[tauri::command]
pub fn update_agent(
    app: AppHandle,
    id: String,
    name: String,
    avatar: Option<String>,
    system_prompt: String,
    source_url: Option<String>,
    source_type: Option<String>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Agent name cannot be empty".to_string());
    }
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_agent(
        &db,
        &id,
        name.trim(),
        avatar.as_deref(),
        system_prompt.trim(),
        source_url.as_deref(),
        &source_type.unwrap_or_else(|| "local".to_string()),
    )
    .map_err(|e| e.to_string())
}

/// Delete an agent (cannot delete the default agent).
#[tauri::command]
pub fn delete_agent(app: AppHandle, id: String) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_agent(&db, &id).map_err(|e| e.to_string())
}

/// Set the skills assigned to an agent.
#[tauri::command]
pub fn set_agent_skills(
    app: AppHandle,
    agent_id: String,
    skill_ids: Vec<String>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_agent_skills(&db, &agent_id, &skill_ids).map_err(|e| e.to_string())
}

/// Set the MCP server connections for an agent.
#[tauri::command]
pub fn set_agent_mcp_connections(
    app: AppHandle,
    agent_id: String,
    mcp_server_ids: Vec<String>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_agent_mcp_connections(&db, &agent_id, &mcp_server_ids).map_err(|e| e.to_string())
}
