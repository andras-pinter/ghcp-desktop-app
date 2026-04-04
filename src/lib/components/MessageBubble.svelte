<script lang="ts">
  import type { Message } from "$lib/types/message";

  interface Props {
    message: Message;
    isStreaming?: boolean;
  }

  let { message, isStreaming = false }: Props = $props();

  let isUser = $derived(message.role === "user");
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
        {#if message.content}
          {message.content}{#if isStreaming}<span class="cursor">▊</span>{/if}
        {:else if isStreaming}
          <span class="thinking">Thinking<span class="dots">...</span></span>
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
    white-space: pre-wrap;
    word-break: break-word;
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

  .thinking {
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
