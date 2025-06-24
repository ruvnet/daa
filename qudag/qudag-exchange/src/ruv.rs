//! Resource Utilization Voucher (rUv) credit system with quantum-secure operations

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use chrono::{DateTime, Utc};
use crate::error::{ExchangeError, Result};
use crate::security::{InputValidator, SecureBytes};
use qudag_crypto::{MlDsa, MlDsaKeyPair, MlDsaPublicKey};
use uuid::Uuid;

/// rUv credit representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RuvCredit(u64);

impl RuvCredit {
    /// Create new rUv credit amount
    pub fn new(amount: u64) -> Result<Self> {
        InputValidator::validate_amount(amount)?;
        Ok(Self(amount))
    }
    
    /// Get the amount
    pub fn amount(&self) -> u64 {
        self.0
    }
    
    /// Add credits (checked)
    pub fn checked_add(&self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
    
    /// Subtract credits (checked)
    pub fn checked_sub(&self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
    
    /// Multiply by factor (checked)
    pub fn checked_mul(&self, factor: u64) -> Option<Self> {
        self.0.checked_mul(factor).map(Self)
    }
}

/// Transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Transfer credits between accounts
    Transfer,
    /// Reward for resource contribution
    Reward,
    /// Fee payment
    Fee,
    /// Stake for consensus participation
    Stake,
    /// Unstake from consensus
    Unstake,
}

/// rUv transaction with quantum-secure signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuvTransaction {
    /// Unique transaction ID
    pub id: String,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount of rUv credits
    pub amount: RuvCredit,
    /// Transaction fee
    pub fee: RuvCredit,
    /// Nonce for replay prevention
    pub nonce: Vec<u8>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Quantum-resistant signature
    pub signature: Option<Vec<u8>>,
    /// Additional metadata
    pub metadata: Option<TransactionMetadata>,
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    /// Resource type being exchanged
    pub resource_type: String,
    /// Duration of resource usage
    pub duration_seconds: Option<u64>,
    /// Zero-knowledge proof (if applicable)
    pub zkp: Option<Vec<u8>>,
}

/// Secure transaction builder
pub struct TransactionBuilder {
    tx_type: TransactionType,
    from: String,
    to: String,
    amount: RuvCredit,
    fee: RuvCredit,
    metadata: Option<TransactionMetadata>,
}

impl TransactionBuilder {
    /// Create new transaction builder
    pub fn new(tx_type: TransactionType) -> Self {
        Self {
            tx_type,
            from: String::new(),
            to: String::new(),
            amount: RuvCredit(0),
            fee: RuvCredit(0),
            metadata: None,
        }
    }
    
    /// Set sender
    pub fn from(mut self, address: String) -> Result<Self> {
        InputValidator::validate_address(&address)?;
        self.from = address;
        Ok(self)
    }
    
    /// Set recipient
    pub fn to(mut self, address: String) -> Result<Self> {
        InputValidator::validate_address(&address)?;
        self.to = address;
        Ok(self)
    }
    
    /// Set amount
    pub fn amount(mut self, amount: u64) -> Result<Self> {
        self.amount = RuvCredit::new(amount)?;
        Ok(self)
    }
    
    /// Set fee
    pub fn fee(mut self, fee: u64) -> Result<Self> {
        self.fee = RuvCredit::new(fee)?;
        Ok(self)
    }
    
    /// Set metadata
    pub fn metadata(mut self, metadata: TransactionMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Build the transaction
    pub fn build(self) -> Result<RuvTransaction> {
        // Validate all fields
        if self.from.is_empty() || self.to.is_empty() {
            return Err(ExchangeError::TransactionValidation(
                "Missing from or to address".to_string()
            ));
        }
        
        // Generate secure random nonce
        let mut nonce = vec![0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut nonce);
        
        Ok(RuvTransaction {
            id: Uuid::new_v4().to_string(),
            tx_type: self.tx_type,
            from: self.from,
            to: self.to,
            amount: self.amount,
            fee: self.fee,
            nonce,
            timestamp: Utc::now(),
            signature: None,
            metadata: self.metadata,
        })
    }
}

impl RuvTransaction {
    /// Sign the transaction with ML-DSA
    pub fn sign(&mut self, keypair: &MlDsaKeyPair) -> Result<()> {
        let message = self.signing_message()?;
        let mut rng = rand::thread_rng();
        
        let signature = keypair.sign(&message, &mut rng)
            .map_err(|e| ExchangeError::Crypto(e.into()))?;
        
        self.signature = Some(signature);
        Ok(())
    }
    
    /// Verify the transaction signature
    pub fn verify(&self, public_key: &MlDsaPublicKey) -> Result<()> {
        let signature = self.signature.as_ref()
            .ok_or(ExchangeError::InvalidSignature)?;
        
        let message = self.signing_message()?;
        
        public_key.verify(&message, signature)
            .map_err(|_| ExchangeError::InvalidSignature)?;
        
        Ok(())
    }
    
    /// Get the message to sign (excludes signature field)
    fn signing_message(&self) -> Result<Vec<u8>> {
        let mut tx_copy = self.clone();
        tx_copy.signature = None;
        
        bincode::serialize(&tx_copy)
            .map_err(|e| ExchangeError::Serialization(e.to_string()))
    }
    
    /// Calculate total cost (amount + fee)
    pub fn total_cost(&self) -> Option<RuvCredit> {
        self.amount.checked_add(self.fee)
    }
    
    /// Validate transaction
    pub fn validate(&self) -> Result<()> {
        // Validate IDs and addresses
        InputValidator::validate_tx_id(&self.id)?;
        InputValidator::validate_address(&self.from)?;
        InputValidator::validate_address(&self.to)?;
        
        // Check for self-transfer
        if self.from == self.to {
            return Err(ExchangeError::TransactionValidation(
                "Self-transfer not allowed".to_string()
            ));
        }
        
        // Validate amounts
        if self.amount.0 == 0 && self.tx_type == TransactionType::Transfer {
            return Err(ExchangeError::TransactionValidation(
                "Transfer amount must be greater than zero".to_string()
            ));
        }
        
        // Check signature exists
        if self.signature.is_none() {
            return Err(ExchangeError::InvalidSignature);
        }
        
        Ok(())
    }
}

/// Secure wallet for managing rUv credits
#[derive(Debug, ZeroizeOnDrop)]
pub struct RuvWallet {
    /// Wallet address
    pub address: String,
    /// Current balance
    balance: RuvCredit,
    /// Private key (zeroized on drop)
    #[zeroize(drop)]
    private_key: Vec<u8>,
    /// Public key
    public_key: Vec<u8>,
}

impl RuvWallet {
    /// Create new wallet
    pub fn new() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng)
            .map_err(|e| ExchangeError::Crypto(e.into()))?;
        
        // Generate address from public key
        let public_key_bytes = keypair.to_public_key()
            .map_err(|e| ExchangeError::Crypto(e.into()))?
            .to_bytes();
        
        let address = Self::generate_address(&public_key_bytes)?;
        
        Ok(Self {
            address,
            balance: RuvCredit(0),
            private_key: keypair.to_bytes(),
            public_key: public_key_bytes,
        })
    }
    
    /// Generate address from public key
    fn generate_address(public_key: &[u8]) -> Result<String> {
        use qudag_crypto::hash::HashFunction;
        
        let hash = HashFunction::Blake3.hash(public_key);
        let address = format!("qd{}", hex::encode(&hash[..20]));
        
        Ok(address)
    }
    
    /// Get current balance
    pub fn balance(&self) -> RuvCredit {
        self.balance
    }
    
    /// Update balance (internal use only)
    pub(crate) fn update_balance(&mut self, new_balance: RuvCredit) {
        self.balance = new_balance;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ruv_credit_arithmetic() {
        let a = RuvCredit::new(100).unwrap();
        let b = RuvCredit::new(50).unwrap();
        
        assert_eq!(a.checked_add(b), Some(RuvCredit(150)));
        assert_eq!(a.checked_sub(b), Some(RuvCredit(50)));
        assert_eq!(b.checked_sub(a), None); // Underflow protection
        assert_eq!(a.checked_mul(2), Some(RuvCredit(200)));
    }
    
    #[test]
    fn test_transaction_builder() {
        let tx = TransactionBuilder::new(TransactionType::Transfer)
            .from("qd123abc".to_string()).unwrap()
            .to("qd456def".to_string()).unwrap()
            .amount(1000).unwrap()
            .fee(10).unwrap()
            .build()
            .unwrap();
        
        assert_eq!(tx.amount, RuvCredit(1000));
        assert_eq!(tx.fee, RuvCredit(10));
        assert_eq!(tx.total_cost(), Some(RuvCredit(1010)));
    }
    
    #[test]
    fn test_transaction_validation() {
        // Self-transfer should fail
        let result = TransactionBuilder::new(TransactionType::Transfer)
            .from("qd123abc".to_string()).unwrap()
            .to("qd123abc".to_string()).unwrap()
            .amount(100).unwrap()
            .build();
        
        assert!(result.is_ok());
        assert!(result.unwrap().validate().is_err());
    }
    
    #[test]
    fn test_wallet_creation() {
        let wallet = RuvWallet::new().unwrap();
        assert!(wallet.address.starts_with("qd"));
        assert_eq!(wallet.balance(), RuvCredit(0));
    }
}