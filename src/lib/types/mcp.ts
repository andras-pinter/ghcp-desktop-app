export type McpTransport = "http" | "stdio";

export type McpServerStatus = "disconnected" | "connecting" | "connected" | "error";

/** MCP server configuration (matches Rust McpServerConfig). */
export interface McpServerConfig {
  id: string;
  name: string;
  transport: McpTransport;
  url?: string | null;
  binaryPath?: string | null;
  args?: string | null;
  authHeader?: string | null;
  fromCatalog: boolean;
  enabled: boolean;
}

/** Full connection info for an MCP server (config + live status). */
export interface McpConnectionInfo {
  config: McpServerConfig;
  status: McpServerStatus;
  error?: string | null;
  toolCount: number;
  tools?: McpToolInfo[] | null;
}

/** An MCP tool discovered from a connected server. */
export interface McpToolInfo {
  name: string;
  description?: string | null;
  inputSchema: Record<string, unknown>;
}

/** Result of an MCP tool invocation. */
export interface McpToolResult {
  content: McpToolContent[];
  isError: boolean;
}

export type McpToolContent =
  | { type: "text"; text: string }
  | { type: "image"; data: string; mimeType: string };

/** A server from the official MCP Registry. */
export interface RegistryServer {
  name: string;
  displayName: string;
  description: string;
  version?: string | null;
  websiteUrl?: string | null;
  repoUrl?: string | null;
  isStdioOnly: boolean;
  remotes: RegistryRemote[];
  packages: RegistryPackage[];
}

/** A remote connection option for a registry server. */
export interface RegistryRemote {
  transportType: string;
  url?: string | null;
  requiresAuth: boolean;
  authDescription?: string | null;
}

/** A package/installation option for a registry server. */
export interface RegistryPackage {
  registryType: string;
  identifier: string;
  version?: string | null;
}
