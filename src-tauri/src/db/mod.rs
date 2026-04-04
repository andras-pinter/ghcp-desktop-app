//! Database initialization and management.

pub mod migrations;
pub mod queries;

use rusqlite::Connection;

/// Initialize the database schema. Runs migrations if needed.
pub fn initialize(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = get_schema_version(conn);
    migrations::run(conn, current_version)?;
    Ok(())
}

/// Get the current schema version from the config table.
fn get_schema_version(conn: &Connection) -> i32 {
    conn.query_row(
        "SELECT value FROM config WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse::<i32>().ok())
    .unwrap_or(0)
}
