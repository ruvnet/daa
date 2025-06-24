//! Error types for the DAA economy module

use thiserror::Error;

/// Result type for economy operations
pub type Result<T> = std::result::Result<T, EconomyError>;

/// Errors that can occur in economic operations
#[derive(Error, Debug, Clone)]
pub enum EconomyError {
    #[error("Market data error: {0}")]
    MarketDataError(String),
    
    #[error("Resource allocation error: {0}")]
    ResourceAllocationError(String),
    
    #[error("Risk assessment error: {0}")]
    RiskAssessmentError(String),
    
    #[error("Trading error: {0}")]
    TradingError(String),
    
    #[error("Optimization error: {0}")]
    OptimizationError(String),
    
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u128, available: u128 },
    
    #[error("Resource not available: {0}")]
    ResourceNotAvailable(String),
    
    #[error("Invalid price: {0}")]
    InvalidPrice(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for EconomyError {
    fn from(err: serde_json::Error) -> Self {
        EconomyError::SerializationError(err.to_string())
    }
}

impl From<anyhow::Error> for EconomyError {
    fn from(err: anyhow::Error) -> Self {
        EconomyError::Internal(err.to_string())
    }
}