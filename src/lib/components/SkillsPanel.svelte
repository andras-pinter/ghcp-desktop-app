<script lang="ts">
  import {
    getSkillStore,
    initSkills,
    toggle,
    removeSkill,
    searchRegistries,
    installFromRegistry,
    clearRegistrySearch,
    discoverGitSkills,
    importFromGit,
    clearGitImport,
  } from "$lib/stores/skills.svelte";
  import { getMcpState } from "$lib/stores/mcp.svelte";
  import type { Skill } from "$lib/types/skill";
  import type { RegistryItem, GitSkillFile } from "$lib/types/registry";
  import { onMount, onDestroy } from "svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const store = getSkillStore();
  const mcp = getMcpState();

  // ── Local state ─────────────────────────────────────────────

  let searchQuery = $state("");
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;

  let gitUrl = $state("");
  let gitError = $state<string | null>(null);

  let registryExpanded = $state(false);
  let gitExpanded = $state(false);

  let installingId = $state<string | null>(null);
  let installedId = $state<string | null>(null);
  let importingPath = $state<string | null>(null);
  let importedPath = $state<string | null>(null);

  let deleteConfirmId = $state<string | null>(null);

  let expandedSkillId = $state<string | null>(null);

  let createExpanded = $state(false);
  let createName = $state("");
  let createDescription = $state("");
  let createInstructions = $state("");
  let createSourceUrl = $state("");
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

  let registrySkills = $derived(
    store.skills.filter((s) => s.source === "registry_skills_sh" || s.source === "registry_aitmpl"),
  );

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
          clearRegistrySearch();
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
        setTimeout(() => {
          installedId = null;
        }, 2000);
      }
    } catch {
      // Error logged in store
    } finally {
      installingId = null;
    }
  }

  async function handleDelete(skill: Skill) {
    if (deleteConfirmId !== skill.id) {
      deleteConfirmId = skill.id;
      return;
    }
    try {
      await removeSkill(skill.id);
    } catch {
      // Error logged in store
    } finally {
      deleteConfirmId = null;
    }
  }

  function cancelDelete() {
    deleteConfirmId = null;
  }

  async function handleGitFetch() {
    const url = gitUrl.trim();
    if (!url) return;
    gitError = null;
    try {
      await discoverGitSkills(url);
    } catch (e) {
      gitError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleGitImport(file: GitSkillFile) {
    importingPath = file.path;
    try {
      const result = await importFromGit(file);
      if (result) {
        importedPath = file.path;
        setTimeout(() => {
          importedPath = null;
        }, 2000);
      }
    } catch {
      // Error logged in store
    } finally {
      importingPath = null;
    }
  }

  function toggleExpandSkill(id: string) {
    expandedSkillId = expandedSkillId === id ? null : id;
  }

  async function handleCreateSkill() {
    const name = createName.trim();
    if (!name) {
      createError = "Name is required";
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
        createDescription.trim() || null,
        "builtin",
        null,
        null,
        createInstructions.trim() || null,
        createSourceUrl.trim() || null,
        "local",
      );
      await initSkills();
      createName = "";
      createDescription = "";
      createInstructions = "";
      createSourceUrl = "";
      createExpanded = false;
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
      case "registry_skills_sh":
        return "skills.sh";
      case "registry_aitmpl":
        return "aitmpl.com";
      case "git":
        return "Git";
      default:
        return skill.source;
    }
  }

  function registrySourceLabel(source: string): string {
    return source === "skills_sh" ? "skills.sh" : "aitmpl.com";
  }

  function isAlreadyInstalled(item: RegistryItem): boolean {
    return store.skills.some((s) => s.id === item.id);
  }

  onMount(() => {
    initSkills();
  });

  onDestroy(() => {
    if (searchDebounce) clearTimeout(searchDebounce);
  });
</script>

<div class="skills-panel">
  <!-- Header -->
  <header class="panel-header">
    <button class="back-btn" onclick={onBack} aria-label="Go back">← Back</button>
    <h2 class="panel-title">Skills</h2>
  </header>

  <div class="panel-content">
    {#if !store.loaded}
      <div class="panel-loading">Loading skills…</div>
    {:else}
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
            <article class="skill-card">
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={() => toggleExpandSkill(skill.id)}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <div class="skill-info">
                  <strong class="skill-name">{skill.name}</strong>
                  <span class="source-badge builtin">{sourceBadge(skill)}</span>
                </div>
                <label class="toggle-switch" aria-label="Toggle {skill.name}">
                  <input
                    type="checkbox"
                    checked={skill.enabled}
                    onchange={() => handleToggle(skill)}
                  />
                  <span class="toggle-track"></span>
                </label>
              </div>
              {#if skill.description}
                <p class="skill-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="skill-details">
                  {#if skill.instructions}
                    <pre class="skill-instructions">{skill.instructions}</pre>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="skill-detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="skill-detail-link">{skill.sourceUrl}</a
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
            <article class="skill-card">
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={() => toggleExpandSkill(skill.id)}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <div class="skill-info">
                  <strong class="skill-name">{skill.name}</strong>
                  <span class="source-badge extension">{sourceBadge(skill)}</span>
                </div>
                <label class="toggle-switch" aria-label="Toggle {skill.name}">
                  <input
                    type="checkbox"
                    checked={skill.enabled}
                    onchange={() => handleToggle(skill)}
                  />
                  <span class="toggle-track"></span>
                </label>
              </div>
              {#if skill.description}
                <p class="skill-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="skill-details">
                  {#if skill.instructions}
                    <pre class="skill-instructions">{skill.instructions}</pre>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="skill-detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="skill-detail-link">{skill.sourceUrl}</a
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
                <article class="skill-card">
                  <div class="skill-main">
                    <button
                      class="skill-expand-btn"
                      class:expanded={expandedSkillId === skill.id}
                      onclick={() => toggleExpandSkill(skill.id)}
                      aria-label={expandedSkillId === skill.id
                        ? "Collapse details"
                        : "Expand details"}>▶</button
                    >
                    <div class="skill-info">
                      <strong class="skill-name">{skill.name}</strong>
                      <span class="source-badge mcp">{sourceBadge(skill)}</span>
                    </div>
                    <label class="toggle-switch" aria-label="Toggle {skill.name}">
                      <input
                        type="checkbox"
                        checked={skill.enabled}
                        onchange={() => handleToggle(skill)}
                      />
                      <span class="toggle-track"></span>
                    </label>
                  </div>
                  {#if skill.description}
                    <p class="skill-desc">{skill.description}</p>
                  {/if}
                  {#if expandedSkillId === skill.id}
                    <div class="skill-details">
                      {#if skill.instructions}
                        <pre class="skill-instructions">{skill.instructions}</pre>
                      {/if}
                      {#if skill.sourceUrl}
                        <div class="skill-detail-row">
                          <span>Source:</span>
                          <a
                            href={skill.sourceUrl}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="skill-detail-link">{skill.sourceUrl}</a
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
            <article class="skill-card">
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={() => toggleExpandSkill(skill.id)}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <div class="skill-info">
                  <strong class="skill-name">{skill.name}</strong>
                  <span class="source-badge registry">{sourceBadge(skill)}</span>
                  {#if skill.sourceUrl}
                    <a
                      href={skill.sourceUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="source-link"
                      aria-label="View on {sourceBadge(skill)}"
                    >
                      ↗
                    </a>
                  {/if}
                </div>
                <div class="skill-actions">
                  {#if deleteConfirmId === skill.id}
                    <button class="action-btn danger" onclick={() => handleDelete(skill)}>
                      Confirm
                    </button>
                    <button class="action-btn" onclick={cancelDelete}>Cancel</button>
                  {:else}
                    <button
                      class="action-btn danger"
                      onclick={() => handleDelete(skill)}
                      aria-label="Delete {skill.name}"
                    >
                      Delete
                    </button>
                  {/if}
                  <label class="toggle-switch" aria-label="Toggle {skill.name}">
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
                <p class="skill-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="skill-details">
                  {#if skill.instructions}
                    <pre class="skill-instructions">{skill.instructions}</pre>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="skill-detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="skill-detail-link">{skill.sourceUrl}</a
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
            <article class="skill-card">
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={() => toggleExpandSkill(skill.id)}
                  aria-label={expandedSkillId === skill.id ? "Collapse details" : "Expand details"}
                  >▶</button
                >
                <div class="skill-info">
                  <strong class="skill-name">{skill.name}</strong>
                  <span class="source-badge git">{sourceBadge(skill)}</span>
                  {#if skill.sourceUrl}
                    <a
                      href={skill.sourceUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="source-link"
                      aria-label="View repository"
                    >
                      ↗
                    </a>
                  {/if}
                </div>
                <div class="skill-actions">
                  {#if deleteConfirmId === skill.id}
                    <button class="action-btn danger" onclick={() => handleDelete(skill)}>
                      Confirm
                    </button>
                    <button class="action-btn" onclick={cancelDelete}>Cancel</button>
                  {:else}
                    <button
                      class="action-btn danger"
                      onclick={() => handleDelete(skill)}
                      aria-label="Delete {skill.name}"
                    >
                      Delete
                    </button>
                  {/if}
                  <label class="toggle-switch" aria-label="Toggle {skill.name}">
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
                <p class="skill-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="skill-details">
                  {#if skill.instructions}
                    <pre class="skill-instructions">{skill.instructions}</pre>
                  {/if}
                  {#if skill.sourceUrl}
                    <div class="skill-detail-row">
                      <span>Source:</span>
                      <a
                        href={skill.sourceUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="skill-detail-link">{skill.sourceUrl}</a
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
      <section class="panel-section">
        <button
          class="collapsible-heading"
          onclick={() => (registryExpanded = !registryExpanded)}
          aria-expanded={registryExpanded}
        >
          <span class="collapse-arrow" class:expanded={registryExpanded}>▶</span>
          <h3 class="section-heading inline">Browse Registries</h3>
          <span class="section-hint">skills.sh · aitmpl.com</span>
        </button>

        {#if registryExpanded}
          <div class="collapsible-body">
            <div class="registry-search">
              <input
                type="text"
                value={searchQuery}
                oninput={(e) => handleSearchInput(e.currentTarget.value)}
                placeholder="Search skills (e.g. memory, web, code)…"
                class="search-input"
              />
              {#if store.registrySearching}
                <span class="search-spinner" role="status" aria-label="Searching"></span>
              {/if}
            </div>

            {#if store.registrySearching && store.registryResults.length === 0}
              <div class="registry-loading">
                <span class="loading-spinner"></span> Searching registries…
              </div>
            {:else if store.registryResults.length > 0}
              {#if store.registryTotal !== null}
                <p class="result-count">
                  {store.registryTotal} result{store.registryTotal !== 1 ? "s" : ""} found
                </p>
              {/if}
              <div class="registry-list">
                {#each store.registryResults as item (item.id)}
                  <div class="registry-item">
                    <div class="registry-item-info">
                      <strong class="registry-item-name">{item.name}</strong>
                      <span class="source-badge registry">{registrySourceLabel(item.source)}</span>
                      {#if item.installs !== null}
                        <span class="install-count">{item.installs} installs</span>
                      {/if}
                    </div>
                    {#if item.description}
                      <p class="registry-item-desc">{item.description}</p>
                    {/if}
                    <div class="registry-item-actions">
                      {#if item.url}
                        <a
                          href={item.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="source-link"
                          aria-label="View source"
                        >
                          ↗
                        </a>
                      {/if}
                      {#if isAlreadyInstalled(item)}
                        <span class="installed-badge">Installed ✓</span>
                      {:else if installedId === item.id}
                        <span class="installed-badge">Installed ✓</span>
                      {:else}
                        <button
                          class="install-btn"
                          onclick={() => handleInstall(item)}
                          disabled={installingId === item.id}
                        >
                          {installingId === item.id ? "Installing…" : "Install"}
                        </button>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {:else if searchQuery.trim() && !store.registrySearching}
              <p class="section-empty">No skills match "{searchQuery}"</p>
            {:else if !searchQuery.trim()}
              <p class="section-empty section-hint-text">
                Enter a search term to find skills from registries.
              </p>
            {/if}
          </div>
        {/if}
      </section>

      <!-- ── Git Import ──────────────────────────────────────── -->
      <section class="panel-section">
        <button
          class="collapsible-heading"
          onclick={() => {
            gitExpanded = !gitExpanded;
            if (!gitExpanded) clearGitImport();
          }}
          aria-expanded={gitExpanded}
        >
          <span class="collapse-arrow" class:expanded={gitExpanded}>▶</span>
          <h3 class="section-heading inline">Import from Git</h3>
        </button>

        {#if gitExpanded}
          <div class="collapsible-body">
            <p class="section-desc">Import SKILL.md files from a GitHub repository.</p>
            <div class="git-input-row">
              <input
                type="text"
                bind:value={gitUrl}
                placeholder="owner/repo or https://github.com/…"
                class="search-input"
                onkeydown={(e) => {
                  if (e.key === "Enter") handleGitFetch();
                }}
              />
              <button
                class="fetch-btn"
                onclick={handleGitFetch}
                disabled={!gitUrl.trim() || store.gitImporting}
              >
                {store.gitImporting ? "Fetching…" : "Fetch"}
              </button>
            </div>

            {#if gitError}
              <div class="git-error">⚠ {gitError}</div>
            {/if}

            {#if store.gitImporting}
              <div class="registry-loading">
                <span class="loading-spinner"></span> Discovering SKILL.md files…
              </div>
            {:else if store.gitDiscoveredFiles.length > 0}
              <div class="git-files-list">
                <p class="result-count">
                  {store.gitDiscoveredFiles.length} skill file{store.gitDiscoveredFiles.length !== 1
                    ? "s"
                    : ""} found
                </p>
                {#each store.gitDiscoveredFiles as file (file.path)}
                  <div class="git-file-item">
                    <div class="git-file-info">
                      <span class="git-file-path">{file.path}</span>
                    </div>
                    {#if importedPath === file.path}
                      <span class="installed-badge">Imported ✓</span>
                    {:else}
                      <button
                        class="install-btn"
                        onclick={() => handleGitImport(file)}
                        disabled={importingPath === file.path}
                      >
                        {importingPath === file.path ? "Importing…" : "Import"}
                      </button>
                    {/if}
                  </div>
                {/each}
              </div>
            {:else if gitUrl.trim() && !store.gitImporting && !gitError}
              <p class="section-empty">No SKILL.md files found in this repository.</p>
            {/if}
          </div>
        {/if}
      </section>

      <!-- ── Create Custom Skill ──────────────────────────────── -->
      <section class="panel-section">
        <button
          class="collapsible-heading"
          onclick={() => (createExpanded = !createExpanded)}
          aria-expanded={createExpanded}
        >
          <span class="collapse-arrow" class:expanded={createExpanded}>▶</span>
          <h3 class="section-heading inline">Create Custom Skill</h3>
        </button>

        {#if createExpanded}
          <div class="section-content create-skill-form">
            {#if createError}
              <div class="create-error" role="alert">{createError}</div>
            {/if}

            <div class="create-field">
              <label class="create-label" for="create-name">Name</label>
              <input
                id="create-name"
                class="create-input"
                type="text"
                bind:value={createName}
                placeholder="e.g. Code Reviewer"
              />
            </div>

            <div class="create-field">
              <label class="create-label" for="create-desc">Description</label>
              <textarea
                id="create-desc"
                class="create-textarea"
                rows={2}
                bind:value={createDescription}
                placeholder="Brief description of what this skill does…"
              ></textarea>
            </div>

            <div class="create-field">
              <label class="create-label" for="create-instructions">
                Instructions / System Prompt
                <span class="create-hint">Markdown supported</span>
              </label>
              <textarea
                id="create-instructions"
                class="create-textarea mono"
                rows={8}
                bind:value={createInstructions}
                placeholder="Describe the skill's behaviour, rules, and capabilities…"
              ></textarea>
            </div>

            <div class="create-field">
              <label class="create-label" for="create-source"
                >Source URL <span class="create-hint">optional</span></label
              >
              <input
                id="create-source"
                class="create-input"
                type="url"
                bind:value={createSourceUrl}
                placeholder="https://example.com/my-skill"
              />
            </div>

            <div class="create-actions">
              <button
                class="action-btn primary"
                onclick={handleCreateSkill}
                disabled={createSaving || !createName.trim()}
              >
                {createSaving ? "Creating…" : "Create Skill"}
              </button>
            </div>
          </div>
        {/if}
      </section>
    {/if}
  </div>
</div>

<style>
  /* ── Panel Layout ── */

  .skills-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
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

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
  }

  .panel-loading {
    text-align: center;
    color: var(--color-text-secondary);
    padding: var(--spacing-2xl);
  }

  /* ── Sections ── */

  .panel-section {
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

  .section-heading.inline {
    margin: 0;
  }

  .section-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-sm) 0;
    line-height: var(--leading-relaxed, 1.6);
  }

  .section-empty {
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    font-style: italic;
  }

  .section-hint-text {
    margin-top: var(--spacing-sm);
  }

  .section-hint {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    margin-left: auto;
    font-weight: var(--font-weight-normal);
    text-transform: none;
    letter-spacing: 0;
  }

  /* ── Group Headings ── */

  .group-heading {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-secondary);
    margin: var(--spacing-md) 0 var(--spacing-xs) 0;
    padding-left: var(--spacing-xs);
  }

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

  /* ── Skill Cards ── */

  .skill-card {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-xs);
    transition: border-color var(--transition-fast);
  }
  .skill-card:hover {
    border-color: var(--color-border-focus);
  }

  .skill-main {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .skill-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex: 1;
    min-width: 0;
  }

  .skill-name {
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .skill-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: var(--spacing-xs) 0 0 0;
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .skill-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    flex-shrink: 0;
  }

  /* ── Source Badges ── */

  .source-badge {
    font-size: var(--font-size-2xs);
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-sm);
    font-weight: var(--font-weight-medium);
    text-transform: uppercase;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .source-badge.builtin {
    color: var(--color-text-tertiary);
    background: var(--color-bg-tertiary);
  }

  .source-badge.extension {
    color: var(--color-accent-copper);
    background: color-mix(in srgb, var(--color-accent-copper) 12%, transparent);
  }

  .source-badge.mcp {
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
  }

  .source-badge.registry {
    color: var(--color-accent-copper);
    background: color-mix(in srgb, var(--color-accent-copper) 12%, transparent);
  }

  .source-badge.git {
    color: var(--color-text-secondary);
    background: var(--color-bg-tertiary);
    font-family: var(--font-mono);
    font-size: var(--font-size-2xs);
  }

  .source-link {
    font-size: var(--font-size-xs);
    color: var(--color-accent-copper);
    text-decoration: none;
    flex-shrink: 0;
    transition: opacity var(--transition-fast);
  }
  .source-link:hover {
    opacity: 0.7;
    text-decoration: underline;
  }

  /* ── Toggle Switch ── */

  .toggle-switch {
    position: relative;
    display: inline-flex;
    align-items: center;
    cursor: pointer;
    flex-shrink: 0;
  }

  .toggle-switch input {
    position: absolute;
    width: 0;
    height: 0;
    opacity: 0;
  }

  .toggle-track {
    width: 36px;
    height: 20px;
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border-primary);
    border-radius: 10px;
    position: relative;
    transition: all var(--transition-normal);
  }

  .toggle-track::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    background: var(--color-text-tertiary);
    border-radius: 50%;
    transition: all var(--transition-normal);
  }

  .toggle-switch input:checked + .toggle-track {
    background: color-mix(in srgb, var(--color-accent-copper) 20%, transparent);
    border-color: var(--color-accent-copper);
  }

  .toggle-switch input:checked + .toggle-track::after {
    left: 18px;
    background: var(--color-accent-copper);
  }

  .toggle-switch input:focus-visible + .toggle-track {
    outline: 2px solid var(--color-accent-copper);
    outline-offset: 2px;
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
    font-family: var(--font-sans);
    white-space: nowrap;
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
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border-color: var(--color-error);
  }

  /* ── Collapsible Sections ── */

  .collapsible-heading {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-sm) 0;
    text-align: left;
    font-family: var(--font-sans);
    transition: opacity var(--transition-fast);
  }
  .collapsible-heading:hover {
    opacity: 0.8;
  }

  .collapse-arrow {
    font-size: var(--font-size-2xs);
    color: var(--color-text-tertiary);
    transition: transform var(--transition-normal);
    display: inline-block;
  }
  .collapse-arrow.expanded {
    transform: rotate(90deg);
  }

  .collapsible-body {
    padding-top: var(--spacing-sm);
    animation: slideDown 200ms ease;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* ── Search / Registry ── */

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
    font-family: var(--font-sans);
    box-sizing: border-box;
  }
  .search-input:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }
  .search-input::placeholder {
    color: var(--color-text-tertiary);
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

  .result-count {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    margin: 0 0 var(--spacing-sm) 0;
  }

  .registry-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .registry-item {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }
  .registry-item:hover {
    border-color: var(--color-accent-copper);
  }

  .registry-item-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .registry-item-name {
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }

  .install-count {
    font-size: var(--font-size-2xs);
    color: var(--color-text-tertiary);
    margin-left: auto;
  }

  .registry-item-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    margin: var(--spacing-xs) 0 0 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .registry-item-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
  }

  .install-btn {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-sans);
    transition: opacity var(--transition-fast);
    white-space: nowrap;
  }
  .install-btn:hover:not(:disabled) {
    opacity: 0.85;
  }
  .install-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .installed-badge {
    font-size: var(--font-size-xs);
    color: var(--color-success);
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
  }

  /* ── Git Import ── */

  .git-input-row {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .git-input-row .search-input {
    flex: 1;
    padding-right: var(--spacing-sm);
  }

  .fetch-btn {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-sans);
    transition: opacity var(--transition-fast);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .fetch-btn:hover:not(:disabled) {
    opacity: 0.85;
  }
  .fetch-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .git-error {
    font-size: var(--font-size-xs);
    color: var(--color-error);
    margin-bottom: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border-radius: var(--radius-sm);
  }

  .git-files-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .git-file-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }
  .git-file-item:hover {
    border-color: var(--color-accent-copper);
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

  /* ── Expand/Collapse ── */

  .skill-expand-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 10px;
    color: var(--color-text-tertiary);
    padding: 2px 4px;
    margin-right: var(--spacing-xs);
    transition:
      transform 0.2s ease,
      color 0.15s;
    flex-shrink: 0;
  }

  .skill-expand-btn:hover {
    color: var(--color-text-primary);
  }

  .skill-expand-btn.expanded {
    transform: rotate(90deg);
  }

  .skill-details {
    border-top: 1px solid var(--color-border);
    padding: var(--spacing-sm) var(--spacing-xs);
    margin-top: var(--spacing-xs);
  }

  .skill-instructions {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    background: var(--color-bg-tertiary, var(--color-bg-secondary));
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    margin: 0 0 var(--spacing-xs);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 200px;
    overflow-y: auto;
    line-height: var(--leading-relaxed, 1.6);
  }

  .skill-detail-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .skill-detail-link {
    color: var(--color-accent);
    text-decoration: none;
    word-break: break-all;
  }

  .skill-detail-link:hover {
    text-decoration: underline;
  }

  /* ── Create Custom Skill ── */

  .create-skill-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    max-width: 640px;
  }

  .create-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xxs, 4px);
  }

  .create-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-primary);
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .create-hint {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-normal);
    color: var(--color-text-tertiary);
  }

  .create-input {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }

  .create-input:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: var(--shadow-input-focus, 0 0 0 2px rgba(180, 83, 9, 0.15));
  }

  .create-textarea {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    resize: vertical;
    line-height: var(--leading-relaxed, 1.6);
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }

  .create-textarea:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: var(--shadow-input-focus, 0 0 0 2px rgba(180, 83, 9, 0.15));
  }

  .create-textarea.mono {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    tab-size: 2;
  }

  .create-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
  }

  .create-error {
    font-size: var(--font-size-sm);
    color: var(--color-error, #dc2626);
    background: color-mix(in srgb, var(--color-error, #dc2626) 8%, transparent);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--color-error, #dc2626) 20%, transparent);
  }
</style>
