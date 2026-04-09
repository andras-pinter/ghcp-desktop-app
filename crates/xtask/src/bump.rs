use semver::Version;

use crate::version::{project_root, read_version, write_version, VERSION_FILES};

/// Bump the version across all project files.
pub fn run(level: &str) -> Result<(), String> {
    let root = project_root()?;

    // Read current version from the workspace Cargo.toml (source of truth)
    let source = VERSION_FILES.first().ok_or("VERSION_FILES is empty")?;
    let current = read_version(&root, source)?;
    let next = bump_version(&current, level)?;

    println!("Bumping version: {current} → {next}");
    println!();

    for vf in VERSION_FILES {
        write_version(&root, vf, &next)?;
        println!("  ✓ {vf}");
    }

    println!();
    println!("Version bumped to {next}");
    println!();
    println!("Next steps:");
    println!("  1. Review changes:     git diff");
    println!("  2. Generate changelog: cargo xtask changelog");
    println!("  3. Commit:             git commit -am \"chore: release v{next}\"");
    println!("  4. Tag:                git tag v{next}");

    Ok(())
}

fn bump_version(current: &Version, level: &str) -> Result<Version, String> {
    let mut next = current.clone();
    match level {
        "major" => {
            next.major += 1;
            next.minor = 0;
            next.patch = 0;
            next.pre = semver::Prerelease::EMPTY;
        }
        "minor" => {
            next.minor += 1;
            next.patch = 0;
            next.pre = semver::Prerelease::EMPTY;
        }
        "patch" => {
            next.patch += 1;
            next.pre = semver::Prerelease::EMPTY;
        }
        other => return Err(format!("Unknown bump level: {other}")),
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_patch() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(
            bump_version(&v, "patch").unwrap(),
            Version::parse("1.2.4").unwrap()
        );
    }

    #[test]
    fn bump_minor() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(
            bump_version(&v, "minor").unwrap(),
            Version::parse("1.3.0").unwrap()
        );
    }

    #[test]
    fn bump_major() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(
            bump_version(&v, "major").unwrap(),
            Version::parse("2.0.0").unwrap()
        );
    }

    #[test]
    fn bump_clears_prerelease() {
        let v = Version::parse("1.0.0-beta.1").unwrap();
        assert_eq!(
            bump_version(&v, "patch").unwrap(),
            Version::parse("1.0.1").unwrap()
        );
    }

    #[test]
    fn bump_invalid_level() {
        let v = Version::parse("1.0.0").unwrap();
        assert!(bump_version(&v, "huge").is_err());
    }
}
