//! Request/response types for the Copilot API.
//!
//! Covers OAuth device flow, Copilot token exchange, chat completions
//! (both non-streaming and SSE streaming), and model discovery.

use serde::{Deserialize, Serialize};

// ── OAuth Device Flow ──────────────────────────────────────────────

/// Response from `POST https://github.com/login/device/code`.
///
/// **Not `Serialize`** — contains `device_code` (polling secret) that must
/// not be sent to the frontend. Use [`DeviceCodeInfo`] for the IPC boundary.
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCodeResponse {
    /// The device verification code (used for polling — internal only).
    pub device_code: String,
    /// The user-facing code to enter at `verification_uri`.
    pub user_code: String,
    /// The URL the user visits to enter the code.
    pub verification_uri: String,
    /// Seconds until the device code expires.
    pub expires_in: u64,
    /// Minimum seconds between poll attempts.
    pub interval: u64,
}

/// User-facing subset of [`DeviceCodeResponse`] safe to send to the frontend.
///
/// Excludes `device_code` (the secret used for token polling).
#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeInfo {
    /// The user-facing code to enter at `verification_uri`.
    pub user_code: String,
    /// The URL the user visits to enter the code.
    pub verification_uri: String,
    /// Seconds until the device code expires.
    pub expires_in: u64,
    /// Minimum seconds between poll attempts.
    pub interval: u64,
}

impl From<&DeviceCodeResponse> for DeviceCodeInfo {
    fn from(resp: &DeviceCodeResponse) -> Self {
        Self {
            user_code: resp.user_code.clone(),
            verification_uri: resp.verification_uri.clone(),
            expires_in: resp.expires_in,
            interval: resp.interval,
        }
    }
}

/// Successful response from polling `POST https://github.com/login/oauth/access_token`.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

/// Error body from the OAuth token polling endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    #[serde(default)]
    pub error_description: Option<String>,
}

// ── Copilot Token ──────────────────────────────────────────────────

/// Response from `GET https://api.github.com/copilot_internal/v2/token`.
#[derive(Debug, Clone, Deserialize)]
pub struct CopilotTokenResponse {
    /// Short-lived JWT for Copilot API calls.
    pub token: String,
    /// Unix timestamp when the token expires.
    pub expires_at: i64,
    /// Map of endpoint URLs (e.g., `api` → base URL for chat completions).
    #[serde(default)]
    pub endpoints: CopilotEndpoints,
}

/// Copilot API endpoints extracted from the token response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CopilotEndpoints {
    /// Base URL for chat completions API.
    #[serde(default)]
    pub api: Option<String>,
}

// ── GitHub User ────────────────────────────────────────────────────

/// Minimal GitHub user info from `GET https://api.github.com/user`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

// ── Chat Messages ──────────────────────────────────────────────────

/// Role of a chat message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// A single chat message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

// ── Chat Completions Request ───────────────────────────────────────

/// Request body for `/chat/completions`.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

// ── Chat Completions Response (non-streaming) ──────────────────────

/// Response from `/chat/completions` (non-streaming).
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<ChatChoice>,
    pub model: String,
}

/// A single choice in a non-streaming response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

// ── SSE Streaming Response ─────────────────────────────────────────

/// A single SSE chunk from the streaming `/chat/completions` endpoint.
///
/// Each `data:` line (except `data: [DONE]`) deserializes to this.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamingChatResponse {
    pub id: String,
    pub choices: Vec<StreamingChoice>,
    #[serde(default)]
    pub model: Option<String>,
}

/// A single choice in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamingChoice {
    pub index: u32,
    pub delta: StreamingDelta,
    pub finish_reason: Option<String>,
}

/// The delta (partial content) in a streaming choice.
///
/// Only the fields present in a given chunk are `Some`.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamingDelta {
    /// The role — typically only present in the first chunk.
    #[serde(default)]
    pub role: Option<MessageRole>,
    /// Partial content token.
    #[serde(default)]
    pub content: Option<String>,
}

// ── Model Discovery ────────────────────────────────────────────────

/// A Copilot model descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

impl Model {
    /// Display name for UI — falls back to the model id.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.id)
    }
}

/// Response from the models endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

// ── Auth State (shared with frontend via Tauri) ────────────────────

/// Serializable auth state sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<GitHubUser>,
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""user""#);

        let deserialized: MessageRole = serde_json::from_str(r#""assistant""#).unwrap();
        assert_eq!(deserialized, MessageRole::Assistant);
    }

    #[test]
    fn test_chat_request_serialization() {
        let req = ChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                name: None,
                tool_call_id: None,
            }],
            temperature: None,
            max_tokens: None,
            stream: true,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["stream"], true);
        assert!(json.get("temperature").is_none());
    }

    #[test]
    fn test_device_code_response_deserialize() {
        let json = r#"{
            "device_code": "abc123",
            "user_code": "ABCD-1234",
            "verification_uri": "https://github.com/login/device",
            "expires_in": 899,
            "interval": 5
        }"#;
        let resp: DeviceCodeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.user_code, "ABCD-1234");
        assert_eq!(resp.interval, 5);
    }

    #[test]
    fn test_oauth_token_response_deserialize() {
        let json = r#"{
            "access_token": "gho_xxxxxxxxxxxx",
            "token_type": "bearer",
            "scope": "copilot"
        }"#;
        let resp: OAuthTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "gho_xxxxxxxxxxxx");
    }

    #[test]
    fn test_copilot_token_response_deserialize() {
        let json = r#"{
            "token": "tid=abc;exp=123;sku=copilot_pro",
            "expires_at": 1700000000,
            "endpoints": {
                "api": "https://api.individual.githubcopilot.com"
            }
        }"#;
        let resp: CopilotTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.expires_at, 1700000000);
        assert_eq!(
            resp.endpoints.api.as_deref(),
            Some("https://api.individual.githubcopilot.com")
        );
    }

    #[test]
    fn test_copilot_token_missing_endpoints() {
        let json = r#"{"token": "abc", "expires_at": 123}"#;
        let resp: CopilotTokenResponse = serde_json::from_str(json).unwrap();
        assert!(resp.endpoints.api.is_none());
    }

    #[test]
    fn test_streaming_response_deserialize() {
        let json = r#"{
            "id": "chatcmpl-abc",
            "choices": [{
                "index": 0,
                "delta": {"content": "Hello"},
                "finish_reason": null
            }],
            "model": "gpt-4o"
        }"#;
        let resp: StreamingChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices[0].delta.content.as_deref(), Some("Hello"));
        assert!(resp.choices[0].finish_reason.is_none());
    }

    #[test]
    fn test_streaming_first_chunk_with_role() {
        let json = r#"{
            "id": "chatcmpl-abc",
            "choices": [{
                "index": 0,
                "delta": {"role": "assistant"},
                "finish_reason": null
            }]
        }"#;
        let resp: StreamingChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices[0].delta.role, Some(MessageRole::Assistant));
        assert!(resp.choices[0].delta.content.is_none());
    }

    #[test]
    fn test_streaming_done_chunk() {
        let json = r#"{
            "id": "chatcmpl-abc",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }]
        }"#;
        let resp: StreamingChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices[0].finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn test_models_response_deserialize() {
        let json = r#"{
            "data": [
                {"id": "gpt-4o", "name": "GPT-4o"},
                {"id": "gpt-4o-mini"}
            ]
        }"#;
        let resp: ModelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].display_name(), "GPT-4o");
        assert_eq!(resp.data[1].display_name(), "gpt-4o-mini");
    }

    #[test]
    fn test_auth_state_serialization() {
        let state = AuthState {
            authenticated: true,
            user: Some(GitHubUser {
                login: "octocat".to_string(),
                name: Some("Octocat".to_string()),
                avatar_url: None,
            }),
        };
        let json = serde_json::to_value(&state).unwrap();
        assert_eq!(json["authenticated"], true);
        assert_eq!(json["user"]["login"], "octocat");
    }
}
