use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use semver::Version;

/// All files that contain the project version.
pub const VERSION_FILES: &[VersionFile] = &[
    VersionFile {
        relative_path: "Cargo.toml",
        format: FileFormat::CargoWorkspace,
    },
    VersionFile {
        relative_path: "package.json",
        format: FileFormat::Json,
    },
    VersionFile {
        relative_path: "src-tauri/tauri.conf.json",
        format: FileFormat::Json,
    },
];

pub struct VersionFile {
    pub relative_path: &'static str,
    pub format: FileFormat,
}

pub enum FileFormat {
    /// Root `Cargo.toml` — version lives at `[workspace.package] version`
    CargoWorkspace,
    /// JSON file — version at top-level `"version"` key
    Json,
}

/// Find the project root by walking up from the current dir looking for `Cargo.toml`
/// with a `[workspace]` table.
pub fn project_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|e| format!("Cannot get current dir: {e}"))?;
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() {
            let content = fs::read_to_string(&candidate)
                .map_err(|e| format!("Cannot read {candidate:?}: {e}"))?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            return Err("Could not find workspace root (no Cargo.toml with [workspace])".into());
        }
    }
}

/// Read the version from a specific file.
pub fn read_version(root: &Path, vf: &VersionFile) -> Result<Version, String> {
    let path = root.join(vf.relative_path);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Cannot read {}: {e}", vf.relative_path))?;

    let raw = match vf.format {
        FileFormat::CargoWorkspace => {
            let doc = content
                .parse::<toml_edit::DocumentMut>()
                .map_err(|e| format!("Cannot parse {}: {e}", vf.relative_path))?;
            doc["workspace"]["package"]["version"]
                .as_str()
                .ok_or_else(|| {
                    format!("{}: missing [workspace.package] version", vf.relative_path)
                })?
                .to_string()
        }
        FileFormat::Json => {
            let json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Cannot parse {}: {e}", vf.relative_path))?;
            json["version"]
                .as_str()
                .ok_or_else(|| format!("{}: missing \"version\" key", vf.relative_path))?
                .to_string()
        }
    };

    Version::parse(&raw).map_err(|e| format!("{}: invalid semver '{}': {e}", vf.relative_path, raw))
}

/// Write a new version into a specific file, preserving formatting.
pub fn write_version(root: &Path, vf: &VersionFile, version: &Version) -> Result<(), String> {
    let path = root.join(vf.relative_path);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Cannot read {}: {e}", vf.relative_path))?;

    let new_content = match vf.format {
        FileFormat::CargoWorkspace => {
            let mut doc = content
                .parse::<toml_edit::DocumentMut>()
                .map_err(|e| format!("Cannot parse {}: {e}", vf.relative_path))?;
            doc["workspace"]["package"]["version"] = toml_edit::value(version.to_string());
            doc.to_string()
        }
        FileFormat::Json => {
            let mut json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Cannot parse {}: {e}", vf.relative_path))?;
            json["version"] = serde_json::Value::String(version.to_string());
            // Preserve 2-space indent + trailing newline (standard for this project)
            let mut out = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Cannot serialize {}: {e}", vf.relative_path))?;
            out.push('\n');
            out
        }
    };

    fs::write(&path, new_content).map_err(|e| format!("Cannot write {}: {e}", vf.relative_path))?;

    Ok(())
}

impl fmt::Display for VersionFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.relative_path)
    }
}
