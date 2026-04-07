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

// ── Registry ────────────────────────────────────────────────────

/// Search skill/agent registries (skills.sh + aitmpl.com).
#[tauri::command]
pub async fn search_registry(
    app: AppHandle,
    query: String,
    limit: Option<u32>,
) -> Result<crate::registry::RegistrySearchResult, String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;
    crate::registry::search_registries(client, &query, limit.unwrap_or(20)).await
}

/// Install a skill from a registry by fetching its SKILL.md content.
#[tauri::command]
pub async fn install_from_registry(
    app: AppHandle,
    skill_id: String,
    source: String,
    source_repo: Option<String>,
) -> Result<crate::registry::RegistryItem, String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;

    let registry_source = match source.as_str() {
        "skills_sh" => crate::registry::RegistrySource::SkillsSh,
        "aitmpl" => crate::registry::RegistrySource::Aitmpl,
        _ => return Err(format!("Unknown registry source: {source}")),
    };

    let content = crate::registry::fetch_skill_content(
        client,
        &skill_id,
        &registry_source,
        source_repo.as_deref(),
    )
    .await?;

    // Parse the SKILL.md — try strict first, then lenient for registry content
    let (name, description, instructions) = match crate::skillmd::parse(&content) {
        Ok(parsed) => (parsed.name, parsed.description, parsed.instructions),
        Err(_) => {
            // Lenient fallback: extract what we can from the content
            crate::registry::parse_content_lenient(&content, &skill_id)
        }
    };

    // Truncate description to fit DB constraints
    let description = if description.len() > 512 {
        format!("{}…", &description[..509])
    } else {
        description
    };

    // Save to database
    let db_id = format!("reg-{}-{}", source, skill_id);
    let source_type = match registry_source {
        crate::registry::RegistrySource::SkillsSh => "registry_skills_sh",
        crate::registry::RegistrySource::Aitmpl => "registry_aitmpl",
    };
    let source_url = match registry_source {
        crate::registry::RegistrySource::SkillsSh => {
            format!("https://skills.sh/{skill_id}")
        }
        crate::registry::RegistrySource::Aitmpl => {
            format!("https://www.aitmpl.com/{skill_id}")
        }
    };

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        queries::create_skill(
            &db,
            &db_id,
            &name,
            Some(&description),
            source_type,
            None,
            None,
            Some(&instructions),
            Some(&source_url),
            source_type,
        )
        .map_err(|e| format!("Failed to save skill: {e}"))?;
    }

    Ok(crate::registry::RegistryItem {
        id: db_id,
        name,
        description: Some(description),
        source: registry_source,
        url: Some(source_url),
        installs: None,
        kind: crate::registry::RegistryItemKind::Skill,
        source_repo: None,
    })
}

// ── Git Import ──────────────────────────────────────────────────

/// Fetch SKILL.md files from a git repository URL.
#[tauri::command]
pub async fn fetch_git_skills(
    app: AppHandle,
    git_url: String,
) -> Result<Vec<crate::registry::GitSkillFile>, String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;
    crate::registry::fetch_git_skills(client, &git_url).await
}

/// Import a single skill from a fetched SKILL.md content.
#[tauri::command]
pub fn import_git_skill(
    app: AppHandle,
    content: String,
    repo_url: String,
    path: String,
) -> Result<queries::Skill, String> {
    let parsed =
        crate::skillmd::parse(&content).map_err(|e| format!("Failed to parse SKILL.md: {e}"))?;

    let db_id = format!("git-{}", parsed.name);

    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let source_url = format!("{repo_url}/blob/main/{path}");

    queries::create_skill(
        &db,
        &db_id,
        &parsed.name,
        Some(&parsed.description),
        "git",
        None,
        None,
        Some(&parsed.instructions),
        Some(&source_url),
        "git",
    )
    .map_err(|e| format!("Failed to save skill: {e}"))
}
