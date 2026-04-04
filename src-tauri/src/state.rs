//! Application-level managed state.

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Shared application state managed by Tauri.
#[allow(dead_code)]
pub struct AppState {
    /// SQLite database connection (wrapped in Mutex for thread safety).
    pub db: Mutex<Connection>,
}

impl AppState {
    /// Initialize application state, including the SQLite database.
    pub fn new(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app.path().app_data_dir()?;

        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("copilot-desktop.db");
        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        crate::db::initialize(&conn)?;

        log::info!("Database opened at {}", db_path.display());

        Ok(Self {
            db: Mutex::new(conn),
        })
    }
}
