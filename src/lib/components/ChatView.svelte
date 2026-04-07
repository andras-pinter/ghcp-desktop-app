<script lang="ts">
  import InputArea from "./InputArea.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import SearchOverlay from "./SearchOverlay.svelte";
  import type { Message, ChatMessage } from "$lib/types/message";
  import type { UrlPreview } from "$lib/types/web-research";
  import type { ChatFileData } from "$lib/types/project";
  import {
    sendMessage,
    stopStreaming,
    updateConversation,
    extractFileText,
    readDroppedFiles,
  } from "$lib/utils/commands";
  import {
    onStreamingToken,
    onStreamingComplete,
    onStreamingError,
    onContextSummarized,
  } from "$lib/utils/events";
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import {
    getConversationStore,
    newConversation,
    addMessage,
    updateMessageContent as updateMessageContentStore,
    appendStreamingToken,
    touchConversation,
    setConversationTitle,
    saveDraft,
    loadDraft,
    clearDraft,
    deleteMessagesAfter,
  } from "$lib/stores/conversations.svelte";
  import { getModelStore, setDefaultModel } from "$lib/stores/models.svelte";
  import { getAgentStore, selectAgent } from "$lib/stores/agents.svelte";

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
  const modelStore = getModelStore();
  const agentStore = getAgentStore();
  let chatContainer: HTMLElement | undefined = $state();
  let streaming = $state(false);
  let selectedModel = $state("gpt-4o");
  let draftText = $state("");
  let draftTimer: ReturnType<typeof setTimeout> | undefined;
  let showSearch = $state(false);
  let extractingFiles = $state(false);
  const greeting = greetings[Math.floor(Math.random() * greetings.length)];

  // Full-window drag-and-drop via Tauri's native onDragDropEvent.
  // HTML5 drag events don't receive file data in Tauri's webview — the
  // native layer intercepts OS-level file drops and provides file paths.
  let viewDropActive = $state(false);
  let pendingDropFiles: ChatFileData[] = $state([]);
  let unlistenDragDrop: UnlistenFn | undefined;

  async function setupDragDrop() {
    const webview = getCurrentWebview();
    unlistenDragDrop = await webview.onDragDropEvent(async (event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        viewDropActive = true;
      } else if (event.payload.type === "leave") {
        viewDropActive = false;
      } else if (event.payload.type === "drop") {
        viewDropActive = false;
        const paths = event.payload.paths;
        if (!paths || paths.length === 0) return;
        try {
          const files = await readDroppedFiles(paths);
          if (files.length > 0) {
            pendingDropFiles = files;
          }
        } catch (e) {
          console.error("Failed to read dropped files:", e);
        }
      }
    });
  }

  function clearPendingDropFiles() {
    pendingDropFiles = [];
  }

  let unlistenToken: UnlistenFn | undefined;
  let unlistenComplete: UnlistenFn | undefined;
  let unlistenError: UnlistenFn | undefined;
  let unlistenSummarized: UnlistenFn | undefined;

  /** Number of older messages that were summarized in the current conversation. */
  let summarizedCount = $state(0);

  // Use persisted default model on first load, then fall back to first available
  let defaultApplied = false;
  $effect(() => {
    if (modelStore.loaded && modelStore.models.length > 0 && !defaultApplied) {
      defaultApplied = true;
      if (
        modelStore.defaultModelId &&
        modelStore.models.some((m) => m.id === modelStore.defaultModelId)
      ) {
        selectedModel = modelStore.defaultModelId;
      } else if (!modelStore.models.some((m) => m.id === selectedModel)) {
        selectedModel = modelStore.models[0].id;
      }
    }
  });

  // Reset summarization banner when switching conversations
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    store.activeConversationId;
    summarizedCount = 0;
  });

  // Track the assistant message ID being streamed so we can persist on complete
  let streamingAssistantId: string | null = $state(null);

  onMount(async () => {
    // Load draft for active conversation
    if (store.activeConversationId) {
      draftText = await loadDraft(store.activeConversationId);
    }

    setupDragDrop();

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
          await updateMessageContentStore(msg.id, msg.content, msg.thinkingContent);
          // Auto-generate title if this is the first exchange (best-effort)
          if (store.activeConversationId && store.messages.length <= 2) {
            generateTitle(store.activeConversationId, store.messages).catch(() => {});
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
        updateMessageContentStore(last.id, `⚠️ Error: ${error}`);
      }
    });

    unlistenSummarized = await onContextSummarized((payload) => {
      summarizedCount = payload.count;
    });
  });

  onDestroy(() => {
    unlistenToken?.();
    unlistenComplete?.();
    unlistenError?.();
    unlistenSummarized?.();
    unlistenDragDrop?.();
    if (draftTimer) clearTimeout(draftTimer);
  });

  async function handleSend(text: string, urls?: UrlPreview[], files?: ChatFileData[]) {
    // Ensure we have an active conversation
    let convId = store.activeConversationId;
    if (!convId) {
      const conv = await newConversation(selectedModel);
      convId = conv.id;
    }

    // Clear draft and cancel any pending save
    if (draftTimer) clearTimeout(draftTimer);
    clearDraft(convId);
    draftText = "";

    // Build user message content — append URL context if provided
    let content = text;
    if (urls && urls.length > 0) {
      const urlContext = urls
        .filter((u) => u.content)
        .map((u) => {
          const title = u.content?.title ? ` — ${u.content.title}` : "";
          const truncNote = u.content?.truncated ? " (truncated)" : "";
          return `\n\n---\n📎 [${u.domain}${title}](${u.url})${truncNote}\n\n${u.content?.text}`;
        })
        .join("");
      if (urlContext) {
        content = text + urlContext;
      }
    }

    // Append file content as context — use Rust text extraction for all formats
    if (files && files.length > 0) {
      extractingFiles = true;
      const extractedParts: string[] = [];
      for (const f of files) {
        try {
          console.log(`Extracting text from: ${f.name} (${f.contentType}, ${f.size} bytes)`);
          const extracted = await extractFileText(f.contentBase64, f.contentType, f.name);
          console.log(
            `Extraction result for ${f.name}: ${extracted ? `${extracted.length} chars` : "null (unsupported)"}`,
          );
          if (extracted) {
            const truncated =
              extracted.length > 50_000
                ? extracted.slice(0, 50_000) + `\n...[truncated, ${extracted.length} chars total]`
                : extracted;
            extractedParts.push(
              `\n\n---\n📎 ${f.name} (${f.contentType})\n\n\`\`\`\n${truncated}\n\`\`\``,
            );
          } else {
            extractedParts.push(
              `\n\n---\n📎 ${f.name} (${f.contentType}, unsupported format — content not shown)`,
            );
          }
        } catch (err) {
          console.error(`Extraction error for ${f.name}:`, err);
          extractedParts.push(`\n\n---\n📎 ${f.name} (${f.contentType}, extraction failed)`);
        }
      }
      extractingFiles = false;
      const fileContext = extractedParts.join("");
      if (fileContext) {
        content = content + fileContext;
      }
    }

    const userMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: convId,
      role: "user",
      content,
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
      await sendMessage(
        apiMessages,
        selectedModel,
        agentStore.selectedAgentId,
        store.activeConversation?.projectId,
      );
    } catch (e) {
      streaming = false;
      streamingAssistantId = null;
      const msgs = store.messages;
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant" && !last.content) {
        const errContent = `⚠️ Error: ${e instanceof Error ? e.message : String(e)}`;
        await updateMessageContentStore(last.id, errContent);
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

  /** Edit a user message: discard it and everything after, load content into input. */
  async function handleEdit(msg: Message) {
    if (streaming || !store.activeConversationId) return;

    // Delete the edited message AND all messages after it (sortOrder - 1 keeps msgs before)
    await deleteMessagesAfter(store.activeConversationId, msg.sortOrder - 1);

    // Put the edited message content back in the input for the user to modify
    draftText = msg.content;
  }

  /** Regenerate the last assistant response. */
  async function handleRegenerate() {
    if (streaming || !store.activeConversationId) return;

    // Defensively stop any in-flight stream before starting a new one
    try {
      await stopStreaming();
    } catch {
      // ignore — may not be streaming
    }

    const msgs = store.messages;
    const lastAssistant = [...msgs].reverse().find((m) => m.role === "assistant");
    if (!lastAssistant) return;

    // Delete the last assistant message (and anything after it)
    await deleteMessagesAfter(store.activeConversationId, lastAssistant.sortOrder - 1);

    // Re-create a placeholder assistant message for streaming
    const assistantMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: store.activeConversationId,
      role: "assistant",
      content: "",
      createdAt: new Date().toISOString(),
      sortOrder: store.messages.length,
    };
    await addMessage(assistantMessage);
    streamingAssistantId = assistantMessage.id;
    streaming = true;

    requestAnimationFrame(() => {
      chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
    });

    // Build API messages from what remains
    const apiMessages: ChatMessage[] = store.messages
      .filter((m) => m.role === "user" || (m.role === "assistant" && m.content))
      .map((m) => ({ role: m.role, content: m.content }));

    try {
      await sendMessage(
        apiMessages,
        selectedModel,
        agentStore.selectedAgentId,
        store.activeConversation?.projectId,
      );
    } catch (e) {
      streaming = false;
      streamingAssistantId = null;
      const last = store.messages[store.messages.length - 1];
      if (last && last.role === "assistant" && !last.content) {
        const errContent = `⚠️ Error: ${e instanceof Error ? e.message : String(e)}`;
        await updateMessageContentStore(last.id, errContent);
      }
    }
  }

  /** Handle Cmd/Ctrl+F to open search overlay. */
  function handleGlobalKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (mod && e.key === "f" && store.messages.length > 0) {
      e.preventDefault();
      showSearch = true;
    }
  }

  /** Compute index of last assistant message. */
  let lastAssistantIndex = $derived(
    (() => {
      for (let i = store.messages.length - 1; i >= 0; i--) {
        if (store.messages[i].role === "assistant") return i;
      }
      return -1;
    })(),
  );
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="chat-view"
  role="region"
  aria-label="Chat"
  tabindex="-1"
  onkeydown={handleGlobalKeydown}
>
  {#if viewDropActive}
    <div class="drop-overlay" role="status" aria-label="Drop files to attach">
      <div class="drop-overlay-inner">
        <span class="drop-icon">📎</span>
        <span class="drop-text">Drop files to attach</span>
      </div>
    </div>
  {/if}

  {#if store.messages.length === 0}
    <div class="welcome-container">
      <div class="welcome">
        <p class="welcome-greeting">{greeting}</p>
      </div>
      <div class="welcome-input">
        <InputArea
          onSend={handleSend}
          {streaming}
          {extractingFiles}
          onStop={handleStop}
          model={selectedModel}
          onModelChange={handleModelChange}
          availableModels={modelStore.models}
          modelsLoaded={modelStore.loaded}
          defaultModelId={modelStore.defaultModelId}
          onSetDefault={setDefaultModel}
          initialValue={draftText}
          onInput={handleDraftChange}
          agents={agentStore.agents}
          agentsLoaded={agentStore.loaded}
          selectedAgentId={agentStore.selectedAgentId}
          onAgentChange={selectAgent}
          externalFiles={pendingDropFiles}
          onExternalFilesConsumed={clearPendingDropFiles}
        />
      </div>
    </div>
  {:else}
    {#if showSearch}
      <SearchOverlay {chatContainer} onClose={() => (showSearch = false)} />
    {/if}
    <div class="chat-messages" bind:this={chatContainer} role="log" aria-label="Chat messages">
      <div class="messages-inner">
        {#if summarizedCount > 0}
          <div class="context-summary-banner" role="status">
            <span class="summary-icon">ℹ️</span>
            <span
              >{summarizedCount} older message{summarizedCount === 1 ? "" : "s"} condensed into summary</span
            >
            <button
              class="summary-dismiss"
              onclick={() => (summarizedCount = 0)}
              aria-label="Dismiss"
            >
              ✕
            </button>
          </div>
        {/if}
        {#each store.messages as message, i (message.id)}
          <div class="message-entry" style:animation-delay="{Math.min(i * 40, 200)}ms">
            <MessageBubble
              {message}
              isStreaming={streaming &&
                i === store.messages.length - 1 &&
                message.role === "assistant"}
              isLastAssistant={i === lastAssistantIndex}
              onEdit={handleEdit}
              onRegenerate={handleRegenerate}
            />
          </div>
        {/each}
      </div>
    </div>
    <div class="chat-input-container">
      <InputArea
        onSend={handleSend}
        {streaming}
        {extractingFiles}
        onStop={handleStop}
        model={selectedModel}
        onModelChange={handleModelChange}
        availableModels={modelStore.models}
        modelsLoaded={modelStore.loaded}
        defaultModelId={modelStore.defaultModelId}
        onSetDefault={setDefaultModel}
        initialValue={draftText}
        onInput={handleDraftChange}
        agents={agentStore.agents}
        agentsLoaded={agentStore.loaded}
        selectedAgentId={agentStore.selectedAgentId}
        onAgentChange={selectAgent}
        externalFiles={pendingDropFiles}
        onExternalFilesConsumed={clearPendingDropFiles}
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

  /* ── Drop overlay ── */

  .drop-overlay {
    position: absolute;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--color-bg) 85%, transparent);
    backdrop-filter: blur(4px);
    animation: fadeIn 150ms ease both;
  }

  .drop-overlay-inner {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-3xl) var(--spacing-4xl);
    border: 2px dashed var(--color-accent-copper);
    border-radius: var(--radius-xl);
    background: color-mix(in srgb, var(--color-accent-copper) 6%, transparent);
  }

  .drop-icon {
    font-size: 2.5rem;
  }

  .drop-text {
    font-family: var(--font-body);
    font-size: var(--font-size-lg);
    font-weight: 500;
    color: var(--color-accent-copper);
    letter-spacing: var(--letter-spacing-tight);
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

  .context-summary-banner {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    margin-bottom: var(--spacing-md);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-secondary);
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    animation: fadeIn 300ms ease both;
  }

  .summary-icon {
    flex-shrink: 0;
  }

  .summary-dismiss {
    margin-left: auto;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-secondary);
    padding: 2px 6px;
    border-radius: var(--radius-xs);
    font-size: var(--font-size-sm);
    opacity: 0.6;
    transition: opacity 150ms ease;
  }

  .summary-dismiss:hover {
    opacity: 1;
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
