//! Web search API client (Bing Web Search API v7).

use crate::types::{BingErrorResponse, BingSearchResponse, SearchResult, WebResearchError};
use reqwest::Client;

/// Bing Web Search API v7 endpoint.
const BING_SEARCH_URL: &str = "https://api.bing.microsoft.com/v7.0/search";

/// Default number of results to return.
const DEFAULT_COUNT: u8 = 5;

/// Search the web via Bing Web Search API.
///
/// # Arguments
/// * `client`  — a reusable `reqwest::Client`
/// * `api_key` — Bing Search API subscription key
/// * `query`   — search query string
/// * `count`   — max results (1–50, default 5)
pub async fn web_search(
    client: &Client,
    api_key: &str,
    query: &str,
    count: Option<u8>,
) -> Result<Vec<SearchResult>, WebResearchError> {
    if api_key.is_empty() {
        return Err(WebResearchError::MissingApiKey);
    }

    let count = count.unwrap_or(DEFAULT_COUNT).clamp(1, 50);

    let response = client
        .get(BING_SEARCH_URL)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .query(&[
            ("q", query),
            ("count", &count.to_string()),
            ("responseFilter", "Webpages"),
            ("textFormat", "Raw"),
        ])
        .send()
        .await?;

    let status = response.status();

    // Rate limit
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30);
        return Err(WebResearchError::RateLimited {
            retry_after_secs: retry_after,
        });
    }

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let message = serde_json::from_str::<BingErrorResponse>(&body)
            .ok()
            .and_then(|e| e.errors)
            .and_then(|errs| errs.into_iter().next())
            .map(|e| e.message)
            .unwrap_or(body);
        return Err(WebResearchError::SearchApi {
            status: status.as_u16(),
            message,
        });
    }

    let body: BingSearchResponse = response.json().await?;

    let results = body
        .web_pages
        .map(|wp| wp.value.into_iter().map(SearchResult::from).collect())
        .unwrap_or_default();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bing_response() {
        let json = r#"{
            "webPages": {
                "totalEstimatedMatches": 1234,
                "value": [
                    {
                        "name": "Rust Programming Language",
                        "url": "https://www.rust-lang.org/",
                        "snippet": "A language empowering everyone.",
                        "displayUrl": "https://www.rust-lang.org"
                    },
                    {
                        "name": "Rust Wikipedia",
                        "url": "https://en.wikipedia.org/wiki/Rust",
                        "snippet": null,
                        "displayUrl": null
                    }
                ]
            }
        }"#;

        let resp: BingSearchResponse = serde_json::from_str(json).unwrap();
        let wp = resp.web_pages.unwrap();
        assert_eq!(wp.value.len(), 2);

        let results: Vec<SearchResult> = wp.value.into_iter().map(SearchResult::from).collect();
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[0].snippet, "A language empowering everyone.");
        assert_eq!(results[1].snippet, ""); // null → empty
    }

    #[test]
    fn parse_bing_empty_response() {
        let json = r#"{ "webPages": null }"#;
        let resp: BingSearchResponse = serde_json::from_str(json).unwrap();
        assert!(resp.web_pages.is_none());
    }

    #[test]
    fn parse_bing_error_response() {
        let json = r#"{ "errors": [{ "message": "Invalid subscription key" }] }"#;
        let resp: BingErrorResponse = serde_json::from_str(json).unwrap();
        let errs = resp.errors.unwrap();
        assert_eq!(errs[0].message, "Invalid subscription key");
    }
}
