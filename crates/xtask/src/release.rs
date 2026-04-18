use std::process::Command;

use crate::bump;
use crate::changelog::{self, get_conventional_commits, get_latest_tag};
use crate::check;
use crate::version::{project_root, read_version, VERSION_FILES};
/// Automated release: detect bump level → bump → changelog → commit → tag.
pub fn run(force_level: Option<&str>, dry_run: bool) -> Result<(), String> {
    let root = project_root()?;
    let source = VERSION_FILES.first().ok_or("VERSION_FILES is empty")?;
    let current = read_version(&root, source)?;

    // Ensure working tree is clean
    if !dry_run {
        ensure_clean_worktree()?;
    }

    // Gather commits since last tag
    let latest_tag = get_latest_tag()?;
    let range = match &latest_tag {
        Some(tag) => format!("{tag}..HEAD"),
        None => "HEAD".to_string(),
    };
    let commits = get_conventional_commits(&range)?;

    if commits.is_empty() {
        println!(
            "No conventional commits since {} — nothing to release.",
            latest_tag.as_deref().unwrap_or("beginning")
        );
        return Ok(());
    }

    // Auto-detect or use forced level
    let level = match force_level {
        Some(l) => l.to_string(),
        None => detect_bump_level(&commits),
    };

    if level == "none" {
        println!(
            "No version-bumping commits since {} ({} commit(s) found, but only docs/test/chore/style/ci/build).",
            latest_tag.as_deref().unwrap_or("beginning"),
            commits.len()
        );
        println!("Use `cargo xtask release --level patch` to force a release.");
        return Ok(());
    }

    println!("Detected bump level: {level}");
    println!(
        "  {} commit(s) since {}",
        commits.len(),
        latest_tag.as_deref().unwrap_or("initial commit")
    );
    println!();

    if dry_run {
        let preview = bump::bump_version(&current, &level)?;
        println!("[dry-run] Would bump: {current} → {preview}");
        println!("[dry-run] Would update CHANGELOG.md, commit, and tag v{preview}");
        return Ok(());
    }

    // 1. Bump version in all files
    bump::run(&level)?;
    println!();

    // Re-read the new version after bump
    let new_version = read_version(&root, source)?;

    // 2. Generate changelog (if this fails, print recovery help)
    if let Err(e) = changelog::run(None) {
        eprintln!();
        eprintln!("Changelog generation failed: {e}");
        eprintln!();
        eprintln!("Version files have already been bumped to v{new_version}.");
        eprintln!("To recover, run:");
        eprintln!("  git checkout -- Cargo.toml package.json src-tauri/tauri.conf.json Cargo.lock");
        return Err(e);
    }
    println!();

    // 3. Verify consistency
    check::run()?;
    println!();

    // 4. Commit everything
    let msg = format!("chore: release v{new_version}");
    git_cmd(&["add", "-A"])?;
    git_cmd(&["commit", "-m", &msg])?;
    println!("✓ Committed: {msg}");

    // 5. Tag
    let tag = format!("v{new_version}");
    git_cmd(&["tag", "-a", &tag, "-m", &format!("Release {tag}")])?;
    println!("✓ Tagged: {tag}");

    println!();
    println!("🎉 Release v{new_version} complete!");
    println!();
    println!("Next step:");
    println!("  git push && git push --tags");

    Ok(())
}

/// Determine bump level from conventional commits.
/// - Any `!` (breaking) → major
/// - Any `feat` → minor
/// - Any `fix`, `perf`, or `chore(deps)` → patch
/// - Everything else (docs, test, chore, style, ci, build, refactor) → none (no bump)
fn detect_bump_level(commits: &[changelog::ConventionalCommit]) -> String {
    if commits.iter().any(|c| c.is_breaking) {
        return "major".to_string();
    }
    if commits.iter().any(|c| c.commit_type == "feat") {
        return "minor".to_string();
    }
    let is_patch = |c: &changelog::ConventionalCommit| {
        let bump_types = ["fix", "perf"];
        if bump_types.contains(&c.commit_type.as_str()) {
            return true;
        }
        // Dependency updates (chore(deps), build(deps)) should trigger a patch release
        if c.scope.as_deref() == Some("deps") {
            return true;
        }
        false
    };
    if commits.iter().any(is_patch) {
        return "patch".to_string();
    }
    "none".to_string()
}

fn ensure_clean_worktree() -> Result<(), String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to run git status: {e}"))?;

    let status = String::from_utf8_lossy(&output.stdout);
    let dirty: Vec<&str> = status.lines().filter(|l| !l.trim().is_empty()).collect();

    if dirty.is_empty() {
        return Ok(());
    }

    // Allow lock file changes — the release commit (git add -A) will include them
    let only_lock_files = dirty.iter().all(|line| {
        let path = line.trim().trim_start_matches(|c: char| !c.is_whitespace());
        let path = path.trim();
        path == "Cargo.lock" || path == "pnpm-lock.yaml"
    });

    if only_lock_files {
        println!("ℹ  Lock file changes detected — will be included in release commit.");
        return Ok(());
    }

    Err(format!(
        "Working tree is not clean. Commit or stash changes before releasing.\nDirty files:\n{}",
        dirty.join("\n")
    ))
}

fn git_cmd(args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git {}: {e}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {stderr}", args.join(" ")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changelog::ConventionalCommit;

    fn commit(typ: &str, breaking: bool) -> ConventionalCommit {
        ConventionalCommit {
            commit_type: typ.to_string(),
            scope: None,
            description: "test".to_string(),
            hash: "abc1234".to_string(),
            is_breaking: breaking,
        }
    }

    fn commit_with_scope(typ: &str, scope: &str) -> ConventionalCommit {
        ConventionalCommit {
            commit_type: typ.to_string(),
            scope: Some(scope.to_string()),
            description: "test".to_string(),
            hash: "abc1234".to_string(),
            is_breaking: false,
        }
    }

    #[test]
    fn detect_patch_from_fixes() {
        let commits = vec![commit("fix", false), commit("chore", false)];
        assert_eq!(detect_bump_level(&commits), "patch");
    }

    #[test]
    fn detect_patch_from_perf() {
        let commits = vec![commit("perf", false), commit("docs", false)];
        assert_eq!(detect_bump_level(&commits), "patch");
    }

    #[test]
    fn detect_minor_from_feat() {
        let commits = vec![commit("fix", false), commit("feat", false)];
        assert_eq!(detect_bump_level(&commits), "minor");
    }

    #[test]
    fn detect_major_from_breaking() {
        let commits = vec![commit("feat", false), commit("fix", true)];
        assert_eq!(detect_bump_level(&commits), "major");
    }

    #[test]
    fn detect_none_from_non_bumping_types() {
        let commits = vec![
            commit("docs", false),
            commit("test", false),
            commit("chore", false),
            commit("style", false),
            commit("ci", false),
            commit("build", false),
            commit("refactor", false),
        ];
        assert_eq!(detect_bump_level(&commits), "none");
    }

    #[test]
    fn detect_none_from_docs_only() {
        let commits = vec![commit("docs", false)];
        assert_eq!(detect_bump_level(&commits), "none");
    }

    #[test]
    fn detect_patch_from_deps_scope() {
        let commits = vec![commit_with_scope("chore", "deps")];
        assert_eq!(detect_bump_level(&commits), "patch");
    }

    #[test]
    fn detect_patch_from_build_deps() {
        let commits = vec![commit_with_scope("build", "deps"), commit("docs", false)];
        assert_eq!(detect_bump_level(&commits), "patch");
    }

    #[test]
    fn detect_none_from_chore_non_deps() {
        let commits = vec![commit_with_scope("chore", "ci")];
        assert_eq!(detect_bump_level(&commits), "none");
    }
}
