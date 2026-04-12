<script lang="ts">
  import { SvelteMap } from "svelte/reactivity";
  import InputArea from "./InputArea.svelte";
  import MessageBubble from "./MessageBubble.svelte";
  import SearchOverlay from "./SearchOverlay.svelte";
  import type { Message, ChatMessage } from "$lib/types/message";
  import type { UrlPreview } from "$lib/types/web-research";
  import type { ChatFileData } from "$lib/types/project";
  import type { MessageOverrides } from "$lib/types/commands";
  import {
    sendMessage,
    stopStreaming,
    extractFileText,
    readDroppedFiles,
    exportConversationMarkdown,
    exportConversationJson,
    saveExportFile,
    webSearch,
  } from "$lib/utils/commands";
  import { onContextSummarized } from "$lib/utils/events";
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import {
    getConversationStore,
    newConversation,
    addMessage,
    updateMessageContent as updateMessageContentStore,
    touchConversation,
    saveDraft,
    loadDraft,
    clearDraft,
    deleteMessagesAfter,
    startStreaming,
    isStreaming as isConvStreaming,
    cancelStreamingState,
    renameConversation,
    toggleFavourite,
  } from "$lib/stores/conversations.svelte";
  import { getModelStore, setDefaultModel } from "$lib/stores/models.svelte";
  import { getAgentStore, selectAgent } from "$lib/stores/agents.svelte";
  import { getSettings } from "$lib/stores/settings.svelte";
  import { getNetwork } from "$lib/stores/network.svelte";
  import { getSkillStore } from "$lib/stores/skills.svelte";
  import type { SearchResult } from "$lib/types/web-research";
  import { SLASH_COMMANDS } from "$lib/types/commands";

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
  const skillStore = getSkillStore();
  const settings = getSettings();
  const network = getNetwork();
  let chatContainer: HTMLElement | undefined = $state();
  let selectedModel = $state("gpt-4o");
  let draftText = $state("");
  let draftTimer: ReturnType<typeof setTimeout> | undefined;
  let showSearch = $state(false);
  let extractingFiles = $state(false);
  let showHelpModal = $state(false);
  const greeting = greetings[Math.floor(Math.random() * greetings.length)];

  // Derive streaming state from store (not local)
  let streaming = $derived(
    store.activeConversationId ? isConvStreaming(store.activeConversationId) : false,
  );

  // Full-window drag-and-drop via Tauri's native onDragDropEvent.
  // HTML5 drag events don't receive file data in Tauri's webview — the
  // native layer intercepts OS-level file drops and provides file paths.
  let viewDropActive = $state(false);
  let pendingDropFiles: ChatFileData[] = $state([]);
  let unlistenDragDrop: UnlistenFn | undefined;

  // Background extraction: cache promises keyed by filename, reactive status for pill UI.
  // Separate from pendingDropFiles so clearing pills doesn't lose extraction results.
  const extractionCache = new SvelteMap<string, Promise<string | null>>();
  let extractionStatuses = $state<Record<string, "extracting" | "done" | "error">>({});

  /** Escape a filename for safe embedding in markdown. Prevents XSS via crafted filenames. */
  function sanitizeFilename(name: string): string {
    let result = "";
    for (const ch of name) {
      const code = ch.charCodeAt(0);
      if (code < 0x20) continue; // strip control characters
      if ("<>&\"'`\\".includes(ch)) {
        result += "_"; // replace chars that could break HTML or markdown fences
      } else {
        result += ch;
      }
    }
    return result;
  }

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

        // Show placeholder pills instantly (name from path, loading state)
        const placeholders: ChatFileData[] = paths.map((p) => ({
          name: p.split("/").pop() || p.split("\\").pop() || p,
          contentType: "application/octet-stream",
          size: 0,
          contentBase64: "",
          loading: true,
        }));
        pendingDropFiles = placeholders;

        // Read actual content, then start extraction immediately in background.
        // Set extraction status BEFORE updating pendingDropFiles so there's no
        // window where loading=false but extraction hasn't started (race condition).
        try {
          // Paths are validated server-side against the OS drag-drop handler
          const files = await readDroppedFiles(paths);
          if (files.length > 0) {
            startExtractions(files);
            pendingDropFiles = files;
          }
        } catch (e) {
          console.error("Failed to read dropped files:", e);
          pendingDropFiles = [];
        }
      }
    });
  }

  /** Start text extraction for each file in parallel, caching promises. */
  function startExtractions(files: ChatFileData[]) {
    for (const file of files) {
      extractionStatuses = { ...extractionStatuses, [file.name]: "extracting" };
      const promise = extractFileText(file.contentBase64, file.contentType, file.name)
        .then((text) => {
          extractionStatuses = { ...extractionStatuses, [file.name]: "done" };
          return text;
        })
        .catch((err) => {
          console.error(`Extraction error for ${file.name}:`, err);
          extractionStatuses = { ...extractionStatuses, [file.name]: "error" };
          return null;
        });
      extractionCache.set(file.name, promise);
    }
  }

  function clearPendingDropFiles() {
    pendingDropFiles = [];
  }

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

  // Reset summarization banner and scroll state when switching conversations
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    store.activeConversationId;
    summarizedCount = 0;
    userScrolledAway = false;
  });

  // Auto-scroll when active conversation receives streaming tokens.
  // Uses RAF instead of setInterval so scroll is synchronized with the
  // browser's render cycle — no gap between content growth and scroll.
  let scrollRafId: number | null = null;
  let userScrolledAway = false;

  function handleChatScroll(): void {
    if (!chatContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = chatContainer;
    // Consider "near bottom" if within 150px of the end
    userScrolledAway = scrollHeight - scrollTop - clientHeight > 150;
  }

  function scrollFollowLoop(): void {
    if (streaming && chatContainer && !userScrolledAway) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
    if (streaming) {
      scrollRafId = requestAnimationFrame(scrollFollowLoop);
    } else {
      scrollRafId = null;
    }
  }

  // Start/stop the scroll-follow RAF loop when streaming state changes
  $effect(() => {
    if (streaming && scrollRafId === null) {
      scrollRafId = requestAnimationFrame(scrollFollowLoop);
    }
    return () => {
      if (scrollRafId !== null) {
        cancelAnimationFrame(scrollRafId);
        scrollRafId = null;
      }
    };
  });

  onMount(async () => {
    // Load draft for active conversation
    if (store.activeConversationId) {
      draftText = await loadDraft(store.activeConversationId);
    }

    setupDragDrop();

    unlistenSummarized = await onContextSummarized((payload) => {
      summarizedCount = payload.count;
    });
  });

  onDestroy(() => {
    unlistenSummarized?.();
    unlistenDragDrop?.();
    if (scrollRafId !== null) cancelAnimationFrame(scrollRafId);
    if (draftTimer) clearTimeout(draftTimer);
  });

  async function handleSend(
    text: string,
    urls?: UrlPreview[],
    files?: ChatFileData[],
    overrides?: MessageOverrides,
  ) {
    // User just sent a message — follow the response
    userScrolledAway = false;

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

    // Build user message content — start with text + URL context
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

    const hasFiles = files && files.length > 0;

    // Add user message immediately so the UI switches to chat view.
    const fileLabelsImmediate = hasFiles
      ? "\n\n" +
        files
          .map((f) => {
            const safe = sanitizeFilename(f.name);
            const status = extractionStatuses[f.name];
            if (status === "done") return `📎 ${safe}`;
            if (status === "error") return `📎 ${safe} (not extractable)`;
            return `📎 ${safe} — extracting…`;
          })
          .join("\n")
      : "";
    const userMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: convId,
      role: "user",
      content: content + fileLabelsImmediate,
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
    startStreaming(convId, assistantMessage.id);

    touchConversation(convId);

    requestAnimationFrame(() => {
      chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
    });

    // Collect extracted text from the background extraction cache.
    // If user sent before extraction finished, await the cached promise (not a new call).
    let extractedParts: string[] = [];
    if (hasFiles) {
      const stillExtracting = files.some(
        (f) => extractionStatuses[f.name] === "extracting" || !extractionStatuses[f.name],
      );
      if (stillExtracting) extractingFiles = true;

      for (const f of files) {
        const safe = sanitizeFilename(f.name);
        let extracted: string | null = null;
        const cached = extractionCache.get(f.name);
        if (cached) {
          try {
            extracted = await cached;
          } catch {
            extracted = null;
          }
        }

        const status = extractionStatuses[f.name];
        if (status === "error" || extracted === null || extracted === undefined) {
          extractedParts.push(
            `\n\n---\n📎 ${safe} (${f.contentType}, unsupported format — content not shown)`,
          );
        } else {
          const truncated =
            extracted.length > 50_000
              ? extracted.slice(0, 50_000) + `\n...[truncated, ${extracted.length} chars total]`
              : extracted;
          extractedParts.push(
            `\n\n---\n📎 ${safe} (${f.contentType})\n\n\`\`\`\n${truncated}\n\`\`\``,
          );
        }
      }

      if (stillExtracting) extractingFiles = false;

      // Update visible message with final file labels
      const fileLabels =
        "\n\n" +
        files
          .map((f) => {
            const safe = sanitizeFilename(f.name);
            const part = extractedParts.find((p) => p.includes(safe));
            const failed = part && (part.includes("unsupported") || part.includes("failed"));
            return failed ? `📎 ${safe} (not extractable)` : `📎 ${safe}`;
          })
          .join("\n");
      await updateMessageContentStore(userMessage.id, content + fileLabels);
    }

    // Build API message array — include all user + non-empty assistant messages.
    // For the current user message, append extracted file content for the API only.
    let apiMessages: ChatMessage[] = store.messages
      .filter((m) => m.role === "user" || (m.role === "assistant" && m.content))
      .map((m) => {
        if (m.id === userMessage.id && hasFiles) {
          return { role: m.role, content: content + extractedParts.join("") };
        }
        return { role: m.role, content: m.content };
      });

    // Clear extraction cache to free memory (extracted text can be several MB per file).
    if (hasFiles) {
      for (const f of files) {
        extractionCache.delete(f.name);
        delete extractionStatuses[f.name];
      }
      extractionStatuses = { ...extractionStatuses };
    }

    // Build skill instructions for per-message skill overrides
    let skillInstructions = "";
    if (overrides?.skillIds && overrides.skillIds.length > 0) {
      const enabledSkills = skillStore.skills.filter((s) => overrides.skillIds.includes(s.id));
      if (enabledSkills.length > 0) {
        skillInstructions = enabledSkills
          .map((s) => `[Skill: ${s.name}]\n${s.instructions}`)
          .join("\n\n");
      }
    }

    // If skill overrides present, prepend instructions to the API messages
    if (skillInstructions && apiMessages.length > 0) {
      apiMessages = [{ role: "system", content: skillInstructions }, ...apiMessages];
    }

    try {
      await sendMessage(
        convId,
        apiMessages,
        overrides?.modelId ?? selectedModel,
        overrides?.agentId ?? agentStore.selectedAgentId,
        store.activeConversation?.projectId,
      );
    } catch (e) {
      // Clean up streaming state on send failure (events won't fire)
      cancelStreamingState(convId);

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
      if (store.activeConversationId) {
        await stopStreaming(store.activeConversationId);
      }
    } catch {
      // Ignore
    }
  }

  function handleModelChange(model: string) {
    selectedModel = model;
  }

  /** Enabled skills for command popup (only show skills that are toggled on). */
  const enabledSkills = $derived(skillStore.skills.filter((s) => s.enabled));

  /** Handle slash command dispatched from InputArea. */
  async function handleCommand(name: string, arg?: string) {
    const convId = store.activeConversationId;
    switch (name) {
      case "clear":
        if (convId && store.messages.length > 0) {
          // Delete all messages (re-use deleteMessagesAfter with sortOrder -1)
          await deleteMessagesAfter(convId, -1);
        }
        break;
      case "favorite":
        if (convId) await toggleFavourite(convId);
        break;
      case "title":
        if (convId && arg) await renameConversation(convId, arg);
        break;
      case "export": {
        if (!convId) break;
        const fmt = arg === "json" ? "json" : "md";
        const data =
          fmt === "json"
            ? await exportConversationJson(convId)
            : await exportConversationMarkdown(convId);
        const title = store.activeConversation?.title ?? "conversation";
        const fileName = `${title.replace(/[^a-zA-Z0-9-_ ]/g, "").slice(0, 50)}.${fmt === "json" ? "json" : "md"}`;
        await saveExportFile(fileName, data);
        break;
      }
      case "web":
        if (arg) {
          try {
            const _results: SearchResult[] = await webSearch(arg);
            // Web results are handled as URL attachments in the InputArea flow — delegate there
            // For now, this just triggers the search. Full integration with the InputArea URL pills
            // would require refactoring the URL attachment flow, so we log and let the user know.
            console.log(`Web search for "${arg}" returned ${_results.length} results`);
          } catch (e) {
            console.error("Web search failed:", e);
          }
        }
        break;
      case "help": {
        // Insert a local-only help message (not sent to API)
        const helpLines = SLASH_COMMANDS.map((c) => `\`/${c.name}\` — ${c.description}`);
        const helpContent = `**Available Commands**\n\n${helpLines.join("\n")}\n\n**@ Mentions**\n\nType \`@\` followed by an agent name to set a per-message agent override.`;
        if (convId) {
          const helpMsg: Message = {
            id: crypto.randomUUID(),
            conversationId: convId,
            role: "assistant",
            content: helpContent,
            createdAt: new Date().toISOString(),
            sortOrder: store.messages.length,
          };
          await addMessage(helpMsg);
        } else {
          // No conversation yet — show a modal popup instead
          showHelpModal = true;
        }
        break;
      }
    }
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
    const convId = store.activeConversationId;

    // Defensively stop any in-flight stream before starting a new one
    try {
      await stopStreaming(convId);
    } catch {
      // ignore — may not be streaming
    }

    const msgs = store.messages;
    const lastAssistant = [...msgs].reverse().find((m) => m.role === "assistant");
    if (!lastAssistant) return;

    // Delete the last assistant message (and anything after it)
    await deleteMessagesAfter(convId, lastAssistant.sortOrder - 1);

    // Re-create a placeholder assistant message for streaming
    const assistantMessage: Message = {
      id: crypto.randomUUID(),
      conversationId: convId,
      role: "assistant",
      content: "",
      createdAt: new Date().toISOString(),
      sortOrder: store.messages.length,
    };
    await addMessage(assistantMessage);
    startStreaming(convId, assistantMessage.id);

    requestAnimationFrame(() => {
      chatContainer?.scrollTo({ top: chatContainer.scrollHeight, behavior: "smooth" });
    });

    // Build API messages from what remains
    const apiMessages: ChatMessage[] = store.messages
      .filter((m) => m.role === "user" || (m.role === "assistant" && m.content))
      .map((m) => ({ role: m.role, content: m.content }));

    try {
      await sendMessage(
        convId,
        apiMessages,
        selectedModel,
        agentStore.selectedAgentId,
        store.activeConversation?.projectId,
      );
    } catch (e) {
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

  {#if !network.isOnline}
    <div class="offline-banner" role="alert">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
        <path
          d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 10.5a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5zM8.75 8a.75.75 0 0 1-1.5 0V5a.75.75 0 0 1 1.5 0v3z"
        />
      </svg>
      <span>You're offline. Conversations are read-only.</span>
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
          defaultAgentId={settings.defaultAgentId}
          onAgentChange={selectAgent}
          externalFiles={pendingDropFiles}
          onExternalFilesConsumed={clearPendingDropFiles}
          {extractionStatuses}
          skills={enabledSkills}
          onCommand={handleCommand}
          hasConversation={false}
        />
      </div>
    </div>
  {:else}
    {#if showSearch}
      <SearchOverlay {chatContainer} onClose={() => (showSearch = false)} />
    {/if}
    <div
      class="chat-messages"
      bind:this={chatContainer}
      role="log"
      aria-label="Chat messages"
      onscroll={handleChatScroll}
    >
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
    <div class="visually-hidden" aria-live="polite" aria-atomic="true">
      {#if streaming}Copilot is responding…{/if}
    </div>
    <div class="chat-input-float">
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
          defaultAgentId={settings.defaultAgentId}
          onAgentChange={selectAgent}
          externalFiles={pendingDropFiles}
          onExternalFilesConsumed={clearPendingDropFiles}
          {extractionStatuses}
          skills={enabledSkills}
          onCommand={handleCommand}
          hasConversation={true}
        />
      </div>
    </div>
  {/if}
</div>

{#if showHelpModal}
  <div
    class="help-overlay"
    role="dialog"
    aria-modal="true"
    aria-label="Command reference"
    tabindex="-1"
    onkeydown={(e) => {
      if (e.key === "Escape") showHelpModal = false;
    }}
    onclick={(e) => {
      if (e.target === e.currentTarget) showHelpModal = false;
    }}
  >
    <div class="help-modal">
      <div class="help-header">
        <h3>Command Reference</h3>
        <button class="help-close" onclick={() => (showHelpModal = false)} aria-label="Close"
          >×</button
        >
      </div>
      <div class="help-body">
        <div class="help-section">
          <h4>Slash Commands</h4>
          <dl class="help-commands">
            {#each SLASH_COMMANDS as cmd (cmd.name)}
              <div class="help-cmd-row">
                <dt><code>/{cmd.name}</code></dt>
                <dd>{cmd.description}</dd>
              </div>
            {/each}
          </dl>
        </div>
        <div class="help-section">
          <h4>@ Mentions</h4>
          <p>Type <code>@</code> followed by an agent name to set a per-message agent override.</p>
        </div>
      </div>
    </div>
  </div>
{/if}

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
    background: color-mix(in srgb, var(--color-bg-primary) 85%, transparent);
    backdrop-filter: blur(4px);
    animation: fadeIn 150ms ease both;
  }

  .drop-overlay-inner {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-lg);
    margin: var(--spacing-xl);
    flex: 1;
    align-self: stretch;
    border: 2px dashed var(--color-accent-copper);
    border-radius: var(--radius-xl);
    background: color-mix(in srgb, var(--color-accent-copper) 6%, transparent);
  }

  .drop-icon {
    font-size: 3.5rem;
  }

  .drop-text {
    font-family: var(--font-body);
    font-size: var(--font-size-xl);
    font-weight: 500;
    color: var(--color-accent-copper);
    letter-spacing: var(--letter-spacing-tight);
  }

  /* ── Offline banner ── */

  .offline-banner {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border-secondary);
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
    flex-shrink: 0;
    animation: fadeIn 200ms ease both;
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
    padding: var(--spacing-2xl) 0 0;
  }

  .messages-inner {
    max-width: var(--content-max-width);
    margin: 0 auto;
    padding: 0 var(--spacing-xl);
    display: flex;
    flex-direction: column;
    min-height: 100%;
    justify-content: flex-end;
    padding-bottom: 11rem;
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

  /* ── Bottom input (floating) ── */

  .chat-input-float {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 2;
    padding-top: 3rem;
    background: linear-gradient(to bottom, transparent, var(--color-bg-primary) 3rem);
    pointer-events: none;
  }

  .chat-input-container {
    max-width: var(--content-max-width);
    margin: 0 auto;
    padding: 0 var(--spacing-xl) var(--spacing-xl);
    pointer-events: auto;
  }

  /* ── Help modal ── */

  .help-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.45);
    animation: fadeIn 120ms ease;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .help-modal {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border-primary);
    border-radius: var(--radius-lg);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
    width: 420px;
    max-width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slideUp 180ms ease;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .help-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--color-border-secondary);
  }

  .help-header h3 {
    margin: 0;
    font-size: var(--font-size-base);
    font-weight: 600;
    font-family: var(--font-sans);
    color: var(--color-text-primary);
  }

  .help-close {
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    color: var(--color-text-tertiary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    line-height: 1;
  }

  .help-close:hover {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .help-body {
    padding: var(--spacing-lg);
    overflow-y: auto;
  }

  .help-section + .help-section {
    margin-top: var(--spacing-lg);
    padding-top: var(--spacing-lg);
    border-top: 1px solid var(--color-border-secondary);
  }

  .help-section h4 {
    margin: 0 0 var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-weight: 600;
    font-family: var(--font-sans);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .help-section p {
    margin: 0;
    font-size: var(--font-size-sm);
    font-family: var(--font-sans);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .help-commands {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .help-cmd-row {
    display: flex;
    align-items: baseline;
    gap: var(--spacing-sm);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
  }

  .help-cmd-row dt {
    flex-shrink: 0;
    min-width: 90px;
  }

  .help-cmd-row dt code {
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    color: var(--color-accent-primary);
    background: var(--color-bg-secondary);
    padding: 1px 5px;
    border-radius: var(--radius-sm);
  }

  .help-cmd-row dd {
    margin: 0;
    color: var(--color-text-secondary);
  }
</style>
