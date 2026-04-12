/**
 * Slash-command and @-mention types for the input area.
 *
 * `/` commands execute actions or set per-message context.
 * `@` mentions set a per-message agent override.
 */

import type { Agent } from "./agent";
import type { Skill } from "./skill";
import type { Model } from "./message";

// ---------------------------------------------------------------------------
// Command definitions
// ---------------------------------------------------------------------------

/** Trigger character that activates the autocomplete popup. */
export type TriggerChar = "/" | "@";

/** How a command behaves when selected. */
export type CommandBehavior =
  /** Executes immediately, clears input (e.g. /clear, /help). */
  | "action"
  /** Sets context for the message being composed (e.g. /model, /web). */
  | "context";

/** Argument type expected after the command keyword. */
export type ArgType = "none" | "text" | "url" | "agent" | "model" | "skill" | "format";

/** A single slash-command definition. */
export interface SlashCommand {
  /** The command keyword without the leading `/` (e.g. "web"). */
  readonly name: string;
  /** Short user-facing description. */
  readonly description: string;
  /** Icon/emoji shown in the autocomplete list. */
  readonly icon: string;
  /** Whether this is an immediate action or a context modifier. */
  readonly behavior: CommandBehavior;
  /** What kind of argument the command expects. */
  readonly argType: ArgType;
  /** Placeholder text shown when the command is selected but needs args. */
  readonly argPlaceholder?: string;
  /** If true, this command is only shown when a conversation is active. */
  readonly requiresConversation?: boolean;
  /** Alternative names that also resolve to this command (e.g. ["web"] for fetch). */
  readonly aliases?: readonly string[];
}

// ---------------------------------------------------------------------------
// Static command registry
// ---------------------------------------------------------------------------

/** All available slash commands, in display order. */
export const SLASH_COMMANDS: readonly SlashCommand[] = [
  // ── Context commands (modify the message) ──────────────────────────────
  {
    name: "model",
    description: "Override model for this message",
    icon: "📊",
    behavior: "context",
    argType: "model",
    argPlaceholder: "model name",
  },
  {
    name: "skill",
    description: "Enable a skill for this message",
    icon: "⚡",
    behavior: "context",
    argType: "skill",
    argPlaceholder: "skill name",
  },
  {
    name: "fetch",
    description: "Fetch URL content",
    icon: "🔗",
    behavior: "context",
    argType: "url",
    argPlaceholder: "https://…",
    aliases: ["web"],
  },

  // ── Action commands (execute immediately) ──────────────────────────────
  {
    name: "file",
    description: "Attach a file",
    icon: "📎",
    behavior: "action",
    argType: "none",
  },
  {
    name: "title",
    description: "Rename conversation",
    icon: "✏️",
    behavior: "action",
    argType: "none",
    requiresConversation: true,
  },
  {
    name: "favorite",
    description: "Toggle favourite",
    icon: "⭐",
    behavior: "action",
    argType: "none",
    requiresConversation: true,
  },
  {
    name: "delete",
    description: "Delete conversation",
    icon: "🗑️",
    behavior: "action",
    argType: "none",
    requiresConversation: true,
  },
  {
    name: "export",
    description: "Export conversation",
    icon: "📤",
    behavior: "action",
    argType: "none",
    requiresConversation: true,
  },
  {
    name: "help",
    description: "Show available commands",
    icon: "❓",
    behavior: "action",
    argType: "none",
    aliases: ["?"],
  },
] as const;

// ---------------------------------------------------------------------------
// Popup item types (unified for / and @)
// ---------------------------------------------------------------------------

/** A single item displayed in the autocomplete popup. */
export type PopupItem =
  | { readonly kind: "command"; readonly command: SlashCommand }
  | { readonly kind: "agent"; readonly agent: Agent }
  | { readonly kind: "model"; readonly model: Model }
  | { readonly kind: "skill"; readonly skill: Skill };

// ---------------------------------------------------------------------------
// Per-message overrides (ephemeral — cleared after send)
// ---------------------------------------------------------------------------

/** Overrides applied to a single message, set via commands or @-mentions. */
export interface MessageOverrides {
  /** Agent to use instead of the conversation's selected agent. */
  agentId: string | null;
  /** Model to use instead of the conversation's selected model. */
  modelId: string | null;
  /** Additional skill IDs to enable for this message. */
  skillIds: string[];
}

/** Create a fresh (empty) overrides object. */
export function emptyOverrides(): MessageOverrides {
  return { agentId: null, modelId: null, skillIds: [] };
}
