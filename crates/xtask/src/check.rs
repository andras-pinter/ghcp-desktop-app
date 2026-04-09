use crate::version::{project_root, read_version, VERSION_FILES};

/// Verify that all version files contain the same version string.
pub fn run() -> Result<(), String> {
    let root = project_root()?;

    let mut versions = Vec::new();
    let mut errors = Vec::new();

    for vf in VERSION_FILES {
        match read_version(&root, vf) {
            Ok(v) => {
                println!("  {}: {v}", vf.relative_path);
                versions.push((vf.relative_path, v));
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("  ✗ {e}");
        }
        return Err(format!("{} file(s) could not be read", errors.len()));
    }

    if versions.is_empty() {
        return Err("No version files configured".to_string());
    }

    // Check all versions match the first (workspace Cargo.toml)
    let (source, expected) = &versions[0];
    let mut mismatches = Vec::new();

    for (path, version) in &versions[1..] {
        if version != expected {
            mismatches.push(format!("  ✗ {} has {version}, expected {expected}", path));
        }
    }

    if mismatches.is_empty() {
        println!();
        println!(
            "✓ All {count} files at version {expected} (source of truth: {source})",
            count = versions.len()
        );
        Ok(())
    } else {
        println!();
        for m in &mismatches {
            eprintln!("{m}");
        }
        Err(format!(
            "{} file(s) out of sync with {source} (v{expected})",
            mismatches.len()
        ))
    }
}
