/** Reactive conversation + message state using Svelte 5 runes. */

/* eslint-disable svelte/prefer-svelte-reactivity -- Date is used imperatively for ISO timestamps, not reactively */

import { SvelteSet, SvelteMap } from "svelte/reactivity";
import type { Conversation } from "$lib/types/conversation";
import type { Message } from "$lib/types/message";
import {
  getConversations,
  createConversation as createConvCmd,
  updateConversation as updateConvCmd,
  deleteConversation as deleteConvCmd,
  getMessages,
  createMessage as createMsgCmd,
  updateMessageContent as updateMsgCmd,
  deleteMessagesAfter as deleteMsgsAfterCmd,
  getDraft,
  saveDraft as saveDraftCmd,
  deleteDraft as deleteDraftCmd,
  stopStreaming,
  generateConversationTitle,
  updateConversation,
  logFrontend,
} from "$lib/utils/commands";
import { getSettings } from "$lib/stores/settings.svelte";
import { selectAgent as selectAgentFn } from "$lib/stores/agents.svelte";
import {
  onStreamingToken,
  onStreamingComplete,
  onStreamingError,
  type StreamingTokenPayload,
  type StreamingCompletePayload,
  type StreamingErrorPayload,
} from "$lib/utils/events";
import type { UnlistenFn } from "@tauri-apps/api/event";

let conversations = $state<Conversation[]>([]);
let activeConversationId = $state<string | null>(null);
let messages = $state<Message[]>([]);
let loadingConversations = $state(false);
let loadingMessages = $state(false);

// ── Per-conversation streaming state (global, not per-component) ──

/** Conversation IDs currently streaming a response. */
const streamingConversations = new SvelteSet<string>();

/** Conversation IDs with unread assistant responses (streaming finished, user hasn't viewed). */
const unreadConversations = new SvelteSet<string>();

/** Buffered streaming content per conversation (for background streams). */
interface StreamingBuffer {
  messageId: string;
  content: string;
  thinkingContent: string;
}
const streamingBuffers = new SvelteMap<string, StreamingBuffer>();

/** Track which conversation is being title-generated to avoid duplicates. */
const titleGeneratingFor = new Set<string>();

/** Global event listener unlisten handles (registered once at store init). */
let unlistenToken: UnlistenFn | undefined;
let unlistenComplete: UnlistenFn | undefined;
let unlistenError: UnlistenFn | undefined;

// ── RAF-batched token updates ───────────────────────────────────
// Tokens arrive per-SSE-event (potentially 100s/sec). We accumulate them
// in the buffer object (cheap string concat) and batch Svelte reactivity
// updates into a single requestAnimationFrame callback (~60/sec max).

let tokenRafId: number | null = null;
const dirtyConversations = new Set<string>();

function flushTokenUpdates(): void {
  tokenRafId = null;

  for (const convId of dirtyConversations) {
    const buffer = streamingBuffers.get(convId);
    if (!buffer) continue;

    // Trigger SvelteMap reactivity once per dirty conversation per frame
    streamingBuffers.set(convId, buffer);

    // Update the active conversation's messages array
    if (convId === activeConversationId) {
      const msg = messages.find((m) => m.id === buffer.messageId);
      if (msg && msg.role === "assistant") {
        msg.content = buffer.content;
      }
    }
  }

  // Trigger Svelte array reactivity once (not per-token)
  if (dirtyConversations.has(activeConversationId ?? "")) {
    // eslint-disable-next-line no-self-assign -- trigger Svelte 5 reactivity on mutation
    messages = messages;
  }

  dirtyConversations.clear();
}

// ── Global event listeners ──────────────────────────────────────

/** Initialize global streaming event listeners. Call once after app startup. */
export async function initStreamingListeners(): Promise<void> {
  unlistenToken = await onStreamingToken(handleStreamingToken);
  unlistenComplete = await onStreamingComplete(handleStreamingComplete);
  unlistenError = await onStreamingError(handleStreamingError);
}

/** Tear down global streaming event listeners. */
export function destroyStreamingListeners(): void {
  unlistenToken?.();
  unlistenComplete?.();
  unlistenError?.();
  if (tokenRafId !== null) {
    cancelAnimationFrame(tokenRafId);
    tokenRafId = null;
  }
}

function handleStreamingToken(payload: StreamingTokenPayload): void {
  const { conversationId, token } = payload;
  const buffer = streamingBuffers.get(conversationId);
  if (!buffer) return;

  // Accumulate immediately (cheap string concat, no reactivity)
  buffer.content += token;

  // Mark dirty and schedule a batched UI flush
  dirtyConversations.add(conversationId);
  if (tokenRafId === null) {
    tokenRafId = requestAnimationFrame(flushTokenUpdates);
  }
}

async function handleStreamingComplete(payload: StreamingCompletePayload): Promise<void> {
  const { conversationId } = payload;
  const buffer = streamingBuffers.get(conversationId);
  streamingConversations.delete(conversationId);
  streamingBuffers.delete(conversationId);

  if (buffer && buffer.content) {
    // Persist the completed assistant message to DB
    await updateMsgCmd(buffer.messageId, buffer.content, buffer.thinkingContent || null);

    // If this is the active conversation, sync the messages array
    if (conversationId === activeConversationId) {
      messages = messages.map((m) =>
        m.id === buffer.messageId ? { ...m, content: buffer.content } : m,
      );
    }

    // Mark as unread if user is NOT viewing this conversation
    if (conversationId !== activeConversationId) {
      unreadConversations.add(conversationId);
    }

    // Retry title generation if still untitled (initial attempt at stream start may have failed)
    autoGenerateTitle(conversationId);
  }
}

async function handleStreamingError(payload: StreamingErrorPayload): Promise<void> {
  const { conversationId, error } = payload;
  const buffer = streamingBuffers.get(conversationId);
  streamingConversations.delete(conversationId);
  streamingBuffers.delete(conversationId);

  if (buffer) {
    const errContent = `⚠️ Error: ${error}`;
    await updateMsgCmd(buffer.messageId, errContent);

    if (conversationId === activeConversationId) {
      messages = messages.map((m) =>
        m.id === buffer.messageId ? { ...m, content: errContent } : m,
      );
    }

    if (conversationId !== activeConversationId) {
      unreadConversations.add(conversationId);
    }
  }
}

// ── Title generation (moved from ChatView to support background streams) ──

async function autoGenerateTitle(conversationId: string): Promise<void> {
  const conv = conversations.find((c) => c.id === conversationId);
  if (conv?.title || titleGeneratingFor.has(conversationId)) return;

  titleGeneratingFor.add(conversationId);
  try {
    // Load messages for this conversation if needed (may not be active)
    let msgs: Message[];
    if (conversationId === activeConversationId) {
      msgs = messages;
    } else {
      msgs = await getMessages(conversationId);
    }

    const firstUser = msgs.find((m) => m.role === "user");
    const firstAssistant = msgs.find((m) => m.role === "assistant");
    if (!firstUser) return;

    const model = conv?.model || "gpt-4o";
    const hasAssistantContent = !!(firstAssistant?.content && firstAssistant.content.trim());

    let title: string | undefined;
    try {
      title = await generateConversationTitle(
        firstUser.content,
        firstAssistant?.content ?? "",
        model,
      );
    } catch {
      // Only fall back to first-line title if the assistant has already responded
      // (i.e. this is the retry at stream-complete). On the first attempt at stream
      // start, silently fail so the retry gets a chance with richer context.
      if (hasAssistantContent) {
        const cleaned = firstUser.content
          .replace(/\n+---\n📎\s*\[.*$/s, "")
          .replace(/\n+📎\s*.*/g, "")
          .trim();
        if (cleaned) {
          title = cleaned.length > 50 ? cleaned.slice(0, 49) + "…" : cleaned;
        }
      }
    }

    if (title) {
      try {
        await updateConversation(conversationId, title);
        setConversationTitle(conversationId, title);
      } catch (e) {
        logFrontend("warn", `Failed to set conversation title: ${e}`);
      }
    }
  } finally {
    titleGeneratingFor.delete(conversationId);
  }
}

// ── Streaming lifecycle ─────────────────────────────────────────

/** Register a conversation as actively streaming with a placeholder assistant message. */
export function startStreaming(conversationId: string, assistantMessageId: string): void {
  streamingConversations.add(conversationId);
  streamingBuffers.set(conversationId, {
    messageId: assistantMessageId,
    content: "",
    thinkingContent: "",
  });

  // Generate title immediately — don't wait for streaming to finish
  autoGenerateTitle(conversationId);
}

/** Check if a specific conversation is currently streaming. */
export function isStreaming(conversationId: string): boolean {
  return streamingConversations.has(conversationId);
}

/** Check if a specific conversation has unread responses. */
export function hasUnread(conversationId: string): boolean {
  return unreadConversations.has(conversationId);
}

/** Mark a conversation as read (clear unread indicator). */
export function markAsRead(conversationId: string): void {
  unreadConversations.delete(conversationId);
}

/** Get the streaming assistant message ID for a conversation (if streaming). */
export function getStreamingMessageId(conversationId: string): string | null {
  return streamingBuffers.get(conversationId)?.messageId ?? null;
}

/** Get buffered content for a conversation (used when switching to a streaming conversation). */
export function getStreamingBuffer(conversationId: string): StreamingBuffer | undefined {
  return streamingBuffers.get(conversationId);
}

/** Clean up streaming state for a conversation (e.g., on send failure before events fire). */
export function cancelStreamingState(conversationId: string): void {
  streamingConversations.delete(conversationId);
  streamingBuffers.delete(conversationId);
}

// ── Initialization ──────────────────────────────────────────────

/** Load the conversation list from the backend. Call once on app startup. */
export async function initConversations(): Promise<void> {
  loadingConversations = true;
  try {
    conversations = await getConversations();
  } catch (e) {
    console.error("Failed to load conversations:", e);
    conversations = [];
  } finally {
    loadingConversations = false;
  }
}

// ── Conversation CRUD ───────────────────────────────────────────

/** Create a new conversation and make it active. Auto-selects the default agent. */
export async function newConversation(model: string): Promise<Conversation> {
  const id = crypto.randomUUID();
  const conv = await createConvCmd(id, null, null, null, model);
  conversations = [conv, ...conversations];
  await switchConversation(conv.id);

  // Auto-select the default agent from settings
  const settings = getSettings();
  if (settings.defaultAgentId) {
    selectAgentFn(settings.defaultAgentId);
  }

  return conv;
}

/** Switch to a conversation — loads its messages from DB. */
export async function switchConversation(id: string): Promise<void> {
  if (activeConversationId === id) return;
  activeConversationId = id;

  // Mark as read when user switches to this conversation
  markAsRead(id);

  loadingMessages = true;
  try {
    messages = await getMessages(id);

    // If this conversation is actively streaming, sync from the buffer
    const buffer = streamingBuffers.get(id);
    if (buffer) {
      const lastMsg = messages.find((m) => m.id === buffer.messageId);
      if (lastMsg) {
        lastMsg.content = buffer.content;
        // eslint-disable-next-line no-self-assign -- trigger Svelte 5 reactivity
        messages = messages;
      }
    }
  } catch (e) {
    console.error("Failed to load messages:", e);
    messages = [];
  } finally {
    loadingMessages = false;
  }
}

/** Clear the active conversation (go to welcome screen). */
export function clearActiveConversation(): void {
  activeConversationId = null;
  messages = [];
}

/** Rename a conversation. */
export async function renameConversation(id: string, title: string): Promise<void> {
  await updateConvCmd(id, title);
  conversations = conversations.map((c) =>
    c.id === id ? { ...c, title, updatedAt: new Date().toISOString() } : c,
  );
}

/** Toggle a conversation's favourite status. */
export async function toggleFavourite(id: string): Promise<void> {
  const conv = conversations.find((c) => c.id === id);
  if (!conv) return;
  const newFav = !conv.isFavourite;
  await updateConvCmd(id, null, newFav);
  conversations = conversations.map((c) =>
    c.id === id ? { ...c, isFavourite: newFav, updatedAt: new Date().toISOString() } : c,
  );
}

/** Delete a conversation. If it was active, clear the view. Cancels streaming if active. */
export async function removeConversation(id: string): Promise<void> {
  // Cancel streaming if this conversation is actively streaming
  if (streamingConversations.has(id)) {
    try {
      await stopStreaming(id);
    } catch {
      // ignore
    }
    streamingConversations.delete(id);
    streamingBuffers.delete(id);
  }
  unreadConversations.delete(id);

  await deleteConvCmd(id);
  conversations = conversations.filter((c) => c.id !== id);
  if (activeConversationId === id) {
    clearActiveConversation();
  }
}

/** Update a conversation's title in the local list (e.g., after auto-generation). */
export function setConversationTitle(id: string, title: string): void {
  conversations = conversations.map((c) => (c.id === id ? { ...c, title } : c));
}

/** Bump a conversation to the top of the list (after new message). */
export function touchConversation(id: string): void {
  const now = new Date().toISOString();
  conversations = conversations.map((c) => (c.id === id ? { ...c, updatedAt: now } : c));
  // Re-sort: favourites first, then by updatedAt desc
  conversations = [...conversations].sort((a, b) => {
    if (a.isFavourite !== b.isFavourite) return a.isFavourite ? -1 : 1;
    return b.updatedAt.localeCompare(a.updatedAt);
  });
}

// ── Message operations ──────────────────────────────────────────

/** Add a message to the current conversation (in-memory + DB). */
export async function addMessage(message: Message): Promise<void> {
  messages = [...messages, message];
  await createMsgCmd(message);
}

/** Update the content of a message (in-memory + DB). */
export async function updateMessageContent(
  id: string,
  content: string,
  thinkingContent?: string | null,
): Promise<void> {
  messages = messages.map((m) =>
    m.id === id ? { ...m, content, thinkingContent: thinkingContent ?? m.thinkingContent } : m,
  );
  await updateMsgCmd(id, content, thinkingContent);
}

/** Append a token to the last assistant message (streaming). In-memory only.
 *  @deprecated Use the store-level streaming listeners instead. Kept for backward compat. */
export function appendStreamingToken(token: string): void {
  const last = messages[messages.length - 1];
  if (last && last.role === "assistant") {
    last.content += token;
    // eslint-disable-next-line no-self-assign -- trigger Svelte 5 reactivity on mutation
    messages = messages;
  }
}

/** Delete all messages after a sort order + remove from local state. */
export async function deleteMessagesAfter(
  conversationId: string,
  afterSortOrder: number,
): Promise<void> {
  await deleteMsgsAfterCmd(conversationId, afterSortOrder);
  messages = messages.filter((m) => m.sortOrder <= afterSortOrder);
}

// ── Drafts ──────────────────────────────────────────────────────

/** Save a draft for the current conversation. */
export async function saveDraft(conversationId: string, content: string): Promise<void> {
  if (!content.trim()) {
    await deleteDraftCmd(conversationId);
    return;
  }
  await saveDraftCmd(conversationId, content);
}

/** Load the draft for a conversation, if any. */
export async function loadDraft(conversationId: string): Promise<string> {
  try {
    const draft = await getDraft(conversationId);
    return draft?.content ?? "";
  } catch {
    return "";
  }
}

/** Delete the draft for a conversation. */
export async function clearDraft(conversationId: string): Promise<void> {
  await deleteDraftCmd(conversationId);
}

// ── Reactive getters ────────────────────────────────────────────

export function getConversationStore() {
  return {
    get conversations() {
      return conversations;
    },
    get activeConversationId() {
      return activeConversationId;
    },
    get activeConversation(): Conversation | null {
      if (!activeConversationId) return null;
      return conversations.find((c) => c.id === activeConversationId) ?? null;
    },
    get messages() {
      return messages;
    },
    get loadingConversations() {
      return loadingConversations;
    },
    get loadingMessages() {
      return loadingMessages;
    },
    get hasConversations() {
      return conversations.length > 0;
    },
    /** Set of conversation IDs currently streaming (reactive). */
    get streamingConversations(): SvelteSet<string> {
      return streamingConversations;
    },
    /** Set of conversation IDs with unread responses (reactive). */
    get unreadConversations(): SvelteSet<string> {
      return unreadConversations;
    },
  };
}
