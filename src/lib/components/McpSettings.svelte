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
    editServer,
  } from "$lib/stores/mcp.svelte";
  import type { McpServerConfig, McpConnectionInfo, CatalogEntry } from "$lib/types/mcp";
  import { onMount } from "svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const mcp = getMcpState();

  // ── Local state ─────────────────────────────────────────────

  let showAddForm = $state(false);
  let editingServer = $state<string | null>(null);
  let testingServer = $state<string | null>(null);
  let testResult = $state<{ serverId: string; success: boolean; message: string } | null>(null);
  let expandedTools = $state<string | null>(null);

  // Form fields
  let formName = $state("");
  let formTransport = $state<"http" | "stdio">("http");
  let formUrl = $state("");
  let formBinaryPath = $state("");
  let formArgs = $state("");
  let formAuthHeader = $state("");
  let formError = $state("");

  const argsPlaceholder = '["--port", "3000"]';

  onMount(() => {
    initMcp();
  });

  // ── Form helpers ────────────────────────────────────────────

  function resetForm() {
    formName = "";
    formTransport = "http";
    formUrl = "";
    formBinaryPath = "";
    formArgs = "";
    formAuthHeader = "";
    formError = "";
    showAddForm = false;
    editingServer = null;
  }

  function startEdit(info: McpConnectionInfo) {
    editingServer = info.config.id;
    formName = info.config.name;
    formTransport = info.config.transport;
    formUrl = info.config.url ?? "";
    formBinaryPath = info.config.binaryPath ?? "";
    formArgs = info.config.args ?? "";
    formAuthHeader = info.config.authHeader ?? "";
    showAddForm = true;
  }

  function fillFromCatalog(entry: CatalogEntry) {
    formName = entry.name;
    formTransport = entry.transport;
    formUrl = entry.defaultUrl ?? "";
    formBinaryPath = entry.defaultBinary ?? "";
    formArgs = "";
    formAuthHeader = "";
    showAddForm = true;
    editingServer = null;
  }

  async function submitForm() {
    formError = "";

    // Validate name
    if (!formName.trim()) {
      formError = "Server name is required";
      return;
    }

    // Validate HTTP URL
    if (formTransport === "http") {
      if (!formUrl.trim()) {
        formError = "URL is required for HTTP transport";
        return;
      }
      try {
        const parsed = new URL(formUrl);
        if (!["http:", "https:"].includes(parsed.protocol)) {
          formError = "Only HTTP and HTTPS URLs are allowed";
          return;
        }
      } catch {
        formError = "Invalid URL format";
        return;
      }
    }

    // Validate stdio binary path
    if (formTransport === "stdio") {
      if (!formBinaryPath.trim()) {
        formError = "Binary path is required for stdio transport";
        return;
      }
      if (!formBinaryPath.startsWith("/")) {
        formError = "Binary path must be absolute (start with /)";
        return;
      }
    }

    // Validate args JSON if provided
    if (formArgs.trim()) {
      try {
        const parsed = JSON.parse(formArgs);
        if (!Array.isArray(parsed)) {
          formError = "Arguments must be a JSON array of strings";
          return;
        }
      } catch {
        formError = 'Arguments must be valid JSON (e.g. ["--port", "3000"])';
        return;
      }
    }

    const config: McpServerConfig = {
      id: editingServer ?? `mcp-${Date.now()}`,
      name: formName,
      transport: formTransport,
      url: formTransport === "http" ? formUrl || null : null,
      binaryPath: formTransport === "stdio" ? formBinaryPath || null : null,
      args: formTransport === "stdio" && formArgs ? formArgs : null,
      authHeader: formAuthHeader || null,
      fromCatalog: false,
      enabled: true,
    };

    if (editingServer) {
      await editServer(config);
    } else {
      await addServer(config);
    }
    resetForm();
  }

  async function handleConnect(serverId: string) {
    try {
      await connectServer(serverId);
    } catch {
      // Error is already in the store
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
        message: `Connected successfully — ${count} tool${count !== 1 ? "s" : ""} discovered`,
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

  // Check if a catalog entry is already added
  function isCatalogAdded(entry: CatalogEntry): boolean {
    return mcp.servers.some((s) => s.config.id === entry.id || s.config.name === entry.name);
  }
</script>

<div class="mcp-settings">
  <!-- Header -->
  <header class="mcp-header">
    <button class="back-btn" onclick={onBack} aria-label="Go back">← Back</button>
    <h2 class="mcp-title">MCP Servers</h2>
  </header>

  <div class="mcp-content">
    {#if mcp.loading}
      <div class="mcp-loading">Loading MCP servers...</div>
    {:else}
      <!-- Connected Servers -->
      <section class="mcp-section">
        <h3 class="section-heading">Configured Servers</h3>

        {#if mcp.servers.length === 0}
          <p class="section-empty">
            No MCP servers configured. Add one below or pick from the catalog.
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
                <button class="action-btn" onclick={() => startEdit(info)}>Edit</button>
                <button class="action-btn danger" onclick={() => handleRemove(info.config.id)}>
                  Remove
                </button>
              </div>
            </div>

            <div class="server-meta">
              <span class="meta-tag">Transport: {info.config.transport.toUpperCase()}</span>
              {#if info.config.transport === "http" && info.config.url}
                <span class="meta-tag">URL: {info.config.url}</span>
              {/if}
              {#if info.config.transport === "stdio" && info.config.binaryPath}
                <span class="meta-tag">Binary: {info.config.binaryPath}</span>
              {/if}
              <span class="meta-tag">
                Tools: {info.toolCount} discovered
                {#if info.status === "connected" && info.toolCount > 0}
                  <button class="tools-toggle" onclick={() => toggleTools(info.config.id)}>
                    {expandedTools === info.config.id ? "▼ Hide" : "▶ Show"}
                  </button>
                {/if}
              </span>
              {#if info.config.fromCatalog}
                <span class="meta-tag catalog-tag">catalog</span>
              {/if}
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

      <!-- Catalog -->
      <section class="mcp-section">
        <h3 class="section-heading">Catalog</h3>
        <div class="catalog-list">
          {#each mcp.catalog as entry (entry.id)}
            <div class="catalog-entry">
              <div class="catalog-info">
                <strong>{entry.name}</strong>
                <span class="catalog-transport">{entry.transport.toUpperCase()}</span>
              </div>
              <p class="catalog-desc">{entry.description}</p>
              {#if isCatalogAdded(entry)}
                <span class="catalog-added">Added</span>
              {:else}
                <button class="action-btn" onclick={() => fillFromCatalog(entry)}>Add</button>
              {/if}
            </div>
          {/each}
        </div>
      </section>

      <!-- Add / Edit Form -->
      {#if showAddForm}
        <section class="mcp-section">
          <h3 class="section-heading">{editingServer ? "Edit Server" : "Add Custom Server"}</h3>
          <form
            class="add-form"
            onsubmit={(e) => {
              e.preventDefault();
              submitForm();
            }}
          >
            <label class="form-field">
              <span class="field-label">Name</span>
              <input type="text" bind:value={formName} placeholder="My MCP Server" required />
            </label>

            <fieldset class="form-field">
              <legend class="field-label">Transport</legend>
              <div class="radio-group">
                <label>
                  <input type="radio" bind:group={formTransport} value="http" /> HTTP
                </label>
                <label>
                  <input type="radio" bind:group={formTransport} value="stdio" /> Stdio
                </label>
              </div>
            </fieldset>

            {#if formTransport === "http"}
              <label class="form-field">
                <span class="field-label">URL</span>
                <input
                  type="url"
                  bind:value={formUrl}
                  placeholder="https://example.com/mcp"
                  required
                />
              </label>
              <label class="form-field">
                <span class="field-label">Auth Header (optional)</span>
                <input type="text" bind:value={formAuthHeader} placeholder="Bearer your-token" />
              </label>
            {:else}
              <label class="form-field">
                <span class="field-label">Binary Path</span>
                <input
                  type="text"
                  bind:value={formBinaryPath}
                  placeholder="/usr/local/bin/mcp-server"
                  required
                />
              </label>
              <label class="form-field">
                <span class="field-label">Arguments (JSON array, optional)</span>
                <input type="text" bind:value={formArgs} placeholder={argsPlaceholder} />
              </label>
            {/if}

            {#if formError}
              <div class="form-error">{formError}</div>
            {/if}

            <div class="form-actions">
              <button type="button" class="action-btn" onclick={resetForm}>Cancel</button>
              <button type="submit" class="action-btn primary">
                {editingServer ? "Save Changes" : "Add Server"}
              </button>
            </div>
          </form>
        </section>
      {:else}
        <button class="add-custom-btn" onclick={() => (showAddForm = true)}>
          + Add Custom Server
        </button>
      {/if}
    {/if}
  </div>
</div>

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

  .catalog-tag {
    color: var(--color-accent-copper);
    font-weight: var(--font-weight-medium);
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
  .action-btn.primary {
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
    border-color: var(--color-text-primary);
  }
  .action-btn.primary:hover {
    opacity: 0.9;
  }
  .action-btn.danger:hover {
    color: var(--color-error);
    border-color: var(--color-error);
  }

  /* ── Catalog ── */

  .catalog-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .catalog-entry {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
  }

  .catalog-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    flex: 1;
    min-width: 0;
  }
  .catalog-info strong {
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }
  .catalog-transport {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    background: var(--color-bg-tertiary);
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-sm);
  }

  .catalog-desc {
    width: 100%;
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: 0;
  }

  .catalog-added {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  /* ── Add Form ── */

  .add-custom-btn {
    display: block;
    width: 100%;
    padding: var(--spacing-md);
    border: 2px dashed var(--color-border-primary);
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: all var(--transition-fast);
  }
  .add-custom-btn:hover {
    border-color: var(--color-accent-copper);
    color: var(--color-accent-copper);
  }

  .add-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    border: none;
    padding: 0;
    margin: 0;
  }

  .field-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
  }

  .form-field input[type="text"],
  .form-field input[type="url"] {
    padding: var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
  }
  .form-field input:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .radio-group {
    display: flex;
    gap: var(--spacing-md);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }
  .radio-group label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    cursor: pointer;
  }

  .form-error {
    color: var(--color-danger, #dc2626);
    font-size: var(--font-size-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: color-mix(in srgb, var(--color-danger, #dc2626) 8%, transparent);
    border-radius: var(--radius-sm);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
  }
</style>
