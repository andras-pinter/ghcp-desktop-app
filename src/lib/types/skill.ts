export type SkillSource = "extension" | "mcp" | "builtin" | "registry_aitmpl" | "git";

export type SkillSourceType = SkillSource;

export interface Skill {
  id: string;
  name: string;
  description: string | null;
  source: SkillSource;
  mcpServerId: string | null;
  config: string | null;
  instructions: string | null;
  sourceUrl: string | null;
  sourceType: SkillSourceType;
  enabled: boolean;
  createdAt: string;
  updatedAt: string | null;
}
