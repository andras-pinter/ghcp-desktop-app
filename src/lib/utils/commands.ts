/** Typed wrappers around Tauri invoke() for all backend commands. */

import { invoke } from "@tauri-apps/api/core";
import type { AuthState, DeviceCodeResponse, GitHubUser } from "$lib/types/auth";
import type { ChatMessage, Model } from "$lib/types/message";

// ── Auth ────────────────────────────────────────────────────────

export async function getAuthState(): Promise<AuthState> {
  return invoke<AuthState>("get_auth_state");
}

export async function authenticate(): Promise<DeviceCodeResponse> {
  return invoke<DeviceCodeResponse>("authenticate");
}

export async function pollAuthToken(deviceCode: string): Promise<GitHubUser> {
  return invoke<GitHubUser>("poll_auth_token", { deviceCode });
}

export async function doLogout(): Promise<void> {
  return invoke("logout");
}

// ── Chat ────────────────────────────────────────────────────────

export async function sendMessage(messages: ChatMessage[], model: string): Promise<void> {
  return invoke("send_message", { messages, model });
}

export async function stopStreaming(): Promise<void> {
  return invoke("stop_streaming");
}

// ── Models ──────────────────────────────────────────────────────

export async function getModels(): Promise<Model[]> {
  return invoke<Model[]>("get_models");
}
