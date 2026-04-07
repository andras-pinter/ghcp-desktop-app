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
    updateGitProgress,
  } from "$lib/stores/skills.svelte";
  import { getMcpState } from "$lib/stores/mcp.svelte";
  import { renderMarkdown, stripFrontmatter } from "$lib/utils/markdown";
  import type { Skill } from "$lib/types/skill";
  import type { RegistryItem, GitSkillFile } from "$lib/types/registry";
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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

  let registryExpanded = $state(true);
  let gitExpanded = $state(false);

  let installingId = $state<string | null>(null);
  let installedId = $state<string | null>(null);
  let importingPath = $state<string | null>(null);
  let importedPath = $state<string | null>(null);

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

  function registrySourceLabel(): string {
    return "Registry";
  }

  function isAlreadyInstalled(item: RegistryItem): boolean {
    return store.skills.some(
      (s) => s.id === item.id || s.id === `reg-aitmpl-${item.id}` || s.name === item.name,
    );
  }

  let unlistenProgress: UnlistenFn | null = null;

  onMount(async () => {
    initSkills();
    unlistenProgress = await listen<{ total: number; fetched: number; phase: string }>(
      "git-import-progress",
      (event) => {
        updateGitProgress(event.payload);
      },
    );
  });

  onDestroy(() => {
    if (searchDebounce) clearTimeout(searchDebounce);
    unlistenProgress?.();
  });
</script>

<div class="skills-panel">
  <!-- Header -->
  <header class="panel-header">
    <button
      class="back-btn"
      onclick={skillView === "create" ? cancelCreateForm : onBack}
      aria-label="Go back">← Back</button
    >
    <h1 class="panel-title">{skillView === "create" ? "Create Skill" : "Skills"}</h1>
    {#if skillView === "list"}
      <button class="header-add-btn" onclick={openCreateForm}>+ New Skill</button>
    {/if}
  </header>

  <div class="panel-content">
    {#if !store.loaded}
      <div class="panel-loading">
        <span class="loading-spinner"></span>
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
              class="skill-card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
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
                    <div class="skill-instructions markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
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
            <article
              class="skill-card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
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
                    <div class="skill-instructions markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
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
                <article
                  class="skill-card"
                  ondblclick={() => toggleExpandSkill(skill.id)}
                  title="Double-click to expand"
                >
                  <div class="skill-main">
                    <button
                      class="skill-expand-btn"
                      class:expanded={expandedSkillId === skill.id}
                      onclick={(e: MouseEvent) => {
                        e.stopPropagation();
                        toggleExpandSkill(skill.id);
                      }}
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
                        <div class="skill-instructions markdown-prose">
                          {@html renderMarkdown(skill.instructions)}
                        </div>
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
            <article
              class="skill-card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
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
                  <button
                    class="action-btn danger"
                    onclick={() => requestDelete(skill)}
                    aria-label="Delete {skill.name}"
                  >
                    Delete
                  </button>
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
                    <div class="skill-instructions markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
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
            <article
              class="skill-card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="skill-main">
                <button
                  class="skill-expand-btn"
                  class:expanded={expandedSkillId === skill.id}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    toggleExpandSkill(skill.id);
                  }}
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
                  <button
                    class="action-btn danger"
                    onclick={() => requestDelete(skill)}
                    aria-label="Delete {skill.name}"
                  >
                    Delete
                  </button>
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
                    <div class="skill-instructions markdown-prose">
                      {@html renderMarkdown(skill.instructions)}
                    </div>
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
                class="search-input"
              />
              {#if store.registrySearching}
                <span class="search-spinner" role="status" aria-label="Searching">⟳</span>
              {/if}
            </div>

            {#if store.registrySearching && store.registryResults.length === 0}
              <div class="registry-loading">
                <span class="loading-spinner"></span> Searching registries…
              </div>
            {:else if store.registryResults.length > 0 && searchQuery.trim()}
              {#if store.registryTotal !== null}
                <p class="result-count">
                  {store.registryTotal} result{store.registryTotal !== 1 ? "s" : ""} found
                </p>
              {/if}
              <div class="registry-results" role="list">
                {#each store.registryResults as item (item.id + item.source + item.kind)}
                  <article
                    class="registry-card"
                    role="listitem"
                    ondblclick={() => toggleExpandRegistry(item)}
                    title="Double-click to expand"
                  >
                    <div class="registry-info">
                      <button
                        class="skill-expand-btn"
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
                      <span class="badge source-badge registry">{registrySourceLabel()}</span>
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
                      {#if item.installs !== null}
                        <span class="install-count">{item.installs} installs</span>
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
                      {#if isAlreadyInstalled(item)}
                        <span class="installed-badge">✓ Installed</span>
                      {:else if installedId === item.id}
                        <span class="installed-badge">✓ Installed</span>
                      {:else}
                        <button
                          class="action-btn primary"
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
            {:else if !searchQuery.trim()}
              <p class="section-empty section-hint-text">
                Enter a search term to find skills from registries.
              </p>
            {/if}
          </div>
        {/if}
      </section>

      <!-- ── Git Import ──────────────────────────────────────── -->
      <section class="catalog-section">
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
          <div class="section-content">
            <p class="section-desc">Import SKILL.md files from a GitHub repository.</p>
            <div class="git-row">
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
                class="action-btn primary"
                onclick={handleGitFetch}
                disabled={!gitUrl.trim() || store.gitImporting}
              >
                {store.gitImporting ? "Scanning…" : "Scan"}
              </button>
            </div>

            {#if gitError}
              <div class="git-error" role="alert">⚠ {gitError}</div>
            {/if}

            {#if store.gitImporting}
              <div class="git-progress-area">
                {#if store.gitProgress}
                  <div class="git-progress-info">
                    {#if store.gitProgress.phase === "tree"}
                      <span class="loading-spinner"></span> Scanning repository structure…
                    {:else}
                      <span class="loading-spinner"></span> Fetching files… {store.gitProgress
                        .fetched}/{store.gitProgress.total}
                    {/if}
                  </div>
                  {#if store.gitProgress.phase === "fetch" && store.gitProgress.total > 0}
                    <div class="git-progress-bar">
                      <div
                        class="git-progress-fill"
                        style="width: {Math.round(
                          (store.gitProgress.fetched / store.gitProgress.total) * 100,
                        )}%"
                      ></div>
                    </div>
                  {/if}
                {:else}
                  <div class="registry-loading">
                    <span class="loading-spinner"></span> Discovering SKILL.md files…
                  </div>
                {/if}
              </div>
            {:else if store.gitDiscoveredFiles.length > 0}
              <div class="git-results" role="list">
                <p class="result-count">
                  {store.gitDiscoveredFiles.length} skill file{store.gitDiscoveredFiles.length !== 1
                    ? "s"
                    : ""} found
                </p>
                {#each store.gitDiscoveredFiles as file (file.path)}
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
                          {importingPath === file.path ? "Importing…" : "Import"}
                        </button>
                      {/if}
                    </div>
                  </article>
                {/each}
              </div>
            {:else if gitUrl.trim() && !store.gitImporting && !gitError}
              <p class="section-empty">No SKILL.md files found in this repository.</p>
            {/if}
          </div>
        {/if}
      </section>
    {:else if skillView === "create"}
      <!-- ── Create Skill Form ────────────────────────────────── -->
      <div class="create-skill-form">
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

        <div class="create-actions">
          <button class="action-btn" onclick={cancelCreateForm}>Cancel</button>
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
  </div>

  {#if confirmDelete}
    <div
      class="confirm-overlay"
      role="alertdialog"
      aria-modal="true"
      aria-label="Confirm skill deletion"
    >
      <div class="confirm-dialog">
        <p class="confirm-message">
          Delete skill <strong>'{confirmDelete.name}'</strong>?
        </p>
        <p class="confirm-detail">This cannot be undone.</p>
        <div class="confirm-actions">
          <button class="action-btn" onclick={cancelDelete} disabled={deleting}>Cancel</button>
          <button class="action-btn danger-fill" disabled={deleting} onclick={confirmDeleteSkill}>
            {deleting ? "Deleting…" : "Delete"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* ── Panel Layout ── */

  .skills-panel {
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
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: inherit;
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
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
    padding: var(--spacing-md) 0;
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
    animation: fadeInUp 200ms ease both;
    transition: border-color var(--transition-fast);
    cursor: default;
    user-select: text;
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

  /* ── Badges ── */

  .badge {
    font-size: var(--font-size-2xs);
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }

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
    font-family: var(--font-body);
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

  /* ── Collapsible Sections ── */

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
    color: var(--color-accent-copper);
  }

  .collapse-arrow {
    font-size: 10px;
    transition: transform 0.2s ease;
    color: var(--color-text-tertiary);
  }
  .collapse-arrow.expanded {
    transform: rotate(90deg);
  }

  .section-content {
    padding: var(--spacing-sm) 0;
  }

  /* ── Expand / Collapse (cards) ── */

  .skill-expand-btn {
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
  .skill-expand-btn:hover {
    color: var(--color-text-secondary);
    background: var(--color-bg-tertiary, rgba(0, 0, 0, 0.05));
  }
  .skill-expand-btn.expanded {
    transform: rotate(90deg);
    color: var(--color-accent-copper);
  }

  .skill-details {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-primary);
    animation: fadeIn 150ms ease both;
  }

  .skill-instructions {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    word-break: break-word;
    max-height: 200px;
    overflow-y: auto;
    line-height: var(--line-height-relaxed);
    margin: 0;
  }

  .skill-detail-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    margin-top: var(--spacing-xs);
  }

  .skill-detail-link {
    color: var(--color-accent-copper);
    text-decoration: none;
    word-break: break-all;
  }
  .skill-detail-link:hover {
    text-decoration: underline;
  }

  /* ── Search / Registry ── */

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
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }
  .search-input::placeholder {
    color: var(--color-text-tertiary);
  }

  .search-spinner {
    animation: spin 0.8s linear infinite;
    color: var(--color-text-tertiary);
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
    width: 16px;
    height: 16px;
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
    border-radius: var(--radius-md);
    background: var(--color-bg-primary);
    transition: border-color var(--transition-fast);
  }
  .registry-card:hover,
  .git-file-card:hover {
    border-color: var(--color-accent-copper);
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

  .install-count {
    font-size: var(--font-size-2xs);
    color: var(--color-text-tertiary);
    margin-left: auto;
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
    color: var(--color-success);
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
  }

  /* ── Git Import ── */

  .git-row {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .git-error {
    font-size: var(--font-size-sm);
    color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-sm);
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

  .create-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  .create-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .create-hint {
    font-weight: var(--font-weight-normal);
    color: var(--color-text-tertiary);
    text-transform: none;
    letter-spacing: 0;
    font-size: var(--font-size-xxs, 0.65rem);
  }

  .create-input {
    width: 100%;
    box-sizing: border-box;
    padding: var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
  }
  .create-input:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .create-textarea {
    width: 100%;
    box-sizing: border-box;
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-body);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    resize: vertical;
    line-height: 1.6;
  }
  .create-textarea:focus {
    outline: none;
    border-color: var(--color-accent-copper);
    box-shadow: var(--shadow-input-focus);
  }

  .create-textarea.mono {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    tab-size: 2;
    min-height: 180px;
  }

  .create-error {
    font-size: var(--font-size-xs);
    color: var(--color-error);
    padding: var(--spacing-sm);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-md);
  }

  .create-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--color-border-primary);
    margin-top: var(--spacing-md);
  }

  @media (max-width: 400px) {
    .create-skill-form {
      gap: var(--spacing-sm);
    }
    .create-actions {
      flex-direction: column;
    }
    .create-actions .action-btn {
      width: 100%;
      text-align: center;
      justify-content: center;
    }
    .search-row,
    .git-row {
      flex-direction: column;
    }
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

  /* ── Keyframes ── */

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
