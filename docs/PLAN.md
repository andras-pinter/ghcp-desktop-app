# Chuck — Implementation Plan & Reference

> Part of the [Chuck project documentation](../AGENTS.md). Covers the feature inventory,
> build instructions, risk mitigations, and design reference.

---

## Implemented Features

All features are complete and shipped. Grouped by domain:

### Copilot API & Auth

- **OAuth device flow** — GitHub OAuth with token refresh, OS keychain storage (per-platform via `keyring` crate)
- **Chat completions** — `/v1/chat/completions` with SSE streaming, file context injection, agent system prompts
- **Model discovery** — Query available models at startup, cache list, deduplicate, fallback to default
- **Context management** — Automatic conversation summarization for long chats, visual indicator when summarization occurs

### Persistence & Data

- **SQLite database** — 12 tables, 11 indexes, forward-only migrations (v1→v6), app data directory
- **Conversation CRUD** — Full lifecycle with reactive Svelte store, auto-generated titles, favourites, date grouping
- **Draft auto-save** — 3-second debounced save/restore across conversation switches and app launches
- **Versioning** — `cargo xtask` CLI: `bump`, `check-version`, `changelog`, `release` — lockstep across Cargo.toml + package.json + tauri.conf.json

### Chat UI

- **Streaming display** — Token-by-token rendering via Tauri events, pulsing indicator with aviation catchphrases, stop button
- **Markdown rendering** — `marked` v15 + DOMPurify sanitization, custom code block placeholders, safe link handling
- **Code blocks** — Shiki v3 with dual themes, 30 pre-loaded languages, copy button, language labels
- **Thinking/reasoning** — Collapsible `<details>` section for model reasoning tokens
- **Message actions** — Edit (discard subsequent + reload), regenerate, copy — with hover reveal
- **In-conversation search** — Cmd+F overlay with real-time highlighting, match navigation, TreeWalker-based
- **Scroll-to-bottom** — Floating ↓ button when scrolled away, auto-scroll on conversation open
- **Slash commands** — `/help`, `/delete`, `/title`, `/export`, `/web`, `/model`, `/edit`, `/regenerate` with popup autocomplete and `@`-mention agents
- **Floating input** — Absolute-positioned input area with gradient fade overlay
- **Chat width toggle** — Centered (680px) or wide (full-width) layout, toggle above input + Settings dropdown

### Sidebar & Navigation

- **Conversation list** — Date-grouped, favourites with star, context menu (rename/favourite/delete), inline rename
- **Icon rail** — Collapsible sidebar (260px → 52px) with compose, search, settings icons
- **Skills & Agents nav** — ⚡ Skills and 🤖 Agents buttons in sidebar bottom section

### Web Research

- **Web search** — Bing Web Search API integration, `WebResultCard.svelte` for cited results, API key in keychain
- **URL fetcher** — HTTPS-only, SSRF protection (private IP blocking), `dom_smoothie` text extraction, 50KB limit

### MCP Integration

- **Protocol client** — MCP spec 2025-03-26 via `rmcp` SDK, HTTP + stdio transports, SSRF protection, DNS rebinding defense, 1MB payload limits, auth header redaction
- **Official registry** — Browse `registry.modelcontextprotocol.io`, server-side search, cursor pagination, infinite scroll, one-click add with auto-filled commands
- **Server management UI** — Add/edit/test/remove servers, live status indicators, tool discovery, confirmation dialogs
- **Binary approval** — Stdio MCP binaries require explicit user approval before first launch, persisted in SQLite

### Skills & Agents

- **SKILL.md parser** — YAML frontmatter + markdown body, strict + lenient parsing modes
- **CRUD** — Full lifecycle for custom skills and agent personas via Tauri commands + SQLite
- **Registry** — aitmpl.com catalog: search, browse, install. SWR cache (1-hour TTL, lock-free background refresh)
- **Git import** — Import from GitHub repos via tree API, progress events, SKILL.md + \*.agent.md discovery
- **Default agents** — Built-in Default + Research agents (protected from deletion), configurable default
- **Agent→skill→MCP mapping** — Agents reference skills and MCP servers; system prompt constructed at send time

### Git Sources & Unified Catalog

- **Persistent sources** — Add git repo URLs, auto-sync on launch (parallel via `tokio::JoinSet`), toggle/rename/delete
- **Unified catalog** — Server-side merge of aitmpl.com + git source items, multi-select source filtering
- **Infinite scroll** — 30-item batches, IntersectionObserver, prefetch, browse cache, error-resilient pagination
- **Catalog UX** — Source-differentiated badges, collapsible sections, sidebar sync spinner

### Projects & File Context

- **Projects** — Named containers with custom instructions, pinned files (BLOB storage), grouped conversations
- **File attachments** — Drag-and-drop + native file picker, async text extraction (PDF/DOCX/XLSX/PPTX/RTF/60+ formats), 50MB limit, never retain paths
- **Extraction pipeline** — Instant placeholder pills, background extraction with status tracking (reading→extracting→✓/⚠)

### Platform Features

- **System tray** — Minimize to tray, streaming continues when hidden, right-click menu (New Chat, Show, Quit)
- **Global hotkey** — Cmd+Shift+Space (configurable) via `tauri-plugin-global-shortcut`
- **Window state** — Persist position/size/maximized state, multi-monitor validation on restore
- **Offline mode** — Full read access, sending disabled with indicator, auto-reconnect toast
- **Keyboard shortcuts** — Cmd+N, Cmd+K, Cmd+F, Cmd+,, Cmd+Shift+S, Escape, configurable send shortcut
- **Accessibility** — Semantic HTML, ARIA roles/labels, keyboard navigation, focus management, visible focus indicators

### Settings

- **Appearance** — Light/dark/system theme, font size, chat width (centered/wide)
- **Defaults** — Default model, default agent, send shortcut
- **Data** — Conversation export (JSON/Markdown), DB size display + cleanup, clear history

### Auto-Update & Distribution

- **Auto-update** — `tauri-plugin-updater` via GitHub Releases, Ed25519 signatures, skip/snooze/dismiss, progress bar
- **CI/CD** — GitHub Actions: tag-triggered release builds + PR/push checks (Rust + frontend)
- **Packaging** — macOS (.dmg + sandbox + notarization), Linux (.AppImage + .deb), Windows (.msi + .nsis)

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

# Development with forced logout (clears stored tokens)
cargo tauri dev -- -- --logout

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

# --- Version management (cargo xtask) ---

# Check all version files are in sync
cargo xtask check-version

# Bump version (patch / minor / major)
cargo xtask bump patch    # 0.1.0 → 0.1.1
cargo xtask bump minor    # 0.1.0 → 0.2.0
cargo xtask bump major    # 0.1.0 → 1.0.0

# Generate/update CHANGELOG.md from conventional commits
cargo xtask changelog

# --- Releasing ---

# Preview what a release would do (recommended first step)
cargo xtask release --dry-run

# Cut a release (auto-detects bump level from commits)
cargo xtask release

# Force a specific bump level
cargo xtask release --bump major

# After release, push (human only — agents must never push)
git push && git push --tags
```

---

## Risks & Mitigations

| Risk                                 | Impact                                                | Mitigation                                                                                                                                                       |
| ------------------------------------ | ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Copilot API not officially public    | API could change or break                             | Modular client design in `copilot-api` crate — easy to swap to official SDK when available                                                                       |
| Short-lived OAuth tokens             | Auth interruptions                                    | Auto-refresh logic + graceful re-auth prompts                                                                                                                    |
| WebView differences across platforms | Inconsistent rendering                                | Test on all 3 platforms in CI; use standard CSS; avoid browser-specific APIs                                                                                     |
| Tauri v2 plugin ecosystem gaps       | Missing functionality                                 | Check plugin availability early; fall back to custom Tauri commands if needed                                                                                    |
| SSE streaming performance in webview | Janky UI during fast responses                        | Batch DOM updates, use `requestAnimationFrame`, debounce store updates                                                                                           |
| MCP server reliability               | Tool calls may fail or timeout                        | Timeout handling, retry logic, graceful fallback in chat                                                                                                         |
| MCP security surface                 | Untrusted servers could return harmful data           | User must explicitly add servers; validate/sanitize all MCP responses; DOMPurify                                                                                 |
| Web search API costs/limits          | Rate limiting or billing                              | Cache results, respect rate limits, show clear errors                                                                                                            |
| Large conversation DB                | Slow queries, high disk usage                         | Indexed columns, lazy loading, pagination, cleanup UI, 500MB warning                                                                                             |
| Schema migration on update           | Data loss or app crash after update                   | Forward-only migrations, backup DB before migration, test migrations in CI                                                                                       |
| Skill registry API changes           | aitmpl.com API may change or go offline               | Cache last-known results, graceful fallback (show error, allow manual git import), abstract registry client behind trait                                         |
| Untrusted SKILL.md content           | Imported skills could contain misleading instructions | SKILL.md content is text only (no code execution); instructions are injected as system prompt context; user reviews before installing; source badge shows origin |
| Git URL fetch failures               | Private repos, rate limits, non-standard hosts        | GitHub API with auth token, tree-based file discovery, clear error messages, GitHub-only for now                                                                 |

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
