//! OAuth device flow authentication for GitHub Copilot.

/// Handles the GitHub OAuth device flow for Copilot authentication.
pub struct DeviceFlowAuth {
    client_id: String,
}

impl DeviceFlowAuth {
    /// Create a new device flow auth handler.
    pub fn new(client_id: String) -> Self {
        Self { client_id }
    }

    /// Get the client ID.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}
