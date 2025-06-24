//! DAA Chain - Blockchain abstraction layer with QuDAG integration
//!
//! This crate provides a unified interface for blockchain operations with full QuDAG support.

use async_trait::async_trait;
use std::collections::HashMap;

pub mod errors;
pub mod types;
pub mod qudag_adapter;

pub use errors::*;
pub use types::*;
pub use qudag_adapter::*;

/// Subscription ID for blockchain events
pub type SubscriptionId = String;

/// Generic trait for blockchain operations
#[async_trait]
pub trait BlockchainAdapter: Send + Sync {
    /// Connect to the blockchain network
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the blockchain network
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Check if connected to the blockchain
    fn is_connected(&self) -> bool;
    
    /// Get network configuration
    fn get_chain_config(&self) -> &ChainConfig;
    
    /// Send a transaction
    async fn send_transaction(&self, tx: Transaction) -> Result<TxHash>;
    
    /// Get transaction by hash
    async fn get_transaction(&self, hash: &TxHash) -> Result<Option<Transaction>>;
    
    /// Get transaction receipt
    async fn get_transaction_receipt(&self, hash: &TxHash) -> Result<Option<TransactionReceipt>>;
    
    /// Get balance for an address
    async fn get_balance(&self, address: &Address) -> Result<Balance>;
    
    /// Get current block number
    async fn get_block_number(&self) -> Result<u64>;
    
    /// Get block by number
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>>;
    
    /// Subscribe to new blocks
    async fn subscribe_blocks(&self) -> Result<SubscriptionId>;
    
    /// Subscribe to pending transactions
    async fn subscribe_pending_transactions(&self) -> Result<SubscriptionId>;
    
    /// Unsubscribe from events
    async fn unsubscribe(&self, subscription_id: &SubscriptionId) -> Result<()>;
    
    /// Estimate gas for a transaction
    async fn estimate_gas(&self, tx: &Transaction) -> Result<u64>;
    
    /// Get gas price
    async fn get_gas_price(&self) -> Result<u128>;
    
    /// Get nonce for an address
    async fn get_nonce(&self, address: &Address) -> Result<u64>;
    
    /// Call a smart contract (read-only)
    async fn call(&self, tx: &Transaction) -> Result<Vec<u8>>;
    
    /// Register a .dark domain
    async fn register_dark_domain(&self, domain: &str, address: &Address) -> Result<TxHash>;
    
    /// Resolve a .dark domain to an address
    async fn resolve_dark_domain(&self, domain: &str) -> Result<Option<Address>>;
}

/// Transaction receipt
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: TxHash,
    pub block_number: u64,
    pub block_hash: TxHash,
    pub transaction_index: u64,
    pub from: Address,
    pub to: Option<Address>,
    pub cumulative_gas_used: u64,
    pub gas_used: u64,
    pub status: TransactionStatus,
    pub logs: Vec<Log>,
}

/// Transaction status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TransactionStatus {
    Success,
    Failed,
    Pending,
}

/// Event log
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<TxHash>,
    pub data: Vec<u8>,
    pub block_number: u64,
    pub transaction_hash: TxHash,
    pub log_index: u64,
}

/// Factory for creating blockchain adapters
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create a QuDAG adapter
    pub async fn create_qudag_adapter(config: QuDAGConfig) -> Result<Box<dyn BlockchainAdapter>> {
        let adapter = QuDAGAdapter::new(config).await?;
        Ok(Box::new(adapter))
    }
    
    /// Create adapter from chain configuration
    pub async fn create_adapter(chain_config: ChainConfig) -> Result<Box<dyn BlockchainAdapter>> {
        match chain_config.name.as_str() {
            "QuDAG" => {
                let qudag_config = QuDAGConfig::from_chain_config(chain_config);
                Self::create_qudag_adapter(qudag_config).await
            }
            _ => Err(AdapterError::UnsupportedChain(chain_config.name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_adapter_factory() {
        let config = ChainConfig {
            chain_id: 1337,
            name: "QuDAG".to_string(),
            rpc_url: "http://localhost:8545".to_string(),
            explorer_url: None,
            native_token_symbol: "rUv".to_string(),
            native_token_decimals: 18,
        };
        
        let result = AdapterFactory::create_adapter(config).await;
        // This test would need actual QuDAG network running
        // assert!(result.is_ok());
    }
}