<script lang="ts">
  import InputArea from "./InputArea.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import type { Message, ChatMessage } from "$lib/types/message";
  import { sendMessage, stopStreaming } from "$lib/utils/commands";
  import { onStreamingToken, onStreamingComplete, onStreamingError } from "$lib/utils/events";
  import { onMount, onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  const greetings = [
    "Your co-pilot is ready. Where to?",
    "Let's break some barriers.",
    "Ready to break the sound barrier?",
    "What barrier are we breaking today?",
    "The sky was never the limit.",
    "Cleared for takeoff. What's the mission?",
    "Strapped in and ready. Let's punch through.",
    "Co-pilot on deck. Say the word.",
  ];

  let messages: Message[] = $state([]);
  let chatContainer: HTMLElement | undefined = $state();
  let streaming = $state(false);
  let selectedModel = $state("gpt-4o");
  const greeting = greetings[Math.floor(Math.random() * greetings.length)];

  let unlistenToken: UnlistenFn | undefined;
  let unlistenComplete: UnlistenFn | undefined;
  let unlistenError: UnlistenFn | undefined;

  onMount(async () => {
    unlistenToken = await onStreamingToken((token) => {
      // Append token to the last assistant message
      const last = messages[messages.length - 1];
      if (last && last.role === "assistant") {
        messages = messages.map((m, i) =>
          i === messages.length - 1 ? { ...m, content: m.content + token } : m,
        );
        requestAnimationFrame(() => {
          chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
        });
      }
    });

    unlistenComplete = await onStreamingComplete(() => {
      streaming = false;
    });

    unlistenError = await onStreamingError((error) => {
      streaming = false;
      const last = messages[messages.length - 1];
      if (last && last.role === "assistant" && !last.content) {
        messages = messages.map((m, i) =>
          i === messages.length - 1 ? { ...m, content: `⚠️ Error: ${error}` } : m,
        );
      }
    });
  });

  onDestroy(() => {
    unlistenToken?.();
    unlistenComplete?.();
    unlistenError?.();
  });

  async function handleSend(text: string) {
    const userMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: "current",
      role: "user",
      content: text,
      createdAt: new Date().toISOString(),
      sortOrder: messages.length,
    };
    messages = [...messages, userMessage];

    // Create a placeholder assistant message for streaming into
    const assistantMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: "current",
      role: "assistant",
      content: "",
      createdAt: new Date().toISOString(),
      sortOrder: messages.length + 1,
    };
    messages = [...messages, assistantMessage];
    streaming = true;

    requestAnimationFrame(() => {
      chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
    });

    // Build API message array from conversation history
    const apiMessages: ChatMessage[] = messages
      .filter((m) => m.role === "user" || (m.role === "assistant" && m.content))
      .slice(0, -1) // Exclude the empty assistant placeholder
      .map((m) => ({ role: m.role, content: m.content }));

    try {
      await sendMessage(apiMessages, selectedModel);
    } catch (e) {
      streaming = false;
      const last = messages[messages.length - 1];
      if (last && last.role === "assistant" && !last.content) {
        messages = messages.map((m, i) =>
          i === messages.length - 1
            ? { ...m, content: `⚠️ Error: ${e instanceof Error ? e.message : String(e)}` }
            : m,
        );
      }
    }
  }

  async function handleStop() {
    try {
      await stopStreaming();
    } catch {
      // Ignore
    }
  }

  function handleModelChange(model: string) {
    selectedModel = model;
  }
</script>

<div class="chat-view">
  {#if messages.length === 0}
    <div class="welcome-container">
      <div class="welcome">
        <p class="welcome-greeting">{greeting}</p>
      </div>
      <div class="welcome-input">
        <InputArea
          onSend={handleSend}
          {streaming}
          onStop={handleStop}
          model={selectedModel}
          onModelChange={handleModelChange}
        />
      </div>
    </div>
  {:else}
    <div class="chat-messages" bind:this={chatContainer} role="log" aria-label="Chat messages">
      <div class="messages-inner">
        {#each messages as message, i (message.id)}
          <div class="message-entry" style="animation-delay: {Math.min(i * 40, 200)}ms">
            <MessageBubble
              {message}
              isStreaming={streaming && i === messages.length - 1 && message.role === "assistant"}
            />
          </div>
        {/each}
      </div>
    </div>
    <div class="chat-input-container">
      <InputArea
        onSend={handleSend}
        {streaming}
        onStop={handleStop}
        model={selectedModel}
        onModelChange={handleModelChange}
      />
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
