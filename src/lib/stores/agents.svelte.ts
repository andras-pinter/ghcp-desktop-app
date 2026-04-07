/** Reactive agents state using Svelte 5 runes. */

import {
  getAgents as getAgentsCmd,
  createAgent as createAgentCmd,
  updateAgent as updateAgentCmd,
  deleteAgent as deleteAgentCmd,
  setAgentSkills as setAgentSkillsCmd,
  setAgentMcpConnections as setAgentMcpCmd,
  logFrontend,
} from "$lib/utils/commands";
import type { Agent } from "$lib/types/agent";

let agents = $state<Agent[]>([]);
let loaded = $state(false);
let selectedAgentId = $state<string | null>(null);

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
  };
}
