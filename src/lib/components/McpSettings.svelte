<script lang="ts">
  import {
    getMcpState,
    initMcp,
    removeServer,
    connectServer,
    disconnectServer,
    testConnection,
    loadTools,
    loadRegistry,
    searchRegistry,
  } from "$lib/stores/mcp.svelte";
  import type { McpConnectionInfo, RegistryServer } from "$lib/types/mcp";
  import McpServerForm from "./McpServerForm.svelte";
  import { onMount } from "svelte";

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
      };

  let view = $state<ViewState>({ kind: "list" });

  // ── Local state ─────────────────────────────────────────────

  let testingServer = $state<string | null>(null);
  let testResult = $state<{ serverId: string; success: boolean; message: string } | null>(null);
  let expandedTools = $state<string | null>(null);
  let registrySearch = $state("");
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;

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

  function returnToList() {
    view = { kind: "list" };
  }

  async function handleConnect(serverId: string) {
    try {
      await connectServer(serverId);
    } catch {
      // Error is in the store
    }
  }

  async function handleDisconnect(serverId: string) {
    await disconnectServer(serverId);
  }

  async function handleTest(info: McpConnectionInfo) {
    testingServer = info.config.id;
    testResult = null;
    try {
      const count = await testConnection(info.config);
      testResult = {
        serverId: info.config.id,
        success: true,
        message: `Connected — ${count} tool${count !== 1 ? "s" : ""} discovered`,
      };
    } catch (e) {
      testResult = {
        serverId: info.config.id,
        success: false,
        message: e instanceof Error ? e.message : String(e),
      };
    } finally {
      testingServer = null;
    }
  }

  async function handleRemove(serverId: string) {
    await removeServer(serverId);
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
</script>

{#if view.kind === "form"}
  <McpServerForm
    editInfo={view.editInfo ?? null}
    registryEntry={view.registryEntry ?? null}
    onBack={returnToList}
  />
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
                <span class="server-status" title={info.status}>{statusIcon(info.status)}</span>
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
                    <button class="tools-toggle" onclick={() => toggleTools(info.config.id)}>
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
              <span class="search-spinner" aria-label="Searching">⟳</span>
            {/if}
          </div>

          {#if mcp.registryLoading && mcp.registry.length === 0}
            <div class="registry-loading">Fetching from registry...</div>
          {:else if mcp.registry.length > 0}
            <div class="registry-list">
              {#each mcp.registry as entry (entry.name)}
                <div class="registry-entry">
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
                      <button class="action-btn" onclick={() => openRegistryForm(entry)}>
                        Add
                      </button>
                    {/if}
                  </div>
                </div>
              {/each}
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
    font-size: 10px;
    line-height: 1;
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
    transform: translateY(-50%);
    font-size: var(--font-size-sm);
    color: var(--color-accent-copper);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: translateY(-50%) rotate(0deg);
    }
    to {
      transform: translateY(-50%) rotate(360deg);
    }
  }

  .registry-loading {
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    font-style: italic;
    padding: var(--spacing-md) 0;
  }

  .registry-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
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
    font-size: 10px;
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-sm);
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
</style>
