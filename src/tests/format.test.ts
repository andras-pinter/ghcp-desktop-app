import { describe, it, expect } from "vitest";
import { formatDateGroup, truncate, formatBytes } from "$lib/utils/format";

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
