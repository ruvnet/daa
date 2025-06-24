//! Error types for the vault library.

use thiserror::Error;

/// Result type alias for vault operations.
pub type VaultResult<T> = Result<T, VaultError>;

/// Errors that can occur during vault operations.
#[derive(Error, Debug)]
pub enum VaultError {
    /// Cryptographic operation failed.
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// DAG operation failed.
    #[error("DAG operation error: {0}")]
    DagOperation(String),

    /// Key derivation failed.
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    /// IO operation failed.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid password provided.
    #[error("Invalid password")]
    InvalidPassword,

    /// Secret not found.
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    /// Vault already exists.
    #[error("Vault already exists at path: {0}")]
    VaultExists(String),

    /// Vault not found.
    #[error("Vault not found at path: {0}")]
    VaultNotFound(String),

    /// Invalid vault format or corrupted data.
    #[error("Invalid vault format: {0}")]
    InvalidFormat(String),

    /// Feature not implemented.
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    /// Generic vault error.
    #[error("Vault error: {0}")]
    Generic(String),
}

// Implement conversions from QuDAG errors
impl From<qudag_crypto::CryptoError> for VaultError {
    fn from(err: qudag_crypto::CryptoError) -> Self {
        VaultError::Crypto(err.to_string())
    }
}

impl From<bincode::Error> for VaultError {
    fn from(err: bincode::Error) -> Self {
        VaultError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(err: serde_json::Error) -> Self {
        VaultError::Serialization(err.to_string())
    }
}

impl From<argon2::Error> for VaultError {
    fn from(err: argon2::Error) -> Self {
        VaultError::KeyDerivation(err.to_string())
    }
}
