/** Registry search result types for skills.sh + aitmpl.com + git import. */

export type RegistrySource = "skills_sh" | "aitmpl";

export type RegistryItemKind = "skill" | "agent";

export interface RegistryItem {
  id: string;
  name: string;
  description: string | null;
  source: RegistrySource;
  url: string | null;
  installs: number | null;
  kind: RegistryItemKind;
  sourceRepo: string | null;
}

export interface RegistrySearchResult {
  items: RegistryItem[];
  total: number | null;
}

/** A discovered SKILL.md file from a git repository. */
export interface GitSkillFile {
  path: string;
  content: string;
  repoUrl: string;
}
