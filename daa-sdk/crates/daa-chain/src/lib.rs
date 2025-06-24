//! # DAA Chain - Blockchain I/O Abstraction
//!
//! This crate provides a unified interface for blockchain interactions,
//! supporting multiple chains through a common adapter trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod ethereum;
pub mod substrate;

#[cfg(test)]
mod tests;

/// Errors that can occur during blockchain operations
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
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Blockchain-agnostic address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(Vec<u8>);

impl Address {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Address(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Transaction hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(String);

impl TxHash {
    pub fn new(hash: String) -> Self {
        TxHash(hash)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Balance representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    pub value: u128,
    pub decimals: u8,
}

impl Balance {
    pub fn new(value: u128, decimals: u8) -> Self {
        Balance { value, decimals }
    }
    
    pub fn from_wei(wei: u128) -> Self {
        Balance { value: wei, decimals: 18 }
    }
}

/// Generic blockchain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: Balance,
    pub data: Vec<u8>,
    pub nonce: Option<u64>,
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<TxHash>,
}

/// Common trait for all blockchain adapters
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait BlockchainAdapter: Send + Sync {
    /// Connect to the blockchain node
    async fn connect(&self) -> Result<(), AdapterError>;
    
    /// Send a transaction to the blockchain
    async fn send_transaction(&self, tx: Transaction) -> Result<TxHash, AdapterError>;
    
    /// Query account balance
    async fn query_balance(&self, account: &Address) -> Result<Balance, AdapterError>;
    
    /// Subscribe to new blocks
    async fn subscribe_blocks<F>(&self, handler: F) -> Result<(), AdapterError>
    where
        F: Fn(Block) + Send + 'static;
    
    /// Get current block number
    async fn get_block_number(&self) -> Result<u64, AdapterError>;
    
    /// Get block by number
    async fn get_block(&self, number: u64) -> Result<Option<Block>, AdapterError>;
}

/// Configuration for blockchain adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_type: ChainType,
    pub endpoint: String,
    pub network_id: Option<u64>,
    pub private_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainType {
    Ethereum,
    Substrate,
    Mock,
}

/// Factory function to create blockchain adapters
pub fn create_adapter(config: ChainConfig) -> Result<Box<dyn BlockchainAdapter>, AdapterError> {
    match config.chain_type {
        ChainType::Ethereum => {
            let adapter = ethereum::EthereumAdapter::new(
                config.endpoint,
                config.private_key.ok_or_else(|| AdapterError::ConfigError("Missing private key".into()))?
            );
            Ok(Box::new(adapter))
        }
        ChainType::Substrate => {
            let adapter = substrate::SubstrateAdapter::new(config.endpoint);
            Ok(Box::new(adapter))
        }
        ChainType::Mock => {
            #[cfg(test)]
            {
                let mut mock = MockBlockchainAdapter::new();
                mock.expect_connect()
                    .returning(|| Ok(()));
                Ok(Box::new(mock))
            }
            #[cfg(not(test))]
            Err(AdapterError::ConfigError("Mock adapter only available in tests".into()))
        }
    }
}