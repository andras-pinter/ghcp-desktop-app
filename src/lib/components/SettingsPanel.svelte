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
  <header class="settings-header" data-tauri-drag-region>
    <button class="back-btn" onclick={onBack} aria-label="Back">
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
    <h1 class="settings-title">Settings</h1>
  </header>

  <div class="tab-bar" role="tablist" tabindex="-1" onkeydown={handleTabKeydown}>
    {#each [{ id: "general", label: "General", icon: "M12 6V4H4v2m8 0H4m8 0v6m-8 0v-6m0 6h8m-8 0H2m10 0h2M8 4V2" }, { id: "account", label: "Account", icon: "M8 8a3 3 0 100-6 3 3 0 000 6zm0 0c-3.3 0-6 1.8-6 4v1h12v-1c0-2.2-2.7-4-6-4z" }, { id: "shortcuts", label: "Shortcuts", icon: "M3 4h10M3 8h6M3 12h8" }, { id: "data", label: "Data", icon: "M3 3h10v10H3zm2 4h6m-6 3h4" }] as tab (tab.id)}
      <button
        id="tab-{tab.id}"
        role="tab"
        class="tab-pill"
        class:active={activeTab === tab.id}
        aria-selected={activeTab === tab.id}
        aria-controls="panel-{tab.id}"
        tabindex={activeTab === tab.id ? 0 : -1}
        onclick={() => (activeTab = tab.id as SettingsTab)}
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d={tab.icon} />
        </svg>
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="tab-content" role="tabpanel" id="panel-{activeTab}" aria-labelledby="tab-{activeTab}">
    {#if activeTab === "general"}
      <div class="settings-card">
        <h2 class="card-heading">Appearance</h2>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Theme</span>
            <span class="setting-desc">Choose how Chuck looks</span>
          </div>
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

        <div class="setting-divider"></div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Font size</span>
            <span class="setting-desc">Base text size for the interface</span>
          </div>
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
      </div>

      <div class="settings-card">
        <h2 class="card-heading">Input</h2>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Send message with</span>
            <span class="setting-desc">How to send messages in the chat input</span>
          </div>
          <div class="segmented-control">
            <button
              class="segment"
              class:active={settings.sendShortcut === "enter"}
              onclick={() => updateSetting(SETTING_KEYS.sendShortcut, "enter")}
              aria-pressed={settings.sendShortcut === "enter"}>Enter</button
            >
            <button
              class="segment"
              class:active={settings.sendShortcut === "cmd-enter"}
              onclick={() => updateSetting(SETTING_KEYS.sendShortcut, "cmd-enter")}
              aria-pressed={settings.sendShortcut === "cmd-enter"}>⌘ Enter</button
            >
          </div>
        </div>
      </div>

      {#if modelStore.models.length > 1}
        <div class="settings-card">
          <h2 class="card-heading">Defaults</h2>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Default model</span>
              <span class="setting-desc">Used for new conversations</span>
            </div>
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
        </div>
      {/if}

      <div class="settings-card">
        <h2 class="card-heading">Auto-Update</h2>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Check for updates</span>
            <span class="setting-desc">Automatically check for new versions</span>
          </div>
          <label class="toggle-label">
            <input
              id="auto-update-toggle"
              type="checkbox"
              class="toggle-input"
              checked={settings.autoUpdateEnabled}
              onchange={(e) =>
                updateSetting(
                  SETTING_KEYS.autoUpdateEnabled,
                  String((e.target as HTMLInputElement).checked),
                )}
            />
            <span class="toggle-switch"></span>
          </label>
        </div>

        {#if settings.autoUpdateEnabled}
          <div class="setting-divider"></div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Check frequency</span>
            </div>
            <select
              id="update-frequency"
              class="setting-select"
              value={settings.autoUpdateFrequency}
              onchange={(e) =>
                updateSetting(
                  SETTING_KEYS.autoUpdateFrequency,
                  (e.target as HTMLSelectElement).value,
                )}
            >
              <option value="startup">On startup</option>
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
            </select>
          </div>

          {#if settings.skippedVersion}
            <div class="setting-divider"></div>

            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Skipping version</span>
                <span class="setting-desc">{settings.skippedVersion} will be ignored</span>
              </div>
              <button
                class="btn-pill"
                onclick={() => updateSetting(SETTING_KEYS.skippedVersion, "")}
              >
                Clear skip
              </button>
            </div>
          {/if}
        {/if}
      </div>
    {:else if activeTab === "account"}
      <div class="settings-card account-card">
        {#if auth.user}
          <div class="account-header">
            <div class="account-avatar">
              <svg
                width="22"
                height="22"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="1.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="8" cy="6" r="3" />
                <path d="M2 14c0-2.2 2.7-4 6-4s6 1.8 6 4" />
              </svg>
            </div>
            <div class="account-details">
              <span class="account-username">@{auth.user.login}</span>
              <span class="account-badge">GitHub Copilot</span>
            </div>
          </div>
        {:else}
          <div class="account-header">
            <div class="account-avatar">
              <svg
                width="22"
                height="22"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="1.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="8" cy="6" r="3" />
                <path d="M2 14c0-2.2 2.7-4 6-4s6 1.8 6 4" />
              </svg>
            </div>
            <div class="account-details">
              <span class="account-username">Not signed in</span>
            </div>
          </div>
        {/if}

        <div class="setting-divider"></div>

        <button class="btn-danger-pill" onclick={handleLogout}>
          <svg
            width="14"
            height="14"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.3"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M6 2H3v12h3m4-6h5m0 0l-3-3m3 3l-3 3" />
          </svg>
          Sign out
        </button>
      </div>
    {:else if activeTab === "shortcuts"}
      <div class="settings-card">
        <h2 class="card-heading">Keyboard shortcuts</h2>

        <div class="shortcuts-list" role="list" aria-label="Keyboard shortcuts">
          {#each shortcuts as shortcut, i (shortcut.keys)}
            {#if i > 0}
              <div class="setting-divider"></div>
            {/if}
            <div class="shortcut-row" role="listitem">
              <span class="shortcut-action">{shortcut.action}</span>
              <kbd class="shortcut-kbd">{shortcut.keys}</kbd>
            </div>
          {/each}
        </div>
      </div>

      <div class="settings-card settings-card-muted">
        <p class="settings-note">
          {#if settings.sendShortcut === "cmd-enter"}
            Send shortcut is set to <kbd>⌘ Enter</kbd> — pressing Enter inserts a newline.
          {:else}
            Send shortcut is set to <kbd>Enter</kbd> — press Shift+Enter for a newline.
          {/if}
          You can change this in the
          <button class="link-btn" onclick={() => (activeTab = "general")}>General</button> tab.
        </p>
      </div>
    {:else if activeTab === "data"}
      <div class="settings-card">
        <h2 class="card-heading">Storage</h2>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Database size</span>
            <span class="setting-desc">Local conversation storage</span>
          </div>
          <span class="setting-value-badge">
            {dbSize !== null ? formatBytes(dbSize) : "…"}
          </span>
        </div>

        {#if dbSize !== null && dbSize > 400 * 1024 * 1024}
          <div class="warning-banner" role="alert">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 1l7 13H1L8 1zm0 4v4m0 2v1" />
            </svg>
            Database is {formatBytes(dbSize)} — consider cleaning up.
          </div>
        {/if}
      </div>

      <div class="settings-card">
        <h2 class="card-heading">Cleanup</h2>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Delete old conversations</span>
            <span class="setting-desc">Permanently remove conversations older than</span>
          </div>
          <div class="cleanup-controls">
            <select id="cleanup-days" class="setting-select" bind:value={cleanupDays}>
              <option value={30}>30 days</option>
              <option value={60}>60 days</option>
              <option value={90}>90 days</option>
              <option value={180}>180 days</option>
              <option value={365}>1 year</option>
            </select>
            <button class="btn-pill btn-pill-danger" onclick={handleCleanup} disabled={cleaningUp}>
              {cleaningUp ? "Deleting…" : "Delete"}
            </button>
          </div>
        </div>

        {#if cleanupResult}
          <p class="cleanup-result">{cleanupResult}</p>
        {/if}
      </div>

      <div class="settings-card">
        <h2 class="card-heading">Export</h2>
        <p class="card-desc">Export all conversations to a file for backup or sharing.</p>

        <div class="export-buttons">
          <button class="btn-export" onclick={() => handleExport("json")} disabled={exporting}>
            <svg
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M4 2h5l3 3v9H4V2z" />
              <path d="M9 2v3h3" />
            </svg>
            JSON
          </button>
          <button class="btn-export" onclick={() => handleExport("markdown")} disabled={exporting}>
            <svg
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M4 2h5l3 3v9H4V2z" />
              <path d="M9 2v3h3" />
            </svg>
            Markdown
          </button>
        </div>
      </div>
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

  /* ── Header ── */

  .settings-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg) var(--spacing-xl);
    padding-top: calc(var(--spacing-lg) + var(--titlebar-height));
    flex-shrink: 0;
    max-width: 95%;
    width: 100%;
    margin: 0 auto;
  }

  .back-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    cursor: pointer;
    color: var(--color-text-secondary);
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
  }

  .back-btn:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
    border-color: var(--color-border-primary);
  }

  .back-btn:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
    border-color: var(--color-border-focus);
  }

  .settings-title {
    font-family: var(--font-display);
    font-size: var(--font-size-xl);
    font-weight: 400;
    font-style: italic;
    color: var(--color-text-primary);
    margin: 0;
  }

  /* ── Tab bar (pill-style) ── */

  .tab-bar {
    display: flex;
    gap: 2px;
    margin-bottom: var(--spacing-xs);
    flex-shrink: 0;
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    padding: 3px;
    max-width: 95%;
    width: calc(100% - var(--spacing-xl) * 2);
    margin-left: auto;
    margin-right: auto;
  }

  .tab-pill {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 7px var(--spacing-md);
    background: transparent;
    border: none;
    border-radius: calc(var(--radius-md) - 2px);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .tab-pill svg {
    opacity: 0.5;
    transition: opacity var(--transition-fast);
  }

  .tab-pill:hover {
    color: var(--color-text-secondary);
  }

  .tab-pill:hover svg {
    opacity: 0.7;
  }

  .tab-pill:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
  }

  .tab-pill.active {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    box-shadow: var(--shadow-sm);
  }

  .tab-pill.active svg {
    opacity: 1;
  }

  /* ── Content area ── */

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg) var(--spacing-xl) var(--spacing-2xl);
    max-width: 95%;
    width: 100%;
    margin: 0 auto;
  }

  /* ── Settings card ── */

  .settings-card {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg) var(--spacing-xl);
    margin-bottom: var(--spacing-md);
  }

  .settings-card-muted {
    background: transparent;
    border: 1px dashed var(--color-border-secondary);
  }

  .card-heading {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin: 0 0 var(--spacing-lg);
  }

  .card-desc {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-lg);
    line-height: 1.5;
  }

  /* ── Setting rows ── */

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-xl);
    min-height: 36px;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .setting-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-primary);
  }

  .setting-desc {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .setting-divider {
    height: 1px;
    background: var(--color-border-secondary);
    margin: var(--spacing-md) 0;
  }

  /* ── Select control ── */

  .setting-select {
    padding: 6px var(--spacing-md);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    background: var(--color-bg-input);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    min-width: 120px;
    transition: all var(--transition-fast);
  }

  .setting-select:hover {
    border-color: var(--color-text-tertiary);
  }

  .setting-select:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
    border-color: var(--color-border-focus);
  }

  /* ── Segmented control (send shortcut) ── */

  .segmented-control {
    display: flex;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    padding: 2px;
    gap: 2px;
  }

  .segment {
    padding: 5px var(--spacing-md);
    background: transparent;
    border: none;
    border-radius: calc(var(--radius-md) - 3px);
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .segment:hover {
    color: var(--color-text-secondary);
  }

  .segment:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
    border-radius: var(--radius-sm);
  }

  .segment.active {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    font-weight: var(--font-weight-medium);
    box-shadow: var(--shadow-sm);
  }

  /* ── Toggle switch ── */

  .toggle-label {
    display: flex;
    align-items: center;
    cursor: pointer;
    flex-shrink: 0;
  }

  .toggle-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-switch {
    position: relative;
    width: 40px;
    height: 22px;
    background: var(--color-border-primary);
    border-radius: 11px;
    transition: background 200ms ease;
    flex-shrink: 0;
  }

  .toggle-switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    background: var(--color-bg-primary);
    border-radius: 50%;
    transition: transform 200ms cubic-bezier(0.34, 1.56, 0.64, 1);
    box-shadow: var(--shadow-sm);
  }

  .toggle-input:checked + .toggle-switch {
    background: var(--color-accent-copper);
  }

  .toggle-input:checked + .toggle-switch::after {
    transform: translateX(18px);
  }

  .toggle-input:focus-visible + .toggle-switch {
    box-shadow: var(--shadow-input-focus);
  }

  /* ── Account card ── */

  .account-card {
    text-align: left;
  }

  .account-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .account-avatar {
    width: 44px;
    height: 44px;
    border-radius: var(--radius-full);
    background: var(--color-bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .account-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .account-username {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
  }

  .account-badge {
    font-size: var(--font-size-xs);
    color: var(--color-accent-copper);
    font-weight: var(--font-weight-medium);
  }

  /* ── Shortcuts ── */

  .shortcuts-list {
    display: flex;
    flex-direction: column;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) 0;
  }

  .shortcut-action {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }

  .shortcut-kbd {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--color-text-primary);
    background: var(--color-bg-primary);
    padding: 3px var(--spacing-sm);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-primary);
    box-shadow: 0 1px 0 var(--color-border-primary);
    min-width: 48px;
    text-align: center;
  }

  .settings-note {
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
    line-height: 1.6;
    margin: 0;
  }

  .settings-note kbd {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    background: var(--color-bg-secondary);
    padding: 1px var(--spacing-xs);
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border-secondary);
  }

  .link-btn {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: var(--color-accent-copper);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .link-btn:hover {
    color: var(--color-accent-copper-hover);
  }

  .link-btn:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
    border-radius: var(--radius-sm);
  }

  /* ── Value badge ── */

  .setting-value-badge {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    font-variant-numeric: tabular-nums;
    color: var(--color-text-primary);
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    padding: 4px var(--spacing-md);
    border-radius: var(--radius-full);
  }

  /* ── Warning banner ── */

  .warning-banner {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-accent-copper) 8%, transparent);
    color: var(--color-accent-copper);
    font-size: var(--font-size-sm);
    margin-top: var(--spacing-md);
  }

  /* ── Cleanup ── */

  .cleanup-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-shrink: 0;
  }

  .cleanup-result {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin-top: var(--spacing-md);
  }

  /* ── Buttons ── */

  .btn-pill {
    padding: 6px var(--spacing-lg);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-full);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
  }

  .btn-pill:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-text-tertiary);
  }

  .btn-pill:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
    border-color: var(--color-border-focus);
  }

  .btn-pill:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-pill-danger {
    color: var(--color-error);
    border-color: color-mix(in srgb, var(--color-error) 30%, transparent);
  }

  .btn-pill-danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border-color: var(--color-error);
  }

  .btn-danger-pill {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 8px var(--spacing-lg);
    border: 1px solid color-mix(in srgb, var(--color-error) 30%, transparent);
    border-radius: var(--radius-full);
    background: transparent;
    color: var(--color-error);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-danger-pill:hover {
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border-color: var(--color-error);
  }

  .btn-danger-pill:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-error) 25%, transparent);
  }

  .btn-export {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px var(--spacing-lg);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-export:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-text-tertiary);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

  .btn-export:focus-visible {
    outline: none;
    box-shadow: var(--shadow-input-focus);
    border-color: var(--color-border-focus);
  }

  .btn-export:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .export-buttons {
    display: flex;
    gap: var(--spacing-sm);
  }
</style>
