/**
 * Markdown rendering pipeline: marked → DOMPurify.
 *
 * Renders markdown to sanitized HTML. Code blocks are wrapped in container
 * divs with `data-code` and `data-lang` attributes so that the Svelte
 * component layer can mount interactive CodeBlock components on top.
 */

import { Marked } from "marked";
import DOMPurify, { type Config as PurifyConfig } from "dompurify";

/** Sentinel class used to locate code block placeholders in rendered HTML. */
export const CODE_BLOCK_CLASS = "md-code-block";

const marked = new Marked({
  gfm: true,
  breaks: false,
  async: false,
  renderer: {
    code({ text, lang }) {
      const escaped = escapeHtml(text);
      const langAttr = lang ? ` data-lang="${escapeAttr(lang)}"` : "";
      return `<div class="${CODE_BLOCK_CLASS}"${langAttr} data-code="${escapeAttr(text)}"><pre><code>${escaped}</code></pre></div>`;
    },
    codespan({ text }) {
      return `<code class="md-inline-code">${text}</code>`;
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
 * Configure DOMPurify to allow our code block containers and data attributes,
 * but strip anything dangerous (scripts, event handlers, etc.).
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
    // Input (task list)
    "input",
  ],
  ALLOWED_ATTR: [
    "href",
    "title",
    "target",
    "rel",
    "class",
    "data-code",
    "data-lang",
    "type",
    "checked",
    "disabled",
    "align",
  ],
  ALLOW_DATA_ATTR: true,
};

/**
 * Render a markdown string to sanitized HTML.
 *
 * Code fences produce `<div class="md-code-block" data-code="..." data-lang="...">` containers.
 * The Svelte layer can query these and mount CodeBlock components on top.
 */
export function renderMarkdown(source: string): string {
  if (!source) return "";
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
