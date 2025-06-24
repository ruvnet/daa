//! Transaction processing module for QuDAG protocol.

use crate::types::ProtocolError;
use qudag_crypto::{ml_dsa::MlDsa65, signature::{Signature, SignatureError}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, info};

/// Transaction processing errors
#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Invalid transaction format")]
    InvalidFormat,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Double spending detected")]
    DoubleSpending,
    
    #[error("Transaction not found")]
    NotFound,
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
}

/// Transaction input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionInput {
    /// Previous transaction hash
    pub prev_tx_hash: [u8; 32],
    /// Output index in previous transaction
    pub prev_output_index: u32,
    /// Signature script
    pub signature_script: Vec<u8>,
}

/// Transaction output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionOutput {
    /// Amount to transfer
    pub amount: u64,
    /// Recipient public key hash
    pub recipient: [u8; 32],
    /// Locking script
    pub locking_script: Vec<u8>,
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// Transaction version
    pub version: u32,
    /// Transaction inputs
    pub inputs: Vec<TransactionInput>,
    /// Transaction outputs
    pub outputs: Vec<TransactionOutput>,
    /// Transaction timestamp
    pub timestamp: u64,
    /// Transaction nonce (for replay protection)
    pub nonce: u64,
    /// Fee amount
    pub fee: u64,
    /// Transaction signature
    pub signature: Vec<u8>,
}

impl Transaction {
    /// Create new transaction
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        fee: u64,
    ) -> Self {
        Self {
            version: 1,
            inputs,
            outputs,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: 0,
            fee,
            signature: Vec::new(),
        }
    }

    /// Calculate transaction hash
    pub fn hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        
        let mut hasher = Sha3_256::new();
        
        // Hash version
        hasher.update(self.version.to_le_bytes());
        
        // Hash inputs
        for input in &self.inputs {
            hasher.update(input.prev_tx_hash);
            hasher.update(input.prev_output_index.to_le_bytes());
            hasher.update(&input.signature_script);
        }
        
        // Hash outputs
        for output in &self.outputs {
            hasher.update(output.amount.to_le_bytes());
            hasher.update(output.recipient);
            hasher.update(&output.locking_script);
        }
        
        // Hash timestamp, nonce, and fee
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.fee.to_le_bytes());
        
        hasher.finalize().into()
    }

    /// Sign transaction
    pub fn sign(&mut self, private_key: &[u8]) -> Result<(), TransactionError> {
        let hash = self.hash();
        
        // Create ML-DSA signature
        let ml_dsa = MlDsa65::new()
            .map_err(|e| TransactionError::CryptoError(e.to_string()))?;
        
        self.signature = ml_dsa.sign(private_key, &hash)
            .map_err(|e| TransactionError::CryptoError(e.to_string()))?;
        
        Ok(())
    }

    /// Verify transaction signature
    pub fn verify_signature(&self, public_key: &[u8]) -> Result<bool, TransactionError> {
        if self.signature.is_empty() {
            return Ok(false);
        }
        
        let hash = self.hash();
        
        let ml_dsa = MlDsa65::new()
            .map_err(|e| TransactionError::CryptoError(e.to_string()))?;
        
        ml_dsa.verify(public_key, &hash, &self.signature)
            .map_err(|e| TransactionError::CryptoError(e.to_string()))
    }

    /// Get total input amount
    pub fn total_input_amount(&self, utxo_set: &UTXOSet) -> Result<u64, TransactionError> {
        let mut total = 0u64;
        
        for input in &self.inputs {
            if let Some(utxo) = utxo_set.get_utxo(&input.prev_tx_hash, input.prev_output_index) {
                total = total.checked_add(utxo.amount)
                    .ok_or(TransactionError::ValidationFailed("Integer overflow".to_string()))?;
            } else {
                return Err(TransactionError::ValidationFailed("UTXO not found".to_string()));
            }
        }
        
        Ok(total)
    }

    /// Get total output amount
    pub fn total_output_amount(&self) -> Result<u64, TransactionError> {
        let mut total = 0u64;
        
        for output in &self.outputs {
            total = total.checked_add(output.amount)
                .ok_or(TransactionError::ValidationFailed("Integer overflow".to_string()))?;
        }
        
        Ok(total)
    }

    /// Validate transaction
    pub fn validate(&self, utxo_set: &UTXOSet, public_key: &[u8]) -> Result<(), TransactionError> {
        // Check basic format
        if self.inputs.is_empty() {
            return Err(TransactionError::ValidationFailed("No inputs".to_string()));
        }
        
        if self.outputs.is_empty() {
            return Err(TransactionError::ValidationFailed("No outputs".to_string()));
        }

        // Verify signature
        if !self.verify_signature(public_key)? {
            return Err(TransactionError::InvalidSignature);
        }

        // Check balance
        let input_amount = self.total_input_amount(utxo_set)?;
        let output_amount = self.total_output_amount()?;
        
        if input_amount < output_amount + self.fee {
            return Err(TransactionError::InsufficientBalance);
        }

        // Check for double spending
        for input in &self.inputs {
            if utxo_set.is_spent(&input.prev_tx_hash, input.prev_output_index) {
                return Err(TransactionError::DoubleSpending);
            }
        }

        Ok(())
    }
}

/// Unspent Transaction Output (UTXO)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXO {
    /// Transaction hash
    pub tx_hash: [u8; 32],
    /// Output index
    pub output_index: u32,
    /// Output amount
    pub amount: u64,
    /// Recipient
    pub recipient: [u8; 32],
    /// Locking script
    pub locking_script: Vec<u8>,
    /// Block height when created
    pub block_height: u64,
}

/// UTXO Set for tracking unspent outputs
#[derive(Debug, Clone)]
pub struct UTXOSet {
    /// Map of (tx_hash, output_index) -> UTXO
    utxos: HashMap<([u8; 32], u32), UTXO>,
    /// Set of spent UTXOs
    spent: HashMap<([u8; 32], u32), bool>,
}

impl UTXOSet {
    /// Create new UTXO set
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
            spent: HashMap::new(),
        }
    }

    /// Add UTXO to set
    pub fn add_utxo(&mut self, utxo: UTXO) {
        let key = (utxo.tx_hash, utxo.output_index);
        self.utxos.insert(key, utxo);
        self.spent.remove(&key); // Unmark as spent if it was
    }

    /// Get UTXO from set
    pub fn get_utxo(&self, tx_hash: &[u8; 32], output_index: u32) -> Option<&UTXO> {
        let key = (*tx_hash, output_index);
        if self.spent.get(&key).copied().unwrap_or(false) {
            None
        } else {
            self.utxos.get(&key)
        }
    }

    /// Mark UTXO as spent
    pub fn spend_utxo(&mut self, tx_hash: &[u8; 32], output_index: u32) {
        let key = (*tx_hash, output_index);
        self.spent.insert(key, true);
    }

    /// Check if UTXO is spent
    pub fn is_spent(&self, tx_hash: &[u8; 32], output_index: u32) -> bool {
        let key = (*tx_hash, output_index);
        self.spent.get(&key).copied().unwrap_or(false)
    }

    /// Get balance for address
    pub fn get_balance(&self, address: &[u8; 32]) -> u64 {
        self.utxos
            .values()
            .filter(|utxo| !self.is_spent(&utxo.tx_hash, utxo.output_index))
            .filter(|utxo| utxo.recipient == *address)
            .map(|utxo| utxo.amount)
            .sum()
    }

    /// Get UTXOs for address
    pub fn get_utxos_for_address(&self, address: &[u8; 32]) -> Vec<&UTXO> {
        self.utxos
            .values()
            .filter(|utxo| !self.is_spent(&utxo.tx_hash, utxo.output_index))
            .filter(|utxo| utxo.recipient == *address)
            .collect()
    }
}

impl Default for UTXOSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction processor
pub struct TransactionProcessor {
    /// UTXO set
    utxo_set: UTXOSet,
    /// Pending transactions
    pending_txs: HashMap<[u8; 32], Transaction>,
    /// Transaction pool
    tx_pool: Vec<Transaction>,
}

impl TransactionProcessor {
    /// Create new transaction processor
    pub fn new() -> Self {
        Self {
            utxo_set: UTXOSet::new(),
            pending_txs: HashMap::new(),
            tx_pool: Vec::new(),
        }
    }

    /// Add transaction to pool
    pub fn add_transaction(&mut self, tx: Transaction, public_key: &[u8]) -> Result<(), TransactionError> {
        // Validate transaction
        tx.validate(&self.utxo_set, public_key)?;
        
        let tx_hash = tx.hash();
        
        // Add to pool
        self.tx_pool.push(tx.clone());
        self.pending_txs.insert(tx_hash, tx);
        
        info!("Added transaction to pool: {:?}", hex::encode(tx_hash));
        Ok(())
    }

    /// Process transaction (commit to UTXO set)
    pub fn process_transaction(&mut self, tx_hash: &[u8; 32]) -> Result<(), TransactionError> {
        let tx = self.pending_txs.remove(tx_hash)
            .ok_or(TransactionError::NotFound)?;

        // Spend inputs
        for input in &tx.inputs {
            self.utxo_set.spend_utxo(&input.prev_tx_hash, input.prev_output_index);
        }

        // Create new UTXOs for outputs
        for (index, output) in tx.outputs.iter().enumerate() {
            let utxo = UTXO {
                tx_hash: *tx_hash,
                output_index: index as u32,
                amount: output.amount,
                recipient: output.recipient,
                locking_script: output.locking_script.clone(),
                block_height: 0, // TODO: Get current block height
            };
            self.utxo_set.add_utxo(utxo);
        }

        // Remove from pool
        self.tx_pool.retain(|pool_tx| pool_tx.hash() != *tx_hash);

        info!("Processed transaction: {:?}", hex::encode(tx_hash));
        Ok(())
    }

    /// Get pending transactions
    pub fn get_pending_transactions(&self) -> Vec<&Transaction> {
        self.tx_pool.iter().collect()
    }

    /// Get UTXO set
    pub fn get_utxo_set(&self) -> &UTXOSet {
        &self.utxo_set
    }

    /// Get balance for address
    pub fn get_balance(&self, address: &[u8; 32]) -> u64 {
        self.utxo_set.get_balance(address)
    }
}

impl Default for TransactionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qudag_crypto::ml_dsa::MlDsa65;

    #[test]
    fn test_transaction_creation() {
        let input = TransactionInput {
            prev_tx_hash: [1; 32],
            prev_output_index: 0,
            signature_script: vec![],
        };

        let output = TransactionOutput {
            amount: 100,
            recipient: [2; 32],
            locking_script: vec![],
        };

        let tx = Transaction::new(vec![input], vec![output], 1);
        
        assert_eq!(tx.version, 1);
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.fee, 1);
    }

    #[test]
    fn test_transaction_hash() {
        let input = TransactionInput {
            prev_tx_hash: [1; 32],
            prev_output_index: 0,
            signature_script: vec![],
        };

        let output = TransactionOutput {
            amount: 100,
            recipient: [2; 32],
            locking_script: vec![],
        };

        let tx1 = Transaction::new(vec![input.clone()], vec![output.clone()], 1);
        let tx2 = Transaction::new(vec![input], vec![output], 1);
        
        // Same transactions should have same hash (timestamp will differ)
        // So we'll just test that hash is deterministic for same content
        let hash1 = tx1.hash();
        let hash2 = tx1.hash();
        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_transaction_signing() {
        let ml_dsa = MlDsa65::new().unwrap();
        let (pk, sk) = ml_dsa.keygen().unwrap();

        let input = TransactionInput {
            prev_tx_hash: [1; 32],
            prev_output_index: 0,
            signature_script: vec![],
        };

        let output = TransactionOutput {
            amount: 100,
            recipient: [2; 32],
            locking_script: vec![],
        };

        let mut tx = Transaction::new(vec![input], vec![output], 1);
        
        // Sign transaction
        tx.sign(sk.as_bytes()).unwrap();
        assert!(!tx.signature.is_empty());

        // Verify signature
        assert!(tx.verify_signature(pk.as_bytes()).unwrap());
    }

    #[test]
    fn test_utxo_set() {
        let mut utxo_set = UTXOSet::new();
        
        let utxo = UTXO {
            tx_hash: [1; 32],
            output_index: 0,
            amount: 100,
            recipient: [2; 32],
            locking_script: vec![],
            block_height: 1,
        };

        // Add UTXO
        utxo_set.add_utxo(utxo.clone());
        
        // Check balance
        assert_eq!(utxo_set.get_balance(&[2; 32]), 100);
        
        // Spend UTXO
        utxo_set.spend_utxo(&[1; 32], 0);
        assert_eq!(utxo_set.get_balance(&[2; 32]), 0);
        assert!(utxo_set.is_spent(&[1; 32], 0));
    }

    #[tokio::test]
    async fn test_transaction_processor() {
        let mut processor = TransactionProcessor::new();
        let ml_dsa = MlDsa65::new().unwrap();
        let (pk, sk) = ml_dsa.keygen().unwrap();

        // Create a genesis UTXO
        let genesis_utxo = UTXO {
            tx_hash: [0; 32],
            output_index: 0,
            amount: 1000,
            recipient: [1; 32],
            locking_script: vec![],
            block_height: 0,
        };
        processor.utxo_set.add_utxo(genesis_utxo);

        // Create transaction spending the genesis UTXO
        let input = TransactionInput {
            prev_tx_hash: [0; 32],
            prev_output_index: 0,
            signature_script: vec![],
        };

        let output = TransactionOutput {
            amount: 900,
            recipient: [2; 32],
            locking_script: vec![],
        };

        let mut tx = Transaction::new(vec![input], vec![output], 100);
        tx.sign(sk.as_bytes()).unwrap();

        // Add transaction to processor
        processor.add_transaction(tx.clone(), pk.as_bytes()).unwrap();
        
        // Process transaction
        let tx_hash = tx.hash();
        processor.process_transaction(&tx_hash).unwrap();

        // Check balances
        assert_eq!(processor.get_balance(&[1; 32]), 0); // Original owner
        assert_eq!(processor.get_balance(&[2; 32]), 900); // New owner
    }
}