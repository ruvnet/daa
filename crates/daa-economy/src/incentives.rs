//! Incentive Engine for DAA Economy
//!
//! This module handles all incentive mechanisms including staking rewards,
//! validator rewards, network participation incentives, and governance rewards.

use crate::{Result, EconomyError, EconomyConfig};
use daa_chain::Address;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, debug, warn};

/// Types of incentives in the DAA economy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncentiveType {
    /// Staking rewards for locked tokens
    StakingReward,
    /// Validator rewards for block production
    ValidatorReward,
    /// Network participation rewards
    NetworkParticipation,
    /// Governance voting rewards
    GovernanceVoting,
    /// Referral rewards
    ReferralReward,
    /// Liquidity provision rewards
    LiquidityProvision,
    /// Bug bounty rewards
    BugBounty,
}

/// Incentive calculation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncentiveParams {
    pub base_rate: Decimal,
    pub multiplier: Decimal,
    pub minimum_amount: Decimal,
    pub maximum_amount: Decimal,
    pub duration_factor: Decimal,
    pub performance_factor: Decimal,
}

impl Default for IncentiveParams {
    fn default() -> Self {
        IncentiveParams {
            base_rate: Decimal::new(5, 2), // 5% base rate
            multiplier: Decimal::new(1, 0), // 1x multiplier
            minimum_amount: Decimal::new(1, 18), // 0.000000000000000001 rUv
            maximum_amount: Decimal::new(1000000, 0), // 1M rUv
            duration_factor: Decimal::new(1, 0), // 1x for duration
            performance_factor: Decimal::new(1, 0), // 1x for performance
        }
    }
}

/// Incentive record for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncentiveRecord {
    pub recipient: Address,
    pub incentive_type: IncentiveType,
    pub amount: Decimal,
    pub timestamp: u64,
    pub transaction_hash: Option<String>,
    pub parameters: IncentiveParams,
    pub metadata: HashMap<String, String>,
}

/// Performance metrics for incentive calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub uptime_percentage: Decimal,
    pub validation_accuracy: Decimal,
    pub network_contribution: Decimal,
    pub governance_participation: Decimal,
    pub referral_count: u64,
    pub liquidity_provided: Decimal,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        PerformanceMetrics {
            uptime_percentage: Decimal::new(100, 2), // 100%
            validation_accuracy: Decimal::new(100, 2), // 100%
            network_contribution: Decimal::new(1, 0), // Neutral
            governance_participation: Decimal::ZERO,
            referral_count: 0,
            liquidity_provided: Decimal::ZERO,
        }
    }
}

/// Incentive engine manages all reward calculations
pub struct IncentiveEngine {
    config: EconomyConfig,
    incentive_params: HashMap<IncentiveType, IncentiveParams>,
    performance_metrics: HashMap<Address, PerformanceMetrics>,
    incentive_history: Vec<IncentiveRecord>,
}

impl IncentiveEngine {
    /// Create a new incentive engine
    pub fn new(config: EconomyConfig) -> Self {
        let mut incentive_params = HashMap::new();
        
        // Initialize default parameters for each incentive type
        incentive_params.insert(IncentiveType::StakingReward, IncentiveParams {
            base_rate: config.staking_reward_rate,
            multiplier: Decimal::new(1, 0),
            minimum_amount: Decimal::new(1, 18),
            maximum_amount: Decimal::new(100000, 0),
            duration_factor: Decimal::new(1, 0),
            performance_factor: Decimal::new(1, 0),
        });
        
        incentive_params.insert(IncentiveType::ValidatorReward, IncentiveParams {
            base_rate: config.validator_reward_rate,
            multiplier: Decimal::new(15, 1), // 1.5x for validators
            minimum_amount: Decimal::new(10, 0),
            maximum_amount: Decimal::new(10000, 0),
            duration_factor: Decimal::new(1, 0),
            performance_factor: Decimal::new(2, 0), // 2x performance weight
        });
        
        incentive_params.insert(IncentiveType::NetworkParticipation, IncentiveParams {
            base_rate: Decimal::new(2, 2), // 2%
            multiplier: Decimal::new(1, 0),
            minimum_amount: Decimal::new(1, 0),
            maximum_amount: Decimal::new(1000, 0),
            duration_factor: Decimal::new(15, 1), // 1.5x for duration
            performance_factor: Decimal::new(1, 0),
        });
        
        incentive_params.insert(IncentiveType::GovernanceVoting, IncentiveParams {
            base_rate: Decimal::new(1, 2), // 1%
            multiplier: Decimal::new(1, 0),
            minimum_amount: Decimal::new(5, 1), // 0.5 rUv
            maximum_amount: Decimal::new(500, 0),
            duration_factor: Decimal::new(1, 0),
            performance_factor: Decimal::new(1, 0),
        });
        
        incentive_params.insert(IncentiveType::ReferralReward, IncentiveParams {
            base_rate: Decimal::new(10, 0), // 10 rUv per referral
            multiplier: Decimal::new(1, 0),
            minimum_amount: Decimal::new(1, 0),
            maximum_amount: Decimal::new(1000, 0),
            duration_factor: Decimal::new(1, 0),
            performance_factor: Decimal::new(1, 0),
        });
        
        incentive_params.insert(IncentiveType::LiquidityProvision, IncentiveParams {
            base_rate: Decimal::new(12, 2), // 12% APY
            multiplier: Decimal::new(1, 0),
            minimum_amount: Decimal::new(1, 0),
            maximum_amount: Decimal::new(50000, 0),
            duration_factor: Decimal::new(2, 0), // 2x for duration
            performance_factor: Decimal::new(1, 0),
        });
        
        incentive_params.insert(IncentiveType::BugBounty, IncentiveParams {
            base_rate: Decimal::new(100, 0), // 100 rUv base
            multiplier: Decimal::new(10, 0), // 10x for severity
            minimum_amount: Decimal::new(10, 0),
            maximum_amount: Decimal::new(10000, 0),
            duration_factor: Decimal::new(1, 0),
            performance_factor: Decimal::new(1, 0),
        });
        
        IncentiveEngine {
            config,
            incentive_params,
            performance_metrics: HashMap::new(),
            incentive_history: Vec::new(),
        }
    }
    
    /// Calculate staking reward for an address
    pub fn calculate_staking_reward(&self, stake_amount: Decimal, voting_power: Decimal) -> Result<Decimal> {
        let params = self.incentive_params.get(&IncentiveType::StakingReward)
            .ok_or_else(|| EconomyError::IncentiveError("Staking reward parameters not found".to_string()))?;
        
        // Base reward calculation: stake_amount * base_rate / 365 (daily reward)
        let base_reward = stake_amount * params.base_rate / Decimal::new(365, 0);
        
        // Apply voting power multiplier
        let voting_multiplier = Decimal::new(1, 0) + (voting_power / stake_amount - Decimal::new(1, 0)) * Decimal::new(1, 1); // 0.1x bonus per voting power ratio
        
        let reward = base_reward * voting_multiplier * params.multiplier;
        
        // Apply bounds
        Ok(reward.clamp(params.minimum_amount, params.maximum_amount))
    }
    
    /// Calculate validator reward based on performance
    pub fn calculate_validator_reward(&self, address: &Address, base_stake: Decimal) -> Result<Decimal> {
        let params = self.incentive_params.get(&IncentiveType::ValidatorReward)
            .ok_or_else(|| EconomyError::IncentiveError("Validator reward parameters not found".to_string()))?;
        
        let metrics = self.performance_metrics.get(address).unwrap_or(&PerformanceMetrics::default());
        
        // Base reward
        let base_reward = base_stake * params.base_rate / Decimal::new(365, 0);
        
        // Performance multiplier based on metrics
        let performance_multiplier = (
            metrics.uptime_percentage / Decimal::new(100, 0) +
            metrics.validation_accuracy / Decimal::new(100, 0) +
            metrics.network_contribution
        ) / Decimal::new(3, 0); // Average of three metrics
        
        let reward = base_reward * performance_multiplier * params.multiplier * params.performance_factor;
        
        debug!("Validator reward for {}: base={}, performance={}, final={}", 
               address, base_reward, performance_multiplier, reward);
        
        Ok(reward.clamp(params.minimum_amount, params.maximum_amount))
    }
    
    /// Calculate network participation reward
    pub fn calculate_network_participation_reward(
        &self,
        address: &Address,
        participation_score: Decimal,
        duration_days: u64,
    ) -> Result<Decimal> {
        let params = self.incentive_params.get(&IncentiveType::NetworkParticipation)
            .ok_or_else(|| EconomyError::IncentiveError("Network participation parameters not found".to_string()))?;
        
        // Base reward scaled by participation score
        let base_reward = participation_score * params.base_rate;
        
        // Duration bonus
        let duration_bonus = if duration_days > 30 {
            Decimal::new(1, 0) + Decimal::new(duration_days.min(365), 0) / Decimal::new(365, 0) * params.duration_factor
        } else {
            Decimal::new(1, 0)
        };
        
        let reward = base_reward * duration_bonus * params.multiplier;
        
        debug!("Network participation reward for {}: score={}, duration_days={}, reward={}", 
               address, participation_score, duration_days, reward);
        
        Ok(reward.clamp(params.minimum_amount, params.maximum_amount))
    }
    
    /// Calculate governance voting reward
    pub fn calculate_governance_reward(&self, address: &Address, votes_cast: u64, voting_power: Decimal) -> Result<Decimal> {
        let params = self.incentive_params.get(&IncentiveType::GovernanceVoting)
            .ok_or_else(|| EconomyError::IncentiveError("Governance reward parameters not found".to_string()))?;
        
        // Reward based on number of votes and voting power
        let vote_bonus = Decimal::new(votes_cast, 0) * params.base_rate;
        let power_bonus = voting_power * params.base_rate / Decimal::new(1000, 0); // Scale down voting power impact
        
        let reward = (vote_bonus + power_bonus) * params.multiplier;
        
        debug!("Governance reward for {}: votes={}, voting_power={}, reward={}", 
               address, votes_cast, voting_power, reward);
        
        Ok(reward.clamp(params.minimum_amount, params.maximum_amount))
    }
    
    /// Calculate referral reward
    pub fn calculate_referral_reward(&self, referrer: &Address, referred_count: u64) -> Result<Decimal> {
        let params = self.incentive_params.get(&IncentiveType::ReferralReward)
            .ok_or_else(|| EconomyError::IncentiveError("Referral reward parameters not found".to_string()))?;
        
        let reward = Decimal::new(referred_count, 0) * params.base_rate * params.multiplier;
        
        debug!("Referral reward for {}: referrals={}, reward={}", 
               referrer, referred_count, reward);
        
        Ok(reward.clamp(params.minimum_amount, params.maximum_amount))
    }
    
    /// Calculate liquidity provision reward
    pub fn calculate_liquidity_reward(
        &self,
        provider: &Address,
        liquidity_amount: Decimal,
        duration_days: u64,
    ) -> Result<Decimal> {
        let params = self.incentive_params.get(&IncentiveType::LiquidityProvision)
            .ok_or_else(|| EconomyError::IncentiveError("Liquidity provision parameters not found".to_string()))?;
        
        // APY-based calculation
        let annual_reward = liquidity_amount * params.base_rate;
        let daily_reward = annual_reward / Decimal::new(365, 0);
        
        // Duration bonus for longer provision
        let duration_bonus = if duration_days > 7 {
            Decimal::new(1, 0) + Decimal::new(duration_days.min(365), 0) / Decimal::new(365, 0) * params.duration_factor
        } else {
            Decimal::new(1, 0)
        };
        
        let reward = daily_reward * Decimal::new(duration_days, 0) * duration_bonus * params.multiplier;
        
        debug!("Liquidity reward for {}: amount={}, duration_days={}, reward={}", 
               provider, liquidity_amount, duration_days, reward);
        
        Ok(reward.clamp(params.minimum_amount, params.maximum_amount))
    }
    
    /// Calculate bug bounty reward based on severity
    pub fn calculate_bug_bounty_reward(&self, reporter: &Address, severity: BugSeverity) -> Result<Decimal> {
        let params = self.incentive_params.get(&IncentiveType::BugBounty)
            .ok_or_else(|| EconomyError::IncentiveError("Bug bounty parameters not found".to_string()))?;
        
        let severity_multiplier = match severity {
            BugSeverity::Low => Decimal::new(1, 0),
            BugSeverity::Medium => Decimal::new(3, 0),
            BugSeverity::High => Decimal::new(7, 0),
            BugSeverity::Critical => Decimal::new(15, 0),
        };
        
        let reward = params.base_rate * severity_multiplier * params.multiplier;
        
        info!("Bug bounty reward for {}: severity={:?}, reward={}", 
              reporter, severity, reward);
        
        Ok(reward.clamp(params.minimum_amount, params.maximum_amount))
    }
    
    /// Update performance metrics for an address
    pub fn update_performance_metrics(&mut self, address: &Address, metrics: PerformanceMetrics) {
        debug!("Updating performance metrics for {}: {:?}", address, metrics);
        self.performance_metrics.insert(address.clone(), metrics);
    }
    
    /// Get performance metrics for an address
    pub fn get_performance_metrics(&self, address: &Address) -> Option<&PerformanceMetrics> {
        self.performance_metrics.get(address)
    }
    
    /// Record an incentive distribution
    pub fn record_incentive(&mut self, record: IncentiveRecord) {
        info!("Recording incentive: {:?} -> {} rUv to {}", 
              record.incentive_type, record.amount, record.recipient);
        self.incentive_history.push(record);
    }
    
    /// Get incentive history for an address
    pub fn get_incentive_history(&self, address: &Address) -> Vec<&IncentiveRecord> {
        self.incentive_history.iter()
            .filter(|record| record.recipient == *address)
            .collect()
    }
    
    /// Get total incentives distributed
    pub fn get_total_incentives_distributed(&self) -> Decimal {
        self.incentive_history.iter()
            .map(|record| record.amount)
            .sum()
    }
    
    /// Get incentive statistics by type
    pub fn get_incentive_stats_by_type(&self) -> HashMap<IncentiveType, (u64, Decimal)> {
        let mut stats = HashMap::new();
        
        for record in &self.incentive_history {
            let entry = stats.entry(record.incentive_type.clone()).or_insert((0u64, Decimal::ZERO));
            entry.0 += 1; // Count
            entry.1 += record.amount; // Total amount
        }
        
        stats
    }
    
    /// Update incentive parameters
    pub fn update_incentive_params(&mut self, incentive_type: IncentiveType, params: IncentiveParams) {
        info!("Updating incentive parameters for {:?}: {:?}", incentive_type, params);
        self.incentive_params.insert(incentive_type, params);
    }
    
    /// Get current incentive parameters
    pub fn get_incentive_params(&self, incentive_type: &IncentiveType) -> Option<&IncentiveParams> {
        self.incentive_params.get(incentive_type)
    }
}

/// Bug severity levels for bounty calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BugSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking_reward_calculation() {
        let config = EconomyConfig::default();
        let engine = IncentiveEngine::new(config);
        
        let stake_amount = Decimal::new(1000, 0); // 1000 rUv
        let voting_power = Decimal::new(1000, 0); // Same as stake
        
        let reward = engine.calculate_staking_reward(stake_amount, voting_power).unwrap();
        
        // Expected: 1000 * 0.08 / 365 = ~0.219 rUv per day
        assert!(reward > Decimal::ZERO);
        assert!(reward < Decimal::new(1, 0)); // Should be less than 1 rUv per day
    }
    
    #[test]
    fn test_validator_reward_calculation() {
        let config = EconomyConfig::default();
        let mut engine = IncentiveEngine::new(config);
        
        let address = Address::from_hex("0x1234567890123456789012345678901234567890").unwrap();
        let stake = Decimal::new(10000, 0); // 10k rUv
        
        // Set high performance metrics
        let metrics = PerformanceMetrics {
            uptime_percentage: Decimal::new(98, 0), // 98%
            validation_accuracy: Decimal::new(995, 1), // 99.5%
            network_contribution: Decimal::new(12, 1), // 1.2
            ..Default::default()
        };
        
        engine.update_performance_metrics(&address, metrics);
        let reward = engine.calculate_validator_reward(&address, stake).unwrap();
        
        assert!(reward > Decimal::ZERO);
        // Should be higher than base staking reward due to validator multiplier
    }
    
    #[test]
    fn test_bug_bounty_rewards() {
        let config = EconomyConfig::default();
        let engine = IncentiveEngine::new(config);
        
        let reporter = Address::from_hex("0x1234567890123456789012345678901234567890").unwrap();
        
        let low_reward = engine.calculate_bug_bounty_reward(&reporter, BugSeverity::Low).unwrap();
        let critical_reward = engine.calculate_bug_bounty_reward(&reporter, BugSeverity::Critical).unwrap();
        
        assert!(critical_reward > low_reward);
        assert!(critical_reward >= Decimal::new(1500, 0)); // Should be >= 1500 rUv for critical
    }
}