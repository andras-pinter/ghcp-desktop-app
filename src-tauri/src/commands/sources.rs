//! Git source management commands: add, remove, toggle, sync, import items.

use crate::db::queries;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

/// An item discovered during a source scan, ready for import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportItem {
    /// File path within the repository.
    pub path: String,
    /// Raw file content (SKILL.md or *.agent.md).
    pub content: String,
    /// Item type: "skill" or "agent".
    pub kind: String,
}

/// Result of creating or syncing a source: the source + discovered files.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceScanResult {
    /// The git source metadata.
    pub source: queries::GitSource,
    /// Files discovered in the repository.
    pub files: Vec<crate::registry::GitSkillFile>,
}

/// List all git sources.
#[tauri::command]
pub fn get_git_sources(app: AppHandle) -> Result<Vec<queries::GitSource>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_git_sources(&db).map_err(|e| e.to_string())
}

/// Get a single git source by ID.
#[tauri::command]
pub fn get_git_source(app: AppHandle, id: String) -> Result<Option<queries::GitSource>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_git_source(&db, &id).map_err(|e| e.to_string())
}

/// Create a new git source: validates URL, creates DB record, scans the repo,
/// and returns the source + discovered files for the user to pick from.
#[tauri::command]
pub async fn create_git_source(
    app: AppHandle,
    url: String,
    name: Option<String>,
) -> Result<SourceScanResult, String> {
    let url = url.trim().to_string();
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    // Derive a display name from the URL if none provided
    let display_name = name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| repo_name_from_url(&url));

    let id = uuid::Uuid::new_v4().to_string();

    // Check for duplicate URL
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let existing = queries::list_git_sources(&db).map_err(|e| e.to_string())?;
        if existing.iter().any(|s| s.url == url) {
            return Err(format!("A source with URL '{url}' already exists"));
        }
        queries::create_git_source(&db, &id, &display_name, &url)
            .map_err(|e| format!("Failed to create source: {e}"))?;
    }

    // Scan the repository for definition files
    let files = scan_source(&app, &url).await?;

    // Update item count (discovered, not yet imported)
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        // Item count = 0 initially (nothing imported yet); we track discovered count client-side
        queries::update_git_source_synced(&db, &id, 0)
            .map_err(|e| format!("Failed to update sync timestamp: {e}"))?;
    }

    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let source = queries::get_git_source(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or("Source not found after creation")?;

    Ok(SourceScanResult { source, files })
}

/// Update a git source's name and/or enabled state.
#[tauri::command]
pub fn update_git_source(
    app: AppHandle,
    id: String,
    name: Option<String>,
    enabled: Option<bool>,
) -> Result<queries::GitSource, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_git_source(&db, &id, name.as_deref(), enabled)
        .map_err(|e| format!("Failed to update source: {e}"))?;
    queries::get_git_source(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Source not found".to_string())
}

/// Delete a git source. Imported items remain as orphaned local copies.
#[tauri::command]
pub fn delete_git_source(app: AppHandle, id: String) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_git_source(&db, &id).map_err(|e| e.to_string())
}

/// Re-scan a source's repository and return discovered files.
/// Also updates already-imported items if their content has changed.
#[tauri::command]
pub async fn sync_git_source(app: AppHandle, id: String) -> Result<SourceScanResult, String> {
    // Get the source to find the URL
    let (url, enabled) = {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let source = queries::get_git_source(&db, &id)
            .map_err(|e| e.to_string())?
            .ok_or("Source not found")?;
        (source.url, source.enabled)
    };

    if !enabled {
        return Err("Cannot sync a disabled source".to_string());
    }

    // Scan the repository
    let files = scan_source(&app, &url).await?;

    // Update existing imported items if their content changed
    update_existing_items(&app, &id, &files)?;

    // Update sync timestamp + actual imported item count
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        queries::refresh_git_source_item_count(&db, &id)
            .map_err(|e| format!("Failed to refresh item count: {e}"))?;
        queries::update_git_source_synced(
            &db,
            &id,
            queries::get_source_items(&db, &id)
                .map_err(|e| e.to_string())?
                .len() as i64,
        )
        .map_err(|e| format!("Failed to update sync timestamp: {e}"))?;
    }

    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let source = queries::get_git_source(&db, &id)
        .map_err(|e| e.to_string())?
        .ok_or("Source not found")?;

    Ok(SourceScanResult { source, files })
}

/// Import selected items from a source scan into the database.
#[tauri::command]
pub fn import_source_items(
    app: AppHandle,
    source_id: String,
    items: Vec<ImportItem>,
) -> Result<usize, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Verify source exists
    let source = queries::get_git_source(&db, &source_id)
        .map_err(|e| e.to_string())?
        .ok_or("Source not found")?;

    let mut imported = 0;

    for item in &items {
        let result = match item.kind.as_str() {
            "skill" => import_skill_item(&db, &source, item),
            "agent" => import_agent_item(&db, &source, item),
            _ => Err(format!("Unknown item kind: {}", item.kind)),
        };

        match result {
            Ok(()) => imported += 1,
            Err(e) => log::warn!("Failed to import {}: {e}", item.path),
        }
    }

    // Refresh item count
    queries::refresh_git_source_item_count(&db, &source_id)
        .map_err(|e| format!("Failed to refresh item count: {e}"))?;

    Ok(imported)
}

/// Sync all enabled sources (called on app launch). Silently updates existing
/// items if their content changed. Returns the number of sources synced.
#[tauri::command]
pub async fn sync_all_sources(app: AppHandle) -> Result<usize, String> {
    let sources = {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        queries::list_git_sources(&db).map_err(|e| e.to_string())?
    };

    let enabled: Vec<_> = sources.into_iter().filter(|s| s.enabled).collect();
    let mut synced = 0;

    for source in &enabled {
        match scan_and_update_source(&app, source).await {
            Ok(()) => synced += 1,
            Err(e) => log::warn!("Failed to sync source '{}': {e}", source.name),
        }
    }

    Ok(synced)
}

/// List skills and agents linked to a specific source.
#[tauri::command]
pub fn get_source_items(
    app: AppHandle,
    source_id: String,
) -> Result<Vec<queries::SourceItem>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_source_items(&db, &source_id).map_err(|e| e.to_string())
}

// ── Internal helpers ────────────────────────────────────────────

/// Scan a repository for definition files (skills + agents).
async fn scan_source(
    app: &AppHandle,
    url: &str,
) -> Result<Vec<crate::registry::GitSkillFile>, String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;
    let token = copilot_api::DeviceFlowAuth::load_github_token().ok();
    let emitter = app.clone();
    crate::registry::fetch_git_definitions(client, url, None, token.as_deref(), |p| {
        let _ = emitter.emit("git-import-progress", &p);
    })
    .await
}

/// Import a single skill item into the database.
fn import_skill_item(
    db: &rusqlite::Connection,
    source: &queries::GitSource,
    item: &ImportItem,
) -> Result<(), String> {
    let parsed = crate::skillmd::parse(&item.content).map_err(|e| format!("Parse error: {e}"))?;

    let db_id = format!("git-{}", parsed.name);
    let source_url = format!("{}/blob/main/{}", source.url, item.path);

    queries::create_skill(
        db,
        &db_id,
        &parsed.name,
        Some(&parsed.description),
        "git",
        None,
        None,
        Some(&parsed.instructions),
        Some(&source_url),
        "git",
        Some(&source.id),
    )
    .map_err(|e| format!("DB error: {e}"))?;

    Ok(())
}

/// Import a single agent item into the database.
fn import_agent_item(
    db: &rusqlite::Connection,
    source: &queries::GitSource,
    item: &ImportItem,
) -> Result<(), String> {
    let parsed = crate::skillmd::parse(&item.content).map_err(|e| format!("Parse error: {e}"))?;

    let id = uuid::Uuid::new_v4().to_string();
    let source_url = format!("{}/blob/main/{}", source.url, item.path);

    queries::create_agent(
        db,
        &id,
        &parsed.name,
        None,
        &parsed.instructions,
        Some(&source_url),
        "git",
        Some(&source.id),
    )
    .map_err(|e| format!("DB error: {e}"))?;

    Ok(())
}

/// Update already-imported items if their content changed in the repo.
fn update_existing_items(
    app: &AppHandle,
    source_id: &str,
    files: &[crate::registry::GitSkillFile],
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let existing_items = queries::get_source_items(&db, source_id).map_err(|e| e.to_string())?;

    for item in &existing_items {
        let source_url = item.source_url.as_deref().unwrap_or_default();
        // Match by path suffix in source_url
        if let Some(file) = files.iter().find(|f| source_url.ends_with(&f.path)) {
            // Re-parse and update content if parseable
            if let Ok(parsed) = crate::skillmd::parse(&file.content) {
                match item.kind.as_str() {
                    "skill" => {
                        let _ = queries::update_skill(
                            &db,
                            &item.id,
                            &parsed.name,
                            Some(&parsed.description),
                            Some(&parsed.instructions),
                            None,
                        );
                    }
                    "agent" => {
                        let _ = queries::update_agent(
                            &db,
                            &item.id,
                            &parsed.name,
                            None,
                            &parsed.instructions,
                            None,
                            "git",
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

/// Silently scan and update a single source (used during auto-sync).
async fn scan_and_update_source(
    app: &AppHandle,
    source: &queries::GitSource,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;
    let token = copilot_api::DeviceFlowAuth::load_github_token().ok();

    // Scan without emitting progress (silent sync)
    let files =
        crate::registry::fetch_git_definitions(client, &source.url, None, token.as_deref(), |_| {})
            .await?;

    // Update existing imported items if their content changed
    update_existing_items(app, &source.id, &files)?;

    // Update sync timestamp + item count
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::refresh_git_source_item_count(&db, &source.id)
        .map_err(|e| format!("Failed to refresh item count: {e}"))?;
    let count = queries::get_source_items(&db, &source.id)
        .map_err(|e| e.to_string())?
        .len() as i64;
    queries::update_git_source_synced(&db, &source.id, count)
        .map_err(|e| format!("Failed to update sync timestamp: {e}"))?;

    Ok(())
}

/// Extract a repository name from a URL for display.
fn repo_name_from_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/').trim_end_matches(".git");
    if let Some(last) = trimmed.rsplit('/').next() {
        if !last.is_empty() {
            return last.to_string();
        }
    }
    url.to_string()
}
