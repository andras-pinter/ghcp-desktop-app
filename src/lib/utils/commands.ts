/** Typed wrappers around Tauri invoke() for all backend commands. */

import { invoke } from "@tauri-apps/api/core";

// Placeholder — commands will be added as backend implements them
export async function getAuthState(): Promise<unknown> {
  return invoke("get_auth_state");
}

export async function getConversations(): Promise<unknown> {
  return invoke("get_conversations");
}

export async function getModels(): Promise<unknown> {
  return invoke("get_models");
}
