//! Error types for the daa-chain crate

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("Transaction failed: {0}")]
    TransactionError(String),
    
    #[error("Query failed: {0}")]
    QueryError(String),
    
    #[error("Subscription failed: {0}")]
    SubscriptionError(String),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },
    
    #[error("Signing failed: {0}")]
    SigningError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<ethers::providers::ProviderError> for AdapterError {
    fn from(err: ethers::providers::ProviderError) -> Self {
        AdapterError::NetworkError(err.to_string())
    }
}

impl From<ethers::signers::WalletError> for AdapterError {
    fn from(err: ethers::signers::WalletError) -> Self {
        AdapterError::SigningError(err.to_string())
    }
}

impl From<serde_json::Error> for AdapterError {
    fn from(err: serde_json::Error) -> Self {
        AdapterError::SerializationError(err.to_string())
    }
}

impl From<hex::FromHexError> for AdapterError {
    fn from(err: hex::FromHexError) -> Self {
        AdapterError::InvalidAddress(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AdapterError>;