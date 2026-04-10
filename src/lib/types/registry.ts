/** Registry search result types for aitmpl.com + git import. */

export type RegistrySource = "aitmpl" | "custom" | "git";

export type RegistryItemKind = "skill" | "agent";

export interface RegistryItem {
  id: string;
  name: string;
  description: string | null;
  source: RegistrySource;
  /** Display name of the source (e.g. "aitmpl.com" or git source name). Absent when not set. */
  sourceName?: string | null;
  url: string | null;
  installs: number | null;
  kind: RegistryItemKind;
  sourceRepo: string | null;
  /** Full SKILL.md content (available for aitmpl items and git catalog items). */
  content?: string | null;
}

export interface RegistrySearchResult {
  items: RegistryItem[];
  total: number | null;
}

/** A discovered definition file (SKILL.md or *.agent.md) from a git repository. */
export interface GitSkillFile {
  path: string;
  content: string;
  repoUrl: string;
  /** "skill" or "agent" */
  kind: string;
}
