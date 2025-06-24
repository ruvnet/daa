//! Transaction handling for QuDAG Exchange
//!
//! Implements transactions with quantum-resistant signatures using QuDAG's crypto primitives

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    account::AccountId,
    types::{rUv, Hash, Nonce, Timestamp},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// Transaction identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(Hash);

impl TransactionId {
    /// Create from hash
    pub fn from_hash(hash: Hash) -> Self {
        Self(hash)
    }

    /// Get the underlying hash
    pub fn hash(&self) -> &Hash {
        &self.0
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx_{}", self.0)
    }
}

/// Transaction status in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction is being processed
    Processing,
    /// Transaction is confirmed and finalized
    Confirmed,
    /// Transaction was rejected
    Rejected,
    /// Transaction expired before confirmation
    Expired,
}

/// Transaction type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Simple transfer of rUv between accounts
    Transfer {
        /// Source account
        from: AccountId,
        /// Destination account
        to: AccountId,
        /// Amount to transfer
        amount: rUv,
    },
    /// Mint new rUv tokens (privileged operation)
    Mint {
        /// Account to receive new tokens
        to: AccountId,
        /// Amount to mint
        amount: rUv,
    },
    /// Burn rUv tokens
    Burn {
        /// Account to burn from
        from: AccountId,
        /// Amount to burn
        amount: rUv,
    },
    /// Create a new account
    CreateAccount {
        /// New account ID
        account: AccountId,
        /// Initial balance (if any)
        initial_balance: rUv,
    },
    /// Update account metadata
    UpdateAccount {
        /// Account to update
        account: AccountId,
        /// New public key (optional)
        public_key: Option<Vec<u8>>,
    },
}

/// Main transaction structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction type and data
    pub tx_type: TransactionType,

    /// Transaction nonce (for ordering and replay protection)
    pub nonce: Nonce,

    /// Timestamp when transaction was created
    pub timestamp: Timestamp,

    /// Fee paid for the transaction (in rUv)
    pub fee: rUv,

    /// Optional expiry timestamp
    pub expires_at: Option<Timestamp>,

    /// Quantum-resistant signature
    pub signature: Option<TransactionSignature>,

    /// Additional metadata
    pub metadata: TransactionMetadata,
}

/// Transaction signature using quantum-resistant algorithms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionSignature {
    /// Algorithm used (e.g., "ML-DSA-87", "ML-DSA-65")
    pub algorithm: String,

    /// Public key of the signer
    pub public_key: Vec<u8>,

    /// The signature bytes
    pub signature: Vec<u8>,
}

/// Transaction metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionMetadata {
    /// Optional memo/note
    pub memo: Option<String>,

    /// Protocol version
    pub version: u32,

    /// Chain ID for replay protection
    pub chain_id: u64,
}

impl Default for TransactionMetadata {
    fn default() -> Self {
        Self {
            memo: None,
            version: 1,
            chain_id: 1,
        }
    }
}

impl Transaction {
    /// Create a new transfer transaction
    pub fn transfer(from: AccountId, to: AccountId, amount: rUv, nonce: Nonce, fee: rUv) -> Self {
        Self {
            tx_type: TransactionType::Transfer { from, to, amount },
            nonce,
            timestamp: Self::current_timestamp(),
            fee,
            expires_at: None,
            signature: None,
            metadata: TransactionMetadata::default(),
        }
    }

    /// Create a new mint transaction
    pub fn mint(to: AccountId, amount: rUv, nonce: Nonce, fee: rUv) -> Self {
        Self {
            tx_type: TransactionType::Mint { to, amount },
            nonce,
            timestamp: Self::current_timestamp(),
            fee,
            expires_at: None,
            signature: None,
            metadata: TransactionMetadata::default(),
        }
    }

    /// Create a new burn transaction
    pub fn burn(from: AccountId, amount: rUv, nonce: Nonce, fee: rUv) -> Self {
        Self {
            tx_type: TransactionType::Burn { from, amount },
            nonce,
            timestamp: Self::current_timestamp(),
            fee,
            expires_at: None,
            signature: None,
            metadata: TransactionMetadata::default(),
        }
    }

    /// Set expiry timestamp
    pub fn with_expiry(mut self, expires_at: Timestamp) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set memo
    pub fn with_memo(mut self, memo: impl Into<String>) -> Self {
        self.metadata.memo = Some(memo.into());
        self
    }

    /// Compute transaction ID
    pub fn id(&self) -> Result<TransactionId> {
        let bytes = self.to_bytes()?;
        let hash = blake3::hash(&bytes);
        Ok(TransactionId::from_hash(Hash::from_bytes(*hash.as_bytes())))
    }

    /// Serialize transaction to bytes for signing/hashing
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // Create a version without signature for signing
        let mut tx_for_signing = self.clone();
        tx_for_signing.signature = None;

        bincode::serialize(&tx_for_signing).map_err(|e| Error::SerializationError(e.to_string()))
    }

    /// Get the sender account (if applicable)
    pub fn sender(&self) -> Option<&AccountId> {
        match &self.tx_type {
            TransactionType::Transfer { from, .. } => Some(from),
            TransactionType::Burn { from, .. } => Some(from),
            TransactionType::UpdateAccount { account, .. } => Some(account),
            _ => None,
        }
    }

    /// Get the total cost (amount + fee)
    pub fn total_cost(&self) -> Result<rUv> {
        let amount = match &self.tx_type {
            TransactionType::Transfer { amount, .. } => *amount,
            TransactionType::Burn { amount, .. } => *amount,
            _ => rUv::ZERO,
        };

        amount
            .checked_add(self.fee)
            .ok_or_else(|| Error::Other("Transaction cost overflow".into()))
    }

    /// Check if transaction is expired
    pub fn is_expired(&self, current_time: Timestamp) -> bool {
        self.expires_at
            .map(|exp| current_time > exp)
            .unwrap_or(false)
    }

    /// Sign the transaction using QuDAG crypto
    #[cfg(feature = "std")]
    pub fn sign(&mut self, keypair: &qudag_crypto::MlDsaKeyPair) -> Result<()> {
        let message = self.to_bytes()?;

        // Sign the message
        let signature = keypair
            .sign(&message, &mut rand::thread_rng())
            .map_err(|e| Error::Other(format!("Signing failed: {:?}", e)))?;

        let public_key = keypair
            .to_public_key()
            .map_err(|e| Error::Other(format!("Public key extraction failed: {:?}", e)))?;

        self.signature = Some(TransactionSignature {
            algorithm: "ML-DSA-87".to_string(),
            public_key: public_key.as_bytes().to_vec(),
            signature,
        });

        Ok(())
    }

    /// Verify the transaction signature
    #[cfg(feature = "std")]
    pub fn verify_signature(&self) -> Result<bool> {
        let sig_data = self
            .signature
            .as_ref()
            .ok_or_else(|| Error::Other("No signature present".into()))?;

        let message = self.to_bytes()?;

        // Create public key from bytes
        let public_key = qudag_crypto::MlDsaPublicKey::from_bytes(&sig_data.public_key)
            .map_err(|e| Error::Other(format!("Invalid public key: {:?}", e)))?;

        // Verify the signature
        match public_key.verify(&message, &sig_data.signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get current timestamp (platform-specific)
    fn current_timestamp() -> Timestamp {
        #[cfg(feature = "std")]
        {
            Timestamp::now()
        }
        #[cfg(not(feature = "std"))]
        {
            // In no_std, timestamp must be provided externally
            Timestamp::new(0)
        }
    }
}

/// Builder for constructing transactions
pub struct TransactionBuilder {
    tx_type: Option<TransactionType>,
    nonce: Option<Nonce>,
    fee: rUv,
    expires_at: Option<Timestamp>,
    memo: Option<String>,
    chain_id: u64,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new() -> Self {
        Self {
            tx_type: None,
            nonce: None,
            fee: rUv::ZERO,
            expires_at: None,
            memo: None,
            chain_id: 1,
        }
    }

    /// Set transaction type
    pub fn with_type(mut self, tx_type: TransactionType) -> Self {
        self.tx_type = Some(tx_type);
        self
    }

    /// Set nonce
    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set fee
    pub fn with_fee(mut self, fee: rUv) -> Self {
        self.fee = fee;
        self
    }

    /// Set expiry
    pub fn with_expiry(mut self, expires_at: Timestamp) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set memo
    pub fn with_memo(mut self, memo: impl Into<String>) -> Self {
        self.memo = Some(memo.into());
        self
    }

    /// Set chain ID
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Build the transaction
    pub fn build(self) -> Result<Transaction> {
        let tx_type = self
            .tx_type
            .ok_or_else(|| Error::Other("Transaction type not set".into()))?;
        let nonce = self
            .nonce
            .ok_or_else(|| Error::Other("Nonce not set".into()))?;

        Ok(Transaction {
            tx_type,
            nonce,
            timestamp: Transaction::current_timestamp(),
            fee: self.fee,
            expires_at: self.expires_at,
            signature: None,
            metadata: TransactionMetadata {
                memo: self.memo,
                version: 1,
                chain_id: self.chain_id,
            },
        })
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let from = AccountId::new("alice");
        let to = AccountId::new("bob");
        let amount = rUv::new(100);
        let fee = rUv::new(1);
        let nonce = Nonce::new(1);

        let tx = Transaction::transfer(from.clone(), to.clone(), amount, nonce, fee);

        match &tx.tx_type {
            TransactionType::Transfer {
                from: f,
                to: t,
                amount: a,
            } => {
                assert_eq!(f, &from);
                assert_eq!(t, &to);
                assert_eq!(*a, amount);
            }
            _ => panic!("Wrong transaction type"),
        }

        assert_eq!(tx.nonce, nonce);
        assert_eq!(tx.fee, fee);
        assert!(tx.signature.is_none());
    }

    #[test]
    fn test_transaction_builder() {
        let to = AccountId::new("charlie");
        let amount = rUv::new(500);
        let fee = rUv::new(5);
        let nonce = Nonce::new(42);

        let tx = TransactionBuilder::new()
            .with_type(TransactionType::Mint {
                to: to.clone(),
                amount,
            })
            .with_nonce(nonce)
            .with_fee(fee)
            .with_memo("Test mint")
            .with_chain_id(42)
            .build()
            .unwrap();

        assert_eq!(tx.fee, fee);
        assert_eq!(tx.nonce, nonce);
        assert_eq!(tx.metadata.memo.as_deref(), Some("Test mint"));
        assert_eq!(tx.metadata.chain_id, 42);
    }

    #[test]
    fn test_transaction_cost() {
        let from = AccountId::new("alice");
        let to = AccountId::new("bob");
        let amount = rUv::new(100);
        let fee = rUv::new(10);

        let tx = Transaction::transfer(from, to, amount, Nonce::new(1), fee);
        assert_eq!(tx.total_cost().unwrap(), rUv::new(110));

        // Test mint (no amount cost)
        let mint_tx = Transaction::mint(AccountId::new("charlie"), amount, Nonce::new(1), fee);
        assert_eq!(mint_tx.total_cost().unwrap(), fee);
    }

    #[test]
    fn test_transaction_id() {
        let tx1 = Transaction::transfer(
            AccountId::new("alice"),
            AccountId::new("bob"),
            rUv::new(100),
            Nonce::new(1),
            rUv::new(1),
        );

        let tx2 = tx1.clone();

        // Same transactions should have same ID
        assert_eq!(tx1.id().unwrap(), tx2.id().unwrap());

        // Different nonce should give different ID
        let mut tx3 = tx1.clone();
        tx3.nonce = Nonce::new(2);
        assert_ne!(tx1.id().unwrap(), tx3.id().unwrap());
    }

    #[test]
    fn test_transaction_expiry() {
        let mut tx = Transaction::transfer(
            AccountId::new("alice"),
            AccountId::new("bob"),
            rUv::new(100),
            Nonce::new(1),
            rUv::new(1),
        );

        // No expiry by default
        assert!(!tx.is_expired(Timestamp::new(u64::MAX)));

        // Set expiry
        tx.expires_at = Some(Timestamp::new(1000));
        assert!(!tx.is_expired(Timestamp::new(999)));
        assert!(!tx.is_expired(Timestamp::new(1000)));
        assert!(tx.is_expired(Timestamp::new(1001)));
    }
}
