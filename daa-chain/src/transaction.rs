//! Transaction management for DAA Chain

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use blake3::Hasher;
use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};

use crate::qudag_stubs::qudag_core::{Transaction, Hash};
use crate::{Result, ChainError};

/// Transaction types supported by DAA Chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Agent registration transaction
    AgentRegistration {
        agent_id: String,
        public_key: Vec<u8>,
        capabilities: Vec<String>,
    },
    
    /// Resource allocation transaction
    ResourceAllocation {
        agent_id: String,
        resource_type: String,
        amount: u64,
    },
    
    /// Task assignment transaction
    TaskAssignment {
        task_id: String,
        agent_id: String,
        parameters: HashMap<String, String>,
    },
    
    /// Reward distribution transaction
    RewardDistribution {
        agent_id: String,
        amount: u64,
        reason: String,
    },
    
    /// Generic data transaction
    Data {
        data: Vec<u8>,
        metadata: HashMap<String, String>,
    },
}

/// DAA-specific transaction builder
pub struct TransactionBuilder {
    transaction_type: Option<TransactionType>,
    nonce: u64,
    gas_limit: u64,
    gas_price: u64,
    metadata: HashMap<String, String>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new() -> Self {
        Self {
            transaction_type: None,
            nonce: 0,
            gas_limit: 21000,
            gas_price: 1,
            metadata: HashMap::new(),
        }
    }

    /// Set the transaction type
    pub fn with_type(mut self, tx_type: TransactionType) -> Self {
        self.transaction_type = Some(tx_type);
        self
    }

    /// Set the nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set gas parameters
    pub fn with_gas(mut self, limit: u64, price: u64) -> Self {
        self.gas_limit = limit;
        self.gas_price = price;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Build and sign the transaction
    pub fn build_and_sign(
        self,
        private_key: &ed25519_dalek::SigningKey,
    ) -> Result<Transaction> {
        let tx_type = self.transaction_type.clone()
            .ok_or_else(|| ChainError::InvalidTransaction("Missing transaction type".to_string()))?;

        // Create transaction data
        let tx_data = DaaTransactionData {
            transaction_type: tx_type,
            nonce: self.nonce,
            gas_limit: self.gas_limit,
            gas_price: self.gas_price,
            metadata: self.metadata.clone(),
        };

        // Serialize transaction data for signing
        let serialized = serde_json::to_vec(&tx_data)
            .map_err(|e| ChainError::InvalidTransaction(format!("Serialization failed: {}", e)))?;

        // Sign the transaction
        let signature = private_key.sign(&serialized);

        // Create final transaction
        let transaction = DaaTransaction {
            data: tx_data,
            signature: signature.to_bytes().to_vec(),
            public_key: private_key.verifying_key().to_bytes().to_vec(),
        };

        // Convert to QuDAG Transaction
        self.to_qudag_transaction(transaction)
    }

    /// Convert DAA transaction to QuDAG transaction
    fn to_qudag_transaction(&self, daa_tx: DaaTransaction) -> Result<Transaction> {
        let serialized = serde_json::to_vec(&daa_tx)
            .map_err(|e| ChainError::InvalidTransaction(format!("Serialization failed: {}", e)))?;

        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        let hash = Hash::from_bytes(hasher.finalize().as_bytes());

        Ok(Transaction::new_with_data(hash, serialized, daa_tx.signature))
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// DAA transaction data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaaTransactionData {
    pub transaction_type: TransactionType,
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub metadata: HashMap<String, String>,
}

/// Complete DAA transaction with signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaaTransaction {
    pub data: DaaTransactionData,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

/// Transaction validation and verification utilities
pub struct TransactionValidator;

impl TransactionValidator {
    /// Validate a QuDAG transaction as a DAA transaction
    pub fn validate_transaction(tx: &Transaction) -> Result<DaaTransaction> {
        // Deserialize transaction data
        let daa_tx: DaaTransaction = serde_json::from_slice(tx.data())
            .map_err(|e| ChainError::InvalidTransaction(format!("Invalid transaction format: {}", e)))?;

        // Verify signature
        Self::verify_signature(&daa_tx)?;

        // Validate transaction type specific rules
        Self::validate_transaction_type(&daa_tx.data.transaction_type)?;

        Ok(daa_tx)
    }

    /// Verify transaction signature
    fn verify_signature(daa_tx: &DaaTransaction) -> Result<()> {
        // Reconstruct the signed data
        let serialized = serde_json::to_vec(&daa_tx.data)
            .map_err(|e| ChainError::InvalidTransaction(format!("Serialization failed: {}", e)))?;

        // Parse public key
        let public_key_bytes: [u8; 32] = daa_tx.public_key.as_slice().try_into()
            .map_err(|_| ChainError::InvalidTransaction("Invalid public key length".to_string()))?;
        
        let public_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|e| ChainError::InvalidTransaction(format!("Invalid public key: {}", e)))?;

        // Parse signature
        let signature_bytes: [u8; 64] = daa_tx.signature.as_slice().try_into()
            .map_err(|_| ChainError::InvalidTransaction("Invalid signature length".to_string()))?;
        
        let signature = Signature::from_bytes(&signature_bytes);

        // Verify signature
        public_key.verify(&serialized, &signature)
            .map_err(|e| ChainError::InvalidTransaction(format!("Signature verification failed: {}", e)))?;

        Ok(())
    }

    /// Validate transaction type specific rules
    fn validate_transaction_type(tx_type: &TransactionType) -> Result<()> {
        match tx_type {
            TransactionType::AgentRegistration { agent_id, public_key, .. } => {
                if agent_id.is_empty() {
                    return Err(ChainError::InvalidTransaction("Empty agent ID".to_string()));
                }
                if public_key.len() != 32 {
                    return Err(ChainError::InvalidTransaction("Invalid public key length".to_string()));
                }
            }
            
            TransactionType::ResourceAllocation { agent_id, amount, .. } => {
                if agent_id.is_empty() {
                    return Err(ChainError::InvalidTransaction("Empty agent ID".to_string()));
                }
                if *amount == 0 {
                    return Err(ChainError::InvalidTransaction("Zero allocation amount".to_string()));
                }
            }
            
            TransactionType::TaskAssignment { task_id, agent_id, .. } => {
                if task_id.is_empty() || agent_id.is_empty() {
                    return Err(ChainError::InvalidTransaction("Empty task or agent ID".to_string()));
                }
            }
            
            TransactionType::RewardDistribution { agent_id, amount, .. } => {
                if agent_id.is_empty() {
                    return Err(ChainError::InvalidTransaction("Empty agent ID".to_string()));
                }
                if *amount == 0 {
                    return Err(ChainError::InvalidTransaction("Zero reward amount".to_string()));
                }
            }
            
            TransactionType::Data { data, .. } => {
                if data.is_empty() {
                    return Err(ChainError::InvalidTransaction("Empty data".to_string()));
                }
            }
        }

        Ok(())
    }
}

/// Transaction pool for managing pending transactions
pub struct TransactionPool {
    pending: HashMap<Hash, DaaTransaction>,
    max_size: usize,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: HashMap::new(),
            max_size,
        }
    }

    /// Add a transaction to the pool
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        if self.pending.len() >= self.max_size {
            return Err(ChainError::InvalidTransaction("Transaction pool full".to_string()));
        }

        let daa_tx = TransactionValidator::validate_transaction(&tx)?;
        self.pending.insert(tx.hash(), daa_tx);

        Ok(())
    }

    /// Remove a transaction from the pool
    pub fn remove_transaction(&mut self, hash: &Hash) -> Option<DaaTransaction> {
        self.pending.remove(hash)
    }

    /// Get pending transactions up to a limit
    pub fn get_pending(&self, limit: usize) -> Vec<&DaaTransaction> {
        self.pending.values().take(limit).collect()
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.pending.len()
    }

    /// Clear the pool
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn test_transaction_builder() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        let tx = TransactionBuilder::new()
            .with_type(TransactionType::Data {
                data: b"test data".to_vec(),
                metadata: HashMap::new(),
            })
            .with_nonce(1)
            .build_and_sign(&signing_key)
            .unwrap();

        assert!(!tx.signature().is_empty());
    }

    #[test]
    fn test_transaction_validation() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        let tx = TransactionBuilder::new()
            .with_type(TransactionType::AgentRegistration {
                agent_id: "test-agent".to_string(),
                public_key: vec![0u8; 32],
                capabilities: vec!["test".to_string()],
            })
            .build_and_sign(&signing_key)
            .unwrap();

        let daa_tx = TransactionValidator::validate_transaction(&tx).unwrap();
        
        match daa_tx.data.transaction_type {
            TransactionType::AgentRegistration { agent_id, .. } => {
                assert_eq!(agent_id, "test-agent");
            }
            _ => panic!("Wrong transaction type"),
        }
    }

    #[test]
    fn test_transaction_pool() {
        let mut pool = TransactionPool::new(10);
        assert_eq!(pool.size(), 0);

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        let tx = TransactionBuilder::new()
            .with_type(TransactionType::Data {
                data: b"test".to_vec(),
                metadata: HashMap::new(),
            })
            .build_and_sign(&signing_key)
            .unwrap();

        pool.add_transaction(tx).unwrap();
        assert_eq!(pool.size(), 1);
    }
}