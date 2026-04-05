<script lang="ts">
  import InputArea from "./InputArea.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import type { Message, ChatMessage } from "$lib/types/message";
  import { sendMessage, stopStreaming, updateConversation } from "$lib/utils/commands";
  import { onStreamingToken, onStreamingComplete, onStreamingError } from "$lib/utils/events";
  import { onMount, onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import {
    getConversationStore,
    newConversation,
    addMessage,
    updateMessageContent,
    appendStreamingToken,
    touchConversation,
    setConversationTitle,
    saveDraft,
    loadDraft,
    clearDraft,
  } from "$lib/stores/conversations.svelte";

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

  const store = getConversationStore();
  let chatContainer: HTMLElement | undefined = $state();
  let streaming = $state(false);
  let selectedModel = $state("gpt-4o");
  let draftText = $state("");
  let draftTimer: ReturnType<typeof setTimeout> | undefined;
  const greeting = greetings[Math.floor(Math.random() * greetings.length)];

  let unlistenToken: UnlistenFn | undefined;
  let unlistenComplete: UnlistenFn | undefined;
  let unlistenError: UnlistenFn | undefined;

  // Track the assistant message ID being streamed so we can persist on complete
  let streamingAssistantId: string | null = $state(null);

  onMount(async () => {
    // Load draft for active conversation
    if (store.activeConversationId) {
      draftText = await loadDraft(store.activeConversationId);
    }

    unlistenToken = await onStreamingToken((token) => {
      appendStreamingToken(token);
      requestAnimationFrame(() => {
        chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
      });
    });

    unlistenComplete = await onStreamingComplete(async () => {
      streaming = false;
      // Persist the completed assistant message
      if (streamingAssistantId) {
        const msg = store.messages.find((m) => m.id === streamingAssistantId);
        if (msg && msg.content) {
          await updateMessageContent(msg.id, msg.content, msg.thinkingContent);
          // Auto-generate title if this is the first exchange
          if (store.activeConversationId && store.messages.length <= 2) {
            generateTitle(store.activeConversationId, store.messages);
          }
        }
        streamingAssistantId = null;
      }
    });

    unlistenError = await onStreamingError((error) => {
      streaming = false;
      streamingAssistantId = null;
      const msgs = store.messages;
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant" && !last.content) {
        updateMessageContent(last.id, `⚠️ Error: ${error}`);
      }
    });
  });

  onDestroy(() => {
    unlistenToken?.();
    unlistenComplete?.();
    unlistenError?.();
    if (draftTimer) clearTimeout(draftTimer);
  });

  async function handleSend(text: string) {
    // Ensure we have an active conversation
    let convId = store.activeConversationId;
    if (!convId) {
      const conv = await newConversation(selectedModel);
      convId = conv.id;
    }

    // Clear draft
    clearDraft(convId);
    draftText = "";

    const userMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: convId,
      role: "user",
      content: text,
      createdAt: new Date().toISOString(),
      sortOrder: store.messages.length,
    };
    await addMessage(userMessage);

    // Create a placeholder assistant message for streaming into
    const assistantMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: convId,
      role: "assistant",
      content: "",
      createdAt: new Date().toISOString(),
      sortOrder: store.messages.length,
    };
    await addMessage(assistantMessage);
    streamingAssistantId = assistantMessage.id;
    streaming = true;

    touchConversation(convId);

    requestAnimationFrame(() => {
      chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
    });

    // Build API message array — include all user + non-empty assistant messages
    const apiMessages: ChatMessage[] = store.messages
      .filter((m) => m.role === "user" || (m.role === "assistant" && m.content))
      .map((m) => ({ role: m.role, content: m.content }));

    try {
      await sendMessage(apiMessages, selectedModel);
    } catch (e) {
      streaming = false;
      streamingAssistantId = null;
      const msgs = store.messages;
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant" && !last.content) {
        const errContent = `⚠️ Error: ${e instanceof Error ? e.message : String(e)}`;
        await updateMessageContent(last.id, errContent);
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

  function handleDraftChange(text: string) {
    draftText = text;
    if (draftTimer) clearTimeout(draftTimer);
    if (store.activeConversationId) {
      const convId = store.activeConversationId;
      draftTimer = setTimeout(() => {
        saveDraft(convId, text);
      }, 3000);
    }
  }

  /** Auto-generate a title from the first user message. */
  async function generateTitle(convId: string, msgs: Message[]): Promise<void> {
    const firstUser = msgs.find((m) => m.role === "user");
    if (!firstUser) return;
    // Simple heuristic: take first 50 chars of user's message as title
    const title =
      firstUser.content.length > 50 ? firstUser.content.slice(0, 49) + "…" : firstUser.content;
    try {
      await updateConversation(convId, title);
      setConversationTitle(convId, title);
    } catch (e) {
      console.error("Failed to set conversation title:", e);
    }
  }
</script>

<div class="chat-view">
  {#if store.messages.length === 0}
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
          initialValue={draftText}
          onInput={handleDraftChange}
        />
      </div>
    </div>
  {:else}
    <div class="chat-messages" bind:this={chatContainer} role="log" aria-label="Chat messages">
      <div class="messages-inner">
        {#each store.messages as message, i (message.id)}
          <div class="message-entry" style="animation-delay: {Math.min(i * 40, 200)}ms">
            <MessageBubble
              {message}
              isStreaming={streaming &&
                i === store.messages.length - 1 &&
                message.role === "assistant"}
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
        initialValue={draftText}
        onInput={handleDraftChange}
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
