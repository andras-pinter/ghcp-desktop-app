<script lang="ts">
  import { stripMarkdown } from "$lib/utils/format";
  import {
    getMcpState,
    initMcp,
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
  import type { McpConnectionInfo, RegistryServer } from "$lib/types/mcp";
  import McpServerForm from "./McpServerForm.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
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
      };

  let view = $state<ViewState>({ kind: "list" });

  // ── Local state ─────────────────────────────────────────────

  let testingServer = $state<string | null>(null);
  let testResult = $state<{ serverId: string; success: boolean; message: string } | null>(null);
  let expandedServerId = $state<string | null>(null);
  let expandedRegistryName = $state<string | null>(null);
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

  function toggleExpandRegistry(entry: RegistryServer) {
    expandedRegistryName = expandedRegistryName === entry.name ? null : entry.name;
  }

  function returnToList() {
    view = { kind: "list" };
    expandedRegistryName = null;
  }

  async function handleConnect(serverId: string, isRetry = false) {
    try {
      await connectServer(serverId);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.startsWith("BINARY_NOT_APPROVED:") && !isRetry) {
        const binaryPath = msg.slice("BINARY_NOT_APPROVED:".length);
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
        const binaryPath = msg.slice("BINARY_NOT_APPROVED:".length);
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

  let pendingRemoveId: string | null = $state(null);

  function handleRemove(serverId: string) {
    pendingRemoveId = serverId;
  }

  async function confirmRemove() {
    if (!pendingRemoveId) return;
    const id = pendingRemoveId;
    pendingRemoveId = null;
    try {
      await removeServer(id);
    } catch {
      // Error is in the store
    }
  }

  function cancelRemove() {
    pendingRemoveId = null;
  }

  async function toggleExpandServer(serverId: string) {
    if (expandedServerId === serverId) {
      expandedServerId = null;
    } else {
      await loadTools(serverId);
      expandedServerId = serverId;
    }
  }

  function isRegistryAdded(entry: RegistryServer): boolean {
    return mcp.servers.some(
      (s) =>
        s.config.name === entry.name ||
        (entry.remotes[0]?.url && s.config.url === entry.remotes[0].url),
    );
  }

  function transportDesc(info: McpConnectionInfo): string {
    const t = info.config.transport.toUpperCase();
    if (info.config.transport === "http" && info.config.url) return `${t} · ${info.config.url}`;
    if (info.config.transport === "stdio" && info.config.binaryPath)
      return `${t} · ${info.config.binaryPath}`;
    return t;
  }

  onDestroy(() => {
    if (searchDebounce) clearTimeout(searchDebounce);
  });
</script>

{#if view.kind === "form"}
  <div class="panel">
    <header class="panel-header" data-tauri-drag-region>
      <button class="panel-back" onclick={returnToList} aria-label="Go back">
        <svg width="18" height="18" viewBox="0 0 16 16" fill="none">
          <path
            d="M10 3L5 8l5 5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <h2 class="panel-title">MCP Servers</h2>
    </header>
    <McpServerForm
      editInfo={view.editInfo ?? null}
      registryEntry={view.registryEntry ?? null}
      onBack={returnToList}
    />
  </div>
{:else}
  <div class="panel">
    <!-- Header -->
    <header class="panel-header" data-tauri-drag-region>
      <button class="panel-back" onclick={onBack} aria-label="Go back">
        <svg width="18" height="18" viewBox="0 0 16 16" fill="none">
          <path
            d="M10 3L5 8l5 5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <h2 class="panel-title">MCP Servers</h2>
      <button class="btn" onclick={openAddForm}>+ New Server</button>
    </header>

    <div class="panel-body">
      {#if mcp.loading}
        <div class="panel-loading">Loading MCP servers...</div>
      {:else}
        <!-- Configured Servers -->
        <section class="panel-section">
          <h3 class="section-heading">Configured Servers</h3>

          {#if mcp.servers.length === 0}
            <p class="section-empty">
              No MCP servers configured yet. Browse the registry below or add a custom server.
            </p>
          {/if}

          {#each mcp.servers as info (info.config.id)}
            <article
              class="card"
              role="listitem"
              ondblclick={() => toggleExpandServer(info.config.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
                <button
                  class="expand-btn"
                  class:expanded={expandedServerId === info.config.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandServer(info.config.id);
                  }}
                  aria-label={expandedServerId === info.config.id
                    ? "Collapse details"
                    : "Expand details"}>▶</button
                >
                <span
                  class="status {info.status === 'connected'
                    ? 'status--connected'
                    : info.status === 'connecting'
                      ? 'status--warning'
                      : 'status--disconnected'}"
                  title={info.status}
                  role="status"
                  aria-label={`Server ${info.status}`}><span class="status-dot"></span></span
                >
                <strong class="card-title">{info.config.name}</strong>
                <span class="badge badge--neutral">{info.config.transport.toUpperCase()}</span>
                <div class="card-actions">
                  <button
                    class="btn btn--sm"
                    onclick={() => handleTest(info)}
                    disabled={testingServer === info.config.id}
                  >
                    {testingServer === info.config.id ? "Testing…" : "Test"}
                  </button>
                  {#if info.status === "connected"}
                    <button class="btn btn--sm" onclick={() => handleDisconnect(info.config.id)}>
                      Disconnect
                    </button>
                  {:else if info.status === "connecting"}
                    <button class="btn btn--sm" disabled>Connecting…</button>
                  {:else}
                    <button class="btn btn--sm" onclick={() => handleConnect(info.config.id)}>
                      Connect
                    </button>
                  {/if}
                  <button class="btn btn--sm" onclick={() => openEditForm(info)}>Edit</button>
                  <button
                    class="btn btn--sm btn--danger"
                    onclick={() => handleRemove(info.config.id)}
                  >
                    Remove
                  </button>
                </div>
              </div>

              {#if expandedServerId !== info.config.id}
                <p class="card-desc">{transportDesc(info)}</p>
              {/if}

              {#if info.error}
                <div class="banner banner--error server-error">⚠ {info.error}</div>
              {/if}

              {#if testResult && testResult.serverId === info.config.id}
                <div
                  class="banner test-result {testResult.success
                    ? 'banner--success'
                    : 'banner--error'}"
                >
                  {testResult.success ? "✓" : "✗"}
                  {testResult.message}
                </div>
              {/if}

              {#if expandedServerId === info.config.id}
                <div class="card-detail">
                  <div class="card-meta">
                    <span class="badge badge--neutral"
                      >Transport: {info.config.transport.toUpperCase()}</span
                    >
                    {#if info.config.transport === "http" && info.config.url}
                      <span class="badge badge--neutral url-tag" title={info.config.url}>
                        {info.config.url}
                      </span>
                    {/if}
                    {#if info.config.transport === "stdio" && info.config.binaryPath}
                      <span class="badge badge--neutral">Binary: {info.config.binaryPath}</span>
                    {/if}
                    <span class="badge badge--neutral">
                      Tools: {info.toolCount}
                    </span>
                  </div>

                  {#if info.status === "connected" && info.tools && info.tools.length > 0}
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
              {/if}
            </article>
          {/each}
        </section>

        <!-- MCP Registry -->
        <section class="panel-section">
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

          <div class="search-row">
            <input
              type="text"
              value={registrySearch}
              oninput={(e) => handleSearchInput(e.currentTarget.value)}
              placeholder="Search servers (e.g. azure, github, postgres)..."
              class="form-input"
            />
            {#if mcp.registryLoading}
              <span class="search-spinner" role="status" aria-label="Searching"></span>
            {/if}
          </div>

          {#if mcp.registryLoading && mcp.registry.length === 0}
            <div class="registry-loading">
              <span class="spinner spinner--sm"></span> Fetching from registry...
            </div>
          {:else if mcp.registry.length > 0}
            <div class="registry-results" role="list">
              {#each mcp.registry as entry, i (entry.name + "-" + i)}
                {@const isExpanded = expandedRegistryName === entry.name}
                {@const isAdded = isRegistryAdded(entry)}
                <article
                  class="card registry-card"
                  role="listitem"
                  ondblclick={() => toggleExpandRegistry(entry)}
                  title="Double-click to expand"
                >
                  <div class="card-header">
                    <button
                      class="expand-btn"
                      class:expanded={isExpanded}
                      onclick={(e: MouseEvent) => {
                        e.stopPropagation();
                        toggleExpandRegistry(entry);
                      }}
                      aria-label={isExpanded ? "Collapse" : "Expand"}>▶</button
                    >
                    <strong class="card-title">{entry.displayName}</strong>
                    {#if entry.isStdioOnly}
                      <span class="badge badge--mono">STDIO</span>
                    {:else}
                      <span class="badge badge--copper">HTTP</span>
                    {/if}
                    <span class="badge badge--neutral">{entry.name}</span>
                  </div>
                  {#if !isExpanded && entry.description}
                    <p class="card-desc">{stripMarkdown(entry.description)}</p>
                  {/if}
                  {#if isExpanded}
                    <div class="card-detail">
                      {#if entry.description}
                        <p class="detail-description">{entry.description}</p>
                      {/if}

                      {#if entry.version}
                        <div class="detail-meta">
                          <span class="badge badge--copper">v{entry.version}</span>
                        </div>
                      {/if}

                      {#if entry.remotes.length > 0}
                        <div class="detail-section">
                          <h4 class="detail-label">Remote Endpoints</h4>
                          {#each entry.remotes as remote (remote.url ?? remote.transportType)}
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
                        </div>
                      {/if}

                      {#if entry.packages.length > 0}
                        <div class="detail-section">
                          <h4 class="detail-label">Packages</h4>
                          {#each entry.packages as pkg (pkg.identifier)}
                            <div class="detail-card">
                              <span class="detail-label">{pkg.registryType}</span>
                              <code class="detail-code"
                                >{pkg.identifier}{pkg.version ? `@${pkg.version}` : ""}</code
                              >
                            </div>
                          {/each}
                        </div>
                      {/if}

                      {#if entry.isStdioOnly && entry.packages.length > 0}
                        <div class="detail-section">
                          <h4 class="detail-label">Run Command</h4>
                          <div class="setup-commands">
                            {#each entry.packages as pkg (pkg.identifier + "-cmd")}
                              {#if pkg.registryType === "npm"}
                                {@const cmd = `npx -y ${pkg.identifier}${pkg.version ? `@${pkg.version}` : ""}`}
                                <div class="code-block">
                                  <code>{cmd}</code>
                                  <button
                                    class="code-block-copy"
                                    onclick={() => copyCommand(cmd)}
                                    aria-label="Copy command"
                                    >{copiedCommand === cmd ? "Copied!" : "Copy"}</button
                                  >
                                </div>
                              {:else if pkg.registryType === "pypi"}
                                {@const cmd = `uvx ${pkg.identifier}${pkg.version ? `==${pkg.version}` : ""}`}
                                <div class="code-block">
                                  <code>{cmd}</code>
                                  <button
                                    class="code-block-copy"
                                    onclick={() => copyCommand(cmd)}
                                    aria-label="Copy command"
                                    >{copiedCommand === cmd ? "Copied!" : "Copy"}</button
                                  >
                                </div>
                              {:else if pkg.registryType === "nuget"}
                                {@const cmd = `dotnet tool run ${pkg.identifier}`}
                                <div class="code-block">
                                  <code>{cmd}</code>
                                  <button
                                    class="code-block-copy"
                                    onclick={() => copyCommand(cmd)}
                                    aria-label="Copy command"
                                    >{copiedCommand === cmd ? "Copied!" : "Copy"}</button
                                  >
                                </div>
                              {/if}
                            {/each}
                          </div>
                        </div>
                      {/if}

                      {#if entry.repoUrl || entry.websiteUrl}
                        <div class="detail-links">
                          {#if entry.repoUrl}
                            <a
                              href={entry.repoUrl}
                              target="_blank"
                              rel="noopener"
                              class="detail-link">📦 Repository</a
                            >
                          {/if}
                          {#if entry.websiteUrl}
                            <a
                              href={entry.websiteUrl}
                              target="_blank"
                              rel="noopener"
                              class="detail-link">🌐 Website</a
                            >
                          {/if}
                        </div>
                      {/if}
                    </div>
                  {/if}
                  <div class="card-actions">
                    {#if isAdded}
                      <span class="badge badge--success">✓ Added</span>
                    {:else}
                      <button
                        class="btn btn--primary"
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          openRegistryForm(entry);
                        }}
                      >
                        Add & Configure
                      </button>
                    {/if}
                  </div>
                </article>
              {/each}
              {#if mcp.hasMoreRegistry}
                <button
                  class="load-more-btn"
                  onclick={loadRegistryMore}
                  disabled={mcp.registryLoading}
                  type="button"
                >
                  {#if mcp.registryLoading}
                    <span class="spinner spinner--sm"></span> Loading…
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

<ConfirmDialog
  open={pendingRemoveId !== null}
  title="Remove this MCP server?"
  detail="This cannot be undone."
  confirmLabel="Remove"
  onconfirm={confirmRemove}
  oncancel={cancelRemove}
/>

<style>
  /* ── MCP-specific layout overrides ── */

  /* ── Server card overrides ── */

  .card-header {
    flex-wrap: wrap;
  }

  .server-error {
    margin-top: var(--spacing-xs);
    font-size: var(--font-size-xs);
  }

  .test-result {
    margin-top: var(--spacing-xs);
    font-size: var(--font-size-xs);
  }

  .url-tag {
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
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

  /* ── MCP Registry ── */

  .registry-count {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-normal);
    color: var(--color-text-tertiary);
    text-transform: none;
    letter-spacing: 0;
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

  /* ── Registry Detail (inline expanded) ── */

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

  .detail-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
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

  .code-block-copy {
    width: auto;
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
  }
  .code-block-copy:hover {
    color: var(--color-accent-copper);
    background: color-mix(in srgb, var(--color-accent-copper) 8%, transparent);
  }
</style>
