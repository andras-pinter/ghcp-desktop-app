/**
 * Syntax highlighting via Shiki.
 *
 * The highlighter is loaded lazily (WASM-based, ~1MB first load) and
 * cached as a singleton. Falls back to plain `<code>` if loading fails
 * or the language isn't supported.
 *
 * All Shiki HTML output is sanitized through DOMPurify as defense-in-depth
 * before being rendered via `{@html}`.
 */

import { createHighlighter, type Highlighter } from "shiki";
import DOMPurify, { type Config as PurifyConfig } from "dompurify";

/** Strict allowlist for Shiki output — only structural + styling elements. */
const SHIKI_PURIFY_CONFIG: PurifyConfig = {
  RETURN_TRUSTED_TYPE: false,
  ALLOWED_TAGS: ["pre", "code", "span"],
  ALLOWED_ATTR: ["class", "style"],
  ALLOW_DATA_ATTR: false,
};

let highlighterPromise: Promise<Highlighter> | null = null;

/** Lazily initialise the Shiki highlighter singleton. */
function getHighlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: ["github-light", "github-dark"],
      langs: [
        "javascript",
        "typescript",
        "rust",
        "python",
        "json",
        "html",
        "css",
        "bash",
        "shell",
        "sql",
        "yaml",
        "toml",
        "markdown",
        "go",
        "java",
        "c",
        "cpp",
        "csharp",
        "ruby",
        "php",
        "swift",
        "kotlin",
        "dockerfile",
        "xml",
        "graphql",
        "svelte",
        "tsx",
        "jsx",
      ],
    });
  }
  return highlighterPromise;
}

/**
 * Highlight a code string and return sanitized HTML.
 *
 * Returns `null` if the highlighter hasn't loaded yet or the language
 * isn't supported — caller should fall back to plain rendering.
 */
export async function highlightCode(code: string, lang?: string): Promise<string | null> {
  try {
    const highlighter = await getHighlighter();
    const loadedLangs = highlighter.getLoadedLanguages() as string[];
    const resolvedLang = lang && loadedLangs.includes(lang) ? lang : "text";

    // If language isn't loaded and isn't text, try loading it dynamically
    if (lang && resolvedLang === "text" && lang !== "text") {
      try {
        await highlighter.loadLanguage(lang as Parameters<typeof highlighter.loadLanguage>[0]);
        const html = highlighter.codeToHtml(code, {
          lang,
          themes: { light: "github-light", dark: "github-dark" },
          defaultColor: false,
        });
        return DOMPurify.sanitize(html, SHIKI_PURIFY_CONFIG) as string;
      } catch {
        // Language not available — fall through to text
      }
    }

    const html = highlighter.codeToHtml(code, {
      lang: resolvedLang,
      themes: { light: "github-light", dark: "github-dark" },
      defaultColor: false,
    });
    return DOMPurify.sanitize(html, SHIKI_PURIFY_CONFIG) as string;
  } catch {
    return null;
  }
}

/**
 * Pre-warm the highlighter (call on app init or first message).
 * Non-blocking — silently ignores errors.
 */
export function preloadHighlighter(): void {
  getHighlighter().catch(() => {});
}
