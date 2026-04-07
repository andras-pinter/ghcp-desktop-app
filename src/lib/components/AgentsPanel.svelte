<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
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
    clearAgentRegistrySearch,
    discoverGitAgents,
    importAgentFromGit,
    updateAgentGitProgress,
  } from "$lib/stores/agents.svelte";
  import { getSkillStore, initSkills } from "$lib/stores/skills.svelte";
  import { getMcpState, initMcp } from "$lib/stores/mcp.svelte";
  import type { Agent } from "$lib/types/agent";
  import type { RegistryItem, GitSkillFile } from "$lib/types/registry";
  import { renderMarkdown, stripFrontmatter } from "$lib/utils/markdown";
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const agentStore = getAgentStore();
  const skillStore = getSkillStore();
  const mcpState = getMcpState();

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

  // ── Registry / Git state ──────────────────────────────────────

  let registryExpanded = $state(true);
  let gitExpanded = $state(false);
  let registrySearchInput = $state("");
  let registrySearchDebounce: ReturnType<typeof setTimeout> | null = null;
  let gitUrl = $state("");
  let gitError = $state<string | null>(null);
  let installingId = $state<string | null>(null);
  let installedId = $state<string | null>(null);
  let importingPath = $state<string | null>(null);
  let importedPath = $state<string | null>(null);

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

  let unlistenProgress: UnlistenFn | null = null;

  onMount(async () => {
    document.addEventListener("click", handleClickOutside);
    if (!agentStore.loaded) initAgents();
    if (!skillStore.loaded) initSkills();
    if (mcpState.servers.length === 0) initMcp();
    unlistenProgress = await listen<{ total: number; fetched: number; phase: string }>(
      "git-import-progress",
      (event) => {
        updateAgentGitProgress(event.payload);
      },
    );
  });

  onDestroy(() => {
    document.removeEventListener("click", handleClickOutside);
    if (registrySearchDebounce) clearTimeout(registrySearchDebounce);
    unlistenProgress?.();
  });

  // ── Registry / Git Handlers ────────────────────────────────

  function handleRegistrySearch(query: string) {
    registrySearchInput = query;
    if (registrySearchDebounce) clearTimeout(registrySearchDebounce);
    if (!query.trim()) {
      clearAgentRegistrySearch();
      return;
    }
    registrySearchDebounce = setTimeout(() => {
      searchAgentRegistries(query.trim());
    }, 400);
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
    return agentStore.agents.some((a) => a.sourceUrl?.includes(item.id) || a.name === item.name);
  }

  async function handleGitDiscover() {
    gitError = null;
    if (!gitUrl.trim()) return;
    try {
      await discoverGitAgents(gitUrl.trim());
    } catch (e: unknown) {
      gitError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleGitImport(file: GitSkillFile) {
    importingPath = file.path;
    try {
      await importAgentFromGit(file);
      importedPath = file.path;
      setTimeout(() => {
        importedPath = null;
      }, 2000);
    } catch {
      // Error logged in store
    } finally {
      importingPath = null;
    }
  }

  function registrySourceLabel(): string {
    return "Registry";
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

  function truncatePrompt(text: string, max: number = 100): string {
    if (text.length <= max) return text;
    return text.slice(0, max).trimEnd() + "…";
  }

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

<div class="agents-panel">
  <!-- ── Header ──────────────────────────────────────────── -->
  <header class="panel-header">
    <button
      class="back-btn"
      onclick={view.kind === "form" ? () => (view = { kind: "list" }) : onBack}
      aria-label="Go back">← Back</button
    >
    <h1 class="panel-title">
      {#if view.kind === "form" && view.editing}
        Edit {view.editing.name}
      {:else if view.kind === "form"}
        Create Agent
      {:else}
        Agents
      {/if}
    </h1>
    {#if view.kind === "list"}
      <button class="header-add-btn" onclick={openCreateForm}>+ New Agent</button>
    {/if}
  </header>

  <!-- ── Content ─────────────────────────────────────────── -->
  <div class="panel-content">
    {#if !agentStore.loaded}
      <!-- Loading -->
      <div class="panel-loading">
        <span class="loading-spinner"></span>
        Loading agents…
      </div>
    {:else if view.kind === "list"}
      <!-- ── List View ───────────────────────────────────── -->
      <div class="agents-list" role="list">
        <!-- Default agent -->
        {#if defaultAgent}
          <div
            class="agent-card default-card"
            role="listitem"
            ondblclick={() => toggleExpandAgent(defaultAgent.id)}
            title="Double-click to expand"
          >
            <div class="agent-header">
              <button
                class="agent-expand-btn"
                class:expanded={expandedAgentId === defaultAgent.id}
                onclick={(e: MouseEvent) => {
                  e.stopPropagation();
                  toggleExpandAgent(defaultAgent.id);
                }}
                aria-label={expandedAgentId === defaultAgent.id
                  ? "Collapse details"
                  : "Expand details"}>▶</button
              >
              <span class="agent-avatar">{defaultAgent.avatar ?? "🤖"}</span>
              <span class="agent-name">{defaultAgent.name}</span>
              <span class="badge built-in-badge">built-in</span>
            </div>
            {#if expandedAgentId !== defaultAgent.id}
              <p class="agent-desc">{truncatePrompt(defaultAgent.systemPrompt)}</p>
            {/if}
            {#if expandedAgentId === defaultAgent.id}
              <div class="agent-details">
                <div class="agent-prompt-full markdown-prose">
                  {@html renderMarkdown(defaultAgent.systemPrompt)}
                </div>
              </div>
            {/if}
            <div class="agent-meta">
              <span class="meta-tag">Default for new conversations</span>
            </div>
          </div>
        {/if}

        <!-- Custom agents -->
        {#if customAgents.length > 0}
          <h2 class="section-heading">Custom Agents</h2>
          {#each customAgents as agent (agent.id)}
            <div
              class="agent-card"
              role="listitem"
              ondblclick={() => toggleExpandAgent(agent.id)}
              title="Double-click to expand"
            >
              <div class="agent-header">
                <button
                  class="agent-expand-btn"
                  class:expanded={expandedAgentId === agent.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandAgent(agent.id);
                  }}
                  aria-label={expandedAgentId === agent.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <span class="agent-avatar">{agent.avatar ?? "🤖"}</span>
                <span class="agent-name">{agent.name}</span>
                {#if sourceLabel(agent)}
                  <span class="badge source-badge">{sourceLabel(agent)}</span>
                {/if}
                <div class="agent-actions">
                  <button
                    class="action-btn"
                    onclick={() => openEditForm(agent)}
                    aria-label="Edit agent {agent.name}"
                  >
                    Edit
                  </button>
                  <button
                    class="action-btn danger"
                    onclick={() => requestDelete(agent)}
                    aria-label="Delete agent {agent.name}"
                  >
                    Delete
                  </button>
                </div>
              </div>
              {#if expandedAgentId !== agent.id}
                <p class="agent-desc">{truncatePrompt(agent.systemPrompt)}</p>
              {/if}
              {#if expandedAgentId === agent.id}
                <div class="agent-details">
                  <div class="agent-prompt-full markdown-prose">
                    {@html renderMarkdown(agent.systemPrompt)}
                  </div>
                  {#if agent.sourceUrl}
                    <div class="agent-detail-row">
                      <span>Source:</span>
                      <a
                        href={agent.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="agent-detail-link">{agent.sourceUrl}</a
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
                class="search-input"
                type="search"
                placeholder="Search agents on aitmpl.com…"
                value={registrySearchInput}
                oninput={(e) => handleRegistrySearch(e.currentTarget.value)}
              />
              {#if agentStore.registrySearching}
                <span class="search-spinner">⟳</span>
              {/if}
            </div>

            {#if agentStore.registryResults.length > 0 && registrySearchInput.trim()}
              <div class="registry-results" role="list">
                {#each agentStore.registryResults as item (item.id + item.source + item.kind)}
                  <article
                    class="registry-card"
                    role="listitem"
                    ondblclick={() => toggleExpandRegistry(item)}
                    title="Double-click to expand"
                  >
                    <div class="registry-info">
                      <button
                        class="agent-expand-btn"
                        class:expanded={expandedRegistryKey === registryKey(item)}
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          toggleExpandRegistry(item);
                        }}
                        aria-label={expandedRegistryKey === registryKey(item)
                          ? "Collapse"
                          : "Expand"}>▶</button
                      >
                      <strong class="registry-name">{item.name}</strong>
                      <span class="source-badge">{registrySourceLabel()}</span>
                      {#if item.url}
                        <a
                          href={item.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="source-link"
                          aria-label="View on {registrySourceLabel()}"
                        >
                          ↗
                        </a>
                      {/if}
                    </div>
                    {#if expandedRegistryKey !== registryKey(item) && item.description}
                      <p class="registry-desc">{item.description}</p>
                    {/if}
                    {#if expandedRegistryKey === registryKey(item)}
                      <div class="registry-expanded markdown-prose">
                        {@html renderMarkdown(
                          stripFrontmatter(item.content ?? item.description ?? ""),
                        )}
                      </div>
                    {/if}
                    <div class="registry-actions">
                      {#if isAgentAlreadyInstalled(item)}
                        <span class="installed-badge">✓ Installed</span>
                      {:else if installedId === item.id}
                        <span class="installed-badge">✓ Installed</span>
                      {:else}
                        <button
                          class="action-btn primary"
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

      <!-- ── Git Import ──────────────────────────────────────── -->
      <section class="catalog-section">
        <button
          class="collapsible-heading"
          onclick={() => (gitExpanded = !gitExpanded)}
          aria-expanded={gitExpanded}
        >
          <span class="collapse-arrow" class:expanded={gitExpanded}>▶</span>
          <h3 class="section-heading inline">Import from Git</h3>
        </button>

        {#if gitExpanded}
          <div class="section-content">
            <p class="section-desc">
              Enter a GitHub or GitLab repository URL to discover SKILL.md agent templates.
            </p>
            <div class="git-row">
              <input
                class="search-input"
                type="url"
                placeholder="https://github.com/user/repo"
                bind:value={gitUrl}
              />
              <button
                class="action-btn primary"
                onclick={handleGitDiscover}
                disabled={agentStore.gitImporting || !gitUrl.trim()}
              >
                {agentStore.gitImporting ? "Scanning…" : "Scan"}
              </button>
            </div>

            {#if gitError}
              <div class="git-error" role="alert">{gitError}</div>
            {/if}

            {#if agentStore.gitImporting}
              <div class="git-progress-area">
                {#if agentStore.gitProgress}
                  <div class="git-progress-info">
                    {#if agentStore.gitProgress.phase === "tree"}
                      <span class="loading-spinner"></span> Scanning repository structure…
                    {:else}
                      <span class="loading-spinner"></span> Fetching files… {agentStore.gitProgress
                        .fetched}/{agentStore.gitProgress.total}
                    {/if}
                  </div>
                  {#if agentStore.gitProgress.phase === "fetch" && agentStore.gitProgress.total > 0}
                    <div class="git-progress-bar">
                      <div
                        class="git-progress-fill"
                        style="width: {Math.round(
                          (agentStore.gitProgress.fetched / agentStore.gitProgress.total) * 100,
                        )}%"
                      ></div>
                    </div>
                  {/if}
                {:else}
                  <div class="registry-loading">
                    <span class="loading-spinner"></span> Discovering agent files…
                  </div>
                {/if}
              </div>
            {:else if agentStore.gitDiscoveredFiles.length > 0}
              <div class="git-results" role="list">
                {#each agentStore.gitDiscoveredFiles as file (file.path)}
                  <article class="git-file-card" role="listitem">
                    <div class="git-file-info">
                      <span class="git-file-path">{file.path}</span>
                    </div>
                    <div class="git-file-actions">
                      {#if importedPath === file.path}
                        <span class="installed-badge">✓ Imported</span>
                      {:else}
                        <button
                          class="action-btn primary"
                          onclick={() => handleGitImport(file)}
                          disabled={importingPath === file.path}
                        >
                          {importingPath === file.path ? "Importing…" : "Import as Agent"}
                        </button>
                      {/if}
                    </div>
                  </article>
                {/each}
              </div>
            {:else if gitUrl.trim() && !agentStore.gitImporting && !gitError}
              <p class="section-empty">No SKILL.md files found in this repository.</p>
            {/if}
          </div>
        {/if}
      </section>

      <!-- ── Delete confirmation overlay ─────────────────── -->
      {#if confirmDelete}
        <div
          class="confirm-overlay"
          role="alertdialog"
          aria-modal="true"
          aria-label="Confirm agent deletion"
        >
          <div class="confirm-dialog">
            <p class="confirm-message">
              Delete agent <strong>'{confirmDelete.name}'</strong>?
            </p>
            <p class="confirm-detail">
              Conversations using this agent will keep their history but use the default agent.
            </p>
            <div class="confirm-actions">
              <button class="action-btn" onclick={cancelDelete} disabled={deleting}>
                Cancel
              </button>
              <button
                class="action-btn danger-fill"
                onclick={confirmDeleteAgent}
                disabled={deleting}
              >
                {deleting ? "Deleting…" : "Delete"}
              </button>
            </div>
          </div>
        </div>
      {/if}
    {:else if view.kind === "form"}
      <!-- ── Create / Edit Form ──────────────────────────── -->
      <div class="agent-form">
        {#if formError}
          <div class="form-error" role="alert">{formError}</div>
        {/if}

        <!-- Avatar + Name row -->
        <div class="form-row">
          <div class="form-field avatar-field">
            <label class="form-label" for="agent-avatar">Avatar</label>
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
            <label class="form-label" for="agent-name">Name</label>
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
          <label class="form-label" for="agent-prompt">
            System Prompt
            <span class="form-hint">Markdown supported</span>
          </label>
          <textarea
            id="agent-prompt"
            class="form-textarea"
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
            <div class="checkbox-list" role="group" aria-label="Skills">
              {#each enabledSkills as skill (skill.id)}
                <label class="checkbox-item">
                  <input
                    type="checkbox"
                    checked={formSkillIds.has(skill.id)}
                    onchange={() => toggleSkill(skill.id)}
                  />
                  <span class="checkbox-label">
                    <span class="checkbox-name">{skill.name}</span>
                    <span class="badge source-badge">{skill.source}</span>
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
            <div class="checkbox-list" role="group" aria-label="MCP servers">
              {#each mcpState.servers as server (server.config.id)}
                <label class="checkbox-item">
                  <input
                    type="checkbox"
                    checked={formMcpIds.has(server.config.id)}
                    onchange={() => toggleMcp(server.config.id)}
                  />
                  <span class="checkbox-label">
                    <span class="checkbox-status" title={server.status}>
                      {statusIcon(server.status)}
                    </span>
                    <span class="checkbox-name">{server.config.name}</span>
                    <span class="badge source-badge">{server.config.transport}</span>
                  </span>
                </label>
              {/each}
            </div>
          </fieldset>
        {/if}

        <!-- Form actions -->
        <div class="form-actions">
          <button class="action-btn" onclick={cancelForm} disabled={formSaving}>Cancel</button>
          <button class="action-btn primary" onclick={handleSave} disabled={formSaving}>
            {#if formSaving}
              <span class="loading-spinner small"></span> Saving…
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
  /* ── Panel Layout ── */

  .agents-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    animation: fadeIn 180ms ease;
  }

  .panel-header {
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

  .panel-title {
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

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
  }

  .panel-loading {
    text-align: center;
    color: var(--color-text-secondary);
    padding: var(--spacing-2xl);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-style: italic;
  }

  /* ── Agent Cards ── */

  .agents-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .section-heading {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-tertiary);
    margin: var(--spacing-md) 0 var(--spacing-sm) 0;
  }

  .section-empty {
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    font-style: italic;
    padding: var(--spacing-md) 0;
  }

  .agent-card {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    animation: fadeInUp 200ms ease both;
    transition: border-color var(--transition-fast);
  }
  .agent-card:hover {
    border-color: var(--color-border-focus);
  }

  .default-card {
    border-left: 3px solid var(--color-accent-copper);
  }

  .agent-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .agent-avatar {
    font-size: var(--font-size-lg);
    line-height: 1;
    flex-shrink: 0;
  }

  .agent-name {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
    flex: 1;
    min-width: 0;
  }

  .agent-actions {
    display: flex;
    gap: var(--spacing-xs);
    flex-shrink: 0;
  }

  .agent-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: var(--spacing-xs) 0 0;
    line-height: var(--line-height-normal);
  }

  .agent-expand-btn {
    all: unset;
    font-size: 10px;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    transition:
      transform 0.2s ease,
      color 0.15s,
      background 0.15s;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .agent-expand-btn:hover {
    color: var(--color-text-secondary);
    background: var(--color-bg-tertiary, rgba(0, 0, 0, 0.05));
  }
  .agent-expand-btn.expanded {
    transform: rotate(90deg);
    color: var(--color-accent-copper);
  }

  .agent-details {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-primary);
    animation: fadeIn 150ms ease both;
  }

  .agent-prompt-full {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    word-break: break-word;
    line-height: var(--line-height-relaxed);
    max-height: 300px;
    overflow-y: auto;
    margin: 0;
  }

  .agent-detail-row {
    display: flex;
    gap: var(--spacing-xs);
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    margin-top: var(--spacing-xs);
  }

  .agent-detail-link {
    color: var(--color-accent-copper);
    text-decoration: none;
    word-break: break-all;
  }
  .agent-detail-link:hover {
    text-decoration: underline;
  }

  .agent-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    margin-top: var(--spacing-xs);
  }

  /* ── Badges ── */

  .badge {
    font-size: var(--font-size-2xs);
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }

  .built-in-badge {
    color: var(--color-accent-copper);
    background: color-mix(in srgb, var(--color-accent-copper) 12%, transparent);
    font-weight: var(--font-weight-medium);
  }

  .source-badge {
    color: var(--color-text-tertiary);
    background: var(--color-bg-tertiary);
    font-weight: var(--font-weight-medium);
  }

  .meta-tag {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    background: var(--color-bg-tertiary);
    padding: 2px var(--spacing-xs);
    border-radius: var(--radius-sm);
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
    font-family: var(--font-body);
  }
  .action-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }
  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .action-btn.danger {
    color: var(--color-error);
    border-color: color-mix(in srgb, var(--color-error) 30%, transparent);
  }
  .action-btn.danger:hover:not(:disabled) {
    color: var(--color-error);
    border-color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
  }
  .action-btn.primary {
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
    border-color: var(--color-text-primary);
    font-weight: var(--font-weight-medium);
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
  }
  .action-btn.primary:hover:not(:disabled) {
    opacity: 0.9;
    color: var(--color-bg-primary);
  }
  .action-btn.danger-fill {
    background: var(--color-error);
    color: #fff;
    border-color: var(--color-error);
    font-weight: var(--font-weight-medium);
  }
  .action-btn.danger-fill:hover:not(:disabled) {
    opacity: 0.9;
    color: #fff;
  }

  /* ── Delete Confirmation ── */

  .confirm-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 120ms ease;
  }

  .confirm-dialog {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xl);
    max-width: 400px;
    width: 90%;
    box-shadow: var(--shadow-lg);
    animation: scaleIn 160ms ease;
  }

  .confirm-message {
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    margin: 0 0 var(--spacing-sm) 0;
  }

  .confirm-detail {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-lg) 0;
    line-height: var(--line-height-normal);
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
  }

  /* ── Form ── */

  .agent-form {
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
    animation: fadeInUp 200ms ease;
  }

  .form-error {
    font-size: var(--font-size-xs);
    color: var(--color-error);
    padding: var(--spacing-sm);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-md);
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

  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  .avatar-field {
    flex-shrink: 0;
  }

  .name-field {
    flex: 1;
    min-width: 0;
    margin-bottom: 0;
  }

  .avatar-field {
    margin-bottom: 0;
  }

  .form-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .form-input {
    padding: var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    box-sizing: border-box;
    width: 100%;
  }
  .form-input:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
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

  .form-hint {
    font-weight: var(--font-weight-normal);
    color: var(--color-text-tertiary);
    text-transform: none;
    letter-spacing: 0;
    margin-left: var(--spacing-xs);
    font-size: var(--font-size-xxs, 0.65rem);
  }

  .form-textarea {
    padding: var(--spacing-sm) var(--spacing-md);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
    box-sizing: border-box;
    width: 100%;
    resize: vertical;
    line-height: 1.6;
    min-height: 180px;
    tab-size: 2;
  }
  .form-textarea:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
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

  .checkbox-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    max-height: 200px;
    overflow-y: auto;
  }

  .checkbox-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .checkbox-item:hover {
    background: var(--color-bg-hover);
  }

  .checkbox-item input[type="checkbox"] {
    accent-color: var(--color-accent-copper);
    flex-shrink: 0;
    width: 14px;
    height: 14px;
    cursor: pointer;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    min-width: 0;
    flex: 1;
  }

  .checkbox-name {
    font-size: var(--font-size-xs);
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .checkbox-status {
    font-size: var(--font-size-2xs);
    flex-shrink: 0;
  }

  /* ── Form Actions ── */

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--color-border-primary);
    margin-top: var(--spacing-md);
  }

  /* ── Loading Spinner ── */

  .loading-spinner {
    display: inline-block;
    width: 16px;
    height: 16px;
    border: 2px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    flex-shrink: 0;
  }

  .loading-spinner.small {
    width: 12px;
    height: 12px;
    border-width: 1.5px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* ── Catalog Sections (Registry + Git) ── */

  .catalog-section {
    margin-top: var(--spacing-lg);
  }

  .collapsible-heading {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-xs) 0;
    width: 100%;
    text-align: left;
    color: var(--color-text-primary);
  }

  .collapsible-heading:hover {
    color: var(--color-accent-copper, var(--color-accent));
  }

  .collapse-arrow {
    font-size: 10px;
    transition: transform 0.2s ease;
    color: var(--color-text-tertiary);
  }

  .collapse-arrow.expanded {
    transform: rotate(90deg);
  }

  .section-heading.inline {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: inherit;
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .section-content {
    padding: var(--spacing-sm) 0;
  }

  .section-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-sm);
    line-height: var(--leading-relaxed, 1.6);
  }

  .section-empty {
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    font-style: italic;
  }

  .search-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .search-input {
    flex: 1;
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: var(--shadow-input-focus, 0 0 0 2px rgba(180, 83, 9, 0.15));
  }

  .search-spinner {
    animation: spin 0.8s linear infinite;
    color: var(--color-text-tertiary);
  }

  .git-row {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .git-error {
    font-size: var(--font-size-sm);
    color: var(--color-error, #dc2626);
    background: color-mix(in srgb, var(--color-error, #dc2626) 8%, transparent);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-sm);
  }

  .registry-results,
  .git-results {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .registry-card,
  .git-file-card {
    padding: var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md, var(--radius-sm));
    background: var(--color-bg-primary);
  }

  .registry-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-xxs, 4px);
  }

  .registry-name {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-primary);
  }

  .source-badge {
    font-size: var(--font-size-xs);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--color-bg-tertiary, var(--color-bg-secondary));
    color: var(--color-text-tertiary);
  }

  .source-link {
    font-size: var(--font-size-xs);
    color: var(--color-accent);
    text-decoration: none;
    margin-left: auto;
  }

  .source-link:hover {
    text-decoration: underline;
  }

  .registry-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-xs);
    line-height: var(--leading-relaxed, 1.6);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .registry-expanded {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: var(--spacing-sm) 0 0 0;
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-primary);
    line-height: var(--line-height-relaxed);
    max-height: 300px;
    overflow-y: auto;
    animation: fadeIn 150ms ease both;
  }

  .registry-actions,
  .git-file-actions {
    display: flex;
    justify-content: flex-end;
  }

  .installed-badge {
    font-size: var(--font-size-xs);
    color: var(--color-success, #16a34a);
    font-weight: var(--font-weight-medium);
  }

  .git-file-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .git-file-info {
    flex: 1;
    min-width: 0;
  }

  .git-file-path {
    font-size: var(--font-size-xs);
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    word-break: break-all;
  }

  /* ── Git Import Progress ── */

  .git-progress-area {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) 0;
  }

  .git-progress-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  .git-progress-bar {
    width: 100%;
    height: 6px;
    background: var(--color-bg-tertiary);
    border-radius: 3px;
    overflow: hidden;
  }

  .git-progress-fill {
    height: 100%;
    background: var(--color-accent);
    border-radius: 3px;
    transition: width 0.2s ease;
  }
</style>
