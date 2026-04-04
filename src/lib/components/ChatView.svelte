<script lang="ts">
  import InputArea from "./InputArea.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import type { Message } from "$lib/types/message";

  interface Props {
    onToggleSidebar: () => void;
    sidebarCollapsed: boolean;
  }

  let { onToggleSidebar, sidebarCollapsed }: Props = $props();

  let messages: Message[] = $state([]);
  let chatContainer: HTMLElement | undefined = $state();

  function handleSend(text: string) {
    const userMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: "demo",
      role: "user",
      content: text,
      createdAt: new Date().toISOString(),
      sortOrder: messages.length,
    };
    messages = [...messages, userMessage];

    requestAnimationFrame(() => {
      chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
    });

    setTimeout(() => {
      const assistantMessage: Message = {
        id: crypto.randomUUID(),
        conversationId: "demo",
        role: "assistant",
        content:
          "I'm **Copilot Desktop** — a native desktop client for GitHub Copilot. I can help you with coding questions, research, brainstorming, and more.\n\nThis is a demo response. The streaming API integration is coming in Phase 2.",
        createdAt: new Date().toISOString(),
        sortOrder: messages.length,
      };
      messages = [...messages, assistantMessage];
      requestAnimationFrame(() => {
        chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
      });
    }, 600);
  }
</script>

<div class="chat-view">
  {#if sidebarCollapsed}
    <button
      class="sidebar-toggle"
      onclick={onToggleSidebar}
      aria-label="Open sidebar"
      title="Open sidebar (⌘⇧S)"
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
        <rect
          x="1"
          y="2"
          width="4"
          height="12"
          rx="1"
          stroke="currentColor"
          stroke-width="1.5"
          fill="none"
        />
        <rect
          x="6"
          y="2"
          width="9"
          height="12"
          rx="1"
          stroke="currentColor"
          stroke-width="1.5"
          fill="none"
        />
      </svg>
    </button>
  {/if}

  {#if messages.length === 0}
    <div class="welcome-container">
      <div class="welcome">
        <p class="welcome-eyebrow">Welcome to</p>
        <h1 class="welcome-title">Copilot Desktop</h1>
        <p class="welcome-subtitle">What are you working on?</p>
      </div>
      <div class="welcome-input">
        <InputArea onSend={handleSend} />
      </div>
    </div>
  {:else}
    <div class="chat-messages" bind:this={chatContainer} role="log" aria-label="Chat messages">
      <div class="messages-inner">
        {#each messages as message, i (message.id)}
          <div class="message-entry" style="animation-delay: {Math.min(i * 40, 200)}ms">
            <MessageBubble {message} />
          </div>
        {/each}
      </div>
    </div>
    <div class="chat-input-container">
      <InputArea onSend={handleSend} />
    </div>
  {/if}
</div>

<style>
  .chat-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    position: relative;
  }

  /* ── Welcome ── */

  .welcome-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    gap: var(--spacing-3xl);
    animation: fadeIn 600ms ease both;
  }

  .welcome {
    text-align: center;
    animation: fadeInUp 600ms ease both;
    animation-delay: 100ms;
  }

  .welcome-eyebrow {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: var(--spacing-sm);
  }

  .welcome-title {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-2xl);
    font-weight: 400;
    color: var(--color-text-primary);
    letter-spacing: var(--letter-spacing-tight);
    line-height: var(--line-height-tight);
    margin-bottom: var(--spacing-md);
  }

  .welcome-subtitle {
    font-size: var(--font-size-md);
    color: var(--color-text-secondary);
  }

  .welcome-input {
    width: 100%;
    max-width: var(--content-max-width);
    animation: fadeInUp 600ms ease both;
    animation-delay: 250ms;
  }

  /* ── Sidebar toggle ── */

  .sidebar-toggle {
    position: absolute;
    top: 14px;
    left: 14px;
    z-index: 10;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .sidebar-toggle:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  /* ── Messages ── */

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-2xl) 0 var(--spacing-lg);
  }

  .messages-inner {
    max-width: var(--content-max-width);
    margin: 0 auto;
    padding: 0 var(--spacing-xl);
    display: flex;
    flex-direction: column;
  }

  .message-entry {
    animation: fadeInUp 300ms ease both;
  }

  /* ── Bottom input ── */

  .chat-input-container {
    flex-shrink: 0;
    max-width: var(--content-max-width);
    width: 100%;
    margin: 0 auto;
    padding: var(--spacing-sm) var(--spacing-xl) var(--spacing-xl);
  }
</style>
