/** Reactive auth state using Svelte 5 runes. */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  getAuthState,
  authenticate,
  pollAuthToken as pollAuthTokenCmd,
  doLogout,
} from "$lib/utils/commands";
import type { AuthenticateResult, GitHubUser } from "$lib/types/auth";

let authenticated = $state(false);
let user = $state<GitHubUser | null>(null);
let loading = $state(true);
let unlistenAuthChange: UnlistenFn | undefined;

/** Initialize auth state — call once on app startup. */
export async function initAuth(): Promise<void> {
  loading = true;
  try {
    const state = await getAuthState();
    authenticated = state.authenticated;
    user = state.user ?? null;
  } catch {
    authenticated = false;
    user = null;
  } finally {
    loading = false;
  }

  // Clean up previous listener (e.g., HMR reload)
  unlistenAuthChange?.();

  // Listen for auth state changes from backend
  unlistenAuthChange = await listen<boolean>("auth-state-changed", (event) => {
    authenticated = event.payload;
    if (!event.payload) {
      user = null;
    }
  });
}

/** Start the device flow — returns auth result with device code + UI info. */
export async function startDeviceFlow(): Promise<AuthenticateResult> {
  return authenticate();
}

/** Poll once for the OAuth token. Returns user on success. */
export async function pollAuthToken(deviceCode: string): Promise<GitHubUser> {
  const ghUser = await pollAuthTokenCmd(deviceCode);
  authenticated = true;
  user = ghUser;
  return ghUser;
}

/** Sign out. */
export async function logout(): Promise<void> {
  await doLogout();
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
