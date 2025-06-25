//! Economic incentive system with staking and slashing mechanisms

use super::{SecureIdentity, SecurityError};
use qudag_crypto::fingerprint::Fingerprint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Staking pool for DAA participants
pub struct StakingPool {
    /// Total staked amount
    total_staked: Arc<Mutex<u64>>,
    
    /// Individual stakes
    stakes: Arc<Mutex<HashMap<Fingerprint, StakeInfo>>>,
    
    /// Slashing history
    slashing_history: Arc<Mutex<Vec<SlashingEvent>>>,
    
    /// Reward distribution history
    reward_history: Arc<Mutex<Vec<RewardDistribution>>>,
    
    /// Pool parameters
    params: StakingParameters,
}

/// Staking parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingParameters {
    /// Minimum stake amount
    pub min_stake: u64,
    
    /// Maximum stake amount
    pub max_stake: u64,
    
    /// Lock-in period (in rounds)
    pub lock_period: u64,
    
    /// Base reward rate per round
    pub base_reward_rate: f64,
    
    /// Slashing rates for different violations
    pub slashing_rates: SlashingRates,
    
    /// Compounding frequency
    pub compound_frequency: u64,
}

/// Different slashing rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingRates {
    /// Offline/unresponsive
    pub offline: f64,
    
    /// Invalid computation
    pub invalid_computation: f64,
    
    /// Malicious behavior
    pub malicious: f64,
    
    /// Challenge failure
    pub challenge_failure: f64,
}

impl Default for StakingParameters {
    fn default() -> Self {
        Self {
            min_stake: 1000,
            max_stake: 1_000_000,
            lock_period: 100,
            base_reward_rate: 0.001, // 0.1% per round
            slashing_rates: SlashingRates {
                offline: 0.01,
                invalid_computation: 0.05,
                malicious: 0.5,
                challenge_failure: 0.02,
            },
            compound_frequency: 10,
        }
    }
}

/// Information about a stake
#[derive(Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    /// Amount staked
    pub amount: u64,
    
    /// When the stake was made
    pub staked_at_round: u64,
    
    /// When the stake can be withdrawn
    pub unlock_round: u64,
    
    /// Accumulated rewards
    pub rewards: u64,
    
    /// Reputation score
    pub reputation: f64,
    
    /// Participation metrics
    pub participation: ParticipationMetrics,
}

/// Participation metrics for reward calculation
#[derive(Clone, Serialize, Deserialize)]
pub struct ParticipationMetrics {
    /// Rounds participated
    pub rounds_participated: u64,
    
    /// Successful aggregations
    pub successful_aggregations: u64,
    
    /// Failed aggregations
    pub failed_aggregations: u64,
    
    /// Challenges passed
    pub challenges_passed: u64,
    
    /// Challenges failed
    pub challenges_failed: u64,
    
    /// Uptime percentage
    pub uptime: f64,
}

impl Default for ParticipationMetrics {
    fn default() -> Self {
        Self {
            rounds_participated: 0,
            successful_aggregations: 0,
            failed_aggregations: 0,
            challenges_passed: 0,
            challenges_failed: 0,
            uptime: 100.0,
        }
    }
}

/// Slashing event record
#[derive(Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    /// Slashed participant
    pub participant: Fingerprint,
    
    /// Amount slashed
    pub amount: u64,
    
    /// Reason for slashing
    pub reason: SlashingReason,
    
    /// Round when slashing occurred
    pub round: u64,
    
    /// Evidence hash
    pub evidence: Vec<u8>,
}

/// Reasons for slashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingReason {
    Offline,
    InvalidComputation,
    MaliciousBehavior,
    ChallengeFailed,
    ProtocolViolation(String),
}

/// Reward distribution event
#[derive(Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    /// Round number
    pub round: u64,
    
    /// Total rewards distributed
    pub total_amount: u64,
    
    /// Individual rewards
    pub rewards: HashMap<Fingerprint, u64>,
    
    /// Distribution timestamp
    pub timestamp: u64,
}

impl StakingPool {
    /// Create a new staking pool
    pub fn new(params: StakingParameters) -> Self {
        Self {
            total_staked: Arc::new(Mutex::new(0)),
            stakes: Arc::new(Mutex::new(HashMap::new())),
            slashing_history: Arc::new(Mutex::new(Vec::new())),
            reward_history: Arc::new(Mutex::new(Vec::new())),
            params,
        }
    }
    
    /// Stake tokens
    pub fn stake(
        &self,
        participant: &Fingerprint,
        amount: u64,
        current_round: u64,
    ) -> Result<(), SecurityError> {
        if amount < self.params.min_stake {
            return Err(SecurityError::InsufficientStake {
                required: self.params.min_stake,
                actual: amount,
            });
        }
        
        if amount > self.params.max_stake {
            return Err(SecurityError::VerificationError(
                format!("Stake exceeds maximum: {} > {}", amount, self.params.max_stake),
            ));
        }
        
        let mut stakes = self.stakes.lock().unwrap();
        let mut total = self.total_staked.lock().unwrap();
        
        let stake_info = StakeInfo {
            amount,
            staked_at_round: current_round,
            unlock_round: current_round + self.params.lock_period,
            rewards: 0,
            reputation: 1.0,
            participation: ParticipationMetrics::default(),
        };
        
        if let Some(existing) = stakes.get_mut(participant) {
            // Add to existing stake
            existing.amount += amount;
            existing.unlock_round = current_round + self.params.lock_period;
        } else {
            stakes.insert(participant.clone(), stake_info);
        }
        
        *total += amount;
        
        Ok(())
    }
    
    /// Withdraw stake (if unlocked)
    pub fn withdraw(
        &self,
        participant: &Fingerprint,
        amount: u64,
        current_round: u64,
    ) -> Result<u64, SecurityError> {
        let mut stakes = self.stakes.lock().unwrap();
        let mut total = self.total_staked.lock().unwrap();
        
        let stake_info = stakes.get_mut(participant)
            .ok_or_else(|| SecurityError::VerificationError(
                "No stake found".to_string()
            ))?;
        
        if current_round < stake_info.unlock_round {
            return Err(SecurityError::VerificationError(
                format!("Stake locked until round {}", stake_info.unlock_round),
            ));
        }
        
        if amount > stake_info.amount {
            return Err(SecurityError::InsufficientStake {
                required: amount,
                actual: stake_info.amount,
            });
        }
        
        stake_info.amount -= amount;
        *total -= amount;
        
        // Remove if fully withdrawn
        if stake_info.amount == 0 {
            stakes.remove(participant);
        }
        
        Ok(amount)
    }
    
    /// Slash a participant
    pub fn slash(
        &self,
        participant: &Fingerprint,
        reason: SlashingReason,
        evidence: Vec<u8>,
        current_round: u64,
    ) -> Result<u64, SecurityError> {
        let mut stakes = self.stakes.lock().unwrap();
        let mut total = self.total_staked.lock().unwrap();
        let mut history = self.slashing_history.lock().unwrap();
        
        let stake_info = stakes.get_mut(participant)
            .ok_or_else(|| SecurityError::VerificationError(
                "No stake found".to_string()
            ))?;
        
        // Calculate slash amount based on reason
        let slash_rate = match &reason {
            SlashingReason::Offline => self.params.slashing_rates.offline,
            SlashingReason::InvalidComputation => self.params.slashing_rates.invalid_computation,
            SlashingReason::MaliciousBehavior => self.params.slashing_rates.malicious,
            SlashingReason::ChallengeFailed => self.params.slashing_rates.challenge_failure,
            SlashingReason::ProtocolViolation(_) => self.params.slashing_rates.malicious,
        };
        
        let slash_amount = (stake_info.amount as f64 * slash_rate) as u64;
        let actual_slashed = slash_amount.min(stake_info.amount);
        
        // Apply slashing
        stake_info.amount -= actual_slashed;
        stake_info.reputation *= 0.9; // Reduce reputation
        *total -= actual_slashed;
        
        // Record event
        history.push(SlashingEvent {
            participant: participant.clone(),
            amount: actual_slashed,
            reason,
            round: current_round,
            evidence,
        });
        
        // Remove if fully slashed
        if stake_info.amount == 0 {
            stakes.remove(participant);
        }
        
        Ok(actual_slashed)
    }
    
    /// Distribute rewards to participants
    pub fn distribute_rewards(
        &self,
        total_reward: u64,
        current_round: u64,
    ) -> Result<RewardDistribution, SecurityError> {
        let mut stakes = self.stakes.lock().unwrap();
        let total_staked = *self.total_staked.lock().unwrap();
        
        if total_staked == 0 {
            return Err(SecurityError::VerificationError(
                "No stakes to distribute rewards".to_string(),
            ));
        }
        
        let mut rewards = HashMap::new();
        let mut distributed = 0u64;
        
        // Calculate rewards based on stake and participation
        for (participant, stake_info) in stakes.iter_mut() {
            let stake_weight = stake_info.amount as f64 / total_staked as f64;
            let participation_multiplier = Self::calculate_participation_multiplier(stake_info);
            let reputation_multiplier = stake_info.reputation;
            
            let reward = (total_reward as f64
                * stake_weight
                * participation_multiplier
                * reputation_multiplier) as u64;
            
            stake_info.rewards += reward;
            distributed += reward;
            rewards.insert(participant.clone(), reward);
            
            // Compound rewards if frequency met
            if current_round % self.params.compound_frequency == 0 {
                stake_info.amount += stake_info.rewards;
                stake_info.rewards = 0;
            }
        }
        
        let distribution = RewardDistribution {
            round: current_round,
            total_amount: distributed,
            rewards,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.reward_history.lock().unwrap().push(distribution.clone());
        
        Ok(distribution)
    }
    
    /// Calculate participation multiplier for rewards
    fn calculate_participation_multiplier(stake_info: &StakeInfo) -> f64 {
        let metrics = &stake_info.participation;
        
        // Base multiplier
        let mut multiplier = 1.0;
        
        // Uptime bonus
        multiplier *= metrics.uptime / 100.0;
        
        // Success rate bonus
        let total_aggregations = metrics.successful_aggregations + metrics.failed_aggregations;
        if total_aggregations > 0 {
            let success_rate = metrics.successful_aggregations as f64 / total_aggregations as f64;
            multiplier *= 0.5 + 0.5 * success_rate;
        }
        
        // Challenge performance
        let total_challenges = metrics.challenges_passed + metrics.challenges_failed;
        if total_challenges > 0 {
            let challenge_rate = metrics.challenges_passed as f64 / total_challenges as f64;
            multiplier *= 0.8 + 0.2 * challenge_rate;
        }
        
        multiplier
    }
    
    /// Update participation metrics
    pub fn update_participation(
        &self,
        participant: &Fingerprint,
        update: ParticipationUpdate,
    ) -> Result<(), SecurityError> {
        let mut stakes = self.stakes.lock().unwrap();
        
        let stake_info = stakes.get_mut(participant)
            .ok_or_else(|| SecurityError::VerificationError(
                "No stake found".to_string()
            ))?;
        
        match update {
            ParticipationUpdate::RoundParticipated => {
                stake_info.participation.rounds_participated += 1;
            }
            ParticipationUpdate::AggregationSuccess => {
                stake_info.participation.successful_aggregations += 1;
            }
            ParticipationUpdate::AggregationFailure => {
                stake_info.participation.failed_aggregations += 1;
            }
            ParticipationUpdate::ChallengePass => {
                stake_info.participation.challenges_passed += 1;
                stake_info.reputation = (stake_info.reputation * 1.01).min(2.0);
            }
            ParticipationUpdate::ChallengeFail => {
                stake_info.participation.challenges_failed += 1;
                stake_info.reputation = (stake_info.reputation * 0.95).max(0.1);
            }
            ParticipationUpdate::UptimeUpdate(uptime) => {
                stake_info.participation.uptime = uptime;
            }
        }
        
        Ok(())
    }
    
    /// Get stake information
    pub fn get_stake_info(&self, participant: &Fingerprint) -> Option<StakeInfo> {
        self.stakes.lock().unwrap().get(participant).cloned()
    }
    
    /// Get total staked amount
    pub fn get_total_staked(&self) -> u64 {
        *self.total_staked.lock().unwrap()
    }
}

/// Updates to participation metrics
#[derive(Debug, Clone)]
pub enum ParticipationUpdate {
    RoundParticipated,
    AggregationSuccess,
    AggregationFailure,
    ChallengePass,
    ChallengeFail,
    UptimeUpdate(f64),
}

/// Economic incentive calculator
pub struct IncentiveCalculator {
    /// Base reward per round
    base_reward: u64,
    
    /// Penalty for non-participation
    non_participation_penalty: u64,
    
    /// Bonus for high availability
    availability_bonus: u64,
}

impl IncentiveCalculator {
    /// Calculate rewards for a round
    pub fn calculate_round_rewards(
        &self,
        participants: &[Fingerprint],
        contributions: &HashMap<Fingerprint, f64>,
    ) -> HashMap<Fingerprint, u64> {
        let mut rewards = HashMap::new();
        
        // Calculate total contribution
        let total_contribution: f64 = contributions.values().sum();
        
        if total_contribution > 0.0 {
            for participant in participants {
                if let Some(&contribution) = contributions.get(participant) {
                    let share = contribution / total_contribution;
                    let reward = (self.base_reward as f64 * share) as u64;
                    rewards.insert(participant.clone(), reward);
                }
            }
        }
        
        rewards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking_lifecycle() {
        let pool = StakingPool::new(StakingParameters::default());
        let participant = Fingerprint::new(b"test").unwrap();
        
        // Stake tokens
        pool.stake(&participant, 5000, 1).unwrap();
        assert_eq!(pool.get_total_staked(), 5000);
        
        // Try to withdraw before unlock
        let result = pool.withdraw(&participant, 1000, 50);
        assert!(result.is_err());
        
        // Withdraw after unlock
        let withdrawn = pool.withdraw(&participant, 1000, 101).unwrap();
        assert_eq!(withdrawn, 1000);
        assert_eq!(pool.get_total_staked(), 4000);
    }
    
    #[test]
    fn test_slashing() {
        let pool = StakingPool::new(StakingParameters::default());
        let participant = Fingerprint::new(b"test").unwrap();
        
        pool.stake(&participant, 10000, 1).unwrap();
        
        // Slash for offline
        let slashed = pool.slash(
            &participant,
            SlashingReason::Offline,
            vec![],
            2,
        ).unwrap();
        
        assert_eq!(slashed, 100); // 1% of 10000
        assert_eq!(pool.get_total_staked(), 9900);
    }
    
    #[test]
    fn test_reward_distribution() {
        let pool = StakingPool::new(StakingParameters::default());
        let participant1 = Fingerprint::new(b"test1").unwrap();
        let participant2 = Fingerprint::new(b"test2").unwrap();
        
        pool.stake(&participant1, 7000, 1).unwrap();
        pool.stake(&participant2, 3000, 1).unwrap();
        
        let distribution = pool.distribute_rewards(1000, 2).unwrap();
        
        assert_eq!(distribution.total_amount, 1000);
        assert!(distribution.rewards.get(&participant1).unwrap() > distribution.rewards.get(&participant2).unwrap());
    }
}