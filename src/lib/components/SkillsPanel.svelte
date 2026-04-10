<script lang="ts">
  import { stripMarkdown } from "$lib/utils/format";
  import {
    getSkillStore,
    initSkills,
    toggle,
    removeSkill,
    searchRegistries,
    installFromRegistry,
    prefetchRegistry,
  } from "$lib/stores/skills.svelte";
  import { getMcpState } from "$lib/stores/mcp.svelte";
  import { renderMarkdown, stripFrontmatter } from "$lib/utils/markdown";
  import type { Skill } from "$lib/types/skill";
  import type { RegistryItem } from "$lib/types/registry";
  import { onMount, onDestroy } from "svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const store = getSkillStore();
  const mcp = getMcpState();

  // ── Local state ─────────────────────────────────────────────

  let searchQuery = $state("");
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;

  let registryExpanded = $state(false);

  let installingId = $state<string | null>(null);
  let installedId = $state<string | null>(null);

  let confirmDelete = $state<Skill | null>(null);
  let deleting = $state(false);

  let expandedSkillId = $state<string | null>(null);
  let expandedRegistryKey = $state<string | null>(null);

  type SkillViewState = "list" | "create";
  let skillView = $state<SkillViewState>("list");

  let createName = $state("");
  let createDescription = $state("");
  let createInstructions = $state("");
  let createSaving = $state(false);
  let createError = $state<string | null>(null);

  // ── Derived: grouped skills ─────────────────────────────────

  let builtinSkills = $derived(store.skills.filter((s) => s.source === "builtin"));
  let extensionSkills = $derived(store.skills.filter((s) => s.source === "extension"));

  let mcpSkills = $derived(store.skills.filter((s) => s.source === "mcp"));
  let mcpGroups = $derived.by(() => {
    const result: Array<[string, { name: string; skills: Skill[] }]> = [];
    for (const skill of mcpSkills) {
      const key = skill.mcpServerId ?? "_unknown";
      let entry = result.find(([k]) => k === key);
      if (!entry) {
        const server = mcp.servers.find((s) => s.config.id === key);
        entry = [key, { name: server?.config.name ?? "Unknown Server", skills: [] }];
        result.push(entry);
      }
      entry[1].skills.push(skill);
    }
    return result;
  });

  let registrySkills = $derived(store.skills.filter((s) => s.source === "registry_aitmpl"));

  let gitSkills = $derived(store.skills.filter((s) => s.source === "git"));

  let hasAnySkills = $derived(store.skills.length > 0);

  // ── Handlers ────────────────────────────────────────────────

  function handleSearchInput(value: string) {
    searchQuery = value;
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(
      () => {
        if (value.trim()) {
          searchRegistries(value.trim());
        } else {
          // Restore prefetched browse results instead of clearing
          prefetchRegistry();
        }
      },
      value.trim() ? 400 : 0,
    );
  }

  async function handleToggle(skill: Skill) {
    try {
      await toggle(skill.id, !skill.enabled);
    } catch {
      // Error logged in store
    }
  }

  async function handleInstall(item: RegistryItem) {
    installingId = item.id;
    try {
      const result = await installFromRegistry(item);
      if (result) {
        installedId = item.id;
      }
    } catch {
      // Error logged in store
    } finally {
      installingId = null;
    }
  }

  function requestDelete(skill: Skill) {
    confirmDelete = skill;
  }

  function cancelDelete() {
    confirmDelete = null;
  }

  async function confirmDeleteSkill() {
    if (!confirmDelete) return;
    deleting = true;
    try {
      await removeSkill(confirmDelete.id);
    } catch {
      // Error handled by store
    } finally {
      deleting = false;
      confirmDelete = null;
    }
  }

  function toggleExpandSkill(id: string) {
    expandedSkillId = expandedSkillId === id ? null : id;
  }

  function registryKey(item: RegistryItem): string {
    return item.id + item.source + item.kind;
  }

  function toggleExpandRegistry(item: RegistryItem) {
    const key = registryKey(item);
    expandedRegistryKey = expandedRegistryKey === key ? null : key;
  }

  function openCreateForm() {
    createName = "";
    createDescription = "";
    createInstructions = "";
    createError = null;
    skillView = "create";
  }

  function cancelCreateForm() {
    skillView = "list";
  }

  async function handleCreateSkill() {
    const name = createName.trim();
    const description = createDescription.trim();
    const instructions = createInstructions.trim();
    if (!name) {
      createError = "Name is required";
      return;
    }
    if (!instructions) {
      createError = "Instructions are required";
      return;
    }
    createSaving = true;
    createError = null;
    try {
      const id = `custom-${name.toLowerCase().replace(/[^a-z0-9-]/g, "-")}`;
      const { createSkill } = await import("$lib/utils/commands");
      await createSkill(
        id,
        name,
        description || null,
        "builtin",
        null,
        null,
        instructions,
        null,
        "local",
      );
      await initSkills();
      skillView = "list";
    } catch (err: unknown) {
      createError = err instanceof Error ? err.message : String(err);
    } finally {
      createSaving = false;
    }
  }

  function sourceBadge(skill: Skill): string {
    switch (skill.source) {
      case "builtin":
        return "Built-in";
      case "mcp":
        return "MCP";
      case "extension":
        return "Extension";
      case "registry_aitmpl":
        return "Registry";
      case "git":
        return "Git";
      default:
        return skill.source;
    }
  }

  function registrySourceLabel(item: RegistryItem): string {
    return item.sourceName ?? "Registry";
  }

  function isAlreadyInstalled(item: RegistryItem): boolean {
    return store.skills.some(
      (s) =>
        s.id === item.id ||
        s.id === `reg-aitmpl-${item.id}` ||
        s.id === `git-${item.name}` ||
        s.name === item.name,
    );
  }

  onMount(async () => {
    initSkills();
  });

  onDestroy(() => {
    if (searchDebounce) clearTimeout(searchDebounce);
  });
</script>

<div class="panel">
  <!-- Header -->
  <header class="panel-header" data-tauri-drag-region>
    <button
      class="panel-back"
      onclick={skillView === "create" ? cancelCreateForm : onBack}
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
    <h2 class="panel-title">{skillView === "create" ? "Create Skill" : "Skills"}</h2>
    {#if skillView === "list"}
      <button class="btn" onclick={openCreateForm}>+ New Skill</button>
    {/if}
  </header>

  <div class="panel-body">
    {#if !store.loaded}
      <div class="panel-loading">
        <span class="spinner"></span>
        Loading skills…
      </div>
    {:else if skillView === "list"}
      <!-- ── Installed Skills ────────────────────────────────── -->
      <section class="panel-section">
        <h3 class="section-heading">Installed Skills</h3>

        {#if !hasAnySkills}
          <p class="section-empty">
            No skills installed. Browse the registry or import from Git below.
          </p>
        {/if}

        <!-- Built-in -->
        {#if builtinSkills.length > 0}
          <h4 class="group-heading">Built-in</h4>
          {#each builtinSkills as skill (skill.id)}
            <article
              class="card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
                <button
                  class="expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
                  aria-expanded={expandedSkillId === skill.id}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <strong class="card-title">{skill.name}</strong>
                <span class="badge badge--neutral">{sourceBadge(skill)}</span>
                <div class="card-actions">
                  <label class="toggle" aria-label="Toggle {skill.name}">
                    <input
                      type="checkbox"
                      checked={skill.enabled}
                      onchange={() => handleToggle(skill)}
                    />
                    <span class="toggle-track"></span>
                  </label>
                </div>
              </div>
              {#if skill.description}
                <p class="card-desc">{stripMarkdown(skill.description)}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
                  {#if skill.instructions}
                    <div class="detail-content-scroll markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="detail-link">{skill.sourceUrl}</a
                      >
                    </div>
                  {/if}
                </div>
              {/if}
            </article>
          {/each}
        {/if}

        <!-- Extensions -->
        {#if extensionSkills.length > 0}
          <h4 class="group-heading">Copilot Extensions</h4>
          {#each extensionSkills as skill (skill.id)}
            <article
              class="card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
                <button
                  class="expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
                  aria-expanded={expandedSkillId === skill.id}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <strong class="card-title">{skill.name}</strong>
                <span class="badge badge--neutral">{sourceBadge(skill)}</span>
                <div class="card-actions">
                  <label class="toggle" aria-label="Toggle {skill.name}">
                    <input
                      type="checkbox"
                      checked={skill.enabled}
                      onchange={() => handleToggle(skill)}
                    />
                    <span class="toggle-track"></span>
                  </label>
                </div>
              </div>
              {#if skill.description}
                <p class="card-desc">{stripMarkdown(skill.description)}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
                  {#if skill.instructions}
                    <div class="detail-content-scroll markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="detail-link">{skill.sourceUrl}</a
                      >
                    </div>
                  {/if}
                </div>
              {/if}
            </article>
          {/each}
        {/if}

        <!-- MCP Tools (grouped by server) -->
        {#if mcpGroups.length > 0}
          <h4 class="group-heading">MCP Tools</h4>
          {#each mcpGroups as [serverId, group] (serverId)}
            <div class="mcp-group">
              <h5 class="mcp-server-name">{group.name}</h5>
              {#each group.skills as skill (skill.id)}
                <article
                  class="card"
                  ondblclick={() => toggleExpandSkill(skill.id)}
                  title="Double-click to expand"
                >
                  <div class="card-header">
                    <button
                      class="expand-btn"
                      class:expanded={expandedSkillId === skill.id}
                      onclick={(e: MouseEvent) => {
                        e.stopPropagation();
                        toggleExpandSkill(skill.id);
                      }}
                      aria-expanded={expandedSkillId === skill.id}
                      aria-label={expandedSkillId === skill.id
                        ? "Collapse details"
                        : "Expand details"}>▶</button
                    >
                    <strong class="card-title">{skill.name}</strong>
                    <span class="badge badge--neutral">{sourceBadge(skill)}</span>
                    <div class="card-actions">
                      <label class="toggle" aria-label="Toggle {skill.name}">
                        <input
                          type="checkbox"
                          checked={skill.enabled}
                          onchange={() => handleToggle(skill)}
                        />
                        <span class="toggle-track"></span>
                      </label>
                    </div>
                  </div>
                  {#if skill.description}
                    <p class="card-desc">{stripMarkdown(skill.description)}</p>
                  {/if}
                  {#if expandedSkillId === skill.id}
                    <div class="card-detail">
                      {#if skill.instructions}
                        <div class="detail-content-scroll markdown-prose">
                          {@html renderMarkdown(skill.instructions)}
                        </div>
                      {/if}
                      {#if skill.sourceUrl}
                        <div class="detail-row">
                          <span>Source:</span>
                          <a
                            href={skill.sourceUrl}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="detail-link">{skill.sourceUrl}</a
                          >
                        </div>
                      {/if}
                    </div>
                  {/if}
                </article>
              {/each}
            </div>
          {/each}
        {/if}

        <!-- Registry-installed -->
        {#if registrySkills.length > 0}
          <h4 class="group-heading">Registry</h4>
          {#each registrySkills as skill (skill.id)}
            <article
              class="card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
                <button
                  class="expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
                  aria-expanded={expandedSkillId === skill.id}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <strong class="card-title">{skill.name}</strong>
                <span class="badge badge--neutral">{sourceBadge(skill)}</span>
                <div class="card-actions">
                  <button
                    class="btn btn--danger"
                    onclick={() => requestDelete(skill)}
                    aria-label="Delete {skill.name}"
                  >
                    Delete
                  </button>
                  <label class="toggle" aria-label="Toggle {skill.name}">
                    <input
                      type="checkbox"
                      checked={skill.enabled}
                      onchange={() => handleToggle(skill)}
                    />
                    <span class="toggle-track"></span>
                  </label>
                </div>
              </div>
              {#if skill.description}
                <p class="card-desc">{stripMarkdown(skill.description)}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
                  {#if skill.instructions}
                    <div class="detail-content-scroll markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="detail-link">{skill.sourceUrl}</a
                      >
                    </div>
                  {/if}
                </div>
              {/if}
            </article>
          {/each}
        {/if}

        <!-- Git-imported -->
        {#if gitSkills.length > 0}
          <h4 class="group-heading">Git Imported</h4>
          {#each gitSkills as skill (skill.id)}
            <article
              class="card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
                <button
                  class="expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
                  aria-expanded={expandedSkillId === skill.id}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <strong class="card-title">{skill.name}</strong>
                <span class="badge badge--neutral">{sourceBadge(skill)}</span>
                <div class="card-actions">
                  <button
                    class="btn btn--danger"
                    onclick={() => requestDelete(skill)}
                    aria-label="Delete {skill.name}"
                  >
                    Delete
                  </button>
                  <label class="toggle" aria-label="Toggle {skill.name}">
                    <input
                      type="checkbox"
                      checked={skill.enabled}
                      onchange={() => handleToggle(skill)}
                    />
                    <span class="toggle-track"></span>
                  </label>
                </div>
              </div>
              {#if skill.description}
                <p class="card-desc">{stripMarkdown(skill.description)}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
                  {#if skill.instructions}
                    <div class="detail-content-scroll markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="detail-link">{skill.sourceUrl}</a
                      >
                    </div>
                  {/if}
                </div>
              {/if}
            </article>
          {/each}
        {/if}
      </section>

      <!-- ── Registry Browser ────────────────────────────────── -->
      <section class="catalog-section">
        <button
          class="collapsible-heading"
          onclick={() => (registryExpanded = !registryExpanded)}
          aria-expanded={registryExpanded}
        >
          <span class="collapse-arrow" class:expanded={registryExpanded}>▶</span>
          <h3 class="section-heading inline">Browse Registries</h3>
          <span class="section-hint">aitmpl.com</span>
        </button>

        {#if registryExpanded}
          <div class="section-content">
            <div class="search-row">
              <input
                type="text"
                value={searchQuery}
                oninput={(e) => handleSearchInput(e.currentTarget.value)}
                placeholder="Search skills (e.g. memory, web, code)…"
                class="form-input"
              />
              {#if store.registrySearching}
                <span class="search-spinner" role="status" aria-label="Searching">⟳</span>
              {/if}
            </div>

            {#if store.registrySearching && store.registryResults.length === 0}
              <div class="registry-loading">
                <span class="spinner"></span> Searching registries…
              </div>
            {:else if store.registryResults.length > 0}
              {#if store.registryTotal !== null && searchQuery.trim()}
                <p class="result-count">
                  {store.registryTotal} result{store.registryTotal !== 1 ? "s" : ""} found
                </p>
              {/if}
              <div class="registry-results" role="list">
                {#each store.registryResults as item (item.id + item.source + item.kind)}
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
                      <span class="badge badge--neutral">{registrySourceLabel(item)}</span>
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
                      {#if item.installs !== null}
                        <span class="install-count">{item.installs} installs</span>
                      {/if}
                    </div>
                    {#if expandedRegistryKey !== registryKey(item) && item.description}
                      <p class="card-desc">{stripMarkdown(item.description)}</p>
                    {/if}
                    {#if expandedRegistryKey === registryKey(item)}
                      <div class="card-detail markdown-prose">
                        {@html renderMarkdown(
                          stripFrontmatter(item.content ?? item.description ?? ""),
                        )}
                      </div>
                    {/if}
                    <div class="card-actions">
                      {#if isAlreadyInstalled(item)}
                        <span class="badge badge--success">✓ Installed</span>
                      {:else if installedId === item.id}
                        <span class="badge badge--success">✓ Installed</span>
                      {:else}
                        <button
                          class="btn btn--primary"
                          onclick={() => handleInstall(item)}
                          disabled={installingId === item.id}
                        >
                          {installingId === item.id ? "Installing…" : "Install"}
                        </button>
                      {/if}
                    </div>
                  </article>
                {/each}
              </div>
            {:else if searchQuery.trim() && !store.registrySearching}
              <p class="section-empty">No skills match "{searchQuery}"</p>
            {/if}
          </div>
        {/if}
      </section>
    {:else if skillView === "create"}
      <!-- ── Create Skill Form ────────────────────────────────── -->
      <div class="create-skill-form">
        {#if createError}
          <div class="banner banner--error" role="alert">{createError}</div>
        {/if}

        <div class="form-field">
          <label class="form-label" for="create-name">Name</label>
          <input
            id="create-name"
            class="form-input"
            type="text"
            bind:value={createName}
            placeholder="e.g. Code Reviewer"
          />
        </div>

        <div class="form-field">
          <label class="form-label" for="create-desc">Description</label>
          <textarea
            id="create-desc"
            class="form-input"
            rows={2}
            bind:value={createDescription}
            placeholder="Brief description of what this skill does…"
          ></textarea>
        </div>

        <div class="form-field">
          <label class="form-label" for="create-instructions">
            Instructions / System Prompt
            <span class="form-hint">Markdown supported</span>
          </label>
          <textarea
            id="create-instructions"
            class="form-input form-input--mono"
            rows={8}
            bind:value={createInstructions}
            placeholder="Describe the skill's behaviour, rules, and capabilities…"
            onkeydown={(e) => {
              if (e.key === "Tab") {
                e.preventDefault();
                const t = e.currentTarget;
                const start = t.selectionStart;
                const end = t.selectionEnd;
                createInstructions =
                  createInstructions.substring(0, start) + "  " + createInstructions.substring(end);
                requestAnimationFrame(() => {
                  t.selectionStart = t.selectionEnd = start + 2;
                });
              }
            }}
          ></textarea>
        </div>

        <div class="form-actions">
          <button class="btn" onclick={cancelCreateForm}>Cancel</button>
          <button
            class="btn btn--primary"
            onclick={handleCreateSkill}
            disabled={createSaving || !createName.trim() || !createInstructions.trim()}
          >
            {createSaving ? "Creating…" : "Create Skill"}
          </button>
        </div>
      </div>
    {/if}
  </div>

  <ConfirmDialog
    open={confirmDelete !== null}
    title="Delete skill '{confirmDelete?.name ?? ''}'?"
    detail="This cannot be undone."
    loading={deleting}
    onconfirm={confirmDeleteSkill}
    oncancel={cancelDelete}
  />
</div>

<style>
  /* ── Component entry animation ── */

  .panel {
    animation: fadeIn 180ms ease;
  }

  /* ── Card overrides (animation + cursor) ── */

  .card {
    animation: fadeInUp 200ms ease both;
    cursor: default;
    user-select: text;
  }

  /* ── MCP skill grouping ── */

  .mcp-group {
    margin-bottom: var(--spacing-sm);
  }

  .mcp-server-name {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-tertiary);
    margin: var(--spacing-xs) 0;
    padding-left: var(--spacing-sm);
    font-family: var(--font-mono);
  }

  /* ── Create Custom Skill ── */

  .create-skill-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
    animation: fadeInUp 200ms ease;
  }

  .create-skill-form .form-field {
    margin-bottom: var(--spacing-md);
  }

  .create-skill-form .form-label {
    font-weight: var(--font-weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  @media (max-width: 400px) {
    .create-skill-form {
      gap: var(--spacing-sm);
    }
    .form-actions {
      flex-direction: column;
    }
    .form-actions .btn {
      width: 100%;
      text-align: center;
      justify-content: center;
    }
  }
</style>
