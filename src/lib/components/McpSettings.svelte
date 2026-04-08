<script lang="ts">
  import {
    getMcpState,
    initMcp,
    addServer,
    removeServer,
    connectServer,
    disconnectServer,
    testConnection,
    loadTools,
    loadRegistry,
    loadRegistryMore,
    searchRegistry,
  } from "$lib/stores/mcp.svelte";
  import { approveMcpBinary } from "$lib/utils/commands";
  import type { McpConnectionInfo, McpServerConfig, RegistryServer } from "$lib/types/mcp";
  import McpServerForm from "./McpServerForm.svelte";
  import { onMount, onDestroy } from "svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const mcp = getMcpState();

  // ── View state ──────────────────────────────────────────────

  type ViewState =
    | { kind: "list" }
    | {
        kind: "form";
        editInfo?: McpConnectionInfo;
        registryEntry?: RegistryServer;
      }
    | { kind: "detail"; entry: RegistryServer };

  let view = $state<ViewState>({ kind: "list" });

  // ── Local state ─────────────────────────────────────────────

  let testingServer = $state<string | null>(null);
  let testResult = $state<{ serverId: string; success: boolean; message: string } | null>(null);
  let expandedTools = $state<string | null>(null);
  let registrySearch = $state("");
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
  let copiedCommand = $state<string | null>(null);

  async function copyCommand(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedCommand = text;
      setTimeout(() => {
        copiedCommand = null;
      }, 2000);
    } catch {
      // Clipboard not available
    }
  }

  // Debounced server-side search
  function handleSearchInput(value: string) {
    registrySearch = value;
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(
      () => {
        if (value.trim()) {
          searchRegistry(value.trim());
        } else {
          searchRegistry("");
        }
      },
      value.trim() ? 400 : 0,
    );
  }

  onMount(() => {
    initMcp();
    loadRegistry();
  });

  // ── Handlers ────────────────────────────────────────────────

  function openAddForm() {
    view = { kind: "form" };
  }

  function openEditForm(info: McpConnectionInfo) {
    view = { kind: "form", editInfo: info };
  }

  function openRegistryForm(entry: RegistryServer) {
    view = { kind: "form", registryEntry: entry };
  }

  /** Build a config and add the server in one click — no form needed. */
  let quickAdding = $state(false);
  async function quickAddFromRegistry(entry: RegistryServer) {
    quickAdding = true;
    try {
      const config: McpServerConfig = {
        id: `mcp-${Date.now()}`,
        name: entry.name,
        transport: "stdio",
        url: null,
        binaryPath: null,
        args: null,
        authHeader: null,
        fromCatalog: false,
        enabled: true,
      };

      // Determine transport + command from packages / remotes
      const npmPkg = entry.packages.find((p) => p.registryType === "npm");
      const pypiPkg = entry.packages.find((p) => p.registryType === "pypi");
      const nugetPkg = entry.packages.find((p) => p.registryType === "nuget");

      if (npmPkg) {
        const pkgRef = npmPkg.version
          ? `${npmPkg.identifier}@${npmPkg.version}`
          : npmPkg.identifier;
        config.binaryPath = "npx";
        config.args = JSON.stringify(["-y", pkgRef, ...npmPkg.arguments]);
      } else if (pypiPkg) {
        const pkgRef = pypiPkg.version
          ? `${pypiPkg.identifier}==${pypiPkg.version}`
          : pypiPkg.identifier;
        config.binaryPath = "uvx";
        config.args = JSON.stringify([pkgRef, ...pypiPkg.arguments]);
      } else if (nugetPkg) {
        config.binaryPath = "dotnet";
        config.args = JSON.stringify(["tool", "run", nugetPkg.identifier, ...nugetPkg.arguments]);
      } else if (!entry.isStdioOnly && entry.remotes.length > 0) {
        // HTTP server — use the first remote
        config.transport = "http";
        config.url = entry.remotes[0]?.url ?? null;
      } else {
        // No package info and no remote — fall back to form
        openRegistryForm(entry);
        quickAdding = false;
        return;
      }

      await addServer(config);
      view = { kind: "list" };
    } catch {
      // On failure, fall back to form so user can adjust
      openRegistryForm(entry);
    } finally {
      quickAdding = false;
    }
  }

  /** Can this registry entry be one-click added (has packages or HTTP remote)? */
  function canQuickAdd(entry: RegistryServer): boolean {
    return (
      entry.packages.some((p) => ["npm", "pypi", "nuget"].includes(p.registryType)) ||
      (!entry.isStdioOnly && entry.remotes.length > 0)
    );
  }

  function openDetail(entry: RegistryServer) {
    view = { kind: "detail", entry };
  }

  function returnToList() {
    view = { kind: "list" };
  }

  async function handleConnect(serverId: string, isRetry = false) {
    try {
      await connectServer(serverId);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.startsWith("BINARY_NOT_APPROVED:") && !isRetry) {
        const binaryPath = msg.replace("BINARY_NOT_APPROVED:", "");
        // approveMcpBinary shows a native OS confirmation dialog server-side
        try {
          await approveMcpBinary(binaryPath);
          await handleConnect(serverId, true);
        } catch {
          // User denied or approval failed — do nothing
        }
      }
    }
  }

  async function handleDisconnect(serverId: string) {
    try {
      await disconnectServer(serverId);
    } catch {
      // Error is in the store
    }
  }

  async function handleTest(info: McpConnectionInfo) {
    testingServer = info.config.id;
    testResult = null;
    try {
      const count = await testConnection(info.config.id);
      testResult = {
        serverId: info.config.id,
        success: true,
        message: `Connected — ${count} tool${count !== 1 ? "s" : ""} discovered`,
      };
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      // Handle binary approval the same way as connect
      if (msg.startsWith("BINARY_NOT_APPROVED:")) {
        const binaryPath = msg.replace("BINARY_NOT_APPROVED:", "");
        try {
          await approveMcpBinary(binaryPath);
          // Retry the test after approval
          const count = await testConnection(info.config.id);
          testResult = {
            serverId: info.config.id,
            success: true,
            message: `Connected — ${count} tool${count !== 1 ? "s" : ""} discovered`,
          };
        } catch (retryErr) {
          testResult = {
            serverId: info.config.id,
            success: false,
            message: retryErr instanceof Error ? retryErr.message : String(retryErr),
          };
        }
      } else {
        testResult = {
          serverId: info.config.id,
          success: false,
          message: msg,
        };
      }
    } finally {
      testingServer = null;
    }
  }

  async function handleRemove(serverId: string) {
    if (!confirm("Remove this MCP server? This cannot be undone.")) return;
    try {
      await removeServer(serverId);
    } catch {
      // Error is in the store
    }
  }

  async function toggleTools(serverId: string) {
    if (expandedTools === serverId) {
      expandedTools = null;
    } else {
      await loadTools(serverId);
      expandedTools = serverId;
    }
  }

  function statusIcon(status: string): string {
    switch (status) {
      case "connected":
        return "🟢";
      case "connecting":
        return "🟡";
      case "error":
        return "🔴";
      default:
        return "⚪";
    }
  }

  function isRegistryAdded(entry: RegistryServer): boolean {
    return mcp.servers.some(
      (s) =>
        s.config.name === entry.name ||
        (entry.remotes[0]?.url && s.config.url === entry.remotes[0].url),
    );
  }

  onDestroy(() => {
    if (searchDebounce) clearTimeout(searchDebounce);
  });
</script>

{#if view.kind === "form"}
  <McpServerForm
    editInfo={view.editInfo ?? null}
    registryEntry={view.registryEntry ?? null}
    onBack={returnToList}
  />
{:else if view.kind === "detail"}
  <!-- Registry Server Detail View -->
  <div class="mcp-settings">
    <header class="mcp-header">
      <button class="back-btn" onclick={returnToList} aria-label="Back to list">← Back</button>
      <h2 class="mcp-title">{view.entry.displayName}</h2>
      {#if !isRegistryAdded(view.entry)}
        <button
          class="header-add-btn"
          onclick={() => {
            if (view.kind === "detail") openRegistryForm(view.entry);
          }}
        >
          + Add Server
        </button>
      {:else}
        <span class="detail-added-badge">Added ✓</span>
      {/if}
    </header>

    <div class="mcp-content">
      <div class="detail-body">
        <!-- Description -->
        {#if view.entry.description}
          <p class="detail-description">{view.entry.description}</p>
        {/if}

        <!-- Meta row -->
        <div class="detail-meta">
          {#if view.entry.version}
            <span class="detail-chip">v{view.entry.version}</span>
          {/if}
          {#if view.entry.isStdioOnly}
            <span class="registry-transport stdio">STDIO</span>
          {:else}
            <span class="registry-transport http">HTTP</span>
          {/if}
          <span class="detail-chip muted">{view.entry.name}</span>
        </div>

        <!-- Remotes -->
        {#if view.entry.remotes.length > 0}
          <section class="detail-section">
            <h3 class="detail-section-heading">Remote Endpoints</h3>
            {#each view.entry.remotes as remote (remote.url ?? remote.transportType)}
              <div class="detail-card">
                <span class="detail-label">{remote.transportType}</span>
                {#if remote.url}
                  <code class="detail-code">{remote.url}</code>
                {/if}
                {#if remote.requiresAuth}
                  <span class="detail-auth-note"
                    >🔑 {remote.authDescription ?? "Auth required"}</span
                  >
                {/if}
              </div>
            {/each}
          </section>
        {/if}

        <!-- Packages -->
        {#if view.entry.packages.length > 0}
          <section class="detail-section">
            <h3 class="detail-section-heading">Install Packages</h3>
            {#each view.entry.packages as pkg (pkg.identifier)}
              <div class="detail-card">
                <span class="detail-label">{pkg.registryType}</span>
                <code class="detail-code"
                  >{pkg.identifier}{pkg.version ? `@${pkg.version}` : ""}</code
                >
              </div>
            {/each}
          </section>
        {/if}

        <!-- Run Command (package-based stdio servers — one-click add) -->
        {#if view.entry.isStdioOnly && view.entry.packages.length > 0}
          <section class="detail-section">
            <h3 class="detail-section-heading">Run Command</h3>
            <p class="setup-hint">
              Click <strong>Add Server</strong> below to configure automatically. Requires
              <code class="inline-code"
                >{view.entry.packages.find((p) => p.registryType === "npm")
                  ? "npx"
                  : view.entry.packages.find((p) => p.registryType === "pypi")
                    ? "uvx"
                    : "dotnet"}</code
              > in your PATH.
            </p>
            <div class="setup-commands">
              {#each view.entry.packages as pkg (pkg.identifier + "-guide")}
                {#if pkg.registryType === "npm"}
                  {@const cmd = `npx -y ${pkg.identifier}${pkg.version ? `@${pkg.version}` : ""}`}
                  <div class="setup-code-block">
                    <code class="setup-code">{cmd}</code>
                    <button
                      class="copy-inline-btn"
                      onclick={() => copyCommand(cmd)}
                      aria-label="Copy command">{copiedCommand === cmd ? "Copied!" : "Copy"}</button
                    >
                  </div>
                {:else if pkg.registryType === "pypi"}
                  {@const cmd = `uvx ${pkg.identifier}${pkg.version ? `==${pkg.version}` : ""}`}
                  <div class="setup-code-block">
                    <code class="setup-code">{cmd}</code>
                    <button
                      class="copy-inline-btn"
                      onclick={() => copyCommand(cmd)}
                      aria-label="Copy command">{copiedCommand === cmd ? "Copied!" : "Copy"}</button
                    >
                  </div>
                {:else if pkg.registryType === "nuget"}
                  {@const cmd = `dotnet tool run ${pkg.identifier}`}
                  <div class="setup-code-block">
                    <code class="setup-code">{cmd}</code>
                    <button
                      class="copy-inline-btn"
                      onclick={() => copyCommand(cmd)}
                      aria-label="Copy command">{copiedCommand === cmd ? "Copied!" : "Copy"}</button
                    >
                  </div>
                {/if}
              {/each}
            </div>
          </section>
        {:else if view.entry.isStdioOnly}
          <!-- Stdio with no packages — manual setup -->
          <section class="detail-section">
            <h3 class="detail-section-heading">Setup</h3>
            <p class="setup-hint">
              This server requires a manually installed binary. Check the repository link for
              installation instructions, then click <strong>Configure Server</strong> below.
            </p>
          </section>
        {:else if view.entry.remotes.length > 0 && view.entry.remotes.some((r) => r.requiresAuth)}
          <!-- HTTP with auth — needs manual API key -->
          <section class="detail-section">
            <h3 class="detail-section-heading">Setup</h3>
            <p class="setup-hint">
              This server requires authentication. Obtain an API key from the provider, then click
              <strong>Configure Server</strong> below to add it.
            </p>
          </section>
        {/if}

        <!-- Links -->
        {#if view.entry.repoUrl || view.entry.websiteUrl}
          <section class="detail-section">
            <h3 class="detail-section-heading">Links</h3>
            <div class="detail-links">
              {#if view.entry.repoUrl}
                <a href={view.entry.repoUrl} target="_blank" rel="noopener" class="detail-link">
                  📦 Repository
                </a>
              {/if}
              {#if view.entry.websiteUrl}
                <a href={view.entry.websiteUrl} target="_blank" rel="noopener" class="detail-link">
                  🌐 Website
                </a>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Bottom CTA -->
        {#if !isRegistryAdded(view.entry)}
          {#if canQuickAdd(view.entry)}
            <button
              class="detail-add-btn"
              disabled={quickAdding}
              onclick={() => {
                if (view.kind === "detail") quickAddFromRegistry(view.entry);
              }}
            >
              {quickAdding ? "Adding…" : "+ Add Server"}
            </button>
          {:else}
            <button
              class="detail-add-btn"
              onclick={() => {
                if (view.kind === "detail") openRegistryForm(view.entry);
              }}
            >
              + Configure Server
            </button>
          {/if}
        {:else}
          <div class="detail-already-added">✓ This server is already configured</div>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <div class="mcp-settings">
    <!-- Header -->
    <header class="mcp-header">
      <button class="back-btn" onclick={onBack} aria-label="Go back">← Back</button>
      <h2 class="mcp-title">MCP Servers</h2>
      <button class="header-add-btn" onclick={openAddForm}>+ Add Custom</button>
    </header>

    <div class="mcp-content">
      {#if mcp.loading}
        <div class="mcp-loading">Loading MCP servers...</div>
      {:else}
        <!-- Configured Servers -->
        <section class="mcp-section">
          <h3 class="section-heading">Configured Servers</h3>

          {#if mcp.servers.length === 0}
            <p class="section-empty">
              No MCP servers configured yet. Browse the registry below or add a custom server.
            </p>
          {/if}

          {#each mcp.servers as info (info.config.id)}
            <div class="server-card">
              <div class="server-header">
                <span
                  class="server-status"
                  title={info.status}
                  role="status"
                  aria-label={`Server ${info.status}`}>{statusIcon(info.status)}</span
                >
                <strong class="server-name">{info.config.name}</strong>
                <div class="server-actions">
                  {#if info.status === "connected"}
                    <button class="action-btn" onclick={() => handleDisconnect(info.config.id)}>
                      Disconnect
                    </button>
                  {:else if info.status === "connecting"}
                    <button class="action-btn" disabled>Connecting...</button>
                  {:else}
                    <button class="action-btn" onclick={() => handleConnect(info.config.id)}>
                      Connect
                    </button>
                  {/if}
                  <button
                    class="action-btn"
                    onclick={() => handleTest(info)}
                    disabled={testingServer === info.config.id}
                  >
                    {testingServer === info.config.id ? "Testing..." : "Test"}
                  </button>
                  <button class="action-btn" onclick={() => openEditForm(info)}>Edit</button>
                  <button class="action-btn danger" onclick={() => handleRemove(info.config.id)}>
                    Remove
                  </button>
                </div>
              </div>

              <div class="server-meta">
                <span class="meta-tag">Transport: {info.config.transport.toUpperCase()}</span>
                {#if info.config.transport === "http" && info.config.url}
                  <span class="meta-tag url-tag" title={info.config.url}>
                    {info.config.url}
                  </span>
                {/if}
                {#if info.config.transport === "stdio" && info.config.binaryPath}
                  <span class="meta-tag">Binary: {info.config.binaryPath}</span>
                {/if}
                <span class="meta-tag">
                  Tools: {info.toolCount}
                  {#if info.status === "connected" && info.toolCount > 0}
                    <button
                      class="tools-toggle"
                      onclick={() => toggleTools(info.config.id)}
                      aria-label="Toggle tools list"
                      aria-expanded={expandedTools === info.config.id}
                    >
                      {expandedTools === info.config.id ? "▼" : "▶"}
                    </button>
                  {/if}
                </span>
              </div>

              {#if info.error}
                <div class="server-error">⚠ {info.error}</div>
              {/if}

              {#if testResult && testResult.serverId === info.config.id}
                <div
                  class="test-result"
                  class:success={testResult.success}
                  class:failure={!testResult.success}
                >
                  {testResult.success ? "✓" : "✗"}
                  {testResult.message}
                </div>
              {/if}

              {#if expandedTools === info.config.id && info.tools}
                <div class="tools-list">
                  {#each info.tools as tool (tool.name)}
                    <div class="tool-item">
                      <span class="tool-name">{tool.name}</span>
                      {#if tool.description}
                        <span class="tool-desc">{tool.description}</span>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </section>

        <!-- MCP Registry -->
        <section class="mcp-section">
          <h3 class="section-heading">
            MCP Registry
            {#if mcp.registry.length > 0 && !mcp.registryLoading}
              <span class="registry-count">({mcp.registry.length})</span>
            {/if}
          </h3>
          <p class="section-desc">
            Browse servers from the official
            <a
              href="https://registry.modelcontextprotocol.io"
              target="_blank"
              rel="noopener noreferrer">MCP Registry</a
            >. Search by name to find specific servers.
          </p>

          <div class="registry-search">
            <input
              type="text"
              value={registrySearch}
              oninput={(e) => handleSearchInput(e.currentTarget.value)}
              placeholder="Search servers (e.g. azure, github, postgres)..."
              class="search-input"
            />
            {#if mcp.registryLoading}
              <span class="search-spinner" role="status" aria-label="Searching"></span>
            {/if}
          </div>

          {#if mcp.registryLoading && mcp.registry.length === 0}
            <div class="registry-loading">
              <span class="loading-spinner"></span> Fetching from registry...
            </div>
          {:else if mcp.registry.length > 0}
            <div class="registry-list">
              {#each mcp.registry as entry (entry.name)}
                <button class="registry-entry" onclick={() => openDetail(entry)} type="button">
                  <div class="registry-info">
                    <strong class="registry-name">{entry.displayName}</strong>
                    {#if entry.isStdioOnly}
                      <span class="registry-transport stdio">STDIO</span>
                    {:else}
                      <span class="registry-transport http">HTTP</span>
                    {/if}
                  </div>
                  {#if entry.description}
                    <p class="registry-desc">{entry.description}</p>
                  {/if}
                  <div class="registry-actions">
                    {#if isRegistryAdded(entry)}
                      <span class="registry-added">Added ✓</span>
                    {:else}
                      <span class="registry-arrow">→</span>
                    {/if}
                  </div>
                </button>
              {/each}
              {#if mcp.hasMoreRegistry}
                <button
                  class="load-more-btn"
                  onclick={loadRegistryMore}
                  disabled={mcp.registryLoading}
                  type="button"
                >
                  {#if mcp.registryLoading}
                    <span class="loading-spinner"></span> Loading…
                  {:else}
                    Load more
                  {/if}
                </button>
              {/if}
            </div>
          {:else if registrySearch.trim() && !mcp.registryLoading}
            <p class="section-empty">No servers match "{registrySearch}"</p>
          {:else if !mcp.registryLoading}
            <p class="section-empty">Could not load the MCP Registry.</p>
          {/if}
        </section>
      {/if}
    </div>
  </div>
{/if}

<style>
  .mcp-settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .mcp-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--color-border-primary);
    flex-shrink: 0;
  }

  .back-btn {
    background: none;
    border: none;
    color: var(--color-accent-copper);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }
  .back-btn:hover {
    background: var(--color-bg-hover);
  }

  .mcp-title {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-xl);
    color: var(--color-text-primary);
    margin: 0;
    flex: 1;
  }

  .header-add-btn {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }
  .header-add-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-accent-copper);
    border-color: var(--color-accent-copper);
  }

  .mcp-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
  }

  .mcp-loading {
    text-align: center;
    color: var(--color-text-secondary);
    padding: var(--spacing-2xl);
  }

  .mcp-section {
    margin-bottom: var(--spacing-xl);
  }

  .section-heading {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-tertiary);
    margin: 0 0 var(--spacing-sm) 0;
  }

  .section-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-sm) 0;
  }
  .section-desc a {
    color: var(--color-accent-copper);
    text-decoration: underline;
  }

  .section-empty {
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    font-style: italic;
  }

  /* ── Server Cards ── */

  .server-card {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
  }

  .server-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .server-status {
    font-size: var(--font-size-2xs);
  }

  .server-name {
    flex: 1;
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }

  .server-actions {
    display: flex;
    gap: var(--spacing-xs);
  }

  .server-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-xs);
  }

  .meta-tag {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    background: var(--color-bg-tertiary);
    padding: 2px var(--spacing-xs);
    border-radius: var(--radius-sm);
  }

  .url-tag {
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .server-error {
    font-size: var(--font-size-xs);
    color: var(--color-error);
    margin-top: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border-radius: var(--radius-sm);
  }

  .test-result {
    font-size: var(--font-size-xs);
    margin-top: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
  }
  .test-result.success {
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 8%, transparent);
  }
  .test-result.failure {
    color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
  }

  .tools-toggle {
    background: none;
    border: none;
    color: var(--color-accent-copper);
    cursor: pointer;
    font-size: var(--font-size-xs);
    padding: 0;
    margin-left: var(--spacing-xs);
  }

  .tools-list {
    margin-top: var(--spacing-sm);
    border-top: 1px solid var(--color-border-secondary);
    padding-top: var(--spacing-sm);
  }

  .tool-item {
    display: flex;
    flex-direction: column;
    padding: var(--spacing-xs) 0;
  }

  .tool-name {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-primary);
    font-family: var(--font-mono);
  }

  .tool-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
  }

  /* ── Action Buttons ── */

  .action-btn {
    font-size: var(--font-size-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }
  .action-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }
  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .action-btn.danger:hover {
    color: var(--color-error);
    border-color: var(--color-error);
  }

  /* ── MCP Registry ── */

  .registry-count {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-normal);
    color: var(--color-text-tertiary);
    text-transform: none;
    letter-spacing: 0;
  }

  .registry-search {
    margin-bottom: var(--spacing-sm);
    position: relative;
  }

  .search-input {
    width: 100%;
    padding: var(--spacing-sm);
    padding-right: var(--spacing-2xl);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    box-sizing: border-box;
  }
  .search-input:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .search-spinner {
    position: absolute;
    right: var(--spacing-sm);
    top: 50%;
    width: 14px;
    height: 14px;
    margin-top: -7px;
    border: 2px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .registry-loading {
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    font-style: italic;
    padding: var(--spacing-md) 0;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .loading-spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    flex-shrink: 0;
  }

  .registry-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .load-more-btn {
    width: 100%;
    padding: var(--spacing-sm);
    background: none;
    border: 1px dashed var(--color-border-secondary);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-xs);
    transition:
      color var(--transition-fast),
      border-color var(--transition-fast);
  }
  .load-more-btn:hover:not(:disabled) {
    color: var(--color-accent-copper);
    border-color: var(--color-accent-copper);
  }
  .load-more-btn:disabled {
    cursor: default;
    opacity: 0.7;
  }

  .registry-entry {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-body);
    width: 100%;
    transition: border-color var(--transition-fast);
  }
  .registry-entry:hover {
    border-color: var(--color-accent-copper);
  }

  .registry-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    flex: 1;
    min-width: 0;
  }

  .registry-name {
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }

  .registry-transport {
    font-size: var(--font-size-2xs);
    padding: 1px var(--spacing-xs);
    text-transform: uppercase;
    font-weight: var(--font-weight-medium);
  }
  .registry-transport.http {
    color: var(--color-accent-copper);
    background: color-mix(in srgb, var(--color-accent-copper) 12%, transparent);
  }
  .registry-transport.stdio {
    color: var(--color-text-tertiary);
    background: var(--color-bg-tertiary);
  }

  .registry-desc {
    width: 100%;
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .registry-actions {
    flex-shrink: 0;
  }

  .registry-added {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .registry-arrow {
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    transition: transform var(--transition-fast);
  }
  .registry-entry:hover .registry-arrow {
    transform: translateX(2px);
    color: var(--color-accent-copper);
  }

  /* ── Detail View ── */

  .detail-added-badge {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    font-style: italic;
    padding: var(--spacing-xs) var(--spacing-sm);
  }

  .detail-body {
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .detail-description {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    line-height: 1.6;
    margin: 0;
  }

  .detail-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .detail-chip {
    font-size: var(--font-size-xs);
    padding: 2px var(--spacing-sm);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent-copper) 12%, transparent);
    color: var(--color-accent-copper);
    font-weight: var(--font-weight-medium);
  }
  .detail-chip.muted {
    background: var(--color-bg-tertiary);
    color: var(--color-text-tertiary);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
  }

  .detail-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .detail-section-heading {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-tertiary);
    margin: 0;
  }

  .detail-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-sm);
    flex-wrap: wrap;
  }

  .detail-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    min-width: 50px;
  }

  .detail-code {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--color-text-primary);
    word-break: break-all;
  }

  .detail-auth-note {
    font-size: var(--font-size-xs);
    color: var(--color-accent-copper);
    font-style: italic;
  }

  .detail-links {
    display: flex;
    gap: var(--spacing-md);
  }

  .detail-link {
    font-size: var(--font-size-sm);
    color: var(--color-accent-copper);
    text-decoration: none;
    transition: opacity var(--transition-fast);
  }
  .detail-link:hover {
    opacity: 0.8;
    text-decoration: underline;
  }

  /* ── Setup Guide ── */

  .setup-commands {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .setup-hint {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-sm);
    line-height: var(--leading-relaxed);
  }

  .inline-code {
    font-family: var(--font-mono);
    font-size: 0.9em;
    background: var(--color-bg-tertiary);
    padding: 1px 5px;
    border-radius: var(--radius-sm);
  }

  .setup-code-block {
    display: flex;
    align-items: center;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .setup-code {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--color-text-primary);
    padding: var(--spacing-sm) var(--spacing-md);
    flex: 1;
    word-break: break-all;
  }

  .copy-inline-btn {
    background: none;
    border: none;
    border-left: 1px solid var(--color-border-secondary);
    cursor: pointer;
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-tertiary);
    transition:
      color var(--transition-fast),
      background var(--transition-fast);
    white-space: nowrap;
  }
  .copy-inline-btn:hover {
    color: var(--color-accent-copper);
    background: color-mix(in srgb, var(--color-accent-copper) 8%, transparent);
  }

  /* ── Detail CTA ── */

  .detail-add-btn {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
    border: none;
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    font-family: var(--font-body);
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }
  .detail-add-btn:hover {
    opacity: 0.85;
  }

  .detail-already-added {
    text-align: center;
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    font-style: italic;
    padding: var(--spacing-sm);
  }
</style>
