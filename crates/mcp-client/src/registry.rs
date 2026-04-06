//! MCP Registry client.
//!
//! Fetches available MCP servers from the official MCP Registry
//! at <https://registry.modelcontextprotocol.io>.

use serde::{Deserialize, Serialize};

use crate::types::McpClientError;

/// Default registry base URL.
const REGISTRY_URL: &str = "https://registry.modelcontextprotocol.io/v0.1/servers";

/// Maximum number of entries to fetch from the registry.
const MAX_FETCH_COUNT: usize = 500;

// ── API response types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ApiResponse {
    servers: Vec<ApiEntry>,
}

#[derive(Debug, Deserialize)]
struct ApiEntry {
    server: ApiServer,
    #[serde(rename = "_meta")]
    meta: Option<ApiMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiServer {
    name: String,
    description: Option<String>,
    title: Option<String>,
    version: Option<String>,
    website_url: Option<String>,
    remotes: Option<Vec<ApiRemote>>,
    repository: Option<ApiRepository>,
}

#[derive(Debug, Deserialize)]
struct ApiRemote {
    #[serde(rename = "type")]
    transport_type: String,
    url: Option<String>,
    headers: Option<Vec<ApiHeader>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct ApiHeader {
    name: String,
    description: Option<String>,
    is_required: Option<bool>,
    is_secret: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ApiRepository {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiMeta {
    #[serde(rename = "io.modelcontextprotocol.registry/official")]
    official: Option<ApiOfficialMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiOfficialMeta {
    is_latest: Option<bool>,
}

// ── Public types ────────────────────────────────────────────────

/// A server entry from the MCP Registry, ready for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryServer {
    /// Unique identifier (e.g. "com.github/github-mcp-server").
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Description of the server.
    pub description: String,
    /// Version string.
    pub version: Option<String>,
    /// Project website URL.
    pub website_url: Option<String>,
    /// Source code repository URL.
    pub repo_url: Option<String>,
    /// Available remote connections.
    pub remotes: Vec<RegistryRemote>,
}

/// A remote connection option for a registry server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryRemote {
    /// Transport type (e.g. "streamable-http", "sse").
    pub transport_type: String,
    /// Server URL.
    pub url: Option<String>,
    /// Whether auth headers are required.
    pub requires_auth: bool,
    /// Description of required auth (if any).
    pub auth_description: Option<String>,
}

// ── Fetch logic ─────────────────────────────────────────────────

/// Fetch MCP servers from the official registry.
///
/// Returns only the latest version of each server, filtered to entries
/// that have at least one remote connection.
pub async fn fetch_registry(count: usize) -> Result<Vec<RegistryServer>, McpClientError> {
    let count = count.min(MAX_FETCH_COUNT);
    let url = format!("{REGISTRY_URL}?count={count}");

    log::info!("Fetching MCP registry: {url}");

    let response = reqwest::get(&url)
        .await
        .map_err(|e| McpClientError::Transport(format!("Failed to fetch MCP registry: {e}")))?;

    if !response.status().is_success() {
        return Err(McpClientError::Transport(format!(
            "MCP registry returned status {}",
            response.status()
        )));
    }

    let api: ApiResponse = response.json().await.map_err(|e| {
        McpClientError::Transport(format!("Failed to parse MCP registry response: {e}"))
    })?;

    // Filter for latest versions only, with at least one remote
    let servers: Vec<RegistryServer> = api
        .servers
        .into_iter()
        .filter(|entry| {
            entry
                .meta
                .as_ref()
                .and_then(|m| m.official.as_ref())
                .and_then(|o| o.is_latest)
                .unwrap_or(false)
        })
        .filter_map(|entry| convert_entry(entry.server))
        .collect();

    log::info!("Fetched {} servers from MCP registry", servers.len());
    Ok(servers)
}

/// Convert an API server to our public type.
fn convert_entry(server: ApiServer) -> Option<RegistryServer> {
    let remotes: Vec<RegistryRemote> = server
        .remotes?
        .into_iter()
        .filter_map(|r| {
            // Only include HTTP-based transports
            match r.transport_type.as_str() {
                "streamable-http" | "sse" | "http" => {}
                _ => return None,
            }
            let requires_auth = r
                .headers
                .as_ref()
                .map(|h| h.iter().any(|hdr| hdr.is_required.unwrap_or(false)))
                .unwrap_or(false);
            let auth_description = r.headers.as_ref().and_then(|h| {
                h.iter()
                    .find(|hdr| hdr.is_required.unwrap_or(false))
                    .and_then(|hdr| hdr.description.clone())
            });
            Some(RegistryRemote {
                transport_type: r.transport_type,
                url: r.url,
                requires_auth,
                auth_description,
            })
        })
        .collect();

    if remotes.is_empty() {
        return None;
    }

    // Use title if available, otherwise derive from name
    let display_name = server.title.unwrap_or_else(|| humanize_name(&server.name));

    Some(RegistryServer {
        name: server.name,
        display_name,
        description: server.description.unwrap_or_default(),
        version: server.version,
        website_url: server.website_url,
        repo_url: server.repository.and_then(|r| r.url),
        remotes,
    })
}

/// Convert a registry name like "com.github/github-mcp-server" to "GitHub MCP Server".
fn humanize_name(name: &str) -> String {
    // Take the part after the last '/'
    let short = name.rsplit('/').next().unwrap_or(name);
    // Replace hyphens with spaces and title-case
    short
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_name() {
        assert_eq!(
            humanize_name("com.github/github-mcp-server"),
            "Github Mcp Server"
        );
        assert_eq!(humanize_name("simple"), "Simple");
        assert_eq!(humanize_name("io.sentry/sentry-mcp"), "Sentry Mcp");
    }

    #[test]
    fn test_convert_entry_no_remotes() {
        let server = ApiServer {
            name: "test/server".to_string(),
            description: Some("Test".to_string()),
            title: None,
            version: Some("1.0".to_string()),
            website_url: None,
            remotes: None,
            repository: None,
        };
        assert!(convert_entry(server).is_none());
    }

    #[test]
    fn test_convert_entry_with_http_remote() {
        let server = ApiServer {
            name: "test/my-server".to_string(),
            description: Some("A test server".to_string()),
            title: Some("My Server".to_string()),
            version: Some("1.0".to_string()),
            website_url: Some("https://example.com".to_string()),
            remotes: Some(vec![ApiRemote {
                transport_type: "streamable-http".to_string(),
                url: Some("https://api.example.com/mcp".to_string()),
                headers: None,
            }]),
            repository: Some(ApiRepository {
                url: Some("https://github.com/test/repo".to_string()),
            }),
        };
        let result = convert_entry(server).unwrap();
        assert_eq!(result.display_name, "My Server");
        assert_eq!(result.remotes.len(), 1);
        assert!(!result.remotes[0].requires_auth);
    }
}
