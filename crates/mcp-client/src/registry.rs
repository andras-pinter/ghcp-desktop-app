//! MCP Registry client.
//!
//! Fetches available MCP servers from the official MCP Registry
//! at <https://registry.modelcontextprotocol.io>.
//!
//! Uses server-side search (`?search=`) and version filtering
//! (`?version=latest`) for efficient queries.

use serde::{Deserialize, Serialize};

use crate::types::McpClientError;

/// Default registry base URL.
const REGISTRY_URL: &str = "https://registry.modelcontextprotocol.io/v0.1/servers";

/// Results per page (API max is 100).
const PAGE_SIZE: usize = 100;

/// Maximum pages to fetch for a single query (safety limit).
const MAX_PAGES: usize = 30;

// ── API response types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ApiResponse {
    servers: Vec<ApiEntry>,
    metadata: Option<ApiPaginationMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPaginationMeta {
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiEntry {
    server: ApiServer,
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
    /// Available remote HTTP connections (empty for stdio-only servers).
    pub remotes: Vec<RegistryRemote>,
    /// Whether this server is stdio-only (no HTTP remotes).
    pub is_stdio_only: bool,
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

/// Search the MCP Registry with an optional query string.
///
/// - `query` — if `Some`, performs server-side substring search by name.
///   If `None`, returns a paginated listing of all latest servers.
/// - Always requests `version=latest` to avoid duplicates.
/// - Paginates to collect all matching results.
pub async fn fetch_registry(query: Option<&str>) -> Result<Vec<RegistryServer>, McpClientError> {
    let client = reqwest::Client::new();
    let mut all_servers = Vec::new();
    let mut cursor: Option<String> = None;

    // For unfiltered browsing, cap at a reasonable number of pages
    let max_pages = if query.is_some() { 5 } else { MAX_PAGES };

    for page in 0..max_pages {
        let mut url = format!("{REGISTRY_URL}?version=latest&limit={PAGE_SIZE}");

        if let Some(q) = query {
            url.push_str(&format!(
                "&search={}",
                url::form_urlencoded::byte_serialize(q.as_bytes()).collect::<String>()
            ));
        }

        if let Some(ref c) = cursor {
            url.push_str(&format!("&cursor={c}"));
        }

        if page == 0 {
            log::info!(
                "Fetching MCP registry{}...",
                query.map(|q| format!(" (search: {q})")).unwrap_or_default()
            );
        }

        let response =
            client.get(&url).send().await.map_err(|e| {
                McpClientError::Transport(format!("Failed to fetch MCP registry: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(McpClientError::Transport(format!(
                "MCP registry returned status {}",
                response.status()
            )));
        }

        let api: ApiResponse = response.json().await.map_err(|e| {
            McpClientError::Transport(format!("Failed to parse MCP registry response: {e}"))
        })?;

        let batch_len = api.servers.len();
        for entry in api.servers {
            if let Some(server) = convert_entry(entry.server) {
                all_servers.push(server);
            }
        }

        cursor = api.metadata.and_then(|m| m.next_cursor);
        if cursor.is_none() || batch_len == 0 {
            break;
        }
    }

    // Sort: first-party servers first (heuristic: name prefix is not io.github.* or
    // ai.smithery/*), then alphabetically by display name within each group.
    all_servers.sort_by(|a, b| {
        let a_first_party = is_first_party(&a.name);
        let b_first_party = is_first_party(&b.name);
        b_first_party.cmp(&a_first_party).then_with(|| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        })
    });

    log::info!("Fetched {} servers from MCP registry", all_servers.len());
    Ok(all_servers)
}

/// Convert an API server to our public type.
fn convert_entry(server: ApiServer) -> Option<RegistryServer> {
    let remotes: Vec<RegistryRemote> = server
        .remotes
        .unwrap_or_default()
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

    let is_stdio_only = remotes.is_empty();

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
        is_stdio_only,
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

/// Heuristic: a server is "first-party" if its reverse-DNS name prefix
/// belongs to the actual company (e.g., `com.microsoft/azure`) rather than
/// a community namespace (`io.github.user/...`, `ai.smithery/...`).
fn is_first_party(name: &str) -> bool {
    !name.starts_with("io.github.")
        && !name.starts_with("ai.smithery/")
        && !name.starts_with("ai.smithery.")
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
    fn test_convert_entry_no_remotes_is_stdio() {
        let server = ApiServer {
            name: "test/server".to_string(),
            description: Some("A stdio server".to_string()),
            title: None,
            version: Some("1.0".to_string()),
            website_url: None,
            remotes: None,
            repository: Some(ApiRepository {
                url: Some("https://github.com/test/server".to_string()),
            }),
        };
        let result = convert_entry(server).unwrap();
        assert!(result.is_stdio_only);
        assert!(result.remotes.is_empty());
        assert_eq!(result.display_name, "Server");
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
        assert!(!result.is_stdio_only);
        assert_eq!(result.remotes.len(), 1);
        assert!(!result.remotes[0].requires_auth);
    }

    #[test]
    fn test_convert_entry_empty_remotes_is_stdio() {
        let server = ApiServer {
            name: "test/stdio-server".to_string(),
            description: Some("Stdio only".to_string()),
            title: None,
            version: None,
            website_url: None,
            remotes: Some(vec![]),
            repository: None,
        };
        let result = convert_entry(server).unwrap();
        assert!(result.is_stdio_only);
    }

    #[test]
    fn test_first_party_heuristic() {
        assert!(is_first_party("com.microsoft/azure"));
        assert!(is_first_party("com.stripe/mcp"));
        assert!(is_first_party("com.cloudflare.mcp/mcp"));
        assert!(is_first_party("io.sentry/sentry-mcp"));
        assert!(!is_first_party("io.github.user/my-server"));
        assert!(!is_first_party("io.github.getsentry/sentry-mcp"));
        assert!(!is_first_party("ai.smithery/some-server"));
        assert!(!is_first_party("ai.smithery.proxy/some-server"));
    }
}
