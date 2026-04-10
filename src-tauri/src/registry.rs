//! Registry client for browsing and installing skills/agents from
//! aitmpl.com, plus importing from git URLs.
//!
//! Uses a pluggable `RegistryProvider` trait so new catalog sources
//! can be added by implementing a single trait.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// A unified registry search result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryItem {
    /// Unique slug/id on the registry.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Short description.
    pub description: Option<String>,
    /// Which registry this came from.
    pub source: RegistrySource,
    /// Human-readable label for the source (e.g., "aitmpl.com" or git source name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    /// URL to the skill/agent on the registry.
    pub url: Option<String>,
    /// Install count (if available).
    pub installs: Option<u64>,
    /// Whether this is a skill or agent template.
    pub kind: RegistryItemKind,
    /// GitHub owner/repo for content fetching.
    pub source_repo: Option<String>,
    /// Full SKILL.md content (available for aitmpl items from components.json).
    /// Skips the need to re-fetch during install.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Which registry a result came from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegistrySource {
    Aitmpl,
    /// For future custom catalog URLs.
    Custom,
    /// From a user-configured git source.
    Git,
}

/// Whether the item is a skill or agent template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegistryItemKind {
    Skill,
    Agent,
}

/// Response from a registry search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySearchResult {
    pub items: Vec<RegistryItem>,
    pub total: Option<u64>,
}

// ── RegistryProvider trait ──────────────────────────────────────

/// A pluggable registry source for browsing and installing skills/agents.
#[async_trait]
pub trait RegistryProvider: Send + Sync {
    /// Human-readable name of this registry (e.g., "aitmpl.com").
    fn name(&self) -> &str;

    /// Search for items matching a query.
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<RegistryItem>, String>;

    /// Fetch the full content for an item by ID.
    /// Returns the raw SKILL.md / AGENT.md content.
    async fn fetch_content(&self, item_id: &str) -> Result<String, String>;
}

// ── aitmpl.com (via GitHub-hosted components.json) ─────────────

/// An item in the aitmpl.com components.json file.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct AitmplComponent {
    name: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    component_type: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

/// The full components.json structure from aitmpl.com GitHub repo.
#[derive(Debug, Deserialize)]
struct AitmplComponentsJson {
    #[serde(default)]
    agents: Vec<AitmplComponent>,
    #[serde(default)]
    skills: Vec<AitmplComponent>,
}

const AITMPL_COMPONENTS_URL: &str =
    "https://raw.githubusercontent.com/davila7/claude-code-templates/main/docs/components.json";

/// Search aitmpl.com by fetching components.json from GitHub and filtering locally.
pub async fn search_aitmpl(
    client: &Client,
    query: &str,
    limit: u32,
) -> Result<Vec<RegistryItem>, String> {
    let resp = client
        .get(AITMPL_COMPONENTS_URL)
        .send()
        .await
        .map_err(|e| format!("aitmpl.com GitHub fetch failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "aitmpl.com components.json returned {}",
            resp.status()
        ));
    }

    let data: AitmplComponentsJson = resp
        .json()
        .await
        .map_err(|e| format!("aitmpl.com parse failed: {e}"))?;

    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let browse_all = query_words.is_empty();

    // Collect all items with a relevance score
    let mut scored: Vec<(RegistryItem, u32)> = Vec::new();

    for item in &data.agents {
        let score = if browse_all {
            1
        } else {
            match_score(&item.name, item.category.as_deref(), &query_words)
        };
        if score > 0 {
            scored.push((
                aitmpl_to_registry_item(item, RegistryItemKind::Agent),
                score,
            ));
        }
    }
    for item in &data.skills {
        let score = if browse_all {
            1
        } else {
            match_score(&item.name, item.category.as_deref(), &query_words)
        };
        if score > 0 {
            scored.push((
                aitmpl_to_registry_item(item, RegistryItemKind::Skill),
                score,
            ));
        }
    }

    if browse_all {
        // Browse mode: sort by popularity (installs descending)
        scored.sort_by(|a, b| {
            b.0.installs
                .unwrap_or(0)
                .cmp(&a.0.installs.unwrap_or(0))
                .then_with(|| a.0.name.cmp(&b.0.name))
        });
    } else {
        // Search mode: sort by relevance score descending, then by name
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));
    }

    let results: Vec<RegistryItem> = scored
        .into_iter()
        .take(limit as usize)
        .map(|(item, _)| item)
        .collect();

    Ok(results)
}

/// Score how well an item matches the query. 0 = no match.
/// Name matches score higher than category matches.
fn match_score(name: &str, category: Option<&str>, query_words: &[&str]) -> u32 {
    let name_lower = name.to_lowercase();
    let cat_lower = category.map(|c| c.to_lowercase());

    let mut score = 0u32;
    for &word in query_words {
        if name_lower.contains(word) {
            score += 10; // name match = high score
        } else if cat_lower.as_ref().is_some_and(|c| c.contains(word)) {
            score += 3; // category match = lower score
        }
        // Skip description matching — it's too noisy with these long descriptions
    }
    score
}

/// Clean a raw description string from aitmpl — strip XML tags,
/// literal `\n`, and truncate to a sensible length for display.
fn clean_description(raw: &str) -> String {
    let cleaned = raw.replace("\\n", " ").replace('\n', " ");

    // Simple XML-like tag stripping
    let mut result = cleaned;
    loop {
        if let Some(start) = result.find('<') {
            if let Some(end) = result[start..].find('>') {
                let tag = &result[start..start + end + 1];
                if tag.len() < 30
                    && (tag.starts_with("</")
                        || tag.chars().nth(1).is_some_and(|c| c.is_alphabetic()))
                {
                    result = format!("{}{}", &result[..start], &result[start + end + 1..]);
                    continue;
                }
            }
        }
        break;
    }

    // Collapse multiple spaces
    let result: String = result.split_whitespace().collect::<Vec<_>>().join(" ");

    // Truncate to ~200 chars at word boundary
    if result.len() > 200 {
        let truncated = &result[..200];
        let end = truncated.rfind(' ').unwrap_or(200);
        format!("{}…", &truncated[..end])
    } else {
        result
    }
}

fn aitmpl_to_registry_item(item: &AitmplComponent, kind: RegistryItemKind) -> RegistryItem {
    // Build unique ID from path (name alone has duplicates across categories)
    let id = item
        .path
        .as_ref()
        .map(|p| p.replace(".md", "").replace('/', "-"))
        .unwrap_or_else(|| item.name.clone());

    // Link to aitmpl.com SPA: /component/{type}/{category}/{name}
    let type_str = match kind {
        RegistryItemKind::Agent => "agent",
        RegistryItemKind::Skill => "skill",
    };
    let url = item.path.as_ref().map(|p| {
        let path_no_ext = p.replace(".md", "");
        format!("https://www.aitmpl.com/component/{type_str}/{path_no_ext}")
    });

    // Extract first paragraph of the markdown body as description
    let description = item
        .content
        .as_ref()
        .and_then(|c| extract_body_description(c))
        .or_else(|| item.description.as_ref().map(|d| clean_description(d)));

    RegistryItem {
        id,
        name: item.name.clone(),
        description,
        source: RegistrySource::Aitmpl,
        source_name: Some("aitmpl.com".to_string()),
        url,
        installs: None,
        kind,
        source_repo: None,
        content: item.content.clone(),
    }
}

/// Extract the first meaningful paragraph from the markdown body
/// (after YAML frontmatter) as a description.
fn extract_body_description(content: &str) -> Option<String> {
    let trimmed = content.trim_start();

    // Skip YAML frontmatter
    let body = if let Some(after) = trimmed.strip_prefix("---") {
        let after_first = after
            .strip_prefix('\n')
            .unwrap_or(after.strip_prefix("\r\n").unwrap_or(after));
        if let Some(closing) = after_first.find("\n---") {
            let rest = &after_first[closing + 4..];
            rest.strip_prefix('\n')
                .unwrap_or(rest.strip_prefix("\r\n").unwrap_or(rest))
        } else {
            return None;
        }
    } else {
        trimmed
    };

    // Take the first non-empty paragraph (split by blank lines)
    let first_para = body
        .split("\n\n")
        .map(|p| p.trim())
        .find(|p| !p.is_empty() && !p.starts_with('#'))?;

    // Clean and truncate
    let cleaned: String = first_para
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.is_empty() {
        return None;
    }

    Some(if cleaned.len() > 200 {
        // Use char boundary-safe truncation to avoid panicking on multi-byte UTF-8
        let safe_end = cleaned
            .char_indices()
            .take_while(|&(i, _)| i < 200)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(cleaned.len());
        let truncated = &cleaned[..safe_end];
        let end = truncated.rfind(' ').unwrap_or(safe_end);
        format!("{}…", &cleaned[..end])
    } else {
        cleaned
    })
}

// ── AitmplProvider ──────────────────────────────────────────────

/// Registry provider backed by the aitmpl.com components.json catalog.
pub struct AitmplProvider {
    client: Client,
}

impl AitmplProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl RegistryProvider for AitmplProvider {
    fn name(&self) -> &str {
        "aitmpl.com"
    }

    async fn search(&self, query: &str, limit: u32) -> Result<Vec<RegistryItem>, String> {
        search_aitmpl(&self.client, query, limit).await
    }

    async fn fetch_content(&self, item_id: &str) -> Result<String, String> {
        fetch_aitmpl_content(&self.client, item_id).await
    }
}

// ── Unified search ─────────────────────────────────────────────

/// Search all registered providers and return unified results.
///
/// Currently uses `AitmplProvider` only. To add another source, create a
/// struct that implements `RegistryProvider` and include it in `providers`.
pub async fn search_registries(
    client: &Client,
    query: &str,
    limit: u32,
) -> Result<RegistrySearchResult, String> {
    let aitmpl = AitmplProvider::new(client.clone());
    let providers: Vec<&dyn RegistryProvider> = vec![&aitmpl];

    let mut items = Vec::new();
    for provider in providers {
        match provider.search(query, limit).await {
            Ok(results) => items.extend(results),
            Err(e) => log::warn!("{} search failed: {e}", provider.name()),
        }
    }

    // Sort by download count (highest first, items without counts last)
    items.sort_by(|a, b| b.installs.unwrap_or(0).cmp(&a.installs.unwrap_or(0)));

    let total = items.len() as u64;
    Ok(RegistrySearchResult {
        items,
        total: Some(total),
    })
}

// ── Lenient content parser ──────────────────────────────────────

/// Parse SKILL.md content leniently — extracts what it can without
/// strict validation. Used as fallback when `skillmd::parse()` fails
/// (e.g., description too long, name has uppercase, etc.).
///
/// Returns `(name, description, instructions)`.
pub fn parse_content_lenient(content: &str, fallback_id: &str) -> (String, String, String) {
    let trimmed = content.trim_start();

    // Try to extract from YAML frontmatter
    if let Some(after_marker) = trimmed.strip_prefix("---") {
        let after_first = after_marker
            .strip_prefix('\n')
            .unwrap_or(after_marker.strip_prefix("\r\n").unwrap_or(after_marker));

        if let Some(closing_idx) = after_first.find("\n---") {
            let yaml_str = &after_first[..closing_idx];
            let body_start = closing_idx + 4;
            let body = if body_start < after_first.len() {
                after_first[body_start..]
                    .strip_prefix('\n')
                    .unwrap_or(&after_first[body_start..])
            } else {
                ""
            };

            // Extract name and description from YAML lines
            let mut name = None;
            let mut desc = None;
            for line in yaml_str.lines() {
                if let Some(val) = line.strip_prefix("name:") {
                    name = Some(val.trim().trim_matches('"').to_string());
                }
                if desc.is_none() {
                    if let Some(val) = line.strip_prefix("description:") {
                        // Take just the first line of description, truncated
                        let d = val.trim().trim_matches('"');
                        let clean = d.lines().next().unwrap_or(d);
                        desc = Some(if clean.len() > 200 {
                            format!("{}…", &clean[..197])
                        } else {
                            clean.to_string()
                        });
                    }
                }
            }

            return (
                name.unwrap_or_else(|| fallback_id.to_string()),
                desc.unwrap_or_else(|| "Imported from registry".to_string()),
                body.trim().to_string(),
            );
        }
    }

    // No frontmatter — use whole content as instructions
    (
        fallback_id.to_string(),
        "Imported from registry".to_string(),
        content.trim().to_string(),
    )
}

// ── Fetch content from registry ─────────────────────────────────

/// Fetch the content for a registry item by ID.
///
/// If `inline_content` is provided (aitmpl items carry content from components.json),
/// it is returned directly without any network request. Otherwise, re-fetches
/// from the components.json catalog.
pub async fn fetch_skill_content(
    client: &Client,
    skill_id: &str,
    _source: &RegistrySource,
    _source_repo: Option<&str>,
    inline_content: Option<&str>,
) -> Result<String, String> {
    // Fast path: use inline content if available (aitmpl items always carry it)
    if let Some(content) = inline_content {
        if !content.is_empty() {
            return Ok(content.to_string());
        }
    }

    // Fallback: re-fetch from components.json
    fetch_aitmpl_content(client, skill_id).await
}

/// Fetch the content for an aitmpl.com item by re-fetching components.json.
async fn fetch_aitmpl_content(client: &Client, item_id: &str) -> Result<String, String> {
    let resp = client
        .get(AITMPL_COMPONENTS_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch components.json: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("components.json returned {}", resp.status()));
    }

    let data: AitmplComponentsJson = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse components.json: {e}"))?;

    // Match by path-based ID or name
    for item in data.agents.iter().chain(data.skills.iter()) {
        let item_id_from_path = item
            .path
            .as_ref()
            .map(|p| p.replace(".md", "").replace('/', "-"))
            .unwrap_or_else(|| item.name.clone());

        if item_id_from_path == item_id || item.name == item_id {
            if let Some(content) = &item.content {
                return Ok(content.clone());
            }
        }
    }

    Err(format!("Item '{item_id}' not found in aitmpl.com catalog"))
}

// ── Git URL Import ─────────────────────────────────────────────

/// A discovered definition file (SKILL.md or *.agent.md) from a git repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSkillFile {
    /// Path within the repository.
    pub path: String,
    /// The raw file content.
    pub content: String,
    /// The repo URL it came from.
    pub repo_url: String,
    /// What kind of definition: "skill" or "agent".
    pub kind: String,
}

/// Parsed git URL components.
struct ParsedGitUrl {
    /// The host domain (e.g., "github.com", "gitlab.com").
    host: String,
    owner: String,
    repo: String,
    file_path: Option<String>,
}

/// Parse a git URL input into structured components.
///
/// Accepts:
/// - `owner/repo` (defaults to github.com)
/// - `https://github.com/owner/repo`
/// - `https://github.com/owner/repo/blob/main/path/to/SKILL.md`
/// - `https://gitlab.com/owner/repo`
///
/// Only GitHub and GitLab domains are allowed for security (SSRF prevention).
fn parse_git_url(input: &str) -> Result<ParsedGitUrl, String> {
    let trimmed = input.trim();

    // Handle short form: owner/repo (defaults to GitHub)
    if !trimmed.contains("://") && trimmed.matches('/').count() == 1 {
        let parts: Vec<&str> = trimmed.split('/').collect();
        return Ok(ParsedGitUrl {
            host: "github.com".to_string(),
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
            file_path: None,
        });
    }

    // Parse full URL
    let url = url::Url::parse(trimmed).map_err(|_| {
        "Invalid URL format. Use owner/repo or a full GitHub/GitLab URL.".to_string()
    })?;

    // Validate domain — only allow GitHub and GitLab
    let host = url.host_str().ok_or("URL must have a host")?.to_lowercase();
    if host != "github.com" && host != "gitlab.com" {
        return Err(format!(
            "Only github.com and gitlab.com are supported, got: {host}"
        ));
    }

    let segments: Vec<&str> = url
        .path_segments()
        .ok_or("Invalid URL path")?
        .filter(|s: &&str| !s.is_empty())
        .collect();

    if segments.len() < 2 {
        return Err("URL must contain at least owner/repo".to_string());
    }

    let owner = segments[0].to_string();
    let repo = segments[1].trim_end_matches(".git").to_string();

    // Extract path if present (after /blob/branch/ or /tree/branch/)
    let file_path = if segments.len() > 3 && (segments[2] == "blob" || segments[2] == "tree") {
        // segments: [owner, repo, "blob"|"tree", branch, ...path]
        Some(segments[4..].join("/"))
    } else {
        None
    };

    Ok(ParsedGitUrl {
        host,
        owner,
        repo,
        file_path,
    })
}

/// Classify a file path as "skill" or "agent" based on filename patterns.
fn classify_definition_file(path: &str) -> Option<&'static str> {
    let name = path.rsplit('/').next().unwrap_or(path);
    if name.eq_ignore_ascii_case("SKILL.md") {
        Some("skill")
    } else if name.ends_with(".agent.md") || name.eq_ignore_ascii_case("AGENT.md") {
        Some("agent")
    } else {
        None
    }
}

/// Progress update for git definition discovery.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitImportProgress {
    /// Total files to fetch.
    pub total: usize,
    /// Files fetched so far.
    pub fetched: usize,
    /// Current phase: "tree" (scanning repo) or "fetch" (downloading files).
    pub phase: String,
}

/// Fetch definition files (SKILL.md + *.agent.md) from a git repository URL.
///
/// Discovers both skills and agents. Optionally filter by `kind_filter`
/// ("skill" or "agent"); pass `None` to get both.
/// If `github_token` is provided, it's used for authenticated GitHub API requests.
/// `on_progress` is called with progress updates during fetching.
pub async fn fetch_git_definitions<F>(
    client: &Client,
    git_url: &str,
    kind_filter: Option<&str>,
    github_token: Option<&str>,
    on_progress: F,
) -> Result<Vec<GitSkillFile>, String>
where
    F: Fn(GitImportProgress),
{
    let parsed = parse_git_url(git_url)?;
    let repo_url = format!("https://{}/{}/{}", parsed.host, parsed.owner, parsed.repo);

    if let Some(path) = parsed.file_path {
        on_progress(GitImportProgress {
            total: 1,
            fetched: 0,
            phase: "fetch".to_string(),
        });
        let kind = classify_definition_file(&path).unwrap_or("skill");
        let content =
            fetch_github_file(client, &parsed.owner, &parsed.repo, &path, github_token).await?;
        on_progress(GitImportProgress {
            total: 1,
            fetched: 1,
            phase: "fetch".to_string(),
        });
        return Ok(vec![GitSkillFile {
            path,
            content,
            repo_url,
            kind: kind.to_string(),
        }]);
    }

    // Phase 1: Scan the repo tree
    on_progress(GitImportProgress {
        total: 0,
        fetched: 0,
        phase: "tree".to_string(),
    });

    let tree_url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/main?recursive=1",
        parsed.owner, parsed.repo
    );
    let mut req = client
        .get(&tree_url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Chuck-Desktop/0.1");
    if let Some(token) = github_token {
        req = req.header("Authorization", format!("token {token}"));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("GitHub tree API failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Could not list repository {}/{}  (HTTP {})",
            parsed.owner,
            parsed.repo,
            resp.status()
        ));
    }

    #[derive(Deserialize)]
    struct TreeItem {
        path: String,
        #[serde(rename = "type")]
        item_type: String,
    }
    #[derive(Deserialize)]
    struct TreeResponse {
        tree: Vec<TreeItem>,
    }

    let tree: TreeResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse tree response: {e}"))?;

    // Collect matching paths: SKILL.md + *.agent.md (cap at 50)
    let def_paths: Vec<(String, &str)> = tree
        .tree
        .iter()
        .filter(|t| t.item_type == "blob")
        .filter_map(|t| {
            let kind = classify_definition_file(&t.path)?;
            if let Some(filter) = kind_filter {
                if kind != filter {
                    return None;
                }
            }
            Some((t.path.clone(), kind))
        })
        .take(50)
        .collect();

    let kind_label = kind_filter.unwrap_or("definition");
    if def_paths.is_empty() {
        return Err(format!(
            "No {kind_label} files found in {}/{}",
            parsed.owner, parsed.repo
        ));
    }

    // Phase 2: Fetch file contents with progress
    let total = def_paths.len();
    on_progress(GitImportProgress {
        total,
        fetched: 0,
        phase: "fetch".to_string(),
    });

    let mut found = Vec::new();
    for (i, (path, kind)) in def_paths.into_iter().enumerate() {
        if let Ok(content) =
            fetch_github_file(client, &parsed.owner, &parsed.repo, &path, github_token).await
        {
            found.push(GitSkillFile {
                path,
                content,
                repo_url: repo_url.clone(),
                kind: kind.to_string(),
            });
        }
        on_progress(GitImportProgress {
            total,
            fetched: i + 1,
            phase: "fetch".to_string(),
        });
    }

    if found.is_empty() {
        Err(format!(
            "Found {kind_label} files but could not fetch their content in {}/{}",
            parsed.owner, parsed.repo
        ))
    } else {
        Ok(found)
    }
}

/// Fetch a single file from GitHub using the Contents API.
async fn fetch_github_file(
    client: &Client,
    owner: &str,
    repo: &str,
    path: &str,
    github_token: Option<&str>,
) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");

    let mut req = client
        .get(&url)
        .header("Accept", "application/vnd.github.raw+json")
        .header("User-Agent", "Chuck-Desktop/0.1");
    if let Some(token) = github_token {
        req = req.header("Authorization", format!("token {token}"));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("GitHub API request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("File not found: {path} (HTTP {})", resp.status()));
    }

    resp.text()
        .await
        .map_err(|e| format!("Failed to read file content: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_url_short_form() {
        let parsed = parse_git_url("octocat/hello-world").unwrap();
        assert_eq!(parsed.host, "github.com");
        assert_eq!(parsed.owner, "octocat");
        assert_eq!(parsed.repo, "hello-world");
        assert!(parsed.file_path.is_none());
    }

    #[test]
    fn test_parse_git_url_github() {
        let parsed = parse_git_url("https://github.com/octocat/hello-world").unwrap();
        assert_eq!(parsed.host, "github.com");
        assert_eq!(parsed.owner, "octocat");
        assert_eq!(parsed.repo, "hello-world");
        assert!(parsed.file_path.is_none());
    }

    #[test]
    fn test_parse_git_url_with_path() {
        let parsed = parse_git_url(
            "https://github.com/octocat/hello-world/blob/main/skills/review/SKILL.md",
        )
        .unwrap();
        assert_eq!(parsed.owner, "octocat");
        assert_eq!(parsed.repo, "hello-world");
        assert_eq!(parsed.file_path, Some("skills/review/SKILL.md".to_string()));
    }

    #[test]
    fn test_parse_git_url_dot_git() {
        let parsed = parse_git_url("https://github.com/octocat/hello-world.git").unwrap();
        assert_eq!(parsed.repo, "hello-world");
        assert_eq!(parsed.owner, "octocat");
    }

    #[test]
    fn test_parse_git_url_gitlab() {
        let parsed = parse_git_url("https://gitlab.com/mygroup/myproject").unwrap();
        assert_eq!(parsed.host, "gitlab.com");
        assert_eq!(parsed.owner, "mygroup");
        assert_eq!(parsed.repo, "myproject");
    }

    #[test]
    fn test_parse_git_url_rejects_unknown_domains() {
        assert!(parse_git_url("https://evil.com/owner/repo").is_err());
        assert!(parse_git_url("https://bitbucket.org/owner/repo").is_err());
    }

    #[test]
    fn test_parse_git_url_invalid() {
        assert!(parse_git_url("not-a-url").is_err());
        assert!(parse_git_url("https://github.com/").is_err());
    }

    #[test]
    fn test_parse_git_url_with_whitespace() {
        let parsed = parse_git_url("  octocat/hello-world  ").unwrap();
        assert_eq!(parsed.owner, "octocat");
        assert_eq!(parsed.repo, "hello-world");
    }
}
