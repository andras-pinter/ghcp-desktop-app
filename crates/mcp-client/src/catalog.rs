//! Built-in catalog of well-known MCP servers.
//!
//! Static list shipped with the app. Users can enable entries,
//! provide required configuration (API keys, paths), and connect.

use serde::Serialize;

use crate::types::McpTransport;

/// A catalog entry for a well-known MCP server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    /// Unique identifier (used as `mcp_servers.id` when added).
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Description of what this server provides.
    pub description: &'static str,
    /// Default transport.
    pub transport: McpTransport,
    /// Default URL (for HTTP transport).
    pub default_url: Option<&'static str>,
    /// Default binary name (for stdio transport).
    pub default_binary: Option<&'static str>,
    /// Required configuration fields the user must provide.
    pub required_fields: &'static [CatalogField],
}

/// A required configuration field for a catalog entry.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogField {
    /// Field identifier.
    pub key: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Placeholder text for the input.
    pub placeholder: &'static str,
    /// Whether the field value is secret (e.g. API key).
    pub secret: bool,
}

/// The built-in catalog of MCP servers.
pub static CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        id: "catalog-github",
        name: "GitHub",
        description: "Repository search, issues, pull requests, and code browsing.",
        transport: McpTransport::Http,
        default_url: Some("https://api.githubcopilot.com/mcp/"),
        default_binary: None,
        required_fields: &[],
    },
    CatalogEntry {
        id: "catalog-brave-search",
        name: "Brave Search",
        description: "Privacy-focused web search via Brave Search API.",
        transport: McpTransport::Http,
        default_url: None,
        default_binary: None,
        required_fields: &[CatalogField {
            key: "url",
            label: "Server URL",
            placeholder: "https://your-brave-mcp.example.com",
            secret: false,
        }],
    },
    CatalogEntry {
        id: "catalog-filesystem",
        name: "Filesystem (read-only)",
        description: "Read-only access to a user-selected directory.",
        transport: McpTransport::Stdio,
        default_url: None,
        default_binary: Some("mcp-server-filesystem"),
        required_fields: &[CatalogField {
            key: "binary_path",
            label: "Binary Path",
            placeholder: "/usr/local/bin/mcp-server-filesystem",
            secret: false,
        }],
    },
    CatalogEntry {
        id: "catalog-postgres",
        name: "PostgreSQL",
        description: "Execute read-only SQL queries against a PostgreSQL database.",
        transport: McpTransport::Stdio,
        default_url: None,
        default_binary: Some("pg-mcp-server"),
        required_fields: &[CatalogField {
            key: "binary_path",
            label: "Binary Path",
            placeholder: "/usr/local/bin/pg-mcp-server",
            secret: false,
        }],
    },
];

/// Look up a catalog entry by ID.
pub fn get_catalog_entry(id: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_entries() {
        assert!(!CATALOG.is_empty());
        assert!(CATALOG.len() >= 4);
    }

    #[test]
    fn catalog_ids_unique() {
        let mut ids: Vec<_> = CATALOG.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CATALOG.len());
    }

    #[test]
    fn lookup_by_id() {
        assert!(get_catalog_entry("catalog-github").is_some());
        assert!(get_catalog_entry("nonexistent").is_none());
    }

    #[test]
    fn github_entry_is_http() {
        let entry = get_catalog_entry("catalog-github").unwrap();
        assert_eq!(entry.transport, McpTransport::Http);
        assert!(entry.default_url.is_some());
        assert!(entry.required_fields.is_empty());
    }

    #[test]
    fn catalog_serialization() {
        let json = serde_json::to_string(&CATALOG[0]).unwrap();
        assert!(json.contains("\"transport\":\"http\""));
    }
}
