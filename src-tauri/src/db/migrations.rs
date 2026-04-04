//! Database schema migrations.

use rusqlite::Connection;

/// Run all pending migrations from `current_version` to the latest.
pub fn run(conn: &Connection, current_version: i32) -> Result<(), Box<dyn std::error::Error>> {
    if current_version < 1 {
        migrate_v1(conn)?;
    }
    Ok(())
}

/// Version 1: Initial schema.
fn migrate_v1(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "
        -- Conversations
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT,
            agent_id TEXT,
            project_id TEXT,
            model TEXT,
            is_favourite INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Messages
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            thinking_content TEXT,
            tool_call_id TEXT,
            tool_name TEXT,
            attachments TEXT,
            created_at TEXT NOT NULL,
            sort_order INTEGER NOT NULL
        );

        -- Projects
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            instructions TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Project files
        CREATE TABLE IF NOT EXISTS project_files (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            content_type TEXT NOT NULL,
            content BLOB NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Agents
        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            avatar TEXT,
            system_prompt TEXT NOT NULL,
            is_default INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Agent-Skill assignments
        CREATE TABLE IF NOT EXISTS agent_skills (
            agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            skill_id TEXT NOT NULL,
            PRIMARY KEY (agent_id, skill_id)
        );

        -- Agent-MCP connections
        CREATE TABLE IF NOT EXISTS agent_mcp_connections (
            agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            mcp_server_id TEXT NOT NULL,
            PRIMARY KEY (agent_id, mcp_server_id)
        );

        -- Skills
        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            source TEXT NOT NULL,
            mcp_server_id TEXT,
            config TEXT,
            enabled INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        );

        -- MCP servers
        CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            transport TEXT NOT NULL,
            url TEXT,
            binary_path TEXT,
            args TEXT,
            auth_header TEXT,
            from_catalog INTEGER DEFAULT 0,
            enabled INTEGER DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- User preferences
        CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Draft auto-save
        CREATE TABLE IF NOT EXISTS drafts (
            conversation_id TEXT PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Foreign key references (added after table creation)
        -- conversations.agent_id -> agents.id (soft reference, no FK constraint to allow orphans)
        -- conversations.project_id -> projects.id (soft reference)

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, sort_order);
        CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_conversations_project ON conversations(project_id);
        CREATE INDEX IF NOT EXISTS idx_conversations_agent ON conversations(agent_id);
        CREATE INDEX IF NOT EXISTS idx_conversations_favourite ON conversations(is_favourite) WHERE is_favourite = 1;
        CREATE INDEX IF NOT EXISTS idx_project_files_project ON project_files(project_id);
        CREATE INDEX IF NOT EXISTS idx_agent_skills_agent ON agent_skills(agent_id);
        CREATE INDEX IF NOT EXISTS idx_skills_source ON skills(source);

        -- Seed data
        INSERT OR IGNORE INTO config (key, value) VALUES ('schema_version', '1');
        INSERT OR IGNORE INTO agents (id, name, avatar, system_prompt, is_default, created_at, updated_at)
            VALUES ('default', 'Default', '🤖', 'You are a helpful AI assistant powered by GitHub Copilot.', 1, datetime('now'), datetime('now'));
        ",
    )?;

    log::info!("Database migrated to schema version 1");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_v1() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn, 0).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"conversations".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"agents".to_string()));
        assert!(tables.contains(&"config".to_string()));

        // Verify schema version
        let version: String = conn
            .query_row(
                "SELECT value FROM config WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "1");

        // Verify default agent
        let agent_name: String = conn
            .query_row("SELECT name FROM agents WHERE is_default=1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(agent_name, "Default");
    }

    #[test]
    fn test_idempotent_migration() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn, 0).unwrap();
        // Running again should not fail
        run(&conn, 0).unwrap();
    }
}
