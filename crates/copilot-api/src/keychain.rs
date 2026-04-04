//! Secure token and API key storage via OS keychain.

use thiserror::Error;

const SERVICE_NAME: &str = "copilot-desktop";

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

/// Store a secret in the OS keychain.
pub fn store(key: &str, value: &str) -> Result<(), KeychainError> {
    let entry =
        keyring::Entry::new(SERVICE_NAME, key).map_err(|e| KeychainError::Store(e.to_string()))?;
    entry
        .set_password(value)
        .map_err(|e| KeychainError::Store(e.to_string()))
}

/// Retrieve a secret from the OS keychain.
pub fn retrieve(key: &str) -> Result<String, KeychainError> {
    let entry = keyring::Entry::new(SERVICE_NAME, key)
        .map_err(|e| KeychainError::Retrieve(e.to_string()))?;
    entry.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => KeychainError::NotFound(key.to_string()),
        other => KeychainError::Retrieve(other.to_string()),
    })
}

/// Delete a secret from the OS keychain.
pub fn delete(key: &str) -> Result<(), KeychainError> {
    let entry =
        keyring::Entry::new(SERVICE_NAME, key).map_err(|e| KeychainError::Delete(e.to_string()))?;
    entry
        .delete_credential()
        .map_err(|e| KeychainError::Delete(e.to_string()))
}
