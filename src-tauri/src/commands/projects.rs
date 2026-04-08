//! Project commands: CRUD for projects + file attachments.

use crate::db::queries;
use crate::state::AppState;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

/// List all projects.
#[tauri::command]
pub fn get_projects(app: AppHandle) -> Result<Vec<queries::Project>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_projects(&db).map_err(|e| e.to_string())
}

/// Get a single project by ID.
#[tauri::command]
pub fn get_project(app: AppHandle, id: String) -> Result<Option<queries::Project>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::get_project(&db, &id).map_err(|e| e.to_string())
}

/// Create a new project.
#[tauri::command]
pub fn create_project(
    app: AppHandle,
    id: String,
    name: String,
    instructions: Option<String>,
) -> Result<queries::Project, String> {
    if name.trim().is_empty() {
        return Err("Project name cannot be empty".to_string());
    }
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::create_project(&db, &id, name.trim(), instructions.as_deref())
        .map_err(|e| e.to_string())
}

/// Update a project's name and/or instructions.
#[tauri::command]
pub fn update_project(
    app: AppHandle,
    id: String,
    name: Option<String>,
    instructions: Option<Option<String>>,
) -> Result<(), String> {
    if let Some(ref n) = name {
        if n.trim().is_empty() {
            return Err("Project name cannot be empty".to_string());
        }
    }
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instr = instructions.as_ref().map(|i| i.as_deref());
    queries::update_project(&db, &id, name.as_deref(), instr).map_err(|e| e.to_string())
}

/// Delete a project (files cascade, conversations are unlinked).
#[tauri::command]
pub fn delete_project(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_project(&db, &id).map_err(|e| e.to_string())
}

/// List files attached to a project (metadata only).
#[tauri::command]
pub fn get_project_files(
    app: AppHandle,
    project_id: String,
) -> Result<Vec<queries::ProjectFile>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_project_files(&db, &project_id).map_err(|e| e.to_string())
}

/// File content received from the frontend (base64-encoded).
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    pub name: String,
    pub content_type: String,
    pub content_base64: String,
}

/// Add a file to a project (content sent as base64 from frontend).
#[tauri::command]
pub fn add_project_file(
    app: AppHandle,
    project_id: String,
    file: FileUpload,
) -> Result<queries::ProjectFile, String> {
    use base64::Engine;
    let content = base64::engine::general_purpose::STANDARD
        .decode(&file.content_base64)
        .map_err(|e| format!("Invalid base64 content: {e}"))?;

    // Enforce max file size (50MB)
    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large ({} bytes). Maximum is {} bytes.",
            content.len(),
            MAX_FILE_SIZE
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_project_file(
        &db,
        &id,
        &project_id,
        &file.name,
        &file.content_type,
        &content,
    )
    .map_err(|e| e.to_string())
}

/// Get the raw content of a project file (returned as base64).
#[tauri::command]
pub fn get_project_file_content(app: AppHandle, file_id: String) -> Result<Option<String>, String> {
    use base64::Engine;
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = queries::get_project_file_content(&db, &file_id).map_err(|e| e.to_string())?;
    Ok(result.map(|(_, content)| base64::engine::general_purpose::STANDARD.encode(content)))
}

/// Remove a file from a project.
#[tauri::command]
pub fn remove_project_file(app: AppHandle, file_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::remove_project_file(&db, &file_id).map_err(|e| e.to_string())
}

/// List conversations belonging to a project.
#[tauri::command]
pub fn get_project_conversations(
    app: AppHandle,
    project_id: String,
) -> Result<Vec<queries::Conversation>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_project_conversations(&db, &project_id).map_err(|e| e.to_string())
}

/// Open a native file dialog and read the selected file for project upload.
///
/// The dialog is opened on the Rust side so the frontend never receives or
/// controls the file path — only the returned content.
#[tauri::command]
pub async fn pick_file_for_upload(app: AppHandle) -> Result<Option<FileUpload>, String> {
    use base64::Engine;

    let file_path = app.dialog().file().blocking_pick_file();
    let path = match file_path {
        Some(fp) => fp.into_path().map_err(|e| e.to_string())?,
        None => return Ok(None), // user cancelled
    };

    let content = std::fs::read(&path).map_err(|e| format!("Failed to read file: {e}"))?;

    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large ({} bytes). Maximum is {} bytes.",
            content.len(),
            MAX_FILE_SIZE
        ));
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let content_type = guess_content_type(&name);

    Ok(Some(FileUpload {
        name,
        content_type,
        content_base64: base64::engine::general_purpose::STANDARD.encode(&content),
    }))
}

/// Open a native file dialog and read the selected file for chat attachments.
///
/// The dialog is opened on the Rust side so the frontend never receives or
/// controls the file path — only the returned content.
#[tauri::command]
pub async fn pick_file_for_chat(app: AppHandle) -> Result<Option<ChatFileData>, String> {
    use base64::Engine;

    let file_path = app.dialog().file().blocking_pick_file();
    let path = match file_path {
        Some(fp) => fp.into_path().map_err(|e| e.to_string())?,
        None => return Ok(None), // user cancelled
    };

    let content = std::fs::read(&path).map_err(|e| format!("Failed to read file: {e}"))?;

    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large ({} bytes). Maximum is {} bytes.",
            content.len(),
            MAX_FILE_SIZE
        ));
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let content_type = guess_content_type(&name);

    Ok(Some(ChatFileData {
        name,
        content_type,
        size: content.len(),
        content_base64: base64::engine::general_purpose::STANDARD.encode(&content),
    }))
}

/// File data returned from disk read for chat attachments.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatFileData {
    pub name: String,
    pub content_type: String,
    pub size: usize,
    pub content_base64: String,
}

/// Simple MIME type guessing based on file extension.
fn guess_content_type(filename: &str) -> String {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "txt" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "rs" => "text/x-rust",
        "ts" | "tsx" => "text/typescript",
        "js" | "jsx" | "mjs" => "text/javascript",
        "py" => "text/x-python",
        "rb" => "text/x-ruby",
        "go" => "text/x-go",
        "java" => "text/x-java",
        "c" | "h" => "text/x-c",
        "cpp" | "cc" | "cxx" | "hpp" => "text/x-c++",
        "cs" => "text/x-csharp",
        "swift" => "text/x-swift",
        "kt" | "kts" => "text/x-kotlin",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "xml" => "text/xml",
        "json" => "application/json",
        "yaml" | "yml" => "text/yaml",
        "toml" => "text/toml",
        "sql" => "text/x-sql",
        "sh" | "bash" | "zsh" => "text/x-shellscript",
        "csv" => "text/csv",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "svelte" => "text/x-svelte",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Extract readable text from a base64-encoded file.
///
/// Delegates to the `text_extract` module which supports PDF, DOCX, XLSX, PPTX,
/// RTF, and all text-based formats. Runs on a blocking thread to avoid stalling
/// the main thread for large files.
#[tauri::command]
pub async fn extract_file_text(
    content_base64: String,
    content_type: String,
    name: String,
) -> Result<Option<String>, String> {
    log::info!(
        "extract_file_text: name={}, content_type={}, base64_len={}",
        name,
        content_type,
        content_base64.len()
    );
    tokio::task::spawn_blocking(move || {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&content_base64)
            .map_err(|e| format!("Invalid base64: {e}"))?;

        log::info!(
            "extract_file_text: decoded {} bytes for {}",
            bytes.len(),
            name
        );
        let result = crate::text_extract::extract(&bytes, &content_type, &name);
        log::info!(
            "extract_file_text: result for {} = {}",
            name,
            match &result {
                Some(s) => format!("{} chars", s.len()),
                None => "None (unsupported)".to_string(),
            }
        );
        Ok(result)
    })
    .await
    .map_err(|e| format!("Extraction task failed: {e}"))?
}

/// Register file paths as allowed for reading (called from native drag-drop
/// or file-picker events). These are consumed once by `read_dropped_files`.
#[tauri::command]
pub fn register_allowed_paths(
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<(), String> {
    let mut allowed = state.allowed_file_paths.lock().map_err(|e| e.to_string())?;
    for p in paths {
        allowed.insert(p);
    }
    Ok(())
}

/// Read files from OS-level drag-and-drop paths (provided by Tauri's
/// `onDragDropEvent`).  Returns base64-encoded content for each valid file.
///
/// Only reads paths that were previously registered via `register_allowed_paths`.
/// Paths are consumed (removed from the allow-set) after reading.
#[tauri::command]
pub async fn read_dropped_files(
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<ChatFileData>, String> {
    use base64::Engine;

    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

    // Validate all requested paths against the allowed set and consume them.
    {
        let mut allowed = state.allowed_file_paths.lock().map_err(|e| e.to_string())?;
        for p in &paths {
            if !allowed.remove(p) {
                return Err(format!(
                    "Path not allowed (not from a drop/picker event): {p}"
                ));
            }
        }
    }

    let mut results = Vec::new();

    for path_str in &paths {
        let path = std::path::Path::new(path_str);

        // Skip directories
        if path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let content = match std::fs::read(path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("read_dropped_files: failed to read {}: {e}", path_str);
                continue;
            }
        };

        if content.len() > MAX_FILE_SIZE {
            log::warn!(
                "read_dropped_files: {} is too large ({} bytes)",
                name,
                content.len()
            );
            continue;
        }

        let content_type = guess_content_type(&name);
        results.push(ChatFileData {
            name,
            content_type,
            size: content.len(),
            content_base64: base64::engine::general_purpose::STANDARD.encode(&content),
        });
    }

    Ok(results)
}
