//! HTTP client for Copilot chat completions with SSE streaming.

/// Client for the GitHub Copilot `/v1/chat/completions` endpoint.
pub struct CopilotClient {
    http: reqwest::Client,
    base_url: String,
}

impl CopilotClient {
    /// Create a new Copilot API client.
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
        }
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get a reference to the HTTP client.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
}
