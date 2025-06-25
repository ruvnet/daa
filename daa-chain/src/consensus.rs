//! Consensus mechanism for DAA Chain using QuDAG consensus primitives

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

use crate::qudag_stubs::qudag_consensus::ConsensusEngine as QuDAGConsensus;
use crate::qudag_stubs::qudag_core::{Block, Hash};
use crate::{ChainConfig, Result, ChainError};

/// Consensus engine for DAA Chain
pub struct ConsensusEngine {
    /// Underlying QuDAG consensus
    qudag_consensus: QuDAGConsensus,
    
    /// Chain configuration
    config: ChainConfig,
    
    /// Current consensus state
    state: Arc<RwLock<DaaConsensusState>>,
    
    /// Event broadcaster
    event_sender: broadcast::Sender<ConsensusEvent>,
    
    /// Validator set
    validators: Arc<RwLock<HashMap<String, ValidatorInfo>>>,
}

/// DAA-specific consensus state
#[derive(Debug, Clone)]
pub struct DaaConsensusState {
    /// Current epoch
    pub epoch: u64,
    
    /// Current round within epoch
    pub round: u64,
    
    /// Current leader for this round
    pub current_leader: Option<String>,
    
    /// Votes collected for current round
    pub votes: HashMap<String, Vote>,
    
    /// Proposed blocks for current round
    pub proposals: HashMap<String, Block>,
    
    /// Committed blocks
    pub committed_blocks: HashSet<Hash>,
    
    /// Last finalized height
    pub finalized_height: u64,
}

impl Default for DaaConsensusState {
    fn default() -> Self {
        Self {
            epoch: 0,
            round: 0,
            current_leader: None,
            votes: HashMap::new(),
            proposals: HashMap::new(),
            committed_blocks: HashSet::new(),
            finalized_height: 0,
        }
    }
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator ID
    pub validator_id: String,
    
    /// Public key for validation
    pub public_key: Vec<u8>,
    
    /// Stake amount
    pub stake: u64,
    
    /// Reputation score
    pub reputation: f64,
    
    /// Last seen timestamp
    pub last_seen: u64,
    
    /// Whether validator is active
    pub is_active: bool,
}

/// Consensus vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Validator ID
    pub validator_id: String,
    
    /// Epoch number
    pub epoch: u64,
    
    /// Round number
    pub round: u64,
    
    /// Block hash being voted on
    pub block_hash: Hash,
    
    /// Vote type
    pub vote_type: VoteType,
    
    /// Signature
    pub signature: Vec<u8>,
    
    /// Timestamp
    pub timestamp: u64,
}

/// Types of consensus votes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    /// Proposal vote (initial proposal)
    Proposal,
    
    /// Prevote (first round of voting)
    Prevote,
    
    /// Precommit (second round of voting)
    Precommit,
    
    /// Commit (final commitment)
    Commit,
}

/// Consensus events
#[derive(Debug, Clone)]
pub enum ConsensusEvent {
    /// New proposal received
    ProposalReceived {
        proposer: String,
        block: Block,
        epoch: u64,
        round: u64,
    },
    
    /// Vote received
    VoteReceived {
        voter: String,
        vote: Vote,
    },
    
    /// Block committed
    BlockCommitted {
        block: Block,
        epoch: u64,
        round: u64,
    },
    
    /// Round timeout
    RoundTimeout {
        epoch: u64,
        round: u64,
    },
    
    /// Leader changed
    LeaderChanged {
        old_leader: Option<String>,
        new_leader: String,
        epoch: u64,
        round: u64,
    },
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub async fn new(config: &ChainConfig) -> Result<Self> {
        let qudag_consensus = QuDAGConsensus::new().await
            .map_err(|e| ChainError::Consensus(format!("QuDAG consensus init failed: {}", e)))?;
        
        let (event_sender, _) = broadcast::channel(1000);
        
        Ok(Self {
            qudag_consensus,
            config: config.clone(),
            state: Arc::new(RwLock::new(DaaConsensusState::default())),
            event_sender,
            validators: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the consensus engine
    pub async fn start(&mut self) -> Result<()> {
        // Start QuDAG consensus
        self.qudag_consensus.start().await
            .map_err(|e| ChainError::Consensus(format!("Failed to start consensus: {}", e)))?;
        
        // Start consensus rounds
        self.start_consensus_loop().await?;
        
        Ok(())
    }

    /// Subscribe to consensus events
    pub fn subscribe(&self) -> broadcast::Receiver<ConsensusEvent> {
        self.event_sender.subscribe()
    }

    /// Check if this node should produce a block
    pub async fn should_produce_block(&self) -> Result<bool> {
        let state = self.state.read().await;
        
        // Simple leader selection based on round-robin
        if let Some(leader) = &state.current_leader {
            // Check if we are the current leader
            // This would be determined by comparing with our node ID
            Ok(leader == "our_node_id") // Placeholder logic
        } else {
            Ok(false)
        }
    }

    /// Propose a block for consensus
    pub async fn propose_block(&mut self, block: Block) -> Result<()> {
        let mut state = self.state.write().await;
        
        // Add proposal to current round
        state.proposals.insert("our_node_id".to_string(), block.clone());
        
        // Broadcast proposal
        let _ = self.event_sender.send(ConsensusEvent::ProposalReceived {
            proposer: "our_node_id".to_string(),
            block,
            epoch: state.epoch,
            round: state.round,
        });
        
        Ok(())
    }

    /// Submit a vote for consensus
    pub async fn submit_vote(&mut self, vote: Vote) -> Result<()> {
        // Validate vote
        self.validate_vote(&vote).await?;
        
        // Add vote to state
        let mut state = self.state.write().await;
        state.votes.insert(vote.validator_id.clone(), vote.clone());
        
        // Check if we have enough votes to proceed
        if self.check_vote_threshold(&state, &vote.vote_type).await? {
            match vote.vote_type {
                VoteType::Prevote => {
                    // Move to precommit phase
                    self.advance_to_precommit(&mut state).await?;
                }
                VoteType::Precommit => {
                    // Check if we can commit the block
                    if let Some(block) = self.find_committable_block(&state).await? {
                        self.commit_block(&mut state, block).await?;
                    }
                }
                _ => {}
            }
        }
        
        // Broadcast vote event
        let _ = self.event_sender.send(ConsensusEvent::VoteReceived {
            voter: vote.validator_id.clone(),
            vote,
        });
        
        Ok(())
    }

    /// Add validator to the set
    pub async fn add_validator(&mut self, validator: ValidatorInfo) -> Result<()> {
        self.validators.write().await.insert(validator.validator_id.clone(), validator);
        Ok(())
    }

    /// Remove validator from the set
    pub async fn remove_validator(&mut self, validator_id: &str) -> Result<()> {
        self.validators.write().await.remove(validator_id);
        Ok(())
    }

    /// Get current validator set
    pub async fn get_validators(&self) -> Vec<ValidatorInfo> {
        self.validators.read().await.values().cloned().collect()
    }

    /// Start consensus loop
    async fn start_consensus_loop(&mut self) -> Result<()> {
        let event_sender = self.event_sender.clone();
        let state = Arc::clone(&self.state);
        let validators = Arc::clone(&self.validators);
        
        tokio::spawn(async move {
            let mut round_timer = tokio::time::interval(Duration::from_secs(15)); // 15 second rounds
            
            loop {
                round_timer.tick().await;
                
                let mut state_guard = state.write().await;
                
                // Check for round timeout
                if Self::should_timeout_round(&state_guard).await {
                    // Advance to next round
                    state_guard.round += 1;
                    state_guard.votes.clear();
                    state_guard.proposals.clear();
                    
                    // Select new leader
                    if let Some(new_leader) = Self::select_leader(&validators, state_guard.epoch, state_guard.round).await {
                        let old_leader = state_guard.current_leader.clone();
                        state_guard.current_leader = Some(new_leader.clone());
                        
                        let _ = event_sender.send(ConsensusEvent::LeaderChanged {
                            old_leader,
                            new_leader,
                            epoch: state_guard.epoch,
                            round: state_guard.round,
                        });
                    }
                    
                    let _ = event_sender.send(ConsensusEvent::RoundTimeout {
                        epoch: state_guard.epoch,
                        round: state_guard.round,
                    });
                }
            }
        });
        
        Ok(())
    }

    /// Validate a consensus vote
    async fn validate_vote(&self, vote: &Vote) -> Result<()> {
        // Check if validator exists
        let validators = self.validators.read().await;
        let validator = validators.get(&vote.validator_id)
            .ok_or_else(|| ChainError::Consensus("Unknown validator".to_string()))?;
        
        if !validator.is_active {
            return Err(ChainError::Consensus("Inactive validator".to_string()));
        }
        
        // Validate signature (placeholder - would use actual crypto verification)
        if vote.signature.is_empty() {
            return Err(ChainError::Consensus("Missing vote signature".to_string()));
        }
        
        Ok(())
    }

    /// Check if vote threshold is met
    async fn check_vote_threshold(&self, state: &DaaConsensusState, vote_type: &VoteType) -> Result<bool> {
        let validators = self.validators.read().await;
        let total_stake: u64 = validators.values().map(|v| v.stake).sum();
        let threshold = (total_stake * 2) / 3; // 2/3 majority
        
        let votes_for_type: u64 = state.votes.values()
            .filter(|v| std::mem::discriminant(&v.vote_type) == std::mem::discriminant(vote_type))
            .filter_map(|v| validators.get(&v.validator_id).map(|val| val.stake))
            .sum();
        
        Ok(votes_for_type > threshold)
    }

    /// Advance to precommit phase
    async fn advance_to_precommit(&self, state: &mut DaaConsensusState) -> Result<()> {
        // Logic to advance consensus to precommit phase
        tracing::info!("Advancing to precommit phase for round {}", state.round);
        Ok(())
    }

    /// Find a block that can be committed
    async fn find_committable_block(&self, state: &DaaConsensusState) -> Result<Option<Block>> {
        // Find block with enough precommit votes
        let validators = self.validators.read().await;
        let threshold = (validators.values().map(|v| v.stake).sum::<u64>() * 2) / 3;
        
        for (_proposer, block) in &state.proposals {
            let votes_for_block: u64 = state.votes.values()
                .filter(|v| matches!(v.vote_type, VoteType::Precommit))
                .filter(|v| v.block_hash == block.hash())
                .filter_map(|v| validators.get(&v.validator_id).map(|val| val.stake))
                .sum();
            
            if votes_for_block > threshold {
                return Ok(Some(block.clone()));
            }
        }
        
        Ok(None)
    }

    /// Commit a block
    async fn commit_block(&self, state: &mut DaaConsensusState, block: Block) -> Result<()> {
        let block_hash = block.hash();
        
        // Mark block as committed
        state.committed_blocks.insert(block_hash);
        state.finalized_height += 1;
        
        // Clear votes and proposals for next round
        state.votes.clear();
        state.proposals.clear();
        state.round += 1;
        
        // Broadcast commit event
        let _ = self.event_sender.send(ConsensusEvent::BlockCommitted {
            block,
            epoch: state.epoch,
            round: state.round - 1,
        });
        
        tracing::info!("Committed block {} at height {}", block_hash, state.finalized_height);
        
        Ok(())
    }

    /// Check if round should timeout
    async fn should_timeout_round(state: &DaaConsensusState) -> bool {
        // Simple timeout logic - in practice would be more sophisticated
        state.votes.is_empty() || state.proposals.is_empty()
    }

    /// Select leader for round
    async fn select_leader(
        validators: &RwLock<HashMap<String, ValidatorInfo>>,
        epoch: u64,
        round: u64,
    ) -> Option<String> {
        let validators_guard = validators.read().await;
        let active_validators: Vec<_> = validators_guard.values()
            .filter(|v| v.is_active)
            .collect();
        
        if active_validators.is_empty() {
            return None;
        }
        
        // Simple round-robin selection
        let index = ((epoch + round) as usize) % active_validators.len();
        Some(active_validators[index].validator_id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChainConfig;

    #[tokio::test]
    async fn test_consensus_engine_creation() {
        let config = ChainConfig::default();
        let engine = ConsensusEngine::new(&config).await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_validator_management() {
        let config = ChainConfig::default();
        let mut engine = ConsensusEngine::new(&config).await.unwrap();
        
        let validator = ValidatorInfo {
            validator_id: "test-validator".to_string(),
            public_key: vec![0u8; 32],
            stake: 1000,
            reputation: 1.0,
            last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            is_active: true,
        };
        
        engine.add_validator(validator).await.unwrap();
        
        let validators = engine.get_validators().await;
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].validator_id, "test-validator");
    }

    #[tokio::test]
    async fn test_vote_validation() {
        let config = ChainConfig::default();
        let mut engine = ConsensusEngine::new(&config).await.unwrap();
        
        // Add validator first
        let validator = ValidatorInfo {
            validator_id: "test-validator".to_string(),
            public_key: vec![0u8; 32],
            stake: 1000,
            reputation: 1.0,
            last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            is_active: true,
        };
        engine.add_validator(validator).await.unwrap();
        
        let vote = Vote {
            validator_id: "test-validator".to_string(),
            epoch: 0,
            round: 0,
            block_hash: Hash::default(),
            vote_type: VoteType::Prevote,
            signature: vec![1u8; 64], // Dummy signature
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        let result = engine.validate_vote(&vote).await;
        assert!(result.is_ok());
    }
}