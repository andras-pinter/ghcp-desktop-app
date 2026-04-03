# Copilot Desktop — Agent Instructions

> A native, cross-platform desktop chat GUI for GitHub Copilot, built with **Rust + GPUI**.
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
┌─────────────────────────┐
│   REVIEW all changes:   │
│   - cargo build         │
│   - cargo clippy        │
│   - cargo test          │
│   - cargo fmt --check   │
│   - cargo audit         │
│   - Manual code review  │
│   - Security audit      │
│   - Doc completeness    │
│   - Dependency check    │
└────────────┬────────────┘
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
> - "The build tools don't apply (no code yet)" — **Partially valid.** Skip `cargo build/clippy/test/fmt` only if there is literally no Rust code in the repo. Still run manual review, security audit, doc completeness, and consistency checks.
> - "I'll do it later" — **No.** The review happens NOW, before the task is marked complete.

### Review Checklist (every cycle)

- [ ] `cargo build --workspace` compiles with zero warnings
- [ ] `cargo clippy --workspace -- -D warnings` passes with zero diagnostics
- [ ] `cargo test --workspace` — all tests pass, no skipped tests without justification
- [ ] `cargo fmt --all -- --check` — formatting is clean
- [ ] **Code review** — logic is correct, no dead code, no TODOs left behind, no hardcoded values
- [ ] **Security review** — no filesystem access beyond app data dir, no token leaks, no unsanitized inputs
- [ ] **Error handling** — all error paths handled, user-friendly messages, no panics/unwraps in production code
- [ ] **Documentation** — all public items have doc comments, README/AGENTS.md updated if needed
- [ ] **Tests** — new code has tests, edge cases covered, integration tests for API interactions
- [ ] **Dependencies** — all crates are latest stable versions, actively maintained, no deprecated or unmaintained crates

### 2. Update Everything

When an agent completes a task, it **MUST** update **all** affected artifacts:

- **Code** — the implementation itself
- **Tests** — new/updated tests covering the changes
- **Documentation** — doc comments, README.md, AGENTS.md (if architecture/scope/phases changed)
- **Dependencies** — Cargo.toml updated, lock file committed
- **Types** — all type definitions, interfaces, and models updated consistently across crates
- **State** — app state models, SQLite schemas, config structures updated
- **Views** — any UI that references changed models/state must be updated
- **Sibling crates** — if a change in `copilot-api` affects `app`, update `app` too
- **Plan** — if the task reveals new work or changes scope, update the plan

**"Update everything" means: no partial changes.** If you modify a struct in `copilot-api/types.rs`,
you MUST also update every file that uses that struct. If you add a new feature, you MUST add it to
the settings UI, keyboard shortcuts, and documentation. If you rename something, you MUST rename it
everywhere. Agents must grep/search the entire workspace to find all references before considering
a change complete.

### 3. Multi-Agent Review Dispatch

For any non-trivial task, the review cycle SHOULD be split across multiple agents:

| Review Agent | Responsibility |
|---|---|
| **Build Agent** | Compile, clippy, fmt, test, audit — mechanical correctness |
| **Code Review Agent** | Logic, architecture, patterns, dead code, consistency |
| **Security Agent** | Filesystem isolation, token handling, input sanitization, network boundaries |
| **Docs Agent** | Doc comments, README, AGENTS.md, inline comments where needed |

Each agent independently reviews and reports issues. ALL reported issues must be fixed
before the task is considered complete. Then the full review cycle runs again.

---

## Project Overview

Copilot Desktop is a standalone desktop application that provides a conversational chat interface
for GitHub Copilot. Think of it as "Claude Desktop, but for Copilot" — a polished, GPU-accelerated
native app with conversation management, file attachments, projects, web research, MCP tool
integration, custom agent personas, and streaming responses.

### Stack

- **Language:** Rust (2021 edition)
- **UI Framework:** [GPUI](https://gpui.rs/) — GPU-accelerated UI framework from the Zed editor
- **Backend API:** GitHub Copilot `/v1/chat/completions` (OAuth token-based, SSE streaming)
- **Storage:** SQLite (via `rusqlite`) for local conversation persistence
- **Platforms:** macOS (Metal), Linux (Vulkan), Windows (DirectX — maturing)

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
- **Light/dark theme** — follow system preference, manual toggle
- **Global hotkey** — summon the app from anywhere (e.g., Cmd+Shift+Space)
- **Keyboard shortcuts** — standard app navigation
- **Conversation persistence** — local SQLite storage
- **Secure auth** — OAuth device flow + OS keychain token storage
- **Auto-update** — check for new versions on startup (and periodically), download + apply seamlessly from GitHub Releases

### ⛔ Hard Requirement: No Filesystem / Machine Access

**This app must NEVER access the user's machine beyond what the user explicitly provides.**

- The app has **zero access** to the filesystem — it cannot read, write, browse, or scan any files or directories on its own
- The **only** way files enter the app is through explicit user action: drag-and-drop or file picker dialog
- File contents are read **once** into memory at the moment the user attaches them — the app does not retain file paths or re-read from disk
- The app stores **only** its own data: conversations (SQLite in app data dir), auth tokens (OS keychain), and user preferences (app config dir)
- No shell execution, no subprocess spawning, no system command access — **with one exception:** MCP stdio transport may spawn user-approved MCP server binaries (see MCP Security below)
- No screen capture, no clipboard snooping, no background scanning
- No network requests except to: GitHub Copilot API, GitHub OAuth endpoints, **user-configured MCP servers**, **web search API**, **user-provided URLs**, and **GitHub Releases API** (for auto-update)
- All outbound network destinations beyond GitHub must be **explicitly configured or initiated by the user**
- **URL fetching safeguards:** the app must block requests to private IP ranges (10.x, 172.16-31.x, 192.168.x), localhost, link-local (169.254.x), and cloud metadata endpoints (169.254.169.254). Only fetch public HTTPS URLs.
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

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                       GPUI Application                           │
│                                                                  │
│  ┌──────────────┐  ┌──────────────────────────────────────────┐  │
│  │   Sidebar    │  │            Main Panel                    │  │
│  │              │  │                                          │  │
│  │ [New Chat]   │  │  ┌──────────────────────────────────┐   │  │
│  │              │  │  │      Message List (scrollable)    │   │  │
│  │ Agents ▾     │  │  │                                  │   │  │
│  │  • Research  │  │  │  [User]  How do I parse JSON?    │   │  │
│  │  • Coder     │  │  │                                  │   │  │
│  │              │  │  │  [Copilot] You can use serde...  │   │  │
│  │ Projects ▾   │  │  │  ```rust                         │   │  │
│  │  └─ Convos   │  │  │  use serde::Deserialize; [Copy]  │   │  │
│  │              │  │  │  ```                             │   │  │
│  │ Recent       │  │  │                                  │   │  │
│  │  • Chat 1    │  │  │  🌐 [Web result: serde docs]    │   │  │
│  │  • Chat 2    │  │  │                                  │   │  │
│  │              │  │  └──────────────────────────────────┘   │  │
│  │ Search 🔍    │  │                                          │  │
│  │              │  │  ┌──────────────────────────────────┐   │  │
│  │ Skills ⚡    │  │  │ [📎 Attach] [🌐 Web] Message... │   │  │
│  │ [⚙ Settings] │  │  │ [Agent: Research ▾]    [Send ➤] │   │  │
│  └──────────────┘  └──────────────────────────────────────────┘  │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │  App State (Model)                        │   │
│  │  conversations[] │ agents[] │ skills[] │ mcp_connections[] │   │
│  │  active_project  │ config   │ auth_state │ update_state     │   │
│  └────────────────────────────┬──────────────────────────────┘   │
└───────────────────────────────┼──────────────────────────────────┘
                                │
       ┌────────────────────────┼────────────────────────────┐
       │                        │                            │
┌──────┴───────────┐  ┌────────┴─────────┐  ┌──────────────┴──────┐
│ Copilot API      │  │  MCP Client      │  │  Web Research       │
│ OAuth + SSE Chat │  │  Tool calls to   │  │  Search API +       │
│ Token Refresh    │  │  user-configured  │  │  URL fetcher +      │
│ File Context     │  │  MCP servers     │  │  content extraction  │
└──────┬───────────┘  └────────┬─────────┘  └──────────────┬──────┘
       │                       │                            │
┌──────┴──────┐    ┌───────────┴──────────┐    ┌───────────┴────────┐
│ GitHub API  │    │ MCP Servers          │    │ Web (search API +  │
│ /v1/chat/   │    │ (user-configured)    │    │  user-provided     │
│ completions │    │                      │    │  URLs)             │
└─────────────┘    └──────────────────────┘    └────────────────────┘
```

---

## Project Structure

```
copilot-desktop/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── app/                    # Main application binary
│   │   ├── src/
│   │   │   ├── main.rs         # Entry point, window setup, global hotkey
│   │   │   ├── app.rs          # Root application component (sidebar + main panel layout)
│   │   │   ├── views/
│   │   │   │   ├── sidebar.rs  # Conversation list, project browser, agents, search
│   │   │   │   ├── chat.rs     # Chat view (message list + input)
│   │   │   │   ├── message.rs  # Individual message rendering (markdown + code blocks + web results)
│   │   │   │   ├── input.rs    # Multi-line input with file attachment + URL input + agent selector
│   │   │   │   ├── auth.rs     # OAuth login/welcome screen
│   │   │   │   ├── settings.rs # Settings panel (modal or slide-over)
│   │   │   │   ├── project.rs  # Project detail view (instructions, files, conversations)
│   │   │   │   ├── agents.rs   # Agent management: create/edit/delete custom agent personas
│   │   │   │   ├── skills.rs   # Skills/extensions browser: enable/disable/configure
│   │   │   │   └── update.rs   # Auto-update notification banner + download progress
│   │   │   ├── state/
│   │   │   │   ├── mod.rs      # App state model
│   │   │   │   ├── conversation.rs  # Conversation + message models
│   │   │   │   ├── project.rs  # Project model (name, instructions, attached files)
│   │   │   │   ├── agent.rs    # Agent model (name, system prompt, assigned skills, MCP connections)
│   │   │   │   ├── skill.rs    # Skill/extension model (id, name, enabled, config)
│   │   │   │   ├── mcp.rs      # MCP server connection state + catalog
│   │   │   │   ├── web.rs      # Web search results + URL fetch state (ephemeral)
│   │   │   │   ├── update.rs   # Auto-update state (available version, download progress, skipped versions)
│   │   │   │   └── config.rs   # User preferences
│   │   │   └── theme/
│   │   │       ├── mod.rs      # Theme definitions (light + dark)
│   │   │       └── colors.rs   # Color palette
│   │   └── Cargo.toml
│   ├── copilot-api/            # GitHub Copilot API client library
│   │   ├── src/
│   │   │   ├── lib.rs          # Public API
│   │   │   ├── auth.rs         # OAuth device flow + token refresh
│   │   │   ├── client.rs       # HTTP client + SSE streaming
│   │   │   ├── types.rs        # Request/response types (messages, roles, attachments)
│   │   │   └── keychain.rs     # Secure token/API key storage (cross-platform via `keyring`)
│   │   └── Cargo.toml
│   ├── mcp-client/             # MCP (Model Context Protocol) client library
│   │   ├── src/
│   │   │   ├── lib.rs          # Public API
│   │   │   ├── client.rs       # MCP server connection + tool invocation
│   │   │   ├── types.rs        # MCP protocol types (tools, resources, prompts)
│   │   │   ├── catalog.rs      # Built-in catalog of popular MCP servers
│   │   │   └── registry.rs     # User-configured MCP server registry
│   │   └── Cargo.toml
│   ├── web-research/           # Web search + URL content extraction
│   │   ├── src/
│   │   │   ├── lib.rs          # Public API
│   │   │   ├── search.rs       # Web search API client (Bing/Google/etc.)
│   │   │   ├── fetcher.rs      # URL fetcher + HTML-to-text extraction
│   │   │   └── types.rs        # Search results, extracted content types
│   │   └── Cargo.toml
│   └── markdown-render/        # Markdown + code rendering for GPUI
│       ├── src/
│       │   ├── lib.rs          # Markdown-to-GPUI element tree
│       │   └── syntax.rs       # Syntax highlighting (syntect)
│       └── Cargo.toml
├── assets/
│   ├── icons/                  # App icons (macOS .icns, Linux .png, Windows .ico)
│   └── fonts/                  # Bundled fonts (e.g., Inter for UI, JetBrains Mono for code)
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

### Dependencies Policy

**Only stable, actively maintained crates at their latest version. No exceptions.**

- **Always use the latest stable release** of every dependency. When adding a crate, check [crates.io](https://crates.io) or [lib.rs](https://lib.rs) for the current version — do not guess or use old versions from memory.
- **No unmaintained crates.** Before adding a dependency, verify it has been updated within the last 12 months and has no "unmaintained" advisory on [RustSec](https://rustsec.org/).
- **No deprecated crates.** If a crate is deprecated in favor of a successor, use the successor.
- **Run `cargo audit`** as part of every review cycle to detect known vulnerabilities in dependencies.
- **Run `cargo update`** regularly to pick up patch/minor version bumps. Lock file (`Cargo.lock`) must be committed.
- **Prefer well-established crates** with broad ecosystem adoption (high download counts, active issue trackers, multiple contributors) over niche alternatives.
- If a listed dependency in this document is outdated or superseded by the time implementation begins, **use the better alternative** and update this document accordingly.

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

**Scope** is the crate or area affected: `app`, `copilot-api`, `mcp-client`, `web-research`, `markdown-render`, `deps`, `ci`

**Examples:**

```
feat(copilot-api): add SSE streaming for chat completions
fix(app): prevent layout thrashing during rapid token updates
docs: update AGENTS.md with MCP security section
refactor(mcp-client): extract transport layer into separate module
chore(deps): update reqwest to 0.12.5
test(web-research): add URL validation tests for private IP blocking
```

**Rules:**
- Subject line: imperative mood, lowercase, no period, max 72 characters
- Body: wrap at 72 characters, explain *what* and *why* (not *how*)
- Breaking changes: add `BREAKING CHANGE:` footer or `!` after type/scope
- Reference issues/tasks when applicable
- **Type must match the actual change** — `feat`, `fix`, `refactor`, `test`, `perf` are for **code changes only**. If a commit only touches documentation (e.g., AGENTS.md, README), use `docs:` regardless of what the documentation describes. Reserve code-level types for commits that modify `.rs`, `.toml`, or other source/config files.

### GPUI Patterns

- Follow GPUI's component model: views own their state via `Model<T>` / `View<T>`
- Use GPUI's built-in layout system (flex-based) — do not reach for CSS or HTML
- Keep view structs focused: one file per view, state logic in `state/` modules
- Use GPUI actions for user interactions (keyboard shortcuts, buttons)
- Avoid `unsafe` code unless absolutely required by GPUI internals

### Crate Boundaries

- **`copilot-api`** is a standalone library with **zero GPUI dependency**. It should be usable
  from any Rust project (CLI, different GUI framework, etc.)
- **`mcp-client`** is a standalone library with **zero GPUI dependency**. Handles MCP protocol,
  server connections, tool invocation, and the built-in catalog.
- **`web-research`** is a standalone library with **zero GPUI dependency**. Handles web search
  API calls and URL content fetching/extraction.
- **`markdown-render`** depends on GPUI for element types but not on any other crate
- **`app`** depends on all other crates — it is the only crate with GPUI views
- No circular dependencies between crates

### Error Handling

- API client errors should be typed and descriptive (auth failures, rate limits, network errors)
- UI should display user-friendly error messages — never show raw stack traces
- Network failures should suggest retry; auth failures should redirect to login

### Security

- **Never log or display OAuth tokens or API keys** in any output
- Tokens and API keys must be stored only in the OS keychain — never in plain text files or config
- Validate all API responses — don't trust server data shapes blindly
- **No filesystem access** — the app cannot read, write, or browse files on its own. Files only enter via explicit user drag-and-drop or file picker. File contents are read into memory once; the app never stores or re-accesses file paths.
- **No shell/subprocess execution** — the app must never spawn processes or run commands, **except** for MCP stdio transport (see MCP Security below)
- **No network requests** except to: GitHub Copilot API, GitHub OAuth, user-configured MCP servers, web search API, user-provided URLs, and GitHub Releases API (for auto-update). All non-GitHub network destinations must be explicitly user-configured or user-initiated.
- **URL fetching:** block private IPs (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16), localhost (127.0.0.0/8), link-local (169.254.0.0/16), and cloud metadata (169.254.169.254). Only fetch public HTTPS URLs.
- **MCP server connections** are user-managed — the app never auto-discovers or connects to MCP servers without explicit user configuration
- **MCP response sanitization** — all MCP tool responses must be sanitized before rendering. Strip HTML/scripts from text content, enforce max payload size (e.g., 1MB), validate JSON structure.
- **macOS App Sandbox required** — enforce filesystem and network restrictions at the OS level via entitlements
- Treat any code path that touches the filesystem (outside app data dir) or spawns a non-MCP process as a **security violation**
- **Auto-update exception:** the `self_update` crate performs an atomic binary swap of the app's own executable. This is the **only** permitted filesystem write outside the app data directory, and it requires explicit user confirmation before executing.

### MCP Security

MCP supports two transports: **HTTP** and **stdio**. Stdio transport spawns a local process to run
an MCP server binary. This is the **only** exception to the no-subprocess rule:

- Stdio MCP servers may **only** be spawned if the user has explicitly configured them in settings
- The binary path must be user-provided — the app never searches the filesystem for binaries
- Each stdio server launch must be logged and visible in the MCP settings UI
- The app should show a clear confirmation dialog the first time a new stdio server binary is launched
- HTTP transport is preferred and should be the default recommendation in the catalog
- If App Sandbox restricts subprocess spawning, document this limitation and fall back to HTTP-only

---

## Key Dependencies

> ⚠️ **Always use the latest stable version.** The crates listed below are recommendations —
> verify versions on [crates.io](https://crates.io) at implementation time. If a crate has been
> superseded or deprecated, use the replacement and update this table.

| Crate | Purpose |
|---|---|
| `gpui` | UI framework (pin to a specific release tag) |
| `reqwest` | HTTP client (enable `stream` feature for SSE) |
| `serde` / `serde_json` | JSON serialization |
| `tokio` | Async runtime |
| `eventsource-stream` | SSE parsing for streaming responses |
| `keyring` | Cross-platform keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager) |
| `syntect` | Syntax highlighting for code blocks |
| `pulldown-cmark` | Markdown parsing |
| `rusqlite` | Local persistence (conversations, projects, agents, skills, MCP configs) |
| `image` / `pdf-extract` | Extract text from PDFs/images for file context |
| `directories` | XDG / platform-appropriate config paths |
| `global-hotkeys` | System-wide keyboard shortcut registration |
| `thiserror` / `anyhow` | Error handling |
| `log` / `env_logger` | Logging |
| `scraper` / `readability` | HTML-to-text extraction for URL fetching |
| `url` | URL parsing and validation |
| `rmcp` or custom | MCP protocol client (target spec version 2025-03-26) |
| `self_update` | Auto-update from GitHub Releases (version check, download, atomic binary swap) |

---

## GitHub Copilot API Integration

### Authentication

Uses the **OAuth device flow** — the same flow VS Code uses to authenticate with Copilot:

1. App requests a device code from GitHub
2. User opens a browser URL and enters the code
3. App polls for the OAuth token
4. Token is stored in the OS keychain
5. Token is refreshed automatically before expiry; if refresh fails, prompt re-auth

### Chat Completions

- Endpoint: `POST /v1/chat/completions`
- Request body follows the OpenAI-compatible chat completions format
- Streaming via Server-Sent Events (SSE) — `stream: true`
- File context is included as part of the message content (text extracted from files)
- System messages can carry project-level custom instructions

### Rate Limits & Errors

- Respect `Retry-After` headers on 429 responses
- Show a non-intrusive toast/banner for rate limit warnings
- Gracefully degrade if the API is unreachable (show cached conversations, disable send)

---

## Auto-Update

### Mechanism

- Uses the `self_update` crate to check for and apply updates from **GitHub Releases**
- On startup (and at a configurable interval), the app queries the GitHub Releases API for the latest version
- Compares the current binary version (`CARGO_PKG_VERSION`) against the latest release tag
- If a new version is available, shows a **non-intrusive notification** in the app (not a blocking dialog)

### Update Flow

1. App checks GitHub Releases API → finds newer version
2. Shows banner: "Version X.Y.Z is available" with changelog summary
3. User clicks "Update now" → downloads platform-specific binary from the release assets
4. Verifies download integrity (checksum + optional signature)
5. Performs atomic binary swap via `self_update`
6. Prompts user to restart the app

### User Controls (in Settings)

- **Auto-check for updates:** on/off (default: on)
- **Check frequency:** startup only, daily, weekly
- **"Skip this version"** — suppress notifications for a specific release
- **"Remind me later"** — snooze for a configurable period

### Security

- Only fetch from the project's own GitHub Releases — no third-party update servers
- Verify release asset checksums before applying
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
- Search results are displayed as **cited cards** in the chat: title, snippet, source URL
- Results are cached in-memory for the session to avoid redundant API calls
- Rate limits: respect API quotas, show clear error if quota exceeded

### URL Fetching

- User pastes a URL → app fetches the page over HTTPS → extracts readable text via `readability` algorithm
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
  - **Stdio** — for local MCP servers. Requires user-approved binary path (see MCP Security).

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

Users can add custom MCP servers in settings:
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

When a conversation uses a custom agent, the app constructs the Copilot API request as follows:

```
System message = [Agent system prompt] + [Project instructions (if any)]
Tools/functions = [Agent's assigned skills as function definitions]
                + [MCP tools from agent's connected MCP servers]
Messages = [Conversation history]
```

- The agent's system prompt is prepended as a `system` role message
- Skills are exposed as `tools` / `functions` in the API request (OpenAI function calling format)
- When the AI calls a tool, the app routes it: Copilot Extensions → GitHub API, MCP tools → MCP server, built-in tools (web search) → web-research crate
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
(`directories::data_dir()/copilot-desktop/data.db`).

### Tables

```sql
-- Conversations
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,           -- UUID
    title TEXT,                    -- Auto-generated or user-edited
    agent_id TEXT REFERENCES agents(id),
    project_id TEXT REFERENCES projects(id),
    model TEXT,                    -- Model used (e.g., "gpt-4o")
    created_at TEXT NOT NULL,      -- ISO 8601
    updated_at TEXT NOT NULL
);

-- Messages
CREATE TABLE messages (
    id TEXT PRIMARY KEY,           -- UUID
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL,            -- "user", "assistant", "system", "tool"
    content TEXT NOT NULL,         -- Message text (markdown for assistant)
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
```

### Persistence Rules

- **Conversations, messages, projects, agents, skills, MCP configs** → SQLite
- **OAuth tokens, API keys** → OS keychain (never in SQLite)
- **User preferences** (theme, font size, hotkey, auto-update) → SQLite `config` table
- **File contents** for project pinned files → SQLite `project_files.content` as BLOB
- **Attached file contents** in chat → stored in `messages.attachments` as metadata only; full content is ephemeral (in-memory during conversation, not persisted)

### Schema Migrations

- Use a `schema_version` key in the `config` table to track the current DB schema version
- On startup, compare `schema_version` against the app's expected version
- Apply sequential migration scripts (embedded in the binary) to bring the schema up to date
- Migrations must be forward-only and non-destructive — never drop data without user consent
- This is critical for auto-update: after a binary swap, the new version may expect a newer schema

### Versioning

- Follow [Semantic Versioning](https://semver.org/) (`MAJOR.MINOR.PATCH`)
- Git tags for releases use the format `vX.Y.Z` (e.g., `v1.2.3`)
- `self_update` compares `CARGO_PKG_VERSION` against the latest GitHub Release tag
- Pre-release versions (e.g., `v1.0.0-beta.1`) should be excluded from auto-update by default

---

## Implementation Plan

### Phase 1: Project Scaffolding & GPUI Hello World
1. **project-setup** — Initialize Rust workspace, configure 5 crates, pin GPUI
2. **gpui-hello-world** — Basic GPUI window with sidebar + main area layout

### Phase 2: Copilot API Client
3. **oauth-device-flow** — GitHub OAuth device flow with token refresh
4. **keychain-storage** — OS keychain token storage (per-platform, using `keyring` crate)
5. **chat-completions-client** — `/v1/chat/completions` with SSE streaming + file context

### Phase 3: Persistence & Data Layer
6. **sqlite-setup** — Initialize SQLite database with full schema (see Data Model section). Migrations support for future schema changes.
7. **conversation-persistence** — CRUD for conversations + messages. Load on startup, lazy-load older messages. Auto-generate conversation titles.

### Phase 4: Core Chat UI (Claude Desktop-style)
8. **sidebar** — Conversation list grouped by date, new chat, projects, agents, search, collapsible
9. **chat-view** — Message list with avatars, timestamps, thinking indicator
10. **input-area** — Multi-line input, file drop zone, attachment pills, agent selector, loading state
11. **streaming-display** — Token-by-token rendering with cursor animation, stop button

### Phase 5: Markdown & Code Rendering
12. **markdown-parser** — Bold, italic, headers, lists, links, code, blockquotes, tables
13. **code-blocks** — Syntax-highlighted fenced blocks with copy button + language label

### Phase 6: Web Research
14. **web-search** — Integrate Bing Web Search API. Triggered by AI (function calling) or user (🌐 button). Results displayed as cited cards in chat. API key stored in keychain.
15. **url-fetcher** — User pastes a URL → app fetches page (HTTPS only, public IPs only) → extracts readable text → includes in context. URL preview card in input area. Max 50KB extracted text.

### Phase 7: MCP Integration
16. **mcp-client** — MCP protocol client (spec 2025-03-26): connect, discover tools, invoke, handle responses. HTTP and stdio transports.
17. **mcp-catalog** — Built-in catalog of popular MCP servers with one-click enable. Show descriptions, required config fields. Persist enabled state to SQLite.
18. **mcp-settings** — UI for managing MCP connections: add custom servers (URL + auth or binary path), enable/disable, test connectivity, view discovered tools.

### Phase 8: Skills & Agents
19. **skills-manager** — Skills management view: browse Copilot Extensions + MCP tools as unified skill list. Toggle on/off, configure per-skill settings. Persist to SQLite.
20. **agents-manager** — Agent management view: create/edit/delete custom agent personas. Each agent has name, avatar, system prompt, assigned skills, MCP connections. Default agent is built-in and undeletable.
21. **agent-selector** — Agent picker in chat input area. Conversations tied to an agent. Agent config maps to API request structure (see Skills & Agents Concepts section).

### Phase 9: Projects & File Context
22. **projects** — Named project containers with custom instructions, pinned files (stored as BLOBs in SQLite), grouped conversations. Project selector in sidebar.
23. **file-context** — User-initiated only: read file contents into memory via drag-and-drop or file picker. Preview in input. Never retain paths or re-read from disk. No filesystem browsing.

### Phase 10: Polish & UX
24. **theme-system** — Light/dark with system detection + manual override
25. **settings-panel** — Account, theme, font size, default model, auto-update preferences, MCP management, conversation export (JSON + Markdown), clear history
26. **keyboard-shortcuts** — Cmd+N (new chat), Cmd+K (search conversations), Cmd+, (settings), Cmd+Shift+S (toggle sidebar), Escape (cancel streaming)
27. **global-hotkey** — System-wide app summon (Cmd+Shift+Space or configurable)

### Phase 11: Auto-Update
28. **auto-update** — Check for new versions on startup and periodically (configurable interval). Compare current version against latest GitHub Release. Show non-intrusive notification when update is available. Download and apply update with user confirmation. Atomic binary swap via `self_update` crate. Verify release signatures. Show changelog/release notes. Allow "skip this version" and "remind me later". Settings toggle to disable auto-update.

### Phase 12: Distribution
29. **app-packaging** — `.app` (macOS with code signing + App Sandbox), `.AppImage`/`.deb` (Linux), `.msi` (Windows), GitHub Actions CI/CD. Publish releases to GitHub Releases for auto-update consumption.

---

## Build & Run

```bash
# Prerequisites: Rust toolchain, Xcode CLI tools (macOS), Vulkan SDK (Linux)

# Build all crates
cargo build

# Run the app
cargo run -p copilot-desktop

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check

# Audit dependencies for vulnerabilities
cargo audit

# Update dependencies to latest compatible versions
cargo update
```

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| GPUI pre-1.0 breaking changes | Build failures on upgrade | Pin version, follow Zed repo, budget time for migration |
| Windows GPUI rendering issues | Poor UX on Windows | Test early, file upstream issues, document known limitations |
| Copilot API not officially public | API could change or break | Modular client design — easy to swap to official SDK when available |
| Short-lived OAuth tokens | Auth interruptions | Auto-refresh logic + graceful re-auth prompts |
| SSE streaming layout thrashing | Janky UI during responses | Batch UI updates, debounce reflows, test with fast streams |
| MCP server reliability | Tool calls may fail or timeout | Timeout handling, retry logic, graceful fallback in chat |
| MCP security surface | Untrusted servers could return harmful data | User must explicitly add servers; validate/sanitize all MCP responses |
| Web search API costs/limits | Rate limiting or billing | Cache results, respect rate limits, show clear errors |
| Auto-update failure | Corrupted binary, failed download | Atomic swap (old binary preserved until verified), checksum validation, user can re-download manually |
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
