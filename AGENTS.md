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
- **MCP integration** — connect to MCP servers for extended tool capabilities; built-in catalog of popular servers + custom server configuration
- **Skills management** — enable/disable/configure Copilot Extensions (tools/plugins) that extend what Copilot can do in conversations
- **Agents management** — create custom agent personas with specific system prompts, assigned skills, and MCP connections
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

### ⛔ Hard Requirement: No Filesystem / Machine Access

**This app must NEVER access the user's machine beyond what the user explicitly provides.**

- The app has **zero access** to the filesystem — it cannot read, write, browse, or scan any files or directories on its own
- The **only** way files enter the app is through explicit user action: drag-and-drop (HTML5 drag events, file read via `FileReader` API in the webview) or native file picker (`tauri-plugin-dialog`)
- File contents are read **once** into memory at the moment the user attaches them — the app does not retain file paths or re-read from disk
- The app stores **only** its own data: conversations (SQLite in app data dir), auth tokens (OS keychain), and user preferences (app config dir)
- No shell execution, no subprocess spawning, no system command access — **with one exception:** MCP stdio transport may spawn user-approved MCP server binaries (see MCP Security below)
- No screen capture, no clipboard snooping, no background scanning
- No network requests except to: GitHub Copilot API, GitHub OAuth endpoints, **user-configured MCP servers**, **web search API**, **user-provided URLs**, and **GitHub Releases API** (for auto-update)
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

> These wireframes are the **canonical reference** for layout, component placement, and interaction
> design. When implementing a view, match this structure exactly. If a wireframe conflicts with
> prose elsewhere in this document, the wireframe wins.

### 1. Main Layout (Sidebar + Chat)

The primary app surface. Sidebar is collapsible (Cmd+Shift+S). Chat fills remaining width.

```
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
```

**Key behaviors:**
- Sidebar width: ~260px, resizable, collapsible
- Favourites pinned at top, then Projects, then Agents, then date-grouped history
- Right-click sidebar conversation → context menu: Rename, Export (JSON/Markdown), Toggle Favourite ★, Delete
- ★ icon appears on hover of conversation items; click to toggle favourite
- Header shows conversation title (click to inline-edit); auto-generated title can be overridden
- Chat scrolls independently; auto-scroll on new tokens, pauses if user scrolled up
- Message actions appear on hover: ✏️ edit (user msgs), 📋 copy, ⟳ regenerate (last assistant msg)
- Code block [Copy] button always visible; message-level 📋 copy appears on hover
- Cmd+F activates `SearchOverlay.svelte`: floating search bar at top of chat with match count, ↑/↓ arrows, Escape to dismiss
- Context summarization banner is dismissible but not deletable
- Status bar shows: connection state, DB size, app version

### 2. Input Area (Detail)

Multi-line input with attachment support, agent/model selection, and action buttons.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  📎 config.json ✕ │ 📎 schema.sql ✕ │ 🌐 https://docs.rs/serde ✕     │  ← attachment pills
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Can you review these files and explain the schema?                      │  ← multi-line input
│  I'm especially interested in the migration strategy.                    │     (auto-grows)
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  📎 Attach  🌐 Web  │  Agent: Research ▾  │  Model: GPT-4o ▾  │ Send ➤ │  ← toolbar
└─────────────────────────────────────────────────────────────────────────┘

Drag-and-drop zone covers entire input area (visual highlight on drag-over)
```

**Key behaviors:**
- Input auto-grows up to ~6 lines, then scrolls internally
- Enter sends (default); Shift+Enter for newline. Configurable in Settings to use Cmd+Enter (Ctrl+Enter) to send instead (Enter always inserts newline in that mode)
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
│  ← Back                    Settings                              │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐       │
│  │ General  │ Account  │  MCP     │ Shortcuts│  Data    │       │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘       │
│                                                                  │
│  ── General ──────────────────────────────────────────────────   │
│                                                                  │
│  Theme                          [ System ▾ ]                     │
│                                  System / Light / Dark           │
│                                                                  │
│  Font Size                      [ 14px  ▾ ]                      │
│                                  12 / 13 / 14 / 15 / 16         │
│                                                                  │
│  Default Model                  [ GPT-4o ▾ ]                     │
│                                                                  │
│  Default Agent                  [ Default ▾ ]                    │
│                                                                  │
│  Send Message With              ( ● ) Enter                      │
│                                 (   ) Cmd+Enter (Ctrl+Enter)     │
│                                                                  │
│  ── Auto-Update ──────────────────────────────────────────────   │
│                                                                  │
│  Check for updates              [✓]  Enabled                     │
│  Check frequency                [ On startup ▾ ]                 │
│                                  On startup / Daily / Weekly     │
│                                                                  │
│  ── Global Hotkey ────────────────────────────────────────────   │
│                                                                  │
│  Summon Chuck                    [ Cmd+Shift+Space ]              │
│                                  (click to rebind)               │
│                                                                  │
│  ── Web Research ─────────────────────────────────────────────   │
│                                                                  │
│  Search API                     [ Bing ▾ ]                       │
│  API Key                        [ ••••••••••  👁 ]   [Change]    │
│                                  (stored in OS keychain)         │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

```
│  ── Account Tab ──────────────────────────────────────────────   │
│                                                                  │
│  Signed in as                   @octocat                         │
│  Copilot Plan                   Copilot Pro                      │
│                                                                  │
│  [ Sign Out ]                                                    │
│                                                                  │
│  ── Data Tab ─────────────────────────────────────────────────   │
│                                                                  │
│  Database Size                  12.3 MB                          │
│                                                                  │
│  Delete old conversations       [ Older than 90 days ▾ ]         │
│                                 [ Delete Now ]                   │
│                                                                  │
│  Export All Conversations       [ JSON ] [ Markdown ]            │
│                                                                  │
│  ⚠️ Database is 487 MB — consider cleaning up old conversations  │
│                                 (shown when > 400MB)             │
└──────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**
- Tab navigation via keyboard (arrow keys) + click
- Changes apply immediately (no save button); persisted to SQLite `config` table
- Hotkey rebind: click field → "Press new shortcut..." → capture next key combo
- API key field masked by default; 👁 toggles visibility
- Sign Out clears keychain + redirects to auth screen
- Delete confirmation dialog before destructive actions
- Export opens native save dialog (`tauri-plugin-dialog`)

### 5. Agents Management

Accessed from sidebar "Agents" section or Settings.

```
┌──────────────────────────────────────────────────────────────────┐
│  ← Back                     Agents                               │
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
- Skills list populated from registered skills (extensions + MCP tools)
- MCP connections list populated from configured MCP servers
- Agent deletion requires confirmation; orphaned conversations keep agent name as text

### 6. Skills Panel

Browse and manage all available skills (Copilot Extensions + MCP tools).

```
┌──────────────────────────────────────────────────────────────────┐
│  ← Back                     Skills                               │
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
│  [ ]  🐛 search_issues                                            │
│       Search issues and pull requests                            │
│       Source: MCP · GitHub                                        │
│                                                                  │
│  [ ]  📝 get_file_contents                                        │
│       Read file contents from a repository                       │
│       Source: MCP · GitHub                                        │
│                                                                  │
│  ── MCP Tools (PostgreSQL Server) ────────────────────────────   │
│                                                                  │
│  [✓]  🗃️ query_database                                           │
│       Execute read-only SQL queries                              │
│       Source: MCP · PostgreSQL          [Configure ⚙️]            │
│                                                                  │
│  ── Copilot Extensions ───────────────────────────────────────   │
│                                                                  │
│  [ ]  🐳 @docker                                                  │
│       Docker container management and debugging                  │
│       Source: extension                                           │
│                                                                  │
│  [ ]  ☁️ @azure                                                    │
│       Azure resource management                                  │
│       Source: extension                                           │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Key behaviors:**
- Toggle checkbox to enable/disable a skill globally
- Skills grouped by source: Built-in → MCP (per server) → Extensions
- Configure button opens per-skill settings (e.g., connection string)
- Filter field does fuzzy search across name + description
- Disabled MCP skills (server disconnected) shown grayed out with status
- Skills assigned to agents are marked but can be toggled independently here

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
│  │    Tools: 12 discovered │ Source: catalog                  │  │
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
│  │    Tools: — │ Source: catalog                               │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ── Catalog ──────────────────────────────────────────────────   │
│                                                                  │
│  [ ] Filesystem (read-only)     Stdio  │ [Add]                   │
│  [ ] Brave Search               HTTP   │ [Add]                   │
│  [ ] Sentry                     HTTP   │ [Add]                   │
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
- Catalog entries pre-fill config; user just provides API keys
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
- During streaming: shows "Thinking..." with animated dots until first content token
- After completion: shows "Thinking (Xs)" with final elapsed time
- User preference in settings to default-expand (for power users)

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
│  │  settings │ theme │ network                                     │ │
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
│  Tauri Core Features: tray-icon │ protocol-asset                   │
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

| Module | Commands |
|---|---|
| `chat.rs` | `send_message` — send chat message, returns streaming event channel; `stop_streaming` — cancel in-flight SSE stream; `regenerate` — re-send last user message for fresh response |
| `auth.rs` | `authenticate` — initiate OAuth device flow; `logout` — clear token from keychain; `get_auth_state` — check current auth status |
| `conversations.rs` | `get_conversations` — list from SQLite; `create_conversation` — new conversation; `update_conversation` — rename/update metadata; `delete_conversation` — remove conversation + messages |
| `agents.rs` | `get_agents` — list agent personas; `create_agent` — new agent; `update_agent` — edit agent; `delete_agent` — remove agent |
| `skills.rs` | `get_skills` — list all skills (MCP tools + extensions); `toggle_skill` — enable/disable; `configure_skill` — update skill config |
| `projects.rs` | `get_projects` — list projects; `create_project` — new project; `update_project` — edit instructions/name; `delete_project` — remove project; `add_project_file` — attach file (BLOB); `remove_project_file` — detach file |
| `mcp.rs` | `get_mcp_servers` — list configured servers; `add_mcp_server` — register new server; `remove_mcp_server` — delete server; `test_mcp_connection` — verify server responds; `mcp_invoke_tool` — call an MCP tool |
| `web_research.rs` | `web_search` — trigger web search via API; `fetch_url` — fetch + extract URL content |
| `models.rs` | `get_models` — fetch available Copilot models |
| `settings.rs` | `get_settings` — read config table; `update_settings` — write config key-value; `export_conversations` — export as JSON/Markdown to user-chosen path; `delete_old_conversations` — cleanup by age; `get_db_size` — return database file size |

**Events** (backend → frontend, push):
- `streaming-token` — individual SSE tokens during chat
- `streaming-complete` — response finished
- `streaming-error` — error during streaming
- `auth-state-changed` — login/logout
- `network-status` — online/offline
- `update-available` — new version found
- etc.

---

## Project Structure

```
copilot-desktop/
├── Cargo.toml                    # Rust workspace manifest
├── package.json                  # Frontend dependencies (pnpm)
├── pnpm-lock.yaml
├── svelte.config.js              # Svelte 5 configuration
├── vite.config.ts                # Vite bundler configuration
├── tsconfig.json                 # TypeScript configuration
├── eslint.config.js              # ESLint flat config
├── .prettierrc                   # Prettier formatting config
├── src/                          # ── Svelte Frontend ──
│   ├── app.html                  # HTML shell (CSP meta tags here)
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
│   │   │   ├── AgentsPanel.svelte       # Agent management (create/edit/delete personas)
│   │   │   ├── SkillsPanel.svelte       # Skills browser (MCP tools + extensions, toggle on/off)
│   │   │   ├── McpSettings.svelte       # MCP server management (add, configure, test, browse tools)
│   │   │   ├── UpdateBanner.svelte      # Auto-update notification + download progress
│   │   │   └── SearchOverlay.svelte     # In-conversation Cmd+F search overlay
│   │   ├── stores/               # Svelte 5 runes-based stores (reactive state)
│   │   │   ├── conversations.svelte.ts  # Conversation + message state
│   │   │   ├── auth.svelte.ts           # Auth state (token, user info)
│   │   │   ├── agents.svelte.ts         # Agent personas state
│   │   │   ├── skills.svelte.ts         # Skills/extensions state
│   │   │   ├── projects.svelte.ts       # Projects state
│   │   │   ├── mcp.svelte.ts            # MCP server connections state
│   │   │   ├── settings.svelte.ts       # User preferences
│   │   │   ├── theme.svelte.ts          # Light/dark theme state
│   │   │   └── network.svelte.ts        # Online/offline state
│   │   ├── types/                # TypeScript type definitions (mirrors Rust types)
│   │   │   ├── conversation.ts
│   │   │   ├── message.ts
│   │   │   ├── agent.ts
│   │   │   ├── skill.ts
│   │   │   ├── project.ts
│   │   │   └── mcp.ts
│   │   ├── strings/               # Centralized user-facing strings (i18n prep)
│   │   │   └── en.ts              # English strings (default)
│   │   └── utils/
│   │       ├── markdown.ts              # Markdown rendering (marked + DOMPurify)
│   │       ├── syntax.ts               # Syntax highlighting (Shiki)
│   │       ├── commands.ts              # Typed wrappers around tauri invoke()
│   │       ├── events.ts               # Typed wrappers around tauri listen()
│   │       ├── shortcuts.ts            # Keyboard shortcut registration
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
│       ├── state.rs              # Tauri managed state (AppState, DB pool, etc.)
│       ├── commands/             # Tauri command handlers (IPC bridge to frontend)
│       │   ├── mod.rs
│       │   ├── chat.rs           # send_message, stop_streaming, regenerate
│       │   ├── auth.rs           # authenticate, logout, get_auth_state
│       │   ├── conversations.rs  # CRUD conversations + messages
│       │   ├── agents.rs         # CRUD agent personas
│       │   ├── skills.rs         # List/toggle/configure skills
│       │   ├── projects.rs       # CRUD projects + file attachments
│       │   ├── mcp.rs            # MCP server management + tool invocation
│       │   ├── web_research.rs   # Web search + URL fetching
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
│   │   │   ├── client.rs         # MCP server connection + tool invocation
│   │   │   ├── types.rs          # MCP protocol types (tools, resources, prompts)
│   │   │   ├── catalog.rs        # Built-in catalog of popular MCP servers
│   │   │   └── registry.rs       # User-configured MCP server registry
│   │   └── Cargo.toml
│   └── web-research/             # Web search + URL content extraction (zero Tauri dependency)
│       ├── src/
│       │   ├── lib.rs            # Public API
│       │   ├── search.rs         # Web search API client (Bing/Google/etc.)
│       │   ├── fetcher.rs        # URL fetcher + HTML-to-text extraction
│       │   └── types.rs          # Search results, extracted content types
│       └── Cargo.toml
├── static/                       # Static assets served by Vite
├── assets/
│   └── fonts/                    # Bundled fonts (Inter for UI, JetBrains Mono for code)
├── AGENTS.md
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
- **`mcp-client`** is a standalone Rust library with **zero Tauri dependency**. Handles MCP protocol, server connections, tool invocation, and the built-in catalog.
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
- Validate all API responses — don't trust server data shapes blindly
- **No filesystem access** — the app cannot read, write, or browse files on its own. Files only enter via explicit user drag-and-drop or Tauri dialog file picker. File contents are read into memory once; the app never stores or re-accesses file paths.
- **No shell/subprocess execution** — the app must never spawn processes or run commands, **except** for MCP stdio transport (see MCP Security below)
- **No network requests** except to: GitHub Copilot API, GitHub OAuth, user-configured MCP servers, web search API, user-provided URLs, and GitHub Releases API (for auto-update). All non-GitHub network destinations must be explicitly user-configured or user-initiated.
- **URL fetching:** block private IPs (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16), localhost (127.0.0.0/8), link-local (169.254.0.0/16), and cloud metadata (169.254.169.254). Only fetch public HTTPS URLs.
- **Tauri capabilities** must be configured with the minimal set of permissions needed. Audit `capabilities/default.json` in every review cycle.
- **Content Security Policy (CSP)** must be configured in `tauri.conf.json` to prevent XSS. No `unsafe-inline` or `unsafe-eval`.
- **Markdown sanitization** — all rendered markdown must be sanitized with DOMPurify before insertion into the DOM. Never use `{@html}` with unsanitized content.
- **MCP response sanitization** — all MCP tool responses must be sanitized before rendering. Strip scripts from text content, enforce max payload size (e.g., 1MB), validate JSON structure.
- **MCP server connections** are user-managed — the app never auto-discovers or connects to MCP servers without explicit user configuration
- **macOS App Sandbox required** — enforce filesystem and network restrictions at the OS level via entitlements
- Treat any code path that touches the filesystem (outside app data dir) or spawns a non-MCP process as a **security violation**
- **Conversation export exception:** exporting conversations (JSON/Markdown) writes to a user-chosen location via the Tauri save-file dialog. The app never picks a destination on its own — the user explicitly selects the output path through the system file picker.
- This is the **only** permitted filesystem write outside the app data directory.

### MCP Security

MCP supports two transports: **HTTP** and **stdio**. Stdio transport spawns a local process to run
an MCP server binary. This is the **only** exception to the no-subprocess rule:

- Stdio MCP servers may **only** be spawned if the user has explicitly configured them in settings
- The binary path must be user-provided — the app never searches the filesystem for binaries
- Each stdio server launch must be logged and visible in the MCP settings UI
- The app should show a clear confirmation dialog the first time a new stdio server binary is launched
- HTTP transport is preferred and should be the default recommendation in the catalog
- Tauri's `shell` plugin scope must be configured to allow **only** user-configured MCP binaries — no wildcard execution
- If App Sandbox restricts subprocess spawning, document this limitation and fall back to HTTP-only

---

## Key Dependencies

> ⚠️ **Always use the latest stable version.** The packages listed below are recommendations —
> verify versions on [crates.io](https://crates.io) and [npmjs.com](https://www.npmjs.com/) at
> implementation time. If a package has been superseded or deprecated, use the replacement and
> update this table.

### Rust Crates (src-tauri + library crates)

| Crate | Purpose |
|---|---|
| `tauri` v2 | Application framework (with features: `tray-icon`, `protocol-asset`) |
| `tauri-plugin-dialog` | Native file picker + save dialog |
| `tauri-plugin-global-shortcut` | System-wide keyboard shortcuts |
| `tauri-plugin-updater` | Auto-update from GitHub Releases |
| `tauri-plugin-notification` | System notifications (used when app is minimized to tray — e.g., streaming complete, update available) |
| `tauri-plugin-shell` | Limited shell access (MCP stdio only, scoped) |
| `tauri-plugin-clipboard-manager` | Copy to clipboard from code blocks |
| `tauri-plugin-store` | Lightweight key-value persistence for non-sensitive UI preferences (e.g., window position, sidebar width). SQLite `config` table handles all app settings; `tauri-plugin-store` is for ephemeral/UI-state that doesn't warrant a SQL write. |
| `reqwest` | HTTP client (enable `stream` feature for SSE) |
| `serde` / `serde_json` | JSON serialization (shared types Rust ↔ frontend) |
| `tokio` | Async runtime |
| `reqwest-eventsource` | SSE client for streaming responses (wraps reqwest + eventsource-stream with auto-retry) |
| `keyring` | Cross-platform keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager) |
| `rusqlite` | Local persistence (conversations, projects, agents, skills, MCP configs) |
| `image` / `pdf-extract` | Extract text from PDFs/images for file context |
| `thiserror` / `anyhow` | Error handling |
| `log` / `env_logger` | Logging |
| `scraper` / `dom_smoothie` | HTML parsing (`scraper`) + readable content extraction (`dom_smoothie`, Readability algorithm) for URL fetching |
| `url` | URL parsing and validation |
| `rmcp` | Official MCP Rust SDK (Model Context Protocol, spec version 2025-03-26+) |

### Frontend (npm packages)

| Package | Purpose |
|---|---|
| `svelte` v5 | UI framework |
| `@sveltejs/vite-plugin-svelte` | Svelte integration for Vite |
| `vite` | Frontend build tool |
| `typescript` | Type safety |
| `@tauri-apps/api` v2 | Tauri frontend IPC (`invoke`, `listen`, etc.) |
| `@tauri-apps/plugin-dialog` | Frontend bindings for dialog plugin |
| `@tauri-apps/plugin-global-shortcut` | Frontend bindings for global shortcut plugin |
| `@tauri-apps/plugin-updater` | Frontend bindings for updater plugin |
| `@tauri-apps/plugin-notification` | Frontend bindings for notification plugin |
| `@tauri-apps/plugin-shell` | Frontend bindings for shell plugin |
| `@tauri-apps/plugin-clipboard-manager` | Frontend bindings for clipboard plugin |
| `@tauri-apps/plugin-store` | Frontend bindings for store plugin (ephemeral UI state only) |
| `@fontsource-variable/plus-jakarta-sans` | Plus Jakarta Sans variable font (body text, UI) |
| `@fontsource/instrument-serif` | Instrument Serif font (display titles, editorial headings) |
| `marked` | Markdown parsing (fast, CommonMark-compliant) |
| `shiki` | Syntax highlighting (VS Code quality, WASM-based) |
| `dompurify` | HTML sanitization for rendered markdown |
| `vitest` | Frontend unit testing |
| `eslint` | Code linting |
| `prettier` | Code formatting |
| `prettier-plugin-svelte` | Prettier support for `.svelte` files |
| `eslint-plugin-svelte` | ESLint rules for `.svelte` files |
| `@testing-library/svelte` | Component testing utilities |
| `@types/dompurify` | TypeScript definitions for DOMPurify |

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
  *"Generate a concise 4-6 word title for this conversation"* with the first user message + response as context
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
- **Security:** only public HTTPS URLs allowed. Block private IPs, localhost, metadata endpoints (see Security section)
- Extracted content is truncated to a reasonable size (e.g., 50KB of text) before inclusion in context
- Show a URL preview card in the input area (title, domain, favicon if available)

---

## MCP Integration

### Protocol

- Target **MCP specification version 2025-03-26** (or latest stable at implementation time)
- Reference: https://modelcontextprotocol.io/specification
- Support two transports:
  - **HTTP (SSE)** — preferred, works with remote servers. Default for catalog entries.
  - **Stdio** — for local MCP servers. Requires user-approved binary path (see MCP Security). Uses `tauri-plugin-shell` with scoped permissions.

### Built-in Catalog (initial entries)

| Server | Description | Transport |
|---|---|---|
| GitHub | Repository search, issues, PRs | HTTP |
| Web Search | Bing/Google search (if not using built-in) | HTTP |
| Filesystem (read-only) | User-selected directory read access | Stdio |
| PostgreSQL | Database queries | HTTP/Stdio |
| Brave Search | Privacy-focused web search | HTTP |

The catalog is a static list shipped with the app. Users can enable/disable entries and provide required config (API keys, connection strings). The catalog can be extended in future releases.

### Custom Servers

Users can add custom MCP servers in settings (`McpSettings.svelte`):
- **HTTP servers:** URL + optional auth header
- **Stdio servers:** binary path + arguments (user-approved, see MCP Security)
- Test connectivity button to verify the server responds
- View discovered tools/resources from the server

---

## Skills & Agents Concepts

### Definitions

| Concept | What It Is | Example |
|---|---|---|
| **Skill** | A capability/tool that extends what the AI can do. Can be an MCP tool (from a connected MCP server) or a built-in tool (e.g., web search). Legacy Copilot Extensions may also be represented as skills if the API still supports them. | "Web Search", "GitHub PR Lookup", "SQL Query" |
| **Agent** | A named persona with a system prompt, a set of assigned skills, and optionally specific MCP server connections. Agents define *how* the AI behaves and *what tools* it has access to. | "Research Agent" with web search + URL fetch skills |
| **Copilot Extension** | A GitHub-hosted plugin/tool. **Note:** GitHub deprecated Extensions in Nov 2025 in favor of MCP. The app should support them if the API still offers them, but prioritize MCP tools as the primary extensibility mechanism. | `@docker`, `@azure` |
| **MCP Tool** | A tool exposed by a connected MCP server. Also represented as a Skill in this app. | `query_database`, `search_files` |

### How Agents Map to API Calls

When a conversation uses a custom agent, the Rust backend constructs the Copilot API request as follows:

```
System message = [Agent system prompt] + [Project instructions (if any)]
Tools/functions = [Agent's assigned skills as function definitions]
                + [MCP tools from agent's connected MCP servers]
Messages = [Conversation history]
```

- The agent's system prompt is prepended as a `system` role message
- Skills are exposed as `tools` / `functions` in the API request (OpenAI function calling format)
- When the AI calls a tool, the Rust backend routes it: Copilot Extensions → GitHub API, MCP tools → MCP server, built-in tools (web search) → web-research crate
- Tool results are sent back as `tool` role messages in the next API call

### Default Agent

The app ships with a **Default Agent** that cannot be deleted:
- System prompt: minimal (just app context)
- Skills: none by default (user can assign)
- MCP connections: none by default
- All new conversations use the Default Agent unless the user selects another

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
    agent_id TEXT REFERENCES agents(id),
    project_id TEXT REFERENCES projects(id),
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
    is_default INTEGER DEFAULT 0,  -- 1 for the built-in default agent
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
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, mcp_server_id)
);

-- Skills (Copilot Extensions + MCP tools registry)
CREATE TABLE skills (
    id TEXT PRIMARY KEY,           -- Unique skill ID
    name TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL,          -- "extension" or "mcp"
    mcp_server_id TEXT REFERENCES mcp_servers(id),  -- NULL for extensions
    config TEXT,                   -- JSON config blob
    enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL
);

-- MCP server configurations
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    transport TEXT NOT NULL,       -- "http" or "stdio"
    url TEXT,                      -- For HTTP transport
    binary_path TEXT,              -- For stdio transport
    args TEXT,                     -- JSON array of arguments for stdio
    auth_header TEXT,              -- Optional auth for HTTP
    from_catalog INTEGER DEFAULT 0, -- 1 if from built-in catalog
    enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
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

-- ── Indexes (performance-critical queries) ──

CREATE INDEX idx_messages_conversation ON messages(conversation_id, sort_order);
CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);
CREATE INDEX idx_conversations_project ON conversations(project_id);
CREATE INDEX idx_conversations_agent ON conversations(agent_id);
CREATE INDEX idx_conversations_favourite ON conversations(is_favourite) WHERE is_favourite = 1;
CREATE INDEX idx_project_files_project ON project_files(project_id);
CREATE INDEX idx_agent_skills_agent ON agent_skills(agent_id);
CREATE INDEX idx_skills_source ON skills(source);

-- ── Initial seed data ──

INSERT INTO config (key, value) VALUES ('schema_version', '1');
```

### Persistence Rules

- **Conversations, messages, projects, agents, skills, MCP configs** → SQLite (managed by Rust backend)
- **OAuth tokens, API keys** → OS keychain via `keyring` crate (never in SQLite or localStorage)
- **User preferences** (theme, font size, hotkey, send shortcut, auto-update) → SQLite `config` table (e.g., keys: `theme`, `font_size`, `global_hotkey`, `send_shortcut`, `auto_update_enabled`, `auto_update_frequency`)
- **File contents** for project pinned files → SQLite `project_files.content` as BLOB
- **Attached file contents** in chat → stored in `messages.attachments` as metadata only; full content is ephemeral (in-memory during conversation, not persisted)
- **No localStorage/sessionStorage** for sensitive data — all persistence goes through Tauri commands to Rust backend

### Schema Migrations

- Use a `schema_version` key in the `config` table to track the current DB schema version
- On startup, compare `schema_version` against the app's expected version
- Apply sequential migration scripts (embedded in the Rust binary) to bring the schema up to date
- Migrations must be forward-only and non-destructive — never drop data without user consent
- This is critical for auto-update: after an update, the new version may expect a newer schema

### Versioning

- Follow [Semantic Versioning](https://semver.org/) (`MAJOR.MINOR.PATCH`)
- Git tags for releases use the format `vX.Y.Z` (e.g., `v1.2.3`)
- `tauri-plugin-updater` compares the app version against the latest GitHub Release tag
- Pre-release versions (e.g., `v1.0.0-beta.1`) should be excluded from auto-update by default

---

## Implementation Plan

> **Legend:** ✅ = complete, 🔧 = in progress, ⬚ = not started

### Phase 1: Project Scaffolding & Hello World ✅
1. ✅ **project-setup** — Initialize Tauri v2 + Svelte 5 + TypeScript project via `create-tauri-app`. Configure Rust workspace with `src-tauri/` + 3 library crates (`copilot-api`, `mcp-client`, `web-research`). Set up ESLint, Prettier, Vitest. Configure `tauri.conf.json` with minimal capabilities.
2. ✅ **hello-world** — Basic Tauri window with Svelte sidebar + main area layout. Verify hot-reload works (`cargo tauri dev`). Light/dark theme via CSS custom properties.
3. ✅ **design-system** — "Warm Ink" design system applied: Instrument Serif + Plus Jakarta Sans typography, warm paper/ink palette with copper accent, grain texture overlay, entrance animations, editorial welcome screen, refined auth screen with GitHub SVG mark, copper focus glow inputs, hover micro-interactions throughout. All components use CSS custom property tokens.
4. ✅ **ui-polish** — Chat message layout: user messages right-aligned bubbles, assistant messages left-aligned (no avatar). Custom macOS title bar with `titleBarStyle: Overlay`, centered "Chuck" title, sidebar toggle button. Sidebar icon rail: collapses from 260px to 52px with icon-only view (compose, search, settings), jank-free animation via fixed-width inner wrapper + overflow clipping. Labels fade with staggered opacity. Dark mode lightened to warm charcoal palette. Random Chuck Yeager-themed welcome greetings. Window drag region with `core:window:allow-start-dragging` + `acceptFirstMouse`.

### Phase 2: Copilot API Client
4. ⬚ **oauth-device-flow** — GitHub OAuth device flow with token refresh in `copilot-api` crate
5. ⬚ **keychain-storage** — OS keychain token storage (per-platform, using `keyring` crate)
6. ⬚ **chat-completions-client** — `/v1/chat/completions` with SSE streaming + file context in `copilot-api` crate
7. ⬚ **model-discovery** — Query API for available models at startup, cache list, fallback to default

### Phase 3: Persistence & Data Layer
8. ⬚ **sqlite-setup** — Initialize SQLite database with full schema (see Data Model section). Migrations support. Tauri app data directory via `app.path().app_data_dir()`. *(Note: schema + migrations are already implemented in `src-tauri/src/db/migrations.rs` as part of Phase 1. This task covers query functions in `db/queries.rs` and wiring CRUD operations.)*
9. ⬚ **conversation-persistence** — CRUD for conversations + messages via Tauri commands. Load on startup, lazy-load older messages. Auto-generate conversation titles via lightweight API call after first response.
10. ⬚ **draft-auto-save** — Persist input text to `drafts` table every few seconds. Restore on launch. Clear on successful send.

### Phase 4: Core Chat UI
11. ⬚ **sidebar** — `Sidebar.svelte`: conversation list grouped by date, new chat, favourites (pinned), projects, agents, search, collapsible sections *(Note: basic sidebar skeleton with search + settings is already implemented. This task adds real data binding, grouped conversation list, favourites, and collapsible sections.)*
12. ⬚ **chat-view** — `ChatView.svelte` + `MessageBubble.svelte` + `ThinkingSection.svelte`: message list with avatars, timestamps, collapsible reasoning/thinking sections, context summarization banner *(Note: basic chat view with demo responses exists. This task replaces demo with real API streaming, adds ThinkingSection and summarization banner.)*
13. ⬚ **input-area** — `InputArea.svelte`: multi-line `<textarea>`, file drop zone, attachment pills, agent selector, model selector, loading state *(Note: basic input with textarea and model selector exists. This task adds file drop zone, attachment pills, agent selector, and loading state.)*
14. ⬚ **streaming-display** — Token-by-token rendering via Tauri events, typing cursor animation, stop button. Save partial response on interruption.
15. ⬚ **message-actions** — Edit sent messages (discard + re-send), regenerate last response, copy individual messages
16. ⬚ **in-conversation-search** — `SearchOverlay.svelte`: Cmd+F / Ctrl+F to find text, highlight matches, navigate with arrows

### Phase 5: Markdown & Code Rendering
17. ⬚ **markdown-rendering** — Render assistant messages with `marked` + `DOMPurify`. Bold, italic, headers, lists, links, blockquotes, tables.
18. ⬚ **code-blocks** — `CodeBlock.svelte`: syntax-highlighted fenced blocks via `shiki`, copy button, language label

### Phase 6: Web Research
19. ⬚ **web-search** — `web-research` crate: Bing Web Search API integration. Tauri command `web_search`. `WebResultCard.svelte` for displaying results as cited cards. API key stored in keychain.
20. ⬚ **url-fetcher** — Tauri command `fetch_url`. HTTPS only, public IPs only. Extract readable text via `dom_smoothie`. URL preview card in input area. Max 50KB extracted text.

### Phase 7: MCP Integration
21. ⬚ **mcp-client** — `mcp-client` crate: MCP protocol client (spec 2025-03-26). Connect, discover tools, invoke, handle responses. HTTP and stdio transports.
22. ⬚ **mcp-catalog** — Built-in catalog of popular MCP servers. One-click enable. Show descriptions, required config fields. Persist enabled state to SQLite.
23. ⬚ **mcp-settings** — `McpSettings.svelte`: manage MCP connections. Add custom servers (URL + auth or binary path), enable/disable, test connectivity, browse discovered tools.

### Phase 8: Skills & Agents
24. ⬚ **skills-manager** — `SkillsPanel.svelte`: browse Copilot Extensions + MCP tools as unified skill list. Toggle on/off, configure per-skill settings. Persist to SQLite.
25. ⬚ **agents-manager** — `AgentsPanel.svelte`: create/edit/delete custom agent personas. Each agent has name, avatar, system prompt, assigned skills, MCP connections. Default agent is built-in and undeletable.
26. ⬚ **agent-selector** — Agent picker in `InputArea.svelte`. Conversations tied to an agent. Agent config maps to API request structure.

### Phase 9: Projects & File Context
27. ⬚ **projects** — `ProjectView.svelte`: named project containers with custom instructions, pinned files (stored as BLOBs in SQLite), grouped conversations. Project selector in sidebar.
28. ⬚ **file-context** — User-initiated only: read file contents into memory via drag-and-drop or `tauri-plugin-dialog` file picker. Preview in input. Never retain paths or re-read from disk.
29. ⬚ **context-window** — Implement conversation summarization for long chats. Older messages summarized into condensed recap. Visual indicator when summarization has occurred.

### Phase 10: Polish & Platform Features
30. ⬚ **settings-panel** — `SettingsPanel.svelte`: account, theme, font size, default model, keyboard shortcuts, MCP management, conversation export (JSON + Markdown), database size display + cleanup, clear history
31. ⬚ **global-hotkey** — System-wide app summon via `tauri-plugin-global-shortcut` (Cmd+Shift+Space or configurable)
32. ⬚ **system-tray** — Tauri core `tray-icon` feature: minimize to tray instead of closing. Streaming continues when window is hidden. Right-click menu: New Chat, Show, Quit. Status indicator.
33. ⬚ **keyboard-shortcuts** — Cmd+N (new chat), Cmd+K (search conversations), Cmd+F (search in conversation), Cmd+, (settings), Cmd+Shift+S (toggle sidebar), Escape (cancel streaming). Send shortcut configuration (Enter vs Cmd+Enter / Ctrl+Enter) persisted via `send_shortcut` config key.
34. ⬚ **offline-mode** — Detect network status. Full read access when offline, sending disabled with clear indicator. Auto-reconnect with "Back online" toast.
35. ⬚ **accessibility** — Semantic HTML, ARIA roles/labels, keyboard navigation, focus management, visible focus indicators, screen reader testing

### Phase 11: Auto-Update
36. ⬚ **auto-update** — Configure `tauri-plugin-updater` with GitHub Releases endpoint. `UpdateBanner.svelte` for notifications. Show changelog/release notes. Allow "skip this version" and "remind me later". Settings toggle to disable auto-update. Ed25519 signature verification.

### Phase 12: Distribution
37. ⬚ **app-packaging** — `cargo tauri build` for all platforms. `.dmg` (macOS with code signing + App Sandbox + notarization), `.AppImage`/`.deb` (Linux), `.msi`/`.nsis` (Windows). GitHub Actions CI/CD for automated builds. Publish releases to GitHub Releases for auto-update consumption.

---

## Build & Run

```bash
# Prerequisites:
#   - Rust toolchain (rustup)
#   - Node.js 20+ and pnpm
#   - Platform-specific: Xcode CLI tools (macOS), webkit2gtk + libjavascriptcoregtk (Linux), WebView2 (Windows)

# Install frontend dependencies
pnpm install

# Development (hot-reload frontend + Rust backend)
cargo tauri dev

# Build for production (current platform)
cargo tauri build

# --- Individual checks ---

# Rust: build all crates
cargo build --workspace

# Rust: lint
cargo clippy --workspace -- -D warnings

# Rust: format check
cargo fmt --all -- --check

# Rust: tests
cargo test --workspace

# Rust: security audit
cargo audit

# Frontend: type check (svelte-check)
pnpm check

# Frontend: lint (ESLint + Prettier)
pnpm lint

# Frontend: tests (Vitest)
pnpm test

# Frontend: production build
pnpm build

# Frontend: security audit
pnpm audit

# Update dependencies
cargo update && pnpm update
```

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Copilot API not officially public | API could change or break | Modular client design in `copilot-api` crate — easy to swap to official SDK when available |
| Short-lived OAuth tokens | Auth interruptions | Auto-refresh logic + graceful re-auth prompts |
| WebView differences across platforms | Inconsistent rendering | Test on all 3 platforms in CI; use standard CSS; avoid browser-specific APIs |
| Tauri v2 plugin ecosystem gaps | Missing functionality | Check plugin availability early; fall back to custom Tauri commands if needed |
| SSE streaming performance in webview | Janky UI during fast responses | Batch DOM updates, use `requestAnimationFrame`, debounce store updates |
| MCP server reliability | Tool calls may fail or timeout | Timeout handling, retry logic, graceful fallback in chat |
| MCP security surface | Untrusted servers could return harmful data | User must explicitly add servers; validate/sanitize all MCP responses; DOMPurify |
| Web search API costs/limits | Rate limiting or billing | Cache results, respect rate limits, show clear errors |
| Large conversation DB | Slow queries, high disk usage | Indexed columns, lazy loading, pagination, cleanup UI, 500MB warning |
| Schema migration on update | Data loss or app crash after update | Forward-only migrations, backup DB before migration, test migrations in CI |

---

## Design Reference

The UX is modeled after **Claude Desktop** (Anthropic's desktop app):
- Clean, minimal sidebar with conversation history
- Central chat panel with streaming markdown responses
- File attachment via drag-and-drop with visual pills
- Project-based organization with custom instructions
- Light/dark theme with system preference detection
- Global hotkey to summon from anywhere

**Key difference:** This app has **no access to the user's machine** — no filesystem browsing, no shell execution, no screen capture. All external capabilities come through explicit user actions (file attach, URL paste) or user-configured connections (MCP servers, web search). It includes seamless auto-updates from GitHub Releases. It's a powerful but sandboxed chat client for GitHub Copilot with extensibility via MCP and custom agents.

**Why Tauri + Svelte:** Tauri v2 provides a production-ready, security-first desktop framework with built-in system tray, global shortcuts, auto-updater, and native dialogs. Svelte 5 delivers a lightweight, reactive frontend with minimal bundle size and excellent developer experience. Together they produce a fast, small (~5-10MB), cross-platform app with world-class UI capabilities and strong security guarantees.

### Visual Design System: "Warm Ink"

The app uses a distinctive **"Warm Ink"** aesthetic — warm paper/ink neutrals with a copper accent. This avoids generic "AI slop" aesthetics (Inter font, purple gradients, etc.) and gives the app an editorial, tactile personality.

**Typography:**
- **Display:** Instrument Serif (italic) — used for welcome titles, auth screen, section headers
- **Body:** Plus Jakarta Sans Variable — used for all UI text
- **Monospace:** JetBrains Mono — used for code blocks, device codes
- Font packages: `@fontsource-variable/plus-jakarta-sans`, `@fontsource/instrument-serif`

**Color Palette:**
- Light: warm paper backgrounds (`#faf9f7`, `#f3f1ed`, `#eae7e1`), ink-dark text (`#1c1917`), copper accent (`#b45309`)
- Dark: deep warm charcoal (`#0f0e0d`, `#171615`), warm light text (`#e7e5e4`), amber accent (`#d97706`)
- Accent buttons use ink-dark (same as text); focus rings and highlights use copper

**Visual Details:**
- Subtle SVG-based grain texture overlay on `body::after` at 2.5% opacity
- Copper glow focus ring on input (`--shadow-input-focus`)
- Entry animations: `fadeIn`, `fadeInUp`, `scaleIn` with staggered delays
- Hover micro-interactions: `translateX` on sidebar items, `scale` on send button
- Spring easing for playful transitions: `cubic-bezier(0.34, 1.56, 0.64, 1)`

**CSS Architecture:**
- All design tokens as CSS custom properties in `src/app.css` (70+ variables)
- Three theme modes: `:root` (light default), `[data-theme="dark"]`, `[data-theme="system"]`
- Components must use `var(--token-name)` — no hardcoded colors, font sizes, or spacing values
- Global reset, scrollbar styling, focus-visible, selection colors defined in `app.css`
