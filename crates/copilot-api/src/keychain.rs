//! Secure token and API key storage.
//!
//! In debug builds, uses a plain-text file in the system temp dir to avoid
//! macOS Keychain password prompts (unsigned/recompiled binaries trigger these).
//! In release builds, uses the OS keychain via the `keyring` crate.

use thiserror::Error;

#[cfg(not(debug_assertions))]
const SERVICE_NAME: &str = "chuck";

/// Errors that can occur during keychain operations.
#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Failed to store secret: {0}")]
    Store(String),
    #[error("Failed to retrieve secret: {0}")]
    Retrieve(String),
    #[error("Failed to delete secret: {0}")]
    Delete(String),
    #[error("Secret not found for key: {0}")]
    NotFound(String),
}

// ── Debug builds: file-based storage ────────────────────────────

#[cfg(debug_assertions)]
fn debug_credential_path(key: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("chuck-dev-credentials");
    if std::fs::create_dir_all(&dir).is_ok() {
        // Restrict directory to owner-only access (0700)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
    }
    dir.join(key)
}

#[cfg(debug_assertions)]
pub fn store(key: &str, value: &str) -> Result<(), KeychainError> {
    let path = debug_credential_path(key);
    std::fs::write(&path, value).map_err(|e| KeychainError::Store(e.to_string()))?;
    // Restrict file to owner-only read/write (0600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

#[cfg(debug_assertions)]
pub fn retrieve(key: &str) -> Result<String, KeychainError> {
    let path = debug_credential_path(key);
    match std::fs::read_to_string(&path) {
        Ok(s) if !s.is_empty() => Ok(s),
        Ok(_) => Err(KeychainError::NotFound(key.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(KeychainError::NotFound(key.to_string()))
        }
        Err(e) => Err(KeychainError::Retrieve(e.to_string())),
    }
}

#[cfg(debug_assertions)]
pub fn delete(key: &str) -> Result<(), KeychainError> {
    let path = debug_credential_path(key);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(KeychainError::Delete(e.to_string())),
    }
}

// ── Release builds: OS keychain ─────────────────────────────────

#[cfg(not(debug_assertions))]
pub fn store(key: &str, value: &str) -> Result<(), KeychainError> {
    let entry =
        keyring::Entry::new(SERVICE_NAME, key).map_err(|e| KeychainError::Store(e.to_string()))?;
    entry
        .set_password(value)
        .map_err(|e| KeychainError::Store(e.to_string()))
}

#[cfg(not(debug_assertions))]
pub fn retrieve(key: &str) -> Result<String, KeychainError> {
    let entry = keyring::Entry::new(SERVICE_NAME, key)
        .map_err(|e| KeychainError::Retrieve(e.to_string()))?;
    entry.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => KeychainError::NotFound(key.to_string()),
        other => KeychainError::Retrieve(other.to_string()),
    })
}

#[cfg(not(debug_assertions))]
pub fn delete(key: &str) -> Result<(), KeychainError> {
    let entry =
        keyring::Entry::new(SERVICE_NAME, key).map_err(|e| KeychainError::Delete(e.to_string()))?;
    entry
        .delete_credential()
        .map_err(|e| KeychainError::Delete(e.to_string()))
}
