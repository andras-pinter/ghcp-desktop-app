/** Typed wrappers around Tauri invoke() for all backend commands. */

import { invoke } from "@tauri-apps/api/core";
import type { AuthState, AuthenticateResult, GitHubUser } from "$lib/types/auth";
import type { Conversation } from "$lib/types/conversation";
import type { Message, ChatMessage, Model } from "$lib/types/message";
import type { SearchResult, ExtractedContent } from "$lib/types/web-research";
import type { Agent } from "$lib/types/agent";
import type { Skill } from "$lib/types/skill";
import type { Project, ProjectFile, FileUpload, ChatFileData } from "$lib/types/project";
import type { RegistrySearchResult, RegistryItem } from "$lib/types/registry";
import type { GitSource, SourceItem, ImportItem, SourceScanResult } from "$lib/types/source";
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
  invoke("log_frontend", { level, message }).catch((e) =>
    console.warn("logFrontend IPC failed:", e),
  );
}

// ── Auth ────────────────────────────────────────────────────────

/** Check current authentication state. */
export async function getAuthState(): Promise<AuthState> {
  return invoke<AuthState>("get_auth_state");
}

/** Start the GitHub OAuth device flow. */
export async function authenticate(): Promise<AuthenticateResult> {
  return invoke<AuthenticateResult>("authenticate");
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
  conversationId: string,
  messages: ChatMessage[],
  model: string,
  agentId?: string | null,
  projectId?: string | null,
): Promise<void> {
  return invoke("send_message", {
    conversationId,
    messages,
    model,
    agentId: agentId ?? null,
    projectId: projectId ?? null,
  });
}

/** Cancel an in-flight streaming response for a conversation (or all if omitted). */
export async function stopStreaming(conversationId?: string | null): Promise<void> {
  return invoke("stop_streaming", {
    conversationId: conversationId ?? null,
  });
}

/** Generate a conversation title via the AI from the first exchange. */
export async function generateConversationTitle(
  userMessage: string,
  assistantMessage: string,
  model: string,
): Promise<string> {
  return invoke<string>("generate_title", { userMessage, assistantMessage, model });
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
  projectId?: string | null | undefined,
): Promise<void> {
  return invoke("update_conversation", {
    id,
    title: title ?? null,
    isFavourite: isFavourite ?? null,
    model: model ?? null,
    projectId: projectId === undefined ? null : projectId,
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

/** Delete conversations older than X days. Returns count deleted. */
export async function deleteOldConversations(olderThanDays: number): Promise<number> {
  return invoke<number>("delete_old_conversations", { olderThanDays });
}

/** Export a single conversation as JSON string. */
export async function exportConversationJson(id: string): Promise<string> {
  return invoke<string>("export_conversation_json", { id });
}

/** Export a single conversation as Markdown string. */
export async function exportConversationMarkdown(id: string): Promise<string> {
  return invoke<string>("export_conversation_markdown", { id });
}

/** Export all conversations as JSON string. */
export async function exportAllConversationsJson(): Promise<string> {
  return invoke<string>("export_all_conversations_json");
}

/** Export all conversations as Markdown string. */
export async function exportAllConversationsMarkdown(): Promise<string> {
  return invoke<string>("export_all_conversations_markdown");
}

/** Save exported content to a user-chosen file via native save dialog (server-side). */
export async function saveExportFile(content: string, defaultName: string): Promise<void> {
  return invoke("save_export_file", { content, defaultName });
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

/** Test an MCP server connection. Returns the number of tools discovered.
 *  Uses server_id to load real config from DB+keychain (avoids redacted auth). */
export async function testMcpConnection(serverId: string): Promise<number> {
  return invoke<number>("test_mcp_connection", { serverId });
}

/** Test an MCP server using a raw config (for unsaved servers in the add form). */
export async function testMcpConnectionConfig(config: McpServerConfig): Promise<number> {
  return invoke<number>("test_mcp_connection_config", { config });
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

/** Approve an MCP stdio binary for execution. */
export async function approveMcpBinary(binaryPath: string): Promise<void> {
  return invoke("approve_mcp_binary", { binaryPath });
}

/** Check if an MCP stdio binary is approved. */
export async function isMcpBinaryApproved(binaryPath: string): Promise<boolean> {
  return invoke<boolean>("is_mcp_binary_approved", { binaryPath });
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

/** Install an agent from a registry (aitmpl.com). */
export async function installAgentFromRegistry(
  itemId: string,
  source: string,
  sourceRepo?: string | null,
  itemUrl?: string | null,
  itemContent?: string | null,
  itemName?: string | null,
): Promise<Agent> {
  return invoke<Agent>("install_agent_from_registry", {
    itemId,
    source,
    sourceRepo: sourceRepo ?? null,
    itemUrl: itemUrl ?? null,
    itemContent: itemContent ?? null,
    itemName: itemName ?? null,
  });
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

/** Search skill registries (aitmpl.com). */
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
  itemUrl?: string | null,
  itemContent?: string | null,
  itemName?: string | null,
): Promise<RegistryItem> {
  return invoke<RegistryItem>("install_from_registry", {
    skillId,
    source,
    sourceRepo: sourceRepo ?? null,
    itemUrl: itemUrl ?? null,
    itemContent: itemContent ?? null,
    itemName: itemName ?? null,
  });
}

// ── Git Sources ─────────────────────────────────────────────────

/** List all git sources. */
export async function getGitSources(): Promise<GitSource[]> {
  return invoke<GitSource[]>("get_git_sources");
}

/** Get a single git source by ID. */
export async function getGitSource(id: string): Promise<GitSource | null> {
  return invoke<GitSource | null>("get_git_source", { id });
}

/** Create a new git source and scan the repository for skills/agents. */
export async function createGitSource(
  url: string,
  name?: string | null,
): Promise<SourceScanResult> {
  return invoke<SourceScanResult>("create_git_source", { url, name: name ?? null });
}

/** Update a git source's name and/or enabled state. */
export async function updateGitSource(
  id: string,
  name?: string | null,
  enabled?: boolean | null,
): Promise<GitSource> {
  return invoke<GitSource>("update_git_source", {
    id,
    name: name ?? null,
    enabled: enabled ?? null,
  });
}

/** Delete a git source. Imported items are kept as local copies. */
export async function deleteGitSource(id: string): Promise<void> {
  return invoke<void>("delete_git_source", { id });
}

/** Re-sync a git source: re-fetch the repo and return discovered files. */
export async function syncGitSource(id: string): Promise<SourceScanResult> {
  return invoke<SourceScanResult>("sync_git_source", { id });
}

/** Import selected items from a source scan. */
export async function importSourceItems(sourceId: string, items: ImportItem[]): Promise<void> {
  return invoke<void>("import_source_items", { sourceId, items });
}

/** Auto-sync all enabled sources (called on app launch). */
export async function syncAllSources(): Promise<void> {
  return invoke<void>("sync_all_sources");
}

/** List skills and agents linked to a specific source. */
export async function getSourceItems(sourceId: string): Promise<SourceItem[]> {
  return invoke<SourceItem[]>("get_source_items", { sourceId });
}

// ── Projects ────────────────────────────────────────────────────

/** List all projects. */
export async function getProjects(): Promise<Project[]> {
  return invoke<Project[]>("get_projects");
}

/** Get a single project by ID. */
export async function getProject(id: string): Promise<Project | null> {
  return invoke<Project | null>("get_project", { id });
}

/** Create a new project. */
export async function createProject(
  id: string,
  name: string,
  instructions?: string | null,
): Promise<Project> {
  return invoke<Project>("create_project", {
    id,
    name,
    instructions: instructions ?? null,
  });
}

/** Update a project's name and/or instructions. */
export async function updateProject(
  id: string,
  name?: string | null,
  instructions?: string | null | undefined,
): Promise<void> {
  return invoke("update_project", {
    id,
    name: name ?? null,
    instructions: instructions === undefined ? null : instructions,
  });
}

/** Delete a project (files cascade, conversations unlinked). */
export async function deleteProject(id: string): Promise<void> {
  return invoke("delete_project", { id });
}

/** List files attached to a project (metadata only). */
export async function getProjectFiles(projectId: string): Promise<ProjectFile[]> {
  return invoke<ProjectFile[]>("get_project_files", { projectId });
}

/** Upload a file to a project (base64-encoded content). */
export async function addProjectFile(projectId: string, file: FileUpload): Promise<ProjectFile> {
  return invoke<ProjectFile>("add_project_file", { projectId, file });
}

/** Get a project file's content as base64. */
export async function getProjectFileContent(fileId: string): Promise<string | null> {
  return invoke<string | null>("get_project_file_content", { fileId });
}

/** Remove a file from a project. */
export async function removeProjectFile(fileId: string): Promise<void> {
  return invoke("remove_project_file", { fileId });
}

/** List conversations belonging to a project. */
export async function getProjectConversations(projectId: string): Promise<Conversation[]> {
  return invoke<Conversation[]>("get_project_conversations", { projectId });
}

/** Open a native file dialog and read the selected file for project upload. Returns null if cancelled. */
export async function pickFileForUpload(): Promise<FileUpload | null> {
  return invoke<FileUpload | null>("pick_file_for_upload");
}

/** Open a native file dialog and read the selected file for chat attachments. Returns null if cancelled. */
export async function pickFileForChat(): Promise<ChatFileData | null> {
  return invoke<ChatFileData | null>("pick_file_for_chat");
}

/** Extract readable text from a base64-encoded file (PDF, DOCX, XLSX, PPTX, RTF, text, etc.). */
export async function extractFileText(
  contentBase64: string,
  contentType: string,
  name: string,
): Promise<string | null> {
  return invoke<string | null>("extract_file_text", { contentBase64, contentType, name });
}

/** Read files from OS-level drag-and-drop paths via Tauri backend.
 *  Paths are validated server-side against the OS drag-drop event (no separate registration needed). */
export async function readDroppedFiles(
  paths: string[],
): Promise<import("$lib/types/project").ChatFileData[]> {
  return invoke<import("$lib/types/project").ChatFileData[]>("read_dropped_files", { paths });
}
