//! Error types for the DAA rules engine

use thiserror::Error;

/// Result type for rule operations
pub type Result<T> = std::result::Result<T, RuleError>;

/// Error types that can occur in the rules engine
#[derive(Error, Debug)]
pub enum RuleError {
    #[error("Rule validation failed: {0}")]
    ValidationFailed(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Invalid rule configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Context missing required field: {0}")]
    MissingContext(String),

    #[error("Rule execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Audit logging failed: {0}")]
    AuditFailed(String),

    #[error("Consensus failure: {0}")]
    ConsensusFailed(String),

    #[error("QuDAG integration error: {0}")]
    QuDAGError(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<anyhow::Error> for RuleError {
    fn from(err: anyhow::Error) -> Self {
        RuleError::Generic(err.to_string())
    }
}