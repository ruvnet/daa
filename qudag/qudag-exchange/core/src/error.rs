//! Error types for QuDAG Exchange Core
//!
//! Provides comprehensive error handling for all core operations

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

use serde::{Deserialize, Serialize};

/// Core error type for QuDAG Exchange operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    /// Insufficient balance for operation
    InsufficientBalance {
        /// Account that lacks balance
        account: String,
        /// Required amount
        required: u64,
        /// Available amount
        available: u64,
    },

    /// Account not found
    AccountNotFound(String),

    /// Transaction validation failed
    InvalidTransaction(String),

    /// Signature verification failed
    SignatureVerificationFailed,

    /// Resource limit exceeded
    ResourceLimitExceeded {
        /// Type of resource
        resource_type: String,
        /// Limit that was exceeded
        limit: u64,
        /// Requested amount
        requested: u64,
    },

    /// Consensus error
    ConsensusError(String),

    /// State corruption detected
    StateCorruption(String),

    /// Vault operation failed
    VaultError(String),

    /// Serialization/deserialization error
    SerializationError(String),

    /// Operation not supported
    NotSupported(String),

    /// Generic error with message
    Other(String),
}

impl Error {
    /// Create an insufficient balance error
    pub fn insufficient_balance(account: impl Into<String>, required: u64, available: u64) -> Self {
        Self::InsufficientBalance {
            account: account.into(),
            required,
            available,
        }
    }

    /// Create a resource limit exceeded error
    pub fn resource_limit_exceeded(
        resource_type: impl Into<String>,
        limit: u64,
        requested: u64,
    ) -> Self {
        Self::ResourceLimitExceeded {
            resource_type: resource_type.into(),
            limit,
            requested,
        }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientBalance {
                account,
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient balance for account {}: required {}, available {}",
                    account, required, available
                )
            }
            Self::AccountNotFound(id) => write!(f, "Account not found: {}", id),
            Self::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            Self::SignatureVerificationFailed => write!(f, "Signature verification failed"),
            Self::ResourceLimitExceeded {
                resource_type,
                limit,
                requested,
            } => {
                write!(
                    f,
                    "Resource limit exceeded for {}: limit {}, requested {}",
                    resource_type, limit, requested
                )
            }
            Self::ConsensusError(msg) => write!(f, "Consensus error: {}", msg),
            Self::StateCorruption(msg) => write!(f, "State corruption: {}", msg),
            Self::VaultError(msg) => write!(f, "Vault error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::NotSupported(msg) => write!(f, "Operation not supported: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "std")]
impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

/// Result type alias for QuDAG Exchange operations
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::insufficient_balance("alice", 100, 50);
        match err {
            Error::InsufficientBalance {
                account,
                required,
                available,
            } => {
                assert_eq!(account, "alice");
                assert_eq!(required, 100);
                assert_eq!(available, 50);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_serialization() {
        let err = Error::AccountNotFound("bob".to_string());
        let serialized = bincode::serialize(&err).unwrap();
        let deserialized: Error = bincode::deserialize(&serialized).unwrap();
        assert_eq!(err, deserialized);
    }
}
