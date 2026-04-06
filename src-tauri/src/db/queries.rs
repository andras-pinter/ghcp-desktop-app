//! Typed query functions for database operations.
//!
//! All functions take a `&Connection` and return `Result<T, rusqlite::Error>`.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// ── Types ───────────────────────────────────────────────────────

/// A conversation (matches frontend `Conversation` type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: Option<String>,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub model: Option<String>,
    pub is_favourite: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A single message in a conversation (matches frontend `Message` type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub thinking_content: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub attachments: Option<String>,
    pub created_at: String,
    pub sort_order: i64,
}

/// A draft (unsent input text for crash recovery).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub conversation_id: String,
    pub content: String,
    pub updated_at: String,
}

// ── Conversations ───────────────────────────────────────────────

/// List all conversations ordered by most recently updated.
pub fn list_conversations(conn: &Connection) -> Result<Vec<Conversation>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, title, agent_id, project_id, model, is_favourite, created_at, updated_at
         FROM conversations ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            agent_id: row.get(2)?,
            project_id: row.get(3)?,
            model: row.get(4)?,
            is_favourite: row.get::<_, i64>(5)? != 0,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

/// Get a single conversation by ID.
pub fn get_conversation(
    conn: &Connection,
    id: &str,
) -> Result<Option<Conversation>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, title, agent_id, project_id, model, is_favourite, created_at, updated_at
         FROM conversations WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            agent_id: row.get(2)?,
            project_id: row.get(3)?,
            model: row.get(4)?,
            is_favourite: row.get::<_, i64>(5)? != 0,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Create a new conversation. Returns the created conversation.
pub fn create_conversation(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    agent_id: Option<&str>,
    project_id: Option<&str>,
    model: Option<&str>,
) -> Result<Conversation, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO conversations (id, title, agent_id, project_id, model, is_favourite, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
        params![id, title, agent_id, project_id, model, now],
    )?;
    // Return the inserted row
    Ok(Conversation {
        id: id.to_string(),
        title: title.map(String::from),
        agent_id: agent_id.map(String::from),
        project_id: project_id.map(String::from),
        model: model.map(String::from),
        is_favourite: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Update a conversation's mutable fields.
pub fn update_conversation(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    is_favourite: Option<bool>,
    model: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    // Build dynamic SET clauses
    if let Some(t) = title {
        conn.execute(
            "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![t, now, id],
        )?;
    }
    if let Some(fav) = is_favourite {
        conn.execute(
            "UPDATE conversations SET is_favourite = ?1, updated_at = ?2 WHERE id = ?3",
            params![fav as i64, now, id],
        )?;
    }
    if let Some(m) = model {
        conn.execute(
            "UPDATE conversations SET model = ?1, updated_at = ?2 WHERE id = ?3",
            params![m, now, id],
        )?;
    }
    Ok(())
}

/// Touch conversation's updated_at timestamp.
pub fn touch_conversation(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

/// Delete a conversation (messages cascade via FK).
pub fn delete_conversation(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Messages ────────────────────────────────────────────────────

/// Get all messages for a conversation, ordered by sort_order.
pub fn get_messages(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, thinking_content, tool_call_id,
                tool_name, attachments, created_at, sort_order
         FROM messages WHERE conversation_id = ?1 ORDER BY sort_order ASC",
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            thinking_content: row.get(4)?,
            tool_call_id: row.get(5)?,
            tool_name: row.get(6)?,
            attachments: row.get(7)?,
            created_at: row.get(8)?,
            sort_order: row.get(9)?,
        })
    })?;
    rows.collect()
}

/// Insert a new message.
pub fn create_message(conn: &Connection, msg: &Message) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, thinking_content,
         tool_call_id, tool_name, attachments, created_at, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            msg.id,
            msg.conversation_id,
            msg.role,
            msg.content,
            msg.thinking_content,
            msg.tool_call_id,
            msg.tool_name,
            msg.attachments,
            msg.created_at,
            msg.sort_order,
        ],
    )?;
    Ok(())
}

/// Update a message's content (used when streaming completes or user edits).
pub fn update_message_content(
    conn: &Connection,
    id: &str,
    content: &str,
    thinking_content: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE messages SET content = ?1, thinking_content = ?2 WHERE id = ?3",
        params![content, thinking_content, id],
    )?;
    Ok(())
}

/// Delete all messages in a conversation with sort_order > the given value.
/// Used when the user edits a message (discard everything after it).
pub fn delete_messages_after(
    conn: &Connection,
    conversation_id: &str,
    after_sort_order: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1 AND sort_order > ?2",
        params![conversation_id, after_sort_order],
    )?;
    Ok(())
}

/// Get the next sort_order value for a conversation.
#[allow(dead_code)]
pub fn next_sort_order(conn: &Connection, conversation_id: &str) -> Result<i64, rusqlite::Error> {
    let max: Option<i64> = conn.query_row(
        "SELECT MAX(sort_order) FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
        |row| row.get(0),
    )?;
    Ok(max.unwrap_or(-1) + 1)
}

// ── Drafts ──────────────────────────────────────────────────────

/// Save or update a draft for a conversation.
pub fn save_draft(
    conn: &Connection,
    conversation_id: &str,
    content: &str,
) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO drafts (conversation_id, content, updated_at)
         VALUES (?1, ?2, ?3)",
        params![conversation_id, content, now],
    )?;
    Ok(())
}

/// Get the draft for a conversation, if any.
pub fn get_draft(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Option<Draft>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT conversation_id, content, updated_at FROM drafts WHERE conversation_id = ?1",
    )?;
    let mut rows = stmt.query_map(params![conversation_id], |row| {
        Ok(Draft {
            conversation_id: row.get(0)?,
            content: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Delete the draft for a conversation.
pub fn delete_draft(conn: &Connection, conversation_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM drafts WHERE conversation_id = ?1",
        params![conversation_id],
    )?;
    Ok(())
}

// ── Settings ────────────────────────────────────────────────────

/// Get a config value by key.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
    match rows.next() {
        Some(val) => Ok(Some(val?)),
        None => Ok(None),
    }
}

/// Set a config value (insert or replace).
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

/// Get the database file size in bytes (requires the path, not the connection).
pub fn get_db_size(db_path: &std::path::Path) -> Result<u64, std::io::Error> {
    let metadata = std::fs::metadata(db_path)?;
    Ok(metadata.len())
}

// ── MCP Servers ─────────────────────────────────────────────────

/// An MCP server configuration row (matches `mcp_servers` table).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerRow {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub url: Option<String>,
    pub binary_path: Option<String>,
    pub args: Option<String>,
    pub auth_header: Option<String>,
    pub from_catalog: bool,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// List all MCP servers.
pub fn get_mcp_servers(conn: &Connection) -> Result<Vec<McpServerRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, transport, url, binary_path, args, auth_header,
                from_catalog, enabled, created_at, updated_at
         FROM mcp_servers
         ORDER BY name ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(McpServerRow {
                id: row.get(0)?,
                name: row.get(1)?,
                transport: row.get(2)?,
                url: row.get(3)?,
                binary_path: row.get(4)?,
                args: row.get(5)?,
                auth_header: row.get(6)?,
                from_catalog: row.get::<_, i64>(7)? != 0,
                enabled: row.get::<_, i64>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Get a single MCP server by ID.
pub fn get_mcp_server(
    conn: &Connection,
    id: &str,
) -> Result<Option<McpServerRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, transport, url, binary_path, args, auth_header,
                from_catalog, enabled, created_at, updated_at
         FROM mcp_servers WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(McpServerRow {
            id: row.get(0)?,
            name: row.get(1)?,
            transport: row.get(2)?,
            url: row.get(3)?,
            binary_path: row.get(4)?,
            args: row.get(5)?,
            auth_header: row.get(6)?,
            from_catalog: row.get::<_, i64>(7)? != 0,
            enabled: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?;
    rows.next().transpose()
}

/// Get only enabled MCP servers.
pub fn get_enabled_mcp_servers(conn: &Connection) -> Result<Vec<McpServerRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, transport, url, binary_path, args, auth_header,
                from_catalog, enabled, created_at, updated_at
         FROM mcp_servers WHERE enabled = 1
         ORDER BY name ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(McpServerRow {
                id: row.get(0)?,
                name: row.get(1)?,
                transport: row.get(2)?,
                url: row.get(3)?,
                binary_path: row.get(4)?,
                args: row.get(5)?,
                auth_header: row.get(6)?,
                from_catalog: row.get::<_, i64>(7)? != 0,
                enabled: row.get::<_, i64>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Insert a new MCP server.
pub fn insert_mcp_server(conn: &Connection, server: &McpServerRow) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO mcp_servers (id, name, transport, url, binary_path, args,
                                  auth_header, from_catalog, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            server.id,
            server.name,
            server.transport,
            server.url,
            server.binary_path,
            server.args,
            server.auth_header,
            server.from_catalog as i64,
            server.enabled as i64,
            server.created_at,
            server.updated_at,
        ],
    )?;
    Ok(())
}

/// Update an existing MCP server.
pub fn update_mcp_server(conn: &Connection, server: &McpServerRow) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE mcp_servers SET name = ?2, transport = ?3, url = ?4,
                binary_path = ?5, args = ?6, auth_header = ?7,
                from_catalog = ?8, enabled = ?9, updated_at = ?10
         WHERE id = ?1",
        params![
            server.id,
            server.name,
            server.transport,
            server.url,
            server.binary_path,
            server.args,
            server.auth_header,
            server.from_catalog as i64,
            server.enabled as i64,
            server.updated_at,
        ],
    )?;
    Ok(())
}

/// Delete an MCP server by ID.
pub fn delete_mcp_server(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM mcp_servers WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        migrations::run(&conn, 0).unwrap();
        conn
    }

    #[test]
    fn test_conversation_crud() {
        let conn = setup_db();

        // Create
        let conv = create_conversation(&conn, "c1", Some("Test Chat"), None, None, Some("gpt-4o"))
            .unwrap();
        assert_eq!(conv.id, "c1");
        assert_eq!(conv.title, Some("Test Chat".to_string()));
        assert!(!conv.is_favourite);

        // List
        let convos = list_conversations(&conn).unwrap();
        assert_eq!(convos.len(), 1);
        assert_eq!(convos[0].id, "c1");

        // Get
        let found = get_conversation(&conn, "c1").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, Some("Test Chat".to_string()));

        // Update
        update_conversation(&conn, "c1", Some("Renamed"), Some(true), None).unwrap();
        let updated = get_conversation(&conn, "c1").unwrap().unwrap();
        assert_eq!(updated.title, Some("Renamed".to_string()));
        assert!(updated.is_favourite);

        // Delete
        delete_conversation(&conn, "c1").unwrap();
        let gone = get_conversation(&conn, "c1").unwrap();
        assert!(gone.is_none());
    }

    #[test]
    fn test_message_crud() {
        let conn = setup_db();
        create_conversation(&conn, "c1", Some("Test"), None, None, None).unwrap();

        let msg = Message {
            id: "m1".to_string(),
            conversation_id: "c1".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            thinking_content: None,
            tool_call_id: None,
            tool_name: None,
            attachments: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            sort_order: 0,
        };
        create_message(&conn, &msg).unwrap();

        let msgs = get_messages(&conn, "c1").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "Hello");

        // Update content
        update_message_content(&conn, "m1", "Hello, world!", None).unwrap();
        let msgs = get_messages(&conn, "c1").unwrap();
        assert_eq!(msgs[0].content, "Hello, world!");

        // Next sort order
        let next = next_sort_order(&conn, "c1").unwrap();
        assert_eq!(next, 1);

        // Delete after
        let msg2 = Message {
            id: "m2".to_string(),
            sort_order: 1,
            role: "assistant".to_string(),
            content: "Hi!".to_string(),
            ..msg.clone()
        };
        create_message(&conn, &msg2).unwrap();
        delete_messages_after(&conn, "c1", 0).unwrap();
        let msgs = get_messages(&conn, "c1").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, "m1");
    }

    #[test]
    fn test_draft_crud() {
        let conn = setup_db();
        create_conversation(&conn, "c1", Some("Test"), None, None, None).unwrap();

        // No draft initially
        assert!(get_draft(&conn, "c1").unwrap().is_none());

        // Save
        save_draft(&conn, "c1", "work in progress").unwrap();
        let draft = get_draft(&conn, "c1").unwrap().unwrap();
        assert_eq!(draft.content, "work in progress");

        // Overwrite
        save_draft(&conn, "c1", "updated draft").unwrap();
        let draft = get_draft(&conn, "c1").unwrap().unwrap();
        assert_eq!(draft.content, "updated draft");

        // Delete
        delete_draft(&conn, "c1").unwrap();
        assert!(get_draft(&conn, "c1").unwrap().is_none());
    }

    #[test]
    fn test_settings() {
        let conn = setup_db();

        // Schema version was seeded
        let ver = get_setting(&conn, "schema_version").unwrap();
        assert_eq!(ver, Some("1".to_string()));

        // Set new value
        set_setting(&conn, "theme", "dark").unwrap();
        assert_eq!(
            get_setting(&conn, "theme").unwrap(),
            Some("dark".to_string())
        );

        // Update
        set_setting(&conn, "theme", "light").unwrap();
        assert_eq!(
            get_setting(&conn, "theme").unwrap(),
            Some("light".to_string())
        );

        // Non-existent key
        assert!(get_setting(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_cascade_delete_messages() {
        let conn = setup_db();
        create_conversation(&conn, "c1", Some("Test"), None, None, None).unwrap();

        let msg = Message {
            id: "m1".to_string(),
            conversation_id: "c1".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            thinking_content: None,
            tool_call_id: None,
            tool_name: None,
            attachments: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            sort_order: 0,
        };
        create_message(&conn, &msg).unwrap();

        // Deleting conversation should cascade to messages
        delete_conversation(&conn, "c1").unwrap();
        let msgs = get_messages(&conn, "c1").unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_cascade_delete_drafts() {
        let conn = setup_db();
        create_conversation(&conn, "c1", Some("Test"), None, None, None).unwrap();
        save_draft(&conn, "c1", "draft text").unwrap();

        // Deleting conversation should cascade to drafts
        delete_conversation(&conn, "c1").unwrap();
        assert!(get_draft(&conn, "c1").unwrap().is_none());
    }

    #[test]
    fn test_mcp_server_crud() {
        let conn = setup_db();
        let now = "2025-01-01T00:00:00Z";

        let server = McpServerRow {
            id: "s1".to_string(),
            name: "Test Server".to_string(),
            transport: "http".to_string(),
            url: Some("https://example.com/mcp".to_string()),
            binary_path: None,
            args: None,
            auth_header: None,
            from_catalog: false,
            enabled: true,
            created_at: now.to_string(),
            updated_at: now.to_string(),
        };

        // Insert
        insert_mcp_server(&conn, &server).unwrap();

        // Get
        let found = get_mcp_server(&conn, "s1").unwrap().unwrap();
        assert_eq!(found.name, "Test Server");
        assert_eq!(found.transport, "http");
        assert!(found.enabled);

        // List
        let all = get_mcp_servers(&conn).unwrap();
        assert_eq!(all.len(), 1);

        // Update
        let mut updated = server.clone();
        updated.name = "Renamed".to_string();
        updated.enabled = false;
        update_mcp_server(&conn, &updated).unwrap();
        let found2 = get_mcp_server(&conn, "s1").unwrap().unwrap();
        assert_eq!(found2.name, "Renamed");
        assert!(!found2.enabled);

        // Enabled filter
        let enabled = get_enabled_mcp_servers(&conn).unwrap();
        assert!(enabled.is_empty());

        // Delete
        delete_mcp_server(&conn, "s1").unwrap();
        assert!(get_mcp_server(&conn, "s1").unwrap().is_none());
    }
}
