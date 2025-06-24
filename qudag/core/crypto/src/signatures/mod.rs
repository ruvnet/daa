pub mod ml_dsa;

use std::error::Error;
use std::fmt;

/// Error type for signature operations
#[derive(Debug)]
pub enum SignatureError {
    /// Error during key generation
    KeyGenError(String),
    /// Error during signing
    SignError(String),
    /// Error during signature verification
    VerifyError(String),
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignatureError::KeyGenError(msg) => write!(f, "Key generation error: {}", msg),
            SignatureError::SignError(msg) => write!(f, "Signing error: {}", msg),
            SignatureError::VerifyError(msg) => write!(f, "Verification error: {}", msg),
        }
    }
}

impl Error for SignatureError {}