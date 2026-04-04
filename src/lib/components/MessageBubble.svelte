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
    {#if isUser}
      <div class="role-avatar user-avatar">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="8" cy="5.5" r="2.8" />
          <path d="M3 14.5c0-2.8 2.2-5 5-5s5 2.2 5 5" />
        </svg>
      </div>
    {:else}
      <div class="role-avatar assistant-avatar">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M8 1a3 3 0 0 0-3 3v1a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3zM4 9.5A1.5 1.5 0 0 0 2.5 11v.5c0 .7.3 1.3.7 1.7C4.2 14.1 5.8 15 8 15s3.8-.9 4.8-1.8c.4-.4.7-1 .7-1.7V11A1.5 1.5 0 0 0 12 9.5H4z"
          />
        </svg>
      </div>
    {/if}
    <span class="role-name">{isUser ? "You" : "Copilot"}</span>
  </div>
  <div class="message-content">
    {message.content}
  </div>
</article>

<style>
  .message {
    padding: var(--spacing-xl) 0;
  }

  .message + :global(.message-entry > .message) {
    border-top: 1px solid var(--color-border-secondary);
  }

  .message-role {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .role-avatar {
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .user-avatar {
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
  }

  .assistant-avatar {
    background: var(--color-accent-subtle);
    color: var(--color-accent-copper);
  }

  .role-name {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
    letter-spacing: var(--letter-spacing-normal);
  }

  .message-content {
    font-size: var(--font-size-base);
    line-height: var(--line-height-relaxed);
    color: var(--color-text-primary);
    white-space: pre-wrap;
    word-break: break-word;
    padding-left: 30px;
  }
</style>
