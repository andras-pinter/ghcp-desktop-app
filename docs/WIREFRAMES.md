# Chuck — UX Wireframes

> Part of the [Chuck project documentation](../AGENTS.md). These wireframes are extracted from
> the main AGENTS.md for context efficiency.

---

## UX Wireframes

> These wireframes are the **canonical reference** for layout, component placement, and interaction
> design. When implementing a view, match this structure exactly. If a wireframe conflicts with
> prose elsewhere in the project documentation, the wireframe wins.

### 1. Main Layout (Sidebar + Chat)

The primary app surface. Sidebar is collapsible (Cmd+Shift+S). Chat fills remaining width.

````
┌─────────────────────────────────────────────────────────────────────────────┐
│ ● ● ●                        Chuck                              ─ □ ✕    │
├────────────────────┬────────────────────────────────────────────────────────┤
│                    │  📝 API Design Q              ⟳ Regenerate   ⋮ Menu    │
│  [+ New Chat]      ├────────────────────────────────────────────────────────┤
│                    │                                                        │
│  🔍 Search...      │  ┌─ Earlier messages summarized ──────────────────┐   │
│                    │  │  ℹ️ 12 older messages condensed into summary   │   │
│  ★ FAVOURITES      │  └────────────────────────────────────────────────┘   │
│  ├─ API Design Q   │                                                        │
│  └─ Rust Macros    │  ┌─────────────────────────────────────────────────┐   │
│                    │  │ 👤 You                              12:34 PM  ✏️│   │
│  📁 PROJECTS       │  │                                                 │   │
│  ├─ Copilot App    │  │  How do I parse JSON in Rust?                   │   │
│  │  ├─ Chat 1      │  └─────────────────────────────────────────────────┘   │
│  │  └─ Chat 2      │                                                        │
│  └─ Blog Engine    │  ┌─────────────────────────────────────────────────┐   │
│                    │  │ 🤖 Copilot                          12:34 PM  📋│   │
│  🤖 AGENTS         │  │                                                 │   │
│  ├─ Research       │  │  ▶ Thinking... (click to expand)                │   │
│  ├─ Code Review    │  │                                                 │   │
│  └─ + New Agent    │  │  You can use **serde** for JSON parsing:        │   │
│                    │  │                                                 │   │
│  📅 TODAY           │  │  ```rust                                        │   │
│  ├─ Chat about X   │  │  use serde::Deserialize;                        │   │
│  └─ Debug help     │  │                                         [Copy]  │   │
│                    │  │  #[derive(Deserialize)]                          │   │
│  📅 YESTERDAY       │  │  struct Config {                                │   │
│  ├─ Chat about Y   │  │      name: String,                              │   │
│  └─ Refactor Q     │  │  }                                              │   │
│                    │  │  ```                                             │   │
│  📅 LAST 7 DAYS     │  │                                                 │   │
│  └─ Old chat       │  │  🌐 Web: serde.rs — Official Docs              │   │
│                    │  └─────────────────────────────────────────────────┘   │
│                    │                                                        │
│                    │  ┌─────────────────────────────────────────────────┐   │
│  ──────────────    │  │ 👤 You                              12:35 PM  ✏️│   │
│  ⚡ Skills          │  │                                                 │   │
│  ⚙️ Settings        │  │  📎 config.json (2.1 KB)                        │   │
│                    │  │  Can you parse this file for me?                 │   │
│                    │  └─────────────────────────────────────────────────┘   │
│                    │                                                        │
│                    ├────────────────────────────────────────────────────────┤
│                    │  ┌── Input Area (see wireframe 2) ────────────────┐   │
│                    │  │                                                 │   │
│                    │  └─────────────────────────────────────────────────┘   │
├────────────────────┴────────────────────────────────────────────────────────┤
│  🟢 Online │ DB: 12.3 MB │ v1.2.0                                          │
└─────────────────────────────────────────────────────────────────────────────┘
````

**Key behaviors:**

- Sidebar width: ~260px, resizable, collapsible
- Favourites pinned at top, then Projects, then Agents, then date-grouped history
- Right-click sidebar conversation → context menu: Rename, Export (JSON/Markdown), Toggle Favourite ★, Delete
- ★ icon appears on hover of conversation items; click to toggle favourite
- Header shows conversation title (click to inline-edit); auto-generated title can be overridden
- Chat scrolls independently; auto-scroll on new tokens, pauses if user scrolled up
- Floating `↓` scroll-to-bottom button appears when user has scrolled away from the bottom; click smooth-scrolls to latest message
- Existing conversations auto-scroll to the bottom when opened
- Message actions appear on hover: ✏️ edit (user msgs), 📋 copy, ⟳ regenerate (last assistant msg)
- Code block [Copy] button always visible; message-level 📋 copy appears on hover
- Cmd+F activates `SearchOverlay.svelte`: floating search bar at top of chat with match count, ↑/↓ arrows, Escape to dismiss
- Context summarization banner is dismissible but not deletable
- Status bar shows: connection state, DB size, app version

### 2. Input Area (Detail)

Multi-line input with attachment support, agent/model selection, slash commands, and action buttons.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  📎 config.json ✕ │ 📎 schema.sql ✕ │ 🌐 https://docs.rs/serde ✕     │  ← attachment pills
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  /                                                                      │  ← typing "/" triggers
│                                                                         │     slash command popup
│  ┌── / Commands ─────────────────────────┐                              │
│  │  /help · /?       Command reference   │  ← highlighted (selected)   │
│  │  /delete          Clear conversation  │                              │
│  │  /title           Regenerate title    │                              │
│  │  /export          Save conversation   │                              │
│  │  /fetch · /web    Toggle web search   │  ← alias shown inline       │
│  │  /model           Per-message model   │                              │
│  │  /edit            Edit last message   │                              │
│  │  /regenerate      Resend last reply   │                              │
│  └───────────────────────────────────────┘                              │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  📎 Attach  🌐 Web  │  Agent: Research ▾  │  Model: GPT-4o ▾  │ Send ➤ │  ← toolbar
└─────────────────────────────────────────────────────────────────────────┘

Drag-and-drop zone covers entire input area (visual highlight on drag-over)
```

**Key behaviors:**

- Input auto-grows up to ~6 lines, then scrolls internally
- Enter sends (default); Shift+Enter for newline. Configurable in Settings to use Cmd+Enter (Ctrl+Enter) to send instead (Enter always inserts newline in that mode)
- **Slash commands:** typing `/` at the start of empty input opens a popup with all available commands. Typing further filters the list (e.g., `/he` matches `/help`). ↑/↓ to navigate (wraps around), Tab or click to accept, Escape to dismiss. Aliases shown inline (e.g., `/fetch · /web`). `/?` alias for `/help`.
- **@-mentions:** typing `@` shows an agent autocomplete popup. Selecting an agent sets it as an override for the current message only.
- Attachment pills show filename + size + ✕ remove button
- URL pills show favicon + domain + ✕ remove; content fetched on paste
- 📎 opens native file picker (`tauri-plugin-dialog`)
- 🌐 toggles a URL input field inline
- Agent dropdown shows custom agents + "Default"; changing mid-conversation warns
- Model dropdown populated from API; disabled if only one model
- Send button disabled when input is empty or offline

### 3. Auth / Login Screen

Shown on first launch or after token expiry. Replaces the entire main area.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                                                                             │
│                          ┌───────────────────────┐                          │
│                          │                       │                          │
│                          │    🐙  Copilot         │                          │
│                          │       Desktop          │                          │
│                          │                       │                          │
│                          │  Sign in with GitHub   │                          │
│                          │  to get started.       │                          │
│                          │                       │                          │
│                          │  ┌─────────────────┐  │                          │
│                          │  │  Sign in with    │  │                          │
│                          │  │  GitHub    →     │  │  ← primary action button │
│                          │  └─────────────────┘  │                          │
│                          │                       │                          │
│                          │  ── or enter code ──  │                          │
│                          │                       │                          │
│                          │  Your code: ABCD-1234 │  ← device code           │
│                          │  ┌─────────────────┐  │                          │
│                          │  │  Copy Code  📋   │  │                          │
│                          │  └─────────────────┘  │                          │
│                          │                       │                          │
│                          │  Waiting for auth...  │  ← spinner / polling     │
│                          │  ●●○○ ←──────────→    │                          │
│                          │                       │                          │
│                          │  Requires an active   │                          │
│                          │  Copilot subscription  │                          │
│                          │                       │                          │
│                          └───────────────────────┘                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**

- "Sign in" opens browser via `tauri-plugin-shell` `open()` to GitHub device auth URL
- Device code shown immediately; user can copy to clipboard
- Polling indicator shows auth check in progress
- On success: transition to main chat with welcome toast
- On failure/timeout: show error with retry button
- No sidebar visible during auth flow

### 4. Settings Panel

Slides over from the right or opens as a modal. Tabbed navigation.

```
┌──────────────────────────────────────────────────────────────────┐
│  ‹                         Settings                              │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ── Account ──────────────────────────────────────────────────   │
│                                                                  │
│  Signed in as                   @octocat                         │
│  Copilot Plan                   Copilot Pro                      │
│                                                                  │
│  [ Sign Out ]                                                    │
│                                                                  │
│  ── Appearance ───────────────────────────────────────────────   │
│                                                                  │
│  Theme                          [ System ▾ ]                     │
│                                  System / Light / Dark           │
│                                                                  │
│  Font Size                      [ 14px  ▾ ]                      │
│                                  12 / 13 / 14 / 15 / 16         │
│                                                                  │
│  ── Input ────────────────────────────────────────────────────   │
│                                                                  │
│  Send Message With              ( ● ) Enter                      │
│                                 (   ) Cmd+Enter (Ctrl+Enter)     │
│                                                                  │
│  ── Defaults ─────────────────────────────────────────────────   │
│                                                                  │
│  Default Model                  [ GPT-4o ▾ ]                     │
│  Default Agent                  [ Default ▾ ]                    │
│                                                                  │
│  ── Auto-Update ──────────────────────────────────────────────   │
│                                                                  │
│  Check for updates              [✓]  Enabled                     │
│  Check frequency                [ On startup ▾ ]                 │
│                                  On startup / Daily / Weekly     │
│                                                                  │
│  ── Keyboard Shortcuts ───────────────────────────────────────   │
│                                                                  │
│  New Chat                        ⌘ N                              │
│  Search Conversations            ⌘ K                              │
│  Search in Conversation          ⌘ F                              │
│  Toggle Sidebar                  ⌘ ⇧ S                           │
│  Settings                        ⌘ ,                              │
│                                                                  │
│  ── Global Hotkey ────────────────────────────────────────────   │
│                                                                  │
│  Summon Chuck                    [ Cmd+Shift+Space ]              │
│                                  (click to rebind)               │
│                                                                  │
│  ── Storage ──────────────────────────────────────────────────   │
│                                                                  │
│  Database Size                  12.3 MB                          │
│                                                                  │
│  ── Cleanup ──────────────────────────────────────────────────   │
│                                                                  │
│  Delete old conversations       [ Older than 90 days ▾ ]         │
│                                 [ Delete Now ]                   │
│                                                                  │
│  ⚠️ Database is 487 MB — consider cleaning up old conversations  │
│                                 (shown when > 400MB)             │
│                                                                  │
│  ── Export ───────────────────────────────────────────────────   │
│                                                                  │
│  Export All Conversations       [ JSON ] [ Markdown ]            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**

- Single scrollable page with section headings (no tabs)
- Changes apply immediately (no save button); persisted to SQLite `config` table
- Hotkey rebind: click field → "Press new shortcut..." → capture next key combo
- Sign Out clears keychain + redirects to auth screen
- Delete confirmation dialog before destructive actions
- Export opens native save dialog (`tauri-plugin-dialog`)

### 5. Agents Management

Accessed from sidebar "Agents" section or Settings.

```
┌──────────────────────────────────────────────────────────────────┐
│  ‹                          Agents                               │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 🔵 Default Agent                              [built-in]  │  │
│  │    General-purpose Copilot assistant.                      │  │
│  │    Skills: none │ MCP: none                                │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 🔬 Research Agent                          [Edit] [Delete] │  │
│  │    Deep research with web search and citations.            │  │
│  │    Skills: web_search, fetch_url │ MCP: none               │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 💻 Code Review Agent                       [Edit] [Delete] │  │
│  │    Thorough code review with best practices.               │  │
│  │    Skills: none │ MCP: GitHub                              │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐  │
│  │              + Create New Agent                            │  │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

┌── Create/Edit Agent ─────────────────────────────────────────────┐
│                                                                  │
│  Avatar                         [ 🔬 ▾ ]  (emoji picker)        │
│                                                                  │
│  Name                           [ Research Agent          ]      │
│                                                                  │
│  System Prompt                                                   │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ You are a research assistant. When asked a question,       │  │
│  │ always search the web first for up-to-date information.    │  │
│  │ Cite your sources with URLs. Be thorough and balanced.     │  │
│  │                                                            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ── Assigned Skills ──────────────────────────────────────────   │
│  [✓] web_search        Web search via Bing/Google API            │
│  [✓] fetch_url         Fetch and extract URL content             │
│  [ ] mcp_query_db      Query database (PostgreSQL MCP)           │
│  [ ] mcp_github_pr     Look up GitHub PRs (GitHub MCP)           │
│                                                                  │
│  ── MCP Connections ──────────────────────────────────────────   │
│  [ ] GitHub MCP Server          (connected ✓)                    │
│  [ ] PostgreSQL MCP Server      (connected ✓)                    │
│  [ ] Local FS MCP Server        (disconnected ✗)                 │
│                                                                  │
│                                    [ Cancel ]  [ Save Agent ]    │
└──────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**

- Default Agent card has no Edit/Delete — it's built-in and immutable
- Edit opens inline form (same screen, replaces list) or slide-over
- System prompt is a textarea with syntax hints
- Skills list populated from registered skills (extensions + MCP tools + registry-imported + git-imported)
- MCP connections list populated from configured MCP servers
- Agent deletion requires confirmation; orphaned conversations keep agent name as text
- Source badge on imported agents: "aitmpl.com" or "git" with link to origin
- **Browse Registry** section at bottom: search aitmpl.com for pre-built agent templates, one-click import
- **Import from Git** field: paste a git URL (e.g., `owner/repo`), click Fetch to discover agent definitions

### 6. Skills Panel

Browse and manage all available skills (built-in + MCP tools + registry-imported + git-imported).

```
┌──────────────────────────────────────────────────────────────────┐
│  ‹                          Skills                               │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  🔍 Filter skills...                                             │
│                                                                  │
│  ── Built-in ─────────────────────────────────────────────────   │
│                                                                  │
│  [✓]  🌐 Web Search                                              │
│       Search the web via Bing/Google API                         │
│       Source: built-in                                            │
│                                                                  │
│  [✓]  🔗 URL Fetcher                                              │
│       Fetch and extract readable content from URLs               │
│       Source: built-in                                            │
│                                                                  │
│  ── MCP Tools (GitHub Server) ────────────────────────────────   │
│                                                                  │
│  [✓]  📋 search_repos                                             │
│       Search GitHub repositories by query                        │
│       Source: MCP · GitHub                                        │
│                                                                  │
│  ── Registry / Git Imported ──────────────────────────────────   │
│                                                                  │
│  [✓]  🎨 frontend-design                                          │
│       Create production-grade frontend interfaces                │
│       Source: aitmpl.com · vercel-labs/agent-skills               │
│                                                                  │
│  [ ]  📝 code-review                                               │
│       Review code for bugs and best practices                    │
│       Source: git · github.com/acme/skills                       │
│                                                                  │
│  ── Copilot Extensions ───────────────────────────────────────   │
│                                                                  │
│  [ ]  🐳 @docker                                                  │
│       Docker container management and debugging                  │
│       Source: extension                                           │
│                                                                  │
│  ═══════════════════════════════════════════════════════════════  │
│                                                                  │
│  ── Browse Registry ──────────────────────────────────────────   │
│                                                                  │
│  🔍 Search aitmpl.com...                                           │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  🎯 frontend-design         aitmpl.com   218K installs   │    │
│  │     Create distinctive frontend interfaces           [+] │    │
│  ├──────────────────────────────────────────────────────────┤    │
│  │  🔬 deep-research            aitmpl.com                   │    │
│  │     Research assistant with citations                 [+] │    │
│  ├──────────────────────────────────────────────────────────┤    │
│  │  📊 data-analysis            aitmpl.com   45K installs    │    │
│  │     Analyze datasets and generate insights            [+] │    │
│  └──────────────────────────────────────────────────────────┘    │
│           ... (infinite scroll loads more) ...                   │
│                                                                  │
│  ── Import from Git ──────────────────────────────────────────   │
│                                                                  │
│  [ owner/repo or full git URL          ]  [ Fetch ]              │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**

- Toggle checkbox to enable/disable a skill globally
- Skills grouped by source: Built-in → MCP (per server) → Registry/Git Imported → Extensions
- Configure button opens per-skill settings (e.g., connection string)
- Filter field does fuzzy search across name + description (local skills only)
- Disabled MCP skills (server disconnected) shown grayed out with status
- Skills assigned to agents are marked but can be toggled independently here
- **Browse Registry** section: search aitmpl.com catalog
  - Results show install count and one-click [+] install button
  - Expandable cards show full description before installing
  - Installing fetches the SKILL.md content, parses it, and saves to SQLite
  - Infinite scroll pagination for registry results
- **Import from Git** section: text field for git URL + Fetch button
  - Accepts: `owner/repo`, full GitHub URLs, direct paths to SKILL.md or \*.agent.md
  - Fetch discovers SKILL.md files → shows selection dialog → import selected skills
  - Imported skills show git source badge with link to origin

### 7. MCP Settings

Manage MCP server connections. Accessed from Settings > MCP tab.

```
┌──────────────────────────────────────────────────────────────────┐
│  ── Connected Servers ────────────────────────────────────────   │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 🟢 GitHub MCP Server                [Test] [Edit] [Remove] │  │
│  │    Transport: HTTP                                         │  │
│  │    URL: https://api.github.com/mcp                         │  │
│  │    Tools: 12 discovered │ Source: registry                 │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 🟢 PostgreSQL                       [Test] [Edit] [Remove] │  │
│  │    Transport: Stdio                                        │  │
│  │    Binary: /usr/local/bin/pg-mcp-server                    │  │
│  │    Tools: 3 discovered │ Source: custom                    │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ 🔴 Brave Search                     [Test] [Edit] [Remove] │  │
│  │    Transport: HTTP │ Status: connection failed              │  │
│  │    URL: https://brave-mcp.example.com                      │  │
│  │    Tools: — │ Source: registry                              │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ── MCP Registry ─────────────────────────────────────────────   │
│                                                                  │
│  🔍 Search registry...                                           │
│                                                                  │
│  [ ] Azure MCP Server           Stdio  │ [Add]                   │
│  [ ] GitHub MCP Server          HTTP   │ [Add]                   │
│  [ ] Brave Search               HTTP   │ [Add]                   │
│           ... (infinite scroll loads more) ...                   │
│                                                                  │
│  ── Add Custom Server ────────────────────────────────────────   │
│                                                                  │
│  Name        [ My Custom Server          ]                       │
│  Transport   ( ) HTTP    (●) Stdio                               │
│                                                                  │
│  Binary Path [ /path/to/mcp-server       ]  [Browse]             │
│  Arguments   [ --port 3000 --verbose     ]                       │
│                                                                  │
│                          [ Test Connection ]  [ Add Server ]     │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**

- 🟢/🔴 indicator shows live connection status
- Test button sends a ping and shows result inline
- Stdio servers show first-launch confirmation dialog
- Registry entries pre-fill config; one-click add for npm/pypi servers
- Browse button for binary path opens native file picker
- Remove requires confirmation; warns if agents reference this server
- Discovered tools count updates after successful connection

### 8. Offline / Error States

```
┌── Offline Banner (top of chat) ──────────────────────────────────┐
│  ⚠️  You're offline. Conversations are read-only.    [Dismiss]   │
└──────────────────────────────────────────────────────────────────┘

┌── Input Area (offline) ──────────────────────────────────────────┐
│                                                                  │
│  📡 You're offline — sending is disabled                         │  ← grayed out
│                                                                  │
│  📎 Attach  🌐 Web  │  Agent: Research ▾  │  Model: GPT-4o ▾  │ ▨ │  ← send disabled
└──────────────────────────────────────────────────────────────────┘

┌── Rate Limit Toast ──────────────────────────────────────────────┐
│  ⏳ Rate limit reached. Retry in 32s...              [Dismiss]   │
└──────────────────────────────────────────────────────────────────┘

┌── Update Banner (below title bar) ───────────────────────────────┐
│  🎉 Version 1.3.0 available!  [View Changes]  [Update Now]      │
│                                                [Skip] [Later]    │
└──────────────────────────────────────────────────────────────────┘

┌── Streaming Interrupted ─────────────────────────────────────────┐
│  🤖 Copilot                                        12:34 PM     │
│                                                                  │
│  Here's how you can implement th—                                │
│                                                                  │
│  ⚠️ (response interrupted)                      [⟳ Regenerate]  │
└──────────────────────────────────────────────────────────────────┘
```

### 9. Thinking / Reasoning Display

```
┌── Collapsed (default) ───────────────────────────────────────────┐
│  🤖 Copilot                                        12:34 PM     │
│                                                                  │
│  ▶ Thinking... (2.3s)                          ← click to expand │
│                                                                  │
│  The answer to your question is that serde provides...           │
└──────────────────────────────────────────────────────────────────┘

┌── Expanded ──────────────────────────────────────────────────────┐
│  🤖 Copilot                                        12:34 PM     │
│                                                                  │
│  ▼ Thinking (2.3s)                             ← click to collapse│
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐ │
│  │ The user is asking about JSON parsing in Rust. I should    │ │
│  │ recommend serde as it's the de facto standard. Let me also │ │
│  │ mention serde_json specifically and show a derive example  │ │
│  │ since that's the most common pattern...                    │ │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │
│                                                                  │
│  The answer to your question is that serde provides...           │
└──────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**

- Collapsed by default; shows elapsed thinking time
- Thinking section styled distinctly: muted text, dashed border, indented
- During streaming: shows a pulsing copper orb with a random aviation-themed catchphrase until streaming completes
- After completion: shows "Thinking (Xs)" with final elapsed time
- User preference in settings to default-expand (for power users)
