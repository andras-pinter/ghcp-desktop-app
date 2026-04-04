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

    // Simulate assistant response
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
  <!-- Sidebar toggle: always visible when sidebar is collapsed -->
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
    <!-- Empty state: centered welcome -->
    <div class="welcome-container">
      <div class="welcome">
        <h1 class="welcome-title">Copilot Desktop</h1>
        <p class="welcome-subtitle">How can I help you today?</p>
      </div>
      <div class="welcome-input">
        <InputArea onSend={handleSend} />
      </div>
    </div>
  {:else}
    <!-- Conversation view -->
    <div class="chat-messages" bind:this={chatContainer} role="log" aria-label="Chat messages">
      <div class="messages-inner">
        {#each messages as message (message.id)}
          <MessageBubble {message} />
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

  /* ── Welcome / empty state ── */

  .welcome-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    gap: var(--spacing-2xl);
  }

  .welcome {
    text-align: center;
  }

  .welcome-title {
    font-size: 28px;
    font-weight: 600;
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
    margin-bottom: var(--spacing-sm);
  }

  .welcome-subtitle {
    font-size: var(--font-size-md);
    color: var(--color-text-secondary);
  }

  .welcome-input {
    width: 100%;
    max-width: 680px;
  }

  /* ── Sidebar toggle (when collapsed) ── */

  .sidebar-toggle {
    position: absolute;
    top: 12px;
    left: 12px;
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

  /* ── Messages area ── */

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xl) 0;
  }

  .messages-inner {
    max-width: 680px;
    margin: 0 auto;
    padding: 0 var(--spacing-xl);
    display: flex;
    flex-direction: column;
  }

  /* ── Input container at bottom ── */

  .chat-input-container {
    flex-shrink: 0;
    max-width: 680px;
    width: 100%;
    margin: 0 auto;
    padding: 0 var(--spacing-xl) var(--spacing-xl);
  }
</style>
