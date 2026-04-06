//! URL fetcher + HTML-to-text extraction with SSRF protection.

use crate::types::{ExtractedContent, WebResearchError, MAX_EXTRACT_BYTES};
use futures_util::StreamExt;
use reqwest::Client;
use std::net::IpAddr;
use url::Url;

/// Maximum response body to download (5 MB — raw HTML before extraction).
const MAX_DOWNLOAD_BYTES: usize = 5 * 1024 * 1024;

/// Fetch a URL and extract readable content.
///
/// Enforces SSRF protection: HTTPS only, blocks private/reserved IPs.
/// IP validation happens at two layers:
/// 1. `validate_url()` rejects IP-literal URLs in blocked ranges
/// 2. The custom DNS resolver in `new_client()` blocks hostnames resolving to
///    private IPs at connect time (eliminating DNS rebinding TOCTOU)
///
/// Content is extracted via the Readability algorithm (dom_smoothie)
/// and truncated to [`MAX_EXTRACT_BYTES`].
pub async fn fetch_url(
    client: &Client,
    raw_url: &str,
) -> Result<ExtractedContent, WebResearchError> {
    let parsed = validate_url(raw_url)?;

    let response = client.get(parsed.as_str()).send().await?;

    if !response.status().is_success() {
        return Err(WebResearchError::Extraction(format!(
            "HTTP {} fetching URL",
            response.status(),
        )));
    }

    let bytes = read_limited_body(response, MAX_DOWNLOAD_BYTES).await?;
    let html = String::from_utf8_lossy(&bytes);

    extract_content(&html, raw_url)
}

/// Validate that a URL is safe to fetch.
fn validate_url(raw: &str) -> Result<Url, WebResearchError> {
    let parsed = Url::parse(raw)?;

    // HTTPS only
    if parsed.scheme() != "https" {
        return Err(WebResearchError::InvalidUrl(format!(
            "Only HTTPS URLs are supported, got: {}",
            parsed.scheme()
        )));
    }

    // Must have a host
    let host = parsed
        .host_str()
        .ok_or_else(|| WebResearchError::InvalidUrl("URL has no host".to_string()))?;

    // Reject IP-literal URLs pointing to blocked ranges
    // (Hostname URLs are checked by the custom DNS resolver at connect time)
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(WebResearchError::BlockedUrl(
                "URL points to a blocked address".to_string(),
            ));
        }
    }

    Ok(parsed)
}

/// Returns true if an IP address is in a blocked range.
///
/// Blocked ranges (SSRF protection):
/// - Loopback:      127.0.0.0/8, ::1
/// - Private:       10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
/// - Link-local:    169.254.0.0/16, fe80::/10
/// - Cloud meta:    169.254.169.254 (AWS/GCP/Azure metadata)
/// - CGNAT:         100.64.0.0/10
/// - Unspecified:   0.0.0.0, ::
/// - Broadcast:     255.255.255.255
/// - Documentation: 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
/// - Reserved:      240.0.0.0/4 (Class E)
/// - Unique-local:  fc00::/7 (IPv6 private equivalent)
/// - Site-local:    fec0::/10 (deprecated but still honoured by some stacks)
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()                     // 127.0.0.0/8
            || v4.is_private()                   // 10.x, 172.16-31.x, 192.168.x
            || v4.is_link_local()                // 169.254.0.0/16
            || v4.is_unspecified()               // 0.0.0.0
            || v4.octets()[0] == 0               // 0.0.0.0/8 ("this network", RFC 1122)
            || v4.is_broadcast()                 // 255.255.255.255
            || v4.is_documentation()             // 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
            || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xC0) == 64) // 100.64.0.0/10 (CGNAT)
            || (v4.octets()[0] & 0xF0) == 240 // 240.0.0.0/4 (Class E reserved)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()                     // ::1
            || v6.is_unspecified()               // ::
            // fe80::/10 (link-local)
            || (v6.segments()[0] & 0xffc0) == 0xfe80
            // fc00::/7 (unique-local — IPv6 private equivalent)
            || (v6.segments()[0] & 0xfe00) == 0xfc00
            // fec0::/10 (site-local, deprecated but still honoured)
            || (v6.segments()[0] & 0xffc0) == 0xfec0
            // Mapped/compat IPv4 — check the embedded v4 address
            || v6.to_ipv4().map(|v4| is_blocked_ip(IpAddr::V4(v4))).unwrap_or(false)
        }
    }
}

/// Read a response body with streaming, enforcing a size limit.
///
/// Uses chunked reads instead of buffering the entire response, preventing
/// memory exhaustion from malicious servers sending huge payloads.
async fn read_limited_body(
    response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, WebResearchError> {
    // Early reject via Content-Length header (compare in u64 to avoid truncation)
    if let Some(len) = response.content_length() {
        if len > limit as u64 {
            return Err(WebResearchError::Extraction(format!(
                "Response too large: {len} bytes (max {limit})"
            )));
        }
    }

    // Stream chunks with running size check
    let mut buf = Vec::with_capacity(limit.min(64 * 1024));
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if buf.len() + chunk.len() > limit {
            return Err(WebResearchError::Extraction(format!(
                "Response exceeded {limit} byte limit during download"
            )));
        }
        buf.extend_from_slice(&chunk);
    }

    Ok(buf)
}

/// Extract readable content from HTML using the Readability algorithm.
fn extract_content(html: &str, url: &str) -> Result<ExtractedContent, WebResearchError> {
    use dom_smoothie::Readability;

    let mut reader = Readability::new(html, Some(url), None)
        .map_err(|e| WebResearchError::Extraction(format!("Parse error: {e}")))?;

    let article = reader
        .parse()
        .map_err(|e| WebResearchError::Extraction(format!("Extraction failed: {e}")))?;

    let full_text = article.text_content.trim().to_string();
    let title = if article.title.is_empty() {
        None
    } else {
        Some(article.title)
    };

    // Truncate to MAX_EXTRACT_BYTES on a char boundary
    let (text, truncated) = if full_text.len() > MAX_EXTRACT_BYTES {
        let mut end = MAX_EXTRACT_BYTES;
        while !full_text.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        (full_text[..end].to_string(), true)
    } else {
        (full_text, false)
    };

    let byte_length = text.len();

    Ok(ExtractedContent {
        title,
        text,
        url: url.to_string(),
        byte_length,
        truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── IP validation tests ──────────────────────────────────────

    #[test]
    fn blocks_loopback_v4() {
        assert!(is_blocked_ip("127.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("127.255.255.255".parse().unwrap()));
    }

    #[test]
    fn blocks_loopback_v6() {
        assert!(is_blocked_ip("::1".parse().unwrap()));
    }

    #[test]
    fn blocks_private_10() {
        assert!(is_blocked_ip("10.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("10.255.255.255".parse().unwrap()));
    }

    #[test]
    fn blocks_private_172() {
        assert!(is_blocked_ip("172.16.0.1".parse().unwrap()));
        assert!(is_blocked_ip("172.31.255.255".parse().unwrap()));
        // 172.32.x is NOT private
        assert!(!is_blocked_ip("172.32.0.1".parse().unwrap()));
    }

    #[test]
    fn blocks_private_192_168() {
        assert!(is_blocked_ip("192.168.0.1".parse().unwrap()));
        assert!(is_blocked_ip("192.168.255.255".parse().unwrap()));
    }

    #[test]
    fn blocks_link_local() {
        assert!(is_blocked_ip("169.254.0.1".parse().unwrap()));
        assert!(is_blocked_ip("169.254.169.254".parse().unwrap())); // metadata
    }

    #[test]
    fn blocks_unspecified() {
        assert!(is_blocked_ip("0.0.0.0".parse().unwrap()));
        assert!(is_blocked_ip("::".parse().unwrap()));
    }

    #[test]
    fn blocks_broadcast() {
        assert!(is_blocked_ip("255.255.255.255".parse().unwrap()));
    }

    #[test]
    fn blocks_cgnat() {
        assert!(is_blocked_ip("100.64.0.1".parse().unwrap()));
        assert!(is_blocked_ip("100.127.255.255".parse().unwrap()));
        // 100.128.x is NOT CGNAT
        assert!(!is_blocked_ip("100.128.0.1".parse().unwrap()));
    }

    #[test]
    fn blocks_class_e() {
        assert!(is_blocked_ip("240.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("250.1.2.3".parse().unwrap()));
    }

    #[test]
    fn blocks_v6_unique_local() {
        assert!(is_blocked_ip("fd12::1".parse().unwrap()));
        assert!(is_blocked_ip("fc00::1".parse().unwrap()));
    }

    #[test]
    fn blocks_v6_site_local() {
        assert!(is_blocked_ip("fec0::1".parse().unwrap()));
    }

    #[test]
    fn allows_public_ipv4() {
        assert!(!is_blocked_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_blocked_ip("1.1.1.1".parse().unwrap()));
        assert!(!is_blocked_ip("93.184.216.34".parse().unwrap()));
    }

    #[test]
    fn allows_public_ipv6() {
        assert!(!is_blocked_ip("2607:f8b0:4004:800::200e".parse().unwrap()));
    }

    #[test]
    fn blocks_v6_link_local() {
        assert!(is_blocked_ip("fe80::1".parse().unwrap()));
    }

    #[test]
    fn blocks_v4_mapped_v6_private() {
        // ::ffff:127.0.0.1 is a v4-mapped v6 address
        assert!(is_blocked_ip("::ffff:127.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("::ffff:10.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("::ffff:192.168.1.1".parse().unwrap()));
    }

    // ── URL validation tests ─────────────────────────────────────

    #[test]
    fn rejects_http() {
        let err = validate_url("http://example.com").unwrap_err();
        assert!(matches!(err, WebResearchError::InvalidUrl(_)));
    }

    #[test]
    fn rejects_ftp() {
        let err = validate_url("ftp://example.com/file").unwrap_err();
        assert!(matches!(err, WebResearchError::InvalidUrl(_)));
    }

    #[test]
    fn accepts_https() {
        let url = validate_url("https://example.com/page").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn rejects_no_host() {
        // data: URLs have no host
        let err = validate_url("data:text/html,<h1>test</h1>").unwrap_err();
        assert!(matches!(err, WebResearchError::InvalidUrl(_)));
    }

    #[test]
    fn rejects_loopback_ip_literal() {
        let err = validate_url("https://127.0.0.1/path").unwrap_err();
        assert!(matches!(err, WebResearchError::BlockedUrl(_)));
    }

    #[test]
    fn rejects_private_ip_literal() {
        let err = validate_url("https://192.168.1.1/admin").unwrap_err();
        assert!(matches!(err, WebResearchError::BlockedUrl(_)));
    }

    // ── Content extraction tests ─────────────────────────────────

    #[test]
    fn extracts_article_content() {
        let html = r#"
        <html>
        <head><title>Test Article</title></head>
        <body>
            <nav>Navigation menu</nav>
            <article>
                <h1>Hello World</h1>
                <p>This is the main article content that should be extracted
                by the readability algorithm. It needs to be long enough to
                be considered real content by the heuristics.</p>
                <p>Adding more paragraphs helps the readability algorithm
                determine this is the main content area. Each paragraph
                adds confidence to the content scoring.</p>
                <p>The third paragraph makes it even more convincing that
                this is genuine article text and not a sidebar or navigation
                element that should be stripped away.</p>
            </article>
            <footer>Footer content</footer>
        </body>
        </html>"#;

        let result = extract_content(html, "https://example.com/article").unwrap();
        assert_eq!(result.url, "https://example.com/article");
        assert!(!result.truncated);
        assert!(result.byte_length > 0);
        // The main content should have been extracted
        assert!(result.text.contains("main article content"));
    }

    #[test]
    fn truncates_long_content() {
        // Create content larger than MAX_EXTRACT_BYTES
        let paragraph = "This is a paragraph of content. ".repeat(2000);
        let html = format!("<html><body><article><p>{paragraph}</p></article></body></html>");

        let result = extract_content(&html, "https://example.com").unwrap();
        assert!(result.truncated);
        assert!(result.byte_length <= MAX_EXTRACT_BYTES);
    }
}
