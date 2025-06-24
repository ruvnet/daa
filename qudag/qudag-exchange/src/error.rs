//! Error types for QuDAG Exchange with security-focused error handling

use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Main error type for the exchange
#[derive(Error, Debug)]
pub enum ExchangeError {
    /// Cryptographic operation failed
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] qudag_crypto::CryptoError),

    /// Vault operation failed
    #[error("Vault error: {0}")]
    Vault(String),

    /// Network operation failed
    #[error("Network error: {0}")]
    Network(String),

    /// DAG consensus error
    #[error("DAG consensus error: {0}")]
    Consensus(String),

    /// Zero-knowledge proof verification failed
    #[error("ZKP verification failed: {0}")]
    ZkpVerification(String),

    /// Resource metering error
    #[error("Resource metering error: {0}")]
    Metering(String),

    /// Transaction validation failed
    #[error("Transaction validation failed: {0}")]
    TransactionValidation(String),

    /// Insufficient rUv credits
    #[error("Insufficient rUv credits: required {required}, available {available}")]
    InsufficientCredits {
        /// Required credits
        required: u64,
        /// Available credits
        available: u64,
    },

    /// Rate limiting exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Authentication failed
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// Authorization failed
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Timing attack detected
    #[error("Timing anomaly detected - potential attack")]
    TimingAnomaly,

    /// Replay attack detected
    #[error("Replay attack detected")]
    ReplayAttack,

    /// Double spending attempt
    #[error("Double spending attempt detected")]
    DoubleSpending,

    /// Resource exhaustion attack
    #[error("Resource exhaustion detected")]
    ResourceExhaustion,

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error (avoid using when possible)
    #[error("Exchange error: {0}")]
    Other(String),
}

/// Security-sensitive error that may contain secrets
#[derive(Debug)]
pub struct SecureError {
    inner: String,
}

impl SecureError {
    /// Create a new secure error
    pub fn new(msg: String) -> Self {
        Self { inner: msg }
    }
}

impl Drop for SecureError {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl std::fmt::Display for SecureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, ExchangeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_error_zeroization() {
        let mut error = SecureError::new("sensitive data".to_string());
        drop(error);
        // After drop, the inner string should be zeroized
    }

    #[test]
    fn test_secure_error_display() {
        let error = SecureError::new("sensitive data".to_string());
        assert_eq!(format!("{}", error), "[REDACTED]");
    }
}
