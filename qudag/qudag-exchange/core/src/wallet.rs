//! Wallet management for QuDAG Exchange

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::Zeroize;

use crate::error::{Error, Result};
use crate::ruv::RuvAmount;
use crate::transaction::{Transaction, TransactionType};

/// A wallet in the QuDAG Exchange system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    /// Wallet address (derived from public key)
    pub address: String,
    
    /// Current balance
    pub balance: RuvAmount,
    
    /// Transaction history (limited to recent)
    pub transaction_history: Vec<String>,
    
    /// Wallet metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Whether this is a vault-backed wallet
    pub vault_backed: bool,
}

impl Wallet {
    /// Create a new wallet
    pub fn new(address: String) -> Self {
        Self {
            address,
            balance: RuvAmount::default(),
            transaction_history: Vec::new(),
            metadata: HashMap::new(),
            vault_backed: false,
        }
    }

    /// Create a vault-backed wallet
    pub fn new_vault_backed(address: String) -> Self {
        let mut wallet = Self::new(address);
        wallet.vault_backed = true;
        wallet
    }

    /// Get current balance
    pub fn balance(&self) -> &RuvAmount {
        &self.balance
    }

    /// Check if wallet can afford a transaction
    pub fn can_afford(&self, amount: &RuvAmount, fee: &RuvAmount) -> Result<bool> {
        let total = amount.checked_add(fee)?;
        Ok(self.balance >= total)
    }

    /// Apply a transaction to the wallet
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<()> {
        match &tx.tx_type {
            TransactionType::Transfer { from, to, amount } => {
                if &self.address == from {
                    // Deduct amount and fee
                    let total = amount.checked_add(&tx.fee)?;
                    self.balance = self.balance.checked_sub(&total)?;
                } else if &self.address == to {
                    // Add amount
                    self.balance = self.balance.checked_add(amount)?;
                }
            }
            TransactionType::Mint { to, contribution } => {
                if &self.address == to {
                    self.balance = self.balance.checked_add(contribution.total_value())?;
                }
            }
            TransactionType::Burn { from, amount } => {
                if &self.address == from {
                    let total = amount.checked_add(&tx.fee)?;
                    self.balance = self.balance.checked_sub(&total)?;
                }
            }
            TransactionType::FeeDistribution { amount, recipients } => {
                // Find this wallet's share
                for (addr, share) in recipients {
                    if &self.address == addr {
                        let share_amount = (amount.as_ruv() * (*share as u64) / 100);
                        let ruv_share = RuvAmount::from_ruv(share_amount);
                        self.balance = self.balance.checked_add(&ruv_share)?;
                    }
                }
            }
            TransactionType::Execute { .. } => {
                // Contract execution handled separately
            }
        }

        // Add to transaction history (keep last 100)
        self.transaction_history.push(tx.id.clone());
        if self.transaction_history.len() > 100 {
            self.transaction_history.remove(0);
        }

        Ok(())
    }

    /// Add metadata to the wallet
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
}

/// Wallet manager for handling multiple wallets
#[derive(Clone, Debug, Default)]
pub struct WalletManager {
    /// Map of address to wallet
    wallets: HashMap<String, Wallet>,
}

impl WalletManager {
    /// Create a new wallet manager
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
        }
    }

    /// Create a new wallet
    pub fn create_wallet(&mut self, address: String, vault_backed: bool) -> &Wallet {
        let wallet = if vault_backed {
            Wallet::new_vault_backed(address.clone())
        } else {
            Wallet::new(address.clone())
        };
        
        self.wallets.insert(address.clone(), wallet);
        self.wallets.get(&address).unwrap()
    }

    /// Get a wallet by address
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    /// Get a mutable wallet by address
    pub fn get_wallet_mut(&mut self, address: &str) -> Option<&mut Wallet> {
        self.wallets.get_mut(address)
    }

    /// Process a transaction across affected wallets
    pub fn process_transaction(&mut self, tx: &Transaction) -> Result<()> {
        // First verify the transaction
        tx.verify()?;

        // Then apply to affected wallets
        match &tx.tx_type {
            TransactionType::Transfer { from, to, .. } => {
                // Check sender has sufficient balance
                if let Some(sender) = self.get_wallet(from) {
                    if !sender.can_afford(
                        match &tx.tx_type {
                            TransactionType::Transfer { amount, .. } => amount,
                            _ => unreachable!(),
                        },
                        &tx.fee,
                    )? {
                        return Err(Error::InsufficientBalance {
                            required: 0, // TODO: Calculate actual required
                            available: sender.balance().as_ruv() as u128,
                        });
                    }
                }

                // Apply to sender
                if let Some(sender) = self.get_wallet_mut(from) {
                    sender.apply_transaction(tx)?;
                }

                // Apply to recipient
                if let Some(recipient) = self.get_wallet_mut(to) {
                    recipient.apply_transaction(tx)?;
                }
            }
            _ => {
                // Handle other transaction types
                // This is simplified - real implementation would handle all cases
            }
        }

        Ok(())
    }

    /// Get total rUv in all wallets
    pub fn total_supply(&self) -> Result<RuvAmount> {
        let mut total = RuvAmount::default();
        for wallet in self.wallets.values() {
            total = total.checked_add(&wallet.balance)?;
        }
        Ok(total)
    }
}

/// Secure key material (placeholder for actual implementation)
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct WalletKeys {
    /// Private key material
    private_key: Vec<u8>,
    /// Public key material
    public_key: Vec<u8>,
}

impl WalletKeys {
    /// Generate new wallet keys (placeholder)
    pub fn generate() -> Self {
        // TODO: Integrate with QuDAG quantum-resistant crypto
        Self {
            private_key: vec![0; 32],
            public_key: vec![0; 32],
        }
    }

    /// Derive address from public key
    pub fn derive_address(&self) -> String {
        // TODO: Implement proper address derivation
        format!("qudag1{}", hex::encode(&self.public_key[..8]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new("test_address".to_string());
        assert_eq!(wallet.address, "test_address");
        assert!(wallet.balance.is_zero());
        assert!(!wallet.vault_backed);
    }

    #[test]
    fn test_wallet_transaction() {
        let mut wallet = Wallet::new("alice".to_string());
        wallet.balance = RuvAmount::from_ruv(1000);

        let tx = Transaction::new(
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: RuvAmount::from_ruv(100),
            },
            RuvAmount::from_ruv(1),
        );

        wallet.apply_transaction(&tx).unwrap();
        assert_eq!(wallet.balance.as_ruv(), 899); // 1000 - 100 - 1
    }

    #[test]
    fn test_wallet_manager() {
        let mut manager = WalletManager::new();
        
        // Create wallets
        manager.create_wallet("alice".to_string(), false);
        manager.create_wallet("bob".to_string(), false);
        
        // Set initial balance for alice
        if let Some(alice) = manager.get_wallet_mut("alice") {
            alice.balance = RuvAmount::from_ruv(1000);
        }

        // Create transfer
        let tx = Transaction::new(
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: RuvAmount::from_ruv(100),
            },
            RuvAmount::from_ruv(1),
        );

        // Process transaction
        manager.process_transaction(&tx).unwrap();

        // Verify balances
        assert_eq!(manager.get_wallet("alice").unwrap().balance.as_ruv(), 899);
        assert_eq!(manager.get_wallet("bob").unwrap().balance.as_ruv(), 100);
    }
}