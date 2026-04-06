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
  }

  let { message, isStreaming = false }: Props = $props();

  let isUser = $derived(message.role === "user");
  let contentEl: HTMLElement | undefined = $state();

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
  });
</script>

<article class="message" class:user={isUser} class:assistant={!isUser}>
  {#if isUser}
    <div class="user-row">
      <div class="user-bubble">
        {message.content}
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
</style>
