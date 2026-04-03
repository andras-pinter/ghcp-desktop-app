# Copilot Desktop — Agent Instructions

> A native, cross-platform desktop chat GUI for GitHub Copilot, built with **Rust + GPUI**.
> Inspired by Claude Desktop's clean chat experience — without code editing, computer use, or agent features.

---

## Project Overview

Copilot Desktop is a standalone desktop application that provides a conversational chat interface
for GitHub Copilot. Think of it as "Claude Desktop, but for Copilot" — a polished, GPU-accelerated
native app with conversation management, file attachments, projects, and streaming responses.

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
- **Model selector** — pick from available Copilot models (if API supports)
- **Light/dark theme** — follow system preference, manual toggle
- **Global hotkey** — summon the app from anywhere (e.g., Cmd+Shift+Space)
- **Keyboard shortcuts** — standard app navigation
- **Conversation persistence** — local SQLite storage
- **Secure auth** — OAuth device flow + OS keychain token storage

### ⛔ Hard Requirement: No Filesystem / Machine Access

**This app must NEVER access the user's machine beyond what the user explicitly provides.**

- The app has **zero access** to the filesystem — it cannot read, write, browse, or scan any files or directories on its own
- The **only** way files enter the app is through explicit user action: drag-and-drop or file picker dialog
- File contents are read **once** into memory at the moment the user attaches them — the app does not retain file paths or re-read from disk
- The app stores **only** its own data: conversations (SQLite in app data dir), auth tokens (OS keychain), and user preferences (app config dir)
- No shell execution, no subprocess spawning, no system command access
- No screen capture, no clipboard snooping, no background scanning
- No network requests except to: GitHub Copilot API, GitHub OAuth endpoints, **user-configured MCP servers**, **web search API**, and **user-provided URLs**
- All outbound network destinations beyond GitHub must be **explicitly configured or initiated by the user**
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
│  │                  App State (Model)                        │   │
│  │  conversations[] │ agents[] │ skills[] │ mcp_connections[] │   │
│  │  active_project  │ config   │ auth_state                  │   │
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
│   │   │   │   └── skills.rs   # Skills/extensions browser: enable/disable/configure
│   │   │   ├── state/
│   │   │   │   ├── mod.rs      # App state model
│   │   │   │   ├── conversation.rs  # Conversation + message models
│   │   │   │   ├── project.rs  # Project model (name, instructions, attached files)
│   │   │   │   ├── agent.rs    # Agent model (name, system prompt, assigned skills, MCP connections)
│   │   │   │   ├── skill.rs    # Skill/extension model (id, name, enabled, config)
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
│   │   │   └── keychain.rs     # Secure token storage (per-platform)
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

- **Never log or display OAuth tokens** in any output
- Tokens must be stored only in the OS keychain — never in plain text files or config
- Validate all API responses — don't trust server data shapes blindly
- **No filesystem access** — the app cannot read, write, or browse files on its own. Files only enter via explicit user drag-and-drop or file picker. File contents are read into memory once; the app never stores or re-accesses file paths.
- **No shell/subprocess execution** — the app must never spawn processes or run commands
- **No network requests** except to: GitHub Copilot API, GitHub OAuth, user-configured MCP servers, web search API, and user-provided URLs. All non-GitHub network destinations must be explicitly user-configured or user-initiated.
- **MCP server connections** are user-managed — the app never auto-discovers or connects to MCP servers without explicit user configuration
- **macOS App Sandbox required** — enforce filesystem and network restrictions at the OS level via entitlements
- Treat any code path that touches the filesystem (outside app data dir) or spawns a process as a **security violation**

---

## Key Dependencies

| Crate | Purpose |
|---|---|
| `gpui` | UI framework (pin to a specific release tag) |
| `reqwest` | HTTP client (enable `stream` feature for SSE) |
| `serde` / `serde_json` | JSON serialization |
| `tokio` | Async runtime |
| `eventsource-stream` | SSE parsing for streaming responses |
| `security-framework` | macOS Keychain access |
| `secret-service` | Linux keychain access |
| `syntect` | Syntax highlighting for code blocks |
| `pulldown-cmark` | Markdown parsing |
| `rusqlite` | Local conversation persistence |
| `image` / `pdf-extract` | Extract text from PDFs/images for file context |
| `directories` | XDG / platform-appropriate config paths |
| `global-hotkeys` | System-wide keyboard shortcut registration |
| `thiserror` / `anyhow` | Error handling |
| `log` / `env_logger` | Logging |
| `scraper` / `readability` | HTML-to-text extraction for URL fetching |
| `url` | URL parsing and validation |

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

## Implementation Phases

### Phase 1: Project Scaffolding & GPUI Hello World
1. **project-setup** — Initialize Rust workspace, configure 5 crates, pin GPUI
2. **gpui-hello-world** — Basic GPUI window with sidebar + main area layout

### Phase 2: Copilot API Client
3. **oauth-device-flow** — GitHub OAuth device flow with token refresh
4. **keychain-storage** — OS keychain token storage (per-platform)
5. **chat-completions-client** — `/v1/chat/completions` with SSE streaming + file context

### Phase 3: Core Chat UI (Claude Desktop-style)
6. **sidebar** — Conversation list grouped by date, new chat, projects, agents, search, collapsible
7. **chat-view** — Message list with avatars, timestamps, thinking indicator
8. **input-area** — Multi-line input, file drop zone, attachment pills, agent selector, loading state
9. **streaming-display** — Token-by-token rendering with cursor animation, stop button

### Phase 4: Markdown & Code Rendering
10. **markdown-parser** — Bold, italic, headers, lists, links, code, blockquotes, tables
11. **code-blocks** — Syntax-highlighted fenced blocks with copy button + language label

### Phase 5: Web Research
12. **web-search** — Integrate a web search API (e.g., Bing Search API). AI can trigger searches; results included in context. Display search results as cited cards in chat.
13. **url-fetcher** — User pastes a URL → app fetches page, extracts readable text content, includes in conversation context. Show URL preview card in input area.

### Phase 6: MCP Integration
14. **mcp-client** — Implement MCP protocol client: connect to MCP servers, discover tools, invoke tools, handle responses. Support stdio and HTTP transports.
15. **mcp-catalog** — Built-in catalog of popular MCP servers (GitHub, web search, databases, etc.) with one-click enable. Show descriptions, required config fields.
16. **mcp-settings** — UI for managing MCP connections: add custom servers (URL + auth), enable/disable catalog servers, test connectivity, view available tools.

### Phase 7: Skills & Agents
17. **skills-manager** — Skills/extensions management view: browse available Copilot Extensions, toggle on/off, configure per-extension settings. Skills are tools/capabilities the AI can use.
18. **agents-manager** — Agent management view: create/edit/delete custom agent personas. Each agent has a name, avatar, system prompt, assigned skills, and MCP connections.
19. **agent-selector** — Agent picker in the chat input area. Conversations are tied to an agent. Default agent uses base Copilot; custom agents add their system prompt + skills + MCP tools.

### Phase 8: Projects & Persistence
20. **conversation-persistence** — SQLite storage, load on startup, lazy-load older
21. **projects** — Named project containers with instructions, pinned files, grouped conversations
22. **file-context** — User-initiated only: read file contents into memory via drag-and-drop or file picker. Preview in input. Never retain paths or re-read from disk. No filesystem browsing.

### Phase 9: Polish & UX
23. **theme-system** — Light/dark with system detection + manual override
24. **settings-panel** — Account, theme, font size, model, MCP, export, clear history
25. **keyboard-shortcuts** — Cmd+N, Cmd+K, Cmd+,, Cmd+Shift+S, Escape
26. **global-hotkey** — System-wide app summon (Cmd+Shift+Space or configurable)

### Phase 10: Distribution
27. **app-packaging** — `.app` (macOS signed), `.AppImage`/`.deb` (Linux), `.msi` (Windows), GitHub Actions CI/CD

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

---

## Design Reference

The UX is modeled after **Claude Desktop** (Anthropic's desktop app):
- Clean, minimal sidebar with conversation history
- Central chat panel with streaming markdown responses
- File attachment via drag-and-drop with visual pills
- Project-based organization with custom instructions
- Light/dark theme with system preference detection
- Global hotkey to summon from anywhere

**Key difference:** This app has **no access to the user's machine** — no filesystem browsing, no shell execution, no screen capture. All external capabilities come through explicit user actions (file attach, URL paste) or user-configured connections (MCP servers, web search). It's a powerful but sandboxed chat client for GitHub Copilot with extensibility via MCP and custom agents.
