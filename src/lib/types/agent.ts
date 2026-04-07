/** Where an agent was installed from. */
export type AgentSourceType = "local" | "registry_aitmpl" | "git";

/** A custom agent persona with a system prompt and optional tool assignments. */
export interface Agent {
  /** Unique identifier (UUID). */
  id: string;
  /** Display name. */
  name: string;
  /** Emoji avatar, or null for default. */
  avatar: string | null;
  /** System prompt prepended to conversations using this agent. */
  systemPrompt: string;
  /** True for the built-in default agent (cannot be deleted). */
  isDefault: boolean;
  /** URL where the agent definition was imported from, if any. */
  sourceUrl: string | null;
  /** Installation source type. */
  sourceType: AgentSourceType;
  /** ISO 8601 creation timestamp. */
  createdAt: string;
  /** ISO 8601 last-update timestamp. */
  updatedAt: string;
}
