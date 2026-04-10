/** Typed wrappers around Tauri listen() for backend events. */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Streaming event payloads (include conversationId for routing) ──

/** Payload emitted per token during SSE streaming. */
export interface StreamingTokenPayload {
  conversationId: string;
  token: string;
}

/** Payload emitted when streaming finishes (success or cancellation). */
export interface StreamingCompletePayload {
  conversationId: string;
}

/** Payload emitted when streaming encounters an error. */
export interface StreamingErrorPayload {
  conversationId: string;
  error: string;
}

/** Listen for individual streaming tokens from the backend. */
export function onStreamingToken(
  callback: (payload: StreamingTokenPayload) => void,
): Promise<UnlistenFn> {
  return listen<StreamingTokenPayload>("streaming-token", (event) => {
    callback(event.payload);
  });
}

/** Listen for the streaming-complete signal (response finished). */
export function onStreamingComplete(
  callback: (payload: StreamingCompletePayload) => void,
): Promise<UnlistenFn> {
  return listen<StreamingCompletePayload>("streaming-complete", (event) => {
    callback(event.payload);
  });
}

/** Listen for streaming errors from the backend. */
export function onStreamingError(
  callback: (payload: StreamingErrorPayload) => void,
): Promise<UnlistenFn> {
  return listen<StreamingErrorPayload>("streaming-error", (event) => {
    callback(event.payload);
  });
}

/** Listen for auth state changes pushed from the backend. */
export function onAuthStateChanged(
  callback: (authenticated: boolean) => void,
): Promise<UnlistenFn> {
  return listen<boolean>("auth-state-changed", (event) => {
    callback(event.payload);
  });
}

/** Payload emitted when older messages are summarized to manage context. */
export interface ContextSummarizedPayload {
  count: number;
}

/** Listen for context summarization events. */
export function onContextSummarized(
  callback: (payload: ContextSummarizedPayload) => void,
): Promise<UnlistenFn> {
  return listen<ContextSummarizedPayload>("context-summarized", (event) => {
    callback(event.payload);
  });
}
