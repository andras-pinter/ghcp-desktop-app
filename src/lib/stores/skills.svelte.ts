/** Reactive skills state using Svelte 5 runes. */

import {
  getSkills as getSkillsCmd,
  createSkill as createSkillCmd,
  deleteSkill as deleteSkillCmd,
  toggleSkill as toggleSkillCmd,
  searchCatalog as searchCatalogCmd,
  installFromRegistry as installFromRegistryCmd,
  installCatalogItem as installCatalogItemCmd,
  logFrontend,
} from "$lib/utils/commands";
import type { Skill } from "$lib/types/skill";
import type { RegistryItem, RegistrySearchResult } from "$lib/types/registry";
import { SvelteSet } from "svelte/reactivity";

const PAGE_SIZE = 30;

let skills = $state<Skill[]>([]);
let loaded = $state(false);

// Registry search state
let registryQuery = $state("");
let registryResults = $state<RegistryItem[]>([]);
let registrySearching = $state(false);
let registryTotal = $state<number | null>(null);
let registryHasMore = $state(false);
let registryLoadingMore = $state(false);
let registryOffset = $state(0);

/** Source filter selection (persists across component mount/unmount). */
const selectedSourceIds = new SvelteSet<string>();

/** Load skills from the backend. Call once after auth. */
export async function initSkills(): Promise<void> {
  try {
    skills = await getSkillsCmd();
  } catch (e) {
    logFrontend("error", `initSkills failed: ${e}`);
    skills = [];
  } finally {
    loaded = true;
  }
}

/** Toggle a skill's enabled state. */
export async function toggle(id: string, enabled: boolean): Promise<void> {
  await toggleSkillCmd(id, enabled);
  skills = skills.map((s) => (s.id === id ? { ...s, enabled } : s));
}

/** Delete a skill and remove from the store. */
export async function removeSkill(id: string): Promise<void> {
  await deleteSkillCmd(id);
  skills = skills.filter((s) => s.id !== id);
}

// ── Registry ────────────────────────────────────────────────────

/** Cached browse results (first page) so clearing search restores instantly. */
let browseCache: RegistryItem[] = [];
let browseCacheHasMore = false;

/** Pending request queued while a search is in-flight. */
let pendingRequest: { query: string; sourceIds?: string[] | null } | null = null;

/** Prefetch skills from catalog (browse mode with empty query). */
export async function prefetchRegistry(sourceIds?: string[] | null): Promise<void> {
  // If we have cached browse results and no source filter, restore instantly
  if (browseCache.length > 0 && (!sourceIds || sourceIds.length === 0)) {
    registryQuery = "";
    registryResults = browseCache;
    registryTotal = browseCache.length;
    registryHasMore = browseCacheHasMore;
    registryOffset = browseCache.length;
    return;
  }
  if (registrySearching) {
    pendingRequest = { query: "", sourceIds };
    return;
  }
  registrySearching = true;
  registryOffset = 0;
  try {
    const result: RegistrySearchResult = await searchCatalogCmd(
      "",
      "skill",
      PAGE_SIZE,
      sourceIds,
      0,
    );
    if (!sourceIds || sourceIds.length === 0) {
      browseCache = result.items;
      browseCacheHasMore = result.hasMore;
    }
    registryQuery = "";
    registryResults = result.items;
    registryTotal = result.total ?? result.items.length;
    registryHasMore = result.hasMore;
    registryOffset = result.items.length;
  } catch (e) {
    logFrontend("warn", `Skills registry prefetch failed: ${e}`);
  } finally {
    registrySearching = false;
    drainPendingRequest();
  }
}

/** Search catalog and update results (resets to first page). */
export async function searchRegistries(query: string, sourceIds?: string[] | null): Promise<void> {
  if (registrySearching) {
    pendingRequest = { query, sourceIds };
    return;
  }
  registryQuery = query;
  registrySearching = true;
  registryOffset = 0;
  try {
    const result: RegistrySearchResult = await searchCatalogCmd(
      query,
      "skill",
      PAGE_SIZE,
      sourceIds,
      0,
    );
    registryResults = result.items;
    registryTotal = result.total ?? result.items.length;
    registryHasMore = result.hasMore;
    registryOffset = result.items.length;
  } catch (e) {
    logFrontend("error", `Registry search failed: ${e}`);
    registryResults = [];
    registryTotal = null;
    registryHasMore = false;
  } finally {
    registrySearching = false;
    drainPendingRequest();
  }
}

/** Load the next page of results and append to the current list. */
export async function loadMoreSkills(sourceIds?: string[] | null): Promise<void> {
  if (registryLoadingMore || !registryHasMore) return;
  registryLoadingMore = true;
  try {
    const result: RegistrySearchResult = await searchCatalogCmd(
      registryQuery,
      "skill",
      PAGE_SIZE,
      sourceIds,
      registryOffset,
    );
    registryResults = [...registryResults, ...result.items];
    registryHasMore = result.hasMore;
    registryOffset += result.items.length;
  } catch (e) {
    logFrontend("error", `Load more skills failed: ${e}`);
  } finally {
    registryLoadingMore = false;
  }
}

/** Fire the latest queued request after the current one finishes. */
function drainPendingRequest(): void {
  if (!pendingRequest) return;
  const { query, sourceIds } = pendingRequest;
  pendingRequest = null;
  if (query) {
    searchRegistries(query, sourceIds);
  } else {
    prefetchRegistry(sourceIds);
  }
}

/** Install a skill from a catalog result (aitmpl.com or git source). */
export async function installFromRegistry(item: RegistryItem): Promise<Skill | null> {
  try {
    if (item.source === "git") {
      // Git source catalog item — install via catalog command
      await installCatalogItemCmd(item.id);
    } else {
      // aitmpl.com registry item — install via registry command
      await installFromRegistryCmd(
        item.id,
        item.source,
        item.sourceRepo,
        item.url,
        item.content,
        item.name,
      );
    }
    // Reload skills to pick up the new one
    await initSkills();
    // Invalidate browse cache so next prefetch includes updated installed status
    browseCache = [];
    return skills[skills.length - 1] ?? null;
  } catch (e) {
    logFrontend("error", `Registry install failed: ${e}`);
    return null;
  }
}

// ── Git Import ── (removed — git import now handled via Sources panel)

/** Create a manual skill (not from registry/git). */
export async function addManualSkill(
  id: string,
  name: string,
  description: string | null,
  instructions: string | null,
): Promise<Skill> {
  const skill = await createSkillCmd(
    id,
    name,
    description,
    "builtin",
    null,
    null,
    instructions,
    null,
    "builtin",
  );
  skills = [...skills, skill];
  return skill;
}

/** Invalidate the browse cache so next prefetch fetches fresh data. */
export function invalidateSkillCatalogCache(): void {
  browseCache = [];
  browseCacheHasMore = false;
  registryResults = [];
  registryTotal = null;
  registryHasMore = false;
  registryOffset = 0;
}

/** Reactive getters. */
export function getSkillStore() {
  return {
    get skills() {
      return skills;
    },
    get loaded() {
      return loaded;
    },
    get registryQuery() {
      return registryQuery;
    },
    get registryResults() {
      return registryResults;
    },
    get registrySearching() {
      return registrySearching;
    },
    get registryTotal() {
      return registryTotal;
    },
    get registryHasMore() {
      return registryHasMore;
    },
    get registryLoadingMore() {
      return registryLoadingMore;
    },
    get selectedSourceIds() {
      return selectedSourceIds;
    },
  };
}
