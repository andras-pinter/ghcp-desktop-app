/** Where a skill was installed from. */
export type SkillSource = "extension" | "mcp" | "builtin" | "registry_aitmpl" | "git";

/** @deprecated Use {@link SkillSource} directly. */
export type SkillSourceType = SkillSource;

/** A skill (tool/capability) that can be assigned to agents. */
export interface Skill {
  /** Unique identifier. */
  id: string;
  /** Display name. */
  name: string;
  /** Short description of the skill. */
  description: string | null;
  /** Installation source category. */
  source: SkillSource;
  /** Associated MCP server ID, if this is an MCP-sourced skill. */
  mcpServerId: string | null;
  /** JSON configuration blob for the skill. */
  config: string | null;
  /** Markdown instructions / system prompt fragment. */
  instructions: string | null;
  /** URL where the skill was imported from, if any. */
  sourceUrl: string | null;
  /** Installation source type. */
  sourceType: SkillSourceType;
  /** Whether the skill is currently active. */
  enabled: boolean;
  /** ISO 8601 creation timestamp. */
  createdAt: string;
  /** ISO 8601 last-update timestamp. */
  updatedAt: string | null;
  /** ID of the git source this skill was imported from, or null. */
  gitSourceId: string | null;
}
