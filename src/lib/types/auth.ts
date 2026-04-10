/** Auth-related types (mirrors Rust copilot-api types). */

/** User-facing device code info (excludes the polling secret). */
export interface DeviceCodeInfo {
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

/** Result of the authenticate command — polling key + safe info. */
export interface AuthenticateResult {
  deviceCode: string;
  info: DeviceCodeInfo;
}

/** Minimal GitHub user profile. */
export interface GitHubUser {
  login: string;
  name?: string | null;
  avatar_url?: string | null;
}

/** Serializable auth state sent from the backend. */
export interface AuthState {
  authenticated: boolean;
  user?: GitHubUser | null;
}
