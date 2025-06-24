//! Error types for the DAA AI system

use thiserror::Error;

/// Result type for AI operations
pub type Result<T> = std::result::Result<T, AIError>;

/// Error types that can occur in the AI system
#[derive(Error, Debug)]
pub enum AIError {
    #[error("MCP connection error: {0}")]
    McpConnectionError(String),

    #[error("MCP protocol error: {0}")]
    McpProtocolError(String),

    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),

    #[error("JSON parsing error: {0}")]
    JsonParsingError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Agent not initialized")]
    AgentNotInitialized,

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("QuDAG integration error: {0}")]
    QuDAGError(String),

    #[error("Rules engine error: {0}")]
    RulesEngineError(#[from] daa_rules::error::RuleError),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<anyhow::Error> for AIError {
    fn from(err: anyhow::Error) -> Self {
        AIError::Generic(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for AIError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AIError::Timeout(err.to_string())
    }
}