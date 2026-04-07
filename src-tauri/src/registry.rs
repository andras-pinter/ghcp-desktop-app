//! Registry client for browsing and installing skills/agents from
//! skills.sh and aitmpl.com, plus importing from git URLs.

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
    /// URL to the skill/agent on the registry.
    pub url: Option<String>,
    /// Install count (if available).
    pub installs: Option<u64>,
    /// Whether this is a skill or agent template.
    pub kind: RegistryItemKind,
    /// GitHub owner/repo for content fetching (skills.sh `source` field).
    pub source_repo: Option<String>,
}

/// Which registry a result came from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegistrySource {
    SkillsSh,
    Aitmpl,
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

// ── skills.sh API ──────────────────────────────────────────────

/// Raw response from skills.sh /api/search.
#[derive(Debug, Deserialize)]
struct SkillsShSearchResponse {
    skills: Vec<SkillsShItem>,
}

#[derive(Debug, Deserialize)]
struct SkillsShItem {
    id: String,
    name: String,
    #[serde(default)]
    installs: Option<u64>,
    #[serde(default)]
    source: Option<String>,
}

/// Search skills.sh for skills matching a query.
pub async fn search_skills_sh(
    client: &Client,
    query: &str,
    limit: u32,
) -> Result<Vec<RegistryItem>, String> {
    let url = format!(
        "https://skills.sh/api/search?q={}&limit={}",
        urlencoding::encode(query),
        limit
    );

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("skills.sh request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("skills.sh returned {}", resp.status()));
    }

    let body: SkillsShSearchResponse = resp
        .json()
        .await
        .map_err(|e| format!("skills.sh response parse failed: {e}"))?;

    Ok(body
        .skills
        .into_iter()
        .map(|s| {
            // source_repo is the GitHub owner/repo (e.g. "anthropics/skills")
            let source_repo = s.source.clone();
            RegistryItem {
                url: Some(format!("https://skills.sh/{}", s.id)),
                id: s.id.clone(),
                name: s.name,
                description: s.source,
                source: RegistrySource::SkillsSh,
                installs: s.installs,
                kind: RegistryItemKind::Skill,
                source_repo,
            }
        })
        .collect())
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

    // Collect all items with a relevance score
    let mut scored: Vec<(RegistryItem, u32)> = Vec::new();

    for item in &data.agents {
        let score = match_score(&item.name, item.category.as_deref(), &query_words);
        if score > 0 {
            scored.push((
                aitmpl_to_registry_item(item, RegistryItemKind::Agent),
                score,
            ));
        }
    }
    for item in &data.skills {
        let score = match_score(&item.name, item.category.as_deref(), &query_words);
        if score > 0 {
            scored.push((
                aitmpl_to_registry_item(item, RegistryItemKind::Skill),
                score,
            ));
        }
    }

    // Sort by score descending, then by name
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));
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
        url,
        installs: None,
        kind,
        source_repo: None,
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
        let end = cleaned[..200].rfind(' ').unwrap_or(200);
        format!("{}…", &cleaned[..end])
    } else {
        cleaned
    })
}

// ── Unified search ─────────────────────────────────────────────

/// Search both registries and return unified results.
pub async fn search_registries(
    client: &Client,
    query: &str,
    limit: u32,
) -> Result<RegistrySearchResult, String> {
    let (skills_sh, aitmpl) = tokio::join!(
        search_skills_sh(client, query, limit),
        search_aitmpl(client, query, limit),
    );

    let mut items = Vec::new();
    match skills_sh {
        Ok(results) => items.extend(results),
        Err(e) => log::warn!("skills.sh search failed: {e}"),
    }
    match aitmpl {
        Ok(results) => items.extend(results),
        Err(e) => log::warn!("aitmpl.com search failed: {e}"),
    }

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
/// For skills.sh: fetches SKILL.md from the item's source GitHub repo.
/// For aitmpl.com: fetches from components.json content field.
pub async fn fetch_skill_content(
    client: &Client,
    skill_id: &str,
    source: &RegistrySource,
    source_repo: Option<&str>,
) -> Result<String, String> {
    match source {
        RegistrySource::SkillsSh => {
            // The source_repo field contains the GitHub owner/repo path
            // e.g. "github/awesome-copilot". Content is at:
            //   raw.githubusercontent.com/{source_repo}/main/skills/{skill_name}/SKILL.md
            // The skill_id is e.g. "github/awesome-copilot/python-mcp-server-generator"
            // so the skill name is the last segment after stripping the source prefix.
            if let Some(repo) = source_repo {
                let skill_name = skill_id
                    .strip_prefix(repo)
                    .and_then(|s| s.strip_prefix('/'))
                    .unwrap_or(skill_id);

                let url = format!(
                    "https://raw.githubusercontent.com/{repo}/main/skills/{skill_name}/SKILL.md",
                );
                let resp = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to fetch SKILL.md: {e}"))?;

                if resp.status().is_success() {
                    return resp
                        .text()
                        .await
                        .map_err(|e| format!("Failed to read content: {e}"));
                }
                log::warn!("skills.sh primary URL returned {}: {url}", resp.status());
            }

            // Fallback: try the skill_id's last segment directly
            let skill_name = skill_id.rsplit('/').next().unwrap_or(skill_id);
            let fallback_url = format!(
                "https://raw.githubusercontent.com/nicepkg/nice-copilot-skills/main/skills/{skill_name}/SKILL.md",
            );
            let resp = client
                .get(&fallback_url)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch from GitHub: {e}"))?;

            if resp.status().is_success() {
                resp.text()
                    .await
                    .map_err(|e| format!("Failed to read content: {e}"))
            } else {
                Err(format!("Skill content not found (HTTP {})", resp.status()))
            }
        }
        RegistrySource::Aitmpl => {
            // Fetch components.json and find the item by name
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

            // Search in agents first, then skills — match by path-based ID or name
            for item in data.agents.iter().chain(data.skills.iter()) {
                let item_id_from_path = item
                    .path
                    .as_ref()
                    .map(|p| p.replace(".md", "").replace('/', "-"))
                    .unwrap_or_else(|| item.name.clone());

                if item_id_from_path == skill_id || item.name == skill_id {
                    if let Some(content) = &item.content {
                        return Ok(content.clone());
                    }
                }
            }

            Err(format!("Item '{skill_id}' not found in aitmpl.com catalog"))
        }
    }
}

// ── Git URL Import ─────────────────────────────────────────────

/// A discovered SKILL.md file from a git repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSkillFile {
    /// Path within the repository.
    pub path: String,
    /// The raw SKILL.md content.
    pub content: String,
    /// The repo URL it came from.
    pub repo_url: String,
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

/// Fetch SKILL.md files from a git repository URL.
///
/// Discovers SKILL.md files in standard locations or at a specific path.
pub async fn fetch_git_skills(client: &Client, git_url: &str) -> Result<Vec<GitSkillFile>, String> {
    let parsed = parse_git_url(git_url)?;
    let repo_url = format!("https://{}/{}/{}", parsed.host, parsed.owner, parsed.repo);

    if let Some(path) = parsed.file_path {
        // Fetch specific file
        let content = fetch_github_file(client, &parsed.owner, &parsed.repo, &path).await?;
        return Ok(vec![GitSkillFile {
            path,
            content,
            repo_url,
        }]);
    }

    // Try standard locations for SKILL.md files
    let search_paths = vec!["SKILL.md", "skills", ".agents/skills", ".copilot/skills"];

    let mut found = Vec::new();

    for search_path in search_paths {
        if search_path.contains('.') {
            // Direct file path
            if let Ok(content) =
                fetch_github_file(client, &parsed.owner, &parsed.repo, search_path).await
            {
                found.push(GitSkillFile {
                    path: search_path.to_string(),
                    content,
                    repo_url: repo_url.clone(),
                });
            }
        } else {
            // Directory — list contents
            if let Ok(entries) =
                list_github_dir(client, &parsed.owner, &parsed.repo, search_path).await
            {
                for entry in entries {
                    if entry.name.ends_with(".md")
                        && (entry.name == "SKILL.md" || entry.name.to_lowercase().contains("skill"))
                    {
                        if let Ok(content) =
                            fetch_github_file(client, &parsed.owner, &parsed.repo, &entry.path)
                                .await
                        {
                            found.push(GitSkillFile {
                                path: entry.path,
                                content,
                                repo_url: repo_url.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    if found.is_empty() {
        Err(format!(
            "No SKILL.md files found in {}/{}. Searched: SKILL.md, skills/, .agents/skills/, .copilot/skills/",
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
) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github.raw+json")
        .header("User-Agent", "Chuck-Desktop/0.1")
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

/// A file/directory entry from the GitHub Contents API.
#[derive(Debug, Deserialize)]
struct GitHubEntry {
    name: String,
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
}

/// List directory contents from GitHub.
async fn list_github_dir(
    client: &Client,
    owner: &str,
    repo: &str,
    path: &str,
) -> Result<Vec<GitHubEntry>, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Chuck-Desktop/0.1")
        .send()
        .await
        .map_err(|e| format!("GitHub API request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Directory not found: {path}"));
    }

    let entries: Vec<GitHubEntry> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse directory listing: {e}"))?;

    Ok(entries
        .into_iter()
        .filter(|e| e.entry_type == "file")
        .collect())
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
