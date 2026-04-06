/** MCP server connection state management. */

import type {
  McpConnectionInfo,
  McpServerConfig,
  McpToolInfo,
  RegistryServer,
} from "$lib/types/mcp";
import {
  getMcpServers,
  addMcpServer,
  updateMcpServer,
  removeMcpServer,
  connectMcpServer,
  disconnectMcpServer,
  testMcpConnection,
  getMcpTools,
  fetchMcpRegistry,
} from "$lib/utils/commands";
import { logFrontend } from "$lib/utils/commands";

// ── State ───────────────────────────────────────────────────────

/** All configured MCP servers with live connection status. */
let servers = $state<McpConnectionInfo[]>([]);

/** Servers from the official MCP Registry. */
let registry = $state<RegistryServer[]>([]);

/** Whether the registry is being fetched. */
let registryLoading = $state(false);

/** Whether the store is loading. */
let loading = $state(false);

/** Last error message. */
let error = $state<string | null>(null);

// ── Init ────────────────────────────────────────────────────────

/** Load MCP servers from backend. */
export async function initMcp(): Promise<void> {
  loading = true;
  error = null;
  try {
    servers = await getMcpServers();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    error = msg;
    logFrontend("error", `Failed to load MCP servers: ${msg}`);
  } finally {
    loading = false;
  }
}

/** Fetch MCP servers from the official registry (initial browse). */
export async function loadRegistry(): Promise<void> {
  if (registryLoading || registry.length > 0) return;
  registryLoading = true;
  try {
    registry = await fetchMcpRegistry();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    logFrontend("warn", `Failed to fetch MCP registry: ${msg}`);
  } finally {
    registryLoading = false;
  }
}

/** Search the MCP registry with a server-side query. */
export async function searchRegistry(query: string): Promise<void> {
  registryLoading = true;
  try {
    registry = await fetchMcpRegistry(query || undefined);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    logFrontend("warn", `Failed to search MCP registry: ${msg}`);
  } finally {
    registryLoading = false;
  }
}

// ── Actions ─────────────────────────────────────────────────────

/** Add a new MCP server and return its connection info. */
export async function addServer(config: McpServerConfig): Promise<McpConnectionInfo | null> {
  try {
    const info = await addMcpServer(config);
    servers = [...servers, info];
    return info;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    logFrontend("error", `Failed to add MCP server: ${msg}`);
    throw e;
  }
}

/** Update an existing server's configuration. */
export async function editServer(config: McpServerConfig): Promise<void> {
  await updateMcpServer(config);
  servers = servers.map((s) => (s.config.id === config.id ? { ...s, config } : s));
}

/** Remove a server. */
export async function removeServer(serverId: string): Promise<void> {
  await removeMcpServer(serverId);
  servers = servers.filter((s) => s.config.id !== serverId);
}

/** Connect to a server. */
export async function connectServer(serverId: string): Promise<void> {
  // Optimistic: set to connecting
  servers = servers.map((s) =>
    s.config.id === serverId ? { ...s, status: "connecting" as const, error: null } : s,
  );

  try {
    const info = await connectMcpServer(serverId);
    servers = servers.map((s) => (s.config.id === serverId ? info : s));
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    servers = servers.map((s) =>
      s.config.id === serverId ? { ...s, status: "error" as const, error: msg } : s,
    );
    throw e;
  }
}

/** Disconnect from a server. */
export async function disconnectServer(serverId: string): Promise<void> {
  await disconnectMcpServer(serverId);
  servers = servers.map((s) =>
    s.config.id === serverId
      ? { ...s, status: "disconnected" as const, error: null, toolCount: 0, tools: null }
      : s,
  );
}

/** Test a server connection. Returns discovered tool count. */
export async function testConnection(config: McpServerConfig): Promise<number> {
  return await testMcpConnection(config);
}

/** Load tools for a specific server. */
export async function loadTools(serverId: string): Promise<McpToolInfo[]> {
  const tools = await getMcpTools(serverId);
  servers = servers.map((s) =>
    s.config.id === serverId ? { ...s, tools, toolCount: tools.length } : s,
  );
  return tools;
}

/** Refresh the server list from the backend. */
export async function refreshServers(): Promise<void> {
  try {
    servers = await getMcpServers();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    logFrontend("error", `Failed to refresh MCP servers: ${msg}`);
  }
}

// ── Getters ─────────────────────────────────────────────────────

export function getMcpState() {
  return {
    get servers() {
      return servers;
    },
    get registry() {
      return registry;
    },
    get registryLoading() {
      return registryLoading;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    get connectedCount() {
      return servers.filter((s) => s.status === "connected").length;
    },
    get totalToolCount() {
      return servers.reduce((sum, s) => sum + s.toolCount, 0);
    },
  };
}
