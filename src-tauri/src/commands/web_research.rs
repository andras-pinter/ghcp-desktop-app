//! Web research commands: web_search, fetch_url.

use crate::state::AppState;
use tauri::State;

/// Search the web via Bing Web Search API.
///
/// The API key is read from the OS keychain (set via Settings → Web Research).
/// Returns a list of search results with title, URL, and snippet.
#[tauri::command]
pub async fn web_search(
    state: State<'_, AppState>,
    query: String,
    count: Option<u8>,
) -> Result<Vec<web_research::SearchResult>, String> {
    if query.trim().is_empty() {
        return Err("Search query cannot be empty".to_string());
    }

    let api_key = state
        .copilot
        .get_search_api_key()
        .map_err(|e| format!("Bing API key not configured: {e}"))?
        .ok_or_else(|| {
            "Bing API key is required. Configure it in Settings → Web Research.".to_string()
        })?;

    web_research::web_search(&state.http_client, &api_key, &query, count)
        .await
        .map_err(|e| e.to_string())
}

/// Fetch a URL and extract its readable content.
///
/// Only HTTPS URLs pointing to public IP addresses are allowed.
/// Content is extracted using the Readability algorithm and truncated to 50 KB.
#[tauri::command]
pub async fn fetch_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<web_research::ExtractedContent, String> {
    if url.trim().is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    web_research::fetch_url(&state.http_client, &url)
        .await
        .map_err(|e| e.to_string())
}
