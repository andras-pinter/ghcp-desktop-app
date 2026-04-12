# Chuck — API & Integration Reference

> Part of the [Chuck project documentation](../AGENTS.md). Covers dependencies,
> GitHub Copilot API integration, MCP protocol, and the skills/agents system.

---

## Key Dependencies

> ⚠️ **Always use the latest stable version.** The packages listed below are recommendations —
> verify versions on [crates.io](https://crates.io) and [npmjs.com](https://www.npmjs.com/) at
> implementation time. If a package has been superseded or deprecated, use the replacement and
> update this table.

### Rust Crates (src-tauri + library crates)

| Crate                            | Purpose                                                                                                                                                                                                                                     |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tauri` v2                       | Application framework (with features: `tray-icon`, `devtools`)                                                                                                                                                                              |
| `tauri-plugin-dialog`            | Native file picker + save dialog                                                                                                                                                                                                            |
| `tauri-plugin-global-shortcut`   | System-wide keyboard shortcuts                                                                                                                                                                                                              |
| `tauri-plugin-updater`           | Auto-update from GitHub Releases                                                                                                                                                                                                            |
| `tauri-plugin-notification`      | System notifications (used when app is minimized to tray — e.g., streaming complete, update available)                                                                                                                                      |
| `tauri-plugin-shell`             | Limited shell access (MCP stdio only, scoped)                                                                                                                                                                                               |
| `tauri-plugin-clipboard-manager` | Copy to clipboard from code blocks                                                                                                                                                                                                          |
| `tauri-plugin-store`             | Lightweight key-value persistence for non-sensitive UI preferences (e.g., window position, sidebar width). SQLite `config` table handles all app settings; `tauri-plugin-store` is for ephemeral/UI-state that doesn't warrant a SQL write. |
| `tauri-plugin-process`           | App process management (relaunch after auto-update)                                                                                                                                                                                         |
| `reqwest`                        | HTTP client (enable `stream` feature for SSE)                                                                                                                                                                                               |
| `serde` / `serde_json`           | JSON serialization (shared types Rust ↔ frontend)                                                                                                                                                                                           |
| `tokio`                          | Async runtime                                                                                                                                                                                                                               |
| `reqwest-eventsource`            | SSE client for streaming responses (wraps reqwest + eventsource-stream with auto-retry)                                                                                                                                                     |
| `keyring`                        | Cross-platform keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager)                                                                                                                                                  |
| `rusqlite`                       | Local persistence (conversations, projects, agents, skills, MCP configs)                                                                                                                                                                    |
| `pdf-extract` / `lopdf`          | Extract text from PDFs (pdf-extract primary, lopdf raw fallback)                                                                                                                                                                            |
| `thiserror` / `anyhow`           | Error handling                                                                                                                                                                                                                              |
| `log` / `env_logger`             | Logging                                                                                                                                                                                                                                     |
| `dom_smoothie`                   | Readable content extraction (Readability algorithm) for URL fetching                                                                                                                                                                        |
| `url`                            | URL parsing and validation                                                                                                                                                                                                                  |
| `uuid`                           | UUID generation (conversation, message, project, agent IDs)                                                                                                                                                                                 |
| `base64`                         | Base64 encoding/decoding (file content transfer between frontend and backend)                                                                                                                                                               |
| `chrono`                         | Date/time handling (ISO 8601 timestamps, changelog generation)                                                                                                                                                                              |
| `futures-util`                   | Async stream combinators (SSE streaming, web fetching)                                                                                                                                                                                      |
| `async-trait`                    | Async fn in traits (registry provider trait)                                                                                                                                                                                                |
| `serde_norway`                   | YAML parsing (SKILL.md frontmatter deserialization)                                                                                                                                                                                         |
| `zip`                            | ZIP archive reading (DOCX/XLSX/PPTX text extraction)                                                                                                                                                                                        |
| `rmcp`                           | Official MCP Rust SDK (Model Context Protocol, spec version 2025-03-26+)                                                                                                                                                                    |
| `semver`                         | Semantic version parsing (xtask version management)                                                                                                                                                                                         |
| `toml_edit`                      | TOML file editing with formatting preservation (xtask version bumping)                                                                                                                                                                      |

### Frontend (npm packages)

| Package                                  | Purpose                                                          |
| ---------------------------------------- | ---------------------------------------------------------------- |
| `svelte` v5                              | UI framework                                                     |
| `@sveltejs/vite-plugin-svelte`           | Svelte integration for Vite                                      |
| `vite`                                   | Frontend build tool                                              |
| `typescript`                             | Type safety                                                      |
| `@tauri-apps/api` v2                     | Tauri frontend IPC (`invoke`, `listen`, etc.)                    |
| `@tauri-apps/plugin-dialog`              | Frontend bindings for dialog plugin                              |
| `@tauri-apps/plugin-global-shortcut`     | Frontend bindings for global shortcut plugin                     |
| `@tauri-apps/plugin-updater`             | Frontend bindings for updater plugin                             |
| `@tauri-apps/plugin-notification`        | Frontend bindings for notification plugin                        |
| `@tauri-apps/plugin-shell`               | Frontend bindings for shell plugin                               |
| `@tauri-apps/plugin-clipboard-manager`   | Frontend bindings for clipboard plugin                           |
| `@tauri-apps/plugin-store`               | Frontend bindings for store plugin (ephemeral UI state only)     |
| `@tauri-apps/plugin-process`             | Frontend bindings for process plugin (app relaunch after update) |
| `@fontsource-variable/plus-jakarta-sans` | Plus Jakarta Sans variable font (body text, UI)                  |
| `@fontsource/instrument-serif`           | Instrument Serif font (display titles, editorial headings)       |
| `marked`                                 | Markdown parsing (fast, CommonMark-compliant)                    |
| `shiki`                                  | Syntax highlighting (VS Code quality, WASM-based)                |
| `dompurify`                              | HTML sanitization for rendered markdown                          |
| `vitest`                                 | Frontend unit testing                                            |
| `svelte-check`                           | Svelte type checker (validates `.svelte` files with TypeScript)  |
| `eslint`                                 | Code linting                                                     |
| `prettier`                               | Code formatting                                                  |
| `prettier-plugin-svelte`                 | Prettier support for `.svelte` files                             |
| `eslint-plugin-svelte`                   | ESLint rules for `.svelte` files                                 |
| `@testing-library/svelte`                | Component testing utilities                                      |
| `@types/dompurify`                       | TypeScript definitions for DOMPurify                             |
| `@eslint/js`                             | ESLint core JavaScript rules                                     |
| `@tsconfig/svelte`                       | Shared TypeScript config for Svelte projects                     |
| `typescript-eslint`                      | TypeScript ESLint plugin and parser                              |
| `globals`                                | Global variable definitions for ESLint                           |
| `jsdom`                                  | DOM implementation for Vitest test environment                   |

---

## GitHub Copilot API Integration

### Authentication

Uses the **OAuth device flow** — the same flow VS Code uses to authenticate with Copilot:

1. App requests a device code from GitHub
2. User opens a browser URL and enters the code (opened via Tauri's `shell.open()`)
3. App polls for the OAuth token
4. Token is stored in the OS keychain
5. Token is refreshed automatically before expiry; if refresh fails, prompt re-auth

### Chat Completions

- Endpoint: `POST /v1/chat/completions`
- Request body follows the OpenAI-compatible chat completions format
- Streaming via Server-Sent Events (SSE) — `stream: true`
- File context is included as part of the message content (text extracted from files)
- System messages can carry project-level custom instructions
- Streaming tokens are pushed to the frontend via Tauri events (`streaming-token`)

### Rate Limits & Errors

- Respect `Retry-After` headers on 429 responses
- Show a non-intrusive toast/banner for rate limit warnings
- Gracefully degrade if the API is unreachable (show cached conversations, disable send)

### Context Window Management

- The app must manage conversation history to fit within the model's context window
- Strategy: **summarization** — when history exceeds the context limit, older messages are summarized into a condensed system-level recap, preserving key context while freeing token budget
- Always retain: the system prompt, project instructions, and the most recent messages
- Show a subtle indicator when older messages have been summarized (e.g., "Earlier messages summarized")
- The summarization prompt is internal and should not be visible to the user as a separate message

### Model Discovery

- On startup (and on auth token change), query the Copilot API for available models
- Cache the model list in memory for the session
- If the API endpoint fails or returns empty, fall back to a hardcoded default model
- The model selector in the UI always renders; if only one model is available, show it as a label instead of a dropdown

### Conversation Title Generation

- After the first assistant response in a new conversation, send a lightweight API call:
  _"Generate a concise 4-6 word title for this conversation"_ with the first user message + response as context
- Title appears in the sidebar immediately; user can always edit it manually
- If the title generation call fails, fall back to the first ~50 characters of the user's first message

### Message Editing & Regeneration

- **Edit:** user can click to edit any of their sent messages. This discards all messages after the edited one and re-sends from that point. No conversation branching — history is linear.
- **Regenerate:** a button on the last assistant message re-sends the last user message to get a fresh response. The previous response is replaced (not preserved).
- Both actions show a confirmation if they would discard more than one message pair.

### Thinking / Reasoning Display

- If the model returns thinking/reasoning tokens (e.g., extended thinking), display them in a **collapsible "Thinking…" section** above the response
- Collapsed by default — user can expand to see the reasoning process
- Visually distinct from the main response (lighter text, indented, in a subtle `<details>` element)

### Crash Recovery

- **Draft auto-save:** persist the current input text to SQLite every few seconds while the user types. Clear on successful send. On next launch, restore draft if present.
- **Interrupted streaming:** if the app crashes or loses connection mid-stream, save whatever tokens were received so far with a "(response interrupted)" marker. The user can regenerate.
- **Startup check:** on launch, detect if the previous session ended abnormally and show a brief "Recovered from unexpected shutdown" toast if drafts were restored.

### Offline Behavior

- **Full read access:** browse all conversations, search, manage projects/agents/skills/settings
- **Sending disabled:** input area shows a clear "You're offline" indicator, send button disabled
- **Auto-reconnect:** monitor network status; when connectivity returns, automatically re-enable sending and show a brief "Back online" toast
- **No queue:** messages are not queued for later sending — the user must explicitly send when online

### Conversation Export

- Available from settings and per-conversation context menu
- Uses `tauri-plugin-dialog` save dialog for choosing the export location
- **JSON format:** full structured data — messages, metadata, timestamps, tool calls. Machine-readable, suitable for backup.
- **Markdown format:** human-readable document — one file per conversation with headers, message attribution, code blocks preserved. Suitable for sharing or archiving.

### Database & Storage

- SQLite database stored in the **platform-standard app data directory** via Tauri's `app.path().app_data_dir()`:
  - macOS: `~/Library/Application Support/com.copilot-desktop.app/`
  - Linux: `~/.local/share/com.copilot-desktop.app/` (XDG_DATA_HOME)
  - Windows: `%APPDATA%\com.copilot-desktop.app\`
- Show database size in settings
- Offer "Delete conversations older than X days" cleanup option
- Warn if database exceeds 500MB with a suggestion to clean up old conversations

---

## Auto-Update

### Mechanism

- Uses **`tauri-plugin-updater`** to check for and apply updates from **GitHub Releases**
- Configured in `tauri.conf.json` under `plugins.updater` with the GitHub Releases endpoint
- On startup (and at a configurable interval), the plugin checks for newer versions
- Compares the current app version against the latest release tag
- If a new version is available, shows a **non-intrusive notification** in the app (not a blocking dialog)

### Update Flow

1. `tauri-plugin-updater` checks GitHub Releases API → finds newer version
2. Frontend receives update event → shows `UpdateBanner.svelte`: "Version X.Y.Z is available" with changelog
3. User clicks "Update now" → plugin downloads platform-specific bundle from release assets
4. Verifies download integrity (signature verification via Tauri's built-in signing)
5. Applies update and prompts user to restart

### User Controls (in Settings)

- **Auto-check for updates:** on/off (default: on)
- **Check frequency:** startup only, daily, weekly
- **"Skip this version"** — suppress notifications for a specific release
- **"Remind me later"** — snooze for a configurable period

### Security

- Only fetch from the project's own GitHub Releases — no third-party update servers
- Tauri's built-in update signature verification (ed25519 key pair)
- HTTPS only for all update traffic
- Never auto-apply without user confirmation — always require explicit action

---

## Web Research Integration

### Search API

- Use **Bing Web Search API** (primary) or Google Custom Search as fallback
- API key stored in OS keychain alongside OAuth tokens
- Search can be triggered two ways:
  1. **AI-initiated:** Copilot requests a web search via function calling / tool use (the app exposes a `web_search` tool in the system prompt). Results are injected into context automatically.
  2. **User-initiated:** User clicks the 🌐 button or pastes a URL in the input area
- Search results are displayed as **cited cards** (`WebResultCard.svelte`) in the chat: title, snippet, source URL
- Results are cached in-memory for the session to avoid redundant API calls
- Rate limits: respect API quotas, show clear error if quota exceeded

### URL Fetching

- User pastes a URL → Rust backend fetches the page over HTTPS → extracts readable text via `dom_smoothie` (Readability algorithm)
- **Security:** only public HTTPS URLs allowed. Block private IPs, localhost, metadata endpoints (see [Security](../AGENTS.md#security))
- Extracted content is truncated to a reasonable size (e.g., 50KB of text) before inclusion in context
- Show a URL preview card in the input area (title, domain, favicon if available)

---

## MCP Integration

### Protocol

- Target **MCP specification version 2025-03-26** (or latest stable at implementation time)
- Reference: https://modelcontextprotocol.io/specification
- Support two transports:
  - **HTTP (SSE)** — preferred, works with remote servers. Default for registry entries with remote URLs.
  - **Stdio** — for local MCP servers. Requires user-approved binary path (see [MCP Security](../AGENTS.md#mcp-security)). Uses `tauri-plugin-shell` with scoped permissions.

### MCP Registry

Instead of a static built-in catalog, the app browses the **official MCP Registry** at `registry.modelcontextprotocol.io`:

- **Server-side search** via `?search=` API parameter — returns servers matching the query by name/description
- **Cursor-based pagination** — fetches 20 servers per page with infinite scroll (load more on reaching bottom)
- **First-party prioritization** — servers from well-known publishers (Microsoft, GitHub, Anthropic, etc.) are sorted to the top via heuristic
- **Multi-package support** — each server may offer npm, pypi, nuget, or docker packages. The app auto-detects the best option and pre-fills `npx -y`, `uvx`, or `dotnet tool run` commands with the correct version and `packageArguments`
- **One-click add** — users can add a server directly from the registry browser; the app constructs the full command (e.g., `npx -y @azure/mcp@3.0.0-beta.1 server start`) and auto-connects
- **Detail view** — clicking a registry entry shows full description, connection options (remote vs stdio), package versions, and setup guidance
- **Auto-connect** — newly added servers connect automatically

### Custom Servers

Users can also add custom MCP servers manually in settings (`McpSettings.svelte` + `McpServerForm.svelte`):

- **HTTP servers:** URL + optional auth header
- **Stdio servers:** binary path + arguments (user-approved, see [MCP Security](../AGENTS.md#mcp-security))
- Test connectivity button to verify the server responds
- View discovered tools/resources from the server

---

## Skills & Agents Concepts

### Definitions

| Concept               | What It Is                                                                                                                                                                                                                                                                                                                    | Example                                             |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| **Skill**             | A capability/instruction set that extends what the AI can do. Can be: a built-in tool (e.g., web search), an MCP tool (from a connected MCP server), a SKILL.md-based instruction set (imported from registries or git), or a legacy Copilot Extension. When enabled, skill instructions are injected into the system prompt. | "Web Search", "Code Review", "Frontend Design"      |
| **Agent**             | A named persona with a system prompt, a set of assigned skills, and optionally specific MCP server connections. Agents define _how_ the AI behaves and _what tools_ it has access to. Can be created locally or imported from registries/git.                                                                                 | "Research Agent" with web search + URL fetch skills |
| **SKILL.md**          | The open standard for defining AI agent skills. A markdown file with YAML frontmatter (`name`, `description`) and a markdown body containing instructions for the AI. Used by 40+ agent platforms (Claude Code, Codex, Cursor, GitHub Copilot, etc.).                                                                         | See SKILL.md Standard section below                 |
| **Copilot Extension** | A GitHub-hosted plugin/tool. **Note:** GitHub deprecated Extensions in Nov 2025 in favor of MCP. The app should support them if the API still offers them, but prioritize MCP tools as the primary extensibility mechanism.                                                                                                   | `@docker`, `@azure`                                 |
| **MCP Tool**          | A tool exposed by a connected MCP server. Also represented as a Skill in this app.                                                                                                                                                                                                                                            | `query_database`, `search_files`                    |

### SKILL.md Standard

Skills follow the [Agent Skills Specification](https://agentskills.io/specification) — an open standard adopted by 40+ AI agent platforms:

```markdown
---
name: code-review
description: Reviews code for bugs, security issues, and best practices.
license: MIT
metadata:
  author: example-org
  version: "1.0"
compatibility: Works with any chat-based AI agent.
---

# Code Review Skill

When asked to review code, follow these steps:

1. Check for bugs and logic errors
2. Identify security vulnerabilities
3. Suggest performance improvements
4. Ensure code follows best practices

## Output Format

Provide feedback as a numbered list with severity levels.
```

**Required fields:** `name` (1-64 lowercase chars, hyphens allowed), `description` (1-1024 chars)
**Optional fields:** `license`, `compatibility`, `metadata` (key-value), `allowed-tools`

In Chuck, the SKILL.md markdown body becomes the skill's `instructions` field in SQLite. When the skill is enabled on an agent, these instructions are injected into the system prompt alongside the agent's own system prompt.

### Skill & Agent Registries

Chuck can browse, search, and install skills/agents from the aitmpl.com public registry plus arbitrary git URLs:

| Source                                   | What It Provides                                                                                                                                      | API                                                          |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| **[aitmpl.com](https://www.aitmpl.com)** | AI Templates marketplace. 1000+ agents, skills, commands, and MCP integrations. Backed by a catalog of SKILL.md / agent.md files hosted on GitHub.    | Web API (agents + skills catalogs)                           |
| **Git URL**                              | Any git repository containing SKILL.md or \*.agent.md files. Supports GitHub shorthand (`owner/repo`), full URLs, and direct paths to specific files. | GitHub Tree + Contents API (authenticated via Copilot token) |

**Prefetch on startup:** All three registries (MCP, Skills, Agents) are prefetched after authentication completes, so popular items are immediately visible when the user opens any catalog panel — no search required. The aitmpl.com catalog is fetched once (browse-all mode with empty query, sorted by installs descending, top 200) and cached client-side; clearing a search restores the cached browse results instantly.

**Registry caching (Stale-While-Revalidate):** The aitmpl.com `components.json` file is cached in-memory in the Rust backend (`src-tauri/src/registry.rs`) using a stale-while-revalidate (SWR) pattern:

- **First fetch:** synchronous HTTP request; result cached with timestamp
- **Within 1-hour TTL:** return cached data immediately (no network request)
- **After TTL expires:** return stale cached data immediately while spawning a background `tokio::spawn` refresh task
- **Background refresh:** lock-free `AtomicBool` flag prevents duplicate spawns; `ResetOnDrop` guard ensures cleanup on panic
- **Safety limits:** 10 MB max response size; 30-second HTTP timeout; re-checks staleness under write lock before spawning
- **Cache is in-memory only** — not persisted to SQLite; recreated on each app launch

**Installation flow:**

1. User searches or browses the aitmpl.com catalog → sees results sorted by download count
2. Clicks to expand → sees full description and content
3. Clicks "Install" → skill/agent saved to SQLite with content and metadata
4. User can assign the skill to agents → instructions injected into system prompt when active

**Git URL import flow:**

1. User pastes a git URL (e.g., `github/awesome-copilot` or `https://github.com/owner/repo`)
2. App fetches the repo tree via GitHub API (authenticated, progress bar shown)
3. Discovers SKILL.md and \*.agent.md files in all directories
4. Shows list of discovered skills → user selects which to import
5. Selected skills saved to SQLite

### How Agents Map to API Calls

When a conversation uses a custom agent, the Rust backend constructs the Copilot API request as follows:

```
System message = [Agent system prompt]
               + [Enabled skill instructions (concatenated)]
               + [Project instructions (if any)]
Tools/functions = [Agent's assigned skills as function definitions]
                + [MCP tools from agent's connected MCP servers]
Messages = [Conversation history]
```

- The agent's system prompt is prepended as a `system` role message
- Enabled skill instructions are appended to the system message (each skill's `instructions` field)
- Skills are exposed as `tools` / `functions` in the API request (OpenAI function calling format)
- When the AI calls a tool, the Rust backend routes it: Copilot Extensions → GitHub API, MCP tools → MCP server, built-in tools (web search) → web-research crate
- Tool results are sent back as `tool` role messages in the next API call

### Slash Commands (Client-Side)

Slash commands are processed entirely in the frontend (`src/lib/utils/slash-commands.ts` + `InputArea.svelte`). They do **not** create new Tauri commands — they invoke existing ones or perform UI actions directly.

| Command       | Alias  | Action                                     | Delegates to                                                |
| ------------- | ------ | ------------------------------------------ | ----------------------------------------------------------- |
| `/help`       | `/?`   | Open command reference modal               | Frontend-only (modal)                                       |
| `/delete`     | —      | Clear all messages in current conversation | `deleteConversation` + `createConversation`                 |
| `/title`      | —      | Regenerate conversation title              | `generate_title` Tauri command                              |
| `/export`     | —      | Export conversation as JSON or Markdown    | `export_conversation_json` / `export_conversation_markdown` |
| `/fetch`      | `/web` | Toggle web search for the next message     | Frontend toggle (`webSearchEnabled`)                        |
| `/model`      | —      | Select model for this message only         | Frontend UI (model dropdown)                                |
| `/edit`       | —      | Edit the last sent user message            | Frontend action (enter edit mode)                           |
| `/regenerate` | —      | Regenerate the last assistant response     | `send_message` with last user message                       |

**Popup behavior:** typing `/` at the start of an empty input opens the slash command popup. Further typing filters commands (e.g., `/he` → `/help`). ↑/↓ arrows navigate with wrap-around; Tab or click accepts; Escape dismisses. Aliases match partial prefixes (e.g., `/w` matches `/fetch` via its `/web` alias).

**@-mentions:** typing `@` opens an agent autocomplete popup. Selecting an agent overrides the conversation's agent for that single message only.

### Default Agent

The app ships with a **Default Agent** that cannot be deleted:

- System prompt: minimal (just app context)
- Skills: none by default (user can assign)
- MCP connections: none by default
- All new conversations use the Default Agent unless the user selects another

---

## Git Sources & Unified Catalog

### Overview

Git Sources provide persistent git repository URL sources for importing skills and agents. Instead of one-off "Import from Git URL" actions, users manage a list of git repository sources that are automatically synced on app launch and can be browsed alongside the aitmpl.com registry in a unified catalog.

**aitmpl.com** is a built-in source that is always present and can be toggled on/off but not deleted. Its enabled state is stored as the `aitmpl_enabled` setting (default: `true`).

### Source Lifecycle

1. **Add source** — `create_git_source(url, name?)` scans the repo for `SKILL.md` and `*.agent.md` files, stores discovered items in `git_source_items`, and returns a `SourceScanResult`
2. **Auto-sync on launch** — `sync_all_sources()` syncs all enabled sources in parallel using `tokio::task::JoinSet`. Per-source progress events (`git-import-progress`) are emitted during fetch. A `git-source-sync-complete` event fires after each source's DB update commits.
3. **Manual re-sync** — `sync_git_source(id)` re-scans a single source
4. **Toggle** — `update_git_source(id, name?, enabled?)` to pause/resume syncing
5. **Remove** — `delete_git_source(id)` removes the source; imported skills/agents are kept as local copies (`ON DELETE SET NULL` on FK); cached catalog items are deleted (`ON DELETE CASCADE` on `git_source_items`)

### Unified Catalog Search

`search_catalog(query, kind?, limit?, source_ids?)` merges results from:

- **aitmpl.com API** — when `aitmpl_enabled` setting is `true` and `source_ids` includes `"aitmpl"` (or is null for all)
- **git_source_items table** — cached discovery data from synced git sources, filtered by `source_ids` (UUIDs)

**Parameters:**

- `query` (string) — search term; LIKE wildcards (`%`, `_`, `\`) are escaped
- `kind` (optional: `"skill"` | `"agent"`) — filter by item type
- `limit` (optional, default 20) — max results
- `source_ids` (optional: `string[]`) — multi-select filter; `null` = all sources; includes `"aitmpl"` for the built-in registry and git source UUIDs for specific repos

**Response:** `{ items: RegistryItem[], total: number | null }`

Content is omitted from search results to reduce IPC payload. Full content is loaded on-demand via `install_catalog_item(item_id)`.

### Catalog Item IDs

Git catalog items use deterministic IDs: `gsi-{full_source_id}-{file_path}`. This enables upsert on re-sync and allows the frontend to check installation status by matching the file path portion.

### Events

| Event                      | Payload                               | When                                           |
| -------------------------- | ------------------------------------- | ---------------------------------------------- |
| `git-import-progress`      | `{ total, fetched, phase, sourceId }` | During source scan/sync (per file fetched)     |
| `git-source-sync-complete` | `{ sourceId }`                        | After a source's sync is fully committed to DB |

---
