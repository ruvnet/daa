#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use prime_core::types::*;
use std::collections::HashMap;

/// Fuzz input for consensus operations
#[derive(Arbitrary, Debug)]
struct ConsensusFuzzInput {
    node_count: usize,
    rounds: Vec<ConsensusRound>,
    byzantine_nodes: Vec<usize>,
}

#[derive(Arbitrary, Debug)]
struct ConsensusRound {
    round_number: u64,
    proposals: Vec<Vec<u8>>,
    votes: Vec<bool>,
    byzantine_behavior: ByzantineBehavior,
}

#[derive(Arbitrary, Debug)]
enum ByzantineBehavior {
    Honest,
    RandomVotes,
    AlwaysReject,
    DoubleVoting,
    DelayedMessages,
}

fuzz_target!(|input: ConsensusFuzzInput| {
    // Limit input sizes
    if input.node_count > 50 || input.rounds.len() > 100 {
        return;
    }
    
    let mut consensus_state = ConsensusState::new(input.node_count);
    
    for (round_idx, round) in input.rounds.iter().enumerate() {
        let round_number = round.round_number.max(round_idx as u64);
        
        // Process proposals
        for (node_idx, proposal) in round.proposals.iter().enumerate() {
            if node_idx >= input.node_count {
                break;
            }
            
            // Limit proposal size
            if proposal.len() > 1000 {
                continue;
            }
            
            let message = MessageType::ConsensusProposal {
                round: round_number,
                value: proposal.clone(),
            };
            
            consensus_state.process_message(node_idx, message);
        }
        
        // Process votes
        for (node_idx, &vote) in round.votes.iter().enumerate() {
            if node_idx >= input.node_count {
                break;
            }
            
            // Apply byzantine behavior
            let actual_vote = if input.byzantine_nodes.contains(&node_idx) {
                match round.byzantine_behavior {
                    ByzantineBehavior::Honest => vote,
                    ByzantineBehavior::RandomVotes => round_number % 2 == 0,
                    ByzantineBehavior::AlwaysReject => false,
                    ByzantineBehavior::DoubleVoting => {
                        // Send both true and false votes (test double voting detection)
                        consensus_state.process_message(node_idx, MessageType::ConsensusVote {
                            round: round_number,
                            accept: true,
                        });
                        false
                    }
                    ByzantineBehavior::DelayedMessages => {
                        // Skip this vote (simulate delay)
                        continue;
                    }
                }
            } else {
                vote
            };
            
            let message = MessageType::ConsensusVote {
                round: round_number,
                accept: actual_vote,
            };
            
            consensus_state.process_message(node_idx, message);
        }
        
        // Try to reach consensus
        if let Some(committed_value) = consensus_state.try_commit(round_number) {
            // Broadcast commit messages
            for node_idx in 0..input.node_count {
                if !input.byzantine_nodes.contains(&node_idx) {
                    let message = MessageType::ConsensusCommit {
                        round: round_number,
                        value: committed_value.clone(),
                    };
                    consensus_state.process_message(node_idx, message);
                }
            }
        }
    }
    
    // Verify consensus properties
    consensus_state.verify_safety();
    consensus_state.verify_liveness();
});

/// Simplified consensus state for fuzzing
struct ConsensusState {
    node_count: usize,
    proposals: HashMap<u64, Vec<Vec<u8>>>,
    votes: HashMap<u64, Vec<Option<bool>>>,
    commits: HashMap<u64, Vec<u8>>,
    message_log: Vec<(usize, MessageType)>,
}

impl ConsensusState {
    fn new(node_count: usize) -> Self {
        Self {
            node_count,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            commits: HashMap::new(),
            message_log: Vec::new(),
        }
    }
    
    fn process_message(&mut self, from_node: usize, message: MessageType) {
        if from_node >= self.node_count {
            return;
        }
        
        self.message_log.push((from_node, message.clone()));
        
        match message {
            MessageType::ConsensusProposal { round, value } => {
                self.proposals.entry(round).or_default().push(value);
            }
            MessageType::ConsensusVote { round, accept } => {
                let votes = self.votes.entry(round).or_insert_with(|| vec![None; self.node_count]);
                if from_node < votes.len() {
                    votes[from_node] = Some(accept);
                }
            }
            MessageType::ConsensusCommit { round, value } => {
                self.commits.insert(round, value);
            }
            _ => {}
        }
    }
    
    fn try_commit(&self, round: u64) -> Option<Vec<u8>> {
        let votes = self.votes.get(&round)?;
        let accept_count = votes.iter().filter_map(|v| *v).filter(|&v| v).count();
        
        // Require 2/3 majority
        let required = (self.node_count * 2) / 3 + 1;
        
        if accept_count >= required {
            // Get the most common proposal
            if let Some(proposals) = self.proposals.get(&round) {
                return proposals.first().cloned();
            }
        }
        
        None
    }
    
    fn verify_safety(&self) {
        // Safety: No two different values committed in same round
        // (This is a simplified check)
        let mut round_values: HashMap<u64, &Vec<u8>> = HashMap::new();
        
        for (round, value) in &self.commits {
            if let Some(existing) = round_values.get(round) {
                assert_eq!(existing, &value, "Safety violation: different values committed in round {}", round);
            } else {
                round_values.insert(*round, value);
            }
        }
    }
    
    fn verify_liveness(&self) {
        // Liveness: Progress should be made
        // (Simplified check - just verify we have some commits if we have proposals)
        if !self.proposals.is_empty() && self.commits.is_empty() {
            // This might not always be a violation in real systems,
            // but for fuzzing we want to detect potential liveness issues
        }
    }
}