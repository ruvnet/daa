//! Reward system for DAA Economy

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

use crate::{Result, EconomyError, RewardConfig};

/// Types of rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardType {
    TaskCompletion,
    HighQualityWork,
    Staking,
    Referral,
    Bug(String), // Bug report with severity
}

/// Reward system
pub struct RewardSystem {
    config: RewardConfig,
    total_rewards_distributed: Decimal,
}

impl RewardSystem {
    /// Create new reward system
    pub fn new(config: RewardConfig) -> Self {
        Self {
            config,
            total_rewards_distributed: Decimal::ZERO,
        }
    }

    /// Calculate reward amount
    pub fn calculate_reward(
        &self,
        reward_type: RewardType,
        performance_score: Option<Decimal>,
    ) -> Result<Decimal> {
        let base_amount = match reward_type {
            RewardType::TaskCompletion => self.config.base_task_reward,
            RewardType::HighQualityWork => {
                let multiplier = performance_score.unwrap_or(Decimal::ONE);
                self.config.base_task_reward * self.config.quality_multiplier * multiplier
            }
            RewardType::Staking => self.config.staking_rewards,
            RewardType::Referral => self.config.base_task_reward / Decimal::from(2),
            RewardType::Bug(_severity) => self.config.base_task_reward * Decimal::from(3),
        };

        Ok(base_amount.max(Decimal::ZERO))
    }

    /// Get total rewards distributed
    pub async fn get_total_rewards_distributed(&self) -> Result<Decimal> {
        Ok(self.total_rewards_distributed)
    }
}