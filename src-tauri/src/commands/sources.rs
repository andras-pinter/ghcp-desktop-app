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
    let files = scan_source(&app, &id, &url).await?;

    // Persist discovered items for catalog browsing
    persist_catalog_items(&app, &id, &files)?;

    // Update item count
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        queries::update_git_source_synced(&db, &id, files.len() as i64)
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
    let files = scan_source(&app, &id, &url).await?;

    // Update existing imported items if their content changed
    update_existing_items(&app, &id, &files)?;

    // Persist discovered items for catalog browsing
    persist_catalog_items(&app, &id, &files)?;

    // Update sync timestamp + discovered item count
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        queries::refresh_git_source_item_count(&db, &id)
            .map_err(|e| format!("Failed to refresh item count: {e}"))?;
        queries::update_git_source_synced(&db, &id, files.len() as i64)
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
    let mut set = tokio::task::JoinSet::new();

    for source in enabled {
        let app_handle = app.clone();
        set.spawn(async move {
            match scan_and_update_source(&app_handle, &source).await {
                Ok(()) => {
                    log::info!("Synced source '{}'", source.name);
                    true
                }
                Err(e) => {
                    log::warn!("Failed to sync source '{}': {e}", source.name);
                    false
                }
            }
        });
    }

    let mut synced = 0;
    while let Some(result) = set.join_next().await {
        if matches!(result, Ok(true)) {
            synced += 1;
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

/// Search the unified catalog: aitmpl.com registry + git source items from enabled sources.
/// Optional `source_ids` filter: include `"aitmpl"` for aitmpl.com, UUIDs for specific git sources,
/// or `None` / empty for all sources.
#[tauri::command]
pub async fn search_catalog(
    app: AppHandle,
    query: String,
    kind: Option<String>,
    limit: Option<u32>,
    source_ids: Option<Vec<String>>,
) -> Result<crate::registry::RegistrySearchResult, String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;

    // Determine which sources to include based on the filter
    let has_filter = source_ids.as_ref().is_some_and(|ids| !ids.is_empty());
    let include_aitmpl = if has_filter {
        source_ids.as_ref().unwrap().iter().any(|id| id == "aitmpl")
    } else {
        true // No filter — include all
    };
    let git_ids: Option<Vec<String>> = if has_filter {
        let ids: Vec<String> = source_ids
            .as_ref()
            .unwrap()
            .iter()
            .filter(|id| *id != "aitmpl")
            .cloned()
            .collect();
        if ids.is_empty() {
            None
        } else {
            Some(ids)
        }
    } else {
        None // No filter — include all git sources
    };
    let include_git = !has_filter || git_ids.is_some();

    // Check if aitmpl.com registry is enabled (default: true)
    let aitmpl_enabled = if include_aitmpl {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        queries::get_setting(&db, "aitmpl_enabled")
            .map_err(|e| e.to_string())?
            .map(|v| v != "false")
            .unwrap_or(true)
    } else {
        false
    };

    // 1) Fetch aitmpl.com results (if enabled and included)
    let mut items: Vec<crate::registry::RegistryItem> = if aitmpl_enabled {
        let aitmpl_result =
            crate::registry::search_registries(client, &query, limit.unwrap_or(200)).await;
        match aitmpl_result {
            Ok(r) => r.items,
            Err(e) => {
                log::warn!("aitmpl.com search failed: {e}");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Filter aitmpl results by kind if specified
    if let Some(ref k) = kind {
        let target_kind = match k.as_str() {
            "skill" => crate::registry::RegistryItemKind::Skill,
            "agent" => crate::registry::RegistryItemKind::Agent,
            _ => return Err(format!("Invalid kind: {k}")),
        };
        items.retain(|i| i.kind == target_kind);
    }

    // 2) Fetch git source catalog items from enabled sources
    if include_git {
        let (git_items, source_names) = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let q = if query.trim().is_empty() {
                None
            } else {
                Some(query.as_str())
            };
            let catalog = queries::get_catalog_entries(&db, kind.as_deref(), q, git_ids.as_deref())
                .map_err(|e| format!("Catalog query failed: {e}"))?;

            // Only load the source name(s) we actually need
            let names: std::collections::HashMap<String, String> = match &git_ids {
                Some(ids) if ids.len() == 1 => queries::get_git_source(&db, &ids[0])
                    .map_err(|e| e.to_string())?
                    .map(|s| {
                        let mut m = std::collections::HashMap::new();
                        m.insert(s.id.clone(), s.name.clone());
                        m
                    })
                    .unwrap_or_default(),
                _ => queries::list_git_sources(&db)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .map(|s| (s.id.clone(), s.name.clone()))
                    .collect(),
            };
            (catalog, names)
        };

        // Convert git catalog items to RegistryItem (include content for preview)
        for gi in git_items {
            let item_kind = match gi.kind.as_str() {
                "agent" => crate::registry::RegistryItemKind::Agent,
                _ => crate::registry::RegistryItemKind::Skill,
            };
            let sname = source_names.get(&gi.git_source_id).cloned();
            items.push(crate::registry::RegistryItem {
                id: gi.id,
                name: gi.name,
                description: gi.description,
                source: crate::registry::RegistrySource::Git,
                source_name: sname,
                url: None,
                installs: None,
                kind: item_kind,
                source_repo: None,
                content: Some(gi.content),
            });
        }
    }

    // Sort: aitmpl items by installs (highest first), then git items alphabetically
    items.sort_by(|a, b| {
        let a_is_git = a.source == crate::registry::RegistrySource::Git;
        let b_is_git = b.source == crate::registry::RegistrySource::Git;
        match (a_is_git, b_is_git) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            (false, false) => b.installs.unwrap_or(0).cmp(&a.installs.unwrap_or(0)),
            (true, true) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    let total = items.len() as u64;
    Ok(crate::registry::RegistrySearchResult {
        items,
        total: Some(total),
    })
}

/// Install a skill or agent from a git source catalog item.
/// Reads the content from `git_source_items` and creates a skill/agent in the DB.
#[tauri::command]
pub fn install_catalog_item(app: AppHandle, item_id: String) -> Result<String, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Look up the catalog item by ID directly
    let catalog_item = queries::get_catalog_entry_by_id(&db, &item_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Catalog item '{item_id}' not found"))?;

    let source = queries::get_git_source(&db, &catalog_item.git_source_id)
        .map_err(|e| e.to_string())?
        .ok_or("Git source not found")?;

    // TODO: source_url uses hardcoded `blob/main` — may be wrong for repos with
    // a different default branch (e.g. `master`). Cosmetic issue only; does not
    // affect functionality since items are fetched via API, not these URLs.
    let source_url = format!("{}/blob/main/{}", source.url, catalog_item.path);

    // Parse content: try strict parser first, fall back to lenient
    let (name, description, instructions) = match crate::skillmd::parse(&catalog_item.content) {
        Ok(parsed) => (parsed.name, parsed.description, parsed.instructions),
        Err(_) => {
            let (n, d, i) =
                crate::registry::parse_content_lenient(&catalog_item.content, &catalog_item.name);
            (n, d, i)
        }
    };

    match catalog_item.kind.as_str() {
        "skill" => {
            let db_id = format!(
                "git-{}-{}",
                &source.id[..8.min(source.id.len())],
                name.to_lowercase().replace(' ', "-")
            );

            // Check for existing skill with same source_url to prevent duplicates
            let existing = queries::list_skills(&db).map_err(|e| e.to_string())?;
            if let Some(s) = existing
                .iter()
                .find(|s| s.source_url.as_deref() == Some(source_url.as_str()))
            {
                return Err(format!(
                    "Skill '{}' is already installed from this source",
                    s.name
                ));
            }

            queries::create_skill(
                &db,
                &db_id,
                &name,
                Some(&description),
                "git",
                None,
                None,
                Some(&instructions),
                Some(&source_url),
                "git",
                Some(&source.id),
            )
            .map_err(|e| format!("Failed to save skill: {e}"))?;

            queries::refresh_git_source_item_count(&db, &source.id)
                .map_err(|e| format!("Failed to refresh count: {e}"))?;

            Ok(db_id)
        }
        "agent" => {
            // Check for existing agent with same source_url to prevent duplicates
            let existing = queries::list_agents(&db).map_err(|e| e.to_string())?;
            if let Some(a) = existing
                .iter()
                .find(|a| a.source_url.as_deref() == Some(source_url.as_str()))
            {
                return Err(format!(
                    "Agent '{}' is already installed from this source",
                    a.name
                ));
            }

            let id = uuid::Uuid::new_v4().to_string();
            queries::create_agent(
                &db,
                &id,
                &name,
                None,
                &instructions,
                Some(&source_url),
                "git",
                Some(&source.id),
            )
            .map_err(|e| format!("Failed to save agent: {e}"))?;

            queries::refresh_git_source_item_count(&db, &source.id)
                .map_err(|e| format!("Failed to refresh count: {e}"))?;

            Ok(id)
        }
        _ => Err(format!("Unknown item kind: {}", catalog_item.kind)),
    }
}

// ── Internal helpers ────────────────────────────────────────────

/// Scan a repository for definition files (skills + agents).
/// Emits `git-import-progress` events with `source_id` attached.
async fn scan_source(
    app: &AppHandle,
    source_id: &str,
    url: &str,
) -> Result<Vec<crate::registry::GitSkillFile>, String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;
    let token = copilot_api::DeviceFlowAuth::load_github_token().ok();
    let emitter = app.clone();
    let sid = source_id.to_string();
    crate::registry::fetch_git_definitions(client, url, None, token.as_deref(), |p| {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ProgressWithSource {
            total: usize,
            fetched: usize,
            phase: String,
            source_id: String,
        }
        let _ = emitter.emit(
            "git-import-progress",
            &ProgressWithSource {
                total: p.total,
                fetched: p.fetched,
                phase: p.phase.clone(),
                source_id: sid.clone(),
            },
        );
    })
    .await
}

/// Import a single skill item into the database.
/// Skips if a skill with the same source_url already exists.
fn import_skill_item(
    db: &rusqlite::Connection,
    source: &queries::GitSource,
    item: &ImportItem,
) -> Result<(), String> {
    // TODO: `blob/main` assumes default branch — cosmetic only, see install_catalog_item
    let source_url = format!("{}/blob/main/{}", source.url, item.path);

    // Skip if already installed from this source URL
    let existing = queries::list_skills(db).map_err(|e| e.to_string())?;
    if existing
        .iter()
        .any(|s| s.source_url.as_deref() == Some(source_url.as_str()))
    {
        return Ok(());
    }

    // Parse content: try strict parser first, fall back to lenient
    let (name, description, instructions) = match crate::skillmd::parse(&item.content) {
        Ok(parsed) => (parsed.name, parsed.description, parsed.instructions),
        Err(_) => crate::registry::parse_content_lenient(&item.content, &item.path),
    };

    let slug = name.to_lowercase().replace(' ', "-");
    let db_id = format!("git-{}-{}", &source.id[..8.min(source.id.len())], slug);

    queries::create_skill(
        db,
        &db_id,
        &name,
        Some(&description),
        "git",
        None,
        None,
        Some(&instructions),
        Some(&source_url),
        "git",
        Some(&source.id),
    )
    .map_err(|e| format!("DB error: {e}"))?;

    Ok(())
}

/// Import a single agent item into the database.
/// Skips if an agent with the same source_url already exists.
fn import_agent_item(
    db: &rusqlite::Connection,
    source: &queries::GitSource,
    item: &ImportItem,
) -> Result<(), String> {
    // TODO: `blob/main` assumes default branch — cosmetic only, see install_catalog_item
    let source_url = format!("{}/blob/main/{}", source.url, item.path);

    // Skip if already installed from this source URL
    let existing = queries::list_agents(db).map_err(|e| e.to_string())?;
    if existing
        .iter()
        .any(|a| a.source_url.as_deref() == Some(source_url.as_str()))
    {
        return Ok(());
    }

    // Parse content: try strict parser first, fall back to lenient
    let (name, _description, instructions) = match crate::skillmd::parse(&item.content) {
        Ok(parsed) => (parsed.name, parsed.description, parsed.instructions),
        Err(_) => crate::registry::parse_content_lenient(&item.content, &item.path),
    };

    let id = uuid::Uuid::new_v4().to_string();

    queries::create_agent(
        db,
        &id,
        &name,
        None,
        &instructions,
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
            // Parse content: try strict first, fall back to lenient
            let (name, description, instructions) = match crate::skillmd::parse(&file.content) {
                Ok(parsed) => (parsed.name, parsed.description, parsed.instructions),
                Err(_) => crate::registry::parse_content_lenient(&file.content, &item.name),
            };
            match item.kind.as_str() {
                "skill" => {
                    let _ = queries::update_skill(
                        &db,
                        &item.id,
                        &name,
                        Some(&description),
                        Some(&instructions),
                        None,
                    );
                }
                "agent" => {
                    let _ = queries::update_agent(
                        &db,
                        &item.id,
                        &name,
                        None,
                        &instructions,
                        None,
                        "git",
                    );
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Scan and update a single source (used during auto-sync).
/// Emits progress events so the UI can show per-card activity.
async fn scan_and_update_source(
    app: &AppHandle,
    source: &queries::GitSource,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let client = &state.http_client;
    let token = copilot_api::DeviceFlowAuth::load_github_token().ok();
    let emitter = app.clone();
    let sid = source.id.clone();

    let files =
        crate::registry::fetch_git_definitions(client, &source.url, None, token.as_deref(), |p| {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct ProgressWithSource {
                total: usize,
                fetched: usize,
                phase: String,
                source_id: String,
            }
            let _ = emitter.emit(
                "git-import-progress",
                &ProgressWithSource {
                    total: p.total,
                    fetched: p.fetched,
                    phase: p.phase.clone(),
                    source_id: sid.clone(),
                },
            );
        })
        .await?;

    // Update existing imported items if their content changed
    update_existing_items(app, &source.id, &files)?;

    // Persist discovered items for catalog browsing
    persist_catalog_items(app, &source.id, &files)?;

    // Update sync timestamp + item count
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::refresh_git_source_item_count(&db, &source.id)
        .map_err(|e| format!("Failed to refresh item count: {e}"))?;
    queries::update_git_source_synced(&db, &source.id, files.len() as i64)
        .map_err(|e| format!("Failed to update sync timestamp: {e}"))?;

    // Notify frontend that this source's sync is fully committed to DB
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SyncComplete {
        source_id: String,
    }
    let _ = app.emit(
        "git-source-sync-complete",
        &SyncComplete {
            source_id: source.id.clone(),
        },
    );

    Ok(())
}

/// Persist discovered files into `git_source_items` for catalog browsing.
/// Parses each file to extract name + description, upserts into the table,
/// and removes stale items no longer in the repo.
fn persist_catalog_items(
    app: &AppHandle,
    source_id: &str,
    files: &[crate::registry::GitSkillFile],
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut current_paths = Vec::new();

    for file in files {
        let kind = if file.path.ends_with(".agent.md") {
            "agent"
        } else {
            "skill"
        };

        // Parse to extract name + description
        let (name, description) = match crate::skillmd::parse(&file.content) {
            Ok(parsed) => (parsed.name, Some(parsed.description)),
            Err(_) => {
                let (n, d, _) = crate::registry::parse_content_lenient(&file.content, &file.path);
                (n, if d.is_empty() { None } else { Some(d) })
            }
        };

        let id = format!("gsi-{}-{}", source_id, file.path);

        queries::upsert_git_source_item(
            &db,
            &id,
            source_id,
            &file.path,
            kind,
            &name,
            description.as_deref(),
            &file.content,
        )
        .map_err(|e| format!("Failed to upsert catalog item: {e}"))?;

        current_paths.push(file.path.clone());
    }

    // Remove items that are no longer in the repo
    queries::delete_stale_source_items(&db, source_id, &current_paths)
        .map_err(|e| format!("Failed to remove stale items: {e}"))?;

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
