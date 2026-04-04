/** Reactive auth state using Svelte 5 runes. */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AuthState, DeviceCodeResponse, GitHubUser } from "$lib/types/auth";

let authenticated = $state(false);
let user = $state<GitHubUser | null>(null);
let loading = $state(true);

/** Initialize auth state — call once on app startup. */
export async function initAuth(): Promise<void> {
  loading = true;
  try {
    const state = await invoke<AuthState>("get_auth_state");
    authenticated = state.authenticated;
    user = state.user ?? null;
  } catch {
    authenticated = false;
    user = null;
  } finally {
    loading = false;
  }

  // Listen for auth state changes from backend
  await listen<boolean>("auth-state-changed", (event) => {
    authenticated = event.payload;
    if (!event.payload) {
      user = null;
    }
  });
}

/** Start the device flow — returns device code info for the UI. */
export async function startDeviceFlow(): Promise<DeviceCodeResponse> {
  return invoke<DeviceCodeResponse>("authenticate");
}

/** Poll once for the OAuth token. Returns user on success. */
export async function pollAuthToken(deviceCode: string): Promise<GitHubUser> {
  const ghUser = await invoke<GitHubUser>("poll_auth_token", { deviceCode });
  authenticated = true;
  user = ghUser;
  return ghUser;
}

/** Sign out. */
export async function logout(): Promise<void> {
  await invoke("logout");
  authenticated = false;
  user = null;
}

/** Reactive getters. */
export function getAuth() {
  return {
    get authenticated() {
      return authenticated;
    },
    get user() {
      return user;
    },
    get loading() {
      return loading;
    },
  };
}
