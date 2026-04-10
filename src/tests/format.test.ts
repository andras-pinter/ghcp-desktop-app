import { describe, it, expect } from "vitest";
import {
  formatDateGroup,
  truncate,
  formatBytes,
  stripMarkdown,
  truncateMarkdown,
} from "$lib/utils/format";

describe("formatDateGroup", () => {
  it("returns 'Today' for today's date", () => {
    expect(formatDateGroup(new Date().toISOString())).toBe("Today");
  });

  it("returns 'Yesterday' for yesterday's date", () => {
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    expect(formatDateGroup(yesterday.toISOString())).toBe("Yesterday");
  });

  it("returns 'Last 7 Days' for dates within the past week", () => {
    const threeDaysAgo = new Date();
    threeDaysAgo.setDate(threeDaysAgo.getDate() - 3);
    expect(formatDateGroup(threeDaysAgo.toISOString())).toBe("Last 7 Days");
  });
});

describe("truncate", () => {
  it("returns the original text if within limit", () => {
    expect(truncate("hello", 10)).toBe("hello");
  });

  it("truncates and adds ellipsis", () => {
    expect(truncate("hello world", 6)).toBe("hello…");
  });
});

describe("formatBytes", () => {
  it("formats bytes correctly", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(1024 * 1024 * 12.3)).toBe("12.3 MB");
  });
});

describe("stripMarkdown", () => {
  it("strips bold formatting", () => {
    expect(stripMarkdown("**Role**: Architect")).toBe("Role: Architect");
  });

  it("strips italic formatting", () => {
    expect(stripMarkdown("*emphasis* here")).toBe("emphasis here");
  });

  it("strips headings", () => {
    expect(stripMarkdown("# Title\nBody text")).toBe("Title Body text");
  });

  it("strips inline code", () => {
    expect(stripMarkdown("Use `serde` for JSON")).toBe("Use serde for JSON");
  });

  it("strips links", () => {
    expect(stripMarkdown("See [docs](https://example.com)")).toBe("See docs");
  });

  it("converts list items", () => {
    expect(stripMarkdown("- item one\n- item two")).toBe("• item one • item two");
  });

  it("collapses whitespace and paragraph breaks", () => {
    expect(stripMarkdown("First paragraph\n\nSecond paragraph")).toBe(
      "First paragraph · Second paragraph",
    );
  });

  it("handles combined markdown", () => {
    const input = "# **Role**: 3D Web Experience Architect\n\nDesigns *immersive* web apps.";
    expect(stripMarkdown(input)).toBe(
      "Role: 3D Web Experience Architect · Designs immersive web apps.",
    );
  });
});

describe("truncateMarkdown", () => {
  it("strips markdown and truncates", () => {
    const input = "**Bold text** that goes on and on and on";
    expect(truncateMarkdown(input, 20)).toBe("Bold text that goes…");
  });

  it("returns full text if short enough", () => {
    expect(truncateMarkdown("**Hello**", 20)).toBe("Hello");
  });
});
