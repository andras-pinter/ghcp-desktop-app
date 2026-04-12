# Chuck — Implementation Plan & Reference

> Part of the [Chuck project documentation](../AGENTS.md). Covers the implementation plan,
> build instructions, risk mitigations, and design reference.

---

## Implementation Plan

> **Legend:** ✅ = complete, 🔧 = in progress, ⬚ = not started

### Phase 1: Project Scaffolding & Hello World ✅

1. ✅ **project-setup** — Initialize Tauri v2 + Svelte 5 + TypeScript project via `create-tauri-app`. Configure Rust workspace with `src-tauri/` + 3 library crates (`copilot-api`, `mcp-client`, `web-research`). Set up ESLint, Prettier, Vitest. Configure `tauri.conf.json` with minimal capabilities.
2. ✅ **hello-world** — Basic Tauri window with Svelte sidebar + main area layout. Verify hot-reload works (`cargo tauri dev`). Light/dark theme via CSS custom properties.
3. ✅ **design-system** — "Warm Ink" design system applied: Instrument Serif + Plus Jakarta Sans typography, warm paper/ink palette with copper accent, grain texture overlay, entrance animations, editorial welcome screen, refined auth screen with GitHub SVG mark, copper focus glow inputs, hover micro-interactions throughout. All components use CSS custom property tokens.
4. ✅ **ui-polish** — Chat message layout: user messages right-aligned bubbles, assistant messages left-aligned (no avatar). Custom macOS title bar with `titleBarStyle: Overlay`, centered "Chuck" title, sidebar toggle button. Sidebar icon rail: collapses from 260px to 52px with icon-only view (compose, search, settings), jank-free animation via fixed-width inner wrapper + overflow clipping. Labels fade with staggered opacity. Dark mode lightened to warm charcoal palette. Random Chuck Yeager-themed welcome greetings. Window drag region with `core:window:allow-start-dragging` + `acceptFirstMouse`.

### Phase 2: Copilot API Client ✅

5. ✅ **oauth-device-flow** — GitHub OAuth device flow with token refresh in `copilot-api` crate
6. ✅ **keychain-storage** — OS keychain token storage (per-platform, using `keyring` crate)
7. ✅ **chat-completions-client** — `/v1/chat/completions` with SSE streaming + file context in `copilot-api` crate
8. ✅ **model-discovery** — Query API for available models at startup, cache list, fallback to default

### Phase 3: Persistence & Data Layer ✅

9. ✅ **sqlite-setup** — SQLite database with full schema (12 tables), migrations support, 500+ line `queries.rs` with complete CRUD, Tauri app data directory. Unit tests for all query functions.
10. ✅ **conversation-persistence** — CRUD for conversations + messages via Tauri commands. Reactive Svelte store with `initConversations()`, `switchConversation()`, `newConversation()`, `renameConversation()`, `toggleFavourite()`, `removeConversation()`. Auto-generate titles from first user message.
11. ✅ **draft-auto-save** — 3-second debounced auto-save to `drafts` table. Restore on conversation switch and app launch. Clear on successful send. Backend commands + frontend store functions.

### Phase 4: Core Chat UI ✅

12. ✅ **sidebar** — `Sidebar.svelte` (444 lines): conversation list grouped by date, new chat, favourites with star icon, context menu (rename, favourite toggle, delete), inline rename editing, real data binding via conversation store. _(Search button exists but handler not yet wired.)_
13. ✅ **chat-view** — `ChatView.svelte`: message list with streaming, welcome screen with random greetings, draft loading/saving, auto-title generation, persisted default model selection, edit/regenerate handlers, Cmd+F search overlay integration, global keyboard shortcut handler. Floating input area with gradient fade overlay.
14. ✅ **input-area** — `InputArea.svelte` (1041 lines): multi-line textarea with auto-height, custom popover model dropdown (replaces native `<select>`) with fade animation, shift+click to set default model (persisted to SQLite via settings), default model marked with copper star (★), agent dropdown selector, send/stop buttons, Enter-to-send, loading spinner while models are fetched. _(File drop zone and attachment pills not yet implemented.)_
15. ✅ **streaming-display** — Token-by-token rendering via Tauri events (`streaming-token`, `streaming-complete`, `streaming-error`), pulsing copper orb indicator with random aviation catchphrases, stop button. Event-driven architecture with proper cleanup on unmount. Messages saved on stream complete.
16. ✅ **message-actions** — Hover action buttons on messages: ✏️ edit user messages (discards subsequent messages, loads content back to input), ⟳ regenerate last assistant response (deletes + re-sends), 📋 copy message content (with 2s check animation). Actions appear on hover with smooth opacity transition. User actions positioned left of bubble; assistant actions below content.
17. ✅ **in-conversation-search** — `SearchOverlay.svelte`: Cmd+F / Ctrl+F opens floating search bar. Real-time text highlighting via DOM TreeWalker, match count display, ↑/↓ arrow navigation with active match scrollIntoView, Enter/Shift+Enter to navigate, Escape to dismiss. Highlights use `.search-highlight` / `.search-highlight-active` classes with copper accent.

### Phase 5: Markdown & Code Rendering ✅

18. ✅ **markdown-rendering** — `src/lib/utils/markdown.ts`: `marked` v15 with custom renderer (code block placeholders with `data-code`/`data-lang` attributes) piped through `DOMPurify` v3 with strict allowed tags/attributes config. `renderMarkdown()` exported. Inline code gets `.md-inline-code` class. Links open in new tab. Images blocked (rendered as links).
19. ✅ **code-blocks** — `CodeBlock.svelte` + `src/lib/utils/syntax.ts`: Shiki v3 lazy-loaded singleton highlighter with dual themes (github-light/github-dark), 28 pre-loaded languages + dynamic loading fallback. Copy-to-clipboard button with check animation. Language label header. Warm Ink styled with proper dark mode support via CSS custom properties (`--shiki-light`/`--shiki-dark`). `ThinkingSection.svelte`: collapsible `<details>` for model reasoning tokens, dashed border, muted styling, animated dots during streaming.

### Phase 6: Web Research ✅

20. ✅ **web-search** — `web-research` crate: Bing Web Search API integration. Tauri command `web_search`. `WebResultCard.svelte` for displaying results as cited cards. API key stored in keychain.
21. ✅ **url-fetcher** — Tauri command `fetch_url`. HTTPS only, public IPs only. Extract readable text via `dom_smoothie`. URL preview card in input area. Max 50KB extracted text.

### Phase 7: MCP Integration ✅

22. ✅ **mcp-client** — `mcp-client` crate: MCP protocol client (spec 2025-03-26) via `rmcp` SDK v1.3. Connect, discover tools, invoke, handle responses. HTTP and stdio transports. SSRF protection with DNS rebinding defense, comprehensive IPv4/IPv6 private range blocking, binary path validation, 1MB payload size limits, image MIME validation, 30-second connection timeouts, server-side input validation (`validate_config`), auth header redaction in IPC responses.
23. ✅ **mcp-registry** — Browse servers from official MCP Registry (`registry.modelcontextprotocol.io`). Server-side search via `?search=` API parameter, cursor-based pagination (20 per page with infinite scroll), first-party server prioritization heuristic. Multi-package registry types (npm/pypi/nuget). One-click add with auto-filled `npx -y`/`uvx`/`dotnet tool run` commands including `packageArguments`. Registry detail view with setup guides and connection options. Auto-connect on server add.
24. ✅ **mcp-settings** — `McpSettings.svelte` + `McpServerForm.svelte`: manage MCP connections. Add custom servers (URL + auth or binary path), enable/disable, test connectivity, browse discovered tools. Server cards with live 🟢/🔴 status indicators. Confirmation dialog on removal. ARIA attributes throughout.

### Phase 8: Skills & Agents ✅

25. ✅ **schema-migration-v2** — DB migration v2: add `instructions`, `source_url`, `source_type` columns to `skills` and `agents` tables. Update Rust structs and TypeScript types.
26. ✅ **skillmd-parser** — `src-tauri/src/skillmd.rs`: parse SKILL.md files (YAML frontmatter + markdown body). Extract `name`, `description`, `license`, `metadata`. Return `ParsedSkillMd` struct. Unit tests.
27. ✅ **agent-skill-queries** — Implement full CRUD queries in `queries.rs` for agents (list, get, create, update, delete, get/set skills, get/set MCP connections) and skills (list, get, create, update, delete, toggle).
28. ✅ **agent-skill-commands** — Tauri commands in `agents.rs` and `skills.rs`: full CRUD operations, register in `lib.rs`.
29. ✅ **registry-client** — `src-tauri/src/registry.rs`: pluggable `RegistryProvider` trait backed by aitmpl.com catalog. Tauri commands: `search_registry`, `install_from_registry`. Sorted by download count. Content passthrough for expand/install.
30. ✅ **git-import** — Git URL skill/agent import: accept `owner/repo`, GitHub URLs, direct file paths. Discover SKILL.md and \*.agent.md files via GitHub tree API (authenticated). Progress bar via `git-import-progress` events. Tauri commands: `fetch_git_skills`, `fetch_git_agents`, `import_git_skill`, `import_agent_from_git`.
31. ✅ **chat-agent-integration** — Modify `send_message()` to accept `agent_id`. Fetch agent + enabled skills from DB, build system prompt with skill instructions, inject as system message.
32. ✅ **skills-agents-frontend** — Frontend command wrappers + Svelte stores (`agents.svelte.ts`, `skills.svelte.ts`). Agent/skill CRUD, registry search, git import wrappers.
33. ✅ **skills-panel** — `SkillsPanel.svelte`: skill list grouped by source, toggle on/off, aitmpl.com registry browser with expand/install, git URL import with progress bar, create custom skill form. Warm Ink styling.
34. ✅ **agents-panel** — `AgentsPanel.svelte`: agent list with CRUD, skill/MCP assignment, registry browser for agent templates, git URL import. Warm Ink styling.
35. ✅ **agent-selector** — Agent picker in `InputArea.svelte` next to model selector. Conversations tied to agents. Mid-conversation change warning.
36. ✅ **sidebar-skills-agents** — Add Skills (⚡) and Agents (🤖) nav buttons to Sidebar bottom section.

### Phase 9: Projects & File Context ✅

37. ✅ **projects** — `ProjectView.svelte`: named project containers with custom instructions, pinned files (stored as BLOBs in SQLite), grouped conversations. Project selector in sidebar.
38. ✅ **file-context** — User-initiated only: drag-and-drop (Tauri native `onDragDropEvent`) or `tauri-plugin-dialog` file picker. Instant placeholder pills on drop, async background text extraction via `text_extract.rs` (PDF via `pdf-extract`+`lopdf` fallback, DOCX/XLSX/PPTX via XML extraction, RTF, 60+ text extensions). Extraction cache (`SvelteMap<string, Promise>`) + reactive status record drives pill UI (reading→extracting→✓/⚠). Extracted content sent to API only (never visible in chat). 50MB file size limit with user warning. Never retain paths or re-read from disk.
39. ✅ **context-window** — Implement conversation summarization for long chats. Older messages summarized into condensed recap. Visual indicator when summarization has occurred.

### Phase 10: Polish & Platform Features ✅

40. ✅ **settings-panel** — `SettingsPanel.svelte`: account, theme, font size, default model, keyboard shortcuts, MCP management, conversation export (JSON + Markdown), database size display + cleanup, clear history
41. ✅ **global-hotkey** — System-wide app summon via `tauri-plugin-global-shortcut` (Cmd+Shift+Space or configurable)
42. ✅ **system-tray** — Tauri core `tray-icon` feature: minimize to tray instead of closing. Streaming continues when window is hidden. Right-click menu: New Chat, Show, Quit. Status indicator.
43. ✅ **keyboard-shortcuts** — Cmd+N (new chat), Cmd+K (search conversations), Cmd+F (search in conversation), Cmd+, (settings), Cmd+Shift+S (toggle sidebar), Escape (cancel streaming). Send shortcut configuration (Enter vs Cmd+Enter / Ctrl+Enter) persisted via `send_shortcut` config key.
44. ✅ **offline-mode** — Detect network status. Full read access when offline, sending disabled with clear indicator. Auto-reconnect with "Back online" toast.
45. ✅ **accessibility** — Semantic HTML, ARIA roles/labels, keyboard navigation, focus management, visible focus indicators, screen reader testing

### Phase 11: Auto-Update ✅

46. ✅ **auto-update** — Configure `tauri-plugin-updater` with GitHub Releases endpoint (placeholder pubkey for Phase 12). `UpdateBanner.svelte` with full lifecycle: check → available (with changelog) → downloading (progress bar) → ready (restart). Skip version (persisted), remind later (24h snooze), dismiss. Auto-Update settings in SettingsPanel (toggle, frequency, skip management). `tauri-plugin-process` for app relaunch. Ed25519 signature verification (keys generated at build time in Phase 12).

### Improvements (post-Phase 11)

47. ✅ **versioning-strategy** — Workspace version inheritance: single `version` in root `Cargo.toml` `[workspace.package]`, all crates use `version.workspace = true`. Rust `xtask` crate with 4 commands: `bump` (major/minor/patch across Cargo.toml + package.json + tauri.conf.json), `check-version` (verify all files in sync), `changelog` (generate from conventional commits), `release` (auto-detect bump level from commit history + bump + changelog + commit + tag).
48. ✅ **window-state-persistence** — Save/restore window position, size, and maximized state via `tauri-plugin-store` (`window-state.json`). Save on close-to-tray and quit. Restore on launch with monitor validation (`is_position_visible()` checks all connected monitors with 200px margin). Size sanity bounds (400–10000). Safe integer conversions via `try_from()`. Entirely Rust-side — no new IPC surface.

### Phase 12: Distribution

49. ⬚ **app-packaging** — `cargo tauri build` for all platforms. `.dmg` (macOS with code signing + App Sandbox + notarization), `.AppImage`/`.deb` (Linux), `.msi`/`.nsis` (Windows). GitHub Actions CI/CD for automated builds. Publish releases to GitHub Releases for auto-update consumption.

### Phase 13: Git Sources & Unified Catalog ✅

50. ✅ **git-sources-backend** — Persistent git repository sources: DB migration v4 (`git_sources` table, FK columns on skills/agents with backfill), v5 (`git_source_items` table for cached catalog data). 11 Tauri commands for CRUD, sync, catalog search, and install. Parallel auto-sync on launch via `tokio::task::JoinSet`.
51. ✅ **git-sources-frontend** — `SourcesPanel.svelte` with source cards (add/toggle/sync/rename/delete), per-source sync progress indicators, and event-driven timestamp refresh. Built-in aitmpl.com card (toggleable, undeletable). TypeScript types, command wrappers, and reactive Svelte store.
52. ✅ **unified-catalog** — `search_catalog` merges aitmpl.com API + git source items server-side. Multi-select source filtering via pill buttons in Skills and Agents panels. Store-persisted filter selection survives panel navigation. Content omitted from search results (loaded on-demand at install).
53. ✅ **catalog-ux** — Collapsible catalog sections. Source-differentiated badges (🌐 copper for aitmpl, 🔀 neutral for git). Badge-styled metadata on source cards (item count, sync status with contextual coloring). Sidebar sync spinner.
54. ✅ **catalog-infinite-scroll** — Paginated catalog loading with 30-item batches. `search_catalog` accepts `offset`/`limit`, applies to merged aitmpl + git results sorted alphabetically. IntersectionObserver-based infinite scroll in SkillsPanel and AgentsPanel. Store-level `loadMore()` with `hasMore` tracking. Prefetch next page on 10-item threshold. Browse cache for instant panel restoration. Error-resilient pagination (stops on fetch failure).

### Improvements (post-Phase 13)

55. ✅ **floating-chat-input** — Chat input area floats over messages using absolute positioning with gradient fade. `chat-input-float` wrapper (z-index 2, pointer-events none) with `chat-input-container` (pointer-events auto). Messages get bottom padding clearance. Gradient from transparent → `--color-bg-primary` over 3rem provides smooth visual transition. Works correctly in both light and dark themes.
56. ✅ **default-agents** — DB migration v6: upgraded Default agent system prompt (smart general assistant), added Research agent (id='research', web search focused). `default_agent_id` config key seeded. Settings store `defaultAgentId` with full init/update/getter. Default Agent dropdown in Settings Panel. Auto-select default agent on `newConversation()`. AgentsPanel shows all built-in agents with "Set as default" button and visual indicator. Both built-in agents protected from deletion/editing (`is_default=1`).

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
