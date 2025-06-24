//! Block management and construction for DAA Chain

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use blake3::Hasher;

use crate::qudag_stubs::qudag_core::{Block, Transaction, Hash};
use crate::{Result, ChainError};

/// Block builder for constructing valid blocks
pub struct Builder {
    transactions: Vec<Transaction>,
    parent_hash: Option<Hash>,
    timestamp: Option<u64>,
    extra_data: Vec<u8>,
}

impl Builder {
    /// Create a new block builder
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            parent_hash: None,
            timestamp: None,
            extra_data: Vec::new(),
        }
    }

    /// Add transactions to the block
    pub fn with_transactions(mut self, transactions: Vec<Transaction>) -> Self {
        self.transactions = transactions;
        self
    }

    /// Set the parent block hash
    pub fn with_parent(mut self, parent_hash: Hash) -> Self {
        self.parent_hash = Some(parent_hash);
        self
    }

    /// Set the block timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Add extra data to the block
    pub fn with_extra_data(mut self, data: Vec<u8>) -> Self {
        self.extra_data = data;
        self
    }

    /// Build the block
    pub fn build(self) -> Result<Block> {
        let timestamp = self.timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        let parent_hash = self.parent_hash.unwrap_or_default();

        // Calculate merkle root of transactions
        let merkle_root = self.calculate_merkle_root(&self.transactions)?;

        // Create block header
        let header = BlockHeader {
            parent_hash,
            merkle_root,
            timestamp,
            transaction_count: self.transactions.len() as u32,
            extra_data: self.extra_data,
        };

        // Calculate block hash
        let hash = header.hash()?;

        Ok(Block::new(hash, header, self.transactions))
    }

    /// Calculate merkle root of transactions
    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> Result<Hash> {
        if transactions.is_empty() {
            return Ok(Hash::default());
        }

        let mut hasher = Hasher::new();
        for tx in transactions {
            hasher.update(&tx.hash().as_bytes());
        }

        Ok(Hash::from_bytes(hasher.finalize().as_bytes()))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Hash of the parent block
    pub parent_hash: Hash,
    
    /// Merkle root of all transactions
    pub merkle_root: Hash,
    
    /// Block timestamp
    pub timestamp: u64,
    
    /// Number of transactions in block
    pub transaction_count: u32,
    
    /// Additional data
    pub extra_data: Vec<u8>,
}

impl BlockHeader {
    /// Calculate the hash of this header
    pub fn hash(&self) -> Result<Hash> {
        let serialized = serde_json::to_vec(self)
            .map_err(|e| ChainError::BlockValidation(format!("Serialization failed: {}", e)))?;
        
        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        
        Ok(Hash::from_bytes(hasher.finalize().as_bytes()))
    }
}

/// Block validation utilities
pub struct Validator;

impl Validator {
    /// Validate a block structure and content
    pub fn validate_block(block: &Block) -> Result<()> {
        // Validate transactions
        for tx in block.transactions() {
            Self::validate_transaction(tx)?;
        }

        // Validate merkle root
        Self::validate_merkle_root(block)?;

        // Validate timestamp
        Self::validate_timestamp(block)?;

        Ok(())
    }

    /// Validate a single transaction
    fn validate_transaction(tx: &Transaction) -> Result<()> {
        if tx.signature().is_empty() {
            return Err(ChainError::InvalidTransaction(
                "Transaction missing signature".to_string()
            ));
        }

        // Additional transaction validation
        Ok(())
    }

    /// Validate the merkle root
    fn validate_merkle_root(block: &Block) -> Result<()> {
        let builder = Builder::new();
        let calculated_root = builder.calculate_merkle_root(block.transactions())?;
        
        if calculated_root != block.header().merkle_root {
            return Err(ChainError::BlockValidation(
                "Invalid merkle root".to_string()
            ));
        }

        Ok(())
    }

    /// Validate block timestamp
    fn validate_timestamp(block: &Block) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Block timestamp should not be too far in the future
        if block.header().timestamp > now + 300 { // 5 minutes tolerance
            return Err(ChainError::BlockValidation(
                "Block timestamp too far in future".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_builder() {
        let block = Builder::new()
            .with_timestamp(1234567890)
            .build()
            .unwrap();

        assert_eq!(block.header().timestamp, 1234567890);
        assert_eq!(block.transactions().len(), 0);
    }

    #[test]
    fn test_block_validation() {
        let block = Builder::new().build().unwrap();
        assert!(Validator::validate_block(&block).is_ok());
    }
}