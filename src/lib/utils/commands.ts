/** Typed wrappers around Tauri invoke() for all backend commands. */

import { invoke } from "@tauri-apps/api/core";
import type { AuthState, DeviceCodeResponse, GitHubUser } from "$lib/types/auth";
import type { Conversation } from "$lib/types/conversation";
import type { Message, ChatMessage, Model } from "$lib/types/message";
import type { SearchResult, ExtractedContent } from "$lib/types/web-research";
import type {
  McpConnectionInfo,
  McpServerConfig,
  McpToolInfo,
  McpToolResult,
  CatalogEntry,
  RegistryServer,
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
export async function sendMessage(messages: ChatMessage[], model: string): Promise<void> {
  return invoke("send_message", { messages, model });
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

/** Get the built-in MCP server catalog. */
export async function getMcpCatalog(): Promise<CatalogEntry[]> {
  return invoke<CatalogEntry[]>("get_mcp_catalog");
}

/** Fetch servers from the official MCP Registry. */
export async function fetchMcpRegistry(count?: number): Promise<RegistryServer[]> {
  return invoke<RegistryServer[]>("fetch_mcp_registry", { count });
}
