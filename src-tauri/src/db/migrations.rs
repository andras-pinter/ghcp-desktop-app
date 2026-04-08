//! Database schema migrations.

use rusqlite::Connection;

/// Run all pending migrations from `current_version` to the latest.
pub fn run(conn: &Connection, current_version: i32) -> Result<(), Box<dyn std::error::Error>> {
    if current_version < 1 {
        migrate_v1(conn)?;
    }
    if current_version < 2 {
        migrate_v2(conn)?;
    }
    if current_version < 3 {
        migrate_v3(conn)?;
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

/// Version 2: Add skill instructions, source tracking, and updated_at to skills.
fn migrate_v2(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "
        -- Skills: add instructions body (markdown from SKILL.md)
        ALTER TABLE skills ADD COLUMN instructions TEXT;

        -- Skills: track where the skill was imported from
        ALTER TABLE skills ADD COLUMN source_url TEXT;

        -- Skills: source type discriminator
        ALTER TABLE skills ADD COLUMN source_type TEXT DEFAULT 'builtin';

        -- Skills: add updated_at for edit tracking
        ALTER TABLE skills ADD COLUMN updated_at TEXT;

        -- Agents: track where the agent template came from
        ALTER TABLE agents ADD COLUMN source_url TEXT;

        -- Agents: source type discriminator
        ALTER TABLE agents ADD COLUMN source_type TEXT DEFAULT 'local';

        -- Bump schema version
        UPDATE config SET value = '2' WHERE key = 'schema_version';
        ",
    )?;

    log::info!("Database migrated to schema version 2");
    Ok(())
}

/// Version 3: Approved MCP binaries table + migrate auth headers to keychain.
fn migrate_v3(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Proactively move existing plaintext auth headers to OS keychain.
    // Only clear each row's auth_header after confirmed keychain storage.
    {
        let mut stmt =
            conn.prepare("SELECT id, auth_header FROM mcp_servers WHERE auth_header IS NOT NULL")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (server_id, auth_header) = match row {
                Ok(pair) => pair,
                Err(e) => {
                    log::warn!("Skipping unreadable mcp_servers row during migration: {e}");
                    continue;
                }
            };
            let key = format!("mcp_auth_{server_id}");
            match copilot_api::keychain::store(&key, &auth_header) {
                Ok(()) => {
                    // Keychain write succeeded — safe to clear this row's plaintext auth
                    conn.execute(
                        "UPDATE mcp_servers SET auth_header = NULL WHERE id = ?1",
                        [&server_id],
                    )?;
                }
                Err(e) => {
                    // Keychain write failed — keep plaintext auth as fallback for this server
                    log::warn!("Failed to migrate MCP auth for {server_id} to keychain: {e}. Keeping plaintext auth as fallback.");
                }
            }
        }
    }

    conn.execute_batch(
        "
        -- Approved MCP stdio binaries (user must approve before first launch)
        CREATE TABLE IF NOT EXISTS approved_mcp_binaries (
            binary_path TEXT PRIMARY KEY,
            approved_at TEXT NOT NULL
        );

        -- Bump schema version
        UPDATE config SET value = '3' WHERE key = 'schema_version';
        ",
    )?;

    log::info!("Database migrated to schema version 3");
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
        migrate_v1(&conn).unwrap();

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
    fn test_full_migration() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn, 0).unwrap();

        // Verify schema version is now 3
        let version: String = conn
            .query_row(
                "SELECT value FROM config WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "3");

        // Verify new skill columns exist
        conn.execute(
            "INSERT INTO skills (id, name, source, instructions, source_url, source_type, updated_at, created_at)
             VALUES ('test', 'Test Skill', 'registry_aitmpl', 'some instructions', 'https://example.com', 'registry_aitmpl', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let instructions: Option<String> = conn
            .query_row(
                "SELECT instructions FROM skills WHERE id = 'test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(instructions, Some("some instructions".to_string()));

        // Verify new agent columns exist
        let source_type: Option<String> = conn
            .query_row(
                "SELECT source_type FROM agents WHERE is_default = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(source_type, Some("local".to_string()));
    }

    #[test]
    fn test_idempotent_migration() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn, 0).unwrap();
        // Running v1 again should not fail (IF NOT EXISTS)
        migrate_v1(&conn).unwrap();
        // v2 ALTER TABLE would fail on re-run, but run() skips it via version check
    }

    #[test]
    fn test_incremental_migration() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        // Simulate existing v1 database
        run(&conn, 0).unwrap();

        // Now simulate upgrading from v1 to latest
        let conn2 = Connection::open_in_memory().unwrap();
        conn2.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        migrate_v1(&conn2).unwrap();
        // Run from v1 should apply v2 and v3
        run(&conn2, 1).unwrap();

        let version: String = conn2
            .query_row(
                "SELECT value FROM config WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "3");
    }
}
