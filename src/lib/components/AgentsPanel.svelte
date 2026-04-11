<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { stripMarkdown, truncateMarkdown } from "$lib/utils/format";
  import {
    getAgentStore,
    initAgents,
    addAgent,
    editAgent,
    removeAgent,
    assignAgentSkills,
    assignAgentMcp,
    searchAgentRegistries,
    installAgentFromRegistry,
    prefetchAgentRegistry,
  } from "$lib/stores/agents.svelte";
  import { getSkillStore, initSkills } from "$lib/stores/skills.svelte";
  import { getMcpState, initMcp } from "$lib/stores/mcp.svelte";
  import { getSourceStore, initSources } from "$lib/stores/sources.svelte";
  import { getSettings } from "$lib/stores/settings.svelte";
  import type { Agent } from "$lib/types/agent";
  import type { RegistryItem } from "$lib/types/registry";
  import { renderMarkdown, stripFrontmatter } from "$lib/utils/markdown";
  import { onMount, onDestroy } from "svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const agentStore = getAgentStore();
  const skillStore = getSkillStore();
  const mcpState = getMcpState();
  const sourceStore = getSourceStore();
  const settings = getSettings();

  // ── View state ──────────────────────────────────────────────

  type ViewState = { kind: "list" } | { kind: "form"; editing?: Agent };

  let view = $state<ViewState>({ kind: "list" });

  // ── Form state ──────────────────────────────────────────────

  let formName = $state("");
  let formAvatar = $state("");
  let formPrompt = $state("");
  let formSkillIds = new SvelteSet<string>();
  let formMcpIds = new SvelteSet<string>();
  let formSaving = $state(false);
  let formError = $state<string | null>(null);
  let formValid = $derived(formName.trim().length > 0 && formPrompt.trim().length > 0);
  let emojiPickerOpen = $state(false);

  const AGENT_EMOJIS = [
    "🤖",
    "🧠",
    "🔬",
    "💻",
    "📝",
    "🎯",
    "🔍",
    "💡",
    "🚀",
    "⚡",
    "🛡️",
    "🎨",
    "📊",
    "🧪",
    "🔧",
    "📚",
    "🌐",
    "🏗️",
    "🤝",
    "💬",
    "📎",
    "🔮",
    "🧩",
    "🎭",
  ];

  // ── Delete confirmation ─────────────────────────────────────

  let confirmDelete = $state<Agent | null>(null);
  let deleting = $state(false);

  // ── Registry state ──────────────────────────────────────────

  let registryExpanded = $state(false);
  let registrySearchInput = $state("");
  let registrySearchDebounce: ReturnType<typeof setTimeout> | null = null;
  let installingId = $state<string | null>(null);
  let installedId = $state<string | null>(null);
  const selectedSources = agentStore.selectedSourceIds;

  // Remove stale source IDs if source is disabled or deleted
  $effect(() => {
    for (const id of selectedSources) {
      if (id !== "aitmpl" && !sourceStore.sources.some((s) => s.enabled && s.id === id)) {
        selectedSources.delete(id);
      }
    }
  });

  /** Convert selection to array for the backend (empty = all). */
  function sourceIdsParam(): string[] | null {
    return selectedSources.size > 0 ? [...selectedSources] : null;
  }

  // Expand/collapse for individual agent cards
  let expandedAgentId = $state<string | null>(null);
  let expandedRegistryKey = $state<string | null>(null);

  // ── Derived ─────────────────────────────────────────────────

  let defaultAgent = $derived(agentStore.agents.find((a) => a.isDefault));
  let customAgents = $derived(agentStore.agents.filter((a) => !a.isDefault));
  let enabledSkills = $derived(skillStore.skills.filter((s) => s.enabled));

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (emojiPickerOpen && !target.closest(".avatar-picker")) {
      emojiPickerOpen = false;
    }
  }

  function handlePromptKeydown(event: KeyboardEvent) {
    if (event.key === "Tab") {
      event.preventDefault();
      const textarea = event.target as HTMLTextAreaElement;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      formPrompt = formPrompt.substring(0, start) + "  " + formPrompt.substring(end);
      requestAnimationFrame(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 2;
      });
    }
  }

  // ── Lifecycle ───────────────────────────────────────────────

  onMount(async () => {
    document.addEventListener("click", handleClickOutside);
    if (!agentStore.loaded) initAgents();
    if (!skillStore.loaded) initSkills();
    if (mcpState.servers.length === 0) initMcp();
    if (!sourceStore.loaded) initSources();
  });

  onDestroy(() => {
    document.removeEventListener("click", handleClickOutside);
    if (registrySearchDebounce) clearTimeout(registrySearchDebounce);
  });

  // ── Registry / Git Handlers ────────────────────────────────

  function handleRegistrySearch(query: string) {
    registrySearchInput = query;
    if (registrySearchDebounce) clearTimeout(registrySearchDebounce);
    if (!query.trim()) {
      prefetchAgentRegistry(sourceIdsParam());
      return;
    }
    registrySearchDebounce = setTimeout(() => {
      searchAgentRegistries(query.trim(), sourceIdsParam());
    }, 400);
  }

  function handleSourceFilterChange(value: string) {
    if (!value) {
      selectedSources.clear();
    } else if (selectedSources.has(value)) {
      selectedSources.delete(value);
    } else {
      selectedSources.add(value);
    }
    if (registrySearchInput.trim()) {
      searchAgentRegistries(registrySearchInput.trim(), sourceIdsParam());
    } else {
      prefetchAgentRegistry(sourceIdsParam());
    }
  }

  async function handleRegistryInstall(item: RegistryItem) {
    installingId = item.id;
    try {
      await installAgentFromRegistry(item);
      installedId = item.id;
    } catch {
      // Error logged in store
    } finally {
      installingId = null;
    }
  }

  function isAgentAlreadyInstalled(item: RegistryItem): boolean {
    if (item.source === "git" && item.id.startsWith("gsi-")) {
      // Extract file path from catalog ID: "gsi-{uuid}-{path}" → "{path}"
      // UUID is 36 chars, so path starts at index 4 + 36 + 1 = 41
      const path = item.id.substring(41);
      if (path) {
        return agentStore.agents.some((a) => a.sourceUrl?.endsWith(path) || a.name === item.name);
      }
    }
    return agentStore.agents.some((a) => a.sourceUrl?.includes(item.id) || a.name === item.name);
  }

  function registrySourceLabel(item: RegistryItem): string {
    if (item.source === "aitmpl") return "🌐 " + (item.sourceName ?? "aitmpl.com");
    return "🔀 " + (item.sourceName ?? "Git");
  }

  function registrySourceBadgeClass(item: RegistryItem): string {
    return item.source === "aitmpl" ? "badge badge--copper" : "badge badge--neutral";
  }

  // ── Handlers ────────────────────────────────────────────────

  function openCreateForm() {
    formName = "";
    formAvatar = "🤖";
    formPrompt = "";
    formSkillIds.clear();
    formMcpIds.clear();
    formError = null;
    view = { kind: "form" };
  }

  function openEditForm(agent: Agent) {
    formName = agent.name;
    formAvatar = agent.avatar ?? "🤖";
    formPrompt = agent.systemPrompt;
    formSkillIds.clear();
    formMcpIds.clear();
    formError = null;
    view = { kind: "form", editing: agent };
  }

  function cancelForm() {
    view = { kind: "list" };
  }

  async function handleSave() {
    const trimmedName = formName.trim();
    const trimmedPrompt = formPrompt.trim();

    if (!trimmedName) {
      formError = "Agent name is required.";
      return;
    }
    if (!trimmedPrompt) {
      formError = "System prompt is required.";
      return;
    }

    formSaving = true;
    formError = null;

    try {
      if (view.kind === "form" && view.editing) {
        const id = view.editing.id;
        await editAgent(id, trimmedName, trimmedPrompt, formAvatar || null);
        await assignAgentSkills(id, [...formSkillIds]);
        await assignAgentMcp(id, [...formMcpIds]);
      } else {
        const created = await addAgent(trimmedName, trimmedPrompt, formAvatar || null);
        await assignAgentSkills(created.id, [...formSkillIds]);
        await assignAgentMcp(created.id, [...formMcpIds]);
      }
      view = { kind: "list" };
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      formSaving = false;
    }
  }

  function requestDelete(agent: Agent) {
    confirmDelete = agent;
  }

  function toggleExpandAgent(id: string) {
    expandedAgentId = expandedAgentId === id ? null : id;
  }

  function registryKey(item: RegistryItem): string {
    return item.id + item.source + item.kind;
  }

  function toggleExpandRegistry(item: RegistryItem) {
    const key = registryKey(item);
    expandedRegistryKey = expandedRegistryKey === key ? null : key;
  }

  function cancelDelete() {
    confirmDelete = null;
  }

  async function confirmDeleteAgent() {
    if (!confirmDelete) return;
    deleting = true;
    try {
      await removeAgent(confirmDelete.id);
    } catch {
      // Error handled by store
    } finally {
      deleting = false;
      confirmDelete = null;
    }
  }

  function toggleSkill(skillId: string) {
    if (formSkillIds.has(skillId)) {
      formSkillIds.delete(skillId);
    } else {
      formSkillIds.add(skillId);
    }
  }

  function toggleMcp(serverId: string) {
    if (formMcpIds.has(serverId)) {
      formMcpIds.delete(serverId);
    } else {
      formMcpIds.add(serverId);
    }
  }

  const truncatePrompt = (text: string, max: number = 100) => truncateMarkdown(text, max);

  function sourceLabel(agent: Agent): string | null {
    if (agent.sourceType === "registry_aitmpl") return "registry";
    if (agent.sourceType === "git") return "git";
    return null;
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
</script>

<div class="panel">
  <!-- ── Header ──────────────────────────────────────────── -->
  <header class="panel-header" data-tauri-drag-region>
    <button
      class="panel-back"
      onclick={view.kind === "form" ? () => (view = { kind: "list" }) : onBack}
      aria-label="Go back"
    >
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
    <h2 class="panel-title">
      {#if view.kind === "form" && view.editing}
        Edit {view.editing.name}
      {:else if view.kind === "form"}
        Create Agent
      {:else}
        Agents
      {/if}
    </h2>
    {#if view.kind === "list"}
      <button class="btn" onclick={openCreateForm}>+ New Agent</button>
    {/if}
  </header>

  <!-- ── Content ─────────────────────────────────────────── -->
  <div class="panel-body">
    {#if !agentStore.loaded}
      <!-- Loading -->
      <div class="panel-loading">
        <span class="spinner"></span>
        Loading agents…
      </div>
    {:else if view.kind === "list"}
      <!-- ── List View ───────────────────────────────────── -->
      <div class="agents-list" role="list">
        <!-- Default agent -->
        {#if defaultAgent}
          <div
            class="card card--featured"
            role="listitem"
            ondblclick={() => toggleExpandAgent(defaultAgent.id)}
            title="Double-click to expand"
          >
            <div class="card-header">
              <button
                class="expand-btn"
                class:expanded={expandedAgentId === defaultAgent.id}
                onclick={(e: MouseEvent) => {
                  e.stopPropagation();
                  toggleExpandAgent(defaultAgent.id);
                }}
                aria-expanded={expandedAgentId === defaultAgent.id}
                aria-label={expandedAgentId === defaultAgent.id
                  ? "Collapse details"
                  : "Expand details"}>▶</button
              >
              <span class="card-icon">{defaultAgent.avatar ?? "🤖"}</span>
              <span class="card-title">{defaultAgent.name}</span>
              <span class="badge badge--copper">built-in</span>
            </div>
            {#if expandedAgentId !== defaultAgent.id}
              <p class="card-desc">{truncatePrompt(defaultAgent.systemPrompt)}</p>
            {/if}
            {#if expandedAgentId === defaultAgent.id}
              <div class="card-detail">
                <div class="detail-content-scroll markdown-prose">
                  {@html renderMarkdown(defaultAgent.systemPrompt)}
                </div>
              </div>
            {/if}
            <div class="card-meta">
              <span class="badge badge--neutral">Default for new conversations</span>
            </div>
          </div>
        {/if}

        <!-- Custom agents -->
        {#if customAgents.length > 0}
          <h2 class="section-heading">Custom Agents</h2>
          {#each customAgents as agent (agent.id)}
            <div
              class="card"
              role="listitem"
              ondblclick={() => toggleExpandAgent(agent.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
                <button
                  class="expand-btn"
                  class:expanded={expandedAgentId === agent.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandAgent(agent.id);
                  }}
                  aria-expanded={expandedAgentId === agent.id}
                  aria-label={expandedAgentId === agent.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <span class="card-icon">{agent.avatar ?? "🤖"}</span>
                <span class="card-title">{agent.name}</span>
                {#if sourceLabel(agent)}
                  <span class="badge badge--neutral">{sourceLabel(agent)}</span>
                {/if}
                <div class="card-actions">
                  <button
                    class="btn btn--sm"
                    onclick={() => openEditForm(agent)}
                    aria-label="Edit agent {agent.name}"
                  >
                    Edit
                  </button>
                  <button
                    class="btn btn--sm btn--danger"
                    onclick={() => requestDelete(agent)}
                    aria-label="Delete agent {agent.name}"
                  >
                    Delete
                  </button>
                </div>
              </div>
              {#if expandedAgentId !== agent.id}
                <p class="card-desc">{truncatePrompt(agent.systemPrompt)}</p>
              {/if}
              {#if expandedAgentId === agent.id}
                <div class="card-detail">
                  <div class="detail-content-scroll markdown-prose">
                    {@html renderMarkdown(agent.systemPrompt)}
                  </div>
                  {#if agent.sourceUrl}
                    <div class="detail-row">
                      <span class="detail-label">Source:</span>
                      <a
                        href={agent.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="detail-link">{agent.sourceUrl}</a
                      >
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        {:else if !defaultAgent}
          <p class="section-empty">No agents configured. Create one to get started.</p>
        {/if}
      </div>

      <!-- ── Registry Browser ────────────────────────────────── -->
      <section class="catalog-section">
        <button
          class="collapsible-heading"
          onclick={() => (registryExpanded = !registryExpanded)}
          aria-expanded={registryExpanded}
        >
          <span class="collapse-arrow" class:expanded={registryExpanded}>▶</span>
          <h3 class="section-heading inline">Browse Agent Catalog</h3>
        </button>

        {#if registryExpanded}
          <div class="section-content">
            <div class="search-row">
              <input
                class="form-input"
                type="search"
                placeholder="Search agents…"
                value={registrySearchInput}
                oninput={(e) => handleRegistrySearch(e.currentTarget.value)}
              />
              {#if agentStore.registrySearching}
                <span class="search-spinner">⟳</span>
              {/if}
            </div>

            <div class="source-pills" role="group" aria-label="Filter by source">
              <button
                class="source-pill"
                class:active={selectedSources.size === 0}
                onclick={() => handleSourceFilterChange("")}
                aria-pressed={selectedSources.size === 0}>All</button
              >
              {#if settings.aitmplEnabled}
                <button
                  class="source-pill"
                  class:active={selectedSources.has("aitmpl")}
                  onclick={() => handleSourceFilterChange("aitmpl")}
                  aria-pressed={selectedSources.has("aitmpl")}>aitmpl.com</button
                >
              {/if}
              {#each sourceStore.sources.filter((s) => s.enabled) as src (src.id)}
                <button
                  class="source-pill"
                  class:active={selectedSources.has(src.id)}
                  onclick={() => handleSourceFilterChange(src.id)}
                  aria-pressed={selectedSources.has(src.id)}>{src.name}</button
                >
              {/each}
            </div>

            {#if agentStore.registrySearching && agentStore.registryResults.length === 0}
              <div class="registry-loading">
                <span class="spinner"></span> Searching registries…
              </div>
            {:else if agentStore.registryResults.length > 0}
              <div class="registry-results" role="list">
                {#each agentStore.registryResults as item (item.id + item.source + item.kind)}
                  <article
                    class="card registry-card"
                    role="listitem"
                    ondblclick={() => toggleExpandRegistry(item)}
                    title="Double-click to expand"
                  >
                    <div class="card-header">
                      <button
                        class="expand-btn"
                        class:expanded={expandedRegistryKey === registryKey(item)}
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          toggleExpandRegistry(item);
                        }}
                        aria-expanded={expandedRegistryKey === registryKey(item)}
                        aria-label={expandedRegistryKey === registryKey(item)
                          ? "Collapse"
                          : "Expand"}>▶</button
                      >
                      <strong class="card-title">{item.name}</strong>
                      <span class={registrySourceBadgeClass(item)}>{registrySourceLabel(item)}</span
                      >
                      {#if item.url}
                        <a
                          href={item.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="source-link"
                          aria-label="View on {registrySourceLabel(item)}"
                        >
                          ↗
                        </a>
                      {/if}
                    </div>
                    {#if expandedRegistryKey !== registryKey(item) && item.description}
                      <p class="card-desc">{stripMarkdown(item.description)}</p>
                    {/if}
                    {#if expandedRegistryKey === registryKey(item)}
                      <div class="card-detail detail-content-scroll markdown-prose">
                        {@html renderMarkdown(
                          stripFrontmatter(item.content ?? item.description ?? ""),
                        )}
                      </div>
                    {/if}
                    <div class="card-actions">
                      {#if isAgentAlreadyInstalled(item)}
                        <span class="badge badge--success">✓ Installed</span>
                      {:else if installedId === item.id}
                        <span class="badge badge--success">✓ Installed</span>
                      {:else}
                        <button
                          class="btn btn--primary"
                          onclick={() => handleRegistryInstall(item)}
                          disabled={installingId === item.id}
                        >
                          {installingId === item.id ? "Installing…" : "Install"}
                        </button>
                      {/if}
                    </div>
                  </article>
                {/each}
              </div>
            {:else if registrySearchInput.trim() && !agentStore.registrySearching}
              <p class="section-empty">No agent templates found.</p>
            {/if}
          </div>
        {/if}
      </section>

      <!-- ── Delete confirmation overlay ─────────────────── -->
      <ConfirmDialog
        open={confirmDelete !== null}
        title="Delete agent '{confirmDelete?.name ?? ''}'?"
        detail="Conversations using this agent will keep their history but use the default agent."
        loading={deleting}
        onconfirm={confirmDeleteAgent}
        oncancel={cancelDelete}
      />
    {:else if view.kind === "form"}
      <!-- ── Create / Edit Form ──────────────────────────── -->
      <div class="agent-form">
        {#if formError}
          <div class="form-error" role="alert">{formError}</div>
        {/if}

        <!-- Avatar + Name row -->
        <div class="form-row">
          <div class="form-field avatar-field">
            <label class="form-label form-label--caps" for="agent-avatar">Avatar</label>
            <div class="avatar-picker">
              <button
                id="agent-avatar"
                type="button"
                class="avatar-trigger"
                onclick={() => (emojiPickerOpen = !emojiPickerOpen)}
                aria-haspopup="listbox"
                aria-expanded={emojiPickerOpen}
              >
                <span class="avatar-preview">{formAvatar || "🤖"}</span>
                <span class="avatar-caret">▾</span>
              </button>
              {#if emojiPickerOpen}
                <div class="avatar-dropdown" role="listbox" aria-label="Choose avatar emoji">
                  {#each AGENT_EMOJIS as emoji (emoji)}
                    <button
                      type="button"
                      class="avatar-option"
                      class:selected={formAvatar === emoji}
                      role="option"
                      aria-selected={formAvatar === emoji}
                      onclick={() => {
                        formAvatar = emoji;
                        emojiPickerOpen = false;
                      }}
                    >
                      {emoji}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
          <div class="form-field name-field">
            <label class="form-label form-label--caps" for="agent-name">Name</label>
            <input
              id="agent-name"
              class="form-input"
              type="text"
              bind:value={formName}
              placeholder="e.g. Research Agent"
            />
          </div>
        </div>

        <!-- System Prompt -->
        <div class="form-field">
          <label class="form-label form-label--caps" for="agent-prompt">
            System Prompt
            <span class="form-hint">Markdown supported</span>
          </label>
          <textarea
            id="agent-prompt"
            class="form-input form-input--mono agent-prompt-textarea"
            rows={8}
            bind:value={formPrompt}
            onkeydown={handlePromptKeydown}
            placeholder="Describe how this agent should behave, what it specialises in, and any rules it should follow…"
          ></textarea>
        </div>

        <!-- Skills assignment -->
        {#if enabledSkills.length > 0}
          <fieldset class="form-fieldset">
            <legend class="form-legend">Assigned Skills</legend>
            <div class="check-list" role="group" aria-label="Skills">
              {#each enabledSkills as skill (skill.id)}
                <label class="check-item">
                  <input
                    type="checkbox"
                    checked={formSkillIds.has(skill.id)}
                    onchange={() => toggleSkill(skill.id)}
                  />
                  <span class="check-item-content">
                    <span class="check-item-label">{skill.name}</span>
                    <span class="badge badge--neutral">{skill.source}</span>
                  </span>
                </label>
              {/each}
            </div>
          </fieldset>
        {/if}

        <!-- MCP connections assignment -->
        {#if mcpState.servers.length > 0}
          <fieldset class="form-fieldset">
            <legend class="form-legend">MCP Connections</legend>
            <div class="check-list" role="group" aria-label="MCP servers">
              {#each mcpState.servers as server (server.config.id)}
                <label class="check-item">
                  <input
                    type="checkbox"
                    checked={formMcpIds.has(server.config.id)}
                    onchange={() => toggleMcp(server.config.id)}
                  />
                  <span class="check-item-content">
                    <span class="check-item-status" title={server.status}>
                      {statusIcon(server.status)}
                    </span>
                    <span class="check-item-label">{server.config.name}</span>
                    <span class="badge badge--neutral">{server.config.transport}</span>
                  </span>
                </label>
              {/each}
            </div>
          </fieldset>
        {/if}

        <!-- Form actions -->
        <div class="form-actions">
          <button class="btn" onclick={cancelForm} disabled={formSaving}>Cancel</button>
          <button class="btn btn--primary" onclick={handleSave} disabled={formSaving || !formValid}>
            {#if formSaving}
              <span class="spinner spinner--sm"></span> Saving…
            {:else}
              {view.editing ? "Save Changes" : "Create Agent"}
            {/if}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* ── Agent list layout ── */

  .agents-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  /* ── Form layout ── */

  .agent-form {
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
    animation: fadeInUp 200ms ease;
  }

  .form-row {
    display: flex;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
    flex-wrap: wrap;
  }

  @media (max-width: 400px) {
    .form-row {
      flex-direction: column;
    }
  }

  .form-label--caps {
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .avatar-field {
    flex-shrink: 0;
  }

  .name-field {
    flex: 1;
    min-width: 0;
  }

  .avatar-picker {
    position: relative;
  }

  .avatar-trigger {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xs) var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    cursor: pointer;
    transition: border-color var(--transition-fast);
  }
  .avatar-trigger:hover {
    border-color: var(--color-border-focus);
  }
  .avatar-trigger:focus-visible {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .avatar-preview {
    font-size: var(--font-size-xl);
    line-height: 1;
  }

  .avatar-caret {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .avatar-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 20;
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 2px;
    padding: var(--spacing-xs);
    margin-top: var(--spacing-xs);
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
    animation: fadeIn 100ms ease;
    max-width: 220px;
  }

  .avatar-option {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    font-size: 1.2rem;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .avatar-option:hover {
    background: var(--color-bg-tertiary);
  }
  .avatar-option.selected {
    background: color-mix(in srgb, var(--color-accent-copper) 15%, transparent);
    outline: 2px solid var(--color-accent-copper);
  }

  .agent-prompt-textarea {
    min-height: 180px;
  }

  /* ── Fieldsets (Skills / MCP) ── */

  .form-fieldset {
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    margin: 0 0 var(--spacing-md) 0;
  }

  .form-legend {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-tertiary);
    padding: 0 var(--spacing-xs);
  }

  .check-item-content {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    min-width: 0;
    flex: 1;
  }

  .check-item-status {
    font-size: var(--font-size-2xs);
    flex-shrink: 0;
  }
</style>
