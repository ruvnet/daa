//! Ledger management for QuDAG Exchange
//!
//! Provides the core ledger functionality for tracking account balances
//! and executing transfers with atomic guarantees

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::collections::BTreeMap;

use crate::{
    account::{Account, AccountId, Balance},
    fee_model::{AgentStatus, FeeModel},
    types::{rUv, Timestamp},
    Error, Result,
};
use serde::{Deserialize, Serialize};

/// The main ledger structure that maintains all account states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    /// Map of account IDs to accounts
    #[cfg(not(feature = "std"))]
    accounts: BTreeMap<AccountId, Account>,

    #[cfg(feature = "std")]
    #[serde(skip)]
    accounts: dashmap::DashMap<AccountId, Account>,

    /// Agent status tracking for fee calculation
    #[cfg(not(feature = "std"))]
    agent_statuses: BTreeMap<AccountId, AgentStatus>,

    #[cfg(feature = "std")]
    #[serde(skip)]
    agent_statuses: dashmap::DashMap<AccountId, AgentStatus>,

    /// Total supply of rUv in circulation
    total_supply: rUv,

    /// Configuration for the ledger
    config: LedgerConfig,

    /// Fee model for calculating transaction fees
    #[serde(skip)]
    fee_model: Option<FeeModel>,
}

/// Ledger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerConfig {
    /// Minimum balance required to keep an account active
    pub min_balance: rUv,

    /// Maximum total supply allowed
    pub max_supply: rUv,

    /// Allow negative balances (for special accounts)
    pub allow_negative: bool,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            min_balance: rUv::ZERO,
            max_supply: rUv::new(u64::MAX),
            allow_negative: false,
        }
    }
}

impl Ledger {
    /// Create a new empty ledger
    pub fn new() -> Self {
        Self::with_config(LedgerConfig::default())
    }

    /// Create a new ledger with custom configuration
    pub fn with_config(config: LedgerConfig) -> Self {
        Self {
            #[cfg(not(feature = "std"))]
            accounts: BTreeMap::new(),
            #[cfg(feature = "std")]
            accounts: dashmap::DashMap::new(),
            #[cfg(not(feature = "std"))]
            agent_statuses: BTreeMap::new(),
            #[cfg(feature = "std")]
            agent_statuses: dashmap::DashMap::new(),
            total_supply: rUv::ZERO,
            config,
            fee_model: None,
        }
    }

    /// Create ledger with fee model
    pub fn with_fee_model(config: LedgerConfig, fee_model: FeeModel) -> Self {
        let mut ledger = Self::with_config(config);
        ledger.fee_model = Some(fee_model);
        ledger
    }

    /// Set fee model
    pub fn set_fee_model(&mut self, fee_model: FeeModel) {
        self.fee_model = Some(fee_model);
    }

    /// Get the total supply of rUv
    pub fn total_supply(&self) -> rUv {
        self.total_supply
    }

    /// Get the number of accounts
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Create a new account with zero balance
    pub fn create_account(&mut self, id: AccountId) -> Result<()> {
        #[cfg(not(feature = "std"))]
        {
            if self.accounts.contains_key(&id) {
                return Err(Error::Other("Account already exists".into()));
            }
            self.accounts.insert(id.clone(), Account::new(id));
        }

        #[cfg(feature = "std")]
        {
            if self.accounts.contains_key(&id) {
                return Err(Error::Other("Account already exists".into()));
            }
            self.accounts.insert(id.clone(), Account::new(id));
        }

        Ok(())
    }

    /// Get an account by ID (immutable)
    pub fn get_account(&self, id: &AccountId) -> Result<Account> {
        #[cfg(not(feature = "std"))]
        {
            self.accounts
                .get(id)
                .cloned()
                .ok_or_else(|| Error::AccountNotFound(id.as_str().into()))
        }

        #[cfg(feature = "std")]
        {
            self.accounts
                .get(id)
                .map(|entry| entry.clone())
                .ok_or_else(|| Error::AccountNotFound(id.as_str().into()))
        }
    }

    /// Get account balance
    pub fn get_balance(&self, id: &AccountId) -> Result<Balance> {
        self.get_account(id).map(|acc| acc.balance)
    }

    /// Credit an account (mint new rUv)
    pub fn mint(&mut self, account: &AccountId, amount: rUv) -> Result<()> {
        // Check if minting would exceed max supply
        let new_supply = self
            .total_supply
            .checked_add(amount)
            .ok_or_else(|| Error::Other("Supply overflow".into()))?;

        if new_supply > self.config.max_supply {
            return Err(Error::resource_limit_exceeded(
                "total_supply",
                self.config.max_supply.amount(),
                new_supply.amount(),
            ));
        }

        // Update account
        self.update_account(account, |acc| {
            acc.credit(amount)?;
            Ok(())
        })?;

        // Update total supply
        self.total_supply = new_supply;
        Ok(())
    }

    /// Burn rUv from an account (reduce supply)
    pub fn burn(&mut self, account: &AccountId, amount: rUv) -> Result<()> {
        // Update account
        self.update_account(account, |acc| {
            acc.debit(amount)?;
            Ok(())
        })?;

        // Update total supply
        self.total_supply = self.total_supply.saturating_sub(amount);
        Ok(())
    }

    /// Create agent status for an account
    pub fn create_agent_status(
        &mut self,
        account: &AccountId,
        verified: bool,
        first_tx_time: Timestamp,
    ) -> Result<()> {
        let status = if verified {
            AgentStatus::new_verified(first_tx_time, vec![]) // Empty proof for now
        } else {
            AgentStatus::new_unverified(first_tx_time)
        };

        #[cfg(not(feature = "std"))]
        {
            self.agent_statuses.insert(account.clone(), status);
        }

        #[cfg(feature = "std")]
        {
            self.agent_statuses.insert(account.clone(), status);
        }

        Ok(())
    }

    /// Get agent status
    pub fn get_agent_status(&self, account: &AccountId) -> Result<AgentStatus> {
        #[cfg(not(feature = "std"))]
        {
            self.agent_statuses
                .get(account)
                .cloned()
                .ok_or_else(|| Error::Other("Agent status not found".into()))
        }

        #[cfg(feature = "std")]
        {
            self.agent_statuses
                .get(account)
                .map(|entry| entry.clone())
                .ok_or_else(|| Error::Other("Agent status not found".into()))
        }
    }

    /// Update agent monthly usage
    pub fn update_agent_usage(&mut self, account: &AccountId, monthly_usage: u64) -> Result<()> {
        #[cfg(not(feature = "std"))]
        {
            if let Some(status) = self.agent_statuses.get_mut(account) {
                status.update_usage(monthly_usage);
                Ok(())
            } else {
                Err(Error::Other("Agent status not found".into()))
            }
        }

        #[cfg(feature = "std")]
        {
            if let Some(mut status_ref) = self.agent_statuses.get_mut(account) {
                status_ref.update_usage(monthly_usage);
                Ok(())
            } else {
                Err(Error::Other("Agent status not found".into()))
            }
        }
    }

    /// Verify an agent
    pub fn verify_agent(&mut self, account: &AccountId, proof: Vec<u8>) -> Result<()> {
        #[cfg(not(feature = "std"))]
        {
            if let Some(status) = self.agent_statuses.get_mut(account) {
                status.verify(proof);
                Ok(())
            } else {
                Err(Error::Other("Agent status not found".into()))
            }
        }

        #[cfg(feature = "std")]
        {
            if let Some(mut status_ref) = self.agent_statuses.get_mut(account) {
                status_ref.verify(proof);
                Ok(())
            } else {
                Err(Error::Other("Agent status not found".into()))
            }
        }
    }

    /// Calculate fee for a transaction
    pub fn calculate_fee(
        &self,
        account: &AccountId,
        amount: rUv,
        current_time: Timestamp,
    ) -> Result<rUv> {
        let fee_model = self
            .fee_model
            .as_ref()
            .ok_or_else(|| Error::Other("Fee model not set".into()))?;

        let agent_status = self.get_agent_status(account)?;
        fee_model.calculate_fee_amount(amount, &agent_status, current_time)
    }

    /// Transfer with automatic fee calculation
    pub fn transfer_with_fee(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        amount: rUv,
        current_time: Timestamp,
    ) -> Result<rUv> {
        if from == to {
            return Err(Error::InvalidTransaction("Cannot transfer to self".into()));
        }

        if amount.is_zero() {
            return Err(Error::InvalidTransaction(
                "Cannot transfer zero amount".into(),
            ));
        }

        // Calculate fee
        let fee = self.calculate_fee(from, amount, current_time)?;
        let total_cost = amount
            .checked_add(fee)
            .ok_or_else(|| Error::Other("Transaction cost overflow".into()))?;

        // Check if sender can afford total cost
        let sender_balance = self.get_balance(from)?;
        if sender_balance < total_cost {
            return Err(Error::insufficient_balance(
                from.as_str(),
                total_cost.amount(),
                sender_balance.amount(),
            ));
        }

        // Perform the transfer (amount + fee from sender, amount to recipient, fee burned)
        self.transfer(from, to, amount)?;
        if !fee.is_zero() {
            self.burn(from, fee)?;
        }

        Ok(fee)
    }

    /// Transfer rUv between accounts atomically
    pub fn transfer(&mut self, from: &AccountId, to: &AccountId, amount: rUv) -> Result<()> {
        if from == to {
            return Err(Error::InvalidTransaction("Cannot transfer to self".into()));
        }

        if amount.is_zero() {
            return Err(Error::InvalidTransaction(
                "Cannot transfer zero amount".into(),
            ));
        }

        // For no_std, we need to handle this differently
        #[cfg(not(feature = "std"))]
        {
            // First check if both accounts exist
            if !self.accounts.contains_key(from) {
                return Err(Error::AccountNotFound(from.as_str().into()));
            }
            if !self.accounts.contains_key(to) {
                return Err(Error::AccountNotFound(to.as_str().into()));
            }

            // Clone accounts for atomic update
            let mut from_account = self.accounts.get(from).unwrap().clone();
            let mut to_account = self.accounts.get(to).unwrap().clone();

            // Perform transfer
            from_account.debit(amount)?;
            to_account.credit(amount)?;

            // Update both accounts atomically
            self.accounts.insert(from.clone(), from_account);
            self.accounts.insert(to.clone(), to_account);
        }

        #[cfg(feature = "std")]
        {
            // Get both accounts (this locks them)
            let mut from_ref = self
                .accounts
                .get_mut(from)
                .ok_or_else(|| Error::AccountNotFound(from.as_str().into()))?;
            let mut to_ref = self
                .accounts
                .get_mut(to)
                .ok_or_else(|| Error::AccountNotFound(to.as_str().into()))?;

            // Perform the transfer
            from_ref.debit(amount)?;
            to_ref.credit(amount)?;
        }

        Ok(())
    }

    /// Update an account with a closure
    fn update_account<F>(&mut self, id: &AccountId, f: F) -> Result<()>
    where
        F: FnOnce(&mut Account) -> Result<()>,
    {
        #[cfg(not(feature = "std"))]
        {
            let mut account = self
                .accounts
                .get(id)
                .ok_or_else(|| Error::AccountNotFound(id.as_str().into()))?
                .clone();
            f(&mut account)?;
            self.accounts.insert(id.clone(), account);
        }

        #[cfg(feature = "std")]
        {
            let mut account_ref = self
                .accounts
                .get_mut(id)
                .ok_or_else(|| Error::AccountNotFound(id.as_str().into()))?;
            f(&mut account_ref)?;
        }

        Ok(())
    }

    /// Get all account IDs
    pub fn account_ids(&self) -> Vec<AccountId> {
        #[cfg(not(feature = "std"))]
        {
            self.accounts.keys().cloned().collect()
        }

        #[cfg(feature = "std")]
        {
            self.accounts
                .iter()
                .map(|entry| entry.key().clone())
                .collect()
        }
    }

    /// Check invariants (for testing and validation)
    pub fn check_invariants(&self) -> Result<()> {
        // Calculate total balance across all accounts
        let mut total_balance = rUv::ZERO;

        #[cfg(not(feature = "std"))]
        let accounts_iter = self.accounts.values();

        #[cfg(feature = "std")]
        let accounts: Vec<_> = self
            .accounts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        for account in accounts.iter() {
            total_balance = total_balance.checked_add(account.balance).ok_or_else(|| {
                Error::StateCorruption("Balance overflow in invariant check".into())
            })?;
        }

        // Total balance should equal total supply
        if total_balance != self.total_supply {
            return Err(Error::StateCorruption(
                format!(
                    "Total balance {} != total supply {}",
                    total_balance.amount(),
                    self.total_supply.amount()
                )
                .into(),
            ));
        }

        Ok(())
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_creation() {
        let ledger = Ledger::new();
        assert_eq!(ledger.total_supply(), rUv::ZERO);
        assert_eq!(ledger.account_count(), 0);
    }

    #[test]
    fn test_account_creation() {
        let mut ledger = Ledger::new();
        let alice = AccountId::new("alice");

        ledger.create_account(alice.clone()).unwrap();
        assert_eq!(ledger.account_count(), 1);

        // Cannot create duplicate
        assert!(ledger.create_account(alice.clone()).is_err());
    }

    #[test]
    fn test_mint_and_burn() {
        let mut ledger = Ledger::new();
        let alice = AccountId::new("alice");

        ledger.create_account(alice.clone()).unwrap();

        // Mint
        ledger.mint(&alice, rUv::new(1000)).unwrap();
        assert_eq!(ledger.get_balance(&alice).unwrap(), rUv::new(1000));
        assert_eq!(ledger.total_supply(), rUv::new(1000));

        // Burn
        ledger.burn(&alice, rUv::new(300)).unwrap();
        assert_eq!(ledger.get_balance(&alice).unwrap(), rUv::new(700));
        assert_eq!(ledger.total_supply(), rUv::new(700));

        // Cannot burn more than balance
        assert!(ledger.burn(&alice, rUv::new(1000)).is_err());
    }

    #[test]
    fn test_transfer() {
        let mut ledger = Ledger::new();
        let alice = AccountId::new("alice");
        let bob = AccountId::new("bob");

        ledger.create_account(alice.clone()).unwrap();
        ledger.create_account(bob.clone()).unwrap();

        // Mint to alice
        ledger.mint(&alice, rUv::new(1000)).unwrap();

        // Transfer
        ledger.transfer(&alice, &bob, rUv::new(300)).unwrap();
        assert_eq!(ledger.get_balance(&alice).unwrap(), rUv::new(700));
        assert_eq!(ledger.get_balance(&bob).unwrap(), rUv::new(300));

        // Total supply unchanged
        assert_eq!(ledger.total_supply(), rUv::new(1000));

        // Cannot transfer more than balance
        assert!(ledger.transfer(&alice, &bob, rUv::new(1000)).is_err());

        // Cannot transfer to self
        assert!(ledger.transfer(&alice, &alice, rUv::new(100)).is_err());

        // Cannot transfer zero
        assert!(ledger.transfer(&alice, &bob, rUv::ZERO).is_err());
    }

    #[test]
    fn test_invariants() {
        let mut ledger = Ledger::new();
        let alice = AccountId::new("alice");
        let bob = AccountId::new("bob");
        let charlie = AccountId::new("charlie");

        ledger.create_account(alice.clone()).unwrap();
        ledger.create_account(bob.clone()).unwrap();
        ledger.create_account(charlie.clone()).unwrap();

        // Initial state
        ledger.check_invariants().unwrap();

        // After minting
        ledger.mint(&alice, rUv::new(1000)).unwrap();
        ledger.mint(&bob, rUv::new(500)).unwrap();
        ledger.check_invariants().unwrap();

        // After transfers
        ledger.transfer(&alice, &bob, rUv::new(200)).unwrap();
        ledger.transfer(&bob, &charlie, rUv::new(100)).unwrap();
        ledger.check_invariants().unwrap();

        // After burning
        ledger.burn(&alice, rUv::new(300)).unwrap();
        ledger.check_invariants().unwrap();

        // Verify final state
        assert_eq!(ledger.total_supply(), rUv::new(1200)); // 1500 - 300
    }
}
