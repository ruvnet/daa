//! DAA Economy - Economic layer and rUv token management with QuDAG integration
//!
//! This crate provides economic primitives, token management, and incentive mechanisms
//! built on top of the QuDAG exchange infrastructure.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use rust_decimal::Decimal;

// QuDAG imports
use qudag_exchange::{Exchange, Token, Order, OrderType, OrderStatus, TradeEvent};
use qudag_core::NodeId;
use qudag_crypto::CryptoProvider;

// DAA imports
use daa_chain::{Address, TxHash, BlockchainAdapter};

pub mod ruv_token;
pub mod incentives;
pub mod accounting;
pub mod fees;
pub mod marketplace;
pub mod slashing;

pub use ruv_token::*;
pub use incentives::*;
pub use accounting::*;
pub use fees::*;
pub use marketplace::*;
pub use slashing::*;

/// Economic layer errors
#[derive(Error, Debug)]
pub enum EconomyError {
    #[error("Exchange error: {0}")]
    ExchangeError(String),
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: Decimal, available: Decimal },
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Incentive calculation failed: {0}")]
    IncentiveError(String),
    
    #[error("Fee calculation failed: {0}")]
    FeeError(String),
    
    #[error("Accounting error: {0}")]
    AccountingError(String),
    
    #[error("Market operation failed: {0}")]
    MarketError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, EconomyError>;

/// Economy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub ruv_token_address: Address,
    pub exchange_address: Address,
    pub fee_collector_address: Address,
    pub base_fee_rate: Decimal,
    pub validator_reward_rate: Decimal,
    pub staking_reward_rate: Decimal,
    pub slashing_rate: Decimal,
    pub min_stake_amount: Decimal,
    pub voting_power_multiplier: Decimal,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        EconomyConfig {
            ruv_token_address: Address::from_hex("0x0000000000000000000000000000000000000001").unwrap(),
            exchange_address: Address::from_hex("0x0000000000000000000000000000000000000002").unwrap(),
            fee_collector_address: Address::from_hex("0x0000000000000000000000000000000000000003").unwrap(),
            base_fee_rate: Decimal::new(1, 3), // 0.1%
            validator_reward_rate: Decimal::new(5, 2), // 5%
            staking_reward_rate: Decimal::new(8, 2), // 8%
            slashing_rate: Decimal::new(10, 2), // 10%
            min_stake_amount: Decimal::new(1000, 0), // 1000 rUv
            voting_power_multiplier: Decimal::new(1, 0), // 1:1
        }
    }
}

/// Main economy manager integrating QuDAG exchange
pub struct EconomyManager {
    config: EconomyConfig,
    exchange: Arc<Exchange>,
    ruv_token: Arc<RuvTokenManager>,
    incentive_engine: Arc<IncentiveEngine>,
    accounting: AccountingManager,
    fee_manager: Arc<FeeManager>,
    marketplace: Arc<ComputeMarketplace>,
    slashing_manager: Arc<SlashingManager>,
    blockchain_adapter: Arc<dyn BlockchainAdapter>,
    crypto_provider: Arc<dyn CryptoProvider>,
    balances: Arc<RwLock<HashMap<Address, Decimal>>>,
    stakes: Arc<RwLock<HashMap<Address, StakeInfo>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub amount: Decimal,
    pub locked_until: u64,
    pub rewards_earned: Decimal,
    pub slashing_applied: Decimal,
    pub voting_power: Decimal,
}

impl EconomyManager {
    /// Create a new economy manager
    pub async fn new(
        config: EconomyConfig,
        blockchain_adapter: Arc<dyn BlockchainAdapter>,
        crypto_provider: Arc<dyn CryptoProvider>,
    ) -> Result<Self> {
        log::info!("Initializing Economy Manager");
        
        // Initialize QuDAG exchange
        let exchange = Exchange::new(crypto_provider.clone())
            .await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Setup rUv token
        let ruv_token = RuvToken {
            address: config.ruv_token_address.clone(),
            symbol: "rUv".to_string(),
            decimals: 18,
            total_supply: Decimal::new(1_000_000_000, 18), // 1B rUv max supply
            circulating_supply: Decimal::ZERO,
        };
        
        let ruv_token_manager = Arc::new(RuvTokenManager::new(
            ruv_token,
            blockchain_adapter.clone(),
            crypto_provider.clone(),
        ).await?);
        
        // Initialize other components
        let incentive_engine = Arc::new(IncentiveEngine::new(config.clone()));
        let fee_manager = Arc::new(FeeManager::new(config.clone()));
        let accounting = AccountingManager::new(config.clone(), crypto_provider.clone()).await?;
        
        // Initialize marketplace
        let marketplace = Arc::new(ComputeMarketplace::new(
            Arc::new(exchange.clone()),
            ruv_token_manager.clone(),
            fee_manager.clone(),
            blockchain_adapter.clone(),
            crypto_provider.clone(),
        ).await?);
        
        // Initialize slashing manager
        let slashing_params = SlashingParams::default();
        let slashing_manager = Arc::new(SlashingManager::new(
            config.clone(),
            slashing_params,
            ruv_token_manager.clone(),
            incentive_engine.clone(),
            blockchain_adapter.clone(),
        ).await?);
        
        Ok(EconomyManager {
            config,
            exchange: Arc::new(exchange),
            ruv_token: ruv_token_manager,
            incentive_engine,
            accounting,
            fee_manager,
            marketplace,
            slashing_manager,
            blockchain_adapter,
            crypto_provider,
            balances: Arc::new(RwLock::new(HashMap::new())),
            stakes: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Start the economy manager
    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting Economy Manager");
        
        // Register rUv token with exchange
        let ruv_token_info = Token {
            address: self.config.ruv_token_address.as_bytes().to_vec(),
            symbol: "rUv".to_string(),
            decimals: 18,
            total_supply: 1_000_000_000_000_000_000_000_000_000u128, // 1B with 18 decimals
        };
        
        self.exchange.register_token(ruv_token_info)
            .await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Start accounting manager
        self.accounting.start().await?;
        
        log::info!("Economy Manager started successfully");
        Ok(())
    }
    
    /// Stop the economy manager
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("Stopping Economy Manager");
        
        self.accounting.stop().await?;
        
        log::info!("Economy Manager stopped");
        Ok(())
    }
    
    /// Get rUv balance for an address
    pub async fn get_ruv_balance(&self, address: &Address) -> Result<Decimal> {
        self.ruv_token.get_balance(address).await
    }
    
    /// Transfer rUv tokens
    pub async fn transfer_ruv(
        &self,
        from: &Address,
        to: &Address,
        amount: Decimal,
    ) -> Result<TxHash> {
        // Calculate fees
        let fee = self.fee_manager.calculate_transfer_fee(amount)?;
        let total_required = amount + fee;
        
        // Check balance
        let balance = self.get_ruv_balance(from).await?;
        if balance < total_required {
            return Err(EconomyError::InsufficientBalance {
                required: total_required,
                available: balance,
            });
        }
        
        // Execute transfer
        let tx_hash = self.ruv_token.transfer(from, to, amount).await?;
        
        // Collect fee
        if fee > Decimal::ZERO {
            self.ruv_token.transfer(from, &self.config.fee_collector_address, fee).await?;
        }
        
        // Record in accounting
        self.accounting.record_transfer(from, to, amount, fee).await?;
        
        log::info!("rUv transfer completed: {} -> {} (amount: {}, fee: {})", from, to, amount, fee);
        Ok(tx_hash)
    }
    
    /// Stake rUv tokens
    pub async fn stake_ruv(&self, staker: &Address, amount: Decimal) -> Result<TxHash> {
        if amount < self.config.min_stake_amount {
            return Err(EconomyError::ConfigurationError(
                format!("Minimum stake amount is {}", self.config.min_stake_amount)
            ));
        }
        
        // Check balance
        let balance = self.get_ruv_balance(staker).await?;
        if balance < amount {
            return Err(EconomyError::InsufficientBalance {
                required: amount,
                available: balance,
            });
        }
        
        // Calculate voting power
        let voting_power = amount * self.config.voting_power_multiplier;
        
        // Lock tokens
        let tx_hash = self.ruv_token.lock(staker, amount).await?;
        
        // Record stake
        let stake_info = StakeInfo {
            amount,
            locked_until: self.get_current_timestamp() + (30 * 24 * 60 * 60), // 30 days
            rewards_earned: Decimal::ZERO,
            slashing_applied: Decimal::ZERO,
            voting_power,
        };
        
        {
            let mut stakes = self.stakes.write().unwrap();
            stakes.insert(staker.clone(), stake_info);
        }
        
        // Record in accounting
        self.accounting.record_stake(staker, amount).await?;
        
        log::info!("Staked {} rUv for address {}", amount, staker);
        Ok(tx_hash)
    }
    
    /// Unstake rUv tokens
    pub async fn unstake_ruv(&self, staker: &Address) -> Result<TxHash> {
        let stake_info = {
            let stakes = self.stakes.read().unwrap();
            stakes.get(staker).cloned()
                .ok_or_else(|| EconomyError::ConfigurationError("No stake found".to_string()))?
        };
        
        // Check if lock period has expired
        let current_time = self.get_current_timestamp();
        if current_time < stake_info.locked_until {
            return Err(EconomyError::ConfigurationError(
                "Stake is still locked".to_string()
            ));
        }
        
        // Calculate final amount after slashing
        let final_amount = stake_info.amount - stake_info.slashing_applied + stake_info.rewards_earned;
        
        // Unlock tokens
        let tx_hash = self.ruv_token.unlock(staker, final_amount).await?;
        
        // Remove stake
        {
            let mut stakes = self.stakes.write().unwrap();
            stakes.remove(staker);
        }
        
        // Record in accounting
        self.accounting.record_unstake(staker, final_amount).await?;
        
        log::info!("Unstaked {} rUv for address {}", final_amount, staker);
        Ok(tx_hash)
    }
    
    /// Create a market order on the exchange
    pub async fn create_market_order(
        &self,
        trader: &Address,
        base_token: &Address,
        quote_token: &Address,
        order_type: QuDAGOrderType,
        amount: Decimal,
        price: Option<Decimal>,
    ) -> Result<String> {
        // Convert to QuDAG order type
        let qudag_order_type = match order_type {
            QuDAGOrderType::Buy => OrderType::Buy,
            QuDAGOrderType::Sell => OrderType::Sell,
        };
        
        // Create order
        let order = Order {
            id: format!("order_{}", rand::random::<u64>()),
            trader: trader.as_bytes().to_vec(),
            base_token: base_token.as_bytes().to_vec(),
            quote_token: quote_token.as_bytes().to_vec(),
            order_type: qudag_order_type,
            amount: amount.to_string().parse().unwrap(),
            price: price.map(|p| p.to_string().parse().unwrap()),
            status: OrderStatus::Pending,
            timestamp: self.get_current_timestamp(),
        };
        
        // Submit to exchange
        let order_id = self.exchange.submit_order(order)
            .await
            .map_err(|e| EconomyError::ExchangeError(e.to_string()))?;
        
        // Record in accounting
        self.accounting.record_trade_order(trader, &order_id, amount).await?;
        
        log::info!("Created market order {} for trader {}", order_id, trader);
        Ok(order_id)
    }
    
    /// Get trading statistics
    pub async fn get_trading_stats(&self, address: &Address) -> Result<TradingStats> {
        self.accounting.get_trading_stats(address).await
    }
    
    /// Calculate and distribute rewards
    pub async fn distribute_rewards(&self) -> Result<Vec<TxHash>> {
        log::info!("Distributing staking rewards");
        
        let mut tx_hashes = Vec::new();
        let stakes = self.stakes.read().unwrap().clone();
        
        for (staker, stake_info) in stakes {
            // Calculate rewards
            let reward = self.incentive_engine.calculate_staking_reward(
                stake_info.amount,
                stake_info.voting_power,
            )?;
            
            if reward > Decimal::ZERO {
                // Mint reward tokens
                let tx_hash = self.ruv_token.mint(&staker, reward).await?;
                tx_hashes.push(tx_hash);
                
                // Update stake info
                {
                    let mut stakes = self.stakes.write().unwrap();
                    if let Some(stake) = stakes.get_mut(&staker) {
                        stake.rewards_earned += reward;
                    }
                }
                
                // Record in accounting
                self.accounting.record_reward(&staker, reward).await?;
                
                log::debug!("Distributed {} rUv reward to {}", reward, staker);
            }
        }
        
        log::info!("Distributed rewards to {} stakers", tx_hashes.len());
        Ok(tx_hashes)
    }
    
    /// Apply slashing to a validator
    pub async fn apply_slashing(&self, validator: &Address, reason: String) -> Result<TxHash> {
        log::warn!("Applying slashing to validator {} for: {}", validator, reason);
        
        let stake_info = {
            let stakes = self.stakes.read().unwrap();
            stakes.get(validator).cloned()
                .ok_or_else(|| EconomyError::ConfigurationError("No stake found".to_string()))?
        };
        
        // Calculate slashing amount
        let slashing_amount = stake_info.amount * self.config.slashing_rate;
        
        // Burn slashed tokens
        let tx_hash = self.ruv_token.burn(validator, slashing_amount).await?;
        
        // Update stake info
        {
            let mut stakes = self.stakes.write().unwrap();
            if let Some(stake) = stakes.get_mut(validator) {
                stake.slashing_applied += slashing_amount;
            }
        }
        
        // Record in accounting
        self.accounting.record_slashing(validator, slashing_amount, reason).await?;
        
        log::info!("Applied slashing of {} rUv to validator {}", slashing_amount, validator);
        Ok(tx_hash)
    }
    
    /// Get marketplace instance
    pub fn marketplace(&self) -> &Arc<ComputeMarketplace> {
        &self.marketplace
    }
    
    /// Get slashing manager instance
    pub fn slashing_manager(&self) -> &Arc<SlashingManager> {
        &self.slashing_manager
    }
    
    /// Get rUv token manager instance
    pub fn ruv_token_manager(&self) -> &Arc<RuvTokenManager> {
        &self.ruv_token
    }
    
    /// Get incentive engine instance
    pub fn incentive_engine(&self) -> &Arc<IncentiveEngine> {
        &self.incentive_engine
    }
    
    /// Get current timestamp
    fn get_current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuDAGOrderType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStats {
    pub total_volume: Decimal,
    pub total_trades: u64,
    pub average_trade_size: Decimal,
    pub total_fees_paid: Decimal,
    pub profit_loss: Decimal,
}

/// Economy manager trait for external integration
#[async_trait]
pub trait EconomyInterface: Send + Sync {
    async fn get_balance(&self, address: &Address) -> Result<Decimal>;
    async fn transfer(&self, from: &Address, to: &Address, amount: Decimal) -> Result<TxHash>;
    async fn stake(&self, staker: &Address, amount: Decimal) -> Result<TxHash>;
    async fn unstake(&self, staker: &Address) -> Result<TxHash>;
    async fn get_trading_stats(&self, address: &Address) -> Result<TradingStats>;
}

#[async_trait]
impl EconomyInterface for EconomyManager {
    async fn get_balance(&self, address: &Address) -> Result<Decimal> {
        self.get_ruv_balance(address).await
    }
    
    async fn transfer(&self, from: &Address, to: &Address, amount: Decimal) -> Result<TxHash> {
        self.transfer_ruv(from, to, amount).await
    }
    
    async fn stake(&self, staker: &Address, amount: Decimal) -> Result<TxHash> {
        self.stake_ruv(staker, amount).await
    }
    
    async fn unstake(&self, staker: &Address) -> Result<TxHash> {
        self.unstake_ruv(staker).await
    }
    
    async fn get_trading_stats(&self, address: &Address) -> Result<TradingStats> {
        self.get_trading_stats(address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daa_chain::{QuDAGAdapter, QuDAGConfig};
    
    #[tokio::test]
    async fn test_economy_manager_creation() {
        let config = EconomyConfig::default();
        let chain_config = daa_chain::ChainConfig {
            chain_id: 1337,
            name: "QuDAG".to_string(),
            rpc_url: "http://localhost:8545".to_string(),
            explorer_url: None,
            native_token_symbol: "rUv".to_string(),
            native_token_decimals: 18,
        };
        
        let qudag_config = QuDAGConfig::from_chain_config(chain_config);
        let adapter = QuDAGAdapter::new(qudag_config).await.unwrap();
        let crypto_provider = Arc::new(qudag_crypto::default_crypto_provider());
        
        let economy = EconomyManager::new(
            config,
            Arc::new(adapter),
            crypto_provider,
        ).await;
        
        assert!(economy.is_ok());
    }
}