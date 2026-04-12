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

/// Check if a paragraph looks like prose (not a table, rule, code, or markup).
fn is_prose_paragraph(text: &str) -> bool {
    let first_line = text.lines().next().unwrap_or("");
    let trimmed = first_line.trim();
    // Skip headings, tables, horizontal rules, HTML, code fences, list-only paras
    if trimmed.starts_with('#')
        || trimmed.starts_with('|')
        || trimmed.starts_with("```")
        || trimmed.starts_with('<')
        || trimmed.starts_with("---")
        || trimmed.starts_with("***")
        || trimmed.starts_with("___")
    {
        return false;
    }
    // Must contain some alphabetic content
    trimmed.chars().any(|c| c.is_alphabetic())
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

    // Find the first paragraph that looks like prose
    let first_para = body
        .split("\n\n")
        .map(|p| p.trim())
        .find(|p| !p.is_empty() && is_prose_paragraph(p))?;

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
            let lines: Vec<&str> = yaml_str.lines().collect();
            let mut i = 0;
            while i < lines.len() {
                let line = lines[i];
                if let Some(val) = line.strip_prefix("name:") {
                    name = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                }
                if desc.is_none() {
                    if let Some(val) = line.strip_prefix("description:") {
                        let trimmed = val.trim();
                        let raw = if trimmed.is_empty()
                            || trimmed == ">"
                            || trimmed == "|"
                            || trimmed == ">-"
                            || trimmed == "|-"
                        {
                            // Multi-line YAML: collect indented continuation lines
                            let mut parts = Vec::new();
                            while i + 1 < lines.len() {
                                let next = lines[i + 1];
                                if next.starts_with(' ') || next.starts_with('\t') {
                                    parts.push(next.trim());
                                    i += 1;
                                } else {
                                    break;
                                }
                            }
                            parts.join(" ")
                        } else {
                            trimmed.trim_matches('"').trim_matches('\'').to_string()
                        };
                        if !raw.is_empty() {
                            let clean = raw.lines().next().unwrap_or(&raw);
                            desc = Some(if clean.len() > 200 {
                                format!("{}…", &clean[..197])
                            } else {
                                clean.to_string()
                            });
                        }
                    }
                }
                i += 1;
            }

            // If no description in frontmatter, try extracting from the markdown body
            if desc.is_none() || desc.as_deref() == Some("") {
                desc = extract_body_description(content);
            }

            return (
                name.unwrap_or_else(|| readable_name_from_path(fallback_id)),
                desc.unwrap_or_default(),
                body.trim().to_string(),
            );
        }
    }

    // No frontmatter — try to extract description from body, use whole content as instructions
    let body_desc = extract_body_description(content).unwrap_or_default();
    (
        readable_name_from_path(fallback_id),
        body_desc,
        content.trim().to_string(),
    )
}

/// Derive a human-readable name from a file path.
///
/// Examples:
/// - `.github/agents/agentic-workflows.agent.md` → `Agentic Workflows`
/// - `skills/code-review/SKILL.md` → `Code Review`
/// - `my-skill.md` → `My Skill`
fn readable_name_from_path(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    // Strip known suffixes
    let stem = file
        .strip_suffix(".agent.md")
        .or_else(|| file.strip_suffix(".md"))
        .unwrap_or(file);
    // If the stem is a generic name like "SKILL" or "AGENT", use the parent dir
    let base = if stem.eq_ignore_ascii_case("SKILL") || stem.eq_ignore_ascii_case("AGENT") {
        path.trim_end_matches('/')
            .rsplit('/')
            .nth(1)
            .unwrap_or(stem)
    } else {
        stem
    };
    // Convert hyphens/underscores to spaces and title-case each word
    base.split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.as_str().to_lowercase();
                    format!("{upper}{rest}")
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

    // Collect all matching paths with a safety cap to prevent DoS from
    // malicious repos with thousands of definition files.
    const MAX_TOTAL_FILES: usize = 1000;
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
        .take(MAX_TOTAL_FILES)
        .collect();

    let kind_label = kind_filter.unwrap_or("definition");
    if def_paths.is_empty() {
        return Err(format!(
            "No {kind_label} files found in {}/{}",
            parsed.owner, parsed.repo
        ));
    }

    // Phase 2: Fetch file contents concurrently in batches with progress
    let total = def_paths.len();
    on_progress(GitImportProgress {
        total,
        fetched: 0,
        phase: "fetch".to_string(),
    });

    let mut found = Vec::new();
    let mut fetched = 0usize;
    const BATCH_SIZE: usize = 20;

    for batch in def_paths.chunks(BATCH_SIZE) {
        let mut handles = Vec::new();
        for (path, kind) in batch {
            let c = client.clone();
            let o = parsed.owner.clone();
            let r = parsed.repo.clone();
            let p = path.clone();
            let k = kind.to_string();
            let tok = github_token.map(|s| s.to_string());
            handles.push(tokio::spawn(async move {
                let result = fetch_github_file(&c, &o, &r, &p, tok.as_deref()).await;
                (p, k, result)
            }));
        }
        for handle in handles {
            if let Ok((path, kind, Ok(content))) = handle.await {
                found.push(GitSkillFile {
                    path,
                    content,
                    repo_url: repo_url.clone(),
                    kind,
                });
            }
            fetched += 1;
            on_progress(GitImportProgress {
                total,
                fetched,
                phase: "fetch".to_string(),
            });
        }
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

    let content = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read file content: {e}"))?;

    // Reject files larger than 512 KB to prevent memory exhaustion
    if content.len() > 512 * 1024 {
        return Err(format!(
            "File too large: {path} ({} KB, max 512 KB)",
            content.len() / 1024
        ));
    }

    Ok(content)
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

    #[test]
    fn test_parse_content_lenient_yaml_description() {
        let content =
            "---\nname: 'My Agent'\ndescription: Does cool things\n---\nInstructions here.\n";
        let (name, desc, body) = parse_content_lenient(content, "fallback");
        assert_eq!(name, "My Agent");
        assert_eq!(desc, "Does cool things");
        assert_eq!(body, "Instructions here.");
    }

    #[test]
    fn test_parse_content_lenient_strips_quotes() {
        let content = "---\nname: 'Quoted Name'\ndescription: \"Quoted desc\"\n---\nBody.\n";
        let (name, desc, _) = parse_content_lenient(content, "fallback");
        assert_eq!(name, "Quoted Name");
        assert_eq!(desc, "Quoted desc");
    }

    #[test]
    fn test_parse_content_lenient_body_description_fallback() {
        // No description in YAML — should fall back to body extraction
        let content = "---\nname: test-agent\n---\n\nThis agent helps with code review.\n\nIt checks for bugs.\n";
        let (name, desc, _) = parse_content_lenient(content, "fallback");
        assert_eq!(name, "test-agent");
        assert_eq!(desc, "This agent helps with code review.");
    }

    #[test]
    fn test_parse_content_lenient_no_frontmatter() {
        let content = "# My Agent\n\nThis is a simple agent.\n\nIt does things.\n";
        let (name, desc, _) = parse_content_lenient(content, "my-fallback");
        assert_eq!(name, "My Fallback");
        assert!(!desc.is_empty(), "Should extract body description");
    }

    #[test]
    fn test_parse_content_lenient_multiline_description() {
        let content = "---\nname: ml-agent\ndescription: >\n  This is a long\n  multi-line description\n---\nBody.\n";
        let (_, desc, _) = parse_content_lenient(content, "fallback");
        assert!(desc.contains("This is a long"));
        assert!(desc.contains("multi-line description"));
    }

    #[test]
    fn test_readable_name_from_path() {
        assert_eq!(
            readable_name_from_path(".github/agents/agentic-workflows.agent.md"),
            "Agentic Workflows"
        );
        assert_eq!(
            readable_name_from_path("skills/code-review/SKILL.md"),
            "Code Review"
        );
        assert_eq!(readable_name_from_path("my-skill.md"), "My Skill");
        assert_eq!(readable_name_from_path("AGENT.md"), "Agent");
        assert_eq!(
            readable_name_from_path("tools/web_scraper.agent.md"),
            "Web Scraper"
        );
    }

    #[test]
    fn test_parse_content_lenient_path_fallback_name() {
        // No name in YAML — should derive from path
        let content = "---\ndescription: Does things\n---\nBody.\n";
        let (name, _, _) =
            parse_content_lenient(content, ".github/agents/agentic-workflows.agent.md");
        assert_eq!(name, "Agentic Workflows");
    }

    #[test]
    fn test_extract_body_description_skips_tables() {
        let content = "---\nname: model-selector\n---\n\n| Model | Best for |\n| --- | --- |\n| gpt-4 | General |\n\nThis skill helps you choose models.\n";
        let desc = extract_body_description(content).unwrap();
        assert_eq!(desc, "This skill helps you choose models.");
    }

    #[test]
    fn test_extract_body_description_skips_rules() {
        let content = "---\nname: divider\n---\n\n---\n\nActual description here.\n";
        let desc = extract_body_description(content).unwrap();
        assert_eq!(desc, "Actual description here.");
    }

    #[test]
    fn test_extract_body_description_skips_code_fences() {
        let content = "---\nname: code-example\n---\n\n```json\n{\"key\": \"value\"}\n```\n\nThis does something useful.\n";
        let desc = extract_body_description(content).unwrap();
        assert_eq!(desc, "This does something useful.");
    }

    #[test]
    fn test_is_prose_paragraph() {
        assert!(is_prose_paragraph("This is a normal paragraph."));
        assert!(is_prose_paragraph("A skill for code review."));
        assert!(!is_prose_paragraph("| col1 | col2 |"));
        assert!(!is_prose_paragraph("---"));
        assert!(!is_prose_paragraph("```rust"));
        assert!(!is_prose_paragraph("# Heading"));
        assert!(!is_prose_paragraph("<div>html</div>"));
        assert!(!is_prose_paragraph("***"));
    }
}
