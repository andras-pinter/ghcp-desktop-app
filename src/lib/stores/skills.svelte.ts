/** Reactive skills state using Svelte 5 runes. */

import {
  getSkills as getSkillsCmd,
  createSkill as createSkillCmd,
  deleteSkill as deleteSkillCmd,
  toggleSkill as toggleSkillCmd,
  searchRegistry as searchRegistryCmd,
  installFromRegistry as installFromRegistryCmd,
  fetchGitSkills as fetchGitSkillsCmd,
  importGitSkill as importGitSkillCmd,
  logFrontend,
} from "$lib/utils/commands";
import type { Skill } from "$lib/types/skill";
import type { RegistryItem, GitSkillFile, RegistrySearchResult } from "$lib/types/registry";

let skills = $state<Skill[]>([]);
let loaded = $state(false);

// Registry search state
let registryQuery = $state("");
let registryResults = $state<RegistryItem[]>([]);
let registrySearching = $state(false);
let registryTotal = $state<number | null>(null);

// Git import state
let gitImportUrl = $state("");
let gitDiscoveredFiles = $state<GitSkillFile[]>([]);
let gitImporting = $state(false);
let gitProgress = $state<{ total: number; fetched: number; phase: string } | null>(null);

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

/** Cached browse results so clearing search restores instantly. */
let browseCache: RegistryItem[] = [];

/** Prefetch popular skills from registries (browse mode with empty query). */
export async function prefetchRegistry(): Promise<void> {
  // If we have cached browse results, restore them instantly
  if (browseCache.length > 0) {
    registryQuery = "";
    registryResults = browseCache;
    registryTotal = browseCache.length;
    return;
  }
  if (registrySearching) return;
  registrySearching = true;
  try {
    const result: RegistrySearchResult = await searchRegistryCmd("", 200);
    browseCache = result.items.filter((i) => i.kind === "skill");
    registryQuery = "";
    registryResults = browseCache;
    registryTotal = browseCache.length;
  } catch (e) {
    logFrontend("warn", `Skills registry prefetch failed: ${e}`);
  } finally {
    registrySearching = false;
  }
}

/** Search registries and update results. */
export async function searchRegistries(query: string): Promise<void> {
  if (registrySearching) return;
  registryQuery = query;
  registrySearching = true;
  try {
    const result: RegistrySearchResult = await searchRegistryCmd(query);
    const filtered = result.items.filter((i) => i.kind === "skill");
    registryResults = filtered;
    registryTotal = filtered.length;
  } catch (e) {
    logFrontend("error", `Registry search failed: ${e}`);
    registryResults = [];
    registryTotal = null;
  } finally {
    registrySearching = false;
  }
}

/** Install a skill from a registry result. */
export async function installFromRegistry(item: RegistryItem): Promise<Skill | null> {
  try {
    const installed: RegistryItem = await installFromRegistryCmd(
      item.id,
      item.source,
      item.sourceRepo,
      item.url,
      item.content,
      item.name,
    );
    // Reload skills to pick up the new one
    await initSkills();
    return skills.find((s) => s.id === installed.id) ?? null;
  } catch (e) {
    logFrontend("error", `Registry install failed: ${e}`);
    return null;
  }
}

// ── Git Import ──────────────────────────────────────────────────

/** Fetch SKILL.md files from a git URL. */
export async function discoverGitSkills(url: string): Promise<void> {
  gitImportUrl = url;
  gitImporting = true;
  gitProgress = null;
  try {
    gitDiscoveredFiles = await fetchGitSkillsCmd(url);
  } catch (e) {
    logFrontend("error", `Git skill discovery failed: ${e}`);
    gitDiscoveredFiles = [];
    throw e;
  } finally {
    gitImporting = false;
    gitProgress = null;
  }
}

/** Import a discovered SKILL.md file as a skill. */
export async function importFromGit(file: GitSkillFile): Promise<Skill | null> {
  try {
    const skill = await importGitSkillCmd(file.content, file.repoUrl, file.path);
    // Reload to pick up the new skill
    await initSkills();
    return skills.find((s) => s.id === skill.id) ?? null;
  } catch (e) {
    logFrontend("error", `Git import failed: ${e}`);
    return null;
  }
}

/** Clear git import state. */
export function clearGitImport(): void {
  gitImportUrl = "";
  gitDiscoveredFiles = [];
  gitProgress = null;
}

/** Update git import progress (called from event listener). */
export function updateGitProgress(
  progress: {
    total: number;
    fetched: number;
    phase: string;
  } | null,
): void {
  gitProgress = progress;
}

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
    get gitImportUrl() {
      return gitImportUrl;
    },
    get gitDiscoveredFiles() {
      return gitDiscoveredFiles;
    },
    get gitImporting() {
      return gitImporting;
    },
    get gitProgress() {
      return gitProgress;
    },
  };
}
