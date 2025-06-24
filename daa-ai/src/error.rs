//! Error types for the DAA AI module

use thiserror::Error;

/// Result type for AI operations
pub type Result<T> = std::result::Result<T, AiError>;

/// Errors that can occur in AI operations
#[derive(Error, Debug, Clone)]
pub enum AiError {
    #[error("Model error: {0}")]
    ModelError(String),
    
    #[error("Agent error: {0}")]
    AgentError(String),
    
    #[error("Decision error: {0}")]
    DecisionError(String),
    
    #[error("Learning error: {0}")]
    LearningError(String),
    
    #[error("MCP integration error: {0}")]
    McpError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Timeout error: operation timed out after {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },
    
    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for AiError {
    fn from(err: serde_json::Error) -> Self {
        AiError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for AiError {
    fn from(err: reqwest::Error) -> Self {
        AiError::NetworkError(err.to_string())
    }
}

impl From<anyhow::Error> for AiError {
    fn from(err: anyhow::Error) -> Self {
        AiError::Internal(err.to_string())
    }
}