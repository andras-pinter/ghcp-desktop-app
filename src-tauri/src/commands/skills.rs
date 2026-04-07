//! Skill commands: list, toggle, configure, create, delete skills.

use crate::db::queries;
use crate::state::AppState;
use tauri::{AppHandle, Manager};

/// List all skills.
#[tauri::command]
pub fn get_skills(app: AppHandle) -> Result<Vec<queries::Skill>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_skills(&db).map_err(|e| e.to_string())
}

/// Create a new skill (used for registry/git imports).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_skill(
    app: AppHandle,
    id: String,
    name: String,
    description: Option<String>,
    source: String,
    mcp_server_id: Option<String>,
    config: Option<String>,
    instructions: Option<String>,
    source_url: Option<String>,
    source_type: Option<String>,
) -> Result<queries::Skill, String> {
    if name.trim().is_empty() {
        return Err("Skill name cannot be empty".to_string());
    }
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::create_skill(
        &db,
        &id,
        name.trim(),
        description.as_deref(),
        &source,
        mcp_server_id.as_deref(),
        config.as_deref(),
        instructions.as_deref(),
        source_url.as_deref(),
        &source_type.unwrap_or_else(|| "builtin".to_string()),
    )
    .map_err(|e| e.to_string())
}

/// Update an existing skill.
#[tauri::command]
pub fn update_skill(
    app: AppHandle,
    id: String,
    name: String,
    description: Option<String>,
    instructions: Option<String>,
    config: Option<String>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_skill(
        &db,
        &id,
        name.trim(),
        description.as_deref(),
        instructions.as_deref(),
        config.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Delete a skill.
#[tauri::command]
pub fn delete_skill(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_skill(&db, &id).map_err(|e| e.to_string())
}

/// Toggle a skill's enabled state.
#[tauri::command]
pub fn toggle_skill(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::toggle_skill(&db, &id, enabled).map_err(|e| e.to_string())
}
