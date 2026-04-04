/** Auth-related types (mirrors Rust copilot-api types). */

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export interface GitHubUser {
  login: string;
  name?: string | null;
  avatar_url?: string | null;
}

export interface AuthState {
  authenticated: boolean;
  user?: GitHubUser | null;
}
