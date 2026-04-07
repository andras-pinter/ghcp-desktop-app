/** Reactive agents state using Svelte 5 runes. */

import {
  getAgents as getAgentsCmd,
  createAgent as createAgentCmd,
  updateAgent as updateAgentCmd,
  deleteAgent as deleteAgentCmd,
  setAgentSkills as setAgentSkillsCmd,
  setAgentMcpConnections as setAgentMcpCmd,
  installAgentFromRegistry as installAgentFromRegistryCmd,
  importAgentFromGit as importAgentFromGitCmd,
  logFrontend,
} from "$lib/utils/commands";
import { searchRegistry as searchRegistryCmd } from "$lib/utils/commands";
import type { Agent } from "$lib/types/agent";
import type { RegistryItem, RegistrySearchResult, GitSkillFile } from "$lib/types/registry";

let agents = $state<Agent[]>([]);
let loaded = $state(false);
let selectedAgentId = $state<string | null>(null);

// ── Registry state ──────────────────────────────────────────────
let registryResults = $state<RegistryItem[]>([]);
let registrySearching = $state(false);
let registryQuery = $state("");

// ── Git import state ────────────────────────────────────────────
let gitDiscoveredFiles = $state<GitSkillFile[]>([]);
let gitImporting = $state(false);
let gitImportUrl = $state("");
let gitProgress = $state<{ total: number; fetched: number; phase: string } | null>(null);

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

/** Search registries for agents and update results. */
export async function searchAgentRegistries(query: string): Promise<void> {
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

/** Clear registry search state. */
export function clearAgentRegistrySearch(): void {
  registryQuery = "";
  registryResults = [];
}

// ── Git Import ──────────────────────────────────────────────────

/** Fetch agent definition files from a git URL. */
export async function discoverGitAgents(url: string): Promise<void> {
  const { fetchGitAgents } = await import("$lib/utils/commands");
  gitImportUrl = url;
  gitImporting = true;
  gitProgress = null;
  try {
    gitDiscoveredFiles = await fetchGitAgents(url);
  } catch (e) {
    logFrontend("error", `Git agent discovery failed: ${e}`);
    gitDiscoveredFiles = [];
    throw e;
  } finally {
    gitImporting = false;
    gitProgress = null;
  }
}

/** Import a discovered agent definition file. */
export async function importAgentFromGit(file: GitSkillFile): Promise<Agent | null> {
  try {
    const agent = await importAgentFromGitCmd(file.content, file.repoUrl, file.path);
    agents = [...agents, agent];
    return agent;
  } catch (e) {
    logFrontend("error", `Git agent import failed: ${e}`);
    return null;
  }
}

/** Clear git import state. */
export function clearAgentGitImport(): void {
  gitImportUrl = "";
  gitDiscoveredFiles = [];
  gitProgress = null;
}

/** Update git import progress (called from event listener). */
export function updateAgentGitProgress(
  progress: {
    total: number;
    fetched: number;
    phase: string;
  } | null,
): void {
  gitProgress = progress;
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
    get gitDiscoveredFiles() {
      return gitDiscoveredFiles;
    },
    get gitImporting() {
      return gitImporting;
    },
    get gitImportUrl() {
      return gitImportUrl;
    },
    get gitProgress() {
      return gitProgress;
    },
  };
}
