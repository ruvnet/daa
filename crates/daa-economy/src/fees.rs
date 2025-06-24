//! Fee Management for DAA Economy
//!
//! This module handles all fee calculations and collection mechanisms
//! integrated with QuDAG's built-in fee models.

use crate::{Result, EconomyError, EconomyConfig};
use daa_chain::Address;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, debug};

/// Types of fees in the DAA economy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FeeType {
    /// Basic transaction fees
    Transfer,
    /// Smart contract execution fees
    ContractExecution,
    /// Staking operation fees
    Staking,
    /// Trading fees on the exchange
    Trading,
    /// Liquidity provision fees
    LiquidityProvision,
    /// Domain registration fees
    DomainRegistration,
    /// Governance voting fees
    Governance,
    /// Validator registration fees
    ValidatorRegistration,
    /// Cross-chain bridge fees
    CrossChain,
}

/// Fee calculation models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeeModel {
    /// Fixed fee amount
    Fixed(Decimal),
    /// Percentage of transaction amount
    Percentage(Decimal),
    /// Dynamic fee based on network congestion
    Dynamic {
        base_fee: Decimal,
        multiplier: Decimal,
        max_fee: Decimal,
    },
    /// Tiered fee based on amount ranges
    Tiered(Vec<FeeTier>),
    /// Gas-based fee model (like Ethereum)
    Gas {
        gas_price: Decimal,
        gas_limit: u64,
    },
}

/// Fee tier for tiered pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeTier {
    pub min_amount: Decimal,
    pub max_amount: Option<Decimal>,
    pub fee_rate: Decimal,
}

/// Fee configuration for different operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeConfig {
    pub fee_type: FeeType,
    pub model: FeeModel,
    pub collector: Address,
    pub enabled: bool,
}

/// Network congestion metrics for dynamic fee calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub transaction_pool_size: u64,
    pub average_block_utilization: Decimal,
    pub validator_count: u64,
    pub network_hashrate: Decimal,
    pub last_updated: u64,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        NetworkMetrics {
            transaction_pool_size: 0,
            average_block_utilization: Decimal::new(5, 1), // 50%
            validator_count: 10,
            network_hashrate: Decimal::new(1000, 0),
            last_updated: 0,
        }
    }
}

/// Fee manager handles all fee calculations and collections
pub struct FeeManager {
    config: EconomyConfig,
    fee_configs: HashMap<FeeType, FeeConfig>,
    network_metrics: NetworkMetrics,
    fee_history: Vec<FeeRecord>,
}

/// Record of fee collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeRecord {
    pub fee_type: FeeType,
    pub amount: Decimal,
    pub payer: Address,
    pub collector: Address,
    pub transaction_hash: Option<String>,
    pub timestamp: u64,
}

impl FeeManager {
    /// Create a new fee manager
    pub fn new(config: EconomyConfig) -> Self {
        let mut fee_configs = HashMap::new();
        
        // Initialize default fee configurations
        fee_configs.insert(FeeType::Transfer, FeeConfig {
            fee_type: FeeType::Transfer,
            model: FeeModel::Percentage(config.base_fee_rate),
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::ContractExecution, FeeConfig {
            fee_type: FeeType::ContractExecution,
            model: FeeModel::Gas {
                gas_price: Decimal::new(1000000000, 0), // 1 Gwei
                gas_limit: 21000,
            },
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::Staking, FeeConfig {
            fee_type: FeeType::Staking,
            model: FeeModel::Fixed(Decimal::new(1, 0)), // 1 rUv
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::Trading, FeeConfig {
            fee_type: FeeType::Trading,
            model: FeeModel::Tiered(vec![
                FeeTier {
                    min_amount: Decimal::ZERO,
                    max_amount: Some(Decimal::new(1000, 0)),
                    fee_rate: Decimal::new(3, 3), // 0.3%
                },
                FeeTier {
                    min_amount: Decimal::new(1000, 0),
                    max_amount: Some(Decimal::new(10000, 0)),
                    fee_rate: Decimal::new(25, 4), // 0.25%
                },
                FeeTier {
                    min_amount: Decimal::new(10000, 0),
                    max_amount: None,
                    fee_rate: Decimal::new(2, 3), // 0.2%
                },
            ]),
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::LiquidityProvision, FeeConfig {
            fee_type: FeeType::LiquidityProvision,
            model: FeeModel::Percentage(Decimal::new(1, 3)), // 0.1%
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::DomainRegistration, FeeConfig {
            fee_type: FeeType::DomainRegistration,
            model: FeeModel::Fixed(Decimal::new(10, 0)), // 10 rUv
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::Governance, FeeConfig {
            fee_type: FeeType::Governance,
            model: FeeModel::Fixed(Decimal::new(1, 1)), // 0.1 rUv
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::ValidatorRegistration, FeeConfig {
            fee_type: FeeType::ValidatorRegistration,
            model: FeeModel::Fixed(Decimal::new(1000, 0)), // 1000 rUv
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        fee_configs.insert(FeeType::CrossChain, FeeConfig {
            fee_type: FeeType::CrossChain,
            model: FeeModel::Dynamic {
                base_fee: Decimal::new(5, 0),
                multiplier: Decimal::new(15, 1), // 1.5x
                max_fee: Decimal::new(100, 0),
            },
            collector: config.fee_collector_address.clone(),
            enabled: true,
        });
        
        FeeManager {
            config,
            fee_configs,
            network_metrics: NetworkMetrics::default(),
            fee_history: Vec::new(),
        }
    }
    
    /// Calculate transfer fee
    pub fn calculate_transfer_fee(&self, amount: Decimal) -> Result<Decimal> {
        self.calculate_fee(FeeType::Transfer, amount, None)
    }
    
    /// Calculate trading fee
    pub fn calculate_trading_fee(&self, trade_amount: Decimal) -> Result<Decimal> {
        self.calculate_fee(FeeType::Trading, trade_amount, None)
    }
    
    /// Calculate staking fee
    pub fn calculate_staking_fee(&self, stake_amount: Decimal) -> Result<Decimal> {
        self.calculate_fee(FeeType::Staking, stake_amount, None)
    }
    
    /// Calculate contract execution fee
    pub fn calculate_contract_fee(&self, gas_used: u64) -> Result<Decimal> {
        self.calculate_fee(FeeType::ContractExecution, Decimal::new(gas_used, 0), None)
    }
    
    /// Calculate domain registration fee
    pub fn calculate_domain_fee(&self, domain_length: usize) -> Result<Decimal> {
        let base_fee = self.calculate_fee(FeeType::DomainRegistration, Decimal::ZERO, None)?;
        
        // Apply length-based pricing
        let length_multiplier = match domain_length {
            1..=3 => Decimal::new(10, 0), // 10x for short domains
            4..=5 => Decimal::new(5, 0),  // 5x for medium domains
            6..=10 => Decimal::new(2, 0), // 2x for normal domains
            _ => Decimal::new(1, 0),      // 1x for long domains
        };
        
        Ok(base_fee * length_multiplier)
    }
    
    /// Generic fee calculation
    pub fn calculate_fee(
        &self,
        fee_type: FeeType,
        amount: Decimal,
        context: Option<HashMap<String, String>>,
    ) -> Result<Decimal> {
        let fee_config = self.fee_configs.get(&fee_type)
            .ok_or_else(|| EconomyError::FeeError(format!("Fee configuration not found for {:?}", fee_type)))?;
        
        if !fee_config.enabled {
            return Ok(Decimal::ZERO);
        }
        
        let fee = match &fee_config.model {
            FeeModel::Fixed(amount) => *amount,
            
            FeeModel::Percentage(rate) => amount * rate,
            
            FeeModel::Dynamic { base_fee, multiplier, max_fee } => {
                let congestion_multiplier = self.calculate_congestion_multiplier();
                let dynamic_fee = base_fee * multiplier * congestion_multiplier;
                dynamic_fee.min(*max_fee)
            },
            
            FeeModel::Tiered(tiers) => {
                self.calculate_tiered_fee(tiers, amount)?
            },
            
            FeeModel::Gas { gas_price, gas_limit } => {
                let gas_used = amount.to_u64().unwrap_or(*gas_limit);
                Decimal::new(gas_used, 0) * gas_price / Decimal::new(10_u64.pow(18), 0) // Convert to rUv
            },
        };
        
        debug!("Calculated fee for {:?}: {} rUv (amount: {})", fee_type, fee, amount);
        Ok(fee)
    }
    
    /// Calculate tiered fee
    fn calculate_tiered_fee(&self, tiers: &[FeeTier], amount: Decimal) -> Result<Decimal> {
        for tier in tiers {
            if amount >= tier.min_amount {
                if let Some(max_amount) = tier.max_amount {
                    if amount <= max_amount {
                        return Ok(amount * tier.fee_rate);
                    }
                } else {
                    // No upper limit for this tier
                    return Ok(amount * tier.fee_rate);
                }
            }
        }
        
        // Default to highest tier if no match
        if let Some(last_tier) = tiers.last() {
            Ok(amount * last_tier.fee_rate)
        } else {
            Ok(Decimal::ZERO)
        }
    }
    
    /// Calculate network congestion multiplier for dynamic fees
    fn calculate_congestion_multiplier(&self) -> Decimal {
        let utilization_factor = self.network_metrics.average_block_utilization;
        let pool_factor = if self.network_metrics.transaction_pool_size > 1000 {
            Decimal::new(15, 1) // 1.5x if pool is large
        } else {
            Decimal::new(1, 0)
        };
        
        let validator_factor = if self.network_metrics.validator_count < 5 {
            Decimal::new(2, 0) // 2x if few validators
        } else {
            Decimal::new(1, 0)
        };
        
        (utilization_factor + pool_factor + validator_factor) / Decimal::new(3, 0)
    }
    
    /// Update network metrics for dynamic fee calculation
    pub fn update_network_metrics(&mut self, metrics: NetworkMetrics) {
        debug!("Updating network metrics: {:?}", metrics);
        self.network_metrics = metrics;
    }
    
    /// Record fee collection
    pub fn record_fee(&mut self, record: FeeRecord) {
        info!("Recording fee: {:?} -> {} rUv from {} to {}", 
              record.fee_type, record.amount, record.payer, record.collector);
        self.fee_history.push(record);
    }
    
    /// Get fee statistics
    pub fn get_fee_statistics(&self) -> HashMap<FeeType, (u64, Decimal)> {
        let mut stats = HashMap::new();
        
        for record in &self.fee_history {
            let entry = stats.entry(record.fee_type.clone()).or_insert((0u64, Decimal::ZERO));
            entry.0 += 1; // Count
            entry.1 += record.amount; // Total amount
        }
        
        stats
    }
    
    /// Get total fees collected
    pub fn get_total_fees_collected(&self) -> Decimal {
        self.fee_history.iter().map(|r| r.amount).sum()
    }
    
    /// Get fee configuration
    pub fn get_fee_config(&self, fee_type: &FeeType) -> Option<&FeeConfig> {
        self.fee_configs.get(fee_type)
    }
    
    /// Update fee configuration
    pub fn update_fee_config(&mut self, fee_type: FeeType, config: FeeConfig) {
        info!("Updating fee configuration for {:?}: {:?}", fee_type, config);
        self.fee_configs.insert(fee_type, config);
    }
    
    /// Enable/disable fee collection for a type
    pub fn set_fee_enabled(&mut self, fee_type: FeeType, enabled: bool) {
        if let Some(config) = self.fee_configs.get_mut(&fee_type) {
            config.enabled = enabled;
            info!("Fee collection for {:?} is now {}", fee_type, if enabled { "enabled" } else { "disabled" });
        }
    }
    
    /// Get fee history for a specific payer
    pub fn get_fee_history(&self, payer: &Address, limit: usize) -> Vec<&FeeRecord> {
        self.fee_history.iter()
            .filter(|record| record.payer == *payer)
            .rev()
            .take(limit)
            .collect()
    }
    
    /// Calculate fee discount based on staking
    pub fn calculate_staking_discount(&self, staked_amount: Decimal) -> Decimal {
        // Discount tiers based on staking amount
        if staked_amount >= Decimal::new(100000, 0) {
            Decimal::new(5, 1) // 50% discount
        } else if staked_amount >= Decimal::new(50000, 0) {
            Decimal::new(3, 1) // 30% discount
        } else if staked_amount >= Decimal::new(10000, 0) {
            Decimal::new(15, 2) // 15% discount
        } else if staked_amount >= Decimal::new(1000, 0) {
            Decimal::new(5, 2) // 5% discount
        } else {
            Decimal::ZERO // No discount
        }
    }
    
    /// Apply staking discount to fee
    pub fn apply_staking_discount(&self, base_fee: Decimal, staked_amount: Decimal) -> Decimal {
        let discount = self.calculate_staking_discount(staked_amount);
        base_fee * (Decimal::new(1, 0) - discount)
    }
    
    /// Estimate gas fee for contract execution
    pub fn estimate_gas_fee(&self, gas_estimate: u64) -> Result<Decimal> {
        let gas_config = self.fee_configs.get(&FeeType::ContractExecution)
            .ok_or_else(|| EconomyError::FeeError("Gas fee configuration not found".to_string()))?;
        
        if let FeeModel::Gas { gas_price, .. } = &gas_config.model {
            let total_gas_cost = Decimal::new(gas_estimate, 0) * gas_price;
            Ok(total_gas_cost / Decimal::new(10_u64.pow(18), 0)) // Convert to rUv
        } else {
            Err(EconomyError::FeeError("Invalid gas fee model".to_string()))
        }
    }
    
    /// Get current network metrics
    pub fn get_network_metrics(&self) -> &NetworkMetrics {
        &self.network_metrics
    }
}

/// Fee calculator trait for external integration
pub trait FeeCalculator {
    fn calculate_fee(&self, fee_type: FeeType, amount: Decimal) -> Result<Decimal>;
    fn get_fee_collector(&self, fee_type: FeeType) -> Option<Address>;
}

impl FeeCalculator for FeeManager {
    fn calculate_fee(&self, fee_type: FeeType, amount: Decimal) -> Result<Decimal> {
        self.calculate_fee(fee_type, amount, None)
    }
    
    fn get_fee_collector(&self, fee_type: FeeType) -> Option<Address> {
        self.fee_configs.get(&fee_type).map(|config| config.collector.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fee_manager_creation() {
        let config = EconomyConfig::default();
        let fee_manager = FeeManager::new(config);
        
        assert!(fee_manager.fee_configs.contains_key(&FeeType::Transfer));
        assert!(fee_manager.fee_configs.contains_key(&FeeType::Trading));
    }
    
    #[test]
    fn test_transfer_fee_calculation() {
        let config = EconomyConfig::default();
        let fee_manager = FeeManager::new(config);
        
        let amount = Decimal::new(1000, 0); // 1000 rUv
        let fee = fee_manager.calculate_transfer_fee(amount).unwrap();
        
        // Should be 1000 * 0.001 = 1 rUv (default 0.1% fee)
        assert!(fee > Decimal::ZERO);
        assert!(fee < amount);
    }
    
    #[test]
    fn test_tiered_trading_fee() {
        let config = EconomyConfig::default();
        let fee_manager = FeeManager::new(config);
        
        // Small trade
        let small_amount = Decimal::new(500, 0);
        let small_fee = fee_manager.calculate_trading_fee(small_amount).unwrap();
        
        // Large trade
        let large_amount = Decimal::new(50000, 0);
        let large_fee = fee_manager.calculate_trading_fee(large_amount).unwrap();
        
        // Large trade should have lower percentage fee
        let small_percentage = small_fee / small_amount;
        let large_percentage = large_fee / large_amount;
        
        assert!(large_percentage < small_percentage);
    }
    
    #[test]
    fn test_staking_discount() {
        let config = EconomyConfig::default();
        let fee_manager = FeeManager::new(config);
        
        let base_fee = Decimal::new(10, 0);
        let high_stake = Decimal::new(100000, 0);
        let low_stake = Decimal::new(100, 0);
        
        let discounted_fee = fee_manager.apply_staking_discount(base_fee, high_stake);
        let regular_fee = fee_manager.apply_staking_discount(base_fee, low_stake);
        
        assert!(discounted_fee < regular_fee);
        assert!(discounted_fee <= base_fee / Decimal::new(2, 0)); // At least 50% discount
    }
    
    #[test]
    fn test_domain_registration_fee() {
        let config = EconomyConfig::default();
        let fee_manager = FeeManager::new(config);
        
        let short_domain_fee = fee_manager.calculate_domain_fee(3).unwrap();
        let long_domain_fee = fee_manager.calculate_domain_fee(15).unwrap();
        
        assert!(short_domain_fee > long_domain_fee);
    }
}