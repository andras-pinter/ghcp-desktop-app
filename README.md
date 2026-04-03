# GHCP Desktop

A native cross-platform desktop chat application for GitHub Copilot, built with **Rust** and **GPUI**.

> [!IMPORTANT]
> This is an **unofficial, independent project**. It is not made by, endorsed by, or affiliated with GitHub, Microsoft, or OpenAI. "GitHub Copilot" is a trademark of GitHub, Inc. This application requires your own active [GitHub Copilot subscription](https://github.com/features/copilot).

## About

GHCP Desktop provides a dedicated chat interface for GitHub Copilot — inspired by standalone AI assistant apps like Claude Desktop. It focuses on **conversations**, not code editing, giving you a distraction-free environment for research, brainstorming, and general-purpose AI assistance powered by your existing Copilot subscription.

### Key Features

- 💬 **Streaming chat** — real-time token-by-token responses via SSE
- ✏️ **Edit & regenerate** — edit any sent message or regenerate the last response
- 📎 **File attachments** — drag-and-drop files (text, PDF, images) as context
- ⭐ **Favourites** — pin important conversations for quick access
- 🔍 **Search** — find conversations in the sidebar; Cmd+F / Ctrl+F within a conversation
- 📂 **Projects** — organize conversations by topic with pinned context files
- 🔌 **MCP integration** — connect external tools via the Model Context Protocol
- 🌐 **Web research** — AI-powered search and URL fetching for grounded answers
- 🤖 **Skills & Agents** — configurable agent personas with custom system prompts and tool access
- 🧠 **Thinking display** — collapsible reasoning/thinking sections from supported models
- 🎨 **Themes** — light/dark with system preference detection and manual toggle
- 🔀 **Model selector** — pick from available Copilot models
- 🖥️ **System tray** — minimize to tray; streaming continues when window is hidden
- 🔒 **Privacy-first** — zero filesystem access; files enter only via drag-and-drop/picker, read once into memory
- ⌨️ **Keyboard shortcuts** — global hotkey to summon from anywhere + full app navigation
- 🖥️ **Native performance** — Rust + GPUI, no Electron, no web views
- 🔄 **Auto-update** — seamless updates from GitHub Releases
- 💾 **Crash recovery** — auto-saved drafts and interrupted stream preservation

### Supported Platforms

| Platform | Status |
|----------|--------|
| macOS | Primary (App Sandbox) |
| Linux | Supported |
| Windows | Supported (maturing) |

## Architecture

A 5-crate Rust workspace:

```
crates/
├── app/              # GPUI desktop application (UI, views, state)
├── copilot-api/      # GitHub Copilot API client (standalone, no GPUI dep)
├── mcp-client/       # MCP protocol client (standalone, no GPUI dep)
├── web-research/     # AI search & URL fetching (standalone, no GPUI dep)
└── markdown-render/  # Markdown → GPUI element rendering
```

The `copilot-api`, `mcp-client`, and `web-research` crates have **zero GPUI dependency** — they're reusable in CLIs, servers, or other contexts.

## Getting Started

> 🚧 **This project is in the planning/specification phase.** No runnable code yet.

### Prerequisites

- Rust stable (latest)
- A valid [GitHub Copilot](https://github.com/features/copilot) subscription
- Platform-specific dependencies for GPUI (see [GPUI docs](https://github.com/zed-industries/zed/tree/main/crates/gpui))

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run -p copilot-desktop
```

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

## Legal

### Disclaimer

This software is provided "as is", without warranty of any kind. This is an independent, community-driven project. It is **not** an official GitHub or Microsoft product.

Use of this application requires a valid GitHub Copilot subscription and is subject to:

- [GitHub Terms of Service](https://docs.github.com/en/site-policy/github-terms/github-terms-of-service)
- [GitHub Generative AI Services Terms](https://github.com/customer-terms)
- [GitHub Acceptable Use Policies](https://docs.github.com/en/site-policy/acceptable-use-policies)

Users are responsible for ensuring their use of this application complies with all applicable terms and agreements.

### License

This project is released into the public domain under the [Unlicense](./LICENSE).

### Third-Party Notices

- **GPUI** — © Zed Industries, licensed under Apache 2.0
- **GitHub Copilot** — trademark of GitHub, Inc.

All other dependencies are listed in `Cargo.lock` with their respective licenses.
