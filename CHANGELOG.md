# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.12.2] — 2026-04-19

### Bug Fixes

- **deps:** regenerate corrupted pnpm-lock.yaml (9d5cf7d)

## [0.12.1] — 2026-04-19

### Bug Fixes

- **ci:** use RELEASE_TOKEN in release workflow for GitHub Releases creation (9934571)
- **ci:** add full check suite to prepare-release workflow (ec9e894)
- **ci:** format tauri.conf.json and add prettier step to prepare-release (18a2a4f)

## [0.12.0] — 2026-04-19

### Features

- **ui:** implement sidebar conversation search (6fe1f17)

### Chores

- bump lock files (952fd2e)
- **deps:** update @tauri-apps/plugin-dialog to 2.7.0 (416976d)

### Style

- fix prettier formatting in tauri.conf.json (d8879a1)

## [0.11.3] — 2026-04-19

### Bug Fixes

- resolve CI clippy and svelte-check failures (e03fd29)
- **tauri:** disable macOS App Sandbox for auto-updater compatibility (684d213)

### Documentation

- update branch refs and release instructions for new workflow (70878e1)

### Chores

- remove post-merge githook (4b7b468)

### CI/CD

- add prepare-release workflow and fix branch refs (e67e975)

### Style

- fix prettier formatting in RELEASE.md and tauri.conf.json (327073c)

## [0.11.2] — 2026-04-19

### Bug Fixes

- **deps:** sync tauri-plugin-dialog Rust crate to 2.7.0 (7e17d4a)
- redact token Debug output, fix docs, update deps (0014c87)
- **ci:** remove notarization env vars to skip notarization (ab841e2)
- **ci:** force bash shell for changelog extraction step (b6128f1)
- **ci:** set up keychain manually for self-signed codesign (5cb8ed7)

### Chores

- release v0.11.1 (3b38e90)
- **deps:** bump globals from 16.5.0 to 17.5.0 (069c5aa)
- **deps:** bump @sveltejs/vite-plugin-svelte from 5.1.1 to 6.2.4 (0bf66c4)
- **deps:** bump shiki from 3.23.0 to 4.0.2 (65d02d9)
- **deps:** bump @tauri-apps/plugin-updater from 2.10.0 to 2.10.1 (#24) (424366d)
- **deps:** bump typescript from 5.9.3 to 6.0.3 (a88ce4c)
- **ci:** bump the actions-all group with 5 updates (0bc23e9)
- **deps:** bump the npm-minor-patch group with 6 updates (#15) (b8047ea)
- **ci:** group Dependabot PRs and auto-merge minor/patch (8703d3b)

## [0.11.1] — 2026-04-19

### Bug Fixes

- redact token Debug output, fix docs, update deps (0014c87)
- **ci:** remove notarization env vars to skip notarization (ab841e2)
- **ci:** force bash shell for changelog extraction step (b6128f1)
- **ci:** set up keychain manually for self-signed codesign (5cb8ed7)

### Chores

- **deps:** bump globals from 16.5.0 to 17.5.0 (069c5aa)
- **deps:** bump @sveltejs/vite-plugin-svelte from 5.1.1 to 6.2.4 (0bf66c4)
- **deps:** bump shiki from 3.23.0 to 4.0.2 (65d02d9)
- **deps:** bump @tauri-apps/plugin-updater from 2.10.0 to 2.10.1 (#24) (424366d)
- **deps:** bump typescript from 5.9.3 to 6.0.3 (a88ce4c)
- **ci:** bump the actions-all group with 5 updates (0bc23e9)
- **deps:** bump the npm-minor-patch group with 6 updates (#15) (b8047ea)
- **ci:** group Dependabot PRs and auto-merge minor/patch (8703d3b)

## [0.11.0] — 2026-04-19

### Features

- **ci:** include changelog in GitHub Release body (a972eaa)

### Bug Fixes

- **xtask:** treat dependency updates as patch releases (805021a)
- **xtask:** allow lock file changes in release clean check (9fd93b7)
- **ci:** add self-signed code signing for macOS auto-updater (1521b94)

### Chores

- **ci:** add Dependabot configuration (b100ffb)

## [0.10.1] — 2026-04-18

### Bug Fixes

- **xtask:** skip version bump for non-bumping commit types (feedb04)
- **ci:** publish releases immediately instead of as drafts (645361a)
- **docs:** correct table count (14), language count (28), CSS vars (160+) (f8b579f)
- **docs:** correct index count, language count, remove duplicate section (bd2e0be)

### Documentation

- consolidate implementation phases into feature inventory (752490d)

### Chores

- gitignore SQLite database artifacts (ebad3d3)

## [0.10.0] — 2026-04-18

### Features

- **ui:** add chat width toggle (centered/wide) (44ade79)

### Bug Fixes

- **ui:** improve width toggle icon and styling (10f09ca)

### Refactoring

- **ui:** move width toggle above input area (1ecf18c)

## [0.9.2] — 2026-04-18

### Bug Fixes

- **tauri:** add explicit default features, update entitlement docs (4854622)
- **tauri:** grant read-write entitlement for file export (43a14a7)
- **tauri:** disable devtools in production builds (ec4b04d)
- **ci:** add contents:write permission to build jobs (e0775db)
- **ci:** resolve Linux dep conflict, Windows build, and clean up workflows (659a0f5)
- **ci:** support unsigned macOS builds and upgrade to ubuntu-24.04 (bcabc8c)

### Chores

- **tauri:** regenerate updater signing keypair (dcf5026)

## [0.9.1] — 2026-04-18

### Bug Fixes

- **ci:** correct actions/setup-node commit SHA pin (b91d7a0)

### Chores

- bump dependency lock (dfec876)

## [0.9.0] — 2026-04-18

### Features

- **ci:** add cross-platform CI/CD and release pipeline (e564ca2)

### Bug Fixes

- **ci:** address security and correctness review findings (c1541fe)
- **ci:** make Apple signing optional in release workflow (3cf0c88)

## [0.8.1] — 2026-04-13

### Bug Fixes

- **registry:** add 30s HTTP timeout to background cache refresh (d957979)
- **tauri:** use AtomicBool for refresh flag to eliminate lock-in-Drop (5c3b6db)
- **tauri:** harden registry cache against races and panics (9d7f9e4)
- **tauri:** address review findings in registry cache (b14fc9a)

### Performance

- **tauri:** use stale-while-revalidate for registry cache (14ebc8f)
- **tauri:** increase registry cache TTL from 5 min to 1 hour (33b8571)
- **tauri:** add in-memory TTL cache for aitmpl.com registry data (a928768)

### Documentation

- update all documentation for slash commands, scroll button, and registry caching (89d1a9e)

## [0.8.0] — 2026-04-12

### Features

- **ui:** add floating scroll-to-bottom button in chat view (157cf52)

### Bug Fixes

- **ui:** scroll to bottom after messages actually load (f18bb92)
- **ui:** scroll existing chats to bottom when opened (3755269)

### Performance

- **registry:** stale-while-revalidate (SWR) cache for aitmpl.com components.json — 1-hour TTL, lock-free AtomicBool background refresh, 10 MB size limit, 30s HTTP timeout

## [0.7.0] — 2026-04-12

### Features

- **ui:** show alias hints in command popup (e.g. /fetch · /web) (08a9dcb)
- **ui:** show modal dialogs for /title and /export commands (f0c86d3)
- **ui:** add slash commands and @-mention autocomplete (4710954)

### Bug Fixes

- **ui:** address review findings — security, dead code, error handling (7d812a5)
- **ui:** show command aliases in Command Reference modal (0f4a9f8)
- **ui:** /web alias matches partial prefixes like /w and /we (6951dde)
- **ui:** wrap-around arrow navigation in command popup and dropdowns (793887e)
- **ui:** swap export arguments — content was used as filename (8d29e20)
- **ui:** fix command dispatch, help modal, and delete behavior (ffe4971)
- **ui:** constrain command popup width to input box (2b57b51)
- **ui:** auto-focus help modal overlay so Escape key works (6360fca)
- **ui:** resolve /model sub-command popup and add /help modal for new chat (260fcf9)
- **ui:** fix slash command bugs — selection, conversation context, scroll, /? alias (89b3182)

### Refactoring

- **ui:** merge /web into /fetch as alias, remove dead code (feeb797)
- **ui:** rename /clear command to /delete (1b63ff9)

## [0.6.0] — 2026-04-12

### Features

- **ui:** add default agent setting with auto-select on new conversations (4f73651)
- **tauri:** add built-in Research agent and upgrade Default agent prompt (1e68a4e)

### Bug Fixes

- **ui:** use correct DB key when resetting default agent on deletion (d25430b)
- **ui:** show loading spinner for agent picker until ready (5b0d9a1)
- **ui:** select default agent on app startup after data loads (da239d6)
- **ui:** reset agent to user default when starting a new chat (6dece8a)
- **ui:** fix agent dropdown to properly distinguish user default from built-in (54c4743)
- **ui:** reset default agent setting when agent is deleted (c4d40be)

## [0.5.0] — 2026-04-12

### Features

- **ui:** make chat input area floating over messages (be1bd3c)

### Documentation

- update PLAN.md and AGENTS.md for floating input and catalog pagination (164fd52)

## [0.4.0] — 2026-04-12

### Features

- **ui:** add infinite scroll pagination to catalog browsers (ecf17fc)

### Bug Fixes

- **ui:** prevent stale loadMore results and fix prefetch error handling (0f0fca1)
- **ui:** cap aitmpl fetch limit and stop pagination on error (8848801)
- clean up CHANGELOG and add --since flag to xtask changelog (ffddfe6)

### Refactoring

- **ui:** remove install count badges from catalog panels (39faa29)

### Chores

- change license from Unlicense to Apache 2.0 (10633e7)

## [Unreleased]

### Features

- support multi-select source filtering in catalog search (df7e660)
- **ui:** show per-source sync progress on source cards (fe18330)
- **ui:** show sync spinner on Sources sidebar button (c2257c5)
- add source filter to catalog search in Skills and Agents panels (3b3ba41)
- **ui:** show aitmpl.com as built-in source with toggle (c5268aa)
- integrate git sources into unified catalog search (79b425f)
- **ui:** add SourcesPanel with sidebar entry and routing (bb613e3)
- **ui:** add git source TypeScript types, commands, and Svelte store (d4ef02e)
- **tauri:** add git source Tauri commands (f9c674b)
- **tauri:** add git_sources table and migration v4 (0ae7e83)

### Bug Fixes

- add duplicate detection to batch import functions (8a27205)
- address review findings — DoS cap, skill dedup, UUID collision (303ac09)
- skip tables, rules, and code fences in description extraction (c5d5919)
- use per-kind caps for git source discovery and fetch concurrently (75a4461)
- resolve catalog name, content, and mount issues (8481562)
- resolve timestamp, install, description, and cache bugs (f9b2308)
- address four user-reported bugs in git sources feature (943a328)
- **tauri:** scope skill IDs by source to prevent cross-source collisions (ffce9aa)
- **ui:** use dedicated sync-complete event for timestamp refresh (f2513be)
- **ui:** refresh source timestamp immediately after sync completes (6932465)
- **ui:** persist source filter selection across panel navigation (89d89d2)
- **ui:** remove translateY from spin animation causing spinner jitter (91140bb)
- address review findings from security and code quality agents (5d1852d)
- **ui:** collapse Browse Registries catalogs by default (8fb0080)
- **ui:** suppress false-positive state_referenced_locally warning (cf6fcb9)

### Performance

- remove artificial caps on git source discovery (51d872a)
- raise per-kind discovery cap to 500 and batch size to 20 (9b75a35)
- **tauri:** sync git sources in parallel on launch (8ae737b)

### Refactoring

- **ui:** extract aitmpl card to snippet, fix filter visibility (2d944e0)
- **ui:** remove one-off git import, add auto-sync on launch (221cdcc)

### Documentation

- update AGENTS.md and DATA-MODEL.md for git sources feature (f715466)

### Style

- **ui:** remove redundant source indicator from catalog headings (afd98d3)
- **ui:** upgrade source and registry card metadata to badges (34a11ac)
- **ui:** replace source filter dropdown with pill buttons (b36c137)
- **tauri:** apply rustfmt formatting (d9379ad)

## [0.2.3] — 2026-04-10

### Bug Fixes

- **docs:** clarify update-available event source, add missing deps (d9f7f19)
- **docs:** address review findings in split documentation (5405f6d)

### Refactoring

- **docs:** split AGENTS.md into modular documentation (2444eaa)

## [0.2.2] — 2026-04-10

### Bug Fixes

- **quality:** improve error handling, logging, and overflow protection (2c48579)
- **security:** harden token handling, SSRF protection, and logging (f5b4a87)

### Documentation

- **agents:** add missing deps, fix skills schema, update project structure (0da09b4)
- fix schema docs, add JSDoc, update project structure (e3f69d7)

## [0.2.1] — 2026-04-10

### Bug Fixes

- **ui:** resolve code block layout collision with global .code-block class (c6da26a)

## [0.2.0] — 2026-04-10

### Features

- **app:** parallel streaming with sidebar status indicators (574ad8a)
- **ui:** show setup guide below MCP server form for registry entries (32e7f3a)
- **ui:** prefetch Skills and Agents registries on startup (d76e8e4)
- **ui:** redesign Settings panel with premium card-based layout (8a901ef)
- **ui:** add star/trash icons and unified ConfirmDialog across app (cfb0bc7)
- **ui:** add local send shortcut toggle in input area (4d22d30)
- **ui:** auto-continue numbered and bullet lists in input area (ffb3727)
- **ui:** replace blinking cursor with pulsing orb streaming indicator (ef172c1)
- **tauri:** persist window position and size across restarts (67b364f)
- **xtask:** add automated release command with bump auto-detection (cd455f4)
- **app:** add xtask crate for version management (57dee73)
- **app:** add auto-update via tauri-plugin-updater (e5e94e4)
- **chat:** generate conversation titles via AI instead of truncation (2784257)
- **app:** implement Phase 10 — polish & platform features (a71b82a)
- **ui:** async file extraction with background cache (4ac84a0)
- **tauri:** add generic text extraction for file attachments (f729d12)
- **app:** implement Phase 9 — projects, file attachments, context-window summarization (f9cf939)
- **ui:** add progress bar for git import in Skills and Agents panels (c285607)
- **registry:** support *.agent.md discovery in git import (7042b2f)
- **ui:** sort catalog results by download count (highest first) (1be339b)
- **ui:** add agent catalog — registry browser + git import (b396686)
- **ui:** polish Skills & Agents panels (5692bbe)
- **ui:** add Skills & Agents panels, agent selector, and sidebar nav (cce0320)
- **ui:** add agent/skill stores, registry types, and command wrappers (7ec2b72)
- **tauri:** add registry client and git import with Tauri commands (c3d6e55)
- **chat:** integrate agent system prompt injection (ab65013)
- **tauri:** implement agent and skill Tauri commands (676ec99)
- **tauri:** add agent and skill CRUD queries (235a3d3)
- **tauri:** add schema migration v2 + SKILL.md parser (ff01a88)
- **mcp:** pagination, auto-connect, bare command support (abe56d3)
- **ui:** one-click MCP server add from registry (836a1be)
- **ui:** auto-fill npx/uvx commands for stdio MCP servers (851cbf4)
- **ui:** add setup guide and bottom CTA to registry detail view (a6eb9a2)
- **ui:** add registry detail view and loading spinner (17c945d)
- **mcp-client:** prioritize first-party servers in registry results (88109a2)
- **mcp-client:** use server-side search for MCP Registry (6ecf229)
- **mcp-client:** add MCP Registry browser and extract server form (93dcaad)
- **mcp-client:** implement MCP integration with rmcp SDK (1d09bd9)
- **web-research:** implement web search and URL fetching (e88e105)
- **ui:** add message actions (edit, regenerate, copy) and in-conversation search (63af8ac)
- **ui:** add markdown rendering, syntax-highlighted code blocks, and thinking section (8de1746)
- **ui:** custom model selector dropdown with shift+click default (341084d)
- **tauri:** add --logout flag, devtools, and frontend logging (08db5c5)
- **tauri:** implement Phase 3 persistence & data layer (789a9bc)
- **ui:** auto-copy device code to clipboard on sign-in click (28fc965)
- **copilot-api:** implement OAuth device flow, SSE streaming, and model discovery (c1e09f1)
- **ui:** randomize welcome greeting on each visit (ced7ec8)
- scaffold Tauri v2 + Svelte 5 + TypeScript project (179a60d)

### Bug Fixes

- **ui:** add explicit RAF cleanup on effect teardown (dca17c3)
- **ui:** revert sticky indicator, use instant scroll during streaming (0dd3be0)
- **app:** address review findings — race conditions, cleanup, limits (2814288)
- **ui:** tune scroll follow — 100ms interval, 150px threshold (f20af3c)
- **app:** harden title generation prompt against instruction leakage (8c68d4c)
- **ui:** increase auto-scroll sticky threshold to half viewport (ebb82af)
- **app:** improve title generation with two-phase retry strategy (b4bd4e9)
- **ui:** pause auto-scroll when user scrolls up during streaming (cd9ff65)
- **app:** generate conversation title at stream start, not end (49fffae)
- **ui:** add aria-expanded to expand buttons, fix returnToList state reset, fix rel attributes (3157520)
- **ui:** add spacing between expanded card-detail and card-actions (aaf3787)
- **ui:** match card icons in More Info section (📦 Repository, 🌐 Website) (ad43610)
- **ui:** rename MCP form 'Setup Guide' to 'More Info' (00c426c)
- **ui:** use unique keys for MCP registry each block (2984d3d)
- **ui:** restyle MCP setup guide as collapsible card-detail panel (fb0eeca)
- **ui:** remove 'From MCP Registry' banner from server form (c02de83)
- **ui:** remove source-link arrows from installed skill cards (784ed9f)
- **ui:** move Test/Edit/Remove buttons to MCP server card header (fda1f8d)
- **ui:** fix MCP form navigation and remove redundant header (5065336)
- **ui:** remove source-link arrow from MCP registry card headers (5bba533)
- **ui:** link MCP registry cards to official registry page (bfab3e4)
- **ui:** move MCP card-actions to separate row and add stripMarkdown utility (18fdd84)
- **ui:** strip markdown from agent card preview text (1a35471)
- **ui:** make skill description optional in Create Skill form (3c54b3e)
- **ui:** disable MCP server buttons until required fields are filled (7f3f245)
- **ui:** require all fields before enabling Create Skill button (4edceb7)
- **ui:** disable Create Agent button when form is incomplete (51162da)
- **ui:** close agent dropdown on outside click (ab63604)
- **ui:** copper chevron color, remove settings header override, drop shortcut note (613c81b)
- **tauri:** use safe integer conversions in window state restore (303323a)
- **xtask:** address code review findings round 2 (36fc8c1)
- **xtask:** address code review findings (528ebcb)
- **app:** replace unsafe .expect() with proper error propagation and log silent catches (00bb0fc)
- **app:** clear stale skip/snooze settings on update check (31c055a)
- **ui:** render changelog as markdown in UpdateBanner (ad562ac)
- **tauri:** fix startup auto-connect, extract helpers, harden path validation (4ccc12b)
- **tauri:** harden MCP binary approval, drag-drop, auth sentinel, test by ID (5df0a8e)
- **tauri:** enforce binary approval on test_mcp_connection, safe migration v3 (24b310d)
- **tauri:** address review findings from security audit (3c0c4d1)
- **tauri:** harden security across 6 attack surfaces (2570cf7)
- **ui:** prevent race condition in title generation (a3d2e8f)
- **tauri:** simplify tray menu — remove dividers, shorten labels (4413f6c)
- **tauri:** fix duplicate tray icon and dock reopen on macOS (b5f0839)
- **app:** fix tray icon, window reopen, and auto-title generation (2d83bce)
- **ui:** address security and code review findings (10e42c7)
- **ui:** hide extracted file content from chat, send to API only (bf7ff81)
- **ui:** replace loading file pills with real data after read (83bf34a)
- **ui:** show file pills instantly on drop with loading state (d1ec3a0)
- **ui:** show chat view immediately when sending with file attachments (fa90952)
- **ui:** enlarge drop overlay and improve PDF extraction (d170f62)
- **ui:** use Tauri native drag-drop instead of HTML5 events (e840054)
- **ui:** make entire chat view a file drop zone (f574cb7)
- **ui:** fix drag-and-drop, async extraction, processing indicator (363d2b3)
- **ui:** show friendly file size warning for oversized uploads (d4a2a12)
- **projects:** raise file size limit to 50MB and show errors in UI (a12d27e)
- **ui:** navigate back to chat when selecting conversation from panels (59e5220)
- **ui:** only show 'no files found' after scan completes (cfcaf6f)
- **registry:** authenticate GitHub API calls in git import (1fa9746)
- **registry:** use tree API for recursive SKILL.md discovery in git import (72d68f8)
- **ui:** cancel draft save timer on send to prevent ghost drafts (0143e3a)
- **ui:** hide catalog results when search field is empty (bd93e5d)
- **ui:** persist installed badge after skill/agent registry install (9716fa0)
- **ui:** fix catalog YAML rendering, descriptions, install names, delete UX (38708cd)
- **ui:** pass content through registry items for catalog expand and install (5ddcdbe)
- **ui:** fix duplicate keys, source links, markdown rendering in expand views (d9cbaa0)
- **ui:** add expand buttons to catalog items, render MD, fix duplicate keys (da9c9ba)
- **ui:** fix agent links, expand buttons, registry defaults, and descriptions (a0e6f58)
- **app:** fix registry install failures and improve search quality (9bd76ea)
- **ui:** fix registry search, install 404s, expand buttons, and form layout (0a1352b)
- **ui:** polish skill cards and fix input borders (aafb903)
- **ui:** improve Skills & Agents panel UX consistency (38c19d7)
- **tauri:** address code review findings — SSRF, URL encoding, missing ID (bd5a17b)
- **mcp:** correct IPv6 localhost check (remove brackets) (651b2c4)
- **mcp:** security hardening and code quality improvements (a421571)
- **mcp:** pass packageArguments from registry to server commands (a56fd0e)
- **ui:** suppress state_referenced_locally warnings in McpServerForm (1370f88)
- **ui:** CSS spinners, copy buttons on setup commands, centered form (2a80c93)
- **ui:** address security, logic, and performance review findings (3e9170a)
- **ui:** show spinner in model selector while models load (5d90601)
- **ui:** load models and conversations after fresh login (8bf9319)
- **ui:** apply persisted default model on first load (de01b8e)
- harden debug credential permissions and log_frontend safety (b90122b)
- **tauri:** stop auto-opening devtools in debug builds (22405d3)
- **ui:** deduplicate models to prevent keyed each block crash (43d8a5d)
- **copilot-api:** use file-based credential storage in debug builds (5e62e0e)
- **ui:** improve model selector with dropdown chevron and loading state (ce76839)
- **copilot-api:** enable real keychain and fix API headers (a6bc068)
- **ui:** add drag region to auth and loading screens (51e3ad1)
- address security and code quality review findings (round 2) (a61f820)
- **ui:** fix auth polling string matching and add welcome taglines (deb4ec3)
- **copilot-api:** address security and code review findings (10d03a3)
- resolve review findings — stale references, dead CSS, docs update (dedf0d6)
- **tauri:** enable window dragging on custom title bar (5008419)
- **ui:** show sidebar toggle on welcome screen + add model selector (101aae6)
- **docs:** resolve 8 review issues across wireframes, IPC, and terminology (ed03d42)

### Performance

- **ui:** replace scroll interval with RAF loop for jank-free follow (a5d6050)
- **ui:** add typewriter character reveal for streaming messages (869ed23)
- **ui:** smooth streaming with RAF batching and render throttle (3a27df6)

### Refactoring

- **ui:** rename MCP registry add button to 'Add & Configure' (d24beeb)
- **ui:** always open form page when adding MCP server from registry (4989e53)
- **ui:** add expand/collapse pattern to MCP server cards (421d934)
- **ui:** redesign MCP registry cards with inline expand/collapse (ed57e25)
- **ui:** unify panel design across Skills, Agents, and MCP (50b44e1)
- **ui:** unify panel headers with SVG chevron + redesign settings to sections (d095e38)
- **ui:** add unified component CSS system and refactor McpServerForm (5d03776)
- **ui:** migrate McpSettings to unified global CSS classes (3c6c06d)
- **ui:** migrate ProjectView.svelte to global CSS classes (9a6c8c0)
- **ui:** replace scoped CSS with global classes in SkillsPanel (7cb91a7)
- **ui:** migrate SettingsPanel to unified global CSS classes (8b49da3)
- **ui:** migrate AgentsPanel to unified global CSS classes (f88d590)
- **registry:** remove skills.sh, extract RegistryProvider trait (e715e97)
- **mcp-client:** replace built-in catalog with paginated MCP Registry (65e732f)
- **ui:** move model fetching to app-level store (e51d4c1)
- **ui:** custom title bar with toggle + jank-free sidebar collapse (23f2d57)
- **ui:** sidebar icon rail pattern with collapsible menu items (23ad8a5)

### Documentation

- add git hooks setup step to README (2fd22bd)
- update AGENTS.md wireframes and STYLE-GUIDE.md for unified design (9837dd6)
- add STYLE-GUIDE.md with full unified design system reference (d4bffb6)
- document cargo xtask release workflow and versioning strategy (6bcf05d)
- document xtask versioning workflow and project structure (47ec8ae)
- mark Phase 11 (Auto-Update) as complete in AGENTS.md (0a3ac90)
- update AGENTS.md and chat.rs comment to reflect actual codebase (af0f15c)
- add pre-merge finalization review protocol to AGENTS.md (0dacccf)
- update AGENTS.md with Phase 9 file context details (f82e288)
- fix stale references in AGENTS.md (028837a)
- update AGENTS.md Phase 8 status, add JSDoc to types, fix UTF-8 truncation (2a1d690)
- mark Phase 8 (Skills & Agents) as complete (670c944)
- **agents-md:** update AGENTS.md for Phase 8 expanded scope (3446bc8)
- update AGENTS.md to reflect MCP Registry and current state (018554a)
- update input-area status to 🔧 in AGENTS.md (7da70f9)
- update AGENTS.md Phase 4 status for model selector (a7e8685)
- update implementation plan with Phase 1 completion status (e5429ef)
- add send shortcut toggle (Enter vs Cmd+Enter) (39cb4b2)
- add UX wireframes for all key screens (6eb746c)
- replace stale dependencies in AGENTS.md (58547c4)
- fix remaining review issues in AGENTS.md (5073c47)
- fix review issues in AGENTS.md Tauri rewrite (da02660)
- rewrite AGENTS.md for Tauri v2 + Svelte 5 + TypeScript stack (c63e1b9)
- fix 21 review findings across AGENTS.md and README (1be9a0b)
- add chat UX behaviors, favourites, system tray, and gap analysis items (a08d83c)
- strengthen review protocol — zero exceptions, explicit warnings (7d6c4c8)
- fix 13 review findings from extensive AGENTS.md review (65542af)
- add auto-update feature — self_update from GitHub Releases (109a041)
- add README with project overview, security model, and legal disclaimer (ac0a80a)
- clarify commit type must match actual change, not content (b6ddb4e)
- add git conventions — never push policy and conventional commits (baf6cf6)
- add mandatory dependency freshness policy (8bdf407)
- resolve 23 review findings in AGENTS.md (3c278c4)
- add mandatory agent review-fix loop protocol (1276778)
- add web research, MCP, skills & agents specs (dfd9211)
- add hard security requirement — no filesystem access (b5e95b5)
- move agent instructions to AGENTS.md (ad3b960)
- add initial agent instructions for Copilot Desktop (70a1736)

### Chores

- add tracked post-merge hook for auto-release (d0144d9)
- initialize CHANGELOG.md from existing commits (5f126f8)
- **ui:** remove debug logs from title generation (56b3781)
- **tauri:** replace app icon with Prismatic Launch design (9b4867a)
- **deps:** upgrade eslint to ^10 to match @eslint/js@10 (7aef46d)
- **deps:** update vite to 6.4.2 to fix security vulnerabilities (c2d452b)
- add new-branch rule to AGENTS.md (3400544)
- remove scaffold greet command, fix lint, update phase status (74474ae)
- rename app from Copilot Desktop to Chuck (ef113d8)

### Build

- **deps:** adopt workspace version inheritance for all crates (23c6b6b)

### Style

- **ui:** remove scroll runway — RAF sync makes it unnecessary (b8507e9)
- **ui:** reduce streaming indicator runway to 0.5em (b0bdfa2)
- **ui:** make streaming indicator sticky at bottom of chat area (7c9567c)
- **ui:** add scroll runway below streaming indicator to prevent flapping (afc316b)
- **ui:** add spacer above streaming indicator to prevent flapping (c9465ba)
- **ui:** add breathing room below streaming indicator (ab65d40)
- apply cargo fmt to chat commands (0ae3aa8)
- **ui:** bold unread conversation titles in sidebar (6dc0b89)
- **ui:** remove redundant testing flag assignment in McpServerForm (22a75f5)
- **ui:** unify SkillsPanel and AgentsPanel visual patterns (34ece03)
- **ui:** flip tagline hierarchy — punchline is the hero (f6999a5)
- **ui:** make tagline punchline pop with italic + accent color (3da06af)
- **ui:** update auth tagline to Chuck Yeager quip (8c47110)
- **ui:** split auth tagline into two styled lines (2a7ba8e)
- **ui:** remove assistant avatar from chat messages (1e922e9)
- **ui:** right-align user messages, fix sidebar toggle timing, lighten dark mode (107b8d3)
- **ui:** apply Warm Ink design system with distinctive typography (9c4382d)
- **ui:** redesign to minimal Claude Desktop-inspired layout (3f8c019)

## [0.1.0] — 2026-04-09

### Features

- **app:** add xtask crate for version management (57dee73)
- **app:** add auto-update via tauri-plugin-updater (e5e94e4)
- **chat:** generate conversation titles via AI instead of truncation (2784257)
- **app:** implement Phase 10 — polish & platform features (a71b82a)
- **ui:** async file extraction with background cache (4ac84a0)
- **tauri:** add generic text extraction for file attachments (f729d12)
- **app:** implement Phase 9 — projects, file attachments, context-window summarization (f9cf939)
- **ui:** add progress bar for git import in Skills and Agents panels (c285607)
- **registry:** support *.agent.md discovery in git import (7042b2f)
- **ui:** sort catalog results by download count (highest first) (1be339b)
- **ui:** add agent catalog — registry browser + git import (b396686)
- **ui:** polish Skills & Agents panels (5692bbe)
- **ui:** add Skills & Agents panels, agent selector, and sidebar nav (cce0320)
- **ui:** add agent/skill stores, registry types, and command wrappers (7ec2b72)
- **tauri:** add registry client and git import with Tauri commands (c3d6e55)
- **chat:** integrate agent system prompt injection (ab65013)
- **tauri:** implement agent and skill Tauri commands (676ec99)
- **tauri:** add agent and skill CRUD queries (235a3d3)
- **tauri:** add schema migration v2 + SKILL.md parser (ff01a88)
- **mcp:** pagination, auto-connect, bare command support (abe56d3)
- **ui:** one-click MCP server add from registry (836a1be)
- **ui:** auto-fill npx/uvx commands for stdio MCP servers (851cbf4)
- **ui:** add setup guide and bottom CTA to registry detail view (a6eb9a2)
- **ui:** add registry detail view and loading spinner (17c945d)
- **mcp-client:** prioritize first-party servers in registry results (88109a2)
- **mcp-client:** use server-side search for MCP Registry (6ecf229)
- **mcp-client:** add MCP Registry browser and extract server form (93dcaad)
- **mcp-client:** implement MCP integration with rmcp SDK (1d09bd9)
- **web-research:** implement web search and URL fetching (e88e105)
- **ui:** add message actions (edit, regenerate, copy) and in-conversation search (63af8ac)
- **ui:** add markdown rendering, syntax-highlighted code blocks, and thinking section (8de1746)
- **ui:** custom model selector dropdown with shift+click default (341084d)
- **tauri:** add --logout flag, devtools, and frontend logging (08db5c5)
- **tauri:** implement Phase 3 persistence & data layer (789a9bc)
- **ui:** auto-copy device code to clipboard on sign-in click (28fc965)
- **copilot-api:** implement OAuth device flow, SSE streaming, and model discovery (c1e09f1)
- **ui:** randomize welcome greeting on each visit (ced7ec8)
- scaffold Tauri v2 + Svelte 5 + TypeScript project (179a60d)

### Bug Fixes

- **app:** replace unsafe .expect() with proper error propagation and log silent catches (00bb0fc)
- **app:** clear stale skip/snooze settings on update check (31c055a)
- **ui:** render changelog as markdown in UpdateBanner (ad562ac)
- **tauri:** fix startup auto-connect, extract helpers, harden path validation (4ccc12b)
- **tauri:** harden MCP binary approval, drag-drop, auth sentinel, test by ID (5df0a8e)
- **tauri:** enforce binary approval on test_mcp_connection, safe migration v3 (24b310d)
- **tauri:** address review findings from security audit (3c0c4d1)
- **tauri:** harden security across 6 attack surfaces (2570cf7)
- **ui:** prevent race condition in title generation (a3d2e8f)
- **tauri:** simplify tray menu — remove dividers, shorten labels (4413f6c)
- **tauri:** fix duplicate tray icon and dock reopen on macOS (b5f0839)
- **app:** fix tray icon, window reopen, and auto-title generation (2d83bce)
- **ui:** address security and code review findings (10e42c7)
- **ui:** hide extracted file content from chat, send to API only (bf7ff81)
- **ui:** replace loading file pills with real data after read (83bf34a)
- **ui:** show file pills instantly on drop with loading state (d1ec3a0)
- **ui:** show chat view immediately when sending with file attachments (fa90952)
- **ui:** enlarge drop overlay and improve PDF extraction (d170f62)
- **ui:** use Tauri native drag-drop instead of HTML5 events (e840054)
- **ui:** make entire chat view a file drop zone (f574cb7)
- **ui:** fix drag-and-drop, async extraction, processing indicator (363d2b3)
- **ui:** show friendly file size warning for oversized uploads (d4a2a12)
- **projects:** raise file size limit to 50MB and show errors in UI (a12d27e)
- **ui:** navigate back to chat when selecting conversation from panels (59e5220)
- **ui:** only show 'no files found' after scan completes (cfcaf6f)
- **registry:** authenticate GitHub API calls in git import (1fa9746)
- **registry:** use tree API for recursive SKILL.md discovery in git import (72d68f8)
- **ui:** cancel draft save timer on send to prevent ghost drafts (0143e3a)
- **ui:** hide catalog results when search field is empty (bd93e5d)
- **ui:** persist installed badge after skill/agent registry install (9716fa0)
- **ui:** fix catalog YAML rendering, descriptions, install names, delete UX (38708cd)
- **ui:** pass content through registry items for catalog expand and install (5ddcdbe)
- **ui:** fix duplicate keys, source links, markdown rendering in expand views (d9cbaa0)
- **ui:** add expand buttons to catalog items, render MD, fix duplicate keys (da9c9ba)
- **ui:** fix agent links, expand buttons, registry defaults, and descriptions (a0e6f58)
- **app:** fix registry install failures and improve search quality (9bd76ea)
- **ui:** fix registry search, install 404s, expand buttons, and form layout (0a1352b)
- **ui:** polish skill cards and fix input borders (aafb903)
- **ui:** improve Skills & Agents panel UX consistency (38c19d7)
- **tauri:** address code review findings — SSRF, URL encoding, missing ID (bd5a17b)
- **mcp:** correct IPv6 localhost check (remove brackets) (651b2c4)
- **mcp:** security hardening and code quality improvements (a421571)
- **mcp:** pass packageArguments from registry to server commands (a56fd0e)
- **ui:** suppress state_referenced_locally warnings in McpServerForm (1370f88)
- **ui:** CSS spinners, copy buttons on setup commands, centered form (2a80c93)
- **ui:** address security, logic, and performance review findings (3e9170a)
- **ui:** show spinner in model selector while models load (5d90601)
- **ui:** load models and conversations after fresh login (8bf9319)
- **ui:** apply persisted default model on first load (de01b8e)
- harden debug credential permissions and log_frontend safety (b90122b)
- **tauri:** stop auto-opening devtools in debug builds (22405d3)
- **ui:** deduplicate models to prevent keyed each block crash (43d8a5d)
- **copilot-api:** use file-based credential storage in debug builds (5e62e0e)
- **ui:** improve model selector with dropdown chevron and loading state (ce76839)
- **copilot-api:** enable real keychain and fix API headers (a6bc068)
- **ui:** add drag region to auth and loading screens (51e3ad1)
- address security and code quality review findings (round 2) (a61f820)
- **ui:** fix auth polling string matching and add welcome taglines (deb4ec3)
- **copilot-api:** address security and code review findings (10d03a3)
- resolve review findings — stale references, dead CSS, docs update (dedf0d6)
- **tauri:** enable window dragging on custom title bar (5008419)
- **ui:** show sidebar toggle on welcome screen + add model selector (101aae6)
- **docs:** resolve 8 review issues across wireframes, IPC, and terminology (ed03d42)

### Refactoring

- **registry:** remove skills.sh, extract RegistryProvider trait (e715e97)
- **mcp-client:** replace built-in catalog with paginated MCP Registry (65e732f)
- **ui:** move model fetching to app-level store (e51d4c1)
- **ui:** custom title bar with toggle + jank-free sidebar collapse (23f2d57)
- **ui:** sidebar icon rail pattern with collapsible menu items (23ad8a5)

### Documentation

- document xtask versioning workflow and project structure (47ec8ae)
- mark Phase 11 (Auto-Update) as complete in AGENTS.md (0a3ac90)
- update AGENTS.md and chat.rs comment to reflect actual codebase (af0f15c)
- add pre-merge finalization review protocol to AGENTS.md (0dacccf)
- update AGENTS.md with Phase 9 file context details (f82e288)
- fix stale references in AGENTS.md (028837a)
- update AGENTS.md Phase 8 status, add JSDoc to types, fix UTF-8 truncation (2a1d690)
- mark Phase 8 (Skills & Agents) as complete (670c944)
- **agents-md:** update AGENTS.md for Phase 8 expanded scope (3446bc8)
- update AGENTS.md to reflect MCP Registry and current state (018554a)
- update input-area status to 🔧 in AGENTS.md (7da70f9)
- update AGENTS.md Phase 4 status for model selector (a7e8685)
- update implementation plan with Phase 1 completion status (e5429ef)
- add send shortcut toggle (Enter vs Cmd+Enter) (39cb4b2)
- add UX wireframes for all key screens (6eb746c)
- replace stale dependencies in AGENTS.md (58547c4)
- fix remaining review issues in AGENTS.md (5073c47)
- fix review issues in AGENTS.md Tauri rewrite (da02660)
- rewrite AGENTS.md for Tauri v2 + Svelte 5 + TypeScript stack (c63e1b9)
- fix 21 review findings across AGENTS.md and README (1be9a0b)
- add chat UX behaviors, favourites, system tray, and gap analysis items (a08d83c)
- strengthen review protocol — zero exceptions, explicit warnings (7d6c4c8)
- fix 13 review findings from extensive AGENTS.md review (65542af)
- add auto-update feature — self_update from GitHub Releases (109a041)
- add README with project overview, security model, and legal disclaimer (ac0a80a)
- clarify commit type must match actual change, not content (b6ddb4e)
- add git conventions — never push policy and conventional commits (baf6cf6)
- add mandatory dependency freshness policy (8bdf407)
- resolve 23 review findings in AGENTS.md (3c278c4)
- add mandatory agent review-fix loop protocol (1276778)
- add web research, MCP, skills & agents specs (dfd9211)
- add hard security requirement — no filesystem access (b5e95b5)
- move agent instructions to AGENTS.md (ad3b960)
- add initial agent instructions for Copilot Desktop (70a1736)

### Chores

- initialize CHANGELOG.md from existing commits (5f126f8)
- **ui:** remove debug logs from title generation (56b3781)
- **tauri:** replace app icon with Prismatic Launch design (9b4867a)
- **deps:** upgrade eslint to ^10 to match @eslint/js@10 (7aef46d)
- **deps:** update vite to 6.4.2 to fix security vulnerabilities (c2d452b)
- add new-branch rule to AGENTS.md (3400544)
- remove scaffold greet command, fix lint, update phase status (74474ae)
- rename app from Copilot Desktop to Chuck (ef113d8)

### Build

- **deps:** adopt workspace version inheritance for all crates (23c6b6b)

### Style

- **ui:** remove redundant testing flag assignment in McpServerForm (22a75f5)
- **ui:** unify SkillsPanel and AgentsPanel visual patterns (34ece03)
- **ui:** flip tagline hierarchy — punchline is the hero (f6999a5)
- **ui:** make tagline punchline pop with italic + accent color (3da06af)
- **ui:** update auth tagline to Chuck Yeager quip (8c47110)
- **ui:** split auth tagline into two styled lines (2a7ba8e)
- **ui:** remove assistant avatar from chat messages (1e922e9)
- **ui:** right-align user messages, fix sidebar toggle timing, lighten dark mode (107b8d3)
- **ui:** apply Warm Ink design system with distinctive typography (9c4382d)
- **ui:** redesign to minimal Claude Desktop-inspired layout (3f8c019)

## [0.1.0] — 2026-04-09

### Features

- **app:** add auto-update via tauri-plugin-updater (e5e94e4)
- **chat:** generate conversation titles via AI instead of truncation (2784257)
- **app:** implement Phase 10 — polish & platform features (a71b82a)
- **ui:** async file extraction with background cache (4ac84a0)
- **tauri:** add generic text extraction for file attachments (f729d12)
- **app:** implement Phase 9 — projects, file attachments, context-window summarization (f9cf939)
- **ui:** add progress bar for git import in Skills and Agents panels (c285607)
- **registry:** support *.agent.md discovery in git import (7042b2f)
- **ui:** sort catalog results by download count (highest first) (1be339b)
- **ui:** add agent catalog — registry browser + git import (b396686)
- **ui:** polish Skills & Agents panels (5692bbe)
- **ui:** add Skills & Agents panels, agent selector, and sidebar nav (cce0320)
- **ui:** add agent/skill stores, registry types, and command wrappers (7ec2b72)
- **tauri:** add registry client and git import with Tauri commands (c3d6e55)
- **chat:** integrate agent system prompt injection (ab65013)
- **tauri:** implement agent and skill Tauri commands (676ec99)
- **tauri:** add agent and skill CRUD queries (235a3d3)
- **tauri:** add schema migration v2 + SKILL.md parser (ff01a88)
- **mcp:** pagination, auto-connect, bare command support (abe56d3)
- **ui:** one-click MCP server add from registry (836a1be)
- **ui:** auto-fill npx/uvx commands for stdio MCP servers (851cbf4)
- **ui:** add setup guide and bottom CTA to registry detail view (a6eb9a2)
- **ui:** add registry detail view and loading spinner (17c945d)
- **mcp-client:** prioritize first-party servers in registry results (88109a2)
- **mcp-client:** use server-side search for MCP Registry (6ecf229)
- **mcp-client:** add MCP Registry browser and extract server form (93dcaad)
- **mcp-client:** implement MCP integration with rmcp SDK (1d09bd9)
- **web-research:** implement web search and URL fetching (e88e105)
- **ui:** add message actions (edit, regenerate, copy) and in-conversation search (63af8ac)
- **ui:** add markdown rendering, syntax-highlighted code blocks, and thinking section (8de1746)
- **ui:** custom model selector dropdown with shift+click default (341084d)
- **tauri:** add --logout flag, devtools, and frontend logging (08db5c5)
- **tauri:** implement Phase 3 persistence & data layer (789a9bc)
- **ui:** auto-copy device code to clipboard on sign-in click (28fc965)
- **copilot-api:** implement OAuth device flow, SSE streaming, and model discovery (c1e09f1)
- **ui:** randomize welcome greeting on each visit (ced7ec8)
- scaffold Tauri v2 + Svelte 5 + TypeScript project (179a60d)

### Bug Fixes

- **app:** replace unsafe .expect() with proper error propagation and log silent catches (00bb0fc)
- **app:** clear stale skip/snooze settings on update check (31c055a)
- **ui:** render changelog as markdown in UpdateBanner (ad562ac)
- **tauri:** fix startup auto-connect, extract helpers, harden path validation (4ccc12b)
- **tauri:** harden MCP binary approval, drag-drop, auth sentinel, test by ID (5df0a8e)
- **tauri:** enforce binary approval on test_mcp_connection, safe migration v3 (24b310d)
- **tauri:** address review findings from security audit (3c0c4d1)
- **tauri:** harden security across 6 attack surfaces (2570cf7)
- **ui:** prevent race condition in title generation (a3d2e8f)
- **tauri:** simplify tray menu — remove dividers, shorten labels (4413f6c)
- **tauri:** fix duplicate tray icon and dock reopen on macOS (b5f0839)
- **app:** fix tray icon, window reopen, and auto-title generation (2d83bce)
- **ui:** address security and code review findings (10e42c7)
- **ui:** hide extracted file content from chat, send to API only (bf7ff81)
- **ui:** replace loading file pills with real data after read (83bf34a)
- **ui:** show file pills instantly on drop with loading state (d1ec3a0)
- **ui:** show chat view immediately when sending with file attachments (fa90952)
- **ui:** enlarge drop overlay and improve PDF extraction (d170f62)
- **ui:** use Tauri native drag-drop instead of HTML5 events (e840054)
- **ui:** make entire chat view a file drop zone (f574cb7)
- **ui:** fix drag-and-drop, async extraction, processing indicator (363d2b3)
- **ui:** show friendly file size warning for oversized uploads (d4a2a12)
- **projects:** raise file size limit to 50MB and show errors in UI (a12d27e)
- **ui:** navigate back to chat when selecting conversation from panels (59e5220)
- **ui:** only show 'no files found' after scan completes (cfcaf6f)
- **registry:** authenticate GitHub API calls in git import (1fa9746)
- **registry:** use tree API for recursive SKILL.md discovery in git import (72d68f8)
- **ui:** cancel draft save timer on send to prevent ghost drafts (0143e3a)
- **ui:** hide catalog results when search field is empty (bd93e5d)
- **ui:** persist installed badge after skill/agent registry install (9716fa0)
- **ui:** fix catalog YAML rendering, descriptions, install names, delete UX (38708cd)
- **ui:** pass content through registry items for catalog expand and install (5ddcdbe)
- **ui:** fix duplicate keys, source links, markdown rendering in expand views (d9cbaa0)
- **ui:** add expand buttons to catalog items, render MD, fix duplicate keys (da9c9ba)
- **ui:** fix agent links, expand buttons, registry defaults, and descriptions (a0e6f58)
- **app:** fix registry install failures and improve search quality (9bd76ea)
- **ui:** fix registry search, install 404s, expand buttons, and form layout (0a1352b)
- **ui:** polish skill cards and fix input borders (aafb903)
- **ui:** improve Skills & Agents panel UX consistency (38c19d7)
- **tauri:** address code review findings — SSRF, URL encoding, missing ID (bd5a17b)
- **mcp:** correct IPv6 localhost check (remove brackets) (651b2c4)
- **mcp:** security hardening and code quality improvements (a421571)
- **mcp:** pass packageArguments from registry to server commands (a56fd0e)
- **ui:** suppress state_referenced_locally warnings in McpServerForm (1370f88)
- **ui:** CSS spinners, copy buttons on setup commands, centered form (2a80c93)
- **ui:** address security, logic, and performance review findings (3e9170a)
- **ui:** show spinner in model selector while models load (5d90601)
- **ui:** load models and conversations after fresh login (8bf9319)
- **ui:** apply persisted default model on first load (de01b8e)
- harden debug credential permissions and log_frontend safety (b90122b)
- **tauri:** stop auto-opening devtools in debug builds (22405d3)
- **ui:** deduplicate models to prevent keyed each block crash (43d8a5d)
- **copilot-api:** use file-based credential storage in debug builds (5e62e0e)
- **ui:** improve model selector with dropdown chevron and loading state (ce76839)
- **copilot-api:** enable real keychain and fix API headers (a6bc068)
- **ui:** add drag region to auth and loading screens (51e3ad1)
- address security and code quality review findings (round 2) (a61f820)
- **ui:** fix auth polling string matching and add welcome taglines (deb4ec3)
- **copilot-api:** address security and code review findings (10d03a3)
- resolve review findings — stale references, dead CSS, docs update (dedf0d6)
- **tauri:** enable window dragging on custom title bar (5008419)
- **ui:** show sidebar toggle on welcome screen + add model selector (101aae6)
- **docs:** resolve 8 review issues across wireframes, IPC, and terminology (ed03d42)

### Refactoring

- **registry:** remove skills.sh, extract RegistryProvider trait (e715e97)
- **mcp-client:** replace built-in catalog with paginated MCP Registry (65e732f)
- **ui:** move model fetching to app-level store (e51d4c1)
- **ui:** custom title bar with toggle + jank-free sidebar collapse (23f2d57)
- **ui:** sidebar icon rail pattern with collapsible menu items (23ad8a5)

### Documentation

- mark Phase 11 (Auto-Update) as complete in AGENTS.md (0a3ac90)
- update AGENTS.md and chat.rs comment to reflect actual codebase (af0f15c)
- add pre-merge finalization review protocol to AGENTS.md (0dacccf)
- update AGENTS.md with Phase 9 file context details (f82e288)
- fix stale references in AGENTS.md (028837a)
- update AGENTS.md Phase 8 status, add JSDoc to types, fix UTF-8 truncation (2a1d690)
- mark Phase 8 (Skills & Agents) as complete (670c944)
- **agents-md:** update AGENTS.md for Phase 8 expanded scope (3446bc8)
- update AGENTS.md to reflect MCP Registry and current state (018554a)
- update input-area status to 🔧 in AGENTS.md (7da70f9)
- update AGENTS.md Phase 4 status for model selector (a7e8685)
- update implementation plan with Phase 1 completion status (e5429ef)
- add send shortcut toggle (Enter vs Cmd+Enter) (39cb4b2)
- add UX wireframes for all key screens (6eb746c)
- replace stale dependencies in AGENTS.md (58547c4)
- fix remaining review issues in AGENTS.md (5073c47)
- fix review issues in AGENTS.md Tauri rewrite (da02660)
- rewrite AGENTS.md for Tauri v2 + Svelte 5 + TypeScript stack (c63e1b9)
- fix 21 review findings across AGENTS.md and README (1be9a0b)
- add chat UX behaviors, favourites, system tray, and gap analysis items (a08d83c)
- strengthen review protocol — zero exceptions, explicit warnings (7d6c4c8)
- fix 13 review findings from extensive AGENTS.md review (65542af)
- add auto-update feature — self_update from GitHub Releases (109a041)
- add README with project overview, security model, and legal disclaimer (ac0a80a)
- clarify commit type must match actual change, not content (b6ddb4e)
- add git conventions — never push policy and conventional commits (baf6cf6)
- add mandatory dependency freshness policy (8bdf407)
- resolve 23 review findings in AGENTS.md (3c278c4)
- add mandatory agent review-fix loop protocol (1276778)
- add web research, MCP, skills & agents specs (dfd9211)
- add hard security requirement — no filesystem access (b5e95b5)
- move agent instructions to AGENTS.md (ad3b960)
- add initial agent instructions for Copilot Desktop (70a1736)

### Chores

- **ui:** remove debug logs from title generation (56b3781)
- **tauri:** replace app icon with Prismatic Launch design (9b4867a)
- **deps:** upgrade eslint to ^10 to match @eslint/js@10 (7aef46d)
- **deps:** update vite to 6.4.2 to fix security vulnerabilities (c2d452b)
- add new-branch rule to AGENTS.md (3400544)
- remove scaffold greet command, fix lint, update phase status (74474ae)
- rename app from Copilot Desktop to Chuck (ef113d8)

### Style

- **ui:** remove redundant testing flag assignment in McpServerForm (22a75f5)
- **ui:** unify SkillsPanel and AgentsPanel visual patterns (34ece03)
- **ui:** flip tagline hierarchy — punchline is the hero (f6999a5)
- **ui:** make tagline punchline pop with italic + accent color (3da06af)
- **ui:** update auth tagline to Chuck Yeager quip (8c47110)
- **ui:** split auth tagline into two styled lines (2a7ba8e)
- **ui:** remove assistant avatar from chat messages (1e922e9)
- **ui:** right-align user messages, fix sidebar toggle timing, lighten dark mode (107b8d3)
- **ui:** apply Warm Ink design system with distinctive typography (9c4382d)
- **ui:** redesign to minimal Claude Desktop-inspired layout (3f8c019)


## [0.1.0] — 2026-04-09

### Features

- **app:** add auto-update via tauri-plugin-updater (e5e94e4)
- **chat:** generate conversation titles via AI instead of truncation (2784257)
- **app:** implement Phase 10 — polish & platform features (a71b82a)
- **ui:** async file extraction with background cache (4ac84a0)
- **tauri:** add generic text extraction for file attachments (f729d12)
- **app:** implement Phase 9 — projects, file attachments, context-window summarization (f9cf939)
- **ui:** add progress bar for git import in Skills and Agents panels (c285607)
- **registry:** support \*.agent.md discovery in git import (7042b2f)
- **ui:** sort catalog results by download count (highest first) (1be339b)
- **ui:** add agent catalog — registry browser + git import (b396686)
- **ui:** polish Skills & Agents panels (5692bbe)
- **ui:** add Skills & Agents panels, agent selector, and sidebar nav (cce0320)
- **ui:** add agent/skill stores, registry types, and command wrappers (7ec2b72)
- **tauri:** add registry client and git import with Tauri commands (c3d6e55)
- **chat:** integrate agent system prompt injection (ab65013)
- **tauri:** implement agent and skill Tauri commands (676ec99)
- **tauri:** add agent and skill CRUD queries (235a3d3)
- **tauri:** add schema migration v2 + SKILL.md parser (ff01a88)
- **mcp:** pagination, auto-connect, bare command support (abe56d3)
- **ui:** one-click MCP server add from registry (836a1be)
- **ui:** auto-fill npx/uvx commands for stdio MCP servers (851cbf4)
- **ui:** add setup guide and bottom CTA to registry detail view (a6eb9a2)
- **ui:** add registry detail view and loading spinner (17c945d)
- **mcp-client:** prioritize first-party servers in registry results (88109a2)
- **mcp-client:** use server-side search for MCP Registry (6ecf229)
- **mcp-client:** add MCP Registry browser and extract server form (93dcaad)
- **mcp-client:** implement MCP integration with rmcp SDK (1d09bd9)
- **web-research:** implement web search and URL fetching (e88e105)
- **ui:** add message actions (edit, regenerate, copy) and in-conversation search (63af8ac)
- **ui:** add markdown rendering, syntax-highlighted code blocks, and thinking section (8de1746)
- **ui:** custom model selector dropdown with shift+click default (341084d)
- **tauri:** add --logout flag, devtools, and frontend logging (08db5c5)
- **tauri:** implement Phase 3 persistence & data layer (789a9bc)
- **ui:** auto-copy device code to clipboard on sign-in click (28fc965)
- **copilot-api:** implement OAuth device flow, SSE streaming, and model discovery (c1e09f1)
- **ui:** randomize welcome greeting on each visit (ced7ec8)
- scaffold Tauri v2 + Svelte 5 + TypeScript project (179a60d)

### Bug Fixes

- **app:** replace unsafe .expect() with proper error propagation and log silent catches (00bb0fc)
- **app:** clear stale skip/snooze settings on update check (31c055a)
- **ui:** render changelog as markdown in UpdateBanner (ad562ac)
- **tauri:** fix startup auto-connect, extract helpers, harden path validation (4ccc12b)
- **tauri:** harden MCP binary approval, drag-drop, auth sentinel, test by ID (5df0a8e)
- **tauri:** enforce binary approval on test_mcp_connection, safe migration v3 (24b310d)
- **tauri:** address review findings from security audit (3c0c4d1)
- **tauri:** harden security across 6 attack surfaces (2570cf7)
- **ui:** prevent race condition in title generation (a3d2e8f)
- **tauri:** simplify tray menu — remove dividers, shorten labels (4413f6c)
- **tauri:** fix duplicate tray icon and dock reopen on macOS (b5f0839)
- **app:** fix tray icon, window reopen, and auto-title generation (2d83bce)
- **ui:** address security and code review findings (10e42c7)
- **ui:** hide extracted file content from chat, send to API only (bf7ff81)
- **ui:** replace loading file pills with real data after read (83bf34a)
- **ui:** show file pills instantly on drop with loading state (d1ec3a0)
- **ui:** show chat view immediately when sending with file attachments (fa90952)
- **ui:** enlarge drop overlay and improve PDF extraction (d170f62)
- **ui:** use Tauri native drag-drop instead of HTML5 events (e840054)
- **ui:** make entire chat view a file drop zone (f574cb7)
- **ui:** fix drag-and-drop, async extraction, processing indicator (363d2b3)
- **ui:** show friendly file size warning for oversized uploads (d4a2a12)
- **projects:** raise file size limit to 50MB and show errors in UI (a12d27e)
- **ui:** navigate back to chat when selecting conversation from panels (59e5220)
- **ui:** only show 'no files found' after scan completes (cfcaf6f)
- **registry:** authenticate GitHub API calls in git import (1fa9746)
- **registry:** use tree API for recursive SKILL.md discovery in git import (72d68f8)
- **ui:** cancel draft save timer on send to prevent ghost drafts (0143e3a)
- **ui:** hide catalog results when search field is empty (bd93e5d)
- **ui:** persist installed badge after skill/agent registry install (9716fa0)
- **ui:** fix catalog YAML rendering, descriptions, install names, delete UX (38708cd)
- **ui:** pass content through registry items for catalog expand and install (5ddcdbe)
- **ui:** fix duplicate keys, source links, markdown rendering in expand views (d9cbaa0)
- **ui:** add expand buttons to catalog items, render MD, fix duplicate keys (da9c9ba)
- **ui:** fix agent links, expand buttons, registry defaults, and descriptions (a0e6f58)
- **app:** fix registry install failures and improve search quality (9bd76ea)
- **ui:** fix registry search, install 404s, expand buttons, and form layout (0a1352b)
- **ui:** polish skill cards and fix input borders (aafb903)
- **ui:** improve Skills & Agents panel UX consistency (38c19d7)
- **tauri:** address code review findings — SSRF, URL encoding, missing ID (bd5a17b)
- **mcp:** correct IPv6 localhost check (remove brackets) (651b2c4)
- **mcp:** security hardening and code quality improvements (a421571)
- **mcp:** pass packageArguments from registry to server commands (a56fd0e)
- **ui:** suppress state_referenced_locally warnings in McpServerForm (1370f88)
- **ui:** CSS spinners, copy buttons on setup commands, centered form (2a80c93)
- **ui:** address security, logic, and performance review findings (3e9170a)
- **ui:** show spinner in model selector while models load (5d90601)
- **ui:** load models and conversations after fresh login (8bf9319)
- **ui:** apply persisted default model on first load (de01b8e)
- harden debug credential permissions and log_frontend safety (b90122b)
- **tauri:** stop auto-opening devtools in debug builds (22405d3)
- **ui:** deduplicate models to prevent keyed each block crash (43d8a5d)
- **copilot-api:** use file-based credential storage in debug builds (5e62e0e)
- **ui:** improve model selector with dropdown chevron and loading state (ce76839)
- **copilot-api:** enable real keychain and fix API headers (a6bc068)
- **ui:** add drag region to auth and loading screens (51e3ad1)
- address security and code quality review findings (round 2) (a61f820)
- **ui:** fix auth polling string matching and add welcome taglines (deb4ec3)
- **copilot-api:** address security and code review findings (10d03a3)
- resolve review findings — stale references, dead CSS, docs update (dedf0d6)
- **tauri:** enable window dragging on custom title bar (5008419)
- **ui:** show sidebar toggle on welcome screen + add model selector (101aae6)
- **docs:** resolve 8 review issues across wireframes, IPC, and terminology (ed03d42)

### Refactoring

- **registry:** remove skills.sh, extract RegistryProvider trait (e715e97)
- **mcp-client:** replace built-in catalog with paginated MCP Registry (65e732f)
- **ui:** move model fetching to app-level store (e51d4c1)
- **ui:** custom title bar with toggle + jank-free sidebar collapse (23f2d57)
- **ui:** sidebar icon rail pattern with collapsible menu items (23ad8a5)

### Documentation

- mark Phase 11 (Auto-Update) as complete in AGENTS.md (0a3ac90)
- update AGENTS.md and chat.rs comment to reflect actual codebase (af0f15c)
- add pre-merge finalization review protocol to AGENTS.md (0dacccf)
- update AGENTS.md with Phase 9 file context details (f82e288)
- fix stale references in AGENTS.md (028837a)
- update AGENTS.md Phase 8 status, add JSDoc to types, fix UTF-8 truncation (2a1d690)
- mark Phase 8 (Skills & Agents) as complete (670c944)
- **agents-md:** update AGENTS.md for Phase 8 expanded scope (3446bc8)
- update AGENTS.md to reflect MCP Registry and current state (018554a)
- update input-area status to 🔧 in AGENTS.md (7da70f9)
- update AGENTS.md Phase 4 status for model selector (a7e8685)
- update implementation plan with Phase 1 completion status (e5429ef)
- add send shortcut toggle (Enter vs Cmd+Enter) (39cb4b2)
- add UX wireframes for all key screens (6eb746c)
- replace stale dependencies in AGENTS.md (58547c4)
- fix remaining review issues in AGENTS.md (5073c47)
- fix review issues in AGENTS.md Tauri rewrite (da02660)
- rewrite AGENTS.md for Tauri v2 + Svelte 5 + TypeScript stack (c63e1b9)
- fix 21 review findings across AGENTS.md and README (1be9a0b)
- add chat UX behaviors, favourites, system tray, and gap analysis items (a08d83c)
- strengthen review protocol — zero exceptions, explicit warnings (7d6c4c8)
- fix 13 review findings from extensive AGENTS.md review (65542af)
- add auto-update feature — self_update from GitHub Releases (109a041)
- add README with project overview, security model, and legal disclaimer (ac0a80a)
- clarify commit type must match actual change, not content (b6ddb4e)
- add git conventions — never push policy and conventional commits (baf6cf6)
- add mandatory dependency freshness policy (8bdf407)
- resolve 23 review findings in AGENTS.md (3c278c4)
- add mandatory agent review-fix loop protocol (1276778)
- add web research, MCP, skills & agents specs (dfd9211)
- add hard security requirement — no filesystem access (b5e95b5)
- move agent instructions to AGENTS.md (ad3b960)
- add initial agent instructions for Copilot Desktop (70a1736)

### Chores

- **ui:** remove debug logs from title generation (56b3781)
- **tauri:** replace app icon with Prismatic Launch design (9b4867a)
- **deps:** upgrade eslint to ^10 to match @eslint/js@10 (7aef46d)
- **deps:** update vite to 6.4.2 to fix security vulnerabilities (c2d452b)
- add new-branch rule to AGENTS.md (3400544)
- remove scaffold greet command, fix lint, update phase status (74474ae)
- rename app from Copilot Desktop to Chuck (ef113d8)

### Style

- **ui:** remove redundant testing flag assignment in McpServerForm (22a75f5)
- **ui:** unify SkillsPanel and AgentsPanel visual patterns (34ece03)
- **ui:** flip tagline hierarchy — punchline is the hero (f6999a5)
- **ui:** make tagline punchline pop with italic + accent color (3da06af)
- **ui:** update auth tagline to Chuck Yeager quip (8c47110)
- **ui:** split auth tagline into two styled lines (2a7ba8e)
- **ui:** remove assistant avatar from chat messages (1e922e9)
- **ui:** right-align user messages, fix sidebar toggle timing, lighten dark mode (107b8d3)
- **ui:** apply Warm Ink design system with distinctive typography (9c4382d)
- **ui:** redesign to minimal Claude Desktop-inspired layout (3f8c019)
