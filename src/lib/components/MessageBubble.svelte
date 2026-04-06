<script lang="ts">
  import type { Message } from "$lib/types/message";
  import { renderMarkdown, CODE_BLOCK_CLASS } from "$lib/utils/markdown";
  import CodeBlock from "./CodeBlock.svelte";
  import ThinkingSection from "./ThinkingSection.svelte";
  import { mount, unmount } from "svelte";
  import { onMount, onDestroy } from "svelte";

  interface Props {
    message: Message;
    isStreaming?: boolean;
    isLastAssistant?: boolean;
    onEdit?: (message: Message) => void;
    onRegenerate?: () => void;
    onCopy?: (content: string) => void;
  }

  let {
    message,
    isStreaming = false,
    isLastAssistant = false,
    onEdit,
    onRegenerate,
    onCopy,
  }: Props = $props();

  let isUser = $derived(message.role === "user");
  let contentEl: HTMLElement | undefined = $state();
  let copied = $state(false);
  let copyTimeout: ReturnType<typeof setTimeout> | undefined;

  /** Track mounted CodeBlock instances for cleanup. */
  let mountedBlocks: Record<string, ReturnType<typeof mount>> = {};

  /** Render markdown and mount CodeBlock components into placeholders. */
  function renderAndMount() {
    if (!contentEl || isUser) return;

    const html = renderMarkdown(message.content);
    // eslint-disable-next-line svelte/no-dom-manipulating -- Intentional: we render sanitized markdown HTML and mount Svelte CodeBlock components into placeholders
    contentEl.innerHTML = html;

    // Clean up old CodeBlock mounts
    for (const comp of Object.values(mountedBlocks)) {
      unmount(comp);
    }
    mountedBlocks = {};

    // Find code block placeholders and mount interactive CodeBlock components
    const placeholders = contentEl.querySelectorAll(`.${CODE_BLOCK_CLASS}`);
    placeholders.forEach((el, i) => {
      const code = el.getAttribute("data-code") || "";
      const lang = el.getAttribute("data-lang") || undefined;

      // Clear the placeholder's static fallback content
      el.innerHTML = "";

      const key = `cb-${i}`;
      mountedBlocks[key] = mount(CodeBlock, {
        target: el as HTMLElement,
        props: { code, lang },
      });
    });
  }

  // Re-render on content change (streaming tokens)
  $effect(() => {
    void message.content;
    void contentEl;
    renderAndMount();
  });

  onMount(() => {
    renderAndMount();
  });

  onDestroy(() => {
    for (const comp of Object.values(mountedBlocks)) {
      unmount(comp);
    }
    mountedBlocks = {};
    if (copyTimeout) clearTimeout(copyTimeout);
  });

  function handleCopy() {
    const text = message.content;
    navigator.clipboard.writeText(text).then(() => {
      copied = true;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copied = false;
      }, 2000);
    });
    onCopy?.(text);
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
          {#if isStreaming}<span class="cursor">▊</span>{/if}
        {:else if isStreaming}
          {#if !message.thinkingContent}
            <span class="thinking-placeholder">Thinking<span class="dots">...</span></span>
          {/if}
        {/if}
        {#if !isStreaming && message.content}
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

  .cursor {
    animation: blink 600ms steps(1) infinite;
    color: var(--color-accent-copper);
    font-weight: 100;
  }

  @keyframes blink {
    50% {
      opacity: 0;
    }
  }

  .thinking-placeholder {
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .dots {
    animation: dots 1.5s steps(4) infinite;
    display: inline-block;
    width: 1.5em;
    text-align: left;
    overflow: hidden;
    vertical-align: bottom;
  }

  @keyframes dots {
    0% {
      width: 0;
    }
    100% {
      width: 1.5em;
    }
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
</style>
