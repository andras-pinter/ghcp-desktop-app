//! Web research library.
//!
//! Provides web search API integration and URL content fetching/extraction.
//! Zero Tauri dependency — usable from any Rust project.

pub mod fetcher;
pub mod search;
pub mod types;

pub use fetcher::fetch_url;
pub use reqwest::Client as HttpClient;
pub use search::web_search;
pub use types::{ExtractedContent, SearchResult, WebResearchError};

use std::net::IpAddr;
use std::sync::Arc;

/// User-Agent sent with all web research requests.
const USER_AGENT: &str = "Chuck/1.0 (Desktop; Copilot Chat)";

/// Default request timeout.
const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// Maximum number of redirects to follow.
const MAX_REDIRECTS: usize = 5;

/// Create a hardened HTTP client for web research requests.
///
/// The client includes SSRF protection:
/// - Custom DNS resolver ([`SafeResolver`]) that blocks private/reserved IP ranges
/// - Custom redirect policy that re-validates each hop
/// - Timeout and User-Agent defaults
///
/// # Errors
///
/// Returns an error if the HTTP client builder fails (e.g. TLS backend issue).
pub fn new_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(DEFAULT_TIMEOUT)
        .dns_resolver(Arc::new(SafeResolver))
        .redirect(safe_redirect_policy())
        .build()
}

/// DNS resolver that blocks connections to private/reserved IP ranges.
///
/// This eliminates DNS rebinding attacks by validating IPs at resolve time,
/// not in a separate pre-check step (which suffers from TOCTOU races).
struct SafeResolver;

impl reqwest::dns::Resolve for SafeResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let addr_str = format!("{host}:0");
            let addrs: Vec<std::net::SocketAddr> = tokio::net::lookup_host(&addr_str)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?
                .collect();

            let safe: Vec<std::net::SocketAddr> = addrs
                .into_iter()
                .filter(|a| !fetcher::is_blocked_ip(a.ip()))
                .collect();

            if safe.is_empty() {
                return Err("Host resolves to a blocked address".into());
            }

            Ok(Box::new(safe.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

/// Redirect policy that re-validates each redirect target.
///
/// Prevents SSRF bypass where an attacker controls `https://evil.com` which
/// 302-redirects to `http://169.254.169.254/metadata`.
fn safe_redirect_policy() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            return attempt.error("too many redirects");
        }

        let url = attempt.url();

        // Must stay HTTPS
        if url.scheme() != "https" {
            return attempt.error("redirect to non-HTTPS URL");
        }

        // Block IP-literal redirects to private ranges
        if let Some(host) = url.host_str() {
            if let Ok(ip) = host.parse::<IpAddr>() {
                if fetcher::is_blocked_ip(ip) {
                    return attempt.error("redirect to blocked IP address");
                }
            }
        }

        // Hostname redirects are validated by SafeResolver at connect time
        attempt.follow()
    })
}
