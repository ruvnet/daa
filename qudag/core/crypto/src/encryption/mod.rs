use std::error::Error;
use std::fmt;

/// Error type for encryption operations
#[derive(Debug)]
pub enum EncryptionError {
    /// Error during key generation
    KeyGenError(String),
    /// Error during encryption
    EncryptError(String),
    /// Error during decryption
    DecryptError(String),
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionError::KeyGenError(msg) => write!(f, "Key generation error: {}", msg),
            EncryptionError::EncryptError(msg) => write!(f, "Encryption error: {}", msg),
            EncryptionError::DecryptError(msg) => write!(f, "Decryption error: {}", msg),
        }
    }
}

impl Error for EncryptionError {}

// HQC module removed - use the main hqc module in crypto instead
