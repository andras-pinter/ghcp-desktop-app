/** Reactive git sources state using Svelte 5 runes. */

import {
  getGitSources as getGitSourcesCmd,
  getGitSource as getGitSourceCmd,
  createGitSource as createGitSourceCmd,
  updateGitSource as updateGitSourceCmd,
  deleteGitSource as deleteGitSourceCmd,
  syncGitSource as syncGitSourceCmd,
  syncAllSources as syncAllSourcesCmd,
  importSourceItems as importSourceItemsCmd,
  getSourceItems as getSourceItemsCmd,
  logFrontend,
} from "$lib/utils/commands";
import type { GitSource, SourceItem, ImportItem, SourceScanResult } from "$lib/types/source";
import { SvelteSet } from "svelte/reactivity";

let sources = $state<GitSource[]>([]);
let loaded = $state(false);

/** Per-source syncing flag (keyed by source ID). */
const syncing: Record<string, boolean> = $state({});

/** Whether syncAllEnabled is running. */
let syncingAll = $state(false);

/** Per-source expanded items (keyed by source ID). */
const expandedItems: Record<string, SourceItem[]> = $state({});

/** Per-source expanded toggle. */
const expandedIds = new SvelteSet<string>();

/** Scan results after adding or syncing a source (for the import picker). */
let scanResult = $state<SourceScanResult | null>(null);

/** Progress during scan (emitted via git-import-progress event). */
let scanProgress = $state<{ total: number; fetched: number; phase: string } | null>(null);

/** Per-source sync progress (keyed by source ID). */
const syncProgress: Record<string, { total: number; fetched: number; phase: string }> = $state({});

/** Whether we're currently creating/scanning a new source. */
let adding = $state(false);

// ── Initialization ──────────────────────────────────────────────

/** Load sources from the backend. Call once after auth. */
export async function initSources(): Promise<void> {
  try {
    sources = await getGitSourcesCmd();
  } catch (e) {
    logFrontend("error", `initSources failed: ${e}`);
    sources = [];
  } finally {
    loaded = true;
  }
}

// ── CRUD ────────────────────────────────────────────────────────

/** Create a new git source and scan the repository. Returns scan result for the import picker. */
export async function addSource(url: string, name?: string | null): Promise<SourceScanResult> {
  adding = true;
  scanProgress = null;
  try {
    const result = await createGitSourceCmd(url, name);
    scanResult = result;
    sources = [...sources, result.source];
    return result;
  } finally {
    adding = false;
  }
}

/** Toggle a source's enabled state. */
export async function toggleSource(id: string, enabled: boolean): Promise<void> {
  const updated = await updateGitSourceCmd(id, null, enabled);
  sources = sources.map((s) => (s.id === id ? updated : s));
}

/** Update a source's display name. */
export async function renameSource(id: string, name: string): Promise<void> {
  const updated = await updateGitSourceCmd(id, name, null);
  sources = sources.map((s) => (s.id === id ? updated : s));
}

/** Delete a source. Imported items are kept as local copies. */
export async function removeSource(id: string): Promise<void> {
  await deleteGitSourceCmd(id);
  sources = sources.filter((s) => s.id !== id);
  // Clean up expanded state
  delete expandedItems[id];
  expandedIds.delete(id);
}

// ── Sync ────────────────────────────────────────────────────────

/** Manual re-sync of a single source. Returns scan result for the import picker. */
export async function syncSource(id: string): Promise<SourceScanResult> {
  syncing[id] = true;
  scanProgress = null;
  try {
    const result = await syncGitSourceCmd(id);
    scanResult = result;
    // Update source metadata in the list
    sources = sources.map((s) => (s.id === id ? result.source : s));
    return result;
  } finally {
    syncing[id] = false;
  }
}

/** Auto-sync all enabled sources (silent, called on app launch). */
export async function syncAllEnabled(): Promise<void> {
  syncingAll = true;
  try {
    await syncAllSourcesCmd();
    // Refresh the full list to pick up updated metadata
    sources = await getGitSourcesCmd();
  } catch (e) {
    logFrontend("warn", `syncAllSources failed: ${e}`);
  } finally {
    syncingAll = false;
  }
}

// ── Import ──────────────────────────────────────────────────────

/** Import selected items from a scan result. */
export async function importItems(sourceId: string, items: ImportItem[]): Promise<void> {
  await importSourceItemsCmd(sourceId, items);
  // Refresh source to update item count
  try {
    const updated = await getGitSourcesCmd();
    sources = updated;
  } catch {
    // Non-critical — the list still shows the previous count
  }
}

// ── Expand / Items ──────────────────────────────────────────────

/** Toggle expanded state and load items if needed. */
export async function toggleExpand(sourceId: string): Promise<void> {
  if (expandedIds.has(sourceId)) {
    expandedIds.delete(sourceId);
    return;
  }
  // Load items if not cached
  if (!expandedItems[sourceId]) {
    try {
      expandedItems[sourceId] = await getSourceItemsCmd(sourceId);
    } catch (e) {
      logFrontend("error", `getSourceItems failed: ${e}`);
      expandedItems[sourceId] = [];
    }
  }
  expandedIds.add(sourceId);
}

/** Refresh items for an expanded source (e.g., after import). */
export async function refreshItems(sourceId: string): Promise<void> {
  try {
    expandedItems[sourceId] = await getSourceItemsCmd(sourceId);
  } catch (e) {
    logFrontend("error", `refreshItems failed: ${e}`);
  }
}

// ── Scan progress ───────────────────────────────────────────────

/** Update scan progress (called from git-import-progress event listener). */
export function updateScanProgress(
  total: number,
  fetched: number,
  phase: string,
  sourceId?: string,
): void {
  scanProgress = { total, fetched, phase };
  if (sourceId) {
    if (fetched >= total && total > 0) {
      delete syncProgress[sourceId];
    } else {
      syncProgress[sourceId] = { total, fetched, phase };
    }
  }
}

/** Handle sync completion event — refresh source metadata from DB. */
export function handleSyncComplete(sourceId: string): void {
  delete syncProgress[sourceId];
  getGitSourceCmd(sourceId).then((updated) => {
    if (updated) {
      sources = sources.map((s) => (s.id === sourceId ? updated : s));
    }
  });
}

/** Clear scan result and progress (e.g., after import or cancel). */
export function clearScanResult(): void {
  scanResult = null;
  scanProgress = null;
}

// ── Reactive getters ────────────────────────────────────────────

export function getSourceStore() {
  return {
    get sources() {
      return sources;
    },
    get loaded() {
      return loaded;
    },
    get syncing() {
      return syncing;
    },
    get expandedItems() {
      return expandedItems;
    },
    get expandedIds() {
      return expandedIds;
    },
    get scanResult() {
      return scanResult;
    },
    get scanProgress() {
      return scanProgress;
    },
    get syncProgress() {
      return syncProgress;
    },
    get adding() {
      return adding;
    },
    get anySyncing() {
      return syncingAll || Object.values(syncing).some(Boolean);
    },
  };
}
