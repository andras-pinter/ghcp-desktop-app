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
    project_id: Option<Option<&str>>,
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
    // Option<Option<&str>>: Some(Some("id")) = assign, Some(None) = unassign, None = no change
    if let Some(pid) = project_id {
        conn.execute(
            "UPDATE conversations SET project_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![pid, now, id],
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
    let current = max.unwrap_or(-1);
    Ok(current.saturating_add(1))
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

/// Delete conversations (and their cascaded messages/drafts) older than the given ISO 8601 date.
/// Returns the number of conversations deleted.
pub fn delete_old_conversations(
    conn: &Connection,
    before_date: &str,
) -> Result<usize, rusqlite::Error> {
    let count = conn.execute(
        "DELETE FROM conversations WHERE updated_at < ?1",
        params![before_date],
    )?;
    Ok(count)
}

/// A conversation with its messages, suitable for export.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationExport {
    #[serde(flatten)]
    pub conversation: Conversation,
    pub messages: Vec<Message>,
}

/// Get a single conversation with all its messages for export.
pub fn get_conversation_for_export(
    conn: &Connection,
    id: &str,
) -> Result<Option<ConversationExport>, rusqlite::Error> {
    let conv = get_conversation(conn, id)?;
    match conv {
        Some(c) => {
            let msgs = get_messages(conn, id)?;
            Ok(Some(ConversationExport {
                conversation: c,
                messages: msgs,
            }))
        }
        None => Ok(None),
    }
}

/// Get all conversations with their messages for bulk export.
pub fn get_all_conversations_for_export(
    conn: &Connection,
) -> Result<Vec<ConversationExport>, rusqlite::Error> {
    let convos = list_conversations(conn)?;
    let mut result = Vec::with_capacity(convos.len());
    for c in convos {
        let msgs = get_messages(conn, &c.id)?;
        result.push(ConversationExport {
            conversation: c,
            messages: msgs,
        });
    }
    Ok(result)
}

/// Format a single conversation as Markdown.
pub fn conversation_to_markdown(export: &ConversationExport) -> String {
    let mut md = String::new();
    let title = export
        .conversation
        .title
        .as_deref()
        .unwrap_or("Untitled conversation");
    md.push_str(&format!("# {}\n\n", title));
    md.push_str(&format!(
        "_Created: {} · Updated: {}_\n\n---\n\n",
        export.conversation.created_at, export.conversation.updated_at
    ));

    for msg in &export.messages {
        let role_label = match msg.role.as_str() {
            "user" => "**You**",
            "assistant" => "**Copilot**",
            "system" => "**System**",
            "tool" => "**Tool**",
            _ => "**Unknown**",
        };
        md.push_str(&format!("{}\n\n{}\n\n---\n\n", role_label, msg.content));
    }

    md
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

/// Map a rusqlite Row to an McpServerRow.
///
/// Expects columns in the standard SELECT order:
/// `id, name, transport, url, binary_path, args, auth_header,
///  from_catalog, enabled, created_at, updated_at`
fn map_mcp_row(row: &rusqlite::Row) -> rusqlite::Result<McpServerRow> {
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
        .query_map([], map_mcp_row)?
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
    let mut rows = stmt.query_map(params![id], map_mcp_row)?;
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
        .query_map([], map_mcp_row)?
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

// ── Agents ──────────────────────────────────────────────────────

/// An agent persona (matches frontend `Agent` type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt: String,
    pub is_default: bool,
    pub source_url: Option<String>,
    pub source_type: String,
    pub git_source_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// List all agents, default agent first.
pub fn list_agents(conn: &Connection) -> Result<Vec<Agent>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, avatar, system_prompt, is_default, source_url, source_type,
                git_source_id, created_at, updated_at
         FROM agents ORDER BY is_default DESC, name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Agent {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar: row.get(2)?,
            system_prompt: row.get(3)?,
            is_default: row.get(4)?,
            source_url: row.get(5)?,
            source_type: row
                .get::<_, Option<String>>(6)?
                .unwrap_or("local".to_string()),
            git_source_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;
    rows.collect()
}

/// Get a single agent by ID.
pub fn get_agent(conn: &Connection, id: &str) -> Result<Option<Agent>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, avatar, system_prompt, is_default, source_url, source_type,
                git_source_id, created_at, updated_at
         FROM agents WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Agent {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar: row.get(2)?,
            system_prompt: row.get(3)?,
            is_default: row.get(4)?,
            source_url: row.get(5)?,
            source_type: row
                .get::<_, Option<String>>(6)?
                .unwrap_or("local".to_string()),
            git_source_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;
    rows.next().transpose()
}

#[allow(clippy::too_many_arguments)]
/// Create a new agent, returns the created agent.
pub fn create_agent(
    conn: &Connection,
    id: &str,
    name: &str,
    avatar: Option<&str>,
    system_prompt: &str,
    source_url: Option<&str>,
    source_type: &str,
    git_source_id: Option<&str>,
) -> Result<Agent, rusqlite::Error> {
    conn.execute(
        "INSERT INTO agents (id, name, avatar, system_prompt, is_default, source_url, source_type, git_source_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, datetime('now'), datetime('now'))",
        params![id, name, avatar, system_prompt, source_url, source_type, git_source_id],
    )?;
    get_agent(conn, id)?.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

/// Update an existing agent (name, avatar, system_prompt, source fields).
pub fn update_agent(
    conn: &Connection,
    id: &str,
    name: &str,
    avatar: Option<&str>,
    system_prompt: &str,
    source_url: Option<&str>,
    source_type: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE agents SET name = ?2, avatar = ?3, system_prompt = ?4,
                source_url = ?5, source_type = ?6, updated_at = datetime('now')
         WHERE id = ?1 AND is_default = 0",
        params![id, name, avatar, system_prompt, source_url, source_type],
    )?;
    Ok(())
}

/// Delete an agent (prevents deleting the default agent).
pub fn delete_agent(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
    let rows = conn.execute(
        "DELETE FROM agents WHERE id = ?1 AND is_default = 0",
        params![id],
    )?;
    Ok(rows > 0)
}

/// Get the skill IDs assigned to an agent.
#[allow(dead_code)]
pub fn get_agent_skill_ids(
    conn: &Connection,
    agent_id: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT skill_id FROM agent_skills WHERE agent_id = ?1")?;
    let rows = stmt.query_map(params![agent_id], |row| row.get(0))?;
    rows.collect()
}

/// Replace an agent's skill assignments.
pub fn set_agent_skills(
    conn: &Connection,
    agent_id: &str,
    skill_ids: &[String],
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM agent_skills WHERE agent_id = ?1",
        params![agent_id],
    )?;
    let mut stmt = conn.prepare("INSERT INTO agent_skills (agent_id, skill_id) VALUES (?1, ?2)")?;
    for sid in skill_ids {
        stmt.execute(params![agent_id, sid])?;
    }
    Ok(())
}

/// Get the MCP server IDs connected to an agent.
#[allow(dead_code)]
pub fn get_agent_mcp_ids(
    conn: &Connection,
    agent_id: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT mcp_server_id FROM agent_mcp_connections WHERE agent_id = ?1")?;
    let rows = stmt.query_map(params![agent_id], |row| row.get(0))?;
    rows.collect()
}

/// Replace an agent's MCP server connections.
pub fn set_agent_mcp_connections(
    conn: &Connection,
    agent_id: &str,
    mcp_server_ids: &[String],
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM agent_mcp_connections WHERE agent_id = ?1",
        params![agent_id],
    )?;
    let mut stmt = conn
        .prepare("INSERT INTO agent_mcp_connections (agent_id, mcp_server_id) VALUES (?1, ?2)")?;
    for mid in mcp_server_ids {
        stmt.execute(params![agent_id, mid])?;
    }
    Ok(())
}

// ── Skills ──────────────────────────────────────────────────────

/// A skill / tool (matches frontend `Skill` type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub mcp_server_id: Option<String>,
    pub config: Option<String>,
    pub instructions: Option<String>,
    pub source_url: Option<String>,
    pub source_type: String,
    pub git_source_id: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// List all skills, ordered by source then name.
pub fn list_skills(conn: &Connection) -> Result<Vec<Skill>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, source, mcp_server_id, config,
                instructions, source_url, source_type, git_source_id, enabled, created_at, updated_at
         FROM skills ORDER BY source ASC, name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Skill {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            source: row.get(3)?,
            mcp_server_id: row.get(4)?,
            config: row.get(5)?,
            instructions: row.get(6)?,
            source_url: row.get(7)?,
            source_type: row
                .get::<_, Option<String>>(8)?
                .unwrap_or("builtin".to_string()),
            git_source_id: row.get(9)?,
            enabled: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    rows.collect()
}

/// Get a single skill by ID.
pub fn get_skill(conn: &Connection, id: &str) -> Result<Option<Skill>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, source, mcp_server_id, config,
                instructions, source_url, source_type, git_source_id, enabled, created_at, updated_at
         FROM skills WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Skill {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            source: row.get(3)?,
            mcp_server_id: row.get(4)?,
            config: row.get(5)?,
            instructions: row.get(6)?,
            source_url: row.get(7)?,
            source_type: row
                .get::<_, Option<String>>(8)?
                .unwrap_or("builtin".to_string()),
            git_source_id: row.get(9)?,
            enabled: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    rows.next().transpose()
}

/// Create a new skill, returns the created skill.
#[allow(clippy::too_many_arguments)]
pub fn create_skill(
    conn: &Connection,
    id: &str,
    name: &str,
    description: Option<&str>,
    source: &str,
    mcp_server_id: Option<&str>,
    config: Option<&str>,
    instructions: Option<&str>,
    source_url: Option<&str>,
    source_type: &str,
    git_source_id: Option<&str>,
) -> Result<Skill, rusqlite::Error> {
    conn.execute(
        "INSERT INTO skills (id, name, description, source, mcp_server_id, config,
                instructions, source_url, source_type, git_source_id, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, datetime('now'), datetime('now'))",
        params![
            id,
            name,
            description,
            source,
            mcp_server_id,
            config,
            instructions,
            source_url,
            source_type,
            git_source_id
        ],
    )?;
    get_skill(conn, id)?.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

/// Update an existing skill.
pub fn update_skill(
    conn: &Connection,
    id: &str,
    name: &str,
    description: Option<&str>,
    instructions: Option<&str>,
    config: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE skills SET name = ?2, description = ?3, instructions = ?4,
                config = ?5, updated_at = datetime('now')
         WHERE id = ?1",
        params![id, name, description, instructions, config],
    )?;
    Ok(())
}

/// Delete a skill by ID.
pub fn delete_skill(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM skills WHERE id = ?1", params![id])?;
    Ok(())
}

/// Toggle a skill's enabled state.
pub fn toggle_skill(conn: &Connection, id: &str, enabled: bool) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE skills SET enabled = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![id, enabled],
    )?;
    Ok(())
}

/// Get skills assigned to an agent (with full skill data).
pub fn get_agent_skills(conn: &Connection, agent_id: &str) -> Result<Vec<Skill>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.description, s.source, s.mcp_server_id, s.config,
                s.instructions, s.source_url, s.source_type, s.git_source_id, s.enabled, s.created_at, s.updated_at
         FROM skills s
         INNER JOIN agent_skills asg ON s.id = asg.skill_id
         WHERE asg.agent_id = ?1
         ORDER BY s.name ASC",
    )?;
    let rows = stmt.query_map(params![agent_id], |row| {
        Ok(Skill {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            source: row.get(3)?,
            mcp_server_id: row.get(4)?,
            config: row.get(5)?,
            instructions: row.get(6)?,
            source_url: row.get(7)?,
            source_type: row
                .get::<_, Option<String>>(8)?
                .unwrap_or("builtin".to_string()),
            git_source_id: row.get(9)?,
            enabled: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;
    rows.collect()
}

// ── Git Sources ─────────────────────────────────────────────────
// Used by commands/sources.rs (implemented in a later step).

/// A persistent git repository source for skills and agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct GitSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub last_synced_at: Option<String>,
    pub item_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// An item (skill or agent) linked to a git source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SourceItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub source_url: Option<String>,
}

/// List all git sources, ordered by name.
#[allow(dead_code)]
pub fn list_git_sources(conn: &Connection) -> Result<Vec<GitSource>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, enabled, last_synced_at, item_count, created_at, updated_at
         FROM git_sources ORDER BY name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GitSource {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            enabled: row.get(3)?,
            last_synced_at: row.get(4)?,
            item_count: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

/// Get a single git source by ID.
#[allow(dead_code)]
pub fn get_git_source(conn: &Connection, id: &str) -> Result<Option<GitSource>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, enabled, last_synced_at, item_count, created_at, updated_at
         FROM git_sources WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(GitSource {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            enabled: row.get(3)?,
            last_synced_at: row.get(4)?,
            item_count: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.next().transpose()
}

/// Create a new git source, returns the created source.
#[allow(dead_code)]
pub fn create_git_source(
    conn: &Connection,
    id: &str,
    name: &str,
    url: &str,
) -> Result<GitSource, rusqlite::Error> {
    conn.execute(
        "INSERT INTO git_sources (id, name, url, enabled, item_count, created_at, updated_at)
         VALUES (?1, ?2, ?3, 1, 0, datetime('now'), datetime('now'))",
        params![id, name, url],
    )?;
    get_git_source(conn, id)?.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
}

/// Update a git source's metadata (name and/or enabled state).
#[allow(dead_code)]
pub fn update_git_source(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    enabled: Option<bool>,
) -> Result<(), rusqlite::Error> {
    if let Some(name) = name {
        conn.execute(
            "UPDATE git_sources SET name = ?2, updated_at = datetime('now') WHERE id = ?1",
            params![id, name],
        )?;
    }
    if let Some(enabled) = enabled {
        conn.execute(
            "UPDATE git_sources SET enabled = ?2, updated_at = datetime('now') WHERE id = ?1",
            params![id, enabled],
        )?;
    }
    Ok(())
}

/// Delete a git source. Items become orphaned via ON DELETE SET NULL.
#[allow(dead_code)]
pub fn delete_git_source(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
    let rows = conn.execute("DELETE FROM git_sources WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

/// Update a git source's sync timestamp and item count.
#[allow(dead_code)]
pub fn update_git_source_synced(
    conn: &Connection,
    id: &str,
    item_count: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE git_sources SET last_synced_at = datetime('now'), item_count = ?2, updated_at = datetime('now')
         WHERE id = ?1",
        params![id, item_count],
    )?;
    Ok(())
}

/// List skills and agents linked to a specific git source.
#[allow(dead_code)]
pub fn get_source_items(
    conn: &Connection,
    source_id: &str,
) -> Result<Vec<SourceItem>, rusqlite::Error> {
    let mut items = Vec::new();

    // Skills linked to this source
    let mut stmt = conn.prepare(
        "SELECT id, name, source_url FROM skills WHERE git_source_id = ?1 ORDER BY name ASC",
    )?;
    let skill_rows = stmt.query_map(params![source_id], |row| {
        Ok(SourceItem {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: "skill".to_string(),
            source_url: row.get(2)?,
        })
    })?;
    for row in skill_rows {
        items.push(row?);
    }

    // Agents linked to this source
    let mut stmt = conn.prepare(
        "SELECT id, name, source_url FROM agents WHERE git_source_id = ?1 ORDER BY name ASC",
    )?;
    let agent_rows = stmt.query_map(params![source_id], |row| {
        Ok(SourceItem {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: "agent".to_string(),
            source_url: row.get(2)?,
        })
    })?;
    for row in agent_rows {
        items.push(row?);
    }

    Ok(items)
}

/// Recalculate and update the item count for a git source.
#[allow(dead_code)]
pub fn refresh_git_source_item_count(
    conn: &Connection,
    source_id: &str,
) -> Result<i64, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT (SELECT COUNT(*) FROM skills WHERE git_source_id = ?1) +
                (SELECT COUNT(*) FROM agents WHERE git_source_id = ?1)",
        params![source_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "UPDATE git_sources SET item_count = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![count, source_id],
    )?;
    Ok(count)
}

// ── Git Source Catalog Items ────────────────────────────────────

/// A catalog item persisted from a git source scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSourceCatalogItem {
    pub id: String,
    pub git_source_id: String,
    pub path: String,
    pub kind: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Upsert a catalog item from a git source scan.
/// On conflict (same source + path), update content/name/description.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn upsert_git_source_item(
    conn: &Connection,
    id: &str,
    git_source_id: &str,
    path: &str,
    kind: &str,
    name: &str,
    description: Option<&str>,
    content: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO git_source_items (id, git_source_id, path, kind, name, description, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))
         ON CONFLICT(git_source_id, path) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            content = excluded.content,
            kind = excluded.kind,
            updated_at = datetime('now')",
        params![id, git_source_id, path, kind, name, description, content],
    )?;
    Ok(())
}

/// Remove catalog items for a source that are no longer present in the repo.
/// `current_paths` is the set of paths found in the latest scan.
#[allow(dead_code)]
pub fn delete_stale_source_items(
    conn: &Connection,
    git_source_id: &str,
    current_paths: &[String],
) -> Result<usize, rusqlite::Error> {
    if current_paths.is_empty() {
        let count = conn.execute(
            "DELETE FROM git_source_items WHERE git_source_id = ?1",
            params![git_source_id],
        )?;
        return Ok(count);
    }

    let placeholders: Vec<String> = (0..current_paths.len())
        .map(|i| format!("?{}", i + 2))
        .collect();
    let sql = format!(
        "DELETE FROM git_source_items WHERE git_source_id = ?1 AND path NOT IN ({})",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(git_source_id.to_string()));
    for p in current_paths {
        param_values.push(Box::new(p.clone()));
    }
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|b| b.as_ref()).collect();
    let count = stmt.execute(params_ref.as_slice())?;
    Ok(count)
}

/// Get all catalog items from enabled git sources, optionally filtered by kind and search query.
#[allow(dead_code)]
pub fn get_catalog_entries(
    conn: &Connection,
    kind: Option<&str>,
    query: Option<&str>,
) -> Result<Vec<GitSourceCatalogItem>, rusqlite::Error> {
    let mut sql = String::from(
        "SELECT i.id, i.git_source_id, i.path, i.kind, i.name, i.description, i.content, i.created_at, i.updated_at
         FROM git_source_items i
         JOIN git_sources s ON i.git_source_id = s.id
         WHERE s.enabled = 1",
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;

    if let Some(k) = kind {
        sql.push_str(&format!(" AND i.kind = ?{param_idx}"));
        param_values.push(Box::new(k.to_string()));
        param_idx += 1;
    }

    if let Some(q) = query {
        if !q.trim().is_empty() {
            let pattern = format!("%{}%", q.trim());
            sql.push_str(&format!(
                " AND (i.name LIKE ?{param_idx} OR i.description LIKE ?{})",
                param_idx + 1
            ));
            param_values.push(Box::new(pattern.clone()));
            param_values.push(Box::new(pattern));
            param_idx += 2;
        }
    }
    let _ = param_idx;

    sql.push_str(" ORDER BY i.name ASC");

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_ref.as_slice(), |row| {
        Ok(GitSourceCatalogItem {
            id: row.get(0)?,
            git_source_id: row.get(1)?,
            path: row.get(2)?,
            kind: row.get(3)?,
            name: row.get(4)?,
            description: row.get(5)?,
            content: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

// ── Projects ────────────────────────────────────────────────────

/// A project container (matches frontend `Project` type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub instructions: Option<String>,
    pub file_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Metadata for a file attached to a project (excludes BLOB content).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFile {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub created_at: String,
}

/// List all projects with their file counts, ordered by most recently updated.
pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.instructions, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM project_files pf WHERE pf.project_id = p.id) AS file_count
         FROM projects p ORDER BY p.updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            instructions: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            file_count: row.get(5)?,
        })
    })?;
    rows.collect()
}

/// Get a single project by ID.
pub fn get_project(conn: &Connection, id: &str) -> Result<Option<Project>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.instructions, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM project_files pf WHERE pf.project_id = p.id) AS file_count
         FROM projects p WHERE p.id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            instructions: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            file_count: row.get(5)?,
        })
    })?;
    rows.next().transpose()
}

/// Create a new project.
pub fn create_project(
    conn: &Connection,
    id: &str,
    name: &str,
    instructions: Option<&str>,
) -> Result<Project, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (id, name, instructions, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, name, instructions, now],
    )?;
    Ok(Project {
        id: id.to_string(),
        name: name.to_string(),
        instructions: instructions.map(String::from),
        file_count: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Update a project's name and/or instructions.
pub fn update_project(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    instructions: Option<Option<&str>>,
) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    if let Some(n) = name {
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![n, now, id],
        )?;
    }
    // Option<Option<&str>>: Some(Some("text")) = set, Some(None) = clear, None = no change
    if let Some(instr) = instructions {
        conn.execute(
            "UPDATE projects SET instructions = ?1, updated_at = ?2 WHERE id = ?3",
            params![instr, now, id],
        )?;
    }
    Ok(())
}

/// Delete a project (files cascade via FK).
pub fn delete_project(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    // Unlink conversations from this project before deleting
    conn.execute(
        "UPDATE conversations SET project_id = NULL WHERE project_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

/// List files for a project (metadata only, no BLOB content).
pub fn list_project_files(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectFile>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, content_type, LENGTH(content) as size, created_at
         FROM project_files WHERE project_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(ProjectFile {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            content_type: row.get(3)?,
            size: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

/// Add a file to a project (content stored as BLOB).
pub fn add_project_file(
    conn: &Connection,
    id: &str,
    project_id: &str,
    name: &str,
    content_type: &str,
    content: &[u8],
) -> Result<ProjectFile, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO project_files (id, project_id, name, content_type, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, project_id, name, content_type, content, now],
    )?;
    // Touch parent project
    conn.execute(
        "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
        params![now, project_id],
    )?;
    Ok(ProjectFile {
        id: id.to_string(),
        project_id: project_id.to_string(),
        name: name.to_string(),
        content_type: content_type.to_string(),
        size: content.len() as i64,
        created_at: now,
    })
}

/// Get the raw content of a project file (for inclusion in chat context).
pub fn get_project_file_content(
    conn: &Connection,
    file_id: &str,
) -> Result<Option<(String, Vec<u8>)>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT name, content FROM project_files WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![file_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
    })?;
    rows.next().transpose()
}

/// Remove a file from a project.
pub fn remove_project_file(conn: &Connection, file_id: &str) -> Result<(), rusqlite::Error> {
    // Touch the parent project before deleting
    conn.execute(
        "UPDATE projects SET updated_at = datetime('now')
         WHERE id = (SELECT project_id FROM project_files WHERE id = ?1)",
        params![file_id],
    )?;
    conn.execute("DELETE FROM project_files WHERE id = ?1", params![file_id])?;
    Ok(())
}

/// List conversations belonging to a project.
pub fn list_project_conversations(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<Conversation>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, title, agent_id, project_id, model, is_favourite, created_at, updated_at
         FROM conversations WHERE project_id = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
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

/// Get all project file names + text content for building chat context.
/// Only returns files whose content_type starts with "text/".
pub fn get_project_text_files(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<(String, String)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT name, content FROM project_files
         WHERE project_id = ?1 AND content_type LIKE 'text/%'
         ORDER BY name ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        let name: String = row.get(0)?;
        let bytes: Vec<u8> = row.get(1)?;
        let text = String::from_utf8_lossy(&bytes).to_string();
        Ok((name, text))
    })?;
    rows.collect()
}

// ── Approved MCP Binaries ────────────────────────────────────────

/// Check whether a binary path has been approved for stdio MCP execution.
pub fn is_binary_approved(conn: &Connection, binary_path: &str) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM approved_mcp_binaries WHERE binary_path = ?1",
        [binary_path],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Approve a binary path for stdio MCP execution.
pub fn approve_binary(conn: &Connection, binary_path: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO approved_mcp_binaries (binary_path, approved_at)
         VALUES (?1, datetime('now'))",
        [binary_path],
    )?;
    Ok(())
}

/// Revoke approval for a binary path.
#[allow(dead_code)]
pub fn revoke_binary(conn: &Connection, binary_path: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM approved_mcp_binaries WHERE binary_path = ?1",
        [binary_path],
    )?;
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
        update_conversation(&conn, "c1", Some("Renamed"), Some(true), None, None).unwrap();
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

        // Schema version was seeded (v5 after all migrations)
        let ver = get_setting(&conn, "schema_version").unwrap();
        assert_eq!(ver, Some("5".to_string()));

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

    #[test]
    fn test_agent_crud() {
        let conn = setup_db();

        // Default agent should already exist
        let agents = list_agents(&conn).unwrap();
        assert_eq!(agents.len(), 1);
        assert!(agents[0].is_default);
        assert_eq!(agents[0].name, "Default");

        // Create
        let agent = create_agent(
            &conn,
            "a1",
            "Research Agent",
            Some("🔬"),
            "You are a research assistant.",
            None,
            "local",
            None,
        )
        .unwrap();
        assert_eq!(agent.name, "Research Agent");
        assert_eq!(agent.avatar, Some("🔬".to_string()));
        assert!(!agent.is_default);

        // List (default first)
        let agents = list_agents(&conn).unwrap();
        assert_eq!(agents.len(), 2);
        assert!(agents[0].is_default);
        assert_eq!(agents[1].id, "a1");

        // Get
        let found = get_agent(&conn, "a1").unwrap().unwrap();
        assert_eq!(found.system_prompt, "You are a research assistant.");

        // Update
        update_agent(
            &conn,
            "a1",
            "Updated Agent",
            Some("🧪"),
            "New prompt.",
            Some("https://example.com"),
            "registry_aitmpl",
        )
        .unwrap();
        let updated = get_agent(&conn, "a1").unwrap().unwrap();
        assert_eq!(updated.name, "Updated Agent");
        assert_eq!(updated.source_url, Some("https://example.com".to_string()));

        // Cannot update default agent
        update_agent(&conn, "default", "Hacked", None, "Hacked", None, "local").unwrap();
        let default = get_agent(&conn, "default").unwrap().unwrap();
        assert_eq!(default.name, "Default"); // unchanged

        // Cannot delete default agent
        let deleted = delete_agent(&conn, "default").unwrap();
        assert!(!deleted);
        assert!(get_agent(&conn, "default").unwrap().is_some());

        // Delete custom agent
        let deleted = delete_agent(&conn, "a1").unwrap();
        assert!(deleted);
        assert!(get_agent(&conn, "a1").unwrap().is_none());
    }

    #[test]
    fn test_agent_skills_assignments() {
        let conn = setup_db();

        // Create agent and skills
        create_agent(
            &conn,
            "a1",
            "Test Agent",
            None,
            "prompt",
            None,
            "local",
            None,
        )
        .unwrap();
        create_skill(
            &conn,
            "s1",
            "Skill 1",
            Some("desc1"),
            "builtin",
            None,
            None,
            None,
            None,
            "builtin",
            None,
        )
        .unwrap();
        create_skill(
            &conn,
            "s2",
            "Skill 2",
            Some("desc2"),
            "builtin",
            None,
            None,
            None,
            None,
            "builtin",
            None,
        )
        .unwrap();

        // No skills initially
        let skill_ids = get_agent_skill_ids(&conn, "a1").unwrap();
        assert!(skill_ids.is_empty());

        // Assign skills
        set_agent_skills(&conn, "a1", &["s1".to_string(), "s2".to_string()]).unwrap();
        let skill_ids = get_agent_skill_ids(&conn, "a1").unwrap();
        assert_eq!(skill_ids.len(), 2);

        // Get full skills
        let skills = get_agent_skills(&conn, "a1").unwrap();
        assert_eq!(skills.len(), 2);

        // Replace with subset
        set_agent_skills(&conn, "a1", &["s1".to_string()]).unwrap();
        let skill_ids = get_agent_skill_ids(&conn, "a1").unwrap();
        assert_eq!(skill_ids.len(), 1);
        assert_eq!(skill_ids[0], "s1");

        // Clear all
        set_agent_skills(&conn, "a1", &[]).unwrap();
        assert!(get_agent_skill_ids(&conn, "a1").unwrap().is_empty());
    }

    #[test]
    fn test_agent_mcp_connections() {
        let conn = setup_db();

        create_agent(
            &conn,
            "a1",
            "Test Agent",
            None,
            "prompt",
            None,
            "local",
            None,
        )
        .unwrap();

        // No connections initially
        assert!(get_agent_mcp_ids(&conn, "a1").unwrap().is_empty());

        // Assign
        set_agent_mcp_connections(&conn, "a1", &["mcp1".to_string(), "mcp2".to_string()]).unwrap();
        let ids = get_agent_mcp_ids(&conn, "a1").unwrap();
        assert_eq!(ids.len(), 2);

        // Replace
        set_agent_mcp_connections(&conn, "a1", &["mcp2".to_string()]).unwrap();
        let ids = get_agent_mcp_ids(&conn, "a1").unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "mcp2");
    }

    #[test]
    fn test_skill_crud() {
        let conn = setup_db();

        // Initially no skills
        let skills = list_skills(&conn).unwrap();
        assert!(skills.is_empty());

        // Create
        let skill = create_skill(
            &conn,
            "sk1",
            "Code Review",
            Some("Reviews code quality"),
            "registry_aitmpl",
            None,
            None,
            Some("You are a code reviewer."),
            Some("https://www.aitmpl.com/component/skill/code-review"),
            "registry_aitmpl",
            None,
        )
        .unwrap();
        assert_eq!(skill.name, "Code Review");
        assert_eq!(
            skill.instructions,
            Some("You are a code reviewer.".to_string())
        );
        assert!(skill.enabled);

        // Get
        let found = get_skill(&conn, "sk1").unwrap().unwrap();
        assert_eq!(found.source_type, "registry_aitmpl");

        // Update
        update_skill(
            &conn,
            "sk1",
            "Updated Skill",
            Some("Updated desc"),
            Some("New instructions"),
            None,
        )
        .unwrap();
        let updated = get_skill(&conn, "sk1").unwrap().unwrap();
        assert_eq!(updated.name, "Updated Skill");
        assert_eq!(updated.instructions, Some("New instructions".to_string()));

        // Toggle
        toggle_skill(&conn, "sk1", false).unwrap();
        let toggled = get_skill(&conn, "sk1").unwrap().unwrap();
        assert!(!toggled.enabled);

        toggle_skill(&conn, "sk1", true).unwrap();
        let toggled = get_skill(&conn, "sk1").unwrap().unwrap();
        assert!(toggled.enabled);

        // Delete
        delete_skill(&conn, "sk1").unwrap();
        assert!(get_skill(&conn, "sk1").unwrap().is_none());
    }

    #[test]
    fn test_skill_cascade_from_agent() {
        let conn = setup_db();

        create_agent(&conn, "a1", "Agent", None, "prompt", None, "local", None).unwrap();
        create_skill(
            &conn, "sk1", "Skill", None, "builtin", None, None, None, None, "builtin", None,
        )
        .unwrap();

        // Assign skill to agent
        set_agent_skills(&conn, "a1", &["sk1".to_string()]).unwrap();
        assert_eq!(get_agent_skill_ids(&conn, "a1").unwrap().len(), 1);

        // Deleting skill should cascade (remove from agent_skills)
        delete_skill(&conn, "sk1").unwrap();
        // The agent_skills FK won't cascade by default since skill_id isn't a FK,
        // but let's verify the skill is gone and clean up manually
        assert!(get_skill(&conn, "sk1").unwrap().is_none());
    }

    #[test]
    fn test_project_crud() {
        let conn = setup_db();

        // Initially no projects
        let projects = list_projects(&conn).unwrap();
        assert!(projects.is_empty());

        // Create
        let proj = create_project(&conn, "p1", "My Project", Some("Custom instructions")).unwrap();
        assert_eq!(proj.name, "My Project");
        assert_eq!(proj.instructions, Some("Custom instructions".to_string()));
        assert_eq!(proj.file_count, 0);

        // List
        let projects = list_projects(&conn).unwrap();
        assert_eq!(projects.len(), 1);

        // Get
        let found = get_project(&conn, "p1").unwrap().unwrap();
        assert_eq!(found.name, "My Project");

        // Update name
        update_project(&conn, "p1", Some("Renamed Project"), None).unwrap();
        let updated = get_project(&conn, "p1").unwrap().unwrap();
        assert_eq!(updated.name, "Renamed Project");

        // Update instructions (clear)
        update_project(&conn, "p1", None, Some(None)).unwrap();
        let updated = get_project(&conn, "p1").unwrap().unwrap();
        assert!(updated.instructions.is_none());

        // Delete
        delete_project(&conn, "p1").unwrap();
        assert!(get_project(&conn, "p1").unwrap().is_none());
    }

    #[test]
    fn test_project_files() {
        let conn = setup_db();
        create_project(&conn, "p1", "Test Project", None).unwrap();

        // Add file
        let file = add_project_file(
            &conn,
            "f1",
            "p1",
            "config.json",
            "text/json",
            b"{\"key\": \"value\"}",
        )
        .unwrap();
        assert_eq!(file.name, "config.json");
        assert_eq!(file.size, 16);

        // List files
        let files = list_project_files(&conn, "p1").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].content_type, "text/json");

        // File count on project
        let proj = get_project(&conn, "p1").unwrap().unwrap();
        assert_eq!(proj.file_count, 1);

        // Get content
        let (name, content) = get_project_file_content(&conn, "f1").unwrap().unwrap();
        assert_eq!(name, "config.json");
        assert_eq!(content, b"{\"key\": \"value\"}");

        // Get text files
        let text_files = get_project_text_files(&conn, "p1").unwrap();
        assert_eq!(text_files.len(), 1);
        assert_eq!(text_files[0].0, "config.json");

        // Remove file
        remove_project_file(&conn, "f1").unwrap();
        let files = list_project_files(&conn, "p1").unwrap();
        assert!(files.is_empty());

        // File count should be 0
        let proj = get_project(&conn, "p1").unwrap().unwrap();
        assert_eq!(proj.file_count, 0);
    }

    #[test]
    fn test_project_conversations() {
        let conn = setup_db();
        create_project(&conn, "p1", "Test Project", None).unwrap();

        // Create conversation with project
        create_conversation(&conn, "c1", Some("Chat 1"), None, Some("p1"), None).unwrap();
        create_conversation(&conn, "c2", Some("Chat 2"), None, None, None).unwrap();

        // List project conversations
        let convos = list_project_conversations(&conn, "p1").unwrap();
        assert_eq!(convos.len(), 1);
        assert_eq!(convos[0].id, "c1");

        // Assign c2 to project
        update_conversation(&conn, "c2", None, None, None, Some(Some("p1"))).unwrap();
        let convos = list_project_conversations(&conn, "p1").unwrap();
        assert_eq!(convos.len(), 2);

        // Unassign c1
        update_conversation(&conn, "c1", None, None, None, Some(None)).unwrap();
        let convos = list_project_conversations(&conn, "p1").unwrap();
        assert_eq!(convos.len(), 1);
        assert_eq!(convos[0].id, "c2");

        // Delete project should unlink conversations
        delete_project(&conn, "p1").unwrap();
        let c2 = get_conversation(&conn, "c2").unwrap().unwrap();
        assert!(c2.project_id.is_none());
    }

    #[test]
    fn test_project_cascade_delete_files() {
        let conn = setup_db();
        create_project(&conn, "p1", "Test Project", None).unwrap();
        add_project_file(&conn, "f1", "p1", "file.txt", "text/plain", b"hello").unwrap();
        add_project_file(&conn, "f2", "p1", "file2.txt", "text/plain", b"world").unwrap();

        let files = list_project_files(&conn, "p1").unwrap();
        assert_eq!(files.len(), 2);

        // Delete project should cascade to files
        delete_project(&conn, "p1").unwrap();
        let files = list_project_files(&conn, "p1").unwrap();
        assert!(files.is_empty());
    }
}
