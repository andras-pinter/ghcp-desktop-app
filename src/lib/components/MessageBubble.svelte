<script lang="ts">
  import type { Message } from "$lib/types/message";

  interface Props {
    message: Message;
  }

  let { message }: Props = $props();

  let isUser = $derived(message.role === "user");
</script>

<article class="message" class:user={isUser} class:assistant={!isUser}>
  <div class="message-role">
    <span class="role-icon" aria-hidden="true">
      {#if isUser}
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="8" cy="5" r="3" />
          <path d="M2 14c0-3.3 2.7-6 6-6s6 2.7 6 6" />
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M8 1.5a2.5 2.5 0 0 0-2.5 2.5v1A2.5 2.5 0 0 0 8 7.5 2.5 2.5 0 0 0 10.5 5V4A2.5 2.5 0 0 0 8 1.5zM4 9a1 1 0 0 0-1 1v1.5c0 .5.2 1 .5 1.3.8.8 2.2 1.7 4.5 1.7s3.7-.9 4.5-1.7c.3-.3.5-.8.5-1.3V10a1 1 0 0 0-1-1H4z"
          />
        </svg>
      {/if}
    </span>
    <span class="role-name">{isUser ? "You" : "Copilot"}</span>
  </div>
  <div class="message-content">
    <!-- TODO: Replace with proper markdown rendering -->
    {message.content}
  </div>
</article>

<style>
  .message {
    padding: var(--spacing-lg) 0;
  }

  .message + :global(.message) {
    border-top: 1px solid var(--color-border-secondary);
  }

  .message-role {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
  }

  .role-icon {
    display: flex;
    align-items: center;
    color: var(--color-text-tertiary);
  }

  .role-name {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .message-content {
    font-size: var(--font-size-base);
    line-height: var(--line-height-relaxed);
    color: var(--color-text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
