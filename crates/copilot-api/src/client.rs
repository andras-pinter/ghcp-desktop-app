//! HTTP client for Copilot chat completions with SSE streaming.

use crate::auth::{AuthError, DeviceFlowAuth};
use crate::types::{ChatRequest, Model, ModelsResponse, StreamingChatResponse};
use reqwest_eventsource::{Event, EventSource};
use thiserror::Error;
use tokio::sync::mpsc;

/// Application version derived from Cargo.toml at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Errors from the Copilot chat client.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),
    #[error("SSE stream error: {0}")]
    Stream(String),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Rate limited — retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("API error: {status} {body}")]
    Api { status: u16, body: String },
}

/// Build the User-Agent header value.
fn user_agent() -> String {
    format!("Chuck/{APP_VERSION} (GitHub Copilot Desktop Client)")
}

/// A streaming event emitted token-by-token during chat.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A partial content token from the assistant.
    Token(String),
    /// The assistant's role was set (first chunk).
    RoleSet,
    /// The stream completed normally.
    Done,
    /// An error occurred during streaming.
    Error(String),
}

/// Client for the GitHub Copilot chat completions and models endpoints.
pub struct CopilotClient {
    auth: DeviceFlowAuth,
}

impl CopilotClient {
    /// Create a new Copilot client that manages auth automatically.
    pub fn new() -> Self {
        Self {
            auth: DeviceFlowAuth::new(),
        }
    }

    /// Create with a custom auth handler (useful for testing).
    pub fn with_auth(auth: DeviceFlowAuth) -> Self {
        Self { auth }
    }

    /// Get a reference to the auth handler.
    pub fn auth(&self) -> &DeviceFlowAuth {
        &self.auth
    }

    /// Send a streaming chat completions request.
    ///
    /// Returns a channel receiver that yields [`StreamEvent`]s as they arrive.
    /// The caller should read from the receiver until it closes or a
    /// [`StreamEvent::Done`] / [`StreamEvent::Error`] is received.
    pub async fn send_message_stream(
        &self,
        request: ChatRequest,
    ) -> Result<mpsc::UnboundedReceiver<StreamEvent>, ClientError> {
        let (copilot_token, api_base) = self.auth.ensure_copilot_token().await?;

        let url = format!("{api_base}/chat/completions");
        let ua = user_agent();

        let http = reqwest::Client::new();
        let req = http
            .post(&url)
            .header("Authorization", format!("Bearer {copilot_token}"))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("User-Agent", &ua)
            .header("Editor-Version", format!("Chuck/{APP_VERSION}"))
            .header("Editor-Plugin-Version", format!("copilot/{APP_VERSION}"))
            .header("Openai-Intent", "conversation-panel")
            .json(&request);

        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn a task to consume the SSE stream
        tokio::spawn(async move {
            let mut es = match EventSource::new(req) {
                Ok(es) => es,
                Err(e) => {
                    let _ = tx.send(StreamEvent::Error(format!(
                        "Failed to initialize stream: {e}"
                    )));
                    return;
                }
            };

            use futures_util::StreamExt;
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => {}
                    Ok(Event::Message(msg)) => {
                        if msg.data == "[DONE]" {
                            let _ = tx.send(StreamEvent::Done);
                            es.close();
                            break;
                        }

                        match serde_json::from_str::<StreamingChatResponse>(&msg.data) {
                            Ok(resp) => {
                                for choice in &resp.choices {
                                    if let Some(ref content) = choice.delta.content {
                                        if !content.is_empty()
                                            && tx.send(StreamEvent::Token(content.clone())).is_err()
                                        {
                                            log::debug!("Stream receiver dropped, stopping SSE");
                                            es.close();
                                            return;
                                        }
                                    }
                                    if choice.delta.role.is_some() {
                                        let _ = tx.send(StreamEvent::RoleSet);
                                    }
                                    if choice.finish_reason.is_some() {
                                        let _ = tx.send(StreamEvent::Done);
                                        es.close();
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse SSE chunk: {e}");
                            }
                        }
                    }
                    Err(reqwest_eventsource::Error::StreamEnded) => {
                        let _ = tx.send(StreamEvent::Done);
                        break;
                    }
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(e.to_string()));
                        es.close();
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Fetch available Copilot models.
    pub async fn get_models(&self) -> Result<Vec<Model>, ClientError> {
        let (copilot_token, api_base) = self.auth.ensure_copilot_token().await?;

        let url = format!("{api_base}/models");

        let resp = reqwest::Client::new()
            .get(&url)
            .header("Authorization", format!("Bearer {copilot_token}"))
            .header("Accept", "application/json")
            .header("User-Agent", user_agent())
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(ClientError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api { status, body });
        }

        let models_resp = resp.json::<ModelsResponse>().await?;
        Ok(models_resp.data)
    }
}

impl Default for CopilotClient {
    fn default() -> Self {
        Self::new()
    }
}
