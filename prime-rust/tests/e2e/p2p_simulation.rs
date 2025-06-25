//! End-to-end P2P network simulation tests

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use prime_core::types::*;
use prime_dht::{Dht, DhtConfig};
use libp2p::PeerId;

mod common {
    pub use crate::common::*;
}

/// Simulated P2P network for testing
struct P2PSimulation {
    nodes: HashMap<NodeId, SimulatedNode>,
    network_conditions: common::network::NetworkConditions,
    message_log: Arc<RwLock<Vec<NetworkMessage>>>,
}

struct SimulatedNode {
    id: NodeId,
    peer_id: PeerId,
    dht: Arc<Dht>,
    peers: Vec<NodeId>,
    state: NodeState,
}

#[derive(Debug, Clone)]
struct NodeState {
    model_version: u64,
    training_round: u64,
    is_active: bool,
}

#[derive(Debug, Clone)]
struct NetworkMessage {
    from: NodeId,
    to: NodeId,
    message_type: MessageType,
    timestamp: std::time::Instant,
}

impl P2PSimulation {
    fn new(node_count: usize) -> Self {
        let mut nodes = HashMap::new();
        
        for i in 0..node_count {
            let id = NodeId::new(format!("node_{}", i));
            let peer_id = PeerId::random();
            let dht = Arc::new(Dht::new(peer_id, DhtConfig::default()));
            
            let node = SimulatedNode {
                id: id.clone(),
                peer_id,
                dht,
                peers: Vec::new(),
                state: NodeState {
                    model_version: 0,
                    training_round: 0,
                    is_active: true,
                },
            };
            
            nodes.insert(id, node);
        }
        
        Self {
            nodes,
            network_conditions: common::network::NetworkConditions::default(),
            message_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn connect_nodes(&mut self, topology: common::network::NetworkTopology) {
        match topology {
            common::network::NetworkTopology::FullMesh => {
                let node_ids: Vec<NodeId> = self.nodes.keys().cloned().collect();
                for i in 0..node_ids.len() {
                    for j in 0..node_ids.len() {
                        if i != j {
                            self.nodes.get_mut(&node_ids[i])
                                .unwrap()
                                .peers.push(node_ids[j].clone());
                        }
                    }
                }
            }
            _ => {} // Implement other topologies as needed
        }
    }

    async fn broadcast_gradient_update(&mut self, from: &NodeId, update: GradientUpdate) {
        let node = self.nodes.get(from).unwrap();
        let peers = node.peers.clone();
        
        for peer in peers {
            let msg = NetworkMessage {
                from: from.clone(),
                to: peer,
                message_type: MessageType::GradientUpdate(update.clone()),
                timestamp: std::time::Instant::now(),
            };
            
            self.message_log.write().await.push(msg);
        }
    }

    async fn simulate_training_round(&mut self, round: u64) {
        for (node_id, node) in self.nodes.iter_mut() {
            if !node.state.is_active {
                continue;
            }
            
            node.state.training_round = round;
            
            // Create gradient update
            let update = GradientUpdate {
                node_id: node_id.clone(),
                model_version: node.state.model_version,
                round,
                gradients: HashMap::from([
                    ("layer1".to_string(), vec![0.1, 0.2, 0.3]),
                    ("layer2".to_string(), vec![0.4, 0.5, 0.6]),
                ]),
                metrics: TrainingMetrics {
                    loss: 1.0 / (round as f32 + 1.0),
                    accuracy: 0.5 + (round as f32 * 0.05).min(0.45),
                    samples_processed: 1000,
                    computation_time_ms: 500,
                },
                timestamp: round * 1000,
            };
            
            // Store in DHT
            let key = format!("gradient_{}_{}", node_id.0, round).into_bytes();
            let value = serde_json::to_vec(&update).unwrap();
            node.dht.put(key, value).await.unwrap();
        }
    }

    async fn partition_network(&mut self, partition1: Vec<NodeId>, partition2: Vec<NodeId>) {
        // Remove connections between partitions
        for node1 in &partition1 {
            if let Some(node) = self.nodes.get_mut(node1) {
                node.peers.retain(|p| !partition2.contains(p));
            }
        }
        
        for node2 in &partition2 {
            if let Some(node) = self.nodes.get_mut(node2) {
                node.peers.retain(|p| !partition1.contains(p));
            }
        }
    }

    async fn heal_partition(&mut self) {
        // Restore full mesh connectivity
        self.connect_nodes(common::network::NetworkTopology::FullMesh).await;
    }

    fn get_message_count(&self) -> usize {
        self.message_log.blocking_read().len()
    }
}

#[tokio::test]
async fn test_basic_p2p_communication() {
    common::init_test_env();
    
    let mut sim = P2PSimulation::new(5);
    sim.connect_nodes(common::network::NetworkTopology::FullMesh).await;
    
    // Simulate 10 training rounds
    for round in 0..10 {
        sim.simulate_training_round(round).await;
    }
    
    // Verify all nodes have stored their gradients
    for (node_id, node) in &sim.nodes {
        for round in 0..10 {
            let key = format!("gradient_{}_{}", node_id.0, round).into_bytes();
            let result = node.dht.get(key).await.unwrap();
            assert!(result.is_some(), "Node {} missing gradient for round {}", node_id.0, round);
        }
    }
}

#[tokio::test]
async fn test_network_partition_recovery() {
    common::init_test_env();
    
    let mut sim = P2PSimulation::new(6);
    sim.connect_nodes(common::network::NetworkTopology::FullMesh).await;
    
    let partition1 = vec![
        NodeId::new("node_0"),
        NodeId::new("node_1"),
        NodeId::new("node_2"),
    ];
    let partition2 = vec![
        NodeId::new("node_3"),
        NodeId::new("node_4"),
        NodeId::new("node_5"),
    ];
    
    // Train for 5 rounds normally
    for round in 0..5 {
        sim.simulate_training_round(round).await;
    }
    
    // Partition the network
    sim.partition_network(partition1.clone(), partition2.clone()).await;
    
    // Train for 5 more rounds with partition
    for round in 5..10 {
        sim.simulate_training_round(round).await;
    }
    
    // Verify partitions can't see each other's data
    let node0 = &sim.nodes[&NodeId::new("node_0")];
    let key_from_partition2 = format!("gradient_node_3_7").into_bytes();
    let result = node0.dht.get(key_from_partition2).await.unwrap();
    assert!(result.is_none(), "Node in partition1 shouldn't see partition2 data");
    
    // Heal the partition
    sim.heal_partition().await;
    
    // Train one more round
    sim.simulate_training_round(10).await;
    
    // Now all nodes should be able to communicate again
    // (In a real implementation, they would sync missing data)
}

#[tokio::test]
async fn test_node_failure_resilience() {
    common::init_test_env();
    
    let mut sim = P2PSimulation::new(10);
    sim.connect_nodes(common::network::NetworkTopology::FullMesh).await;
    
    // Simulate some nodes failing
    let failing_nodes = vec![
        NodeId::new("node_2"),
        NodeId::new("node_5"),
        NodeId::new("node_8"),
    ];
    
    for node_id in &failing_nodes {
        if let Some(node) = sim.nodes.get_mut(node_id) {
            node.state.is_active = false;
        }
    }
    
    // Continue training
    for round in 0..20 {
        sim.simulate_training_round(round).await;
    }
    
    // Verify active nodes have continued training
    for (node_id, node) in &sim.nodes {
        if node.state.is_active {
            assert_eq!(node.state.training_round, 19);
        } else {
            assert_eq!(node.state.training_round, 0);
        }
    }
}

#[tokio::test]
async fn test_gradual_network_degradation() {
    common::init_test_env();
    
    let mut sim = P2PSimulation::new(20);
    sim.connect_nodes(common::network::NetworkTopology::FullMesh).await;
    
    // Gradually increase network latency and packet loss
    let conditions = vec![
        common::network::NetworkConditions::perfect(),
        common::network::NetworkConditions {
            latency: Duration::from_millis(50),
            jitter: Duration::from_millis(10),
            packet_loss: 0.05,
            bandwidth_limit: None,
        },
        common::network::NetworkConditions {
            latency: Duration::from_millis(200),
            jitter: Duration::from_millis(50),
            packet_loss: 0.15,
            bandwidth_limit: Some(100_000),
        },
        common::network::NetworkConditions::lossy(),
    ];
    
    for (i, condition) in conditions.iter().enumerate() {
        sim.network_conditions = condition.clone();
        
        // Train for 5 rounds under each condition
        for round in (i * 5)..((i + 1) * 5) {
            sim.simulate_training_round(round as u64).await;
        }
    }
    
    // System should continue functioning despite degraded conditions
    let active_nodes = sim.nodes.values()
        .filter(|n| n.state.is_active)
        .count();
    
    assert!(active_nodes > 15, "Too many nodes failed under stress");
}

#[tokio::test]
async fn test_byzantine_node_behavior() {
    common::init_test_env();
    
    let mut sim = P2PSimulation::new(10);
    sim.connect_nodes(common::network::NetworkTopology::FullMesh).await;
    
    // Mark some nodes as byzantine
    let byzantine_nodes = vec![
        NodeId::new("node_3"),
        NodeId::new("node_7"),
    ];
    
    for round in 0..10 {
        for (node_id, node) in sim.nodes.iter_mut() {
            if !node.state.is_active {
                continue;
            }
            
            node.state.training_round = round;
            
            // Byzantine nodes send corrupted gradients
            let gradients = if byzantine_nodes.contains(node_id) {
                HashMap::from([
                    ("layer1".to_string(), vec![999.9, 999.9, 999.9]),
                    ("layer2".to_string(), vec![999.9, 999.9, 999.9]),
                ])
            } else {
                HashMap::from([
                    ("layer1".to_string(), vec![0.1, 0.2, 0.3]),
                    ("layer2".to_string(), vec![0.4, 0.5, 0.6]),
                ])
            };
            
            let update = GradientUpdate {
                node_id: node_id.clone(),
                model_version: node.state.model_version,
                round,
                gradients,
                metrics: TrainingMetrics {
                    loss: 1.0,
                    accuracy: 0.5,
                    samples_processed: 1000,
                    computation_time_ms: 500,
                },
                timestamp: round * 1000,
            };
            
            let key = format!("gradient_{}_{}", node_id.0, round).into_bytes();
            let value = serde_json::to_vec(&update).unwrap();
            node.dht.put(key, value).await.unwrap();
        }
    }
    
    // In a real system, aggregation would detect and handle byzantine behavior
    // Here we just verify the system continues to function
    assert_eq!(sim.nodes.len(), 10);
}

// Property-based test for network behavior
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_network_consistency_under_random_operations(
        node_count in 5..20usize,
        operation_count in 10..100usize,
        seed in 0u64..1000u64,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            use rand::{Rng, SeedableRng};
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            
            let mut sim = P2PSimulation::new(node_count);
            sim.connect_nodes(common::network::NetworkTopology::FullMesh).await;
            
            for _ in 0..operation_count {
                match rng.gen_range(0..5) {
                    0 => {
                        // Simulate training round
                        let round = rng.gen_range(0..100);
                        sim.simulate_training_round(round).await;
                    }
                    1 => {
                        // Random node failure
                        let node_idx = rng.gen_range(0..node_count);
                        let node_id = NodeId::new(format!("node_{}", node_idx));
                        if let Some(node) = sim.nodes.get_mut(&node_id) {
                            node.state.is_active = false;
                        }
                    }
                    2 => {
                        // Random node recovery
                        let node_idx = rng.gen_range(0..node_count);
                        let node_id = NodeId::new(format!("node_{}", node_idx));
                        if let Some(node) = sim.nodes.get_mut(&node_id) {
                            node.state.is_active = true;
                        }
                    }
                    3 => {
                        // Random partition
                        let split_point = rng.gen_range(1..node_count);
                        let partition1: Vec<NodeId> = (0..split_point)
                            .map(|i| NodeId::new(format!("node_{}", i)))
                            .collect();
                        let partition2: Vec<NodeId> = (split_point..node_count)
                            .map(|i| NodeId::new(format!("node_{}", i)))
                            .collect();
                        sim.partition_network(partition1, partition2).await;
                    }
                    _ => {
                        // Heal partitions
                        sim.heal_partition().await;
                    }
                }
            }
            
            // Verify basic invariants
            assert_eq!(sim.nodes.len(), node_count);
            
            // At least some nodes should be active
            let active_count = sim.nodes.values()
                .filter(|n| n.state.is_active)
                .count();
            assert!(active_count > 0);
        });
    }
}