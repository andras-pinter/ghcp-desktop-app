/** Typed wrappers around Tauri invoke() for all backend commands. */

import { invoke } from "@tauri-apps/api/core";
import type { AuthState, DeviceCodeResponse, GitHubUser } from "$lib/types/auth";
import type { Conversation } from "$lib/types/conversation";
import type { Message, ChatMessage, Model } from "$lib/types/message";
import type { SearchResult, ExtractedContent } from "$lib/types/web-research";
import type { Agent } from "$lib/types/agent";
import type { Skill } from "$lib/types/skill";
import type { RegistrySearchResult, GitSkillFile, RegistryItem } from "$lib/types/registry";
import type {
  McpConnectionInfo,
  McpServerConfig,
  McpToolInfo,
  McpToolResult,
  RegistryPage,
} from "$lib/types/mcp";

// ── Logging ─────────────────────────────────────────────────────

/** Log a message from the frontend to the Rust console. Best-effort — IPC failures are ignored. */
export function logFrontend(level: "info" | "warn" | "error" | "debug", message: string): void {
  invoke("log_frontend", { level, message }).catch(() => {});
}

// ── Auth ────────────────────────────────────────────────────────

/** Check current authentication state. */
export async function getAuthState(): Promise<AuthState> {
  return invoke<AuthState>("get_auth_state");
}

/** Start the GitHub OAuth device flow. */
export async function authenticate(): Promise<DeviceCodeResponse> {
  return invoke<DeviceCodeResponse>("authenticate");
}

/** Poll once for the OAuth token after user authorizes. */
export async function pollAuthToken(deviceCode: string): Promise<GitHubUser> {
  return invoke<GitHubUser>("poll_auth_token", { deviceCode });
}

/** Sign out and clear stored tokens. */
export async function doLogout(): Promise<void> {
  return invoke("logout");
}

// ── Chat ────────────────────────────────────────────────────────

/** Send chat messages and start streaming the response. */
export async function sendMessage(
  messages: ChatMessage[],
  model: string,
  agentId?: string | null,
): Promise<void> {
  return invoke("send_message", { messages, model, agentId: agentId ?? null });
}

/** Cancel an in-flight streaming response. */
export async function stopStreaming(): Promise<void> {
  return invoke("stop_streaming");
}

// ── Models ──────────────────────────────────────────────────────

/** Fetch available Copilot models. */
export async function getModels(): Promise<Model[]> {
  return invoke<Model[]>("get_models");
}

// ── Conversations ───────────────────────────────────────────────

/** List all conversations (ordered by most recent). */
export async function getConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>("get_conversations");
}

/** Get a single conversation by ID. */
export async function getConversation(id: string): Promise<Conversation | null> {
  return invoke<Conversation | null>("get_conversation", { id });
}

/** Create a new conversation. */
export async function createConversation(
  id: string,
  title?: string | null,
  agentId?: string | null,
  projectId?: string | null,
  model?: string | null,
): Promise<Conversation> {
  return invoke<Conversation>("create_conversation", {
    id,
    title: title ?? null,
    agentId: agentId ?? null,
    projectId: projectId ?? null,
    model: model ?? null,
  });
}

/** Update a conversation's fields. */
export async function updateConversation(
  id: string,
  title?: string | null,
  isFavourite?: boolean | null,
  model?: string | null,
): Promise<void> {
  return invoke("update_conversation", {
    id,
    title: title ?? null,
    isFavourite: isFavourite ?? null,
    model: model ?? null,
  });
}

/** Delete a conversation (messages + drafts cascade). */
export async function deleteConversation(id: string): Promise<void> {
  return invoke("delete_conversation", { id });
}

// ── Messages ────────────────────────────────────────────────────

/** Get all messages for a conversation. */
export async function getMessages(conversationId: string): Promise<Message[]> {
  return invoke<Message[]>("get_messages", { conversationId });
}

/** Insert a message into the database. */
export async function createMessage(message: Message): Promise<void> {
  return invoke("create_message", { message });
}

/** Update a message's content (after streaming or edit). */
export async function updateMessageContent(
  id: string,
  content: string,
  thinkingContent?: string | null,
): Promise<void> {
  return invoke("update_message_content", {
    id,
    content,
    thinkingContent: thinkingContent ?? null,
  });
}

/** Delete all messages after a given sort order (for editing). */
export async function deleteMessagesAfter(
  conversationId: string,
  afterSortOrder: number,
): Promise<void> {
  return invoke("delete_messages_after", { conversationId, afterSortOrder });
}

// ── Settings ────────────────────────────────────────────────────

/** Get a config value. */
export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

/** Set a config value. */
export async function updateSetting(key: string, value: string): Promise<void> {
  return invoke("update_setting", { key, value });
}

/** Get the database file size in bytes. */
export async function getDbSize(): Promise<number> {
  return invoke<number>("get_db_size");
}

// ── Drafts ──────────────────────────────────────────────────────

/** Save a draft for a conversation. */
export async function saveDraft(conversationId: string, content: string): Promise<void> {
  return invoke("save_draft", { conversationId, content });
}

/** Get the draft for a conversation, if any. */
export async function getDraft(
  conversationId: string,
): Promise<{ conversationId: string; content: string; updatedAt: string } | null> {
  return invoke("get_draft", { conversationId });
}

/** Delete the draft for a conversation. */
export async function deleteDraft(conversationId: string): Promise<void> {
  return invoke("delete_draft", { conversationId });
}

// ── Web Research ────────────────────────────────────────────────

/** Search the web via Bing Web Search API. API key is read from keychain by backend. */
export async function webSearch(query: string, count?: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("web_search", { query, count: count ?? null });
}

/** Fetch a URL and extract its readable content. */
export async function fetchUrl(url: string): Promise<ExtractedContent> {
  return invoke<ExtractedContent>("fetch_url", { url });
}

// ── MCP ─────────────────────────────────────────────────────────

/** List all configured MCP servers with connection status. */
export async function getMcpServers(): Promise<McpConnectionInfo[]> {
  return invoke<McpConnectionInfo[]>("get_mcp_servers");
}

/** Add a new MCP server. */
export async function addMcpServer(config: McpServerConfig): Promise<McpConnectionInfo> {
  return invoke<McpConnectionInfo>("add_mcp_server", { config });
}

/** Update an existing MCP server configuration. */
export async function updateMcpServer(config: McpServerConfig): Promise<void> {
  return invoke("update_mcp_server", { config });
}

/** Remove an MCP server. */
export async function removeMcpServer(serverId: string): Promise<void> {
  return invoke("remove_mcp_server", { serverId });
}

/** Connect to an MCP server. */
export async function connectMcpServer(serverId: string): Promise<McpConnectionInfo> {
  return invoke<McpConnectionInfo>("connect_mcp_server", { serverId });
}

/** Disconnect from an MCP server. */
export async function disconnectMcpServer(serverId: string): Promise<void> {
  return invoke("disconnect_mcp_server", { serverId });
}

/** Test an MCP server connection. Returns the number of tools discovered. */
export async function testMcpConnection(config: McpServerConfig): Promise<number> {
  return invoke<number>("test_mcp_connection", { config });
}

/** Get discovered tools from a connected MCP server. */
export async function getMcpTools(serverId: string): Promise<McpToolInfo[]> {
  return invoke<McpToolInfo[]>("get_mcp_tools", { serverId });
}

/** Invoke a tool on a connected MCP server. */
export async function invokeMcpTool(
  serverId: string,
  toolName: string,
  arguments_?: Record<string, unknown> | null,
): Promise<McpToolResult> {
  return invoke<McpToolResult>("invoke_mcp_tool", {
    serverId,
    toolName,
    arguments: arguments_ ?? null,
  });
}

/** Fetch a page of servers from the official MCP Registry.
 *  If `query` is provided, performs server-side search by name.
 *  If `cursor` is provided, fetches the next page. */
export async function fetchMcpRegistry(query?: string, cursor?: string): Promise<RegistryPage> {
  return invoke<RegistryPage>("fetch_mcp_registry", {
    query: query ?? null,
    cursor: cursor ?? null,
  });
}

// ── Agents ──────────────────────────────────────────────────────

/** List all agents. */
export async function getAgents(): Promise<Agent[]> {
  return invoke<Agent[]>("get_agents");
}

/** Get a single agent by ID. */
export async function getAgent(id: string): Promise<Agent> {
  return invoke<Agent>("get_agent", { id });
}

/** Create a new agent. */
export async function createAgent(
  name: string,
  systemPrompt: string,
  avatar?: string | null,
  sourceUrl?: string | null,
  sourceType?: string | null,
): Promise<Agent> {
  return invoke<Agent>("create_agent", {
    name,
    systemPrompt,
    avatar: avatar ?? null,
    sourceUrl: sourceUrl ?? null,
    sourceType: sourceType ?? null,
  });
}

/** Update an agent's fields. */
export async function updateAgent(
  id: string,
  name: string,
  systemPrompt: string,
  avatar?: string | null,
  sourceUrl?: string | null,
  sourceType?: string | null,
): Promise<void> {
  return invoke("update_agent", {
    id,
    name,
    systemPrompt,
    avatar: avatar ?? null,
    sourceUrl: sourceUrl ?? null,
    sourceType: sourceType ?? null,
  });
}

/** Delete an agent. Cannot delete the default agent. */
export async function deleteAgent(id: string): Promise<void> {
  return invoke("delete_agent", { id });
}

/** Set skills assigned to an agent (replaces existing). */
export async function setAgentSkills(agentId: string, skillIds: string[]): Promise<void> {
  return invoke("set_agent_skills", { agentId, skillIds });
}

/** Set MCP connections assigned to an agent (replaces existing). */
export async function setAgentMcpConnections(
  agentId: string,
  mcpServerIds: string[],
): Promise<void> {
  return invoke("set_agent_mcp_connections", { agentId, mcpServerIds });
}

/** Install an agent from a registry (skills.sh or aitmpl.com). */
export async function installAgentFromRegistry(
  itemId: string,
  source: string,
  sourceRepo?: string | null,
): Promise<Agent> {
  return invoke<Agent>("install_agent_from_registry", {
    itemId,
    source,
    sourceRepo: sourceRepo ?? null,
  });
}

/** Import an agent from a git SKILL.md file. */
export async function importAgentFromGit(
  content: string,
  repoUrl: string,
  path: string,
): Promise<Agent> {
  return invoke<Agent>("import_agent_from_git", { content, repoUrl, path });
}

// ── Skills ──────────────────────────────────────────────────────

/** List all skills. */
export async function getSkills(): Promise<Skill[]> {
  return invoke<Skill[]>("get_skills");
}

/** Create a skill. */
export async function createSkill(
  id: string,
  name: string,
  description: string | null,
  source: string,
  mcpServerId: string | null,
  config: string | null,
  instructions: string | null,
  sourceUrl: string | null,
  sourceType: string,
): Promise<Skill> {
  return invoke<Skill>("create_skill", {
    id,
    name,
    description,
    source,
    mcpServerId,
    config,
    instructions,
    sourceUrl,
    sourceType,
  });
}

/** Update a skill. */
export async function updateSkill(
  id: string,
  name: string,
  description: string | null,
  config: string | null,
  instructions: string | null,
): Promise<void> {
  return invoke("update_skill", { id, name, description, config, instructions });
}

/** Delete a skill. */
export async function deleteSkill(id: string): Promise<void> {
  return invoke("delete_skill", { id });
}

/** Toggle a skill's enabled state. */
export async function toggleSkill(id: string, enabled: boolean): Promise<void> {
  return invoke("toggle_skill", { id, enabled });
}

// ── Registry ────────────────────────────────────────────────────

/** Search both skill registries (skills.sh + aitmpl.com). */
export async function searchRegistry(query: string, limit?: number): Promise<RegistrySearchResult> {
  return invoke<RegistrySearchResult>("search_registry", {
    query,
    limit: limit ?? null,
  });
}

/** Install a skill from a registry. */
export async function installFromRegistry(
  skillId: string,
  source: string,
  sourceRepo?: string | null,
): Promise<RegistryItem> {
  return invoke<RegistryItem>("install_from_registry", {
    skillId,
    source,
    sourceRepo: sourceRepo ?? null,
  });
}

// ── Git Import ──────────────────────────────────────────────────

/** Fetch SKILL.md files from a git repository URL. */
export async function fetchGitSkills(gitUrl: string): Promise<GitSkillFile[]> {
  return invoke<GitSkillFile[]>("fetch_git_skills", { gitUrl });
}

/** Import a parsed SKILL.md content as a skill. */
export async function importGitSkill(
  content: string,
  repoUrl: string,
  path: string,
): Promise<Skill> {
  return invoke<Skill>("import_git_skill", { content, repoUrl, path });
}
