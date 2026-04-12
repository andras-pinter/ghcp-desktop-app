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
    if current_version < 4 {
        migrate_v4(conn)?;
    }
    if current_version < 5 {
        migrate_v5(conn)?;
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

/// Version 4: Git sources table + link skills/agents to sources.
fn migrate_v4(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "
        -- Git repository sources for skills and agents
        CREATE TABLE IF NOT EXISTS git_sources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL UNIQUE,
            enabled INTEGER DEFAULT 1,
            last_synced_at TEXT,
            item_count INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_git_sources_enabled ON git_sources(enabled);

        -- Link skills to their git source
        ALTER TABLE skills ADD COLUMN git_source_id TEXT REFERENCES git_sources(id) ON DELETE SET NULL;

        -- Link agents to their git source
        ALTER TABLE agents ADD COLUMN git_source_id TEXT REFERENCES git_sources(id) ON DELETE SET NULL;
        ",
    )?;

    // Back-fill: create git sources from existing git-imported items.
    // Parse base repo URL from source_url (strip /blob/main/... suffix).
    backfill_git_sources(conn)?;

    conn.execute(
        "UPDATE config SET value = '4' WHERE key = 'schema_version'",
        [],
    )?;

    log::info!("Database migrated to schema version 4");
    Ok(())
}

/// Back-fill git_source_id for existing items with source_type = 'git'.
///
/// Extracts the base repository URL from the per-file source_url (which has
/// format `https://github.com/owner/repo/blob/main/path`), creates a
/// `git_sources` row for each unique repo, then links the items.
fn backfill_git_sources(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    // Collect unique repo URLs from skills
    let mut repo_map: HashMap<String, String> = HashMap::new(); // repo_url -> source_id

    let mut stmt = conn.prepare(
        "SELECT id, source_url FROM skills WHERE source_type = 'git' AND source_url IS NOT NULL",
    )?;
    let skill_rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt = conn.prepare(
        "SELECT id, source_url FROM agents WHERE source_type = 'git' AND source_url IS NOT NULL",
    )?;
    let agent_rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    // Extract base repo URLs and create sources
    for (_item_id, source_url) in skill_rows.iter().chain(agent_rows.iter()) {
        let repo_url = extract_repo_url(source_url);
        if let std::collections::hash_map::Entry::Vacant(entry) = repo_map.entry(repo_url.clone()) {
            let source_id = format!("migrated-{}", uuid_v4());
            let name = repo_name_from_url(&repo_url);
            conn.execute(
                "INSERT OR IGNORE INTO git_sources (id, name, url, enabled, item_count, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 1, 0, datetime('now'), datetime('now'))",
                rusqlite::params![source_id, name, repo_url],
            )?;
            entry.insert(source_id);
        }
    }

    // Link skills to their source
    for (item_id, source_url) in &skill_rows {
        let repo_url = extract_repo_url(source_url);
        if let Some(source_id) = repo_map.get(&repo_url) {
            conn.execute(
                "UPDATE skills SET git_source_id = ?1 WHERE id = ?2",
                rusqlite::params![source_id, item_id],
            )?;
        }
    }

    // Link agents to their source
    for (item_id, source_url) in &agent_rows {
        let repo_url = extract_repo_url(source_url);
        if let Some(source_id) = repo_map.get(&repo_url) {
            conn.execute(
                "UPDATE agents SET git_source_id = ?1 WHERE id = ?2",
                rusqlite::params![source_id, item_id],
            )?;
        }
    }

    // Update item counts
    for (repo_url, source_id) in &repo_map {
        let count: i64 = conn.query_row(
            "SELECT (SELECT COUNT(*) FROM skills WHERE git_source_id = ?1) +
                    (SELECT COUNT(*) FROM agents WHERE git_source_id = ?1)",
            rusqlite::params![source_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE git_sources SET item_count = ?1 WHERE id = ?2",
            rusqlite::params![count, source_id],
        )?;
        log::info!("Back-filled git source '{}' with {} items", repo_url, count);
    }

    Ok(())
}

/// Extract base repository URL from a per-file source_url.
/// e.g., `https://github.com/user/repo/blob/main/skills/foo.md` → `https://github.com/user/repo`
fn extract_repo_url(source_url: &str) -> String {
    // Handle GitHub/GitLab blob URLs: cut at /blob/ or /tree/
    if let Some(pos) = source_url.find("/blob/") {
        return source_url[..pos].to_string();
    }
    if let Some(pos) = source_url.find("/tree/") {
        return source_url[..pos].to_string();
    }
    // Handle raw URLs: https://raw.githubusercontent.com/owner/repo/branch/...
    if source_url.contains("raw.githubusercontent.com") {
        let parts: Vec<&str> = source_url.splitn(6, '/').collect();
        if parts.len() >= 5 {
            return format!("https://github.com/{}/{}", parts[3], parts[4]);
        }
    }
    // Fallback: return as-is
    source_url.to_string()
}

/// Derive a human-readable name from a repo URL.
/// e.g., `https://github.com/user/awesome-skills` → `awesome-skills`
fn repo_name_from_url(repo_url: &str) -> String {
    repo_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

/// Generate a UUID v4 string (simple implementation for migration use).
/// Uses nanosecond timestamp combined with a static counter to ensure
/// uniqueness even when called in rapid succession.
fn uuid_v4() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:024x}{:08x}", seed, count)
}

/// Version 5: Catalog items table for persisting scanned git source contents.
fn migrate_v5(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS git_source_items (
            id TEXT PRIMARY KEY,
            git_source_id TEXT NOT NULL REFERENCES git_sources(id) ON DELETE CASCADE,
            path TEXT NOT NULL,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(git_source_id, path)
        );

        CREATE INDEX IF NOT EXISTS idx_git_source_items_source ON git_source_items(git_source_id);
        CREATE INDEX IF NOT EXISTS idx_git_source_items_kind ON git_source_items(kind);

        UPDATE config SET value = '5' WHERE key = 'schema_version';
        ",
    )?;

    log::info!("Database migrated to schema version 5");
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

        // Verify schema version is now 5
        let version: String = conn
            .query_row(
                "SELECT value FROM config WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "5");

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
        // Run from v1 should apply v2, v3, v4, and v5
        run(&conn2, 1).unwrap();

        let version: String = conn2
            .query_row(
                "SELECT value FROM config WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "5");

        // Verify git_sources table exists
        let table_exists: bool = conn2
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='git_sources'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists);

        // Verify git_source_id column exists on skills
        conn2
            .execute(
                "INSERT INTO skills (id, name, source, created_at, git_source_id)
                 VALUES ('s1', 'Test', 'custom', datetime('now'), NULL)",
                [],
            )
            .unwrap();
    }

    #[test]
    fn test_v4_backfill_git_sources() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        // Run up to v3
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();

        // Insert git-imported skills with source_url
        conn.execute(
            "INSERT INTO skills (id, name, source, source_url, source_type, created_at)
             VALUES ('s1', 'Skill A', 'git', 'https://github.com/user/repo/blob/main/skills/a.md', 'git', datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO skills (id, name, source, source_url, source_type, created_at)
             VALUES ('s2', 'Skill B', 'git', 'https://github.com/user/repo/blob/main/skills/b.md', 'git', datetime('now'))",
            [],
        )
        .unwrap();
        // Different repo
        conn.execute(
            "INSERT INTO skills (id, name, source, source_url, source_type, created_at)
             VALUES ('s3', 'Skill C', 'git', 'https://github.com/other/tools/blob/main/skill.md', 'git', datetime('now'))",
            [],
        )
        .unwrap();
        // Non-git skill (should not be affected)
        conn.execute(
            "INSERT INTO skills (id, name, source, source_type, created_at)
             VALUES ('s4', 'Local Skill', 'custom', 'builtin', datetime('now'))",
            [],
        )
        .unwrap();

        // Run v4 migration
        migrate_v4(&conn).unwrap();

        // Should have created 2 git sources (one per unique repo)
        let source_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM git_sources", [], |row| row.get(0))
            .unwrap();
        assert_eq!(source_count, 2);

        // s1 and s2 should share the same git_source_id
        let s1_source: Option<String> = conn
            .query_row(
                "SELECT git_source_id FROM skills WHERE id = 's1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let s2_source: Option<String> = conn
            .query_row(
                "SELECT git_source_id FROM skills WHERE id = 's2'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(s1_source.is_some());
        assert_eq!(s1_source, s2_source);

        // s3 should have a different git_source_id
        let s3_source: Option<String> = conn
            .query_row(
                "SELECT git_source_id FROM skills WHERE id = 's3'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(s3_source.is_some());
        assert_ne!(s1_source, s3_source);

        // s4 (non-git) should have no git_source_id
        let s4_source: Option<String> = conn
            .query_row(
                "SELECT git_source_id FROM skills WHERE id = 's4'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(s4_source.is_none());

        // Verify item counts
        let repo_source = conn
            .query_row(
                "SELECT item_count FROM git_sources WHERE url = 'https://github.com/user/repo'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(repo_source, 2); // s1 + s2

        let other_source = conn
            .query_row(
                "SELECT item_count FROM git_sources WHERE url = 'https://github.com/other/tools'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(other_source, 1); // s3
    }

    #[test]
    fn test_extract_repo_url() {
        assert_eq!(
            extract_repo_url("https://github.com/user/repo/blob/main/skills/a.md"),
            "https://github.com/user/repo"
        );
        assert_eq!(
            extract_repo_url("https://gitlab.com/org/project/blob/master/agents/bot.md"),
            "https://gitlab.com/org/project"
        );
        assert_eq!(
            extract_repo_url("https://github.com/user/repo/tree/main/directory"),
            "https://github.com/user/repo"
        );
        assert_eq!(
            extract_repo_url("https://github.com/user/repo"),
            "https://github.com/user/repo"
        );
    }
}
