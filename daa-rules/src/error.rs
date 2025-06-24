//! Error types for the DAA rules engine

use thiserror::Error;

/// Result type for rule operations
pub type Result<T> = std::result::Result<T, RuleError>;

/// Errors that can occur in the rule engine
#[derive(Error, Debug, Clone)]
pub enum RuleError {
    #[error("Rule validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Rule execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Context error: {0}")]
    ContextError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Audit error: {0}")]
    AuditError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for RuleError {
    fn from(err: serde_json::Error) -> Self {
        RuleError::SerializationError(err.to_string())
    }
}

impl From<anyhow::Error> for RuleError {
    fn from(err: anyhow::Error) -> Self {
        RuleError::Internal(err.to_string())
    }
}