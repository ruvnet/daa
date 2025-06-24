//! Error types for the DAA orchestrator

use thiserror::Error;

/// Result type for orchestrator operations
pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Errors that can occur in orchestrator operations
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("Autonomy loop error: {0}")]
    AutonomyError(String),
    
    #[error("QuDAG integration error: {0}")]
    QuDAGError(String),
    
    #[error("MCP server error: {0}")]
    McpError(String),
    
    #[error("API server error: {0}")]
    ApiError(String),
    
    #[error("Chain integration error: {0}")]
    ChainError(#[from] daa_chain::ChainError),
    
    #[error("Economy error: {0}")]
    EconomyError(#[from] daa_economy::EconomyError),
    
    #[error("Rules engine error: {0}")]
    RulesError(#[from] daa_rules::RuleError),
    
    #[error("AI error: {0}")]
    AiError(#[from] daa_ai::AiError),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Timeout error: operation timed out after {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },
    
    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for OrchestratorError {
    fn from(err: serde_json::Error) -> Self {
        OrchestratorError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for OrchestratorError {
    fn from(err: std::io::Error) -> Self {
        OrchestratorError::Internal(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for OrchestratorError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        OrchestratorError::TimeoutError { timeout_ms: 30000 }
    }
}