/** Reactive conversation + message state using Svelte 5 runes. */

/* eslint-disable svelte/prefer-svelte-reactivity -- Date is used imperatively for ISO timestamps, not reactively */

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
} from "$lib/utils/commands";

let conversations = $state<Conversation[]>([]);
let activeConversationId = $state<string | null>(null);
let messages = $state<Message[]>([]);
let loadingConversations = $state(false);
let loadingMessages = $state(false);

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

/** Create a new conversation and make it active. */
export async function newConversation(model: string): Promise<Conversation> {
  const id = crypto.randomUUID();
  const conv = await createConvCmd(id, null, null, null, model);
  conversations = [conv, ...conversations];
  await switchConversation(conv.id);
  return conv;
}

/** Switch to a conversation — loads its messages from DB. */
export async function switchConversation(id: string): Promise<void> {
  if (activeConversationId === id) return;
  activeConversationId = id;
  loadingMessages = true;
  try {
    messages = await getMessages(id);
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

/** Delete a conversation. If it was active, clear the view. */
export async function removeConversation(id: string): Promise<void> {
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

/** Append a token to the last assistant message (streaming). In-memory only. */
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
  };
}
