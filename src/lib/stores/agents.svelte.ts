/** Reactive agents state using Svelte 5 runes. */

import {
  getAgents as getAgentsCmd,
  createAgent as createAgentCmd,
  updateAgent as updateAgentCmd,
  deleteAgent as deleteAgentCmd,
  setAgentSkills as setAgentSkillsCmd,
  setAgentMcpConnections as setAgentMcpCmd,
  installAgentFromRegistry as installAgentFromRegistryCmd,
  logFrontend,
} from "$lib/utils/commands";
import { searchRegistry as searchRegistryCmd } from "$lib/utils/commands";
import type { Agent } from "$lib/types/agent";
import type { RegistryItem, RegistrySearchResult } from "$lib/types/registry";

let agents = $state<Agent[]>([]);
let loaded = $state(false);
let selectedAgentId = $state<string | null>(null);

// ── Registry state ──────────────────────────────────────────────
let registryResults = $state<RegistryItem[]>([]);
let registrySearching = $state(false);
let registryQuery = $state("");

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

/** Prefetch popular agents from registries (browse mode with empty query). */
export async function prefetchAgentRegistry(): Promise<void> {
  // If we have cached browse results, restore them instantly
  if (browseCache.length > 0) {
    registryQuery = "";
    registryResults = browseCache;
    return;
  }
  if (registrySearching) return;
  registrySearching = true;
  try {
    const result: RegistrySearchResult = await searchRegistryCmd("", 200);
    browseCache = result.items.filter((i) => i.kind === "agent");
    registryQuery = "";
    registryResults = browseCache;
  } catch (e) {
    logFrontend("warn", `Agent registry prefetch failed: ${e}`);
  } finally {
    registrySearching = false;
  }
}

/** Search registries for agents and update results. */
export async function searchAgentRegistries(query: string): Promise<void> {
  if (registrySearching) return;
  registryQuery = query;
  registrySearching = true;
  try {
    const result: RegistrySearchResult = await searchRegistryCmd(query);
    // Filter to agent-kind items only
    registryResults = result.items.filter((i) => i.kind === "agent");
  } catch (e) {
    logFrontend("error", `Agent registry search failed: ${e}`);
    registryResults = [];
  } finally {
    registrySearching = false;
  }
}

/** Install an agent from a registry result. */
export async function installAgentFromRegistry(item: RegistryItem): Promise<Agent | null> {
  try {
    const agent = await installAgentFromRegistryCmd(
      item.id,
      item.source,
      item.sourceRepo,
      item.url,
      item.content,
      item.name,
    );
    agents = [...agents, agent];
    return agent;
  } catch (e) {
    logFrontend("error", `Agent registry install failed: ${e}`);
    return null;
  }
}

// ── Git Import ── (removed — git import now handled via Sources panel)

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
  };
}
