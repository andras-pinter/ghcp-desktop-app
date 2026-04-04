//! Web search and URL fetcher types.

use serde::{Deserialize, Serialize};

/// A web search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Extracted content from a URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    pub title: Option<String>,
    pub text: String,
    pub url: String,
}
