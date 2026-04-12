# Chuck — Data Model (SQLite)

> Part of the [Chuck project documentation](../AGENTS.md). Covers the SQLite schema,
> persistence rules, migrations, and versioning strategy.

---

## Data Model (SQLite)

All persistent data is stored in a single SQLite database in the app data directory
(via Tauri's `app.path().app_data_dir()`).

### Tables

```sql
-- Conversations
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,           -- UUID
    title TEXT,                    -- Auto-generated or user-edited
    agent_id TEXT,                  -- Soft reference to agents(id)
    project_id TEXT,                -- Soft reference to projects(id)
    model TEXT,                    -- Model used (e.g., "gpt-4o")
    is_favourite INTEGER DEFAULT 0, -- 1 if pinned to top of sidebar
    created_at TEXT NOT NULL,      -- ISO 8601
    updated_at TEXT NOT NULL
);

-- Messages
CREATE TABLE messages (
    id TEXT PRIMARY KEY,           -- UUID
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL,            -- "user", "assistant", "system", "tool"
    content TEXT NOT NULL,         -- Message text (markdown for assistant)
    thinking_content TEXT,         -- Thinking/reasoning tokens (if model provides them)
    tool_call_id TEXT,             -- For tool responses
    tool_name TEXT,                -- For tool calls
    attachments TEXT,              -- JSON array of {name, type, size} for attached files
    created_at TEXT NOT NULL,
    sort_order INTEGER NOT NULL    -- Ordering within conversation
);

-- Projects
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    instructions TEXT,             -- Custom system instructions
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Project pinned files (content stored, not paths)
CREATE TABLE project_files (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,            -- Original filename
    content_type TEXT NOT NULL,    -- MIME type
    content BLOB NOT NULL,         -- File content (stored in DB, not on filesystem)
    created_at TEXT NOT NULL
);

-- Agents
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    avatar TEXT,                   -- Emoji or icon identifier
    system_prompt TEXT NOT NULL,
    is_default INTEGER DEFAULT 0,  -- 1 for built-in agents (protected from deletion)
    source_url TEXT,               -- Registry permalink or git URL (NULL for local)
    source_type TEXT DEFAULT 'local', -- "local", "registry_aitmpl", "git"
    git_source_id TEXT REFERENCES git_sources(id) ON DELETE SET NULL, -- Links to managing git source
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Agent ↔ Skill assignments
CREATE TABLE agent_skills (
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL,        -- Skill identifier (extension ID or MCP tool ID)
    PRIMARY KEY (agent_id, skill_id)
);

-- Agent ↔ MCP server connections
CREATE TABLE agent_mcp_connections (
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    mcp_server_id TEXT NOT NULL,
    PRIMARY KEY (agent_id, mcp_server_id)
);

-- Skills (built-in + MCP tools + registry-imported + git-imported)
CREATE TABLE skills (
    id TEXT PRIMARY KEY,           -- Unique skill ID
    name TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL,          -- "extension" or "mcp" (legacy)
    source_type TEXT DEFAULT 'builtin', -- "builtin", "mcp", "registry_skills_sh", "registry_aitmpl", "git"
    source_url TEXT,               -- Registry permalink or git URL (NULL for built-in/MCP)
    instructions TEXT,             -- Markdown body from SKILL.md (injected into system prompt when active)
    mcp_server_id TEXT,             -- Soft reference to mcp_servers(id); NULL for non-MCP skills
    config TEXT,                   -- JSON config blob
    enabled INTEGER DEFAULT 1,
    git_source_id TEXT REFERENCES git_sources(id) ON DELETE SET NULL, -- Links to managing git source
    created_at TEXT NOT NULL,
    updated_at TEXT                -- Added in migration v2
);

-- MCP server configurations
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    transport TEXT NOT NULL,       -- "http" or "stdio"
    url TEXT,                      -- For HTTP transport
    binary_path TEXT,              -- For stdio transport
    args TEXT,                     -- JSON array of arguments for stdio
    auth_header TEXT,              -- DEPRECATED: migrated to OS keychain in v3; column kept for schema compat
    from_catalog INTEGER DEFAULT 0, -- 1 if added from MCP Registry
    enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Approved MCP stdio binary paths (user must approve before first launch)
CREATE TABLE approved_mcp_binaries (
    binary_path TEXT PRIMARY KEY,
    approved_at TEXT NOT NULL      -- ISO 8601
);

-- User preferences
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Draft auto-save (crash recovery)
CREATE TABLE drafts (
    conversation_id TEXT PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    content TEXT NOT NULL,         -- Draft input text
    updated_at TEXT NOT NULL
);

-- Git Sources (persistent git repository sources for importing skills/agents)
CREATE TABLE git_sources (
    id TEXT PRIMARY KEY,           -- UUID
    name TEXT NOT NULL,            -- Display name
    url TEXT NOT NULL UNIQUE,      -- Git repository URL
    enabled INTEGER DEFAULT 1,    -- 1 = active (syncs on launch), 0 = paused
    last_synced_at TEXT,           -- ISO 8601 timestamp of last successful sync
    item_count INTEGER DEFAULT 0, -- Number of imported items from this source
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Git Source Items (cached discovery data for catalog browsing)
CREATE TABLE git_source_items (
    id TEXT PRIMARY KEY,                  -- UUID
    git_source_id TEXT NOT NULL REFERENCES git_sources(id) ON DELETE CASCADE,
    path TEXT NOT NULL,                   -- File path in the repo (e.g. "skills/SKILL.md")
    kind TEXT NOT NULL,                   -- "skill" or "agent"
    name TEXT NOT NULL,                   -- Parsed name from SKILL.md/agent.md frontmatter
    description TEXT,                     -- Parsed description
    content TEXT NOT NULL,                -- Full file content
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(git_source_id, path)
);

-- ── Indexes (performance-critical queries) ──

CREATE INDEX idx_messages_conversation ON messages(conversation_id, sort_order);
CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);
CREATE INDEX idx_conversations_project ON conversations(project_id);
CREATE INDEX idx_conversations_agent ON conversations(agent_id);
CREATE INDEX idx_conversations_favourite ON conversations(is_favourite) WHERE is_favourite = 1;
CREATE INDEX idx_project_files_project ON project_files(project_id);
CREATE INDEX idx_agent_skills_agent ON agent_skills(agent_id);
CREATE INDEX idx_skills_source ON skills(source);
CREATE INDEX idx_git_sources_enabled ON git_sources(enabled);
CREATE INDEX idx_git_source_items_source ON git_source_items(git_source_id);
CREATE INDEX idx_git_source_items_kind ON git_source_items(kind);

-- ── Initial seed data ──

INSERT INTO config (key, value) VALUES ('schema_version', '6');
```

> _Note: The schema above reflects the **final state** after all migrations (v1→v2→v3→v4→v5→v6). See `src-tauri/src/db/migrations.rs` for the incremental ALTER TABLE statements that evolve the schema across versions. Migration v4 adds the `git_sources` table, `git_source_id` FK columns on skills/agents, and backfills sources from existing git-imported items. Migration v5 adds the `git_source_items` table for persisting discovered items from source scans, enabling catalog browsing without re-fetching from git. Migration v6 upgrades the built-in Default agent prompt, inserts a built-in Research agent (id='research', is_default=1), and seeds the `default_agent_id` config key for user-configurable default agent selection._

### Persistence Rules

- **Conversations, messages, projects, agents, skills, MCP configs** → SQLite (managed by Rust backend)
- **OAuth tokens, API keys, MCP auth headers** → OS keychain via `keyring` crate (never in SQLite or localStorage)
- **User preferences** (theme, font size, hotkey, send shortcut, auto-update, default agent) → SQLite `config` table (e.g., keys: `theme`, `font_size`, `global_hotkey`, `send_shortcut`, `auto_update_enabled`, `auto_update_frequency`, `default_agent_id`)
- **File contents** for project pinned files → SQLite `project_files.content` as BLOB
- **Attached file contents** in chat → stored in `messages.attachments` as metadata only; full content is ephemeral (in-memory during conversation, not persisted)
- **Registry cache** — in-memory SWR cache for aitmpl.com `components.json` (not persisted to SQLite); recreated on each app launch with 1-hour TTL
- **No localStorage/sessionStorage** for sensitive data — all persistence goes through Tauri commands to Rust backend

### Schema Migrations

- Use a `schema_version` key in the `config` table to track the current DB schema version
- On startup, compare `schema_version` against the app's expected version
- Apply sequential migration scripts (embedded in the Rust binary) to bring the schema up to date
- Migrations must be forward-only and non-destructive — never drop data without user consent
- This is critical for auto-update: after an update, the new version may expect a newer schema

### Versioning

- Follow [Semantic Versioning](https://semver.org/) (`MAJOR.MINOR.PATCH`)
- **Lockstep versioning:** all Rust crates share a single version via `[workspace.package]` in the root `Cargo.toml`
- **Single source of truth:** root `Cargo.toml` → all crates use `version.workspace = true`
- **Three files kept in sync:** `Cargo.toml` (workspace), `package.json`, `src-tauri/tauri.conf.json`
- Git tags for releases use the format `vX.Y.Z` (e.g., `v1.2.3`)
- `tauri-plugin-updater` compares the app version against the latest GitHub Release tag
- Pre-release versions (e.g., `v1.0.0-beta.1`) should be excluded from auto-update by default

#### xtask Commands

| Command                                  | Purpose                                                                      |
| ---------------------------------------- | ---------------------------------------------------------------------------- |
| `cargo xtask bump <patch\|minor\|major>` | Bump version across all 3 files                                              |
| `cargo xtask check-version`              | Verify all version strings are in sync (CI-friendly)                         |
| `cargo xtask changelog`                  | Generate/update `CHANGELOG.md` from conventional commits since last tag      |
| `cargo xtask release`                    | **Automated release** — auto-detect bump level, bump, changelog, commit, tag |
| `cargo xtask release --dry-run`          | Preview what a release would do without making changes                       |
| `cargo xtask release --bump <level>`     | Force a specific bump level instead of auto-detecting                        |

#### Release Flow

The recommended way to cut a release:

```
cargo xtask release              # auto-detect + release
cargo xtask release --dry-run    # preview first (recommended)
git push && git push --tags      # publish after review
```

**Auto-detection logic** (from conventional commits since last tag):

- Any `!` (breaking change indicator) → **major** bump
- Any `feat` commit → **minor** bump
- Otherwise (fix, chore, refactor, etc.) → **patch** bump

**What `cargo xtask release` does:**

1. Verifies the working tree is clean
2. Scans conventional commits since the last git tag
3. Auto-detects bump level (or uses `--bump` override)
4. Bumps version across all 3 files (`bump::run`)
5. Generates/updates `CHANGELOG.md` (`changelog::run`)
6. Verifies version consistency (`check::run`)
7. Creates a git commit: `chore: release vX.Y.Z`
8. Creates an annotated git tag: `vX.Y.Z`
9. Prints a reminder to `git push && git push --tags`

If changelog generation or later steps fail after files are bumped, the tool prints recovery instructions (`git checkout -- <files>`).

---
