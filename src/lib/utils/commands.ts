/** Typed wrappers around Tauri invoke() for all backend commands. */

import { invoke } from "@tauri-apps/api/core";
import type { AuthState, DeviceCodeResponse, GitHubUser } from "$lib/types/auth";
import type { ChatMessage, Model } from "$lib/types/message";

// ── Auth ────────────────────────────────────────────────────────

/** Check current authentication state. */
export async function getAuthState(): Promise<AuthState> {
  return invoke<AuthState>("get_auth_state");
}

/** Start the GitHub OAuth device flow. */
export async function authenticate(): Promise<DeviceCodeResponse> {
  return invoke<DeviceCodeResponse>("authenticate");
}

/** Poll once for the OAuth token after user authorizes. */
export async function pollAuthToken(deviceCode: string): Promise<GitHubUser> {
  return invoke<GitHubUser>("poll_auth_token", { deviceCode });
}

/** Sign out and clear stored tokens. */
export async function doLogout(): Promise<void> {
  return invoke("logout");
}

// ── Chat ────────────────────────────────────────────────────────

/** Send chat messages and start streaming the response. */
export async function sendMessage(messages: ChatMessage[], model: string): Promise<void> {
  return invoke("send_message", { messages, model });
}

/** Cancel an in-flight streaming response. */
export async function stopStreaming(): Promise<void> {
  return invoke("stop_streaming");
}

// ── Models ──────────────────────────────────────────────────────

/** Fetch available Copilot models. */
export async function getModels(): Promise<Model[]> {
  return invoke<Model[]>("get_models");
}
