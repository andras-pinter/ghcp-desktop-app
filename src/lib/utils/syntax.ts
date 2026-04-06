/**
 * Syntax highlighting via Shiki.
 *
 * The highlighter is loaded lazily (WASM-based, ~1MB first load) and
 * cached as a singleton. Falls back to plain `<code>` if loading fails
 * or the language isn't supported.
 */

import { createHighlighter, type Highlighter } from "shiki";

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
 * Highlight a code string and return HTML.
 *
 * Returns `null` if the highlighter hasn't loaded yet or the language
 * isn't supported — caller should fall back to plain rendering.
 */
export async function highlightCode(code: string, lang?: string): Promise<string | null> {
  try {
    const highlighter = await getHighlighter();
    const loadedLangs = highlighter.getLoadedLanguages();
    const resolvedLang = lang && loadedLangs.includes(lang as never) ? lang : "text";

    // If language isn't loaded and isn't text, try loading it dynamically
    if (lang && resolvedLang === "text" && lang !== "text") {
      try {
        await highlighter.loadLanguage(lang as never);
        return highlighter.codeToHtml(code, {
          lang,
          themes: { light: "github-light", dark: "github-dark" },
          defaultColor: false,
        });
      } catch {
        // Language not available — fall through to text
      }
    }

    return highlighter.codeToHtml(code, {
      lang: resolvedLang,
      themes: { light: "github-light", dark: "github-dark" },
      defaultColor: false,
    });
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
