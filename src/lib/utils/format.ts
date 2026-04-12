/** Date formatting, text truncation, and general utility functions. */

/**
 * Format a date string into a human-friendly relative label.
 * Returns "Today", "Yesterday", "Last 7 Days", or the formatted date.
 */
export function formatDateGroup(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return "Last 7 Days";
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
}

/** Truncate text to a maximum length, adding ellipsis if needed. */
export function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - 1) + "…";
}

/** Strip markdown formatting to produce plain-text suitable for card previews. */
export function stripMarkdown(text: string): string {
  return text
    .replace(/^#{1,6}\s+/gm, "") // headings
    .replace(/\*\*(.+?)\*\*/g, "$1") // bold
    .replace(/\*(.+?)\*/g, "$1") // italic
    .replace(/`(.+?)`/g, "$1") // inline code
    .replace(/\[(.+?)\]\(.+?\)/g, "$1") // links
    .replace(/^[-*]\s+/gm, "• ") // list items
    .replace(/\n{2,}/g, " · ") // paragraph breaks → separator
    .replace(/\n/g, " ") // remaining newlines
    .replace(/\s{2,}/g, " ") // collapse whitespace
    .trim();
}

/** Strip markdown and truncate for card preview text. */
export function truncateMarkdown(text: string, maxLength: number = 100): string {
  const plain = stripMarkdown(text);
  if (plain.length <= maxLength) return plain;
  return plain.slice(0, maxLength).trimEnd() + "…";
}

/** Format bytes into a human-readable string (e.g., "12.3 MB"). */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

/** Format a large count into a compact string (e.g., 1500 → "1.5k", 23400 → "23k"). */
export function formatCount(n: number): string {
  if (n >= 1_000_000) {
    const m = n / 1_000_000;
    return m >= 10 ? `${Math.round(m)}M` : `${m.toFixed(1).replace(/\.0$/, "")}M`;
  }
  if (n >= 1_000) {
    const k = n / 1_000;
    return k >= 10 ? `${Math.round(k)}k` : `${k.toFixed(1).replace(/\.0$/, "")}k`;
  }
  return String(n);
}
