//! Protocol message validation implementation.

use crate::message::Message;
use thiserror::Error;

/// Errors that can occur during validation.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Invalid message format
    #[error("Invalid message format")]
    InvalidFormat,

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Message too old
    #[error("Message too old")]
    MessageTooOld,

    /// Invalid protocol version
    #[error("Invalid protocol version")]
    InvalidVersion,
}

/// Validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Message is valid
    Valid,

    /// Message is invalid
    Invalid,

    /// Message requires further validation
    PendingValidation,
}

/// Validation configuration.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum message age in seconds
    pub max_age: u64,

    /// Minimum protocol version
    pub min_version: u32,

    /// Required signature types
    pub required_signatures: Vec<String>,
}

/// Message validation trait defining the interface for validation operations.
pub trait MessageValidation {
    /// Initialize validation with configuration.
    fn init(config: ValidationConfig) -> Result<(), ValidationError>;

    /// Validate a message.
    fn validate_message(&self, message: &Message) -> Result<ValidationResult, ValidationError>;

    /// Validate message signature.
    fn validate_signature(&self, message: &Message) -> Result<bool, ValidationError>;

    /// Check message freshness.
    fn check_freshness(&self, message: &Message) -> bool;

    /// Validate protocol version.
    fn validate_version(&self, version: u32) -> bool;
}
