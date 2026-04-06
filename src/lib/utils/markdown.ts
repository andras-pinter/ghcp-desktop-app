/**
 * Markdown rendering pipeline: marked → DOMPurify.
 *
 * Renders markdown to sanitized HTML. Code blocks are wrapped in container
 * divs with the sentinel class. Code content is stored in a JS-side array
 * (never as DOM attributes) to prevent spoofing via injected HTML.
 */

import { Marked } from "marked";
import DOMPurify, { type Config as PurifyConfig } from "dompurify";

/** Maximum source size we'll attempt to render (200 KB). */
const MAX_RENDER_SIZE = 200_000;

/** Sentinel class used to locate code block placeholders in rendered HTML. */
export const CODE_BLOCK_CLASS = "md-code-block";

/**
 * Code blocks extracted during the most recent `renderMarkdown()` call.
 * Stored in JS memory — never exposed as DOM attributes — so injected HTML
 * cannot spoof code block content.
 */
let lastCodeBlocks: Array<{ code: string; lang?: string }> = [];

/** Get code blocks from the most recent `renderMarkdown()` call. */
export function getLastCodeBlocks(): ReadonlyArray<{ code: string; lang?: string }> {
  return lastCodeBlocks;
}

const marked = new Marked({
  gfm: true,
  breaks: false,
  async: false,
  renderer: {
    code({ text, lang }) {
      lastCodeBlocks.push({ code: text, lang: lang || undefined });
      const escaped = escapeHtml(text);
      return `<div class="${CODE_BLOCK_CLASS}"><pre><code>${escaped}</code></pre></div>`;
    },
    codespan({ text }) {
      return `<code class="md-inline-code">${escapeHtml(text)}</code>`;
    },
    link({ href, title, text }) {
      const titleAttr = title ? ` title="${escapeAttr(title)}"` : "";
      return `<a href="${escapeAttr(href)}"${titleAttr} target="_blank" rel="noopener noreferrer">${text}</a>`;
    },
    image({ href, title, text }) {
      // Block images for security — render as link instead
      const titleAttr = title ? ` title="${escapeAttr(title)}"` : "";
      return `<a href="${escapeAttr(href)}"${titleAttr} target="_blank" rel="noopener noreferrer">🖼 ${escapeHtml(text || "image")}</a>`;
    },
  },
});

/**
 * DOMPurify configuration — strict allowlist.
 *
 * - `ALLOW_DATA_ATTR: false` prevents arbitrary data-* injection.
 * - `<input>` is forced to `type="checkbox" disabled` via a hook below
 *   to prevent form element spoofing.
 */
const PURIFY_CONFIG: PurifyConfig = {
  RETURN_TRUSTED_TYPE: false,
  ALLOWED_TAGS: [
    // Block
    "p",
    "br",
    "hr",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "blockquote",
    "pre",
    "code",
    "div",
    // Lists
    "ul",
    "ol",
    "li",
    // Tables
    "table",
    "thead",
    "tbody",
    "tr",
    "th",
    "td",
    // Inline
    "a",
    "strong",
    "em",
    "del",
    "s",
    "sub",
    "sup",
    "span",
    // Definition list (GFM)
    "dl",
    "dt",
    "dd",
    // Details
    "details",
    "summary",
    // Input (task list checkboxes only — enforced by hook)
    "input",
  ],
  ALLOWED_ATTR: ["href", "title", "target", "rel", "class", "type", "checked", "disabled", "align"],
  ALLOW_DATA_ATTR: false,
};

// Force all <input> elements to be disabled checkboxes (GFM task lists)
DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if (node.nodeName === "INPUT") {
    node.setAttribute("type", "checkbox");
    node.setAttribute("disabled", "");
  }
});

/**
 * Render a markdown string to sanitized HTML.
 *
 * Code fences produce `<div class="md-code-block">` containers with static
 * fallback content. Retrieve the actual code content via `getLastCodeBlocks()`
 * and match by sequential index.
 */
export function renderMarkdown(source: string): string {
  if (!source) return "";
  if (source.length > MAX_RENDER_SIZE) {
    const sizeKb = Math.round(source.length / 1024);
    return `<p>⚠️ Message too large to render (${sizeKb} KB)</p>`;
  }
  lastCodeBlocks = [];
  const raw = marked.parse(source) as string;
  return DOMPurify.sanitize(raw, PURIFY_CONFIG) as string;
}

/** HTML-escape for text content. */
function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** Escape for use inside HTML attribute values. */
function escapeAttr(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
