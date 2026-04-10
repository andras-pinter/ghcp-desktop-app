//! HTTP client for Copilot chat completions with SSE streaming.

use crate::auth::{AuthError, DeviceFlowAuth};
use crate::types::{ChatRequest, Model, ModelsResponse, StreamingChatResponse};
use crate::{editor_plugin_version, editor_version, user_agent};
use reqwest_eventsource::{Event, EventSource};
use thiserror::Error;
use tokio::sync::mpsc;

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
/// Allowed domain suffixes for the Copilot API base URL.
const ALLOWED_API_DOMAINS: &[&str] = &[".githubcopilot.com", ".github.com"];

/// Validate that an API base URL points to a trusted GitHub domain.
fn validate_api_base(api_base: &str) -> Result<(), ClientError> {
    let parsed = url::Url::parse(api_base).map_err(|e| ClientError::Stream(e.to_string()))?;
    if parsed.scheme() != "https" {
        return Err(ClientError::Stream("API base URL must use HTTPS".into()));
    }
    let host = parsed.host_str().unwrap_or("");
    if !ALLOWED_API_DOMAINS.iter().any(|d| host.ends_with(d)) {
        return Err(ClientError::Stream(format!(
            "API base URL host '{host}' is not a trusted GitHub domain"
        )));
    }
    Ok(())
}

/// The main Copilot API client.
pub struct CopilotClient {
    auth: DeviceFlowAuth,
    http: reqwest::Client,
}

impl CopilotClient {
    /// Create a new Copilot client that manages auth automatically.
    pub fn new() -> Self {
        Self {
            auth: DeviceFlowAuth::new(),
            http: reqwest::Client::new(),
        }
    }

    /// Create with a custom auth handler (useful for testing).
    pub fn with_auth(auth: DeviceFlowAuth) -> Self {
        Self {
            auth,
            http: reqwest::Client::new(),
        }
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
        validate_api_base(&api_base)?;

        let url = format!("{api_base}/chat/completions");
        let ua = user_agent();

        let req = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {copilot_token}"))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("User-Agent", &ua)
            .header("Editor-Version", editor_version())
            .header("Editor-Plugin-Version", editor_plugin_version())
            .header("Openai-Intent", "conversation-panel")
            .json(&request);

        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn a task to consume the SSE stream
        tokio::spawn(async move {
            let mut es = match EventSource::new(req) {
                Ok(es) => es,
                Err(e) => {
                    if tx
                        .send(StreamEvent::Error(format!(
                            "Failed to initialize stream: {e}"
                        )))
                        .is_err()
                    {
                        log::trace!("Stream receiver dropped before error could be sent");
                    }
                    return;
                }
            };

            use futures_util::StreamExt;
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => {}
                    Ok(Event::Message(msg)) => {
                        if msg.data == "[DONE]" {
                            if tx.send(StreamEvent::Done).is_err() {
                                log::trace!("Stream receiver dropped on [DONE]");
                            }
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
                                    if choice.delta.role.is_some()
                                        && tx.send(StreamEvent::RoleSet).is_err()
                                    {
                                        log::trace!("Stream receiver dropped on RoleSet");
                                        es.close();
                                        return;
                                    }
                                    if choice.finish_reason.is_some() {
                                        if tx.send(StreamEvent::Done).is_err() {
                                            log::trace!("Stream receiver dropped on finish");
                                        }
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
                        if tx.send(StreamEvent::Done).is_err() {
                            log::trace!("Stream receiver dropped on StreamEnded");
                        }
                        break;
                    }
                    Err(e) => {
                        if tx.send(StreamEvent::Error(e.to_string())).is_err() {
                            log::trace!("Stream receiver dropped before error could be sent");
                        }
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
        validate_api_base(&api_base)?;

        let url = format!("{api_base}/models");

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {copilot_token}"))
            .header("Accept", "application/json")
            .header("User-Agent", user_agent())
            .header("Editor-Version", editor_version())
            .header("Editor-Plugin-Version", editor_plugin_version())
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

    /// Store a web search API key in the OS keychain.
    pub fn set_search_api_key(&self, key: &str) -> Result<(), crate::keychain::KeychainError> {
        crate::keychain::store("bing_api_key", key)
    }

    /// Retrieve the web search API key from the OS keychain.
    ///
    /// Returns `Ok(None)` if no key is stored, `Ok(Some(key))` if found.
    pub fn get_search_api_key(&self) -> Result<Option<String>, crate::keychain::KeychainError> {
        match crate::keychain::retrieve("bing_api_key") {
            Ok(key) => Ok(Some(key)),
            Err(crate::keychain::KeychainError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Default for CopilotClient {
    fn default() -> Self {
        Self::new()
    }
}
