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
        .map(|s| RegistryItem {
            url: Some(format!("https://skills.sh/{}", s.id)),
            id: s.id.clone(),
            name: s.name,
            description: s.source,
            source: RegistrySource::SkillsSh,
            installs: s.installs,
            kind: RegistryItemKind::Skill,
        })
        .collect())
}

// ── aitmpl.com ─────────────────────────────────────────────────

/// Search aitmpl.com for agents/skills.
///
/// Note: aitmpl.com is a JS-rendered site. We attempt their search API
/// if available, otherwise return a link to the website.
pub async fn search_aitmpl(
    client: &Client,
    query: &str,
    limit: u32,
) -> Result<Vec<RegistryItem>, String> {
    // Try the /api/search endpoint (undocumented, may not exist)
    let url = format!(
        "https://aitmpl.com/api/search?q={}&limit={}",
        urlencoding::encode(query),
        limit
    );

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            // Try to parse as JSON array of items
            if let Ok(items) = r.json::<Vec<AitmplItem>>().await {
                return Ok(items
                    .into_iter()
                    .map(|item| RegistryItem {
                        url: item.url,
                        id: item.slug.unwrap_or_else(|| item.name.clone()),
                        name: item.name,
                        description: item.description,
                        source: RegistrySource::Aitmpl,
                        installs: None,
                        kind: if item.kind.as_deref() == Some("agent") {
                            RegistryItemKind::Agent
                        } else {
                            RegistryItemKind::Skill
                        },
                    })
                    .collect());
            }
        }
        _ => {}
    }

    // Fallback: return empty with a note (site may need browser access)
    log::warn!("aitmpl.com API not available, returning empty results");
    Ok(vec![])
}

#[derive(Debug, Deserialize)]
struct AitmplItem {
    name: String,
    slug: Option<String>,
    description: Option<String>,
    url: Option<String>,
    kind: Option<String>,
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

// ── Fetch SKILL.md content from registry ───────────────────────

/// Fetch the raw SKILL.md content for a skills.sh skill by ID.
pub async fn fetch_skill_content(
    client: &Client,
    skill_id: &str,
    source: &RegistrySource,
) -> Result<String, String> {
    match source {
        RegistrySource::SkillsSh => {
            // skills.sh hosts SKILL.md on GitHub, try raw content
            let encoded_id = urlencoding::encode(skill_id);
            let url = format!("https://skills.sh/api/skill/{encoded_id}");
            let resp = client
                .get(&url)
                .header("Accept", "text/plain")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch skill content: {e}"))?;

            if resp.status().is_success() {
                return resp
                    .text()
                    .await
                    .map_err(|e| format!("Failed to read skill content: {e}"));
            }

            // Fallback: try GitHub raw
            let encoded_id = urlencoding::encode(skill_id);
            let github_url = format!(
                "https://raw.githubusercontent.com/nicepkg/nice-copilot-skills/main/skills/{encoded_id}/SKILL.md"
            );
            let resp = client
                .get(&github_url)
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
            Err("Direct SKILL.md fetch not yet supported for aitmpl.com".to_string())
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
