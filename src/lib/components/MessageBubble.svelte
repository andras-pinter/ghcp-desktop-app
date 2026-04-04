<script lang="ts">
  import type { Message } from "$lib/types/message";

  interface Props {
    message: Message;
  }

  let { message }: Props = $props();

  let isUser = $derived(message.role === "user");
  let timestamp = $derived(
    new Date(message.createdAt).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    }),
  );
</script>

<article class="message-bubble" class:user={isUser} class:assistant={!isUser}>
  <div class="message-header">
    <span class="message-avatar" aria-hidden="true">
      {isUser ? "👤" : "🤖"}
    </span>
    <span class="message-role">{isUser ? "You" : "Copilot"}</span>
    <time class="message-time" datetime={message.createdAt}>{timestamp}</time>
  </div>
  <div class="message-content">
    <!-- TODO: Replace with proper markdown rendering -->
    {message.content}
  </div>
</article>

<style>
  .message-bubble {
    max-width: 800px;
    width: 100%;
    margin: 0 auto;
  }

  .message-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-xs);
  }

  .message-avatar {
    font-size: var(--font-size-md);
  }

  .message-role {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .message-time {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    margin-left: auto;
  }

  .message-content {
    padding: var(--spacing-md);
    border-radius: var(--radius-md);
    font-size: var(--font-size-base);
    line-height: var(--line-height-relaxed);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .user .message-content {
    background: var(--color-bg-secondary);
  }

  .assistant .message-content {
    background: var(--color-bg-tertiary);
  }
</style>
