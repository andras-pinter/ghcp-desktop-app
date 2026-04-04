<script lang="ts">
  import InputArea from "./InputArea.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import type { Message } from "$lib/types/message";

  const greetings = [
    "Break through.",
    "Let's break some barriers.",
    "Ready to break the barrier?",
    "Past the barrier, into the unknown.",
    "What barrier are we breaking today?",
    "The sky was never the limit.",
    "Let's punch through.",
  ];

  let messages: Message[] = $state([]);
  let chatContainer: HTMLElement | undefined = $state();
  const greeting = greetings[Math.floor(Math.random() * greetings.length)];

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
          "I'm **Chuck** — a native desktop client for GitHub Copilot. I can help you with coding questions, research, brainstorming, and more.\n\nThis is a demo response. The streaming API integration is coming in Phase 2.",
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
  {#if messages.length === 0}
    <div class="welcome-container">
      <div class="welcome">
        <p class="welcome-greeting">{greeting}</p>
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

  .welcome-greeting {
    font-family: var(--font-display);
    font-style: italic;
    font-size: var(--font-size-xl);
    font-weight: 400;
    color: var(--color-text-secondary);
    letter-spacing: var(--letter-spacing-tight);
    line-height: var(--line-height-tight);
  }

  .welcome-input {
    width: 100%;
    max-width: var(--content-max-width);
    animation: fadeInUp 600ms ease both;
    animation-delay: 250ms;
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
