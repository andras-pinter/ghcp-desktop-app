/**
 * Parse trigger characters (`/` and `@`) from textarea input to drive
 * the autocomplete popup.
 *
 * Runs on every `input` event — intentionally lightweight (no regex hot-paths).
 */

import {
  SLASH_COMMANDS,
  type PopupItem,
  type SlashCommand,
  type TriggerChar,
} from "$lib/types/commands";
import type { Agent } from "$lib/types/agent";
import type { Model } from "$lib/types/message";
import type { Skill } from "$lib/types/skill";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/** Result of parsing the textarea value at the current cursor position. */
export interface ParseResult {
  /** Which trigger character was detected. */
  trigger: TriggerChar;
  /** The text typed after the trigger (used to filter the popup list). */
  query: string;
  /** Start index of the trigger character in the textarea value. */
  rangeStart: number;
  /** End index (exclusive) — current cursor position. */
  rangeEnd: number;
  /** Filtered popup items matching the query. */
  items: PopupItem[];
}

// ---------------------------------------------------------------------------
// Sub-command argument matching
// ---------------------------------------------------------------------------

/**
 * When the user has typed `/model gpt` the parser is in "sub-command" mode:
 * the command is resolved and we need to filter its argument list.
 */
export interface SubCommandParseResult {
  trigger: "/";
  /** The resolved command. */
  command: SlashCommand;
  /** Argument query text (after the space). */
  argQuery: string;
  /** Start index of the full `/command arg` text. */
  rangeStart: number;
  /** End index (cursor). */
  rangeEnd: number;
  /** Filtered items (models, skills, etc.) matching argQuery. */
  items: PopupItem[];
}

// ---------------------------------------------------------------------------
// Combined result
// ---------------------------------------------------------------------------

export type CommandParseResult = ParseResult | SubCommandParseResult | null;

/** Type guard: is the result a sub-command (argument) parse? */
export function isSubCommand(r: CommandParseResult): r is SubCommandParseResult {
  return r !== null && "command" in r;
}

// ---------------------------------------------------------------------------
// Core parser
// ---------------------------------------------------------------------------

/**
 * Parse the textarea value at the given cursor position.
 *
 * @param value  Full textarea text.
 * @param cursor `selectionStart` — the caret position.
 * @param agents Available agents (for `@` mentions).
 * @param models Available models (for `/model` sub-command).
 * @param skills Available skills (for `/skill` sub-command).
 * @param hasConversation Whether an active conversation exists (hides conversation-only commands).
 * @returns A parse result, or `null` if no trigger is active.
 */
export function parseCommand(
  value: string,
  cursor: number,
  agents: Agent[],
  models: Model[],
  skills: Skill[],
  hasConversation: boolean = true,
): CommandParseResult {
  if (cursor === 0 || value.length === 0) return null;

  // ── Try slash command first ────────────────────────────────────────────
  const slashResult = parseSlash(value, cursor, models, skills, hasConversation);
  if (slashResult) return slashResult;

  // ── Try @ mention ─────────────────────────────────────────────────────
  return parseMention(value, cursor, agents);
}

// ---------------------------------------------------------------------------
// Slash parsing
// ---------------------------------------------------------------------------

/**
 * Detect `/` at the start of the current line. The slash must be the very
 * first non-whitespace character on its line.
 */
function parseSlash(
  value: string,
  cursor: number,
  models: Model[],
  skills: Skill[],
  hasConversation: boolean,
): CommandParseResult {
  // Walk backward from cursor to find the start of the current line.
  const lineStart = findLineStart(value, cursor);

  // The first non-space char on the line must be `/`.
  let i = lineStart;
  while (i < cursor && value[i] === " ") i++;
  if (i >= cursor || value[i] !== "/") return null;

  const slashPos = i;
  const textAfterSlash = value.slice(slashPos + 1, cursor);

  // Check if there's a space — indicates we're in sub-command arg mode.
  const spaceIdx = textAfterSlash.indexOf(" ");

  if (spaceIdx === -1) {
    // ── Still typing the command name ──────────────────────────────────
    const query = textAfterSlash.toLowerCase();
    const filtered = filterCommands(query, hasConversation);
    if (filtered.length === 0) return null;

    return {
      trigger: "/",
      query,
      rangeStart: slashPos,
      rangeEnd: cursor,
      items: filtered.map((c) => ({ kind: "command", command: c })),
    };
  }

  // ── Sub-command: command name is resolved, filtering args ─────────────
  const cmdName = textAfterSlash.slice(0, spaceIdx).toLowerCase();
  const command = SLASH_COMMANDS.find((c) => c.name === cmdName);
  if (!command) return null;

  const argQuery = textAfterSlash.slice(spaceIdx + 1);
  const argItems = filterArgItems(command, argQuery, models, skills);

  // For commands that take free text (web, fetch, title) or no args,
  // don't show a popup — let the user type freely.
  if (argItems === null) return null;

  return {
    trigger: "/",
    command,
    argQuery,
    rangeStart: slashPos,
    rangeEnd: cursor,
    items: argItems,
  };
}

// ---------------------------------------------------------------------------
// @ mention parsing
// ---------------------------------------------------------------------------

/**
 * Detect `@` preceded by whitespace, newline, or at position 0.
 */
function parseMention(value: string, cursor: number, agents: Agent[]): ParseResult | null {
  // Walk backward from cursor to find `@`.
  let i = cursor - 1;
  while (i >= 0) {
    const ch = value[i];
    if (ch === "@") {
      // `@` must be at start or preceded by whitespace/newline.
      if (i === 0 || /\s/.test(value[i - 1])) {
        const query = value.slice(i + 1, cursor).toLowerCase();
        const filtered = filterAgents(query, agents);
        if (filtered.length === 0 && query.length > 0) return null;

        return {
          trigger: "@",
          query,
          rangeStart: i,
          rangeEnd: cursor,
          items: filtered,
        };
      }
      return null;
    }
    // Stop at whitespace — the mention must be one contiguous word.
    if (/\s/.test(ch)) return null;
    i--;
  }
  return null;
}

// ---------------------------------------------------------------------------
// Filtering helpers
// ---------------------------------------------------------------------------

function filterCommands(query: string, hasConversation: boolean): SlashCommand[] {
  const available = hasConversation
    ? [...SLASH_COMMANDS]
    : SLASH_COMMANDS.filter((c) => !c.requiresConversation);

  if (query.length === 0) return available;

  // /? is an alias for /help
  if (query === "?") {
    const help = SLASH_COMMANDS.find((c) => c.name === "help");
    return help ? [help] : [];
  }

  return available.filter((c) => c.name.startsWith(query));
}

function filterAgents(query: string, agents: Agent[]): PopupItem[] {
  if (query.length === 0) {
    return agents.map((a) => ({ kind: "agent", agent: a }));
  }
  const q = query.toLowerCase();
  return agents
    .filter((a) => a.name.toLowerCase().includes(q))
    .map((a) => ({ kind: "agent", agent: a }));
}

/**
 * Return filtered arg items for commands that have enumerable arguments
 * (model, skill, format). Returns `null` for free-text commands.
 */
function filterArgItems(
  command: SlashCommand,
  query: string,
  models: Model[],
  skills: Skill[],
): PopupItem[] | null {
  const q = query.toLowerCase();

  switch (command.argType) {
    case "model": {
      const list =
        q.length === 0
          ? models
          : models.filter((m) => {
              const label = (m.name ?? m.id).toLowerCase();
              return label.includes(q) || m.id.toLowerCase().includes(q);
            });
      return list.map((m) => ({ kind: "model", model: m }));
    }
    case "skill": {
      const enabled = skills.filter((s) => s.enabled);
      const list =
        q.length === 0 ? enabled : enabled.filter((s) => s.name.toLowerCase().includes(q));
      return list.map((s) => ({ kind: "skill", skill: s }));
    }
    case "format": {
      // Static list — no dynamic data needed.
      return [];
    }
    // Free-text args: don't show a popup.
    case "text":
    case "url":
    case "none":
    default:
      return null;
  }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/** Find the index of the start of the line containing `pos`. */
function findLineStart(value: string, pos: number): number {
  let i = pos - 1;
  while (i >= 0 && value[i] !== "\n") i--;
  return i + 1;
}
