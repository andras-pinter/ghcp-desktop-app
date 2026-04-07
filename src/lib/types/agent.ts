export type AgentSourceType = "local" | "registry_aitmpl" | "git";

export interface Agent {
  id: string;
  name: string;
  avatar: string | null;
  systemPrompt: string;
  isDefault: boolean;
  sourceUrl: string | null;
  sourceType: AgentSourceType;
  createdAt: string;
  updatedAt: string;
}
