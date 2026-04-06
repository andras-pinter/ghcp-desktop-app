//! Application-level managed state.

use copilot_api::CopilotClient;
use mcp_client::McpManager;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex as TokioMutex;

/// Shared application state managed by Tauri.
pub struct AppState {
    /// SQLite database connection (wrapped in Mutex for thread safety).
    /// Used by Phase 3+ for conversation persistence, settings, etc.
    #[allow(dead_code)]
    pub db: Mutex<Connection>,
    /// Copilot API client (handles auth + streaming + models).
    pub copilot: CopilotClient,
    /// Flag to cancel an in-flight streaming response.
    pub cancel_stream: TokioMutex<Option<tokio::sync::watch::Sender<bool>>>,
    /// Shared HTTP client for web research (hardened with SSRF protection).
    pub http_client: web_research::HttpClient,
    /// MCP connection manager (handles multiple server connections).
    pub mcp: McpManager,
}

impl AppState {
    /// Initialize application state, including the SQLite database.
    pub fn new(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app.path().app_data_dir()?;

        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("chuck.db");
        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        crate::db::initialize(&conn)?;

        log::info!("Database opened at {}", db_path.display());

        Ok(Self {
            db: Mutex::new(conn),
            copilot: CopilotClient::new(),
            cancel_stream: TokioMutex::new(None),
            http_client: web_research::new_client()?,
            mcp: McpManager::new(),
        })
    }
}
