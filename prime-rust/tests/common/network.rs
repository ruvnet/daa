//! Network simulation and testing utilities

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use libp2p::{Multiaddr, PeerId};
use std::time::Duration;

/// Simulated network conditions
#[derive(Debug, Clone)]
pub struct NetworkConditions {
    pub latency: Duration,
    pub jitter: Duration,
    pub packet_loss: f64,
    pub bandwidth_limit: Option<usize>, // bytes per second
}

impl Default for NetworkConditions {
    fn default() -> Self {
        Self {
            latency: Duration::from_millis(10),
            jitter: Duration::from_millis(2),
            packet_loss: 0.0,
            bandwidth_limit: None,
        }
    }
}

impl NetworkConditions {
    pub fn perfect() -> Self {
        Self {
            latency: Duration::ZERO,
            jitter: Duration::ZERO,
            packet_loss: 0.0,
            bandwidth_limit: None,
        }
    }

    pub fn lossy() -> Self {
        Self {
            latency: Duration::from_millis(50),
            jitter: Duration::from_millis(20),
            packet_loss: 0.1, // 10% loss
            bandwidth_limit: Some(1_000_000), // 1MB/s
        }
    }

    pub fn slow() -> Self {
        Self {
            latency: Duration::from_millis(200),
            jitter: Duration::from_millis(50),
            packet_loss: 0.01,
            bandwidth_limit: Some(100_000), // 100KB/s
        }
    }
}

/// Network topology for testing
#[derive(Debug, Clone)]
pub enum NetworkTopology {
    FullMesh,
    Star { hub: PeerId },
    Ring,
    Tree { root: PeerId, fanout: usize },
    Random { connectivity: f64 },
    Partitioned { partitions: Vec<Vec<PeerId>> },
}

/// Simulated network for testing
pub struct SimulatedNetwork {
    nodes: HashMap<PeerId, NetworkNode>,
    topology: NetworkTopology,
    conditions: Arc<RwLock<NetworkConditions>>,
}

struct NetworkNode {
    peer_id: PeerId,
    address: Multiaddr,
    tx: mpsc::Sender<NetworkMessage>,
    rx: mpsc::Receiver<NetworkMessage>,
}

#[derive(Debug, Clone)]
struct NetworkMessage {
    from: PeerId,
    to: PeerId,
    payload: Vec<u8>,
    timestamp: std::time::Instant,
}

impl SimulatedNetwork {
    pub fn new(topology: NetworkTopology, conditions: NetworkConditions) -> Self {
        Self {
            nodes: HashMap::new(),
            topology,
            conditions: Arc::new(RwLock::new(conditions)),
        }
    }

    pub async fn add_node(&mut self, peer_id: PeerId, address: Multiaddr) {
        let (tx, rx) = mpsc::channel(1000);
        let node = NetworkNode {
            peer_id,
            address,
            tx,
            rx,
        };
        self.nodes.insert(peer_id, node);
    }

    pub async fn send_message(&self, from: PeerId, to: PeerId, payload: Vec<u8>) -> Result<(), String> {
        if !self.is_connected(&from, &to).await {
            return Err("Nodes not connected".to_string());
        }

        let conditions = self.conditions.read().await;
        
        // Simulate packet loss
        if rand::random::<f64>() < conditions.packet_loss {
            return Ok(()); // Message lost
        }

        // Simulate latency and jitter
        let delay = conditions.latency + 
            Duration::from_millis((rand::random::<f64>() * conditions.jitter.as_millis() as f64) as u64);
        
        tokio::time::sleep(delay).await;

        // Deliver message
        if let Some(node) = self.nodes.get(&to) {
            let msg = NetworkMessage {
                from,
                to,
                payload,
                timestamp: std::time::Instant::now(),
            };
            let _ = node.tx.send(msg).await;
        }

        Ok(())
    }

    async fn is_connected(&self, from: &PeerId, to: &PeerId) -> bool {
        match &self.topology {
            NetworkTopology::FullMesh => true,
            NetworkTopology::Star { hub } => from == hub || to == hub,
            NetworkTopology::Ring => {
                // Simple ring connectivity check
                true
            }
            NetworkTopology::Tree { root, fanout } => {
                // Tree connectivity logic
                true
            }
            NetworkTopology::Random { connectivity } => {
                rand::random::<f64>() < *connectivity
            }
            NetworkTopology::Partitioned { partitions } => {
                // Check if nodes are in same partition
                partitions.iter().any(|p| p.contains(from) && p.contains(to))
            }
        }
    }

    pub async fn simulate_partition(&mut self, duration: Duration) {
        let mut conditions = self.conditions.write().await;
        let original = conditions.clone();
        conditions.packet_loss = 1.0; // 100% loss
        drop(conditions);

        tokio::time::sleep(duration).await;

        let mut conditions = self.conditions.write().await;
        *conditions = original;
    }

    pub async fn get_node_addresses(&self) -> HashMap<PeerId, Multiaddr> {
        self.nodes
            .iter()
            .map(|(id, node)| (*id, node.address.clone()))
            .collect()
    }
}

/// Network event recorder for testing
pub struct NetworkEventRecorder {
    events: Arc<RwLock<Vec<NetworkEvent>>>,
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    NodeJoined { peer_id: PeerId, timestamp: std::time::Instant },
    NodeLeft { peer_id: PeerId, timestamp: std::time::Instant },
    MessageSent { from: PeerId, to: PeerId, size: usize, timestamp: std::time::Instant },
    MessageReceived { from: PeerId, to: PeerId, size: usize, timestamp: std::time::Instant },
    ConnectionEstablished { peer1: PeerId, peer2: PeerId, timestamp: std::time::Instant },
    ConnectionLost { peer1: PeerId, peer2: PeerId, timestamp: std::time::Instant },
}

impl NetworkEventRecorder {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record(&self, event: NetworkEvent) {
        self.events.write().await.push(event);
    }

    pub async fn get_events(&self) -> Vec<NetworkEvent> {
        self.events.read().await.clone()
    }

    pub async fn clear(&self) {
        self.events.write().await.clear();
    }

    pub async fn count_events<F>(&self, predicate: F) -> usize
    where
        F: Fn(&NetworkEvent) -> bool,
    {
        self.events.read().await.iter().filter(|e| predicate(e)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_conditions() {
        let perfect = NetworkConditions::perfect();
        assert_eq!(perfect.latency, Duration::ZERO);
        assert_eq!(perfect.packet_loss, 0.0);

        let lossy = NetworkConditions::lossy();
        assert!(lossy.packet_loss > 0.0);
        assert!(lossy.latency > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_simulated_network() {
        let mut network = SimulatedNetwork::new(
            NetworkTopology::FullMesh,
            NetworkConditions::perfect(),
        );

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        
        network.add_node(peer1, "/ip4/127.0.0.1/tcp/8001".parse().unwrap()).await;
        network.add_node(peer2, "/ip4/127.0.0.1/tcp/8002".parse().unwrap()).await;

        let result = network.send_message(peer1, peer2, vec![1, 2, 3]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_network_event_recorder() {
        let recorder = NetworkEventRecorder::new();
        
        let peer1 = PeerId::random();
        recorder.record(NetworkEvent::NodeJoined {
            peer_id: peer1,
            timestamp: std::time::Instant::now(),
        }).await;

        let events = recorder.get_events().await;
        assert_eq!(events.len(), 1);
        
        let join_count = recorder.count_events(|e| {
            matches!(e, NetworkEvent::NodeJoined { .. })
        }).await;
        assert_eq!(join_count, 1);
    }
}