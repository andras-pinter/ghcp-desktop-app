# Chuck

A native cross-platform desktop chat application for GitHub Copilot, built with **Tauri v2 + Svelte 5 + TypeScript**.

> [!IMPORTANT]
> This is an **unofficial, independent project**. It is not made by, endorsed by, or affiliated with GitHub, Microsoft, or OpenAI. "GitHub Copilot" is a trademark of GitHub, Inc. This application requires your own active [GitHub Copilot subscription](https://github.com/features/copilot).

## About

Chuck provides a dedicated chat interface for GitHub Copilot — inspired by standalone AI assistant apps like Claude Desktop. It focuses on **conversations**, not code editing, giving you a distraction-free environment for research, brainstorming, and general-purpose AI assistance powered by your existing Copilot subscription. Named after Chuck Yeager, who broke the sound barrier.

### Key Features

- 💬 **Streaming chat** — real-time token-by-token responses via SSE
- ✏️ **Edit & regenerate** — edit any sent message or regenerate the last response
- 📎 **File attachments** — drag-and-drop files (text, PDF, images) as context
- ⭐ **Favourites** — pin important conversations for quick access
- 🔍 **Search** — find conversations in the sidebar; Cmd+F / Ctrl+F within a conversation
- 📂 **Projects** — organize conversations by topic with pinned context files
- 🔌 **MCP integration** — connect external tools via the Model Context Protocol
- 🌐 **Web research** — AI-powered search and URL fetching for grounded answers
- 🤖 **Skills & Agents** — configurable agent personas with custom system prompts and tool access; browse and install from the aitmpl.com registry and persistent git repository sources
- 🧠 **Thinking display** — collapsible reasoning/thinking sections from supported models
- 🎨 **Themes** — light/dark with system preference detection and manual toggle
- 🔀 **Model selector** — pick from available Copilot models
- 🖥️ **System tray** — minimize to tray; streaming continues when window is hidden
- 🔒 **Privacy-first** — zero filesystem access; files enter only via drag-and-drop/picker, read once into memory
- ⌨️ **Slash commands** — quick actions via `/` prefix: `/help`, `/delete`, `/title`, `/export`, `/web`, `/model`, `/edit`, `/regenerate`
- ⌨️ **Keyboard shortcuts** — global hotkey to summon from anywhere + full app navigation
- 🖥️ **Native performance** — Rust backend + system webview, ~5-10MB bundle
- 🔄 **Auto-update** — seamless updates from GitHub Releases
- 💾 **Crash recovery** — auto-saved drafts and interrupted stream preservation

### Supported Platforms

| Platform | Status |
|----------|--------|
| macOS | Primary (WebKit, App Sandbox) |
| Linux | Supported (WebKitGTK) |
| Windows | Supported (WebView2) |

## Architecture

```
chuck/
├── src/                  # Svelte 5 + TypeScript frontend
│   ├── lib/components/   # Svelte components (Sidebar, ChatView, InputArea, etc.)
│   ├── lib/stores/       # Svelte 5 runes-based stores
│   ├── lib/types/        # TypeScript types (mirrors Rust types)
│   └── lib/utils/        # Tauri IPC wrappers, formatting helpers
├── src-tauri/            # Tauri v2 Rust backend
│   ├── src/commands/     # Tauri IPC command handlers
│   └── src/db/           # SQLite persistence + migrations
└── crates/               # Standalone Rust library crates
    ├── copilot-api/      # GitHub Copilot API client (OAuth + SSE streaming)
    ├── mcp-client/       # MCP protocol client (HTTP + stdio transports)
    └── web-research/     # Web search API + URL content extraction
```

The `copilot-api`, `mcp-client`, and `web-research` crates have **zero Tauri dependency** — they're reusable in CLIs, servers, or other contexts.

## Getting Started

### Prerequisites

- Rust stable (latest) + `cargo-tauri` CLI
- Node.js 20+ and pnpm
- A valid [GitHub Copilot](https://github.com/features/copilot) subscription
- Platform-specific: Xcode CLI tools (macOS), webkit2gtk + libjavascriptcoregtk (Linux), WebView2 (Windows)

### Setup

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Install frontend dependencies
pnpm install

# Development (hot-reload frontend + Rust backend)
cargo tauri dev

# Build for production
cargo tauri build
```

### Individual Checks

```bash
# Rust
cargo build --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
cargo test --workspace

# Frontend
pnpm check        # svelte-check
pnpm lint          # ESLint + Prettier
pnpm test          # Vitest
pnpm build         # Vite production build
```

## macOS Code Signing & Auto-Update

The macOS auto-updater requires a consistent code signing identity across builds.
Without it, macOS blocks the in-place app replacement (ad-hoc signatures are unique per build).

**Current setup:** A self-signed code signing certificate is used via GitHub Actions secrets:

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` file |
| `APPLE_SIGNING_IDENTITY` | Certificate Common Name (e.g., `Chuck Self-Signed`) |

This enables the auto-updater to work without a paid Apple Developer account.
Users will see a Gatekeeper warning on first launch (right-click → Open to bypass).

**To upgrade to notarized builds** (removes Gatekeeper warning), add these additional secrets
from a paid Apple Developer account ($99/year):

| Secret | Value |
|---|---|
| `APPLE_ID` | Your Apple ID email |
| `APPLE_PASSWORD` | App-specific password |
| `APPLE_TEAM_ID` | Your Apple Developer Team ID |

## Security Model

This application enforces a **strict no-filesystem-access policy**:

- No reading, writing, or browsing the user's filesystem
- Files enter **only** via drag-and-drop or file picker dialogs, read once into memory
- MCP stdio is the only subprocess exception (user-approved binaries, confirmation dialog required)
- URL fetching blocks private IPs, localhost, and cloud metadata endpoints
- macOS builds require App Sandbox entitlements

See [AGENTS.md](./AGENTS.md) for the complete security specification.

## Project Documentation

- **[AGENTS.md](./AGENTS.md)** — comprehensive project specification, architecture, coding conventions, implementation plan, and agent instructions

**Extended Documentation** (in `docs/`):
- **[docs/PLAN.md](./docs/PLAN.md)** — implementation plan, build commands, risks, design reference
- **[docs/API-REFERENCE.md](./docs/API-REFERENCE.md)** — dependencies, Copilot API, MCP protocol, skills/agents system
- **[docs/DATA-MODEL.md](./docs/DATA-MODEL.md)** — SQLite schema, migrations, versioning
- **[docs/WIREFRAMES.md](./docs/WIREFRAMES.md)** — 9 ASCII wireframes for all screens
- **[docs/RELEASE.md](./docs/RELEASE.md)** — release workflow, code signing, distribution

## Legal

### Disclaimer

This software is provided "as is", without warranty of any kind. This is an independent, community-driven project. It is **not** an official GitHub or Microsoft product.

Use of this application requires a valid GitHub Copilot subscription and is subject to:

- [GitHub Terms of Service](https://docs.github.com/en/site-policy/github-terms/github-terms-of-service)
- [GitHub Generative AI Services Terms](https://github.com/customer-terms)
- [GitHub Acceptable Use Policies](https://docs.github.com/en/site-policy/acceptable-use-policies)

Users are responsible for ensuring their use of this application complies with all applicable terms and agreements.

### License

This project is licensed under the [Apache License 2.0](./LICENSE).

Copyright 2026 Andras Pinter.

### Third-Party Notices

- **Tauri** — © Tauri Contributors, licensed under Apache 2.0 / MIT
- **Svelte** — © Svelte Contributors, licensed under MIT
- **GitHub Copilot** — trademark of GitHub, Inc.

All other dependencies are listed in `Cargo.lock` and `pnpm-lock.yaml` with their respective licenses.
