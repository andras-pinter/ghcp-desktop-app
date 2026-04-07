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
  } from "$lib/stores/agents.svelte";
  import { getSkillStore, initSkills } from "$lib/stores/skills.svelte";
  import { getMcpState, initMcp } from "$lib/stores/mcp.svelte";
  import type { Agent } from "$lib/types/agent";
  import { onMount } from "svelte";

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

  // ── Delete confirmation ─────────────────────────────────────

  let confirmDelete = $state<Agent | null>(null);
  let deleting = $state(false);

  // ── Derived ─────────────────────────────────────────────────

  let defaultAgent = $derived(agentStore.agents.find((a) => a.isDefault));
  let customAgents = $derived(agentStore.agents.filter((a) => !a.isDefault));
  let enabledSkills = $derived(skillStore.skills.filter((s) => s.enabled));

  // ── Lifecycle ───────────────────────────────────────────────

  onMount(() => {
    if (!agentStore.loaded) initAgents();
    if (!skillStore.loaded) initSkills();
    if (mcpState.servers.length === 0) initMcp();
  });

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
    <button class="back-btn" onclick={onBack} aria-label="Go back">← Back</button>
    <h1 class="panel-title">Agents</h1>
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
          <div class="agent-card default-card" role="listitem">
            <div class="agent-header">
              <span class="agent-avatar">{defaultAgent.avatar ?? "🤖"}</span>
              <span class="agent-name">{defaultAgent.name}</span>
              <span class="badge built-in-badge">built-in</span>
            </div>
            <p class="agent-desc">{truncatePrompt(defaultAgent.systemPrompt)}</p>
            <div class="agent-meta">
              <span class="meta-tag">Default for new conversations</span>
            </div>
          </div>
        {/if}

        <!-- Custom agents -->
        {#if customAgents.length > 0}
          <h2 class="section-heading">Custom Agents</h2>
          {#each customAgents as agent (agent.id)}
            <div class="agent-card" role="listitem">
              <div class="agent-header">
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
              <p class="agent-desc">{truncatePrompt(agent.systemPrompt)}</p>
            </div>
          {/each}
        {:else if !defaultAgent}
          <p class="section-empty">No agents configured. Create one to get started.</p>
        {/if}

        <!-- Create new CTA (bottom) -->
        <button class="create-cta" onclick={openCreateForm}>+ Create New Agent</button>
      </div>

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
        <h2 class="form-title">
          {view.editing ? `Edit ${view.editing.name}` : "Create New Agent"}
        </h2>

        {#if formError}
          <div class="form-error" role="alert">{formError}</div>
        {/if}

        <!-- Avatar + Name row -->
        <div class="form-row">
          <div class="form-field avatar-field">
            <label class="form-label" for="agent-avatar">Avatar</label>
            <input
              id="agent-avatar"
              class="form-input avatar-input"
              type="text"
              maxlength={2}
              bind:value={formAvatar}
              placeholder="🤖"
            />
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
          <label class="form-label" for="agent-prompt">System Prompt</label>
          <textarea
            id="agent-prompt"
            class="form-textarea"
            rows={8}
            bind:value={formPrompt}
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
  .action-btn.danger:hover:not(:disabled) {
    color: var(--color-error);
    border-color: var(--color-error);
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

  /* ── Create CTA (dashed) ── */

  .create-cta {
    width: 100%;
    padding: var(--spacing-md);
    background: none;
    border: 1px dashed var(--color-border-secondary);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      border-color var(--transition-fast);
    margin-top: var(--spacing-sm);
  }
  .create-cta:hover {
    color: var(--color-accent-copper);
    border-color: var(--color-accent-copper);
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
    max-width: 560px;
    animation: fadeInUp 200ms ease;
  }

  .form-title {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-lg);
    color: var(--color-text-primary);
    margin: 0 0 var(--spacing-lg) 0;
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

  .avatar-input {
    width: 3rem;
    text-align: center;
    font-size: var(--font-size-lg);
    padding: var(--spacing-xs);
  }

  .form-textarea {
    padding: var(--spacing-sm);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    box-sizing: border-box;
    width: 100%;
    resize: vertical;
    line-height: var(--line-height-normal);
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
</style>
