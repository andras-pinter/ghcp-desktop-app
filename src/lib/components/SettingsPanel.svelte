<script lang="ts">
  import { getSettings, updateSetting, SETTING_KEYS } from "$lib/stores/settings.svelte";
  import { getAuth, logout } from "$lib/stores/auth.svelte";
  import { getModelStore } from "$lib/stores/models.svelte";
  import { getAgentStore, initAgents } from "$lib/stores/agents.svelte";
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
  const agentStore = getAgentStore();

  let dbSize = $state<number | null>(null);
  let cleanupDays = $state(90);
  let cleanupResult = $state<string | null>(null);
  let exporting = $state(false);
  let cleaningUp = $state(false);

  onMount(() => {
    loadDbSize();
    if (!agentStore.loaded) initAgents();
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

<div class="panel">
  <header class="panel-header" data-tauri-drag-region>
    <button class="panel-back" onclick={onBack} aria-label="Go back">
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
    <h2 class="panel-title">Settings</h2>
  </header>

  <div class="panel-body-narrow">
    <!-- ═══════════════════ ACCOUNT ═══════════════════ -->
    <h2 class="section-heading">Account</h2>
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

      <button class="btn btn--danger btn--pill" onclick={handleLogout}>
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

    <!-- ═══════════════════ APPEARANCE ═══════════════════ -->
    <h2 class="section-heading">Appearance</h2>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Theme</span>
          <span class="setting-desc">Choose how Chuck looks</span>
        </div>
        <select
          id="theme-select"
          class="form-select"
          value={settings.theme}
          onchange={(e) => updateSetting(SETTING_KEYS.theme, (e.target as HTMLSelectElement).value)}
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
          class="form-select"
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

    <!-- ═══════════════════ INPUT ═══════════════════ -->
    <h2 class="section-heading">Input</h2>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Send message with</span>
          <span class="setting-desc">How to send messages in the chat input</span>
        </div>
        <div class="segmented">
          <button
            class="segmented-item"
            class:segmented-item--active={settings.sendShortcut === "enter"}
            onclick={() => updateSetting(SETTING_KEYS.sendShortcut, "enter")}
            aria-pressed={settings.sendShortcut === "enter"}>Enter</button
          >
          <button
            class="segmented-item"
            class:segmented-item--active={settings.sendShortcut === "cmd-enter"}
            onclick={() => updateSetting(SETTING_KEYS.sendShortcut, "cmd-enter")}
            aria-pressed={settings.sendShortcut === "cmd-enter"}>⌘ Enter</button
          >
        </div>
      </div>
    </div>

    <!-- ═══════════════════ DEFAULTS ═══════════════════ -->
    <h2 class="section-heading">Defaults</h2>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Default agent</span>
          <span class="setting-desc">Pre-selected agent for new conversations</span>
        </div>
        <select
          id="default-agent"
          class="form-select"
          value={settings.defaultAgentId}
          onchange={(e) => {
            const val = (e.target as HTMLSelectElement).value;
            if (val) updateSetting(SETTING_KEYS.defaultAgentId, val);
          }}
        >
          {#each agentStore.agents as agent (agent.id)}
            <option value={agent.id}>
              {agent.avatar ?? "🤖"}
              {agent.name}{agent.isDefault ? " (built-in)" : ""}
            </option>
          {/each}
        </select>
      </div>

      {#if modelStore.models.length > 1}
        <div class="setting-divider"></div>

        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Default model</span>
            <span class="setting-desc">Used for new conversations</span>
          </div>
          <select
            id="default-model"
            class="form-select"
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
    </div>

    <!-- ═══════════════════ AUTO-UPDATE ═══════════════════ -->
    <h2 class="section-heading">Auto-Update</h2>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Check for updates</span>
          <span class="setting-desc">Automatically check for new versions</span>
        </div>
        <label class="toggle">
          <input
            id="auto-update-toggle"
            type="checkbox"
            checked={settings.autoUpdateEnabled}
            onchange={(e) =>
              updateSetting(
                SETTING_KEYS.autoUpdateEnabled,
                String((e.target as HTMLInputElement).checked),
              )}
          />
          <span class="toggle-track"></span>
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
            class="form-select"
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
              class="btn btn--pill"
              onclick={() => updateSetting(SETTING_KEYS.skippedVersion, "")}
            >
              Clear skip
            </button>
          </div>
        {/if}
      {/if}
    </div>

    <!-- ═══════════════════ KEYBOARD SHORTCUTS ═══════════════════ -->
    <h2 class="section-heading">Keyboard Shortcuts</h2>
    <div class="settings-card">
      <div class="shortcuts-list" role="list" aria-label="Keyboard shortcuts">
        {#each shortcuts as shortcut, i (shortcut.keys)}
          {#if i > 0}
            <div class="setting-divider"></div>
          {/if}
          <div class="shortcut-row" role="listitem">
            <span class="shortcut-action">{shortcut.action}</span>
            <kbd class="kbd">{shortcut.keys}</kbd>
          </div>
        {/each}
      </div>
    </div>

    <!-- ═══════════════════ STORAGE ═══════════════════ -->
    <h2 class="section-heading">Storage</h2>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Database size</span>
          <span class="setting-desc">Local conversation storage</span>
        </div>
        <span class="badge badge--neutral">
          {dbSize !== null ? formatBytes(dbSize) : "…"}
        </span>
      </div>

      {#if dbSize !== null && dbSize > 400 * 1024 * 1024}
        <div class="banner banner--warning" role="alert">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1l7 13H1L8 1zm0 4v4m0 2v1" />
          </svg>
          Database is {formatBytes(dbSize)} — consider cleaning up.
        </div>
      {/if}
    </div>

    <!-- ═══════════════════ CLEANUP ═══════════════════ -->
    <h2 class="section-heading">Cleanup</h2>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Delete old conversations</span>
          <span class="setting-desc">Permanently remove conversations older than</span>
        </div>
        <div class="cleanup-controls">
          <select id="cleanup-days" class="form-select" bind:value={cleanupDays}>
            <option value={30}>30 days</option>
            <option value={60}>60 days</option>
            <option value={90}>90 days</option>
            <option value={180}>180 days</option>
            <option value={365}>1 year</option>
          </select>
          <button class="btn btn--danger btn--pill" onclick={handleCleanup} disabled={cleaningUp}>
            {cleaningUp ? "Deleting…" : "Delete"}
          </button>
        </div>
      </div>

      {#if cleanupResult}
        <p class="cleanup-result">{cleanupResult}</p>
      {/if}
    </div>

    <!-- ═══════════════════ EXPORT ═══════════════════ -->
    <h2 class="section-heading">Export</h2>
    <div class="settings-card">
      <p class="card-desc">Export all conversations to a file for backup or sharing.</p>

      <div class="export-buttons">
        <button class="btn" onclick={() => handleExport("json")} disabled={exporting}>
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
        <button class="btn" onclick={() => handleExport("markdown")} disabled={exporting}>
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
  </div>
</div>

<style>
  /* ── Panel overrides ── */
  .panel {
    animation: fadeIn 200ms ease both;
  }

  /* ── Content area override ── */
  .panel-body-narrow {
    padding: var(--spacing-lg) var(--spacing-xl) var(--spacing-2xl);
    max-width: 95%;
  }

  /* ── Section heading override (above card) ── */
  .section-heading {
    margin: var(--spacing-lg) 0 var(--spacing-sm);
  }

  .section-heading:first-child {
    margin-top: 0;
  }

  /* ── Setting row override ── */
  .setting-row {
    gap: var(--spacing-xl);
    min-height: 36px;
    padding: 0;
  }

  /* ── Settings card (component-specific) ── */
  .settings-card {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border-secondary);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg) var(--spacing-xl);
  }

  .card-desc {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-lg);
    line-height: 1.5;
  }

  .setting-divider {
    height: 1px;
    background: var(--color-border-secondary);
    margin: var(--spacing-md) 0;
  }

  /* ── Badge override (DB size display) ── */
  .settings-card .badge {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    font-variant-numeric: tabular-nums;
    padding: 4px var(--spacing-md);
    border-radius: var(--radius-full);
  }

  /* ── Banner override ── */
  .banner {
    margin-top: var(--spacing-md);
  }

  /* ── Account section ── */
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

  /* ── Shortcuts section ── */
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

  /* ── Cleanup section ── */
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

  /* ── Export section ── */
  .export-buttons {
    display: flex;
    gap: var(--spacing-sm);
  }
</style>
