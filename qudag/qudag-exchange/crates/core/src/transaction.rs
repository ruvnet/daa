//! Transaction types and processing

use crate::{AccountId, Balance};
use serde::{Deserialize, Serialize};

/// Unique transaction identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub String);

/// Transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionId,
    pub from: AccountId,
    pub to: AccountId,
    pub amount: Balance,
    pub timestamp: u64,
    pub nonce: u64,
    pub signature: Vec<u8>,
}

impl Transaction {
    /// Compute transaction hash
    pub fn hash(&self) -> Vec<u8> {
        // Placeholder - will use blake3
        vec![]
    }
    
    /// Verify transaction signature
    pub fn verify_signature(&self) -> bool {
        // Placeholder - will use qudag-crypto
        true
    }
}

/// Transaction status in the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Rejected(String),
}