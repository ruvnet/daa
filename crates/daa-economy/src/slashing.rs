//! Slashing Mechanisms for Malicious Nodes
//!
//! This module implements comprehensive slashing mechanisms to penalize malicious behavior
//! in the network, including evidence tracking, appeal processes, and reputation management.

use crate::{Result, EconomyError, EconomyConfig, RuvTokenManager, IncentiveEngine};
use daa_chain::{Address, TxHash, BlockchainAdapter};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use log::{info, debug, warn, error};

/// Types of slashable offenses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SlashingOffense {
    /// Double signing blocks (validator misconduct)
    DoubleSigning { 
        block_height: u64,
        signature1: Vec<u8>,
        signature2: Vec<u8>,
    },
    
    /// Validator downtime/unavailability
    Downtime {
        start_block: u64,
        end_block: u64,
        missed_blocks: u64,
    },
    
    /// Invalid block proposal
    InvalidBlock {
        block_height: u64,
        reason: String,
        evidence_hash: String,
    },
    
    /// Censorship of valid transactions
    Censorship {
        censored_tx_hashes: Vec<String>,
        block_range: (u64, u64),
    },
    
    /// Resource provider misconduct
    ResourceMisconduct {
        reservation_id: String,
        violation_type: ResourceViolationType,
    },
    
    /// Network attack participation
    NetworkAttack {
        attack_type: AttackType,
        severity: SeverityLevel,
        evidence: Vec<u8>,
    },
    
    /// Data integrity violation
    DataIntegrity {
        data_hash: String,
        expected_hash: String,
        affected_nodes: Vec<Address>,
    },
    
    /// Governance manipulation
    GovernanceManipulation {
        proposal_id: String,
        manipulation_type: String,
        evidence: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceViolationType {
    /// Failed to provide promised resources
    ResourceUnavailable,
    /// Provided substandard resources
    ResourceQualityViolation,
    /// Overcharged for resources
    PriceManipulation,
    /// Falsified performance metrics
    MetricsFalsification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AttackType {
    DDoS,
    Sybil,
    Eclipse,
    RoutingAttack,
    ConsensusAttack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Evidence for a slashing offense
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvidence {
    pub offense: SlashingOffense,
    pub reporter: Address,
    pub timestamp: u64,
    pub cryptographic_proof: Option<Vec<u8>>,
    pub witness_signatures: Vec<(Address, Vec<u8>)>,
    pub additional_data: HashMap<String, String>,
}

/// Slashing decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingDecision {
    pub id: String,
    pub offender: Address,
    pub offense: SlashingOffense,
    pub evidence: SlashingEvidence,
    pub penalty_amount: Decimal,
    pub reputation_penalty: i32,
    pub ban_duration: Option<u64>, // in seconds
    pub status: SlashingStatus,
    pub decided_at: u64,
    pub executed_at: Option<u64>,
    pub appeal: Option<Appeal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlashingStatus {
    Pending,
    UnderReview,
    Approved,
    Executed,
    Appealed,
    Reversed,
    Expired,
}

/// Appeal against slashing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appeal {
    pub id: String,
    pub appellant: Address,
    pub reason: String,
    pub evidence: Vec<u8>,
    pub submitted_at: u64,
    pub status: AppealStatus,
    pub decision: Option<AppealDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppealStatus {
    Pending,
    UnderReview,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppealDecision {
    pub decided_by: Vec<Address>, // Committee members
    pub decision: AppealStatus,
    pub reason: String,
    pub decided_at: u64,
}

/// Node reputation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReputation {
    pub address: Address,
    pub reputation_score: i32, // Can go negative
    pub total_offenses: u64,
    pub offense_history: VecDeque<(SlashingOffense, u64)>, // offense, timestamp
    pub last_offense_time: Option<u64>,
    pub is_banned: bool,
    pub ban_expiry: Option<u64>,
}

/// Slashing parameters configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingParams {
    pub double_signing_penalty_rate: Decimal,
    pub downtime_penalty_rate: Decimal,
    pub invalid_block_penalty_rate: Decimal,
    pub censorship_penalty_rate: Decimal,
    pub resource_misconduct_penalty_rate: Decimal,
    pub network_attack_penalty_multiplier: Decimal,
    pub min_witnesses_required: u32,
    pub evidence_expiry_time: u64, // in seconds
    pub appeal_window: u64, // in seconds
    pub reputation_recovery_rate: i32, // points per day
    pub ban_threshold_score: i32,
}

impl Default for SlashingParams {
    fn default() -> Self {
        SlashingParams {
            double_signing_penalty_rate: Decimal::new(20, 2), // 20%
            downtime_penalty_rate: Decimal::new(5, 2), // 5%
            invalid_block_penalty_rate: Decimal::new(15, 2), // 15%
            censorship_penalty_rate: Decimal::new(10, 2), // 10%
            resource_misconduct_penalty_rate: Decimal::new(10, 2), // 10%
            network_attack_penalty_multiplier: Decimal::new(2, 0), // 2x
            min_witnesses_required: 3,
            evidence_expiry_time: 7 * 24 * 3600, // 7 days
            appeal_window: 48 * 3600, // 48 hours
            reputation_recovery_rate: 10, // 10 points per day
            ban_threshold_score: -1000,
        }
    }
}

/// Slashing manager handles all slashing operations
pub struct SlashingManager {
    config: EconomyConfig,
    params: SlashingParams,
    ruv_token: Arc<RuvTokenManager>,
    incentive_engine: Arc<IncentiveEngine>,
    blockchain_adapter: Arc<dyn BlockchainAdapter>,
    
    // Slashing state
    evidence_pool: Arc<RwLock<HashMap<String, SlashingEvidence>>>,
    decisions: Arc<RwLock<HashMap<String, SlashingDecision>>>,
    reputations: Arc<RwLock<HashMap<Address, NodeReputation>>>,
    offense_counters: Arc<RwLock<HashMap<(Address, SlashingOffense), u64>>>,
    
    // Committee for appeals
    slashing_committee: Arc<RwLock<Vec<Address>>>,
}

impl SlashingManager {
    /// Create a new slashing manager
    pub async fn new(
        config: EconomyConfig,
        params: SlashingParams,
        ruv_token: Arc<RuvTokenManager>,
        incentive_engine: Arc<IncentiveEngine>,
        blockchain_adapter: Arc<dyn BlockchainAdapter>,
    ) -> Result<Self> {
        info!("Initializing Slashing Manager");
        
        Ok(SlashingManager {
            config,
            params,
            ruv_token,
            incentive_engine,
            blockchain_adapter,
            evidence_pool: Arc::new(RwLock::new(HashMap::new())),
            decisions: Arc::new(RwLock::new(HashMap::new())),
            reputations: Arc::new(RwLock::new(HashMap::new())),
            offense_counters: Arc::new(RwLock::new(HashMap::new())),
            slashing_committee: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Report a slashing offense with evidence
    pub async fn report_offense(
        &self,
        reporter: &Address,
        offense: SlashingOffense,
        cryptographic_proof: Option<Vec<u8>>,
        additional_data: HashMap<String, String>,
    ) -> Result<String> {
        info!("Reporting offense by {}: {:?}", reporter, offense);
        
        // Validate reporter is not the offender (in applicable cases)
        if let Some(offender) = self.get_offender_from_offense(&offense) {
            if offender == *reporter {
                return Err(EconomyError::ConfigurationError(
                    "Cannot self-report offense".to_string()
                ));
            }
        }
        
        // Create evidence
        let evidence_id = format!("evidence_{}", uuid::Uuid::new_v4());
        let evidence = SlashingEvidence {
            offense: offense.clone(),
            reporter: reporter.clone(),
            timestamp: self.get_current_timestamp(),
            cryptographic_proof,
            witness_signatures: Vec::new(),
            additional_data,
        };
        
        // Store evidence
        {
            let mut pool = self.evidence_pool.write().unwrap();
            pool.insert(evidence_id.clone(), evidence);
        }
        
        // Check if we have enough evidence to create a slashing decision
        self.check_evidence_threshold(&evidence_id).await?;
        
        info!("Offense reported with evidence ID: {}", evidence_id);
        Ok(evidence_id)
    }
    
    /// Add witness signature to evidence
    pub async fn add_witness(
        &self,
        evidence_id: &str,
        witness: &Address,
        signature: Vec<u8>,
    ) -> Result<()> {
        info!("Adding witness {} to evidence {}", witness, evidence_id);
        
        let mut pool = self.evidence_pool.write().unwrap();
        let evidence = pool.get_mut(evidence_id)
            .ok_or_else(|| EconomyError::ConfigurationError("Evidence not found".to_string()))?;
        
        // Check witness is not already present
        if evidence.witness_signatures.iter().any(|(addr, _)| addr == witness) {
            return Err(EconomyError::ConfigurationError(
                "Witness already signed".to_string()
            ));
        }
        
        // Add witness
        evidence.witness_signatures.push((witness.clone(), signature));
        
        drop(pool);
        
        // Check if we now have enough witnesses
        self.check_evidence_threshold(evidence_id).await?;
        
        Ok(())
    }
    
    /// Check if evidence has enough witnesses to proceed
    async fn check_evidence_threshold(&self, evidence_id: &str) -> Result<()> {
        let evidence = {
            let pool = self.evidence_pool.read().unwrap();
            pool.get(evidence_id).cloned()
        };
        
        if let Some(evidence) = evidence {
            if evidence.witness_signatures.len() >= self.params.min_witnesses_required as usize {
                // Create slashing decision
                self.create_slashing_decision(evidence).await?;
                
                // Remove from pool
                let mut pool = self.evidence_pool.write().unwrap();
                pool.remove(evidence_id);
            }
        }
        
        Ok(())
    }
    
    /// Create a slashing decision based on evidence
    async fn create_slashing_decision(&self, evidence: SlashingEvidence) -> Result<String> {
        let offender = self.get_offender_from_offense(&evidence.offense)
            .ok_or_else(|| EconomyError::ConfigurationError("Cannot determine offender".to_string()))?;
        
        info!("Creating slashing decision for {} based on offense: {:?}", offender, evidence.offense);
        
        // Calculate penalty
        let (penalty_amount, reputation_penalty, ban_duration) = 
            self.calculate_penalties(&offender, &evidence.offense).await?;
        
        // Create decision
        let decision_id = format!("slash_{}", uuid::Uuid::new_v4());
        let decision = SlashingDecision {
            id: decision_id.clone(),
            offender: offender.clone(),
            offense: evidence.offense.clone(),
            evidence,
            penalty_amount,
            reputation_penalty,
            ban_duration,
            status: SlashingStatus::Pending,
            decided_at: self.get_current_timestamp(),
            executed_at: None,
            appeal: None,
        };
        
        // Store decision
        {
            let mut decisions = self.decisions.write().unwrap();
            decisions.insert(decision_id.clone(), decision);
        }
        
        // Auto-execute if no appeal window
        if self.params.appeal_window == 0 {
            self.execute_slashing(&decision_id).await?;
        }
        
        Ok(decision_id)
    }
    
    /// Execute a slashing decision
    pub async fn execute_slashing(&self, decision_id: &str) -> Result<TxHash> {
        info!("Executing slashing decision: {}", decision_id);
        
        let mut decision = {
            let decisions = self.decisions.read().unwrap();
            decisions.get(decision_id).cloned()
                .ok_or_else(|| EconomyError::ConfigurationError("Decision not found".to_string()))?
        };
        
        // Check if already executed
        if decision.status == SlashingStatus::Executed {
            return Err(EconomyError::ConfigurationError("Already executed".to_string()));
        }
        
        // Check if appeal window has passed
        let current_time = self.get_current_timestamp();
        if current_time < decision.decided_at + self.params.appeal_window {
            return Err(EconomyError::ConfigurationError("Still in appeal window".to_string()));
        }
        
        // Check if appealed
        if decision.status == SlashingStatus::Appealed {
            return Err(EconomyError::ConfigurationError("Decision is under appeal".to_string()));
        }
        
        // Execute penalty
        let tx_hash = if decision.penalty_amount > Decimal::ZERO {
            // Burn the slashed tokens
            self.ruv_token.burn(&decision.offender, decision.penalty_amount).await?
        } else {
            TxHash::default()
        };
        
        // Update reputation
        self.update_reputation(
            &decision.offender,
            decision.reputation_penalty,
            &decision.offense,
            decision.ban_duration,
        ).await?;
        
        // Update offense counter
        {
            let mut counters = self.offense_counters.write().unwrap();
            let key = (decision.offender.clone(), decision.offense.clone());
            let count = counters.entry(key).or_insert(0);
            *count += 1;
        }
        
        // Update decision status
        decision.status = SlashingStatus::Executed;
        decision.executed_at = Some(current_time);
        
        {
            let mut decisions = self.decisions.write().unwrap();
            decisions.insert(decision_id.to_string(), decision);
        }
        
        // Reward reporter and witnesses
        self.distribute_slashing_rewards(&decision.evidence).await?;
        
        info!("Slashing executed successfully with tx: {}", tx_hash);
        Ok(tx_hash)
    }
    
    /// Submit an appeal against a slashing decision
    pub async fn submit_appeal(
        &self,
        decision_id: &str,
        appellant: &Address,
        reason: String,
        evidence: Vec<u8>,
    ) -> Result<String> {
        info!("Submitting appeal for decision {} by {}", decision_id, appellant);
        
        let mut decision = {
            let decisions = self.decisions.read().unwrap();
            decisions.get(decision_id).cloned()
                .ok_or_else(|| EconomyError::ConfigurationError("Decision not found".to_string()))?
        };
        
        // Verify appellant is the offender
        if decision.offender != *appellant {
            return Err(EconomyError::ConfigurationError(
                "Only the offender can appeal".to_string()
            ));
        }
        
        // Check if still in appeal window
        let current_time = self.get_current_timestamp();
        if current_time > decision.decided_at + self.params.appeal_window {
            return Err(EconomyError::ConfigurationError("Appeal window has closed".to_string()));
        }
        
        // Check if already appealed
        if decision.appeal.is_some() {
            return Err(EconomyError::ConfigurationError("Already appealed".to_string()));
        }
        
        // Create appeal
        let appeal_id = format!("appeal_{}", uuid::Uuid::new_v4());
        let appeal = Appeal {
            id: appeal_id.clone(),
            appellant: appellant.clone(),
            reason,
            evidence,
            submitted_at: current_time,
            status: AppealStatus::Pending,
            decision: None,
        };
        
        // Update decision
        decision.appeal = Some(appeal);
        decision.status = SlashingStatus::Appealed;
        
        {
            let mut decisions = self.decisions.write().unwrap();
            decisions.insert(decision_id.to_string(), decision);
        }
        
        info!("Appeal {} submitted successfully", appeal_id);
        Ok(appeal_id)
    }
    
    /// Process an appeal (committee decision)
    pub async fn process_appeal(
        &self,
        decision_id: &str,
        committee_decisions: Vec<(Address, bool)>, // address, accept_appeal
        reason: String,
    ) -> Result<()> {
        info!("Processing appeal for decision {}", decision_id);
        
        let mut decision = {
            let decisions = self.decisions.read().unwrap();
            decisions.get(decision_id).cloned()
                .ok_or_else(|| EconomyError::ConfigurationError("Decision not found".to_string()))?
        };
        
        let appeal = decision.appeal.as_mut()
            .ok_or_else(|| EconomyError::ConfigurationError("No appeal found".to_string()))?;
        
        // Verify committee members
        let committee = self.slashing_committee.read().unwrap();
        for (member, _) in &committee_decisions {
            if !committee.contains(member) {
                return Err(EconomyError::ConfigurationError(
                    format!("{} is not a committee member", member)
                ));
            }
        }
        
        // Count votes
        let accept_votes = committee_decisions.iter().filter(|(_, accept)| *accept).count();
        let total_votes = committee_decisions.len();
        let accepted = accept_votes > total_votes / 2;
        
        // Create appeal decision
        let appeal_decision = AppealDecision {
            decided_by: committee_decisions.iter().map(|(addr, _)| addr.clone()).collect(),
            decision: if accepted { AppealStatus::Accepted } else { AppealStatus::Rejected },
            reason,
            decided_at: self.get_current_timestamp(),
        };
        
        appeal.status = appeal_decision.decision.clone();
        appeal.decision = Some(appeal_decision);
        
        // Update main decision status
        if accepted {
            decision.status = SlashingStatus::Reversed;
            
            // Reverse reputation penalty
            if let Some(reputation) = self.reputations.write().unwrap().get_mut(&decision.offender) {
                reputation.reputation_score += decision.reputation_penalty;
                reputation.total_offenses = reputation.total_offenses.saturating_sub(1);
            }
        } else {
            // Execute the slashing since appeal was rejected
            drop(committee);
            self.execute_slashing(decision_id).await?;
            return Ok(());
        }
        
        {
            let mut decisions = self.decisions.write().unwrap();
            decisions.insert(decision_id.to_string(), decision);
        }
        
        info!("Appeal processed: {}", if accepted { "Accepted" } else { "Rejected" });
        Ok(())
    }
    
    /// Calculate penalties based on offense type and history
    async fn calculate_penalties(
        &self,
        offender: &Address,
        offense: &SlashingOffense,
    ) -> Result<(Decimal, i32, Option<u64>)> {
        // Get offender's stake
        let stake = self.ruv_token.get_locked_balance(offender).await?;
        
        // Get offense history
        let offense_count = {
            let counters = self.offense_counters.read().unwrap();
            counters.get(&(offender.clone(), offense.clone())).copied().unwrap_or(0)
        };
        
        // Base penalty calculation
        let (base_penalty_rate, base_reputation_penalty, base_ban_duration) = match offense {
            SlashingOffense::DoubleSigning { .. } => {
                (self.params.double_signing_penalty_rate, -100, Some(30 * 24 * 3600)) // 30 days
            },
            SlashingOffense::Downtime { missed_blocks, .. } => {
                let rate = self.params.downtime_penalty_rate * 
                    Decimal::from(*missed_blocks) / Decimal::from(100); // Scale by missed blocks
                (rate.min(Decimal::new(50, 2)), -20, None) // Max 50% penalty
            },
            SlashingOffense::InvalidBlock { .. } => {
                (self.params.invalid_block_penalty_rate, -50, Some(7 * 24 * 3600)) // 7 days
            },
            SlashingOffense::Censorship { censored_tx_hashes, .. } => {
                let rate = self.params.censorship_penalty_rate * 
                    Decimal::from(censored_tx_hashes.len() as u64);
                (rate.min(Decimal::new(30, 2)), -40, None) // Max 30% penalty
            },
            SlashingOffense::ResourceMisconduct { violation_type, .. } => {
                let (rate, rep) = match violation_type {
                    ResourceViolationType::ResourceUnavailable => (Decimal::new(10, 2), -30),
                    ResourceViolationType::ResourceQualityViolation => (Decimal::new(5, 2), -20),
                    ResourceViolationType::PriceManipulation => (Decimal::new(15, 2), -40),
                    ResourceViolationType::MetricsFalsification => (Decimal::new(20, 2), -60),
                };
                (rate, rep, None)
            },
            SlashingOffense::NetworkAttack { severity, .. } => {
                let base_rate = match severity {
                    SeverityLevel::Low => Decimal::new(10, 2),
                    SeverityLevel::Medium => Decimal::new(25, 2),
                    SeverityLevel::High => Decimal::new(50, 2),
                    SeverityLevel::Critical => Decimal::new(100, 2),
                };
                let rate = base_rate * self.params.network_attack_penalty_multiplier;
                let rep = match severity {
                    SeverityLevel::Low => -50,
                    SeverityLevel::Medium => -100,
                    SeverityLevel::High => -200,
                    SeverityLevel::Critical => -500,
                };
                let ban = match severity {
                    SeverityLevel::Low => None,
                    SeverityLevel::Medium => Some(7 * 24 * 3600),
                    SeverityLevel::High => Some(30 * 24 * 3600),
                    SeverityLevel::Critical => Some(365 * 24 * 3600), // 1 year
                };
                (rate, rep, ban)
            },
            SlashingOffense::DataIntegrity { affected_nodes, .. } => {
                let rate = Decimal::new(5, 2) * Decimal::from(affected_nodes.len() as u64);
                (rate.min(Decimal::new(25, 2)), -30, None)
            },
            SlashingOffense::GovernanceManipulation { .. } => {
                (Decimal::new(30, 2), -150, Some(14 * 24 * 3600)) // 14 days
            },
        };
        
        // Apply repeat offender multiplier
        let repeat_multiplier = Decimal::new(1, 0) + Decimal::from(offense_count) * Decimal::new(5, 1);
        let penalty_rate = (base_penalty_rate * repeat_multiplier).min(Decimal::new(100, 2)); // Max 100%
        
        let penalty_amount = stake * penalty_rate;
        let reputation_penalty = base_reputation_penalty * (1 + offense_count as i32);
        
        // Extend ban duration for repeat offenders
        let ban_duration = base_ban_duration.map(|d| d * (1 + offense_count));
        
        Ok((penalty_amount, reputation_penalty, ban_duration))
    }
    
    /// Update node reputation
    async fn update_reputation(
        &self,
        address: &Address,
        reputation_change: i32,
        offense: &SlashingOffense,
        ban_duration: Option<u64>,
    ) -> Result<()> {
        let mut reputations = self.reputations.write().unwrap();
        
        let reputation = reputations.entry(address.clone()).or_insert(NodeReputation {
            address: address.clone(),
            reputation_score: 0,
            total_offenses: 0,
            offense_history: VecDeque::new(),
            last_offense_time: None,
            is_banned: false,
            ban_expiry: None,
        });
        
        // Update reputation score
        reputation.reputation_score += reputation_change;
        reputation.total_offenses += 1;
        reputation.last_offense_time = Some(self.get_current_timestamp());
        
        // Add to offense history (keep last 100)
        reputation.offense_history.push_back((offense.clone(), self.get_current_timestamp()));
        if reputation.offense_history.len() > 100 {
            reputation.offense_history.pop_front();
        }
        
        // Apply ban if necessary
        if let Some(duration) = ban_duration {
            reputation.is_banned = true;
            reputation.ban_expiry = Some(self.get_current_timestamp() + duration);
        }
        
        // Auto-ban if reputation too low
        if reputation.reputation_score <= self.params.ban_threshold_score {
            reputation.is_banned = true;
            reputation.ban_expiry = Some(self.get_current_timestamp() + 365 * 24 * 3600); // 1 year
            warn!("Node {} auto-banned due to low reputation score: {}", 
                  address, reputation.reputation_score);
        }
        
        Ok(())
    }
    
    /// Distribute rewards to reporter and witnesses
    async fn distribute_slashing_rewards(&self, evidence: &SlashingEvidence) -> Result<()> {
        // In a real implementation, a portion of slashed tokens could be
        // distributed to the reporter and witnesses as incentive
        
        // For now, we just log it
        info!("Distributing slashing rewards to reporter {} and {} witnesses",
              evidence.reporter, evidence.witness_signatures.len());
        
        Ok(())
    }
    
    /// Get offender address from offense
    fn get_offender_from_offense(&self, offense: &SlashingOffense) -> Option<Address> {
        // In a real implementation, this would extract the offender's address
        // from the offense details. For now, we return a placeholder
        match offense {
            SlashingOffense::ResourceMisconduct { reservation_id, .. } => {
                // Would look up reservation to find provider
                Some(Address::from_hex("0x0000000000000000000000000000000000000000").unwrap())
            },
            _ => {
                // Would extract from block/transaction data
                Some(Address::from_hex("0x0000000000000000000000000000000000000000").unwrap())
            }
        }
    }
    
    /// Check if a node is banned
    pub async fn is_banned(&self, address: &Address) -> bool {
        let reputations = self.reputations.read().unwrap();
        
        if let Some(reputation) = reputations.get(address) {
            if reputation.is_banned {
                if let Some(expiry) = reputation.ban_expiry {
                    return self.get_current_timestamp() < expiry;
                }
                return true;
            }
        }
        
        false
    }
    
    /// Get node reputation
    pub async fn get_reputation(&self, address: &Address) -> Option<NodeReputation> {
        let reputations = self.reputations.read().unwrap();
        reputations.get(address).cloned()
    }
    
    /// Update slashing committee
    pub async fn update_committee(&self, members: Vec<Address>) -> Result<()> {
        info!("Updating slashing committee with {} members", members.len());
        
        if members.len() < 3 {
            return Err(EconomyError::ConfigurationError(
                "Committee must have at least 3 members".to_string()
            ));
        }
        
        let mut committee = self.slashing_committee.write().unwrap();
        *committee = members;
        
        Ok(())
    }
    
    /// Process reputation recovery over time
    pub async fn process_reputation_recovery(&self) -> Result<()> {
        let current_time = self.get_current_timestamp();
        let mut reputations = self.reputations.write().unwrap();
        
        for reputation in reputations.values_mut() {
            // Skip if recently offended
            if let Some(last_offense) = reputation.last_offense_time {
                if current_time - last_offense < 24 * 3600 {
                    continue;
                }
            }
            
            // Recover reputation gradually
            if reputation.reputation_score < 0 {
                reputation.reputation_score = 
                    (reputation.reputation_score + self.params.reputation_recovery_rate).min(0);
            }
            
            // Check ban expiry
            if reputation.is_banned {
                if let Some(expiry) = reputation.ban_expiry {
                    if current_time >= expiry {
                        reputation.is_banned = false;
                        reputation.ban_expiry = None;
                        info!("Ban expired for node {}", reputation.address);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get slashing statistics
    pub async fn get_slashing_stats(&self) -> SlashingStats {
        let decisions = self.decisions.read().unwrap();
        let reputations = self.reputations.read().unwrap();
        
        let total_slashed: Decimal = decisions.values()
            .filter(|d| d.status == SlashingStatus::Executed)
            .map(|d| d.penalty_amount)
            .sum();
        
        let banned_nodes = reputations.values()
            .filter(|r| r.is_banned)
            .count() as u64;
        
        SlashingStats {
            total_decisions: decisions.len() as u64,
            executed_slashings: decisions.values()
                .filter(|d| d.status == SlashingStatus::Executed)
                .count() as u64,
            pending_decisions: decisions.values()
                .filter(|d| d.status == SlashingStatus::Pending)
                .count() as u64,
            appealed_decisions: decisions.values()
                .filter(|d| d.status == SlashingStatus::Appealed)
                .count() as u64,
            total_amount_slashed: total_slashed,
            banned_nodes,
            active_evidence: self.evidence_pool.read().unwrap().len() as u64,
        }
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
pub struct SlashingStats {
    pub total_decisions: u64,
    pub executed_slashings: u64,
    pub pending_decisions: u64,
    pub appealed_decisions: u64,
    pub total_amount_slashed: Decimal,
    pub banned_nodes: u64,
    pub active_evidence: u64,
}

/// Slashing interface for external integration
#[async_trait]
pub trait SlashingInterface: Send + Sync {
    async fn report_offense(
        &self,
        reporter: &Address,
        offense: SlashingOffense,
        proof: Option<Vec<u8>>,
    ) -> Result<String>;
    
    async fn is_banned(&self, address: &Address) -> bool;
    
    async fn get_reputation(&self, address: &Address) -> Option<NodeReputation>;
    
    async fn get_slashing_stats(&self) -> SlashingStats;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_severity_ordering() {
        assert!(SeverityLevel::Low < SeverityLevel::Medium);
        assert!(SeverityLevel::Medium < SeverityLevel::High);
        assert!(SeverityLevel::High < SeverityLevel::Critical);
    }
    
    #[test]
    fn test_offense_creation() {
        let offense = SlashingOffense::DoubleSigning {
            block_height: 12345,
            signature1: vec![1, 2, 3],
            signature2: vec![4, 5, 6],
        };
        
        assert!(matches!(offense, SlashingOffense::DoubleSigning { .. }));
    }
    
    #[tokio::test]
    async fn test_penalty_calculation() {
        let config = EconomyConfig::default();
        let params = SlashingParams::default();
        
        // Mock dependencies would be needed for a full test
        // This is a placeholder for the test structure
        
        let offense = SlashingOffense::Downtime {
            start_block: 1000,
            end_block: 2000,
            missed_blocks: 100,
        };
        
        // Test would verify penalty calculation logic
        assert_eq!(params.downtime_penalty_rate, Decimal::new(5, 2));
    }
}