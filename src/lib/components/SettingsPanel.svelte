<script lang="ts">
  import { getSettings, updateSetting, SETTING_KEYS } from "$lib/stores/settings.svelte";
  import { getAuth, logout } from "$lib/stores/auth.svelte";
  import { getModelStore } from "$lib/stores/models.svelte";
  import {
    getDbSize,
    deleteOldConversations,
    exportAllConversationsJson,
    exportAllConversationsMarkdown,
    saveExportFile,
  } from "$lib/utils/commands";
  import { formatBytes } from "$lib/utils/format";
  import { onMount } from "svelte";

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  const settings = getSettings();
  const auth = getAuth();
  const modelStore = getModelStore();

  type SettingsTab = "general" | "account" | "shortcuts" | "data";
  const TABS: SettingsTab[] = ["general", "account", "shortcuts", "data"];
  let activeTab = $state<SettingsTab>("general");

  function handleTabKeydown(event: KeyboardEvent) {
    const idx = TABS.indexOf(activeTab);
    let next = idx;
    if (event.key === "ArrowRight") {
      event.preventDefault();
      next = (idx + 1) % TABS.length;
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      next = (idx - 1 + TABS.length) % TABS.length;
    } else if (event.key === "Home") {
      event.preventDefault();
      next = 0;
    } else if (event.key === "End") {
      event.preventDefault();
      next = TABS.length - 1;
    } else {
      return;
    }
    activeTab = TABS[next];
    const tabEl = document.getElementById(`tab-${activeTab}`);
    tabEl?.focus();
  }

  let dbSize = $state<number | null>(null);
  let cleanupDays = $state(90);
  let cleanupResult = $state<string | null>(null);
  let exporting = $state(false);
  let cleaningUp = $state(false);

  onMount(() => {
    loadDbSize();
  });

  async function loadDbSize() {
    try {
      dbSize = await getDbSize();
    } catch {
      dbSize = null;
    }
  }

  async function handleCleanup() {
    cleaningUp = true;
    cleanupResult = null;
    try {
      const count = await deleteOldConversations(cleanupDays);
      cleanupResult = `Deleted ${count} conversation${count === 1 ? "" : "s"}.`;
      await loadDbSize();
    } catch (e) {
      cleanupResult = `Error: ${e}`;
    } finally {
      cleaningUp = false;
    }
  }

  async function handleExport(format: "json" | "markdown") {
    exporting = true;
    try {
      const ext = format === "json" ? "json" : "md";
      const defaultName = `chuck-conversations.${ext}`;

      const content =
        format === "json"
          ? await exportAllConversationsJson()
          : await exportAllConversationsMarkdown();

      await saveExportFile(content, defaultName);
    } catch (e) {
      // User cancelled the dialog — not an error
      if (String(e) === "Export cancelled") return;
      console.error("Export failed:", e);
    } finally {
      exporting = false;
    }
  }

  async function handleLogout() {
    await logout();
  }

  const shortcuts = [
    { keys: "⌘ N", action: "New chat" },
    { keys: "⌘ ,", action: "Open settings" },
    { keys: "⌘ ⇧ S", action: "Toggle sidebar" },
    { keys: "⌘ F", action: "Search in conversation" },
    { keys: "Escape", action: "Close overlay / stop streaming" },
  ];
</script>

<div class="settings-panel">
  <header class="settings-header">
    <button class="back-btn" onclick={onBack} aria-label="Back">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path
          d="M10 3L5 8l5 5"
          stroke="currentColor"
          stroke-width="1.5"
          fill="none"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <h1 class="settings-title">Settings</h1>
  </header>

  <div class="tabs" role="tablist" tabindex="-1" onkeydown={handleTabKeydown}>
    {#each [{ id: "general", label: "General" }, { id: "account", label: "Account" }, { id: "shortcuts", label: "Shortcuts" }, { id: "data", label: "Data" }] as tab (tab.id)}
      <button
        id="tab-{tab.id}"
        role="tab"
        class="tab"
        class:active={activeTab === tab.id}
        aria-selected={activeTab === tab.id}
        aria-controls="panel-{tab.id}"
        tabindex={activeTab === tab.id ? 0 : -1}
        onclick={() => (activeTab = tab.id as SettingsTab)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="tab-content" role="tabpanel" id="panel-{activeTab}" aria-labelledby="tab-{activeTab}">
    {#if activeTab === "general"}
      <section class="settings-section">
        <h2 class="section-title">Appearance</h2>

        <div class="setting-row">
          <label class="setting-label" for="theme-select">Theme</label>
          <select
            id="theme-select"
            class="setting-select"
            value={settings.theme}
            onchange={(e) =>
              updateSetting(SETTING_KEYS.theme, (e.target as HTMLSelectElement).value)}
          >
            <option value="system">System</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>

        <div class="setting-row">
          <label class="setting-label" for="font-size-select">Font size</label>
          <select
            id="font-size-select"
            class="setting-select"
            value={String(settings.fontSize)}
            onchange={(e) =>
              updateSetting(SETTING_KEYS.fontSize, (e.target as HTMLSelectElement).value)}
          >
            {#each [12, 13, 14, 15, 16] as size (size)}
              <option value={String(size)}>{size}px</option>
            {/each}
          </select>
        </div>
      </section>

      <section class="settings-section">
        <h2 class="section-title">Input</h2>

        <div class="setting-row">
          <span class="setting-label">Send message with</span>
          <div class="radio-group">
            <label class="radio-label">
              <input
                type="radio"
                name="send-shortcut"
                value="enter"
                checked={settings.sendShortcut === "enter"}
                onchange={() => updateSetting(SETTING_KEYS.sendShortcut, "enter")}
              />
              Enter
            </label>
            <label class="radio-label">
              <input
                type="radio"
                name="send-shortcut"
                value="cmd-enter"
                checked={settings.sendShortcut === "cmd-enter"}
                onchange={() => updateSetting(SETTING_KEYS.sendShortcut, "cmd-enter")}
              />
              ⌘ Enter
            </label>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h2 class="section-title">Defaults</h2>

        {#if modelStore.models.length > 1}
          <div class="setting-row">
            <label class="setting-label" for="default-model">Default model</label>
            <select
              id="default-model"
              class="setting-select"
              value={modelStore.defaultModelId ?? modelStore.models[0]?.id ?? ""}
              onchange={(e) => {
                const val = (e.target as HTMLSelectElement).value;
                if (val) {
                  import("$lib/stores/models.svelte").then(({ setDefaultModel }) =>
                    setDefaultModel(val),
                  );
                }
              }}
            >
              {#each modelStore.models as m (m.id)}
                <option value={m.id}>{m.name ?? m.id}</option>
              {/each}
            </select>
          </div>
        {/if}
      </section>
    {:else if activeTab === "account"}
      <section class="settings-section">
        <h2 class="section-title">Account</h2>

        {#if auth.user}
          <div class="account-info">
            <div class="account-row">
              <span class="account-label">Signed in as</span>
              <span class="account-value">@{auth.user.login}</span>
            </div>
          </div>
        {/if}

        <button class="btn-danger" onclick={handleLogout}>Sign out</button>
      </section>
    {:else if activeTab === "shortcuts"}
      <section class="settings-section">
        <h2 class="section-title">Keyboard shortcuts</h2>

        <div class="shortcuts-table" role="table" aria-label="Keyboard shortcuts">
          {#each shortcuts as shortcut (shortcut.keys)}
            <div class="shortcut-row" role="row">
              <kbd class="shortcut-keys">{shortcut.keys}</kbd>
              <span class="shortcut-action">{shortcut.action}</span>
            </div>
          {/each}
        </div>

        <p class="settings-note">
          {#if settings.sendShortcut === "cmd-enter"}
            Send shortcut: <kbd>⌘ Enter</kbd> (Enter inserts newline)
          {:else}
            Send shortcut: <kbd>Enter</kbd> (Shift+Enter inserts newline)
          {/if}
        </p>
      </section>
    {:else if activeTab === "data"}
      <section class="settings-section">
        <h2 class="section-title">Database</h2>

        <div class="setting-row">
          <span class="setting-label">Database size</span>
          <span class="setting-value">
            {dbSize !== null ? formatBytes(dbSize) : "…"}
          </span>
        </div>

        {#if dbSize !== null && dbSize > 400 * 1024 * 1024}
          <div class="warning-banner" role="alert">
            Database is {formatBytes(dbSize)} — consider cleaning up old conversations.
          </div>
        {/if}
      </section>

      <section class="settings-section">
        <h2 class="section-title">Cleanup</h2>

        <div class="cleanup-row">
          <label class="setting-label" for="cleanup-days">Delete conversations older than</label>
          <div class="cleanup-controls">
            <select id="cleanup-days" class="setting-select" bind:value={cleanupDays}>
              <option value={30}>30 days</option>
              <option value={60}>60 days</option>
              <option value={90}>90 days</option>
              <option value={180}>180 days</option>
              <option value={365}>1 year</option>
            </select>
            <button class="btn-secondary" onclick={handleCleanup} disabled={cleaningUp}>
              {cleaningUp ? "Deleting…" : "Delete now"}
            </button>
          </div>
        </div>

        {#if cleanupResult}
          <p class="cleanup-result">{cleanupResult}</p>
        {/if}
      </section>

      <section class="settings-section">
        <h2 class="section-title">Export</h2>

        <p class="settings-note">Export all conversations to a file.</p>

        <div class="export-buttons">
          <button class="btn-secondary" onclick={() => handleExport("json")} disabled={exporting}>
            Export JSON
          </button>
          <button
            class="btn-secondary"
            onclick={() => handleExport("markdown")}
            disabled={exporting}
          >
            Export Markdown
          </button>
        </div>
      </section>
    {/if}
  </div>
</div>

<style>
  .settings-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    animation: fadeIn 200ms ease both;
  }

  .settings-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg) var(--spacing-xl);
    padding-top: calc(var(--spacing-lg) + var(--titlebar-height));
    border-bottom: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
  }

  .back-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .back-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .settings-title {
    font-family: var(--font-display);
    font-size: var(--font-size-xl);
    font-weight: 400;
    font-style: italic;
    color: var(--color-text-primary);
    margin: 0;
  }

  /* ── Tabs ── */

  .tabs {
    display: flex;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-xl);
    border-bottom: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
    overflow-x: auto;
  }

  .tab {
    padding: var(--spacing-xs) var(--spacing-md);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .tab:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .tab.active {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  /* ── Content ── */

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xl);
  }

  .settings-section {
    margin-bottom: var(--spacing-2xl);
  }

  .section-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 var(--spacing-lg);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) 0;
    gap: var(--spacing-lg);
  }

  .setting-label {
    font-size: var(--font-size-base);
    color: var(--color-text-primary);
  }

  .setting-value {
    font-size: var(--font-size-base);
    color: var(--color-text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .setting-select {
    padding: var(--spacing-xs) var(--spacing-md);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-input);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    min-width: 120px;
  }

  .setting-select:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
  }

  /* ── Radio group ── */

  .radio-group {
    display: flex;
    gap: var(--spacing-lg);
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    cursor: pointer;
  }

  .radio-label input[type="radio"] {
    accent-color: var(--color-accent-copper);
  }

  /* ── Account ── */

  .account-info {
    margin-bottom: var(--spacing-lg);
  }

  .account-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm) 0;
  }

  .account-label {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  .account-value {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-primary);
  }

  /* ── Shortcuts table ── */

  .shortcuts-table {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-lg);
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    background: var(--color-bg-secondary);
  }

  .shortcut-keys {
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
    min-width: 80px;
    background: var(--color-bg-primary);
    padding: 2px var(--spacing-sm);
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border-secondary);
  }

  .shortcut-action {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  /* ── Cleanup ── */

  .cleanup-row {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .cleanup-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .cleanup-result {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin-top: var(--spacing-sm);
  }

  /* ── Export ── */

  .export-buttons {
    display: flex;
    gap: var(--spacing-sm);
  }

  /* ── Warning banner ── */

  .warning-banner {
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    background: var(
      --color-warning-subtle,
      color-mix(in srgb, var(--color-accent-copper) 10%, transparent)
    );
    color: var(--color-accent-copper);
    font-size: var(--font-size-sm);
    margin-top: var(--spacing-sm);
  }

  /* ── Notes ── */

  .settings-note {
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    margin-bottom: var(--spacing-md);
  }

  .settings-note kbd {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    background: var(--color-bg-secondary);
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border-secondary);
  }

  /* ── Buttons ── */

  .btn-secondary {
    padding: var(--spacing-xs) var(--spacing-lg);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-sm);
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-danger {
    padding: var(--spacing-xs) var(--spacing-lg);
    border: 1px solid var(--color-error, #dc2626);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-error, #dc2626);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-danger:hover {
    background: var(
      --color-error-subtle,
      color-mix(in srgb, var(--color-error, #dc2626) 10%, transparent)
    );
  }
</style>
