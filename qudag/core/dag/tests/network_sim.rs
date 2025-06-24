use blake3::Hash;
use qudag_dag::{Graph, Node, NodeState, QrAvalanche};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

const NETWORK_SIZE: usize = 10;
const LATENCY_MIN: u64 = 10; // 10ms minimum latency
const LATENCY_MAX: u64 = 100; // 100ms maximum latency
const PACKET_LOSS_RATE: f64 = 0.05; // 5% packet loss

struct NetworkSimulator {
    nodes: Vec<SimNode>,
    graph: Arc<Graph>,
}

struct SimNode {
    id: Hash,
    consensus: Arc<QrAvalanche>,
    events_rx: mpsc::Receiver<ConsensusEvent>,
    peers: Vec<Hash>,
}

impl NetworkSimulator {
    fn new(size: usize) -> Self {
        let graph = Arc::new(Graph::new());
        let mut nodes = Vec::with_capacity(size);
        let mut peer_map = HashMap::new();

        // Create nodes
        for i in 0..size {
            let id = blake3::hash(&[i as u8]);
            let (consensus, events_rx) = QrAvalanche::new(graph.clone());
            let consensus = Arc::new(consensus);

            nodes.push(SimNode {
                id,
                consensus,
                events_rx,
                peers: Vec::new(),
            });

            peer_map.insert(id, i);
        }

        // Assign peers (fully connected topology for simplicity)
        for i in 0..size {
            let mut peers = Vec::new();
            for j in 0..size {
                if i != j {
                    peers.push(blake3::hash(&[j as u8]));
                }
            }
            nodes[i].peers = peers;
        }

        Self { nodes, graph }
    }

    async fn simulate_consensus(&mut self, test_node: Node) {
        let node_hash = *test_node.hash();

        // Add node to graph
        self.graph.add_node(test_node).unwrap();
        self.graph
            .update_node_state(&node_hash, NodeState::Verified)
            .unwrap();

        // Start consensus on all nodes
        for node in &self.nodes {
            node.consensus.process_node(node_hash).await.unwrap();
        }

        // Simulate network communication
        let mut vote_tasks = Vec::new();

        for node in &self.nodes {
            let consensus = node.consensus.clone();
            let node_id = node.id;
            let peers = node.peers.clone();

            let task = tokio::spawn(async move {
                for peer_id in peers {
                    if rand::random::<f64>() > PACKET_LOSS_RATE {
                        // Simulate network latency
                        let latency =
                            rand::random::<u64>() % (LATENCY_MAX - LATENCY_MIN) + LATENCY_MIN;
                        sleep(Duration::from_millis(latency)).await;

                        // Random vote (biased towards acceptance)
                        let accept = rand::random::<f64>() < 0.8;
                        let _ = consensus.record_vote(node_hash, peer_id, accept).await;
                    }
                }
            });

            vote_tasks.push(task);
        }

        // Wait for all votes to complete
        for task in vote_tasks {
            let _ = task.await;
        }

        // Allow time for consensus to finalize
        sleep(Duration::from_secs(1)).await;
    }

    async fn collect_results(&mut self) -> HashMap<Hash, NodeState> {
        let mut results = HashMap::new();

        for node in &self.nodes {
            if let Some(node_state) = node
                .events_rx
                .try_recv()
                .ok()
                .and_then(|event| match event {
                    ConsensusEvent::NodeFinalized(hash) => Some((hash, NodeState::Final)),
                    ConsensusEvent::NodeRejected(hash) => Some((hash, NodeState::Rejected)),
                    _ => None,
                })
            {
                results.insert(node.id, node_state.1);
            }
        }

        results
    }
}

#[tokio::test]
async fn test_network_consensus_basic() {
    let mut sim = NetworkSimulator::new(NETWORK_SIZE);

    // Create test node
    let test_node = Node::new(vec![1, 2, 3], vec![]);
    let node_hash = *test_node.hash();

    // Run simulation
    sim.simulate_consensus(test_node).await;

    // Check results
    let results = sim.collect_results().await;

    // Verify consensus was reached
    let finalized = results
        .values()
        .filter(|&&state| state == NodeState::Final)
        .count();

    assert!(
        finalized >= (NETWORK_SIZE * 2) / 3,
        "Expected at least 2/3 nodes to reach finality"
    );

    // Verify final state in graph
    let node = sim.graph.get_node(&node_hash).unwrap();
    assert_eq!(node.state(), NodeState::Final);
}

#[tokio::test]
async fn test_network_consensus_with_failures() {
    let mut sim = NetworkSimulator::new(NETWORK_SIZE);

    // Create test node with invalid data
    let test_node = Node::new(vec![0; 1024], vec![]); // Large invalid payload
    let node_hash = *test_node.hash();

    // Run simulation
    sim.simulate_consensus(test_node).await;

    // Check results
    let results = sim.collect_results().await;

    // Verify rejection
    let rejected = results
        .values()
        .filter(|&&state| state == NodeState::Rejected)
        .count();

    assert!(
        rejected >= (NETWORK_SIZE * 2) / 3,
        "Expected at least 2/3 nodes to reject invalid node"
    );

    // Verify rejected state in graph
    let node = sim.graph.get_node(&node_hash).unwrap();
    assert_eq!(node.state(), NodeState::Rejected);
}

#[tokio::test]
async fn test_network_consensus_partition() {
    let mut sim = NetworkSimulator::new(NETWORK_SIZE);

    // Create network partition by removing half of each node's peers
    for node in &mut sim.nodes {
        node.peers.truncate(node.peers.len() / 2);
    }

    // Create test node
    let test_node = Node::new(vec![1, 2, 3], vec![]);
    let node_hash = *test_node.hash();

    // Run simulation
    sim.simulate_consensus(test_node).await;

    // Check results
    let results = sim.collect_results().await;

    // Verify consensus cannot be reached with network partition
    let finalized = results
        .values()
        .filter(|&&state| state == NodeState::Final)
        .count();

    assert!(
        finalized < (NETWORK_SIZE * 2) / 3,
        "Expected consensus to fail under network partition"
    );

    // Verify node remains in Verified state
    let node = sim.graph.get_node(&node_hash).unwrap();
    assert_eq!(node.state(), NodeState::Verified);
}
