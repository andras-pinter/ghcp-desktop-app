//! Web search and URL fetcher types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Errors ──────────────────────────────────────────────────────

/// Errors from web research operations.
#[derive(Debug, Error)]
pub enum WebResearchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Blocked URL: {0}")]
    BlockedUrl(String),

    #[error("Content extraction failed: {0}")]
    Extraction(String),

    #[error("Search API error: {status} — {message}")]
    SearchApi { status: u16, message: String },

    #[error("No API key configured")]
    MissingApiKey,

    #[error("Rate limited — retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
}

// ── Search types ────────────────────────────────────────────────

/// A web search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// Page title.
    pub title: String,
    /// Page URL.
    pub url: String,
    /// Short snippet / description.
    pub snippet: String,
    /// Display URL (may differ from actual URL).
    #[serde(default)]
    pub display_url: Option<String>,
}

/// Bing Web Search API v7 — top-level response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BingSearchResponse {
    pub web_pages: Option<BingWebPages>,
}

/// Bing `webPages` wrapper.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BingWebPages {
    pub value: Vec<BingWebResult>,
}

/// A single Bing web result.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BingWebResult {
    pub name: String,
    pub url: String,
    pub snippet: Option<String>,
    pub display_url: Option<String>,
}

impl From<BingWebResult> for SearchResult {
    fn from(r: BingWebResult) -> Self {
        Self {
            title: r.name,
            url: r.url,
            snippet: r.snippet.unwrap_or_default(),
            display_url: r.display_url,
        }
    }
}

/// Bing API error response.
#[derive(Debug, Deserialize)]
pub(crate) struct BingErrorResponse {
    pub errors: Option<Vec<BingError>>,
}

/// Single Bing API error.
#[derive(Debug, Deserialize)]
pub(crate) struct BingError {
    pub message: String,
}

// ── Fetcher types ───────────────────────────────────────────────

/// Extracted readable content from a URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedContent {
    /// Page title (if found).
    pub title: Option<String>,
    /// Extracted readable text (truncated to MAX_EXTRACT_BYTES).
    pub text: String,
    /// The URL that was fetched.
    pub url: String,
    /// Byte length of the extracted text.
    pub byte_length: usize,
    /// Whether the text was truncated.
    pub truncated: bool,
}

/// Maximum extracted text size (50 KB).
pub const MAX_EXTRACT_BYTES: usize = 50 * 1024;
