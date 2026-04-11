<script lang="ts">
  import {
    getSourceStore,
    initSources,
    addSource,
    toggleSource,
    removeSource,
    syncSource,
    toggleExpand,
    updateScanProgress,
    clearScanResult,
    renameSource,
  } from "$lib/stores/sources.svelte";
  import { getSettings, updateSetting, SETTING_KEYS } from "$lib/stores/settings.svelte";
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const store = getSourceStore();
  const settings = getSettings();

  type ViewState = { kind: "list" } | { kind: "add" };

  let view = $state<ViewState>({ kind: "list" });
  let addUrl = $state("");
  let addName = $state("");
  let addError = $state<string | null>(null);
  let filterQuery = $state("");
  let pendingDeleteId = $state<string | null>(null);
  let editingNameId = $state<string | null>(null);
  let editingNameValue = $state("");

  let unlistenProgress: UnlistenFn | undefined;

  onMount(async () => {
    if (!store.loaded) await initSources();
    unlistenProgress = await listen<{ total: number; fetched: number; phase: string }>(
      "git-import-progress",
      (event) => {
        updateScanProgress(event.payload.total, event.payload.fetched, event.payload.phase);
      },
    );
  });

  onDestroy(() => {
    unlistenProgress?.();
  });

  // ── Derived ───────────────────────────────────────────────────

  let filteredSources = $derived(
    filterQuery.trim()
      ? store.sources.filter(
          (s) =>
            s.name.toLowerCase().includes(filterQuery.toLowerCase()) ||
            s.url.toLowerCase().includes(filterQuery.toLowerCase()),
        )
      : store.sources,
  );

  // ── Add flow ──────────────────────────────────────────────────

  function openAdd() {
    addUrl = "";
    addName = "";
    addError = null;
    clearScanResult();
    view = { kind: "add" };
  }

  async function handleAdd() {
    addError = null;
    const url = addUrl.trim();
    if (!url) {
      addError = "Please enter a git repository URL.";
      return;
    }
    try {
      await addSource(url, addName.trim() || null);
      clearScanResult();
      view = { kind: "list" };
    } catch (e) {
      addError = String(e);
    }
  }

  // ── Sync flow ─────────────────────────────────────────────────

  async function handleSync(sourceId: string) {
    try {
      await syncSource(sourceId);
    } catch (e) {
      addError = String(e);
    }
  }

  // ── Delete ────────────────────────────────────────────────────

  function confirmDelete() {
    if (pendingDeleteId) {
      removeSource(pendingDeleteId);
      pendingDeleteId = null;
    }
  }

  function cancelDelete() {
    pendingDeleteId = null;
  }

  // ── Rename ────────────────────────────────────────────────────

  function startRename(id: string, currentName: string) {
    editingNameId = id;
    editingNameValue = currentName;
  }

  async function commitRename() {
    if (editingNameId && editingNameValue.trim()) {
      await renameSource(editingNameId, editingNameValue.trim());
    }
    editingNameId = null;
    editingNameValue = "";
  }

  // ── Helpers ───────────────────────────────────────────────────

  function shortUrl(url: string): string {
    try {
      const u = new URL(url);
      return u.hostname + u.pathname.replace(/\.git$/, "");
    } catch {
      return url;
    }
  }

  function timeAgo(iso: string): string {
    const diff = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(diff / 60_000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    return `${days}d ago`;
  }
</script>

<header class="panel-header" data-tauri-drag-region>
  <button
    class="panel-back"
    onclick={view.kind !== "list"
      ? () => {
          clearScanResult();
          view = { kind: "list" };
        }
      : onBack}
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
    {#if view.kind === "add"}
      Add Source
    {:else}
      Sources
    {/if}
  </h2>
  {#if view.kind === "list"}
    <button class="btn" onclick={openAdd}>+ Add Source</button>
  {/if}
</header>

<div class="panel-body">
  {#if !store.loaded}
    <div class="panel-loading">
      <span class="spinner"></span>
      Loading sources…
    </div>
  {:else if view.kind === "add"}
    <!-- ── Add Source Form ──────────────────────────────────── -->
    <section class="panel-section">
      <label class="form-label" for="source-url">Repository URL</label>
      <input
        id="source-url"
        class="form-input"
        type="url"
        placeholder="https://github.com/user/repo"
        bind:value={addUrl}
        onkeydown={(e) => {
          if (e.key === "Enter") handleAdd();
        }}
      />

      <label class="form-label" for="source-name">Display name (optional)</label>
      <input
        id="source-name"
        class="form-input"
        type="text"
        placeholder="Auto-detected from URL"
        bind:value={addName}
      />

      {#if addError}
        <p class="form-error">{addError}</p>
      {/if}

      {#if store.scanProgress}
        <div class="scan-progress">
          <span class="spinner spinner--sm"></span>
          <span class="scan-progress-text">
            {store.scanProgress.phase}… {store.scanProgress.fetched}/{store.scanProgress.total}
          </span>
        </div>
      {/if}

      <div class="form-actions">
        <button
          class="btn"
          onclick={() => {
            clearScanResult();
            view = { kind: "list" };
          }}
        >
          Cancel
        </button>
        <button
          class="btn btn--accent"
          onclick={handleAdd}
          disabled={store.adding || !addUrl.trim()}
        >
          {#if store.adding}
            <span class="spinner spinner--sm"></span> Scanning…
          {:else}
            Scan Repository
          {/if}
        </button>
      </div>
    </section>
  {:else}
    <!-- ── Source List ──────────────────────────────────────── -->

    {#if store.sources.length > 0}
      <div class="filter-bar">
        <input
          class="form-input"
          type="text"
          placeholder="Filter sources…"
          bind:value={filterQuery}
        />
      </div>
    {/if}

    {#if filteredSources.length === 0 && store.sources.length === 0}
      <div class="panel-empty">
        <p class="section-empty">No additional git sources configured yet.</p>
        <p class="section-empty">Add a repository URL to import skills and agents.</p>
        <button class="btn btn--accent" onclick={openAdd}>+ Add Source</button>
      </div>

      <!-- Always show aitmpl.com built-in even when no git sources -->
      <section class="panel-section builtin-section">
        <h3 class="section-title">Built-in</h3>
        <article class="card" class:card--disabled={!settings.aitmplEnabled}>
          <div class="card-header">
            <span class="builtin-icon">📦</span>
            <strong class="card-title">aitmpl.com</strong>
            <div class="card-actions">
              <label class="toggle" aria-label="Toggle aitmpl.com registry">
                <input
                  type="checkbox"
                  checked={settings.aitmplEnabled}
                  onchange={() =>
                    updateSetting(
                      SETTING_KEYS.aitmplEnabled,
                      settings.aitmplEnabled ? "false" : "true",
                    )}
                />
                <span class="toggle-track"></span>
              </label>
            </div>
          </div>
          <div class="card-meta">
            <span class="badge badge--mono">aitmpl.com</span>
            <span class="source-stats">Community skill &amp; agent registry</span>
          </div>
        </article>
      </section>
    {:else if filteredSources.length === 0}
      <p class="section-empty">No sources match "{filterQuery}"</p>
    {:else}
      <!-- Built-in aitmpl.com source (always shown, cannot be deleted) -->
      <article class="card" class:card--disabled={!settings.aitmplEnabled}>
        <div class="card-header">
          <span class="builtin-icon">📦</span>
          <strong class="card-title">aitmpl.com</strong>
          <div class="card-actions">
            <label class="toggle" aria-label="Toggle aitmpl.com registry">
              <input
                type="checkbox"
                checked={settings.aitmplEnabled}
                onchange={() =>
                  updateSetting(
                    SETTING_KEYS.aitmplEnabled,
                    settings.aitmplEnabled ? "false" : "true",
                  )}
              />
              <span class="toggle-track"></span>
            </label>
          </div>
        </div>
        <div class="card-meta">
          <span class="badge badge--mono">aitmpl.com</span>
          <span class="source-stats">Community skill &amp; agent registry</span>
        </div>
      </article>

      {#each filteredSources as source (source.id)}
        <article class="card" class:card--disabled={!source.enabled}>
          <div class="card-header">
            <button
              class="expand-btn"
              class:expanded={store.expandedIds.has(source.id)}
              onclick={() => toggleExpand(source.id)}
              aria-expanded={store.expandedIds.has(source.id)}
              aria-label={store.expandedIds.has(source.id) ? "Collapse" : "Expand"}>▶</button
            >

            {#if editingNameId === source.id}
              <input
                class="form-input inline-edit"
                type="text"
                bind:value={editingNameValue}
                onkeydown={(e) => {
                  if (e.key === "Enter") commitRename();
                  if (e.key === "Escape") {
                    editingNameId = null;
                  }
                }}
                onblur={commitRename}
              />
            {:else}
              <strong
                class="card-title"
                ondblclick={() => startRename(source.id, source.name)}
                title="Double-click to rename">{source.name}</strong
              >
            {/if}

            <div class="card-actions">
              <button
                class="btn btn--icon"
                title="Re-sync"
                disabled={store.syncing[source.id]}
                onclick={() => handleSync(source.id)}
                aria-label="Sync {source.name}"
              >
                {#if store.syncing[source.id]}
                  <span class="spinner spinner--sm"></span>
                {:else}
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 16 16"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                  >
                    <path d="M2 8a6 6 0 0 1 10.3-4.1M14 8a6 6 0 0 1-10.3 4.1" />
                    <polyline points="2 3 2 7 6 7" />
                    <polyline points="14 13 14 9 10 9" />
                  </svg>
                {/if}
              </button>

              <label class="toggle" aria-label="Toggle {source.name}">
                <input
                  type="checkbox"
                  checked={source.enabled}
                  onchange={() => toggleSource(source.id, !source.enabled)}
                />
                <span class="toggle-track"></span>
              </label>

              <button
                class="btn btn--icon btn--danger"
                title="Remove source"
                onclick={() => {
                  pendingDeleteId = source.id;
                }}
                aria-label="Delete {source.name}"
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.5"
                >
                  <path
                    d="M3 4h10M5.5 4V3a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v1M6 7v5M10 7v5M4 4l.8 9a1 1 0 0 0 1 .9h4.4a1 1 0 0 0 1-.9L12 4"
                  />
                </svg>
              </button>
            </div>
          </div>

          <div class="card-meta">
            <span class="badge badge--mono">{shortUrl(source.url)}</span>
            <span class="source-stats">
              {source.itemCount} item{source.itemCount !== 1 ? "s" : ""}
              {#if source.lastSyncedAt}
                · synced {timeAgo(source.lastSyncedAt)}
              {:else}
                · never synced
              {/if}
            </span>
          </div>

          {#if store.expandedIds.has(source.id)}
            <div class="source-items">
              {#if !store.expandedItems[source.id]}
                <div class="panel-loading"><span class="spinner spinner--sm"></span></div>
              {:else if store.expandedItems[source.id].length === 0}
                <p class="section-empty">No items imported from this source.</p>
              {:else}
                {#each store.expandedItems[source.id] as item (item.id)}
                  <div class="source-item-row">
                    <span class="source-item-icon">{item.kind === "agent" ? "🤖" : "⚡"}</span>
                    <span class="source-item-name">{item.name}</span>
                    <span class="badge badge--neutral">{item.kind}</span>
                  </div>
                {/each}
              {/if}
            </div>
          {/if}
        </article>
      {/each}
    {/if}
  {/if}
</div>

<ConfirmDialog
  open={pendingDeleteId !== null}
  title="Remove this source?"
  detail="Imported skills and agents will be kept as local copies."
  onconfirm={confirmDelete}
  oncancel={cancelDelete}
/>

<style>
  /* ── Add form ── */

  .form-label {
    display: block;
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-secondary);
    margin-bottom: var(--spacing-xs);
    margin-top: var(--spacing-md);
  }

  .form-label:first-child {
    margin-top: 0;
  }

  .form-error {
    color: var(--color-error);
    font-size: var(--font-size-xs);
    margin-top: var(--spacing-xs);
  }

  .form-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
    margin-top: var(--spacing-lg);
  }

  .scan-progress {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
  }

  /* ── Source list ── */

  .filter-bar {
    margin-bottom: var(--spacing-md);
  }

  .panel-empty {
    text-align: center;
    padding: var(--spacing-2xl) var(--spacing-lg);
  }

  .panel-empty .btn {
    margin-top: var(--spacing-md);
  }

  .card--disabled {
    opacity: 0.6;
  }

  .builtin-icon {
    flex-shrink: 0;
    font-size: var(--font-size-sm);
  }

  .builtin-section {
    margin-top: var(--spacing-lg);
  }

  .source-stats {
    font-size: var(--font-size-2xs);
    color: var(--color-text-tertiary);
  }

  .inline-edit {
    flex: 1;
    padding: 2px var(--spacing-xs);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
  }

  /* ── Expanded items ── */

  .source-items {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-primary);
  }

  .source-item-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) 0;
    font-size: var(--font-size-xs);
  }

  .source-item-row + .source-item-row {
    border-top: 1px solid var(--color-border-primary);
  }

  .source-item-name {
    flex: 1;
    color: var(--color-text-primary);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-item-icon {
    flex-shrink: 0;
    font-size: var(--font-size-sm);
  }
</style>
