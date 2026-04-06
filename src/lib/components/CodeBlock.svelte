<script lang="ts">
  /**
   * Syntax-highlighted code block with language label and copy button.
   *
   * Receives raw code + language, highlights via Shiki, and provides
   * a one-click copy-to-clipboard action.
   */

  import { onDestroy } from "svelte";
  import { highlightCode } from "$lib/utils/syntax";

  interface Props {
    code: string;
    lang?: string;
  }

  let { code, lang }: Props = $props();

  let highlightedHtml = $state<string | null>(null);
  let copied = $state(false);
  let copyTimeout: ReturnType<typeof setTimeout> | undefined;

  // Highlight when code or lang changes (including initial mount)
  $effect(() => {
    void code;
    void lang;
    doHighlight();
  });

  onDestroy(() => {
    if (copyTimeout) clearTimeout(copyTimeout);
  });

  async function doHighlight() {
    const result = await highlightCode(code, lang);
    highlightedHtml = result;
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      // Clipboard API may not be available
    }
  }

  let displayLang = $derived(lang || "text");
</script>

<div class="code-block">
  <div class="code-header">
    <span class="code-lang">{displayLang}</span>
    <button class="code-copy" onclick={handleCopy} aria-label="Copy code to clipboard">
      {#if copied}
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg
        >
        <span>Copied</span>
      {:else}
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path
            d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
          ></path></svg
        >
        <span>Copy</span>
      {/if}
    </button>
  </div>
  <div class="code-body">
    {#if highlightedHtml}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -- Shiki output is trusted; code is user-provided content displayed as-is -->
      {@html highlightedHtml}
    {:else}
      <pre><code>{code}</code></pre>
    {/if}
  </div>
</div>

<style>
  .code-block {
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border-primary);
    overflow: hidden;
    margin: var(--spacing-md) 0;
    background: var(--color-bg-code);
  }

  .code-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-xs) var(--spacing-md);
    border-bottom: 1px solid var(--color-border-primary);
    background: var(--color-bg-secondary);
  }

  .code-lang {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    text-transform: lowercase;
    letter-spacing: 0.02em;
  }

  .code-copy {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px var(--spacing-sm);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-tertiary);
    font-family: var(--font-sans);
    font-size: var(--font-size-xs);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      background var(--transition-fast);
  }

  .code-copy:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .code-body {
    overflow-x: auto;
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    line-height: var(--line-height-relaxed);
  }

  /* Plain fallback (before shiki loads) */
  .code-body pre {
    margin: 0;
    padding: var(--spacing-md);
    background: transparent;
  }

  .code-body code {
    font-family: inherit;
    color: var(--color-text-code);
  }

  /* ── Shiki overrides ── */
  .code-body :global(.shiki) {
    margin: 0;
    padding: var(--spacing-md);
    background: transparent !important;
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
  }

  .code-body :global(.shiki code) {
    font-family: inherit;
    counter-reset: step;
    counter-increment: step 0;
  }

  /* Dual theme: light uses .light span colors, dark uses .dark span colors */
  :global([data-theme="dark"]) .code-body :global(.shiki),
  :global([data-theme="dark"]) .code-body :global(.shiki span) {
    color: var(--shiki-dark) !important;
    background-color: transparent !important;
  }

  :global(:root) .code-body :global(.shiki),
  :global(:root) .code-body :global(.shiki span),
  :global([data-theme="light"]) .code-body :global(.shiki),
  :global([data-theme="light"]) .code-body :global(.shiki span) {
    color: var(--shiki-light) !important;
    background-color: transparent !important;
  }

  @media (prefers-color-scheme: dark) {
    :global([data-theme="system"]) .code-body :global(.shiki),
    :global([data-theme="system"]) .code-body :global(.shiki span) {
      color: var(--shiki-dark) !important;
      background-color: transparent !important;
    }
  }
</style>
