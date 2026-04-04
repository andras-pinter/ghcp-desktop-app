//! Request/response types for the Copilot API.

use serde::{Deserialize, Serialize};

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

/// Request body for `/v1/chat/completions`.
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

/// Response from `/v1/chat/completions` (non-streaming).
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<ChatChoice>,
    pub model: String,
}

/// A single choice in the chat response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// A Copilot model descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
}

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
}
