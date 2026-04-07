/** Typed wrappers around Tauri listen() for backend events. */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function onStreamingToken(callback: (token: string) => void): Promise<UnlistenFn> {
  return listen<string>("streaming-token", (event) => {
    callback(event.payload);
  });
}

export function onStreamingComplete(callback: () => void): Promise<UnlistenFn> {
  return listen("streaming-complete", () => {
    callback();
  });
}

export function onStreamingError(callback: (error: string) => void): Promise<UnlistenFn> {
  return listen<string>("streaming-error", (event) => {
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
