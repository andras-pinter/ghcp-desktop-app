<script lang="ts">
  import { check, type Update, type DownloadEvent } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { getSettings, updateSetting, SETTING_KEYS } from "$lib/stores/settings.svelte";
  import { renderMarkdown } from "$lib/utils/markdown";
  import { onMount } from "svelte";

  const settings = getSettings();

  /** Possible states of the update lifecycle. */
  type UpdateState =
    | { kind: "idle" }
    | { kind: "checking" }
    | { kind: "available"; update: Update; version: string; body: string; date: string | null }
    | { kind: "downloading"; progress: number; total: number | null }
    | { kind: "ready" }
    | { kind: "error"; message: string };

  let updateState: UpdateState = $state({ kind: "idle" });
  let dismissed = $state(false);
  let lastCheckTime = $state(0);

  /** Duration (ms) for "remind me later" snooze — 24 hours. */
  const SNOOZE_DURATION_MS = 24 * 60 * 60 * 1000;

  /** Check interval thresholds in milliseconds. */
  const CHECK_INTERVALS: Record<string, number> = {
    startup: 0,
    daily: 24 * 60 * 60 * 1000,
    weekly: 7 * 24 * 60 * 60 * 1000,
  };

  /** Whether the banner should be visible. */
  let visible = $derived(
    !dismissed &&
      updateState.kind !== "idle" &&
      (updateState as { kind: string }).kind !== "checking",
  );

  onMount(() => {
    checkForUpdates();
  });

  /** Determine if enough time has passed since last check based on frequency setting. */
  function shouldCheck(): boolean {
    if (!settings.autoUpdateEnabled) return false;

    const interval = CHECK_INTERVALS[settings.autoUpdateFrequency] ?? 0;
    if (interval === 0) return true;

    const now = Date.now();
    return now - lastCheckTime >= interval;
  }

  /** Check if the available version is snoozed or skipped. */
  function isVersionSuppressed(version: string): boolean {
    if (settings.skippedVersion === version) return true;

    if (settings.updateSnoozedUntil) {
      const snoozedUntil = new Date(settings.updateSnoozedUntil).getTime();
      if (Date.now() < snoozedUntil) return true;
    }

    return false;
  }

  /** Clear stale suppression settings that no longer apply to the given version. */
  async function clearStaleSuppression(version: string): Promise<void> {
    if (settings.skippedVersion && settings.skippedVersion !== version) {
      await updateSetting(SETTING_KEYS.skippedVersion, "");
    }
    if (settings.updateSnoozedUntil) {
      const snoozedUntil = new Date(settings.updateSnoozedUntil).getTime();
      if (Date.now() >= snoozedUntil) {
        await updateSetting(SETTING_KEYS.updateSnoozedUntil, "");
      }
    }
  }

  /** Check for available updates. */
  export async function checkForUpdates(): Promise<void> {
    if (!shouldCheck()) return;

    updateState = { kind: "checking" };
    lastCheckTime = Date.now();

    try {
      const update = await check();

      if (update) {
        await clearStaleSuppression(update.version);

        if (isVersionSuppressed(update.version)) {
          updateState = { kind: "idle" };
          return;
        }

        updateState = {
          kind: "available",
          update,
          version: update.version,
          body: update.body ?? "",
          date: update.date ?? null,
        };
      } else {
        updateState = { kind: "idle" };
      }
    } catch (e) {
      console.warn("Update check failed:", e);
      // Silently fail — don't show error for routine checks
      updateState = { kind: "idle" };
    }
  }

  /** Download and install the available update. */
  async function handleUpdateNow(): Promise<void> {
    if (updateState.kind !== "available") return;

    const { update } = updateState;

    updateState = { kind: "downloading", progress: 0, total: null };

    try {
      let contentLength: number | null = null;
      let downloaded = 0;

      await update.downloadAndInstall((event: DownloadEvent) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? null;
            updateState = { kind: "downloading", progress: 0, total: contentLength };
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            updateState = { kind: "downloading", progress: downloaded, total: contentLength };
            break;
          case "Finished":
            updateState = { kind: "ready" };
            break;
        }
      });

      updateState = { kind: "ready" };
    } catch (e) {
      updateState = { kind: "error", message: String(e) };
    }
  }

  /** Restart the app to apply the update. */
  async function handleRestart(): Promise<void> {
    await relaunch();
  }

  /** Skip this version permanently (until a newer release). */
  async function handleSkip(): Promise<void> {
    if (updateState.kind === "available") {
      await updateSetting(SETTING_KEYS.skippedVersion, updateState.version);
    }
    dismissed = true;
  }

  /** Snooze the update notification for 24 hours. */
  async function handleRemindLater(): Promise<void> {
    const snoozedUntil = new Date(Date.now() + SNOOZE_DURATION_MS).toISOString();
    await updateSetting(SETTING_KEYS.updateSnoozedUntil, snoozedUntil);
    dismissed = true;
  }

  /** Dismiss the banner. */
  function handleDismiss(): void {
    dismissed = true;
  }

  /** Format download progress as a percentage. */
  function progressPercent(progress: number, total: number | null): string {
    if (!total || total === 0) return "Downloading…";
    const pct = Math.round((progress / total) * 100);
    return `${pct}%`;
  }

  /** Format bytes for download display. */
  function formatMB(bytes: number): string {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

{#if visible}
  <div class="update-banner" role="status" aria-live="polite">
    {#if updateState.kind === "available"}
      <div class="banner-content">
        <svg class="banner-icon" width="16" height="16" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.5" />
          <path
            d="M8 4v5M6 7l2 2 2-2"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <circle cx="8" cy="12" r="0.5" fill="currentColor" />
        </svg>
        <span class="banner-text">
          Version <strong>{updateState.version}</strong> is available
        </span>
        <div class="banner-actions">
          <button class="btn-banner btn-primary-banner" onclick={handleUpdateNow}>
            Update now
          </button>
          <button class="btn-banner btn-ghost" onclick={handleSkip}> Skip </button>
          <button class="btn-banner btn-ghost" onclick={handleRemindLater}> Later </button>
          <button class="btn-dismiss" onclick={handleDismiss} aria-label="Dismiss">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M4 4l6 6M10 4l-6 6"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
      </div>

      {#if updateState.body}
        <details class="changelog">
          <summary class="changelog-toggle">View changes</summary>
          <!-- eslint-disable-next-line svelte/no-at-html-tags -- sanitized by renderMarkdown/DOMPurify -->
          <div class="changelog-body">{@html renderMarkdown(updateState.body)}</div>
        </details>
      {/if}
    {:else if updateState.kind === "downloading"}
      <div class="banner-content">
        <div class="spinner-small"></div>
        <span class="banner-text">
          Downloading update…
          {#if updateState.total}
            {progressPercent(updateState.progress, updateState.total)}
            ({formatMB(updateState.progress)} / {formatMB(updateState.total)})
          {/if}
        </span>
      </div>
      {#if updateState.total}
        <div class="progress-track">
          <div
            class="progress-fill"
            style:width="{Math.round((updateState.progress / updateState.total) * 100)}%"
          ></div>
        </div>
      {/if}
    {:else if updateState.kind === "ready"}
      <div class="banner-content">
        <svg class="banner-icon success" width="16" height="16" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.5" />
          <path
            d="M5 8l2 2 4-4"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span class="banner-text">Update downloaded. Restart to apply.</span>
        <div class="banner-actions">
          <button class="btn-banner btn-primary-banner" onclick={handleRestart}>
            Restart now
          </button>
          <button class="btn-banner btn-ghost" onclick={handleDismiss}> Later </button>
        </div>
      </div>
    {:else if updateState.kind === "error"}
      <div class="banner-content">
        <svg class="banner-icon error" width="16" height="16" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.5" />
          <path
            d="M8 5v3M8 10.5v.5"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
        <span class="banner-text">Update failed: {updateState.message}</span>
        <div class="banner-actions">
          <button class="btn-banner btn-ghost" onclick={checkForUpdates}> Retry </button>
          <button class="btn-dismiss" onclick={handleDismiss} aria-label="Dismiss">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M4 4l6 6M10 4l-6 6"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .update-banner {
    flex-shrink: 0;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border-secondary);
    padding: var(--spacing-sm) var(--spacing-lg);
    animation: slideDown 250ms ease both;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-100%);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .banner-content {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    min-height: 28px;
  }

  .banner-icon {
    flex-shrink: 0;
    color: var(--color-accent-copper);
  }

  .banner-icon.success {
    color: var(--color-success, #16a34a);
  }

  .banner-icon.error {
    color: var(--color-error, #dc2626);
  }

  .banner-text {
    flex: 1;
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }

  .banner-text strong {
    font-weight: var(--font-weight-semibold);
    color: var(--color-accent-copper);
  }

  .banner-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    flex-shrink: 0;
  }

  .btn-banner {
    padding: 3px var(--spacing-md);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all var(--transition-fast);
    white-space: nowrap;
    border: none;
  }

  .btn-primary-banner {
    background: var(--color-text-primary);
    color: var(--color-bg-primary);
  }

  .btn-primary-banner:hover {
    opacity: 0.85;
  }

  .btn-ghost {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .btn-ghost:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .btn-dismiss {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .btn-dismiss:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  /* ── Changelog ── */

  .changelog {
    margin-top: var(--spacing-xs);
  }

  .changelog-toggle {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    cursor: pointer;
    user-select: none;
    padding: var(--spacing-xs) 0;
  }

  .changelog-toggle:hover {
    color: var(--color-accent-copper);
  }

  .changelog-body {
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    line-height: 1.6;
    padding: var(--spacing-sm) var(--spacing-md);
    margin-top: var(--spacing-xs);
    background: var(--color-bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-secondary);
    max-height: 160px;
    overflow-y: auto;
  }

  .changelog-body :global(h2),
  .changelog-body :global(h3) {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    margin: var(--spacing-sm) 0 var(--spacing-xs);
  }

  .changelog-body :global(h2:first-child),
  .changelog-body :global(h3:first-child) {
    margin-top: 0;
  }

  .changelog-body :global(ul) {
    margin: var(--spacing-xs) 0;
    padding-left: var(--spacing-lg);
  }

  .changelog-body :global(li) {
    margin-bottom: 2px;
  }

  /* ── Progress bar ── */

  .progress-track {
    height: 3px;
    background: var(--color-border-secondary);
    border-radius: 2px;
    margin-top: var(--spacing-xs);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-accent-copper);
    border-radius: 2px;
    transition: width 200ms ease;
  }

  /* ── Spinner ── */

  .spinner-small {
    width: 16px;
    height: 16px;
    border: 2px solid var(--color-border-primary);
    border-top-color: var(--color-accent-copper);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
