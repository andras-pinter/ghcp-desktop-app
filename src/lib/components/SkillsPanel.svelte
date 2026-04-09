<script lang="ts">
  import {
    getSkillStore,
    initSkills,
    toggle,
    removeSkill,
    searchRegistries,
    installFromRegistry,
    prefetchRegistry,
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

  let gitUrl = $state("");
  let gitError = $state<string | null>(null);
  let gitScanned = $state(false);

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

  async function handleGitFetch() {
    const url = gitUrl.trim();
    if (!url) return;
    gitError = null;
    gitScanned = false;
    try {
      await discoverGitSkills(url);
    } catch (e) {
      gitError = e instanceof Error ? e.message : String(e);
    }
    gitScanned = true;
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
                  <strong class="card-title">{skill.name}</strong>
                  <span class="badge badge--neutral">{sourceBadge(skill)}</span>
                </div>
                <label class="toggle" aria-label="Toggle {skill.name}">
                  <input
                    type="checkbox"
                    checked={skill.enabled}
                    onchange={() => handleToggle(skill)}
                  />
                  <span class="toggle-track"></span>
                </label>
              </div>
              {#if skill.description}
                <p class="card-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
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
              class="card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
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
                  <strong class="card-title">{skill.name}</strong>
                  <span class="badge badge--copper">{sourceBadge(skill)}</span>
                </div>
                <label class="toggle" aria-label="Toggle {skill.name}">
                  <input
                    type="checkbox"
                    checked={skill.enabled}
                    onchange={() => handleToggle(skill)}
                  />
                  <span class="toggle-track"></span>
                </label>
              </div>
              {#if skill.description}
                <p class="card-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
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
                  class="card"
                  ondblclick={() => toggleExpandSkill(skill.id)}
                  title="Double-click to expand"
                >
                  <div class="card-header">
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
                      <strong class="card-title">{skill.name}</strong>
                      <span class="badge badge--success">{sourceBadge(skill)}</span>
                    </div>
                    <label class="toggle" aria-label="Toggle {skill.name}">
                      <input
                        type="checkbox"
                        checked={skill.enabled}
                        onchange={() => handleToggle(skill)}
                      />
                      <span class="toggle-track"></span>
                    </label>
                  </div>
                  {#if skill.description}
                    <p class="card-desc">{skill.description}</p>
                  {/if}
                  {#if expandedSkillId === skill.id}
                    <div class="card-detail">
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
              class="card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
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
                  <strong class="card-title">{skill.name}</strong>
                  <span class="badge badge--copper">{sourceBadge(skill)}</span>
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
                <p class="card-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
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
              class="card"
              ondblclick={() => toggleExpandSkill(skill.id)}
              title="Double-click to expand"
            >
              <div class="card-header">
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
                  <strong class="card-title">{skill.name}</strong>
                  <span class="badge badge--mono">{sourceBadge(skill)}</span>
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
                <p class="card-desc">{skill.description}</p>
              {/if}
              {#if expandedSkillId === skill.id}
                <div class="card-detail">
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
                      <strong class="card-title">{item.name}</strong>
                      <span class="badge badge--copper">{registrySourceLabel()}</span>
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
                      <p class="card-desc">{item.description}</p>
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
                class="form-input"
                onkeydown={(e) => {
                  if (e.key === "Enter") handleGitFetch();
                }}
              />
              <button
                class="btn btn--primary"
                onclick={handleGitFetch}
                disabled={!gitUrl.trim() || store.gitImporting}
              >
                {store.gitImporting ? "Scanning…" : "Scan"}
              </button>
            </div>

            {#if gitError}
              <div class="banner banner--error" role="alert">⚠ {gitError}</div>
            {/if}

            {#if store.gitImporting}
              <div class="git-progress-area">
                {#if store.gitProgress}
                  <div class="git-progress-info">
                    {#if store.gitProgress.phase === "tree"}
                      <span class="spinner"></span> Scanning repository structure…
                    {:else}
                      <span class="spinner"></span> Fetching files… {store.gitProgress
                        .fetched}/{store.gitProgress.total}
                    {/if}
                  </div>
                  {#if store.gitProgress.phase === "fetch" && store.gitProgress.total > 0}
                    <div class="progress">
                      <div
                        class="progress-fill"
                        style="width: {Math.round(
                          (store.gitProgress.fetched / store.gitProgress.total) * 100,
                        )}%"
                      ></div>
                    </div>
                  {/if}
                {:else}
                  <div class="registry-loading">
                    <span class="spinner"></span> Discovering SKILL.md files…
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
                  <article class="card git-file-card" role="listitem">
                    <div class="git-file-info">
                      <span class="git-file-path">{file.path}</span>
                    </div>
                    <div class="card-actions">
                      {#if importedPath === file.path}
                        <span class="badge badge--success">✓ Imported</span>
                      {:else}
                        <button
                          class="btn btn--primary"
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
            {:else if gitScanned && gitUrl.trim() && !store.gitImporting && !gitError}
              <p class="section-empty">No SKILL.md files found in this repository.</p>
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
            disabled={createSaving || !createName.trim()}
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

  /* ── Loading state ── */

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

  /* ── Skill card layout helpers ── */

  .skill-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex: 1;
    min-width: 0;
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

  /* ── Expanded skill detail helpers ── */

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

  /* ── Search / Registry ── */

  .search-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .search-row .form-input {
    flex: 1;
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

  /* Registry card overrides on .card base */

  .registry-card {
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
  }
  .registry-card:hover {
    border-color: var(--color-accent-copper);
  }

  .registry-card > .card-actions {
    justify-content: flex-end;
  }

  .registry-card .card-detail {
    max-height: 300px;
    overflow-y: auto;
  }

  .install-count {
    font-size: var(--font-size-2xs);
    color: var(--color-text-tertiary);
    margin-left: auto;
  }

  /* ── Git Import ── */

  .git-row {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .git-row .form-input {
    flex: 1;
  }

  /* Git file card overrides on .card base */

  .git-file-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
  }
  .git-file-card:hover {
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
    .search-row,
    .git-row {
      flex-direction: column;
    }
  }
</style>
