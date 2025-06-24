//! Account management for QuDAG Exchange
//!
//! Handles user accounts, balances, and identity management

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

use crate::{
    types::{rUv, Hash, Nonce},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// Account identifier - a unique ID for each account
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AccountId(String);

impl AccountId {
    /// Create a new account ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create from public key bytes
    pub fn from_public_key(public_key: &[u8]) -> Self {
        let hash = blake3::hash(public_key);
        let hex = hash.to_hex();
        Self(format!("acc_{}", &hex[..16]))
    }
}

impl From<String> for AccountId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AccountId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Balance type alias for clarity
pub type Balance = rUv;

/// Account structure representing a user in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// Unique account identifier
    pub id: AccountId,

    /// Current rUv balance
    pub balance: Balance,

    /// Transaction nonce for ordering
    pub nonce: Nonce,

    /// Public key associated with the account (optional for initial implementation)
    pub public_key: Option<Vec<u8>>,

    /// Account metadata (e.g., creation time, last activity)
    pub metadata: AccountMetadata,
}

/// Account metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountMetadata {
    /// Creation timestamp (milliseconds since epoch)
    pub created_at: u64,

    /// Last activity timestamp
    pub last_active: u64,

    /// Total transactions sent
    pub tx_count: u64,

    /// Account flags (e.g., frozen, privileged)
    pub flags: AccountFlags,
}

/// Account flags for special statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountFlags {
    /// Account is frozen (cannot send transactions)
    pub frozen: bool,

    /// Account has special privileges
    pub privileged: bool,

    /// Account requires additional verification
    pub requires_verification: bool,
}

impl Default for AccountFlags {
    fn default() -> Self {
        Self {
            frozen: false,
            privileged: false,
            requires_verification: false,
        }
    }
}

impl Account {
    /// Create a new account with zero balance
    pub fn new(id: AccountId) -> Self {
        Self::with_balance(id, Balance::ZERO)
    }

    /// Create a new account with initial balance
    pub fn with_balance(id: AccountId, balance: Balance) -> Self {
        let now = Self::current_timestamp();
        Self {
            id,
            balance,
            nonce: Nonce::ZERO,
            public_key: None,
            metadata: AccountMetadata {
                created_at: now,
                last_active: now,
                tx_count: 0,
                flags: AccountFlags::default(),
            },
        }
    }

    /// Credit the account (add balance)
    pub fn credit(&mut self, amount: Balance) -> Result<()> {
        self.balance = self
            .balance
            .checked_add(amount)
            .ok_or_else(|| Error::Other("Balance overflow".into()))?;
        self.update_activity();
        Ok(())
    }

    /// Debit the account (subtract balance)
    pub fn debit(&mut self, amount: Balance) -> Result<()> {
        if self.metadata.flags.frozen {
            return Err(Error::Other("Account is frozen".into()));
        }

        self.balance = self.balance.checked_sub(amount).ok_or_else(|| {
            Error::insufficient_balance(self.id.as_str(), amount.amount(), self.balance.amount())
        })?;
        self.update_activity();
        Ok(())
    }

    /// Check if account can afford an amount
    pub fn can_afford(&self, amount: Balance) -> bool {
        !self.metadata.flags.frozen && self.balance >= amount
    }

    /// Increment nonce and return the new value
    pub fn increment_nonce(&mut self) -> Nonce {
        self.nonce.increment();
        self.update_activity();
        self.nonce
    }

    /// Update last activity timestamp
    fn update_activity(&mut self) {
        self.metadata.last_active = Self::current_timestamp();
    }

    /// Get current timestamp (platform-specific)
    fn current_timestamp() -> u64 {
        #[cfg(feature = "std")]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        }
        #[cfg(not(feature = "std"))]
        {
            // In no_std environment, timestamp must be provided externally
            0
        }
    }

    /// Set the public key for the account
    pub fn set_public_key(&mut self, public_key: Vec<u8>) {
        self.public_key = Some(public_key);
    }

    /// Freeze the account
    pub fn freeze(&mut self) {
        self.metadata.flags.frozen = true;
    }

    /// Unfreeze the account
    pub fn unfreeze(&mut self) {
        self.metadata.flags.frozen = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let id = AccountId::new("alice");
        let account = Account::new(id.clone());

        assert_eq!(account.id, id);
        assert_eq!(account.balance, Balance::ZERO);
        assert_eq!(account.nonce, Nonce::ZERO);
        assert!(!account.metadata.flags.frozen);
    }

    #[test]
    fn test_account_credit_debit() {
        let mut account = Account::new(AccountId::new("bob"));

        // Credit
        account.credit(rUv::new(100)).unwrap();
        assert_eq!(account.balance, rUv::new(100));

        // Debit
        account.debit(rUv::new(30)).unwrap();
        assert_eq!(account.balance, rUv::new(70));

        // Insufficient funds
        let result = account.debit(rUv::new(100));
        assert!(result.is_err());
        assert_eq!(account.balance, rUv::new(70)); // Balance unchanged
    }

    #[test]
    fn test_account_nonce() {
        let mut account = Account::new(AccountId::new("charlie"));
        assert_eq!(account.nonce.value(), 0);

        let new_nonce = account.increment_nonce();
        assert_eq!(new_nonce.value(), 1);
        assert_eq!(account.nonce.value(), 1);

        account.increment_nonce();
        assert_eq!(account.nonce.value(), 2);
    }

    #[test]
    fn test_frozen_account() {
        let mut account = Account::with_balance(AccountId::new("frozen"), rUv::new(1000));

        // Normal operation
        assert!(account.can_afford(rUv::new(100)));
        account.debit(rUv::new(100)).unwrap();

        // Freeze account
        account.freeze();
        assert!(!account.can_afford(rUv::new(100)));
        assert!(account.debit(rUv::new(100)).is_err());

        // Credit still works when frozen
        account.credit(rUv::new(50)).unwrap();
        assert_eq!(account.balance, rUv::new(950));

        // Unfreeze
        account.unfreeze();
        assert!(account.can_afford(rUv::new(100)));
        account.debit(rUv::new(100)).unwrap();
    }

    #[test]
    fn test_account_id_from_public_key() {
        let public_key = b"test_public_key_bytes";
        let id = AccountId::from_public_key(public_key);
        assert!(id.as_str().starts_with("acc_"));
        assert_eq!(id.as_str().len(), 20); // "acc_" + 16 hex chars
    }
}
