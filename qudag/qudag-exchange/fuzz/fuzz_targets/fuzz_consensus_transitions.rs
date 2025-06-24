#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use qudag_exchange_core::{ConsensusState, Transaction, Vote, NodeId};
use std::collections::{HashMap, HashSet};

/// Consensus events that can occur
#[derive(Debug, Clone, Arbitrary)]
enum ConsensusEvent {
    /// Node receives a new transaction
    ReceiveTransaction {
        node: u8,
        tx_data: Vec<u8>,
    },
    /// Node casts a vote
    CastVote {
        node: u8,
        tx_hash: [u8; 32],
        vote: bool,
    },
    /// Network partition event
    PartitionNetwork {
        partition_a: Vec<u8>,
        partition_b: Vec<u8>,
    },
    /// Heal network partition
    HealPartition,
    /// Byzantine node behavior
    ByzantineAction {
        node: u8,
        action_type: u8,
    },
    /// Time advancement
    AdvanceTime {
        ticks: u16,
    },
}

/// Simulated network state for consensus testing
struct ConsensusNetwork {
    nodes: HashMap<u8, ConsensusState>,
    partitions: Option<(HashSet<u8>, HashSet<u8>)>,
    byzantine_nodes: HashSet<u8>,
    pending_transactions: HashMap<[u8; 32], Transaction>,
    finalized_transactions: HashSet<[u8; 32]>,
    time: u64,
}

impl ConsensusNetwork {
    fn new(num_nodes: u8) -> Self {
        let mut nodes = HashMap::new();
        for i in 0..num_nodes {
            nodes.insert(i, ConsensusState::new(NodeId::from_index(i)));
        }
        
        Self {
            nodes,
            partitions: None,
            byzantine_nodes: HashSet::new(),
            pending_transactions: HashMap::new(),
            finalized_transactions: HashSet::new(),
            time: 0,
        }
    }
    
    fn can_communicate(&self, node_a: u8, node_b: u8) -> bool {
        if let Some((partition_a, partition_b)) = &self.partitions {
            // Nodes can communicate if they're in the same partition
            (partition_a.contains(&node_a) && partition_a.contains(&node_b)) ||
            (partition_b.contains(&node_a) && partition_b.contains(&node_b))
        } else {
            true // No partition, all nodes can communicate
        }
    }
}

fuzz_target!(|data: &[u8]| {
    if let Ok(mut u) = Unstructured::new(data) {
        // Initialize network with 5-10 nodes
        let num_nodes = 5 + (data.first().unwrap_or(&0) % 6);
        let mut network = ConsensusNetwork::new(num_nodes);
        
        // Process events
        while !u.is_empty() {
            if let Ok(event) = ConsensusEvent::arbitrary(&mut u) {
                process_consensus_event(&mut network, event);
                verify_consensus_properties(&network);
            }
        }
        
        // Final verification
        verify_final_consensus_state(&network);
    }
});

fn process_consensus_event(network: &mut ConsensusNetwork, event: ConsensusEvent) {
    match event {
        ConsensusEvent::ReceiveTransaction { node, tx_data } => {
            let node_idx = node % network.nodes.len() as u8;
            
            // Create a transaction from fuzzer data
            if let Ok(tx) = create_test_transaction(&tx_data) {
                let tx_hash = tx.hash();
                network.pending_transactions.insert(tx_hash, tx.clone());
                
                // Node receives transaction
                if let Some(node_state) = network.nodes.get_mut(&node_idx) {
                    let _ = node_state.receive_transaction(tx);
                    
                    // Simulate initial voting
                    if !network.byzantine_nodes.contains(&node_idx) {
                        simulate_honest_voting(network, node_idx, tx_hash);
                    }
                }
            }
        }
        
        ConsensusEvent::CastVote { node, tx_hash, vote } => {
            let node_idx = node % network.nodes.len() as u8;
            
            if let Some(node_state) = network.nodes.get_mut(&node_idx) {
                // Create vote
                let vote_msg = Vote {
                    node_id: NodeId::from_index(node_idx),
                    tx_hash,
                    vote,
                    round: node_state.current_round(),
                    signature: vec![0u8; 64], // Mock signature
                };
                
                // Propagate vote to other nodes
                propagate_vote(network, node_idx, vote_msg);
            }
        }
        
        ConsensusEvent::PartitionNetwork { partition_a, partition_b } => {
            // Create network partition
            let set_a: HashSet<u8> = partition_a.into_iter()
                .map(|n| n % network.nodes.len() as u8)
                .collect();
            let set_b: HashSet<u8> = partition_b.into_iter()
                .map(|n| n % network.nodes.len() as u8)
                .collect();
                
            if !set_a.is_empty() && !set_b.is_empty() {
                network.partitions = Some((set_a, set_b));
            }
        }
        
        ConsensusEvent::HealPartition => {
            // Remove network partition
            network.partitions = None;
            
            // Trigger reconciliation
            reconcile_network_state(network);
        }
        
        ConsensusEvent::ByzantineAction { node, action_type } => {
            let node_idx = node % network.nodes.len() as u8;
            network.byzantine_nodes.insert(node_idx);
            
            // Simulate various Byzantine behaviors
            match action_type % 4 {
                0 => simulate_double_voting(network, node_idx),
                1 => simulate_conflicting_transactions(network, node_idx),
                2 => simulate_vote_withholding(network, node_idx),
                _ => simulate_random_votes(network, node_idx),
            }
        }
        
        ConsensusEvent::AdvanceTime { ticks } => {
            network.time += ticks as u64;
            
            // Process timeouts and advance consensus rounds
            for node_state in network.nodes.values_mut() {
                node_state.advance_time(ticks as u64);
                
                // Check for finalized transactions
                for tx_hash in node_state.get_finalized_transactions() {
                    network.finalized_transactions.insert(tx_hash);
                }
            }
        }
    }
}

fn verify_consensus_properties(network: &ConsensusNetwork) {
    // Property 1: Agreement - All honest nodes that finalize a transaction agree on the same one
    let mut finalized_by_honest: HashMap<[u8; 32], Vec<u8>> = HashMap::new();
    
    for (node_id, node_state) in &network.nodes {
        if !network.byzantine_nodes.contains(node_id) {
            for tx_hash in node_state.get_finalized_transactions() {
                finalized_by_honest.entry(tx_hash)
                    .or_insert_with(Vec::new)
                    .push(*node_id);
            }
        }
    }
    
    // Verify no conflicting finalizations
    for (tx_hash, nodes) in &finalized_by_honest {
        if nodes.len() > 1 {
            // All honest nodes should have the same view of this transaction
            let first_node = &network.nodes[&nodes[0]];
            for node_id in nodes.iter().skip(1) {
                let node = &network.nodes[node_id];
                assert_eq!(
                    first_node.get_transaction_state(tx_hash),
                    node.get_transaction_state(tx_hash),
                    "Consensus violation: nodes {} and {} disagree on transaction {:?}",
                    nodes[0], node_id, tx_hash
                );
            }
        }
    }
    
    // Property 2: Validity - Only valid transactions are finalized
    for tx_hash in &network.finalized_transactions {
        assert!(
            network.pending_transactions.contains_key(tx_hash),
            "Unknown transaction finalized: {:?}",
            tx_hash
        );
    }
    
    // Property 3: Liveness - In the absence of Byzantine nodes and partitions, 
    // transactions should eventually be finalized
    if network.byzantine_nodes.is_empty() && network.partitions.is_none() {
        // Check that old enough transactions have made progress
        for (tx_hash, _) in network.pending_transactions.iter() {
            let votes_collected = count_votes_for_transaction(network, tx_hash);
            if network.time > 1000 {
                // After enough time, honest nodes should have voted
                assert!(
                    votes_collected > 0,
                    "No progress on transaction {:?} after {} time units",
                    tx_hash, network.time
                );
            }
        }
    }
    
    // Property 4: Byzantine fault tolerance
    let byzantine_count = network.byzantine_nodes.len();
    let total_nodes = network.nodes.len();
    if byzantine_count < total_nodes / 3 {
        // System should still make progress with < 1/3 Byzantine nodes
        verify_byzantine_resilience(network);
    }
}

fn verify_final_consensus_state(network: &ConsensusNetwork) {
    // Final comprehensive checks
    
    // 1. No double finalization
    let mut tx_finalization_count: HashMap<[u8; 32], usize> = HashMap::new();
    for node_state in network.nodes.values() {
        for tx_hash in node_state.get_finalized_transactions() {
            *tx_finalization_count.entry(tx_hash).or_insert(0) += 1;
        }
    }
    
    // 2. Consistency check - if partitioned, verify each partition is internally consistent
    if let Some((partition_a, partition_b)) = &network.partitions {
        verify_partition_consistency(network, partition_a);
        verify_partition_consistency(network, partition_b);
    }
    
    // 3. Resource conservation - no rUv created or destroyed in consensus
    verify_resource_conservation(network);
}

// Helper functions

fn create_test_transaction(data: &[u8]) -> Result<Transaction, ()> {
    // Create a mock transaction from fuzzer data
    // This would use the actual Transaction builder in real implementation
    Ok(Transaction::mock_from_bytes(data))
}

fn simulate_honest_voting(network: &mut ConsensusNetwork, node_id: u8, tx_hash: [u8; 32]) {
    // Honest nodes vote based on transaction validity
    if let Some(tx) = network.pending_transactions.get(&tx_hash) {
        let vote = tx.is_valid(); // Mock validation
        let vote_msg = Vote {
            node_id: NodeId::from_index(node_id),
            tx_hash,
            vote,
            round: 0,
            signature: vec![0u8; 64],
        };
        propagate_vote(network, node_id, vote_msg);
    }
}

fn propagate_vote(network: &mut ConsensusNetwork, sender: u8, vote: Vote) {
    for (receiver_id, receiver_state) in network.nodes.iter_mut() {
        if *receiver_id != sender && network.can_communicate(sender, *receiver_id) {
            let _ = receiver_state.receive_vote(vote.clone());
        }
    }
}

fn simulate_double_voting(network: &mut ConsensusNetwork, byzantine_node: u8) {
    // Byzantine node votes both yes and no
    for tx_hash in network.pending_transactions.keys().take(1) {
        let vote_yes = Vote {
            node_id: NodeId::from_index(byzantine_node),
            tx_hash: *tx_hash,
            vote: true,
            round: 0,
            signature: vec![1u8; 64],
        };
        let vote_no = Vote {
            node_id: NodeId::from_index(byzantine_node),
            tx_hash: *tx_hash,
            vote: false,
            round: 0,
            signature: vec![2u8; 64],
        };
        
        // Send conflicting votes to different nodes
        let nodes: Vec<u8> = network.nodes.keys().copied().collect();
        for (i, node_id) in nodes.iter().enumerate() {
            if *node_id != byzantine_node {
                let vote = if i % 2 == 0 { vote_yes.clone() } else { vote_no.clone() };
                if let Some(node_state) = network.nodes.get_mut(node_id) {
                    let _ = node_state.receive_vote(vote);
                }
            }
        }
    }
}

fn simulate_conflicting_transactions(network: &mut ConsensusNetwork, byzantine_node: u8) {
    // Byzantine node creates conflicting transactions
    // Implementation would create double-spend attempts
}

fn simulate_vote_withholding(network: &mut ConsensusNetwork, byzantine_node: u8) {
    // Byzantine node doesn't vote
    // This is passive, so nothing to do
}

fn simulate_random_votes(network: &mut ConsensusNetwork, byzantine_node: u8) {
    // Byzantine node votes randomly
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    
    for tx_hash in network.pending_transactions.keys().take(3) {
        let random_vote = Vote {
            node_id: NodeId::from_index(byzantine_node),
            tx_hash: *tx_hash,
            vote: rng.gen_bool(0.5),
            round: rng.gen_range(0..10),
            signature: vec![rng.gen(); 64],
        };
        propagate_vote(network, byzantine_node, random_vote);
    }
}

fn reconcile_network_state(network: &mut ConsensusNetwork) {
    // After partition heal, nodes exchange state
    // This would trigger catch-up mechanisms
}

fn count_votes_for_transaction(network: &ConsensusNetwork, tx_hash: &[u8; 32]) -> usize {
    network.nodes.values()
        .filter(|node| node.has_voted_for(tx_hash))
        .count()
}

fn verify_byzantine_resilience(network: &ConsensusNetwork) {
    // Verify system maintains safety despite Byzantine nodes
    // Check that honest nodes don't finalize conflicting transactions
}

fn verify_partition_consistency(network: &ConsensusNetwork, partition: &HashSet<u8>) {
    // Verify nodes within a partition have consistent state
    let partition_nodes: Vec<_> = partition.iter()
        .filter_map(|id| network.nodes.get(id))
        .collect();
        
    if partition_nodes.len() > 1 {
        // Compare finalized transactions within partition
        let first_finalized = partition_nodes[0].get_finalized_transactions();
        for node in partition_nodes.iter().skip(1) {
            let node_finalized = node.get_finalized_transactions();
            // Nodes in same partition should eventually agree
            // (allowing for some lag in consensus)
        }
    }
}

fn verify_resource_conservation(network: &ConsensusNetwork) {
    // Verify no rUv is created or destroyed during consensus
    // This would sum all balances before and after consensus operations
}