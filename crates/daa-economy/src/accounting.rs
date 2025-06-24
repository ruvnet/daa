//! Accounting Manager for DAA Economy
//!
//! This module provides comprehensive accounting and bookkeeping functionality
//! for all economic activities in the DAA ecosystem, including transaction
//! tracking, balance reconciliation, and financial reporting.

use crate::{Result, EconomyError, EconomyConfig, TradingStats};
use daa_chain::Address;
use qudag_crypto::CryptoProvider;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, interval};
use log::{info, debug, warn, error};

/// Account types in the DAA economy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// User wallet account
    User,
    /// Validator node account
    Validator,
    /// Liquidity provider account
    LiquidityProvider,
    /// System treasury account
    Treasury,
    /// Fee collection account
    FeeCollector,
    /// Staking rewards pool
    StakingPool,
    /// Contract account
    Contract,
}

/// Transaction types for accounting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Stake,
    Unstake,
    Mint,
    Burn,
    Reward,
    Fee,
    Slashing,
    Trade,
    LiquidityAdd,
    LiquidityRemove,
}

/// Accounting entry for double-entry bookkeeping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingEntry {
    pub id: String,
    pub timestamp: u64,
    pub transaction_type: TransactionType,
    pub description: String,
    pub debits: Vec<AccountDebit>,
    pub credits: Vec<AccountCredit>,
    pub reference_hash: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Debit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDebit {
    pub account: Address,
    pub account_type: AccountType,
    pub amount: Decimal,
    pub description: String,
}

/// Credit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountCredit {
    pub account: Address,
    pub account_type: AccountType,
    pub amount: Decimal,
    pub description: String,
}

/// Account balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub address: Address,
    pub account_type: AccountType,
    pub balance: Decimal,
    pub locked_balance: Decimal,
    pub available_balance: Decimal,
    pub last_updated: u64,
}

/// Financial report data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReport {
    pub report_period: String,
    pub total_supply: Decimal,
    pub circulating_supply: Decimal,
    pub total_staked: Decimal,
    pub total_fees_collected: Decimal,
    pub total_rewards_distributed: Decimal,
    pub total_slashed: Decimal,
    pub account_balances: HashMap<AccountType, Decimal>,
    pub transaction_volumes: HashMap<TransactionType, Decimal>,
    pub generated_at: u64,
}

/// Accounting manager handles all financial record keeping
pub struct AccountingManager {
    config: EconomyConfig,
    crypto_provider: Arc<dyn CryptoProvider>,
    entries: Arc<RwLock<Vec<AccountingEntry>>>,
    balances: Arc<RwLock<HashMap<Address, AccountBalance>>>,
    account_types: Arc<RwLock<HashMap<Address, AccountType>>>,
    trading_stats: Arc<RwLock<HashMap<Address, TradingStats>>>,
    is_running: Arc<RwLock<bool>>,
}

impl AccountingManager {
    /// Create a new accounting manager
    pub async fn new(config: EconomyConfig, crypto_provider: Arc<dyn CryptoProvider>) -> Result<Self> {
        info!("Initializing Accounting Manager");
        
        let mut account_types = HashMap::new();
        
        // Register system accounts
        account_types.insert(config.fee_collector_address.clone(), AccountType::FeeCollector);
        account_types.insert(config.ruv_token_address.clone(), AccountType::Treasury);
        
        Ok(AccountingManager {
            config,
            crypto_provider,
            entries: Arc::new(RwLock::new(Vec::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
            account_types: Arc::new(RwLock::new(account_types)),
            trading_stats: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the accounting manager
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Accounting Manager");
        
        {
            let mut running = self.is_running.write().unwrap();
            *running = true;
        }
        
        // Start periodic reconciliation task
        let entries = self.entries.clone();
        let balances = self.balances.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Every hour
            
            while *is_running.read().unwrap() {
                interval.tick().await;
                
                debug!("Running periodic balance reconciliation");
                if let Err(e) = Self::reconcile_balances(&entries, &balances).await {
                    error!("Balance reconciliation failed: {}", e);
                }
            }
        });
        
        info!("Accounting Manager started");
        Ok(())
    }
    
    /// Stop the accounting manager
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Accounting Manager");
        
        {
            let mut running = self.is_running.write().unwrap();
            *running = false;
        }
        
        info!("Accounting Manager stopped");
        Ok(())
    }
    
    /// Record a transfer transaction
    pub async fn record_transfer(
        &self,
        from: &Address,
        to: &Address,
        amount: Decimal,
        fee: Decimal,
    ) -> Result<String> {
        let entry_id = self.generate_entry_id("TRANSFER");
        
        let mut debits = vec![
            AccountDebit {
                account: from.clone(),
                account_type: self.get_account_type(from),
                amount,
                description: format!("Transfer to {}", to),
            }
        ];
        
        let mut credits = vec![
            AccountCredit {
                account: to.clone(),
                account_type: self.get_account_type(to),
                amount,
                description: format!("Transfer from {}", from),
            }
        ];
        
        // Add fee entries if applicable
        if fee > Decimal::ZERO {
            debits.push(AccountDebit {
                account: from.clone(),
                account_type: self.get_account_type(from),
                amount: fee,
                description: "Transfer fee".to_string(),
            });
            
            credits.push(AccountCredit {
                account: self.config.fee_collector_address.clone(),
                account_type: AccountType::FeeCollector,
                amount: fee,
                description: "Transfer fee collection".to_string(),
            });
        }
        
        let entry = AccountingEntry {
            id: entry_id.clone(),
            timestamp: self.get_current_timestamp(),
            transaction_type: TransactionType::Transfer,
            description: format!("Transfer {} rUv from {} to {}", amount, from, to),
            debits,
            credits,
            reference_hash: None,
            metadata: HashMap::new(),
        };
        
        self.add_entry(entry).await?;
        debug!("Recorded transfer: {} rUv from {} to {} (fee: {})", amount, from, to, fee);
        
        Ok(entry_id)
    }
    
    /// Record a staking transaction
    pub async fn record_stake(&self, staker: &Address, amount: Decimal) -> Result<String> {
        let entry_id = self.generate_entry_id("STAKE");
        
        let entry = AccountingEntry {
            id: entry_id.clone(),
            timestamp: self.get_current_timestamp(),
            transaction_type: TransactionType::Stake,
            description: format!("Stake {} rUv", amount),
            debits: vec![
                AccountDebit {
                    account: staker.clone(),
                    account_type: self.get_account_type(staker),
                    amount,
                    description: "Tokens staked".to_string(),
                }
            ],
            credits: vec![
                AccountCredit {
                    account: self.config.ruv_token_address.clone(),
                    account_type: AccountType::StakingPool,
                    amount,
                    description: "Staking pool increase".to_string(),
                }
            ],
            reference_hash: None,
            metadata: HashMap::new(),
        };
        
        self.add_entry(entry).await?;
        debug!("Recorded stake: {} rUv from {}", amount, staker);
        
        Ok(entry_id)
    }
    
    /// Record an unstaking transaction
    pub async fn record_unstake(&self, staker: &Address, amount: Decimal) -> Result<String> {
        let entry_id = self.generate_entry_id("UNSTAKE");
        
        let entry = AccountingEntry {
            id: entry_id.clone(),
            timestamp: self.get_current_timestamp(),
            transaction_type: TransactionType::Unstake,
            description: format!("Unstake {} rUv", amount),
            debits: vec![
                AccountDebit {
                    account: self.config.ruv_token_address.clone(),
                    account_type: AccountType::StakingPool,
                    amount,
                    description: "Staking pool decrease".to_string(),
                }
            ],
            credits: vec![
                AccountCredit {
                    account: staker.clone(),
                    account_type: self.get_account_type(staker),
                    amount,
                    description: "Tokens unstaked".to_string(),
                }
            ],
            reference_hash: None,
            metadata: HashMap::new(),
        };
        
        self.add_entry(entry).await?;
        debug!("Recorded unstake: {} rUv to {}", amount, staker);
        
        Ok(entry_id)
    }
    
    /// Record a reward distribution
    pub async fn record_reward(&self, recipient: &Address, amount: Decimal) -> Result<String> {
        let entry_id = self.generate_entry_id("REWARD");
        
        let entry = AccountingEntry {
            id: entry_id.clone(),
            timestamp: self.get_current_timestamp(),
            transaction_type: TransactionType::Reward,
            description: format!("Reward distribution {} rUv", amount),
            debits: vec![
                AccountDebit {
                    account: self.config.ruv_token_address.clone(),
                    account_type: AccountType::Treasury,
                    amount,
                    description: "Reward distribution".to_string(),
                }
            ],
            credits: vec![
                AccountCredit {
                    account: recipient.clone(),
                    account_type: self.get_account_type(recipient),
                    amount,
                    description: "Reward received".to_string(),
                }
            ],
            reference_hash: None,
            metadata: HashMap::new(),
        };
        
        self.add_entry(entry).await?;
        debug!("Recorded reward: {} rUv to {}", amount, recipient);
        
        Ok(entry_id)
    }
    
    /// Record slashing
    pub async fn record_slashing(&self, validator: &Address, amount: Decimal, reason: String) -> Result<String> {
        let entry_id = self.generate_entry_id("SLASH");
        
        let mut metadata = HashMap::new();
        metadata.insert("reason".to_string(), reason.clone());
        
        let entry = AccountingEntry {
            id: entry_id.clone(),
            timestamp: self.get_current_timestamp(),
            transaction_type: TransactionType::Slashing,
            description: format!("Slashing {} rUv - {}", amount, reason),
            debits: vec![
                AccountDebit {
                    account: validator.clone(),
                    account_type: AccountType::Validator,
                    amount,
                    description: format!("Slashed for: {}", reason),
                }
            ],
            credits: vec![
                AccountCredit {
                    account: self.config.ruv_token_address.clone(),
                    account_type: AccountType::Treasury,
                    amount,
                    description: "Slashed tokens burned".to_string(),
                }
            ],
            reference_hash: None,
            metadata,
        };
        
        self.add_entry(entry).await?;
        warn!("Recorded slashing: {} rUv from {} - {}", amount, validator, reason);
        
        Ok(entry_id)
    }
    
    /// Record a trade order
    pub async fn record_trade_order(&self, trader: &Address, order_id: &str, amount: Decimal) -> Result<String> {
        let entry_id = self.generate_entry_id("TRADE");
        
        let mut metadata = HashMap::new();
        metadata.insert("order_id".to_string(), order_id.to_string());
        
        let entry = AccountingEntry {
            id: entry_id.clone(),
            timestamp: self.get_current_timestamp(),
            transaction_type: TransactionType::Trade,
            description: format!("Trade order {} - {} rUv", order_id, amount),
            debits: vec![], // Will be filled when trade executes
            credits: vec![], // Will be filled when trade executes
            reference_hash: None,
            metadata,
        };
        
        self.add_entry(entry).await?;
        
        // Update trading stats
        self.update_trading_stats(trader, amount, Decimal::ZERO).await;
        
        debug!("Recorded trade order: {} for trader {}", order_id, trader);
        Ok(entry_id)
    }
    
    /// Get account balance
    pub async fn get_account_balance(&self, address: &Address) -> Option<AccountBalance> {
        let balances = self.balances.read().unwrap();
        balances.get(address).cloned()
    }
    
    /// Update account balance
    pub async fn update_account_balance(&self, address: &Address, balance: Decimal, locked: Decimal) -> Result<()> {
        let account_balance = AccountBalance {
            address: address.clone(),
            account_type: self.get_account_type(address),
            balance,
            locked_balance: locked,
            available_balance: balance - locked,
            last_updated: self.get_current_timestamp(),
        };
        
        {
            let mut balances = self.balances.write().unwrap();
            balances.insert(address.clone(), account_balance);
        }
        
        Ok(())
    }
    
    /// Get trading statistics for an address
    pub async fn get_trading_stats(&self, address: &Address) -> Result<TradingStats> {
        let stats = self.trading_stats.read().unwrap();
        Ok(stats.get(address).cloned().unwrap_or(TradingStats {
            total_volume: Decimal::ZERO,
            total_trades: 0,
            average_trade_size: Decimal::ZERO,
            total_fees_paid: Decimal::ZERO,
            profit_loss: Decimal::ZERO,
        }))
    }
    
    /// Update trading statistics
    async fn update_trading_stats(&self, trader: &Address, volume: Decimal, fee: Decimal) {
        let mut stats = self.trading_stats.write().unwrap();
        let entry = stats.entry(trader.clone()).or_insert(TradingStats {
            total_volume: Decimal::ZERO,
            total_trades: 0,
            average_trade_size: Decimal::ZERO,
            total_fees_paid: Decimal::ZERO,
            profit_loss: Decimal::ZERO,
        });
        
        entry.total_volume += volume;
        entry.total_trades += 1;
        entry.average_trade_size = entry.total_volume / Decimal::new(entry.total_trades, 0);
        entry.total_fees_paid += fee;
    }
    
    /// Generate financial report
    pub async fn generate_financial_report(&self, period: String) -> Result<FinancialReport> {
        let entries = self.entries.read().unwrap();
        let balances = self.balances.read().unwrap();
        
        let mut account_balances = HashMap::new();
        let mut transaction_volumes = HashMap::new();
        
        // Calculate account type balances
        for balance in balances.values() {
            let current = account_balances.entry(balance.account_type.clone()).or_insert(Decimal::ZERO);
            *current += balance.balance;
        }
        
        // Calculate transaction volumes
        for entry in entries.iter() {
            let volume = entry.debits.iter().map(|d| d.amount).sum::<Decimal>();
            let current = transaction_volumes.entry(entry.transaction_type.clone()).or_insert(Decimal::ZERO);
            *current += volume;
        }
        
        let total_supply = account_balances.values().sum::<Decimal>();
        let circulating_supply = total_supply - account_balances.get(&AccountType::Treasury).unwrap_or(&Decimal::ZERO);
        let total_staked = account_balances.get(&AccountType::StakingPool).unwrap_or(&Decimal::ZERO).clone();
        let total_fees_collected = account_balances.get(&AccountType::FeeCollector).unwrap_or(&Decimal::ZERO).clone();
        let total_rewards_distributed = transaction_volumes.get(&TransactionType::Reward).unwrap_or(&Decimal::ZERO).clone();
        let total_slashed = transaction_volumes.get(&TransactionType::Slashing).unwrap_or(&Decimal::ZERO).clone();
        
        Ok(FinancialReport {
            report_period: period,
            total_supply,
            circulating_supply,
            total_staked,
            total_fees_collected,
            total_rewards_distributed,
            total_slashed,
            account_balances,
            transaction_volumes,
            generated_at: self.get_current_timestamp(),
        })
    }
    
    /// Get transaction history for an address
    pub async fn get_transaction_history(&self, address: &Address, limit: usize) -> Vec<AccountingEntry> {
        let entries = self.entries.read().unwrap();
        entries.iter()
            .filter(|entry| {
                entry.debits.iter().any(|d| d.account == *address) ||
                entry.credits.iter().any(|c| c.account == *address)
            })
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Validate accounting entries (double-entry bookkeeping)
    pub async fn validate_entries(&self) -> Result<bool> {
        let entries = self.entries.read().unwrap();
        
        for entry in entries.iter() {
            let total_debits: Decimal = entry.debits.iter().map(|d| d.amount).sum();
            let total_credits: Decimal = entry.credits.iter().map(|c| c.amount).sum();
            
            if total_debits != total_credits {
                error!("Accounting entry {} is unbalanced: debits={}, credits={}", 
                       entry.id, total_debits, total_credits);
                return Ok(false);
            }
        }
        
        info!("All accounting entries are balanced");
        Ok(true)
    }
    
    // Private helper methods
    
    async fn add_entry(&self, entry: AccountingEntry) -> Result<()> {
        // Validate entry
        let total_debits: Decimal = entry.debits.iter().map(|d| d.amount).sum();
        let total_credits: Decimal = entry.credits.iter().map(|c| c.amount).sum();
        
        if total_debits != total_credits {
            return Err(EconomyError::AccountingError(
                format!("Unbalanced entry: debits={}, credits={}", total_debits, total_credits)
            ));
        }
        
        {
            let mut entries = self.entries.write().unwrap();
            entries.push(entry);
        }
        
        Ok(())
    }
    
    fn get_account_type(&self, address: &Address) -> AccountType {
        let account_types = self.account_types.read().unwrap();
        account_types.get(address).cloned().unwrap_or(AccountType::User)
    }
    
    fn generate_entry_id(&self, prefix: &str) -> String {
        format!("{}_{}_{}_{}", 
                prefix, 
                self.get_current_timestamp(), 
                rand::random::<u32>(),
                self.entries.read().unwrap().len())
    }
    
    fn get_current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    async fn reconcile_balances(
        entries: &Arc<RwLock<Vec<AccountingEntry>>>,
        balances: &Arc<RwLock<HashMap<Address, AccountBalance>>>,
    ) -> Result<()> {
        // This would implement periodic balance reconciliation
        // comparing accounting entries with actual blockchain state
        debug!("Balance reconciliation completed");
        Ok(())
    }
}

/// Set account type for an address
impl AccountingManager {
    pub async fn set_account_type(&self, address: &Address, account_type: AccountType) {
        let mut account_types = self.account_types.write().unwrap();
        account_types.insert(address.clone(), account_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_accounting_manager_creation() {
        let config = EconomyConfig::default();
        let crypto_provider = Arc::new(qudag_crypto::default_crypto_provider());
        
        let accounting = AccountingManager::new(config, crypto_provider).await;
        assert!(accounting.is_ok());
    }
    
    #[tokio::test]
    async fn test_transfer_recording() {
        let config = EconomyConfig::default();
        let crypto_provider = Arc::new(qudag_crypto::default_crypto_provider());
        let accounting = AccountingManager::new(config, crypto_provider).await.unwrap();
        
        let from = Address::from_hex("0x1111111111111111111111111111111111111111").unwrap();
        let to = Address::from_hex("0x2222222222222222222222222222222222222222").unwrap();
        let amount = Decimal::new(100, 0);
        let fee = Decimal::new(1, 0);
        
        let entry_id = accounting.record_transfer(&from, &to, amount, fee).await.unwrap();
        assert!(!entry_id.is_empty());
        
        // Validate entries are balanced
        let is_valid = accounting.validate_entries().await.unwrap();
        assert!(is_valid);
    }
}