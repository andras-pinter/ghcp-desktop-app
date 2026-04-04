<script lang="ts">
  import type { Message } from "$lib/types/message";

  interface Props {
    message: Message;
  }

  let { message }: Props = $props();

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
      <div class="role-avatar">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M8 1a3 3 0 0 0-3 3v1a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3zM4 9.5A1.5 1.5 0 0 0 2.5 11v.5c0 .7.3 1.3.7 1.7C4.2 14.1 5.8 15 8 15s3.8-.9 4.8-1.8c.4-.4.7-1 .7-1.7V11A1.5 1.5 0 0 0 12 9.5H4z"
          />
        </svg>
      </div>
      <div class="assistant-content">
        {message.content}
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

  /* ── Assistant: left-aligned with avatar ── */

  .assistant-row {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-sm);
  }

  .role-avatar {
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
    background: var(--color-accent-subtle);
    color: var(--color-accent-copper);
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
</style>
