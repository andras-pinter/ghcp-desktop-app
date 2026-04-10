# Chuck — Agent Instructions

> **Chuck** (named after Chuck Yeager) is a native, cross-platform desktop chat GUI for GitHub Copilot, built with **Tauri v2 + Svelte 5 + TypeScript**.
> Inspired by Claude Desktop's chat experience — with web research, MCP tools, custom agents, and a strict no-machine-access security model.

---

## ⛔ MANDATORY: Agent Task Completion Protocol

> **🚨 STOP. READ THIS BEFORE DOING ANYTHING.**
>
> Every agent working on this project **MUST** follow the rules below.
> These rules apply to **ALL tasks** — code, documentation, configuration, dependencies, refactoring, **everything**.
> There are **ZERO exceptions**. "It's just a docs change" is not an excuse. "It's a small fix" is not an excuse.
> **No task is complete until the review-fix loop exits with zero issues.**
>
> If you skip this protocol, your work is considered incomplete and invalid.

### 1. Review-Fix Loop (Zero Issues Required)

After completing **any** task — no matter how small — the agent **MUST** run a review-fix cycle:

```
┌─────────────────────────┐
│   Complete the task      │
└────────────┬────────────┘
             ▼
┌─────────────────────────────────┐
│   REVIEW all changes:           │
│                                 │
│   Rust backend:                 │
│   - cargo build --workspace     │
│   - cargo clippy --workspace -- -D warnings │
│   - cargo test --workspace      │
│   - cargo fmt --all -- --check  │
│   - cargo audit                 │
│                                 │
│   Frontend:                     │
│   - pnpm check                  │
│   - pnpm lint                   │
│   - pnpm test                   │
│   - pnpm build                  │
│                                 │
│   Manual:                       │
│   - Code review                 │
│   - Security audit              │
│   - Doc completeness            │
│   - Dependency check            │
└────────────┬────────────────────┘
             ▼
        ┌─────────┐     YES     ┌─────────────────┐
        │ Issues? │────────────▶│  FIX all issues  │──┐
        └────┬────┘             └─────────────────┘  │
             │ NO                                     │
             ▼                                        │
┌─────────────────────────┐          ┌────────────────┘
│   ✅ Task complete       │          │
│   (0 issues confirmed)   │          ▼
└──────────────────────────┘    (loop back to REVIEW)
```

**The loop MUST repeat until a full review pass finds ZERO issues.** There is no
"good enough" — the review cycle terminates only at zero. This process can and should
be dispatched to multiple agents in parallel (e.g., one agent reviews code quality,
another reviews tests, another reviews docs).

> **⚠️ Common excuses that DO NOT exempt you from this protocol:**
> - "It's only a documentation change" — **No.** Review for consistency, broken links, contradictions, missing updates.
> - "It's a one-line fix" — **No.** One-line fixes can introduce regressions. Review.
> - "I already know it's correct" — **No.** Run the checks anyway. Trust the process, not your assumptions.
> - "The build tools don't apply (no code yet)" — **Partially valid.** Skip cargo/pnpm checks only if there is literally no code in the repo. Still run manual review, security audit, doc completeness, and consistency checks.
> - "I'll do it later" — **No.** The review happens NOW, before the task is marked complete.

### Review Checklist (every cycle)

**Rust backend:**

- [ ] `cargo build --workspace` compiles with zero warnings
- [ ] `cargo clippy --workspace -- -D warnings` passes with zero diagnostics
- [ ] `cargo test --workspace` — all tests pass, no skipped tests without justification
- [ ] `cargo fmt --all -- --check` — formatting is clean
- [ ] `cargo audit` — no known vulnerabilities

**Frontend:**

- [ ] `pnpm check` — svelte-check passes with zero errors
- [ ] `pnpm lint` — ESLint + Prettier pass with zero issues
- [ ] `pnpm test` — all Vitest tests pass
- [ ] `pnpm build` — Vite production build succeeds
- [ ] `pnpm audit` — no known vulnerabilities in npm dependencies

**Manual:**

- [ ] **Code review** — logic is correct, no dead code, no TODOs left behind, no hardcoded values
- [ ] **Security review** — no filesystem access beyond app data dir, no token leaks, no unsanitized inputs, Tauri capabilities are minimal
- [ ] **Error handling** — all error paths handled, user-friendly messages, no panics/unwraps in production Rust code
- [ ] **Documentation** — all public Rust items have doc comments, JSDoc on exported TS functions, README/AGENTS.md updated if needed
- [ ] **Tests** — new code has tests, edge cases covered, integration tests for API interactions
- [ ] **Dependencies** — all Rust crates and npm packages are latest stable, actively maintained, no deprecated packages

### 2. Update Everything

When an agent completes a task, it **MUST** update **all** affected artifacts:

- **Code** — the implementation itself (Rust and/or Svelte/TypeScript)
- **Tests** — new/updated tests covering the changes (Rust unit tests + Vitest frontend tests)
- **Documentation** — doc comments, JSDoc, README.md, AGENTS.md (if architecture/scope/phases changed)
- **Dependencies** — Cargo.toml and/or package.json updated, lock files committed
- **Types** — all type definitions updated consistently across Rust types AND TypeScript types
- **State** — Svelte runes (`$state`, `$derived`), Tauri managed state, SQLite schemas, config structures updated
- **Components** — any Svelte component that references changed state/types must be updated
- **Sibling crates** — if a change in `copilot-api` affects `src-tauri`, update `src-tauri` too
- **Tauri commands** — if Rust types change, update corresponding Tauri command signatures AND frontend `invoke()` calls
- **Plan** — if the task reveals new work or changes scope, update the plan

**"Update everything" means: no partial changes.** If you modify a type in `copilot-api/types.rs`,
you MUST also update the corresponding TypeScript type, every Tauri command that uses it, and every
Svelte component that consumes it. If you add a new feature, you MUST add it to the settings UI,
keyboard shortcuts, and documentation. If you rename something, you MUST rename it everywhere.
Agents must grep/search the entire workspace to find all references before considering a change complete.

### 3. Multi-Agent Review Dispatch

For any non-trivial task, the review cycle SHOULD be split across multiple agents:

| Review Agent | Responsibility |
|---|---|
| **Build Agent** | Compile (cargo + pnpm), clippy, fmt, svelte-check, lint, test, audit — mechanical correctness |
| **Code Review Agent** | Logic, architecture, patterns, dead code, consistency across Rust + Svelte |
| **Security Agent** | Tauri capabilities audit, filesystem isolation, token handling, input sanitization, CSP, network boundaries |
| **Docs Agent** | Doc comments, JSDoc, README, AGENTS.md, inline comments where needed |

Each agent independently reviews and reports issues. ALL reported issues must be fixed
before the task is considered complete. Then the full review cycle runs again.

### 4. Pre-Merge / Finalization Review

When the user requests finalization, merge preparation, or uses phrases like *"let's prepare for merge"*, *"let's finalize"*, *"give it a final review"*, or *"prepare for merging"*, the agent **MUST** trigger a **full extensive review cycle** — regardless of whether the agent believes the code is already clean.

**This is NOT the same as the per-task review in section 1.** The pre-merge review is a comprehensive, branch-wide audit:

1. **Run the full check suite** (all Rust + frontend checks from the Review Checklist above)
2. **Dispatch parallel review agents** (at minimum: Security Agent + Code Review Agent)
3. **Fix ALL reported issues** — no deferral, no "cosmetic only" exceptions
4. **Re-run the full check suite + re-dispatch review agents**
5. **Repeat steps 2–4 until BOTH review agents report zero issues**

```
User says "prepare for merge / finalize"
              │
              ▼
┌─────────────────────────────────┐
│  Run full check suite           │
│  (cargo + pnpm, all checks)    │
└────────────┬────────────────────┘
             ▼
┌─────────────────────────────────┐
│  Dispatch review agents:        │
│  • Security Agent               │
│  • Code Review Agent            │
│  (+ Docs Agent if applicable)   │
└────────────┬────────────────────┘
             ▼
        ┌─────────┐     YES     ┌──────────────────┐
        │ Issues? │────────────▶│  FIX all issues   │──┐
        └────┬────┘             │  Commit fixes     │  │
             │ NO               │  Re-run checks    │  │
             ▼                  └──────────────────┘  │
┌─────────────────────────┐          ┌────────────────┘
│  ✅ Branch ready for     │          │
│  merge (0 issues across  │          ▼
│  ALL review agents)      │    (loop back to dispatch)
└──────────────────────────┘
```

**The loop terminates ONLY when all dispatched review agents independently report zero findings.** One clean pass is not enough if a fix introduced a new issue — the full cycle must re-run.

---

## Project Overview

Chuck is a standalone desktop application that provides a conversational chat interface
for GitHub Copilot. Think of it as "Claude Desktop, but for Copilot" — a polished, native desktop
app with conversation management, file attachments, projects, web research, MCP tool integration,
custom agent personas, and streaming responses.

### Stack

- **Backend:** Rust (2021 edition) via [Tauri v2](https://v2.tauri.app/) — handles API calls, MCP, persistence, security
- **Frontend:** [Svelte 5](https://svelte.dev/) + TypeScript — UI components, state management, user interactions
- **Build Tool:** [Vite](https://vite.dev/) — frontend bundling and dev server
- **Backend API:** GitHub Copilot `/v1/chat/completions` (OAuth token-based, SSE streaming)
- **Storage:** SQLite (via `rusqlite` in Rust backend) for local conversation persistence
- **Platforms:** macOS (WebKit), Linux (WebKitGTK), Windows (WebView2) — all first-class via Tauri

### Why Tauri v2

- **Production-ready** — stable API, battle-tested, large community
- **True cross-platform** — consistent behavior on macOS, Linux, Windows via system webview
- **Built-in features** — system tray, global shortcuts, auto-updater, dialogs, notifications, clipboard — no custom implementation needed
- **Security-first** — capabilities system, CSP, IPC permissions align perfectly with our no-machine-access requirement
- **Small bundle** — uses system webview (no bundled Chromium), resulting in ~5-10MB app vs ~150MB+ Electron
- **Rich UI** — full HTML/CSS/JS means world-class UI toolkit, accessibility, animation, and styling
- **App Sandbox** — native macOS sandbox support, plus Tauri's own capability-based security layer

---

## Scope

### In Scope

- **Conversation sidebar** — list of past conversations, search, new chat button, date grouping
- **Chat window** — streaming message display with markdown + syntax-highlighted code blocks
- **File attachments** — drag-and-drop files into chat as context (text, PDF, images)
- **Projects** — group conversations + attached files under named projects with custom instructions
- **Web research** — AI-driven web search (via search API) + manual URL fetching/extraction for context
- **MCP integration** — connect to MCP servers for extended tool capabilities; browse the official MCP Registry + custom server configuration
- **Skills management** — enable/disable/configure skills that extend what Copilot can do in conversations. Skills can come from MCP tools, built-in capabilities, or external registries. Browse and install skills from the **aitmpl.com** registry.
- **Agents management** — create custom agent personas with specific system prompts, assigned skills, and MCP connections. Browse and install pre-built agent templates from the **aitmpl.com** registry.
- **Git Sources** — manage persistent git repository sources for importing skills and agents. Add repo URLs, scan for SKILL.md/AGENT.md files, pick items to import, toggle sources on/off, auto-sync on app launch, manual re-sync. Removing a source keeps imported items as local copies.
- **Model selector** — pick from available Copilot models (implement always; gracefully hide if API returns only one model)
- **Light/dark theme** — follow system preference, manual toggle (CSS custom properties)
- **Global hotkey** — summon the app from anywhere (e.g., Cmd+Shift+Space) via `tauri-plugin-global-shortcut`
- **Keyboard shortcuts** — standard app navigation
- **Conversation persistence** — local SQLite storage
- **Secure auth** — OAuth device flow + OS keychain token storage
- **Auto-update** — seamless updates via `tauri-plugin-updater` from GitHub Releases
- **Message editing** — edit a sent message (discards everything after it, re-sends); regenerate last assistant response
- **Favourites** — pin important conversations to the top of the sidebar
- **In-conversation search** — Cmd+F / Ctrl+F to find text within the current conversation
- **System tray / menu bar** — minimize to tray instead of closing (Tauri core `tray-icon` feature); streaming continues when window is hidden; right-click menu (New Chat, Show, Quit)
- **Thinking/reasoning display** — show model thinking tokens in a collapsible section, collapsed by default
- **Context window management** — automatic summarization of older messages to stay within model limits; visual indicator when summarization has occurred
- **Conversation title generation** — auto-generate titles via lightweight API call after first exchange; user can edit
- **Crash recovery** — auto-save input drafts to SQLite; preserve partial responses on interruption; restore on next launch
- **Offline mode** — full read access when offline, sending disabled with clear indicator, auto-reconnect
- **Conversation export** — export conversations as JSON (structured backup) or Markdown (human-readable) via `tauri-plugin-dialog` save dialog
- **Database management** — show DB size in settings, offer cleanup of old conversations, warn at 500MB
- **Accessibility** — semantic HTML, ARIA attributes, keyboard navigation, screen reader support, focus management
- **Window state persistence** — remember window position, size, and maximized state across restarts via `tauri-plugin-store`; validate against connected monitors on restore

### ⛔ Hard Requirement: No Filesystem / Machine Access

**This app must NEVER access the user's machine beyond what the user explicitly provides.**

- The app has **zero access** to the filesystem — it cannot read, write, browse, or scan any files or directories on its own
- The **only** way files enter the app is through explicit user action: drag-and-drop (HTML5 drag events, file read via `FileReader` API in the webview) or native file picker (`tauri-plugin-dialog`)
- File contents are read **once** into memory at the moment the user attaches them — the app does not retain file paths or re-read from disk
- The app stores **only** its own data: conversations (SQLite in app data dir), auth tokens (OS keychain), and user preferences (app config dir)
- No shell execution, no subprocess spawning, no system command access — **with one exception:** MCP stdio transport may spawn user-approved MCP server binaries (see MCP Security below)
- No screen capture, no clipboard snooping, no background scanning
- No network requests except to: GitHub Copilot API, GitHub OAuth endpoints, **user-configured MCP servers**, **web search API**, **user-provided URLs**, **GitHub Releases API** (for auto-update), **aitmpl.com API** (skill/agent registry), and **GitHub raw content APIs** (for git URL skill/agent imports)
- All outbound network destinations beyond GitHub must be **explicitly configured or initiated by the user**
- **URL fetching safeguards:** the app must block requests to private IP ranges (10.x, 172.16-31.x, 192.168.x), localhost, link-local (169.254.x), and cloud metadata endpoints (169.254.169.254). Only fetch public HTTPS URLs.
- **Tauri capabilities** must be configured with minimal permissions — only the specific APIs each window/webview actually needs
- macOS builds should use **App Sandbox** entitlements to enforce this at the OS level
- This is a **non-negotiable security boundary** — any feature that requires filesystem or machine access is out of scope

### Out of Scope

- Computer Use / screen control / autonomous desktop agents
- Cowork (background task execution on the user's machine)
- Code editing / IDE features / inline code suggestions
- File creation/modification on disk
- Filesystem browsing or scanning
- Shell/command execution
- Voice input (possible future phase)
- Conversation sharing as a cloud-hosted shareable link (possible future phase)
- Data portability / DB import-export (possible future phase)
- Drag-and-drop reordering of sidebar items (possible future phase)
- Localization / i18n — English only for v1 (possible future phase; string centralization makes this easier later)

---

## UX Wireframes

> 📄 **See [docs/WIREFRAMES.md](docs/WIREFRAMES.md)** for the complete set of 9 ASCII wireframes
> covering every major view and interaction state.
>
> **Wireframes are the canonical reference** for layout, component placement, and interaction
> design. When implementing a view, match the wireframe structure exactly. If a wireframe conflicts
> with prose elsewhere, the wireframe wins.
>
> Wireframes included: Main Layout, Input Area, Auth Screen, Settings Panel,
> Agents Management, Skills Panel, MCP Settings, Offline/Error States, Thinking/Reasoning Display.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Tauri v2 Application                               │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Frontend (Svelte 5 + TypeScript — system webview)              │ │
│  │                                                                 │ │
│  │  ┌──────────────┐  ┌────────────────────────────────────────┐  │ │
│  │  │   Sidebar    │  │          Main Panel                    │  │ │
│  │  │              │  │                                        │  │ │
│  │  │ [New Chat]   │  │  ┌────────────────────────────────┐   │  │ │
│  │  │              │  │  │   Message List (scrollable)     │   │  │ │
│  │  │ Agents ▾     │  │  │                                │   │  │ │
│  │  │  • Research  │  │  │  [User]  How do I parse JSON?  │   │  │ │
│  │  │  • Coder     │  │  │                                │   │  │ │
│  │  │              │  │  │  [Copilot] You can use serde.. │   │  │ │
│  │  │ Projects ▾   │  │  │  ```rust                       │   │  │ │
│  │  │  └─ Convos   │  │  │  use serde::Deserialize; 📋   │   │  │ │
│  │  │              │  │  │  ```                           │   │  │ │
│  │  │ Recent       │  │  │                                │   │  │ │
│  │  │  • Chat 1    │  │  │  🌐 [Web result: serde docs]  │   │  │ │
│  │  │  • Chat 2    │  │  │                                │   │  │ │
│  │  │              │  │  └────────────────────────────────┘   │  │ │
│  │  │ Search 🔍    │  │                                        │  │ │
│  │  │              │  │  ┌────────────────────────────────┐   │  │ │
│  │  │ Skills ⚡    │  │  │ [📎 Attach] [🌐 Web] Msg...  │   │  │ │
│  │  │ [⚙ Settings] │  │  │ [Agent: Research ▾]   [Send ➤]│   │  │ │
│  │  └──────────────┘  └────────────────────────────────────────┘  │ │
│  │                                                                 │ │
│  │  Svelte Stores:                                                 │ │
│  │  conversations │ auth │ agents │ skills │ projects │ mcp │       │ │
│  │  models │ settings │ network                                    │ │
│  └─────────────────────────┬───────────────────────────────────────┘ │
│                            │ Tauri IPC (invoke / listen)             │
│  ┌─────────────────────────┴───────────────────────────────────────┐ │
│  │  Rust Backend (src-tauri)                                       │ │
│  │                                                                 │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │ │
│  │  │ Tauri        │  │ App State    │  │ SQLite Database      │  │ │
│  │  │ Commands     │  │ (managed)    │  │ (conversations,      │  │ │
│  │  │ (IPC bridge) │  │              │  │  agents, skills,     │  │ │
│  │  └──────┬───────┘  └──────────────┘  │  projects, config)   │  │ │
│  │         │                            └──────────────────────┘  │ │
│  │  ┌──────┴─────────────────────────────────────────────────────┐│ │
│  │  │              Rust Library Crates                            ││ │
│  │  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  ││ │
│  │  │  │ copilot-api │  │ mcp-client   │  │ web-research     │  ││ │
│  │  │  │ OAuth + SSE │  │ Tool calls   │  │ Search API +     │  ││ │
│  │  │  │ Chat API    │  │ HTTP + Stdio │  │ URL fetcher      │  ││ │
│  │  │  └──────┬──────┘  └──────┬───────┘  └──────┬───────────┘  ││ │
│  │  └─────────┼────────────────┼──────────────────┼──────────────┘│ │
│  └────────────┼────────────────┼──────────────────┼───────────────┘ │
│  Tauri Plugins: updater │ global-shortcut │ dialog │ notification   │
│                clipboard-manager │ shell │ store                    │
│  Tauri Core Features: tray-icon │ devtools                         │
└───────────────┼────────────────┼──────────────────┼──────────────────┘
                │                │                  │
         ┌──────┴──────┐  ┌─────┴──────────┐  ┌───┴────────────────┐
         │ GitHub API  │  │ MCP Servers    │  │ Web (search API +  │
         │ /v1/chat/   │  │ (user-config)  │  │  user-provided     │
         │ completions │  │                │  │  URLs)             │
         └─────────────┘  └────────────────┘  └────────────────────┘
```

### IPC Design (Tauri Commands)

The frontend communicates with the Rust backend exclusively through **Tauri commands** (`invoke()`)
and **events** (`listen()`/`emit()`). This is the only bridge between the two layers.

**Commands** (frontend → backend, request/response):

| Module | Commands | Status |
|---|---|---|
| `mod.rs` | `get_app_info` — return app name + version; `log_frontend` — surface frontend log messages to Rust console | ✅ |
| `chat.rs` | `send_message` — send chat message, starts streaming via events; `stop_streaming` — cancel in-flight SSE stream; `generate_title` — auto-generate conversation title from first exchange | ✅ |
| `auth.rs` | `authenticate` — initiate OAuth device flow; `poll_auth_token` — poll for token after user authorizes; `logout` — clear token from keychain; `get_auth_state` — check current auth status | ✅ |
| `conversations.rs` | `get_conversations` — list from SQLite; `get_conversation` — single by ID; `create_conversation` — new conversation; `update_conversation` — rename/update metadata; `delete_conversation` — remove conversation + messages; `get_messages` — messages for a conversation; `create_message` — insert message; `update_message_content` — update after streaming/edit; `delete_messages_after` — discard messages after sort order (for editing) | ✅ |
| `models.rs` | `get_models` — fetch available Copilot models (deduplicates API response) | ✅ |
| `settings.rs` | `get_setting` — read config key; `update_setting` — write config key-value; `get_db_size` — return database file size; `save_export_file` — export conversations via server-side save dialog (Rust controls path selection); `delete_old_conversations` — remove conversations older than cutoff date; `export_conversation_json` — export single conversation as JSON; `export_conversation_markdown` — export single conversation as Markdown; `export_all_conversations_json` — bulk export all as JSON; `export_all_conversations_markdown` — bulk export all as Markdown; `save_draft` — persist input draft; `get_draft` — retrieve draft for conversation; `delete_draft` — clear draft | ✅ |
| `agents.rs` | `get_agents` — list agent personas; `get_agent` — single by ID; `create_agent` — new agent; `update_agent` — edit agent; `delete_agent` — remove agent (blocks default); `set_agent_skills` — assign skills; `set_agent_mcp_connections` — assign MCP servers; `install_agent_from_registry` — install from aitmpl.com; `import_agent_from_git` — import from git (internal, used by sources); `fetch_git_agents` — discover agent files from git repo (internal, used by sources) | ✅ |
| `skills.rs` | `get_skills` — list all skills; `create_skill` — add new skill; `update_skill` — edit skill; `delete_skill` — remove skill; `toggle_skill` — enable/disable; `search_registry` — search aitmpl.com registry; `install_from_registry` — fetch SKILL.md + save; `fetch_git_skills` — discover SKILL.md files from git URL (internal, used by sources); `import_git_skill` — save parsed skill from git (internal, used by sources) | ✅ |
| `projects.rs` | `get_projects` — list projects; `get_project` — single by ID; `create_project` — new project; `update_project` — edit instructions/name; `delete_project` — remove project; `get_project_files` — list files; `add_project_file` — attach file (BLOB); `get_project_file_content` — read file content; `remove_project_file` — detach file; `get_project_conversations` — list conversations in project; `pick_file_for_upload` — native file picker for project files; `pick_file_for_chat` — native file picker for chat attachments; `extract_file_text` — async text extraction (PDF, DOCX, XLSX, PPTX, RTF, 60+ text formats); `read_dropped_files` — read file paths from Tauri drag-drop events (validated against OS-registered allowed paths) | ✅ |
| `mcp.rs` | `get_mcp_servers` — list configured servers; `add_mcp_server` — register new server; `update_mcp_server` — update server config; `remove_mcp_server` — delete server; `connect_mcp_server` — connect to server (auth_header redacted in response; stdio binaries require prior approval); `disconnect_mcp_server` — disconnect; `test_mcp_connection` — verify server responds; `test_mcp_connection_config` — test unsaved server config from add/edit form; `get_mcp_tools` — list discovered tools; `invoke_mcp_tool` — call an MCP tool; `fetch_mcp_registry` — browse official MCP Registry; `approve_mcp_binary` — approve a stdio binary for execution (persisted to SQLite); `is_mcp_binary_approved` — check if a binary is approved | ✅ |
| `web_research.rs` | `web_search` — trigger web search via API; `fetch_url` — fetch + extract URL content | ✅ |
| `sources.rs` | `get_git_sources` — list all git sources; `get_git_source` — single by ID; `create_git_source` — add + scan repo; `update_git_source` — rename/toggle; `delete_git_source` — remove source (items kept as local copies); `sync_git_source` — re-scan repo + update imported items; `import_source_items` — import selected skills/agents from scan; `sync_all_sources` — auto-sync all enabled sources (called on app launch); `get_source_items` — list skills/agents linked to source | ✅ |

**Events** (backend → frontend, push):
- `streaming-token` — individual SSE tokens during chat
- `streaming-complete` — response finished
- `streaming-error` — error during streaming
- `auth-state-changed` — login/logout
- `git-import-progress` — progress updates during git source scan/sync (total, fetched, phase)
- `context-summarized` — older messages were condensed into a summary to manage context window
- `tray-new-chat` — user clicked "New Chat" in system tray menu
- `update-available` — new version found (via `tauri-plugin-updater`, not custom emit)

---

## Project Structure

```
copilot-desktop/
├── Cargo.toml                    # Rust workspace manifest
├── Cargo.lock                    # Rust dependency lock file (committed)
├── package.json                  # Frontend dependencies (pnpm)
├── pnpm-lock.yaml                # pnpm lock file (committed)
├── pnpm-workspace.yaml           # pnpm workspace configuration
├── index.html                    # HTML shell — Vite entry point (CSP meta tags here)
├── svelte.config.js              # Svelte 5 configuration
├── vite.config.ts                # Vite bundler configuration
├── vitest.config.ts              # Vitest test runner configuration
├── tsconfig.json                 # TypeScript configuration
├── eslint.config.js              # ESLint flat config
├── .prettierrc                   # Prettier formatting config
├── .prettierignore               # Prettier ignore patterns
├── .githooks/                    # Git hooks (post-merge)
│   └── post-merge                # Auto-run pnpm install after merges
├── src/                          # ── Svelte Frontend ──
│   ├── main.ts                   # Svelte app bootstrap (mount to #app)
│   ├── vite-env.d.ts             # Vite client type declarations
│   ├── app.css                   # Global styles, CSS custom properties (theme)
│   ├── App.svelte                # Root component (sidebar + main panel layout)
│   ├── lib/
│   │   ├── components/           # Svelte 5 components
│   │   │   ├── Sidebar.svelte           # Conversation list, search, agents, projects, favourites
│   │   │   ├── ChatView.svelte          # Chat view (message list + input + search)
│   │   │   ├── MessageBubble.svelte     # Single message (markdown + code blocks + web results + thinking)
│   │   │   ├── InputArea.svelte         # Multi-line input, file drop, attachment pills, agent/model selector
│   │   │   ├── CodeBlock.svelte         # Syntax-highlighted code with copy button + language label
│   │   │   ├── ThinkingSection.svelte   # Collapsible reasoning/thinking display
│   │   │   ├── WebResultCard.svelte     # Cited web search result card
│   │   │   ├── AuthScreen.svelte        # OAuth login/welcome screen
│   │   │   ├── SettingsPanel.svelte     # Settings (account, theme, model, MCP, export, DB, shortcuts)
│   │   │   ├── ProjectView.svelte       # Project detail (instructions, files, conversations)
│   │   │   ├── AgentsPanel.svelte       # Agent management (create/edit/delete + registry browse)
│   │   │   ├── SkillsPanel.svelte       # Skills browser (local + registry, toggle on/off)
│   │   │   ├── McpSettings.svelte       # MCP server management (add, configure, test, browse registry)
│   │   │   ├── McpServerForm.svelte    # MCP server add/edit form with registry pre-fill
│   │   │   ├── ConfirmDialog.svelte    # Reusable confirmation dialog modal
│   │   │   ├── UpdateBanner.svelte      # Auto-update notification + download progress
│   │   │   ├── SearchOverlay.svelte     # In-conversation Cmd+F search overlay
│   │   │   └── SourcesPanel.svelte      # Git sources management (add, sync, browse, import items)
│   │   ├── stores/               # Svelte 5 runes-based stores (reactive state)
│   │   │   ├── conversations.svelte.ts  # Conversation + message state
│   │   │   ├── auth.svelte.ts           # Auth state (token, user info)
│   │   │   ├── models.svelte.ts         # Available models state
│   │   │   ├── mcp.svelte.ts            # MCP server connections state
│   │   │   ├── agents.svelte.ts         # Agent personas state
│   │   │   ├── skills.svelte.ts         # Skills/extensions state
│   │   │   ├── projects.svelte.ts       # Projects state
│   │   │   ├── settings.svelte.ts       # User preferences + theme management (applies data-theme)
│   │   │   ├── network.svelte.ts        # Online/offline state
│   │   │   └── sources.svelte.ts        # Git sources state (CRUD, sync, import)
│   │   ├── types/                # TypeScript type definitions (mirrors Rust types)
│   │   │   ├── auth.ts
│   │   │   ├── conversation.ts
│   │   │   ├── message.ts
│   │   │   ├── mcp.ts
│   │   │   ├── web-research.ts
│   │   │   ├── agent.ts
│   │   │   ├── skill.ts
│   │   │   ├── registry.ts
│   │   │   ├── project.ts
│   │   │   └── source.ts
│   │   ├── strings/               # Centralized user-facing strings (i18n prep)
│   │   │   └── en.ts              # English strings (default)
│   │   └── utils/
│   │       ├── markdown.ts              # Markdown rendering (marked + DOMPurify)
│   │       ├── syntax.ts               # Syntax highlighting (Shiki)
│   │       ├── commands.ts              # Typed wrappers around tauri invoke()
│   │       ├── events.ts               # Typed wrappers around tauri listen()
│   │       └── format.ts              # Date formatting, text truncation, etc.
│   └── tests/                    # Frontend tests (Vitest)
│       ├── setup.ts
│       └── *.test.ts
├── src-tauri/                    # ── Tauri v2 Rust Backend ──
│   ├── Cargo.toml                # Depends on workspace crates + Tauri plugins
│   ├── tauri.conf.json           # Tauri configuration (window, security, plugins, updater)
│   ├── capabilities/             # Tauri v2 capability definitions (minimal permissions)
│   │   └── default.json
│   ├── icons/                    # App icons (macOS .icns, Linux .png, Windows .ico)
│   ├── build.rs                  # Tauri build script
│   └── src/
│       ├── main.rs               # Entry point (Tauri bootstrap)
│       ├── lib.rs                # Tauri app setup, plugin registration, state init
│       ├── state.rs              # Tauri managed state (AppState, DB pool, allowed file paths, etc.)
│       ├── skillmd.rs            # SKILL.md parser (YAML frontmatter + markdown body)
│       ├── registry.rs           # Skill/agent registry client (aitmpl.com API, git import)
│       ├── text_extract.rs       # Text extraction from files (PDF, DOCX, XLSX, PPTX, RTF, 60+ text formats)
│       ├── commands/             # Tauri command handlers (IPC bridge to frontend)
│       │   ├── mod.rs
│       │   ├── chat.rs           # send_message, stop_streaming, regenerate
│       │   ├── auth.rs           # authenticate, logout, get_auth_state
│       │   ├── conversations.rs  # CRUD conversations + messages
│       │   ├── agents.rs         # CRUD agent personas + registry import
│       │   ├── skills.rs         # List/toggle/configure skills + registry search
│       │   ├── projects.rs       # CRUD projects + file attachments + drag-drop + text extraction
│       │   ├── mcp.rs            # MCP server management + tool invocation
│       │   ├── web_research.rs   # Web search + URL fetching
│       │   ├── sources.rs        # Git sources management (CRUD, sync, import)
│       │   ├── models.rs         # Model discovery + selection
│       │   └── settings.rs       # User preferences + export + DB management
│       └── db/                   # Database layer
│           ├── mod.rs            # DB initialization, connection pool
│           ├── migrations.rs     # Schema versioning + migration scripts
│           └── queries.rs        # Typed query functions
├── crates/                       # ── Standalone Rust Library Crates ──
│   ├── copilot-api/              # GitHub Copilot API client (zero Tauri dependency)
│   │   ├── src/
│   │   │   ├── lib.rs            # Public API
│   │   │   ├── auth.rs           # OAuth device flow + token refresh
│   │   │   ├── client.rs         # HTTP client + SSE streaming
│   │   │   ├── types.rs          # Request/response types (messages, roles, attachments)
│   │   │   └── keychain.rs       # Secure token/API key storage (cross-platform via keyring)
│   │   └── Cargo.toml
│   ├── mcp-client/               # MCP protocol client (zero Tauri dependency)
│   │   ├── src/
│   │   │   ├── lib.rs            # Public API
│   │   │   ├── client.rs         # MCP server connection + tool invocation (SSRF protection)
│   │   │   ├── manager.rs        # Connection lifecycle manager (RwLock-based pool)
│   │   │   ├── registry.rs       # Official MCP Registry client (search, pagination, package info)
│   │   │   └── types.rs          # MCP protocol types (tools, resources, prompts)
│   │   └── Cargo.toml
│   ├── web-research/             # Web search + URL content extraction (zero Tauri dependency)
│   │   ├── src/
│   │   │   ├── lib.rs            # Public API
│   │   │   ├── search.rs         # Web search API client (Bing/Google/etc.)
│   │   │   ├── fetcher.rs        # URL fetcher + HTML-to-text extraction
│   │   │   └── types.rs          # Search results, extracted content types
│   │   └── Cargo.toml
│   └── xtask/                    # Version management + release automation CLI (cargo xtask)
│       ├── src/
│       │   ├── main.rs           # CLI entry: bump, check-version, changelog, release subcommands
│       │   ├── version.rs        # Shared: project_root(), read/write version across files
│       │   ├── bump.rs           # Bump version in Cargo.toml + package.json + tauri.conf.json
│       │   ├── check.rs          # Verify all version files are in sync
│       │   ├── changelog.rs      # Generate CHANGELOG.md from conventional commits
│       │   └── release.rs        # Automated release: auto-detect bump + changelog + commit + tag
│       └── Cargo.toml
├── .cargo/
│   └── config.toml               # Cargo aliases (xtask)
├── CHANGELOG.md                   # Auto-generated from conventional commits (cargo xtask changelog)
├── AGENTS.md                      # Core project documentation (conventions, architecture, security)
├── STYLE-GUIDE.md                 # Warm Ink design system reference (tokens, components, per-panel guides)
├── LICENSE                        # Project license
├── docs/                          # Extended documentation (extracted from AGENTS.md for context efficiency)
│   ├── WIREFRAMES.md              # 9 UX wireframes — canonical layout reference
│   ├── API-REFERENCE.md           # Dependencies, Copilot API, MCP, skills/agents
│   ├── DATA-MODEL.md              # SQLite schema, persistence, migrations, versioning
│   └── PLAN.md                    # Implementation plan, build commands, risks, design reference
└── README.md
```

---

## Coding Conventions

### Rust Style

- Use **Rust 2021 edition**
- Follow standard `rustfmt` formatting (default config)
- Use `clippy` with default lints — treat warnings as errors in CI
- Prefer `thiserror` for library error types, `anyhow` for application-level errors
- Use `log` + `env_logger` for logging (not `println!` for debug output)
- All public API items must have doc comments (`///`)
- Tauri commands must return `Result<T, String>` or use a custom serializable error type

### Svelte + TypeScript Style

- Use **Svelte 5** with runes (`$state`, `$derived`, `$effect`, `$props`) — no legacy `$:` reactive declarations
- Use **TypeScript strict mode** (`"strict": true` in tsconfig)
- Follow [Svelte conventions](https://svelte.dev/docs): one component per file, PascalCase filenames
- Use `.svelte.ts` extension for files that use Svelte runes outside components (stores)
- **CSS scoping:** use Svelte's built-in `<style>` scoping. Global styles only in `app.css`
- **Unified Design System:** all panel components (Agents, Skills, MCP, Settings, Projects) must use the shared component classes defined in `app.css`. See **[STYLE-GUIDE.md](STYLE-GUIDE.md)** for the full design system reference — tokens, components, do's/don'ts, and per-panel guides.
- **Theme:** use CSS custom properties (`--color-bg`, `--color-text`, etc.) defined in `app.css` and toggled via a `data-theme` attribute on `<html>`
- **No inline styles** — use CSS classes. Exception: dynamic values that must be computed (use `style:` directive)
- **No `any` type** — every value must be properly typed. Use `unknown` + type guards when dealing with external data
- **i18n preparation:** English only for v1, but centralize all user-facing strings in dedicated constant files (e.g., `src/lib/strings/`) rather than scattering hardcoded strings across components. This makes future localization extraction easier.
- **Accessibility:** use semantic HTML (`<nav>`, `<main>`, `<article>`, `<button>`), ARIA attributes where needed, visible focus indicators, and keyboard-navigable interactive elements
- Prefer named exports over default exports
- Use `$inspect()` for development debugging, never in production

### Dependencies Policy

**Only stable, actively maintained packages at their latest version. No exceptions.**

**Rust crates:**
- **Always use the latest stable release** of every dependency. Check [crates.io](https://crates.io) or [lib.rs](https://lib.rs) for the current version — do not guess or use old versions from memory.
- **No unmaintained crates.** Verify each crate has been updated within the last 12 months and has no advisory on [RustSec](https://rustsec.org/).
- **No deprecated crates.** Use the successor if one exists.
- **Run `cargo audit`** as part of every review cycle.
- **Run `cargo update`** regularly. Lock file (`Cargo.lock`) must be committed.

**npm packages:**
- **Always use the latest stable release.** Check [npmjs.com](https://www.npmjs.com/) for current versions.
- **No deprecated packages.** Check for deprecation notices on npm.
- **Run `pnpm audit`** as part of every review cycle.
- **Run `pnpm update`** regularly. Lock file (`pnpm-lock.yaml`) must be committed.
- **Prefer packages with TypeScript types** built-in or via `@types/*`.
- **Minimize frontend dependencies.** The Svelte + Vite stack is intentionally lightweight; avoid adding large framework-level libraries.

### Git Conventions

#### ⛔ Always Branch First

**Agents must ALWAYS create a new branch before starting work.** No commits on `main` — ever.

- Before making any changes, create a descriptive branch: `git checkout -b <type>/<short-description>` (e.g., `feat/mcp-settings`, `fix/streaming-crash`, `docs/update-agents-md`)
- Branch naming follows the same `<type>` prefixes as Conventional Commits (see below)
- If the agent is resuming work on an existing task branch, it may reuse that branch instead of creating a new one
- **Never commit directly to `main`** — this applies to all agents, all tasks, all circumstances

#### ⛔ Never Push

**Agents must NEVER push to any remote.** Commit locally only. The human reviews and pushes.

- `git push` is **forbidden** — no exceptions, no force push, no push to any branch
- Agents commit to the local branch only
- The human owner reviews all commits and decides when/where to push
- This applies to all agents, all tasks, all circumstances

#### Conventional Commits

All commits **must** follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <short summary>

<optional body>

<optional footer(s)>
```

**Types:**

| Type | When to use |
|---|---|
| `feat` | A new feature or capability |
| `fix` | A bug fix |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `docs` | Documentation only changes |
| `style` | Formatting, missing semicolons, etc. (no code change) |
| `test` | Adding or updating tests |
| `chore` | Build process, dependencies, tooling, CI changes |
| `perf` | Performance improvements |
| `ci` | CI/CD configuration changes |
| `build` | Build system or external dependency changes |

**Scope** is the crate/area affected: `app`, `ui`, `copilot-api`, `mcp-client`, `web-research`, `tauri`, `deps`, `ci`

**Examples:**

```
feat(copilot-api): add SSE streaming for chat completions
fix(ui): prevent layout shift during rapid token updates
feat(tauri): add system tray with context menu
docs: update AGENTS.md with Tauri architecture
refactor(mcp-client): extract transport layer into separate module
chore(deps): update @tauri-apps/api to 2.2.0
test(web-research): add URL validation tests for private IP blocking
```

**Rules:**
- Subject line: imperative mood, lowercase, no period, max 72 characters
- Body: wrap at 72 characters, explain *what* and *why* (not *how*)
- Breaking changes: add `BREAKING CHANGE:` footer or `!` after type/scope
- Reference issues/tasks when applicable
- **Type must match the actual change** — use `docs:` for documentation-only changes, even if the docs describe code features. Reserve code-level types for commits that modify `.rs`, `.ts`, `.svelte`, `.toml`, `.json`, or other source/config files.

### Tauri Patterns

- **Commands** are the IPC bridge: define in Rust with `#[tauri::command]`, call from Svelte with `invoke()`
- **Events** for backend-to-frontend push: use `app.emit()` in Rust, `listen()` in Svelte
- **Managed state** via `app.manage()` — shared across all commands (DB pool, auth state, MCP connections)
- **Keep commands thin** — they should validate input, delegate to library crates, and return results. Business logic lives in `copilot-api`, `mcp-client`, `web-research` crates.
- **Serialize everything** — all Tauri command inputs/outputs must be `Serialize + Deserialize`. Use shared type definitions and keep Rust types in sync with TypeScript types.
- **Capabilities** — configure `src-tauri/capabilities/` with minimal permissions. Each capability should grant only what's needed.
- **Error handling** — Tauri commands should return `Result<T, String>` or a custom error type that serializes to a user-friendly message. Never expose stack traces to the frontend.

### Crate Boundaries

- **`copilot-api`** is a standalone Rust library with **zero Tauri dependency**. It should be usable from any Rust project (CLI, different GUI framework, etc.)
- **`mcp-client`** is a standalone Rust library with **zero Tauri dependency**. Handles MCP protocol, server connections, tool invocation, connection lifecycle management, and the official MCP Registry client.
- **`web-research`** is a standalone Rust library with **zero Tauri dependency**. Handles web search API calls and URL content fetching/extraction.
- **`src-tauri`** depends on all library crates — it is the only crate that imports Tauri and defines commands.
- **Frontend** depends only on `@tauri-apps/*` packages for IPC — all heavy logic runs in Rust.
- No circular dependencies between crates.
- **TypeScript types must mirror Rust types.** When a Rust struct changes, the corresponding TS type must be updated.

### Error Handling

- API client errors should be typed and descriptive (auth failures, rate limits, network errors)
- Tauri commands return `Result<T, E>` — errors are serialized and sent to the frontend
- Frontend displays user-friendly error messages via toast notifications or inline banners — never show raw stack traces
- Network failures should suggest retry; auth failures should redirect to login
- Use Svelte's `{#if error}` blocks or error boundaries for graceful UI degradation

### Security

- **Never log or display OAuth tokens or API keys** in any output (Rust logs or browser console)
- Tokens and API keys must be stored only in the OS keychain — never in plain text files, SQLite, or localStorage
- **MCP auth headers** are stored in the OS keychain (key pattern: `mcp_auth_{server_id}`), never in SQLite. Migration v3 proactively migrates any pre-existing plaintext auth headers to keychain.
- **Token types must not derive `Serialize`** — `OAuthTokenResponse` and `CopilotTokenResponse` are `Deserialize`-only to prevent accidental serialization back to the frontend
- **Sensitive fields must not appear in Debug output** — types containing secrets (e.g., `McpServerConfig`) use custom `Debug` impls that redact sensitive fields
- Validate all API responses — don't trust server data shapes blindly
- **No filesystem access** — the app cannot read, write, or browse files on its own. Files only enter via explicit user drag-and-drop or Tauri dialog file picker. File contents are read into memory once; the app never stores or re-accesses file paths.
- **Drag-and-drop path validation** — dropped file paths are registered in `AppState.allowed_file_paths` from the OS drag event, then validated when `read_dropped_files` is called. Paths are consumed (one-time use) to prevent replay.
- **No shell/subprocess execution** — the app must never spawn processes or run commands, **except** for MCP stdio transport (see MCP Security below)
- **No network requests** except to: GitHub Copilot API, GitHub OAuth, user-configured MCP servers, web search API, user-provided URLs, and GitHub Releases API (for auto-update). All non-GitHub network destinations must be explicitly user-configured or user-initiated.
- **URL fetching:** block private IPs (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16), localhost (127.0.0.0/8), link-local (169.254.0.0/16), and cloud metadata (169.254.169.254). Only fetch public HTTPS URLs.
- **Tauri capabilities** must be configured with the minimal set of permissions needed. Audit `capabilities/default.json` in every review cycle.
- **Content Security Policy (CSP)** must be configured in `tauri.conf.json` to prevent XSS. `style-src 'unsafe-inline'` is required for Svelte scoped styles and Shiki runtime injection — this is an accepted trade-off. No `unsafe-eval`.
- **IPC boundary hardening** — all Tauri commands that return data to the frontend must redact sensitive fields (auth headers, tokens). File save operations must use server-side dialogs (Rust controls path selection via `tauri-plugin-dialog`).
- **Markdown sanitization** — all rendered markdown must be sanitized with DOMPurify before insertion into the DOM. Never use `{@html}` with unsanitized content.
- **MCP response sanitization** — all MCP tool responses must be sanitized before rendering. Strip scripts from text content, enforce max payload size (e.g., 1MB), validate JSON structure.
- **MCP server connections** are user-managed — the app never auto-discovers or connects to MCP servers without explicit user configuration
- **macOS App Sandbox required** — enforce filesystem and network restrictions at the OS level via entitlements
- Treat any code path that touches the filesystem (outside app data dir) or spawns a non-MCP process as a **security violation**
- **Conversation export exception:** exporting conversations (JSON/Markdown) writes to a user-chosen location via the Tauri server-side save dialog. The Rust backend controls the dialog and writes the file — the frontend never receives or handles the file path.
- This is the **only** permitted filesystem write outside the app data directory.

### MCP Security

MCP supports two transports: **HTTP** and **stdio**. Stdio transport spawns a local process to run
an MCP server binary. This is the **only** exception to the no-subprocess rule:

- **Binary approval enforcement** — stdio MCP servers require explicit user approval before first launch. Approved binary paths are persisted in the `approved_mcp_binaries` SQLite table. The Rust backend checks `is_binary_approved()` before connecting; if not approved, returns a `BINARY_NOT_APPROVED:{path}` sentinel error. The frontend shows a confirmation dialog and calls `approve_mcp_binary` on user consent (single retry to prevent infinite loops).
- The binary path must be user-provided — the app never searches the filesystem for binaries
- Each stdio server launch must be logged and visible in the MCP settings UI
- **Auth header security** — MCP server auth headers are stored in the OS keychain (key pattern: `mcp_auth_{server_id}`), never in SQLite. The `connect_mcp_server` command redacts `auth_header` to `"[REDACTED]"` before returning data to the frontend via `redact_connection_info()`. `McpServerConfig` uses a custom `Debug` implementation that masks auth headers as `"••••••••"`.
- HTTP transport is preferred and should be the default recommendation in the registry
- Tauri's `shell` plugin scope must be configured to allow **only** user-configured MCP binaries — no wildcard execution
- If App Sandbox restricts subprocess spawning, document this limitation and fall back to HTTP-only

---

## Key Dependencies, API Integration, MCP, Skills & Agents

> 📄 **See [docs/API-REFERENCE.md](docs/API-REFERENCE.md)** for the complete reference covering:
> - **Key Dependencies** — Rust crates (25+) and npm packages (20+) with version policy
> - **GitHub Copilot API** — OAuth device flow, SSE streaming, model discovery, context window management, title generation, message editing, crash recovery, offline mode, export
> - **Auto-Update** — `tauri-plugin-updater` via GitHub Releases with signature verification
> - **Web Research** — Bing Web Search API + URL fetching with SSRF protection
> - **MCP Integration** — Protocol spec 2025-03-26, HTTP + stdio transports, official MCP Registry, custom servers, security model
> - **Skills & Agents** — SKILL.md standard, aitmpl.com + git registries, agent→skill→MCP mapping, API call construction

---

## Data Model (SQLite)

> 📄 **See [docs/DATA-MODEL.md](docs/DATA-MODEL.md)** for the complete SQLite schema (12 tables, 8 indexes),
> persistence rules, migration strategy (v1→v2→v3), and the versioning/release system.
>
> **Key points:**
> - All persistent data in SQLite (app data dir via `app.path().app_data_dir()`)
> - Tokens and API keys in OS keychain only — never in SQLite or localStorage
> - Forward-only schema migrations with `schema_version` tracking
> - Lockstep versioning: `Cargo.toml` ↔ `package.json` ↔ `tauri.conf.json` via `cargo xtask`

---

## Implementation Plan, Build & Run, Risks, Design Reference

> 📄 **See [docs/PLAN.md](docs/PLAN.md)** for the full implementation plan (12 phases, all ✅ through Phase 11),
> build/run commands, risk mitigations, and the "Warm Ink" visual design system reference.
>
> **Quick build reference:**
> ```bash
> pnpm install                              # Install frontend deps
> cargo tauri dev                           # Development (hot-reload)
> cargo tauri build                         # Production build
> cargo build --workspace                   # Rust only
> cargo clippy --workspace -- -D warnings   # Rust lint
> cargo test --workspace                    # Rust tests
> pnpm check && pnpm lint && pnpm test      # Frontend checks
> ```
