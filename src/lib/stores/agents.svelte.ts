/** Reactive agents state using Svelte 5 runes. */

import {
  getAgents as getAgentsCmd,
  createAgent as createAgentCmd,
  updateAgent as updateAgentCmd,
  deleteAgent as deleteAgentCmd,
  setAgentSkills as setAgentSkillsCmd,
  setAgentMcpConnections as setAgentMcpCmd,
  installAgentFromRegistry as installAgentFromRegistryCmd,
  searchCatalog as searchCatalogCmd,
  installCatalogItem as installCatalogItemCmd,
  logFrontend,
} from "$lib/utils/commands";
import type { Agent } from "$lib/types/agent";
import type { RegistryItem, RegistrySearchResult } from "$lib/types/registry";
import { SvelteSet } from "svelte/reactivity";

let agents = $state<Agent[]>([]);
let loaded = $state(false);
let selectedAgentId = $state<string | null>(null);

// ── Registry state ──────────────────────────────────────────────
let registryResults = $state<RegistryItem[]>([]);
let registrySearching = $state(false);
let registryQuery = $state("");

/** Source filter selection (persists across component mount/unmount). */
const selectedSourceIds = new SvelteSet<string>();

/** Load agents from the backend. Call once after auth. */
export async function initAgents(): Promise<void> {
  try {
    agents = await getAgentsCmd();
  } catch (e) {
    logFrontend("error", `initAgents failed: ${e}`);
    agents = [];
  } finally {
    loaded = true;
  }
}

/** Create a new agent and add it to the store. */
export async function addAgent(
  name: string,
  systemPrompt: string,
  avatar?: string | null,
): Promise<Agent> {
  const agent = await createAgentCmd(name, systemPrompt, avatar);
  agents = [...agents, agent];
  return agent;
}

/** Update an agent and refresh the store. */
export async function editAgent(
  id: string,
  name: string,
  systemPrompt: string,
  avatar?: string | null,
): Promise<void> {
  await updateAgentCmd(id, name, systemPrompt, avatar);
  agents = agents.map((a) =>
    a.id === id ? { ...a, name, systemPrompt, avatar: avatar ?? a.avatar } : a,
  );
}

/** Delete an agent and remove from the store. */
export async function removeAgent(id: string): Promise<void> {
  await deleteAgentCmd(id);
  agents = agents.filter((a) => a.id !== id);
  if (selectedAgentId === id) {
    selectedAgentId = null;
  }
}

/** Assign skills to an agent. */
export async function assignAgentSkills(agentId: string, skillIds: string[]): Promise<void> {
  await setAgentSkillsCmd(agentId, skillIds);
}

/** Assign MCP connections to an agent. */
export async function assignAgentMcp(agentId: string, mcpServerIds: string[]): Promise<void> {
  await setAgentMcpCmd(agentId, mcpServerIds);
}

/** Set the selected agent for new conversations. */
export function selectAgent(agentId: string | null): void {
  selectedAgentId = agentId;
}

/** Find the default agent. */
export function getDefaultAgent(): Agent | undefined {
  return agents.find((a) => a.isDefault);
}

// ── Registry ────────────────────────────────────────────────────

/** Cached browse results so clearing search restores instantly. */
let browseCache: RegistryItem[] = [];

/** Pending request queued while a search is in-flight. */
let pendingRequest: { query: string; sourceIds?: string[] | null } | null = null;

/** Prefetch popular agents from registries (browse mode with empty query). */
export async function prefetchAgentRegistry(sourceIds?: string[] | null): Promise<void> {
  // If we have cached browse results and no source filter, restore instantly
  if (browseCache.length > 0 && (!sourceIds || sourceIds.length === 0)) {
    registryQuery = "";
    registryResults = browseCache;
    return;
  }
  if (registrySearching) {
    pendingRequest = { query: "", sourceIds };
    return;
  }
  registrySearching = true;
  try {
    const result: RegistrySearchResult = await searchCatalogCmd("", "agent", 200, sourceIds);
    if (!sourceIds || sourceIds.length === 0) {
      browseCache = result.items;
    }
    registryQuery = "";
    registryResults = result.items;
  } catch (e) {
    logFrontend("warn", `Agent registry prefetch failed: ${e}`);
  } finally {
    registrySearching = false;
    drainPendingRequest();
  }
}

/** Search registries for agents and update results. */
export async function searchAgentRegistries(
  query: string,
  sourceIds?: string[] | null,
): Promise<void> {
  if (registrySearching) {
    pendingRequest = { query, sourceIds };
    return;
  }
  registryQuery = query;
  registrySearching = true;
  try {
    const result: RegistrySearchResult = await searchCatalogCmd(query, "agent", null, sourceIds);
    registryResults = result.items;
  } catch (e) {
    logFrontend("error", `Agent registry search failed: ${e}`);
    registryResults = [];
  } finally {
    registrySearching = false;
    drainPendingRequest();
  }
}

/** Fire the latest queued request after the current one finishes. */
function drainPendingRequest(): void {
  if (!pendingRequest) return;
  const { query, sourceIds } = pendingRequest;
  pendingRequest = null;
  if (query) {
    searchAgentRegistries(query, sourceIds);
  } else {
    prefetchAgentRegistry(sourceIds);
  }
}

/** Install an agent from a catalog result (aitmpl.com or git source). */
export async function installAgentFromRegistry(item: RegistryItem): Promise<Agent | null> {
  try {
    if (item.source === "git") {
      // Git source catalog item — install via catalog command
      await installCatalogItemCmd(item.id);
    } else {
      // aitmpl.com registry item — install via existing registry command
      await installAgentFromRegistryCmd(
        item.id,
        item.source,
        item.sourceRepo,
        item.url,
        item.content,
        item.name,
      );
    }
    // Reload agents to pick up the new one
    await initAgents();
    // Invalidate browse cache so next prefetch includes updated installed status
    browseCache = [];
    return agents[agents.length - 1] ?? null;
  } catch (e) {
    logFrontend("error", `Agent registry install failed: ${e}`);
    return null;
  }
}

// ── Git Import ── (removed — git import now handled via Sources panel)

/** Invalidate the browse cache so next prefetch fetches fresh data. */
export function invalidateAgentCatalogCache(): void {
  browseCache = [];
  registryResults = [];
}

/** Reactive getters. */
export function getAgentStore() {
  return {
    get agents() {
      return agents;
    },
    get loaded() {
      return loaded;
    },
    get selectedAgentId() {
      return selectedAgentId;
    },
    get registryResults() {
      return registryResults;
    },
    get registrySearching() {
      return registrySearching;
    },
    get registryQuery() {
      return registryQuery;
    },
    get selectedSourceIds() {
      return selectedSourceIds;
    },
  };
}
