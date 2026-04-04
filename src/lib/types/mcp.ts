export type McpTransport = "http" | "stdio";

export interface McpServer {
  id: string;
  name: string;
  transport: McpTransport;
  url: string | null;
  binaryPath: string | null;
  args: string[] | null;
  authHeader: string | null;
  fromCatalog: boolean;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}
