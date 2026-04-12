import { describe, it, expect } from "vitest";
import { parseCommand, isSubCommand, type CommandParseResult } from "$lib/utils/command-parser";
import type { Agent } from "$lib/types/agent";
import type { Model } from "$lib/types/message";
import type { Skill } from "$lib/types/skill";

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

const agents: Agent[] = [
  {
    id: "a1",
    name: "Research",
    avatar: "🔬",
    systemPrompt: "You are a researcher.",
    isDefault: false,
    sourceUrl: null,
    sourceType: "local",
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: "2025-01-01T00:00:00Z",
    gitSourceId: null,
  },
  {
    id: "a2",
    name: "Coder",
    avatar: "💻",
    systemPrompt: "You are a coder.",
    isDefault: true,
    sourceUrl: null,
    sourceType: "local",
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: "2025-01-01T00:00:00Z",
    gitSourceId: null,
  },
];

const models: Model[] = [
  { id: "gpt-4o", name: "GPT-4o" },
  { id: "gpt-4o-mini", name: "GPT-4o mini" },
  { id: "claude-3.5-sonnet", name: "Claude 3.5 Sonnet" },
];

const skills: Skill[] = [
  {
    id: "s1",
    name: "code-review",
    description: "Review code",
    source: "builtin",
    mcpServerId: null,
    config: null,
    instructions: null,
    sourceUrl: null,
    sourceType: "builtin",
    enabled: true,
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: null,
    gitSourceId: null,
  },
  {
    id: "s2",
    name: "summarize",
    description: "Summarize text",
    source: "builtin",
    mcpServerId: null,
    config: null,
    instructions: null,
    sourceUrl: null,
    sourceType: "builtin",
    enabled: true,
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: null,
    gitSourceId: null,
  },
  {
    id: "s3",
    name: "disabled-skill",
    description: "Disabled",
    source: "builtin",
    mcpServerId: null,
    config: null,
    instructions: null,
    sourceUrl: null,
    sourceType: "builtin",
    enabled: false,
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: null,
    gitSourceId: null,
  },
];

function parse(
  value: string,
  cursor?: number,
  hasConversation: boolean = true,
): CommandParseResult {
  return parseCommand(value, cursor ?? value.length, agents, models, skills, hasConversation);
}

// ---------------------------------------------------------------------------
// Slash command detection
// ---------------------------------------------------------------------------

describe("slash command detection", () => {
  it("returns null for empty input", () => {
    expect(parse("", 0)).toBeNull();
  });

  it("returns null for plain text", () => {
    expect(parse("hello world")).toBeNull();
  });

  it("detects / at position 0", () => {
    const r = parse("/");
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("/");
    expect(r!.items.length).toBeGreaterThan(0);
  });

  it("filters commands by prefix", () => {
    const r = parse("/fe");
    expect(r).not.toBeNull();
    expect(r!.items).toHaveLength(1);
    expect(r!.items[0].kind).toBe("command");
    if (r!.items[0].kind === "command") {
      expect(r!.items[0].command.name).toBe("fetch");
    }
  });

  it("matches multiple commands with shared prefix", () => {
    const r = parse("/f");
    expect(r).not.toBeNull();
    const names = r!.items.map((i) => (i.kind === "command" ? i.command.name : ""));
    expect(names).toContain("file");
    expect(names).toContain("fetch");
    expect(names).toContain("favorite");
  });

  it("returns null for non-matching prefix", () => {
    expect(parse("/zzz")).toBeNull();
  });

  it("provides correct range", () => {
    const r = parse("/fetch");
    expect(r).not.toBeNull();
    expect(r!.rangeStart).toBe(0);
    expect(r!.rangeEnd).toBe(6);
  });

  it("detects / after a newline", () => {
    const text = "some text\n/he";
    const r = parse(text);
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("/");
    if (!isSubCommand(r)) {
      expect(r!.query).toBe("he");
    }
  });

  it("ignores / in the middle of a line", () => {
    expect(parse("hello /fetch", 12)).toBeNull();
  });

  it("shows all commands for bare /", () => {
    const r = parse("/");
    expect(r).not.toBeNull();
    expect(r!.items.length).toBe(9);
  });
});

// ---------------------------------------------------------------------------
// Sub-command argument parsing
// ---------------------------------------------------------------------------

describe("sub-command argument parsing", () => {
  it("enters sub-command mode after space", () => {
    const r = parse("/model ");
    expect(r).not.toBeNull();
    expect(isSubCommand(r)).toBe(true);
    if (isSubCommand(r)) {
      expect(r.command.name).toBe("model");
      expect(r.argQuery).toBe("");
      expect(r.items).toHaveLength(3); // all models
    }
  });

  it("filters models by query", () => {
    const r = parse("/model gpt");
    expect(isSubCommand(r)).toBe(true);
    if (isSubCommand(r)) {
      expect(r.items).toHaveLength(2); // gpt-4o and gpt-4o-mini
    }
  });

  it("filters skills by query", () => {
    const r = parse("/skill code");
    expect(isSubCommand(r)).toBe(true);
    if (isSubCommand(r)) {
      expect(r.items).toHaveLength(1);
      expect(r.items[0].kind).toBe("skill");
    }
  });

  it("excludes disabled skills", () => {
    const r = parse("/skill disabled");
    expect(isSubCommand(r)).toBe(true);
    if (isSubCommand(r)) {
      expect(r.items).toHaveLength(0);
    }
  });

  it("returns null for free-text commands (no popup needed)", () => {
    expect(parse("/fetch https://example.com")).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// @ mention detection
// ---------------------------------------------------------------------------

describe("@ mention detection", () => {
  it("detects @ at position 0", () => {
    const r = parse("@");
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("@");
    expect(r!.items).toHaveLength(2); // all agents
  });

  it("filters agents by query", () => {
    const r = parse("@res");
    expect(r).not.toBeNull();
    expect(r!.items).toHaveLength(1);
    if (r!.items[0].kind === "agent") {
      expect(r!.items[0].agent.name).toBe("Research");
    }
  });

  it("detects @ after whitespace", () => {
    const r = parse("hello @co");
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("@");
    if (r!.items[0].kind === "agent") {
      expect(r!.items[0].agent.name).toBe("Coder");
    }
  });

  it("detects @ after newline", () => {
    const r = parse("line one\n@res");
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("@");
  });

  it("ignores @ in the middle of a word", () => {
    expect(parse("email@test")).toBeNull();
  });

  it("provides correct range for inline @", () => {
    const text = "ask @Research about this";
    // cursor right after "h" in "Research" (index 13 — before the space)
    const r = parse(text, 13);
    expect(r).not.toBeNull();
    expect(r!.rangeStart).toBe(4); // position of @
    expect(r!.rangeEnd).toBe(13);
  });

  it("returns null for no matching agents", () => {
    expect(parse("@zzz")).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

describe("edge cases", () => {
  it("handles cursor at 0", () => {
    expect(parse("hello", 0)).toBeNull();
  });

  it("handles cursor in the middle of slash text", () => {
    // Cursor is at position 2, so query is just "w" from "/w"
    const r = parse("/fetch url", 2);
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("/");
    if (!isSubCommand(r)) {
      expect(r!.query).toBe("f");
    }
  });

  it("prefers slash over @ when / is at line start", () => {
    // If text starts with /, it's a slash command even if @ is later.
    const r = parse("/");
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("/");
  });

  it("handles multiline with @ on second line", () => {
    const text = "first line\n@Cod";
    const r = parse(text);
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("@");
  });

  it("handles / on second line of multiline input", () => {
    const text = "first line\n/mod";
    const r = parse(text);
    expect(r).not.toBeNull();
    expect(r!.trigger).toBe("/");
    if (!isSubCommand(r)) {
      expect(r!.query).toBe("mod");
    }
  });
});

// ---------------------------------------------------------------------------
// hasConversation filtering
// ---------------------------------------------------------------------------

describe("hasConversation filtering", () => {
  it("hides conversation-only commands when hasConversation is false", () => {
    const r = parse("/", undefined, false);
    expect(r).not.toBeNull();
    const names = r!.items.map((i) => (i.kind === "command" ? i.command.name : ""));
    expect(names).not.toContain("title");
    expect(names).not.toContain("favorite");
    expect(names).not.toContain("delete");
    expect(names).not.toContain("export");
    // These should still be present
    expect(names).toContain("model");
    expect(names).toContain("fetch");
    expect(names).toContain("help");
  });

  it("shows all commands when hasConversation is true", () => {
    const r = parse("/", undefined, true);
    expect(r).not.toBeNull();
    expect(r!.items.length).toBe(9);
  });

  it("hides /favorite from /f results when no conversation", () => {
    const r = parse("/f", undefined, false);
    expect(r).not.toBeNull();
    const names = r!.items.map((i) => (i.kind === "command" ? i.command.name : ""));
    expect(names).toContain("file");
    expect(names).toContain("fetch");
    expect(names).not.toContain("favorite");
  });
});

// ---------------------------------------------------------------------------
// /? alias for /help
// ---------------------------------------------------------------------------

describe("/? alias", () => {
  it("returns /help for /?", () => {
    const r = parse("/?");
    expect(r).not.toBeNull();
    expect(r!.items).toHaveLength(1);
    if (r!.items[0].kind === "command") {
      expect(r!.items[0].command.name).toBe("help");
    }
  });
});

// ---------------------------------------------------------------------------
// /web alias for /fetch
// ---------------------------------------------------------------------------

describe("/web alias", () => {
  it("returns /fetch for /web", () => {
    const r = parse("/web");
    expect(r).not.toBeNull();
    expect(r!.items).toHaveLength(1);
    if (r!.items[0].kind === "command") {
      expect(r!.items[0].command.name).toBe("fetch");
    }
  });
});
