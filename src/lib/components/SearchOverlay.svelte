<script lang="ts">
  /**
   * In-conversation search overlay (Cmd+F / Ctrl+F).
   *
   * Floating search bar at top of chat area with match count,
   * up/down navigation arrows, and Escape to dismiss.
   */

  import { onDestroy } from "svelte";

  interface Props {
    /** Plain text content of all messages, one per entry, with their element IDs. */
    onClose: () => void;
    chatContainer?: HTMLElement;
  }

  let { onClose, chatContainer }: Props = $props();

  let query = $state("");
  let matchCount = $state(0);
  let currentMatch = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  /** CSS class applied to highlighted matches. */
  const HIGHLIGHT_CLASS = "search-highlight";
  const HIGHLIGHT_ACTIVE_CLASS = "search-highlight-active";

  let previousMarks: HTMLElement[] = [];

  onDestroy(() => {
    clearHighlights();
  });

  function clearHighlights() {
    for (const mark of previousMarks) {
      const parent = mark.parentNode;
      if (parent) {
        parent.replaceChild(document.createTextNode(mark.textContent || ""), mark);
        parent.normalize();
      }
    }
    previousMarks = [];
    matchCount = 0;
    currentMatch = 0;
  }

  function highlightMatches(searchText: string) {
    clearHighlights();
    if (!searchText.trim() || !chatContainer) return;

    const walker = document.createTreeWalker(chatContainer, NodeFilter.SHOW_TEXT, {
      acceptNode(node) {
        const parent = node.parentElement;
        if (!parent) return NodeFilter.FILTER_REJECT;
        // Skip search overlay UI, scripts, styles, and form elements
        if (parent.closest(".search-overlay")) return NodeFilter.FILTER_REJECT;
        const tag = parent.tagName;
        if (tag === "SCRIPT" || tag === "STYLE" || tag === "TEXTAREA" || tag === "INPUT") {
          return NodeFilter.FILTER_REJECT;
        }
        return NodeFilter.FILTER_ACCEPT;
      },
    });

    const lowerSearch = searchText.toLowerCase();
    const textNodes: Text[] = [];
    let node: Node | null;
    while ((node = walker.nextNode())) {
      textNodes.push(node as Text);
    }

    const marks: HTMLElement[] = [];

    for (const textNode of textNodes) {
      const text = textNode.textContent || "";
      const lowerText = text.toLowerCase();
      const indices: number[] = [];
      let startIdx = 0;

      while (startIdx < lowerText.length) {
        const idx = lowerText.indexOf(lowerSearch, startIdx);
        if (idx === -1) break;
        indices.push(idx);
        startIdx = idx + 1;
      }

      if (indices.length === 0) continue;

      // Split the text node and wrap matches in <mark> elements
      const frag = document.createDocumentFragment();
      let lastEnd = 0;

      for (const idx of indices) {
        if (idx > lastEnd) {
          frag.appendChild(document.createTextNode(text.slice(lastEnd, idx)));
        }
        const mark = document.createElement("mark");
        mark.className = HIGHLIGHT_CLASS;
        mark.textContent = text.slice(idx, idx + searchText.length);
        frag.appendChild(mark);
        marks.push(mark);
        lastEnd = idx + searchText.length;
      }

      if (lastEnd < text.length) {
        frag.appendChild(document.createTextNode(text.slice(lastEnd)));
      }

      textNode.parentNode?.replaceChild(frag, textNode);
    }

    previousMarks = marks;
    matchCount = marks.length;
    currentMatch = marks.length > 0 ? 1 : 0;

    if (marks.length > 0) {
      scrollToMatch(0);
    }
  }

  function scrollToMatch(index: number) {
    // Remove active class from all
    for (const m of previousMarks) {
      m.classList.remove(HIGHLIGHT_ACTIVE_CLASS);
    }
    if (index >= 0 && index < previousMarks.length) {
      previousMarks[index].classList.add(HIGHLIGHT_ACTIVE_CLASS);
      previousMarks[index].scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }

  function goNext() {
    if (matchCount === 0) return;
    currentMatch = currentMatch >= matchCount ? 1 : currentMatch + 1;
    scrollToMatch(currentMatch - 1);
  }

  function goPrev() {
    if (matchCount === 0) return;
    currentMatch = currentMatch <= 1 ? matchCount : currentMatch - 1;
    scrollToMatch(currentMatch - 1);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      handleClose();
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) {
        goPrev();
      } else {
        goNext();
      }
    }
  }

  function handleInput() {
    highlightMatches(query);
  }

  function handleClose() {
    clearHighlights();
    onClose();
  }

  // Focus input on mount
  $effect(() => {
    inputEl?.focus();
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="search-overlay" role="search" tabindex="-1" onkeydown={handleKeydown}>
  <div class="search-bar">
    <svg
      class="search-icon"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="11" cy="11" r="8"></circle>
      <path d="m21 21-4.3-4.3"></path>
    </svg>
    <input
      bind:this={inputEl}
      bind:value={query}
      oninput={handleInput}
      type="text"
      class="search-input"
      placeholder="Find in conversation…"
      aria-label="Search in conversation"
    />
    {#if query}
      <span class="match-count">
        {#if matchCount > 0}
          {currentMatch}/{matchCount}
        {:else}
          No matches
        {/if}
      </span>
    {/if}
    <div class="search-nav">
      <button
        class="nav-btn"
        onclick={goPrev}
        disabled={matchCount === 0}
        aria-label="Previous match"
        title="Previous (Shift+Enter)"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"><path d="m18 15-6-6-6 6" /></svg
        >
      </button>
      <button
        class="nav-btn"
        onclick={goNext}
        disabled={matchCount === 0}
        aria-label="Next match"
        title="Next (Enter)"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"><path d="m6 9 6 6 6-6" /></svg
        >
      </button>
    </div>
    <button
      class="close-btn"
      onclick={handleClose}
      aria-label="Close search"
      title="Close (Escape)"
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"><path d="M18 6 6 18" /><path d="m6 6 12 12" /></svg
      >
    </button>
  </div>
</div>

<style>
  .search-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    z-index: 50;
    padding: var(--spacing-sm) var(--spacing-xl);
    animation: fadeIn 150ms ease;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    max-width: var(--content-max-width);
    margin: 0 auto;
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--color-bg-input);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
  }

  .search-icon {
    flex-shrink: 0;
    color: var(--color-text-tertiary);
  }

  .search-input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    color: var(--color-text-primary);
  }

  .search-input::placeholder {
    color: var(--color-text-tertiary);
  }

  .match-count {
    flex-shrink: 0;
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .search-nav {
    display: flex;
    gap: 1px;
  }

  .nav-btn,
  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      background var(--transition-fast);
  }

  .nav-btn:hover:not(:disabled),
  .close-btn:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  /* ── Highlight styles (global because marks are injected into DOM) ── */

  :global(.search-highlight) {
    background: var(--color-accent-subtle);
    color: inherit;
    border-radius: 2px;
    padding: 0 1px;
  }

  :global(.search-highlight-active) {
    background: var(--color-accent-copper);
    color: var(--color-text-inverse);
  }
</style>
