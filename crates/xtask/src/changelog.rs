use std::fmt::Write;
use std::fs;
use std::process::Command;

use crate::version::{project_root, read_version, VERSION_FILES};

/// Generate or update CHANGELOG.md from conventional commits.
pub fn run() -> Result<(), String> {
    let root = project_root()?;
    let current_version = read_version(
        &root,
        VERSION_FILES.first().ok_or("VERSION_FILES is empty")?,
    )?;

    // Find the latest tag to use as the range start
    let latest_tag = get_latest_tag()?;
    let range = match &latest_tag {
        Some(tag) => format!("{tag}..HEAD"),
        None => "HEAD".to_string(),
    };

    let commits = get_conventional_commits(&range)?;
    if commits.is_empty() {
        println!(
            "No conventional commits found since {}",
            latest_tag.as_deref().unwrap_or("beginning")
        );
        return Ok(());
    }

    let section = format_changelog_section(&current_version.to_string(), &commits);

    let changelog_path = root.join("CHANGELOG.md");
    let existing = fs::read_to_string(&changelog_path).unwrap_or_default();

    let new_content = if existing.is_empty() {
        format!(
            "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\
             Format follows [Keep a Changelog](https://keepachangelog.com/).\n\n{section}"
        )
    } else {
        // Insert the new section after the header (after first blank line following "# Changelog")
        insert_after_header(&existing, &section)
    };

    fs::write(&changelog_path, new_content)
        .map_err(|e| format!("Cannot write CHANGELOG.md: {e}"))?;

    println!("✓ CHANGELOG.md updated for v{current_version}");
    println!("  {} commit(s) categorized", commits.len());

    Ok(())
}

struct ConventionalCommit {
    commit_type: String,
    scope: Option<String>,
    description: String,
    hash: String,
}

fn get_latest_tag() -> Result<Option<String>, String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    if output.status.success() {
        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if tag.is_empty() { None } else { Some(tag) })
    } else {
        Ok(None) // No tags exist yet
    }
}

fn get_conventional_commits(range: &str) -> Result<Vec<ConventionalCommit>, String> {
    let output = Command::new("git")
        .args(["log", range, "--pretty=format:%h %s"])
        .output()
        .map_err(|e| format!("Failed to run git log: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git log failed: {stderr}"));
    }

    let log = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in log.lines() {
        if let Some(commit) = parse_conventional_commit(line) {
            commits.push(commit);
        }
    }

    Ok(commits)
}

fn parse_conventional_commit(line: &str) -> Option<ConventionalCommit> {
    let (hash, subject) = line.split_once(' ')?;

    // Match: type(scope): description  OR  type: description
    // Also handle type! for breaking changes
    let subject = subject.trim();
    let paren_idx = subject.find('(');
    let colon_idx = subject.find(':')?;

    // Type must come before colon
    let (commit_type, scope) = if let Some(pi) = paren_idx {
        if pi < colon_idx {
            let close = subject.find(')')?;
            let t = &subject[..pi];
            let s = &subject[pi + 1..close];
            (t.trim_end_matches('!'), Some(s.to_string()))
        } else {
            let t = &subject[..colon_idx];
            (t.trim_end_matches('!'), None)
        }
    } else {
        let t = &subject[..colon_idx];
        (t.trim_end_matches('!'), None)
    };

    // Only keep known conventional commit types
    let known_types = [
        "feat", "fix", "refactor", "docs", "style", "test", "chore", "perf", "ci", "build",
    ];
    if !known_types.contains(&commit_type) {
        return None;
    }

    let description = subject[colon_idx + 1..].trim().to_string();

    Some(ConventionalCommit {
        commit_type: commit_type.to_string(),
        scope,
        description,
        hash: hash.to_string(),
    })
}

fn format_changelog_section(version: &str, commits: &[ConventionalCommit]) -> String {
    let date = chrono_today();
    let mut out = String::new();
    writeln!(out, "## [{version}] — {date}").unwrap();
    writeln!(out).unwrap();

    let categories: &[(&str, &str)] = &[
        ("feat", "Features"),
        ("fix", "Bug Fixes"),
        ("perf", "Performance"),
        ("refactor", "Refactoring"),
        ("docs", "Documentation"),
        ("test", "Tests"),
        ("chore", "Chores"),
        ("ci", "CI/CD"),
        ("build", "Build"),
        ("style", "Style"),
    ];

    for (type_key, heading) in categories {
        let matching: Vec<&ConventionalCommit> = commits
            .iter()
            .filter(|c| c.commit_type == *type_key)
            .collect();
        if matching.is_empty() {
            continue;
        }

        writeln!(out, "### {heading}").unwrap();
        writeln!(out).unwrap();
        for c in matching {
            let scope_str = c
                .scope
                .as_ref()
                .map(|s| format!("**{s}:** "))
                .unwrap_or_default();
            writeln!(out, "- {scope_str}{} ({})", c.description, c.hash).unwrap();
        }
        writeln!(out).unwrap();
    }

    out
}

fn insert_after_header(existing: &str, section: &str) -> String {
    let lines: Vec<&str> = existing.lines().collect();

    // Find insertion point: right after the header block, before the first ## version section.
    // Header block = "# Changelog" line + any description lines + blank line(s).
    let mut insert_pos = lines.len();
    let mut past_title = false;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") {
            past_title = true;
            continue;
        }
        if !past_title {
            continue;
        }
        // First existing version section — insert right before it
        if line.starts_with("## ") {
            insert_pos = i;
            break;
        }
    }

    let mut out = String::new();
    for line in &lines[..insert_pos] {
        writeln!(out, "{line}").unwrap();
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    out.push_str(section);
    for line in &lines[insert_pos..] {
        writeln!(out, "{line}").unwrap();
    }

    out
}

fn chrono_today() -> String {
    // Use git to get current date in YYYY-MM-DD format (avoids adding chrono as a dep)
    let output = Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    output.unwrap_or_else(|| "YYYY-MM-DD".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_feat_with_scope() {
        let c = parse_conventional_commit("abc1234 feat(ui): add dark mode").unwrap();
        assert_eq!(c.commit_type, "feat");
        assert_eq!(c.scope.as_deref(), Some("ui"));
        assert_eq!(c.description, "add dark mode");
        assert_eq!(c.hash, "abc1234");
    }

    #[test]
    fn parse_fix_no_scope() {
        let c = parse_conventional_commit("def5678 fix: prevent crash on empty input").unwrap();
        assert_eq!(c.commit_type, "fix");
        assert_eq!(c.scope, None);
        assert_eq!(c.description, "prevent crash on empty input");
    }

    #[test]
    fn parse_breaking_change() {
        let c = parse_conventional_commit("aaa1111 feat!: redesign API").unwrap();
        assert_eq!(c.commit_type, "feat");
        assert_eq!(c.description, "redesign API");
    }

    #[test]
    fn parse_non_conventional_returns_none() {
        assert!(parse_conventional_commit("bbb2222 Update README").is_none());
    }

    #[test]
    fn parse_unknown_type_returns_none() {
        assert!(parse_conventional_commit("ccc3333 yolo: do stuff").is_none());
    }
}
