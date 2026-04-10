<script lang="ts">
  import type { Message } from "$lib/types/message";
  import type { SearchResult } from "$lib/types/web-research";
  import { renderMarkdown, CODE_BLOCK_CLASS, getLastCodeBlocks } from "$lib/utils/markdown";
  import CodeBlock from "./CodeBlock.svelte";
  import ThinkingSection from "./ThinkingSection.svelte";
  import WebResultCard from "./WebResultCard.svelte";
  import { mount, unmount } from "svelte";
  import { onDestroy } from "svelte";

  interface Props {
    message: Message;
    isStreaming?: boolean;
    isLastAssistant?: boolean;
    /** Web search results to display alongside this message. */
    webResults?: SearchResult[];
    onEdit?: (message: Message) => void;
    onRegenerate?: () => void;
  }

  let {
    message,
    isStreaming = false,
    isLastAssistant = false,
    onEdit,
    onRegenerate,
    webResults = [],
  }: Props = $props();

  let isUser = $derived(message.role === "user");
  let contentEl: HTMLElement | undefined = $state();
  let copied = $state(false);
  let copyTimeout: ReturnType<typeof setTimeout> | undefined;
  let renderTimer: ReturnType<typeof setTimeout> | undefined;

  const streamingPhrases = [
    "Pushing the throttle…",
    "Breaking through…",
    "Climbing to altitude…",
    "Gaining speed…",
    "Going supersonic…",
    "Punching through the clouds…",
    "Full afterburner…",
    "Reading the instruments…",
    "Locking in the vector…",
    "Cleared for approach…",
    "Riding the shockwave…",
    "Eyes on the horizon…",
  ];

  const streamingPhrase = streamingPhrases[Math.floor(Math.random() * streamingPhrases.length)];

  /** Track mounted CodeBlock instances for cleanup. */
  let mountedBlocks: Record<string, ReturnType<typeof mount>> = {};

  // ── Typewriter reveal ────────────────────────────────────────────
  // During streaming, characters are revealed progressively to create
  // a smooth typing effect. This decouples the bursty SSE token
  // delivery from the visual display.

  /** How many characters of message.content are currently visible.
   *  Initialized to current length so historical messages don't re-type. */
  const initialLength = message.content.length;
  let revealedLength = $state(initialLength);
  let revealRafId: number | null = null;
  let lastRenderTime = 0;

  /** True while the typewriter is still catching up (streaming or flushing). */
  let isRevealing = $state(false);

  const RENDER_INTERVAL = 60; // ms between markdown re-renders during typing
  const CHARS_PER_FRAME = 3; // base chars/frame at 60fps ≈ 180 chars/sec

  /** Get the content to render — sliced during reveal, full otherwise. */
  function getRevealedContent(): string {
    if (isUser) return message.content;
    if (isRevealing) return message.content.slice(0, revealedLength);
    return message.content;
  }

  function revealTick(): void {
    revealRafId = null;
    const target = message.content.length;

    if (revealedLength < target) {
      // Adaptive speed: catch up faster when buffer grows ahead
      const gap = target - revealedLength;
      const speed =
        gap > 200 ? Math.ceil(gap * 0.12) : gap > 60 ? CHARS_PER_FRAME * 3 : CHARS_PER_FRAME;
      revealedLength = Math.min(revealedLength + speed, target);
    }

    // Throttled markdown render
    const now = Date.now();
    if (now - lastRenderTime >= RENDER_INTERVAL) {
      lastRenderTime = now;
      renderAndMount();
    }

    // Keep loop alive while streaming or still flushing remaining chars
    if (isStreaming || revealedLength < message.content.length) {
      isRevealing = true;
      revealRafId = requestAnimationFrame(revealTick);
    } else {
      isRevealing = false;
      // Final render to ensure everything is properly formatted
      renderAndMount();
    }
  }

  // Start the reveal loop when streaming begins
  $effect(() => {
    if (isStreaming && !isUser && revealRafId === null) {
      isRevealing = true;
      revealRafId = requestAnimationFrame(revealTick);
    }
  });

  // When streaming ends, keep flushing if there are unrevealed chars.
  // The currently-running RAF loop handles this via its continuation
  // check, but if the loop already stopped (edge case: stream ends
  // exactly when revealedLength catches up), restart it.
  $effect(() => {
    if (!isStreaming && !isUser && revealedLength < message.content.length) {
      if (revealRafId === null) {
        isRevealing = true;
        revealRafId = requestAnimationFrame(revealTick);
      }
    }
  });

  /** Render markdown and mount CodeBlock components into placeholders. */
  function renderAndMount() {
    if (!contentEl || isUser) return;

    const html = renderMarkdown(getRevealedContent());
    const codeBlocks = getLastCodeBlocks();

    // eslint-disable-next-line svelte/no-dom-manipulating -- Intentional: we render sanitized markdown HTML and mount Svelte CodeBlock components into placeholders
    contentEl.innerHTML = html;

    // Clean up old CodeBlock mounts
    for (const comp of Object.values(mountedBlocks)) {
      unmount(comp);
    }
    mountedBlocks = {};

    // Find code block placeholders and mount interactive CodeBlock components.
    // Only mount blocks that have a corresponding renderer entry (by index)
    // to prevent spoofed HTML from injecting fake code blocks.
    const placeholders = contentEl.querySelectorAll(`.${CODE_BLOCK_CLASS}`);
    placeholders.forEach((el, i) => {
      if (i >= codeBlocks.length) return;
      const { code, lang } = codeBlocks[i];

      // Clear the placeholder's static fallback content
      el.innerHTML = "";

      const key = `cb-${i}`;
      mountedBlocks[key] = mount(CodeBlock, {
        target: el as HTMLElement,
        props: { code, lang },
      });
    });
  }

  // Non-streaming render: immediate on content load, conversation switch, etc.
  // During streaming/reveal, the RAF loop handles all rendering.
  $effect(() => {
    void message.content;
    void contentEl;

    if (isStreaming || isRevealing) return;
    if (renderTimer) clearTimeout(renderTimer);
    renderAndMount();
  });

  onDestroy(() => {
    for (const comp of Object.values(mountedBlocks)) {
      unmount(comp);
    }
    mountedBlocks = {};
    if (copyTimeout) clearTimeout(copyTimeout);
    if (renderTimer) clearTimeout(renderTimer);
    if (revealRafId !== null) {
      cancelAnimationFrame(revealRafId);
      revealRafId = null;
    }
  });

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(message.content);
      copied = true;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      // Clipboard API may not be available in all WebView contexts
    }
  }

  function handleEdit() {
    onEdit?.(message);
  }

  function handleRegenerate() {
    onRegenerate?.();
  }
</script>

<article class="message" class:user={isUser} class:assistant={!isUser}>
  {#if isUser}
    <div class="user-row">
      <div class="user-bubble">
        {message.content}
      </div>
      <div class="message-actions user-actions">
        {#if onEdit && !isStreaming}
          <button class="action-btn" onclick={handleEdit} aria-label="Edit message" title="Edit">
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" /><path
                d="m15 5 4 4"
              /></svg
            >
          </button>
        {/if}
        <button
          class="action-btn"
          onclick={handleCopy}
          aria-label="Copy message"
          title={copied ? "Copied!" : "Copy"}
        >
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
          {/if}
        </button>
      </div>
    </div>
  {:else}
    <div class="assistant-row">
      <div class="assistant-content">
        {#if message.thinkingContent}
          <ThinkingSection
            content={message.thinkingContent}
            isStreaming={isStreaming && !message.content}
          />
        {/if}
        {#if message.content}
          <div class="markdown-prose" bind:this={contentEl}></div>
        {/if}
        {#if isStreaming || isRevealing}
          <div class="streaming-indicator-spacer" aria-hidden="true"></div>
          <div class="streaming-indicator" aria-hidden="true">
            <span class="streaming-orb"></span>
            <span class="streaming-phrase">{streamingPhrase}</span>
          </div>
          <div class="streaming-scroll-runway" aria-hidden="true"></div>
        {/if}
        {#if webResults.length > 0}
          <nav class="web-results" aria-label="Web sources">
            {#each webResults as result (result.url)}
              <WebResultCard {result} />
            {/each}
          </nav>
        {/if}
        {#if !isStreaming && !isRevealing && message.content}
          <div class="message-actions assistant-actions">
            <button
              class="action-btn"
              onclick={handleCopy}
              aria-label="Copy message"
              title={copied ? "Copied!" : "Copy"}
            >
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
              {/if}
            </button>
            {#if isLastAssistant && onRegenerate}
              <button
                class="action-btn"
                onclick={handleRegenerate}
                aria-label="Regenerate response"
                title="Regenerate"
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path
                    d="M3 3v5h5"
                  /><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" /><path
                    d="M16 16h5v5"
                  /></svg
                >
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</article>

<style>
  .message {
    padding: var(--spacing-md) 0;
  }

  /* ── User: right-aligned bubble ── */

  .user-row {
    display: flex;
    justify-content: flex-end;
    align-items: flex-end;
    gap: var(--spacing-xs);
  }

  .user-bubble {
    max-width: 85%;
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--color-bg-tertiary);
    border-radius: var(--radius-lg) var(--radius-lg) var(--radius-sm) var(--radius-lg);
    font-size: var(--font-size-base);
    line-height: var(--line-height-relaxed);
    color: var(--color-text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }

  /* ── Assistant: left-aligned ── */

  .assistant-row {
    display: flex;
    align-items: flex-start;
  }

  .assistant-content {
    flex: 1;
    min-width: 0;
    font-size: var(--font-size-base);
    line-height: var(--line-height-relaxed);
    color: var(--color-text-primary);
  }

  .streaming-indicator-spacer {
    height: 1em;
  }

  /* Tall invisible zone below the indicator. Auto-scroll targets
     scrollHeight which includes this runway, so the indicator
     stays comfortably above the viewport edge even as content
     grows line-by-line — preventing the scroll-chase flap. */
  .streaming-scroll-runway {
    height: 4em;
  }

  .streaming-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    margin-bottom: 16px;
    min-height: 24px;
  }

  .streaming-orb {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-accent-copper);
    flex-shrink: 0;
    animation: orb-breathe 2s ease-in-out infinite;
    box-shadow: 0 0 6px rgba(180, 83, 9, 0.35);
  }

  @keyframes orb-breathe {
    0%,
    100% {
      transform: scale(1);
      box-shadow: 0 0 4px rgba(180, 83, 9, 0.25);
      opacity: 0.6;
    }
    50% {
      transform: scale(1.4);
      box-shadow: 0 0 12px rgba(180, 83, 9, 0.5);
      opacity: 1;
    }
  }

  .streaming-phrase {
    font-size: 0.78rem;
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  /* ── Message actions ── */

  .message-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .message:hover .message-actions,
  .message:focus-within .message-actions {
    opacity: 1;
  }

  .user-actions {
    flex-shrink: 0;
    order: -1;
  }

  .assistant-actions {
    margin-top: var(--spacing-xs);
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition:
      color var(--transition-fast),
      background var(--transition-fast);
  }

  .action-btn:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .action-btn:active {
    background: var(--color-bg-active);
  }

  /* ── Web results ── */

  .web-results {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-md);
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--color-border-primary);
  }
</style>
