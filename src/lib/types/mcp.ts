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

/** A catalog entry for a well-known MCP server. */
export interface CatalogEntry {
  id: string;
  name: string;
  description: string;
  transport: McpTransport;
  defaultUrl?: string | null;
  defaultBinary?: string | null;
  requiredFields: CatalogField[];
}

/** A required configuration field for a catalog entry. */
export interface CatalogField {
  key: string;
  label: string;
  placeholder: string;
  secret: boolean;
}
