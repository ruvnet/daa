//! Ledger management for rUv token balances

use crate::{AccountId, ExchangeError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Balance type for rUv tokens
pub type Balance = u64;

/// Thread-safe ledger for managing account balances
pub struct Ledger {
    balances: DashMap<AccountId, Balance>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            balances: DashMap::new(),
        }
    }
    
    /// Get account balance
    pub fn get_balance(&self, account: &AccountId) -> Result<Balance> {
        self.balances
            .get(account)
            .map(|entry| *entry.value())
            .ok_or_else(|| ExchangeError::AccountNotFound(account.clone()))
    }
    
    /// Credit an account
    pub fn credit(&self, account: &AccountId, amount: Balance) -> Result<()> {
        self.balances
            .entry(account.clone())
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
        Ok(())
    }
    
    /// Debit an account
    pub fn debit(&self, account: &AccountId, amount: Balance) -> Result<()> {
        let mut entry = self.balances
            .get_mut(account)
            .ok_or_else(|| ExchangeError::AccountNotFound(account.clone()))?;
            
        let balance = entry.value_mut();
        if *balance < amount {
            return Err(ExchangeError::InsufficientBalance {
                required: amount,
                available: *balance,
            });
        }
        
        *balance -= amount;
        Ok(())
    }
    
    /// Transfer between accounts
    pub fn transfer(&self, from: &AccountId, to: &AccountId, amount: Balance) -> Result<()> {
        // Atomic transfer - debit first, then credit
        self.debit(from, amount)?;
        self.credit(to, amount)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub balances: Vec<(AccountId, Balance)>,
    pub total_supply: Balance,
}