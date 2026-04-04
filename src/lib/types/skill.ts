export type SkillSource = "extension" | "mcp";

export interface Skill {
  id: string;
  name: string;
  description: string | null;
  source: SkillSource;
  mcpServerId: string | null;
  config: string | null;
  enabled: boolean;
  createdAt: string;
}
