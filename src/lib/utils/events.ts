/** Typed wrappers around Tauri listen() for backend events. */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Streaming event payloads (include conversationId for routing) ──

export interface StreamingTokenPayload {
  conversationId: string;
  token: string;
}

export interface StreamingCompletePayload {
  conversationId: string;
}

export interface StreamingErrorPayload {
  conversationId: string;
  error: string;
}

export function onStreamingToken(
  callback: (payload: StreamingTokenPayload) => void,
): Promise<UnlistenFn> {
  return listen<StreamingTokenPayload>("streaming-token", (event) => {
    callback(event.payload);
  });
}

export function onStreamingComplete(
  callback: (payload: StreamingCompletePayload) => void,
): Promise<UnlistenFn> {
  return listen<StreamingCompletePayload>("streaming-complete", (event) => {
    callback(event.payload);
  });
}

export function onStreamingError(
  callback: (payload: StreamingErrorPayload) => void,
): Promise<UnlistenFn> {
  return listen<StreamingErrorPayload>("streaming-error", (event) => {
    callback(event.payload);
  });
}

export function onAuthStateChanged(
  callback: (authenticated: boolean) => void,
): Promise<UnlistenFn> {
  return listen<boolean>("auth-state-changed", (event) => {
    callback(event.payload);
  });
}

export function onNetworkStatus(callback: (online: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("network-status", (event) => {
    callback(event.payload);
  });
}

export interface ContextSummarizedPayload {
  count: number;
}

export function onContextSummarized(
  callback: (payload: ContextSummarizedPayload) => void,
): Promise<UnlistenFn> {
  return listen<ContextSummarizedPayload>("context-summarized", (event) => {
    callback(event.payload);
  });
}
