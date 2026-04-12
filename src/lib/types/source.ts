/** A persistent git repository source for skills and agents. */
export interface GitSource {
  /** Unique identifier (UUID). */
  id: string;
  /** Display name (usually the repo name). */
  name: string;
  /** Git repository URL. */
  url: string;
  /** Whether syncing is enabled. */
  enabled: boolean;
  /** ISO 8601 timestamp of the last successful sync, or null if never synced. */
  lastSyncedAt: string | null;
  /** Number of items (skills + agents) imported from this source. */
  itemCount: number;
  /** ISO 8601 creation timestamp. */
  createdAt: string;
  /** ISO 8601 last-update timestamp. */
  updatedAt: string;
}

/** A skill or agent linked to a git source. */
export interface SourceItem {
  /** Item ID (skill or agent ID). */
  id: string;
  /** Display name. */
  name: string;
  /** "skill" or "agent". */
  kind: "skill" | "agent";
  /** URL where the item was imported from, if any. */
  sourceUrl: string | null;
}

/** An item the user wants to import from a scanned source. */
export interface ImportItem {
  /** File path within the repository. */
  path: string;
  /** Raw file content (SKILL.md or *.agent.md). */
  content: string;
  /** Item type: "skill" or "agent". */
  kind: "skill" | "agent";
}

/** Result returned when creating or syncing a source. */
export interface SourceScanResult {
  /** The git source metadata. */
  source: GitSource;
  /** Files discovered in the repository. */
  files: import("$lib/types/registry").GitSkillFile[];
}
