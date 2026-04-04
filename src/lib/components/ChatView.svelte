<script lang="ts">
  import InputArea from "./InputArea.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import type { Message } from "$lib/types/message";

  // Placeholder messages for demonstration
  let messages: Message[] = $state([
    {
      id: "1",
      conversationId: "demo",
      role: "user",
      content: "Hello! What can you help me with?",
      createdAt: new Date().toISOString(),
      sortOrder: 0,
    },
    {
      id: "2",
      conversationId: "demo",
      role: "assistant",
      content:
        "I'm **Copilot Desktop** — your AI assistant. I can help with coding questions, research, brainstorming, and much more. Try asking me anything!",
      createdAt: new Date().toISOString(),
      sortOrder: 1,
    },
  ]);

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

    // Auto-scroll to bottom
    requestAnimationFrame(() => {
      chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
    });
  }
</script>

<div class="chat-view">
  <header class="chat-header">
    <h1 class="chat-title">New Conversation</h1>
  </header>

  <div class="chat-messages" bind:this={chatContainer} role="log" aria-label="Chat messages">
    {#each messages as message (message.id)}
      <MessageBubble {message} />
    {/each}

    {#if messages.length === 0}
      <div class="empty-state">
        <p class="empty-title">Start a conversation</p>
        <p class="empty-hint">Ask anything — coding, research, brainstorming.</p>
      </div>
    {/if}
  </div>

  <InputArea onSend={handleSend} />
</div>

<style>
  .chat-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .chat-header {
    padding: var(--spacing-md) var(--spacing-xl);
    border-bottom: 1px solid var(--color-border-secondary);
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .chat-title {
    font-size: var(--font-size-md);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-xl);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: var(--spacing-sm);
  }

  .empty-title {
    font-size: var(--font-size-lg);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .empty-hint {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
  }
</style>
