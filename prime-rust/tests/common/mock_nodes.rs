//! Mock nodes for testing P2P networking and distributed behavior

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use libp2p::{PeerId, Multiaddr};
use async_trait::async_trait;

/// Mock P2P node for testing
pub struct MockNode {
    pub peer_id: PeerId,
    pub address: Multiaddr,
    pub state: Arc<RwLock<NodeState>>,
    pub message_handler: Arc<dyn MessageHandler>,
    pub network: Arc<MockNetwork>,
    inbox: Arc<Mutex<mpsc::Receiver<Message>>>,
    outbox: mpsc::Sender<Message>,
}

#[derive(Debug, Clone)]
pub struct NodeState {
    pub is_online: bool,
    pub connections: Vec<PeerId>,
    pub stored_data: HashMap<String, Vec<u8>>,
    pub model_version: u64,
    pub training_round: u64,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: PeerId,
    pub to: PeerId,
    pub payload: MessagePayload,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum MessagePayload {
    Ping,
    Pong,
    DhtPut { key: Vec<u8>, value: Vec<u8> },
    DhtGet { key: Vec<u8> },
    DhtResponse { key: Vec<u8>, value: Option<Vec<u8>> },
    GradientUpdate { round: u64, gradients: Vec<f32> },
    ModelSync { version: u64, parameters: Vec<f32> },
    ConsensusProposal { round: u64, value: Vec<u8> },
    ConsensusVote { round: u64, accept: bool },
    Custom(Vec<u8>),
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, msg: Message, node: &MockNode) -> Option<Message>;
}

/// Default message handler implementation
pub struct DefaultMessageHandler;

#[async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle_message(&self, msg: Message, node: &MockNode) -> Option<Message> {
        match msg.payload {
            MessagePayload::Ping => Some(Message {
                from: node.peer_id,
                to: msg.from,
                payload: MessagePayload::Pong,
                timestamp: std::time::Instant::now(),
            }),
            MessagePayload::DhtGet { key } => {
                let state = node.state.read().await;
                let value = state.stored_data.get(&hex::encode(&key)).cloned();
                Some(Message {
                    from: node.peer_id,
                    to: msg.from,
                    payload: MessagePayload::DhtResponse { key, value },
                    timestamp: std::time::Instant::now(),
                })
            }
            MessagePayload::DhtPut { key, value } => {
                let mut state = node.state.write().await;
                state.stored_data.insert(hex::encode(&key), value);
                None
            }
            _ => None,
        }
    }
}

/// Mock network for connecting nodes
pub struct MockNetwork {
    nodes: Arc<RwLock<HashMap<PeerId, mpsc::Sender<Message>>>>,
    topology: Arc<RwLock<NetworkTopology>>,
    conditions: Arc<RwLock<NetworkConditions>>,
}

#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub connections: HashMap<PeerId, Vec<PeerId>>,
}

#[derive(Debug, Clone)]
pub struct NetworkConditions {
    pub latency: Duration,
    pub packet_loss: f64,
    pub partitions: Vec<Vec<PeerId>>,
}

impl MockNode {
    pub async fn new(
        peer_id: PeerId,
        address: Multiaddr,
        network: Arc<MockNetwork>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        let node = Self {
            peer_id,
            address,
            state: Arc::new(RwLock::new(NodeState {
                is_online: true,
                connections: Vec::new(),
                stored_data: HashMap::new(),
                model_version: 0,
                training_round: 0,
            })),
            message_handler: Arc::new(DefaultMessageHandler),
            network: network.clone(),
            inbox: Arc::new(Mutex::new(rx)),
            outbox: tx.clone(),
        };

        network.register_node(peer_id, tx).await;
        node
    }

    pub async fn with_handler<H: MessageHandler + 'static>(mut self, handler: H) -> Self {
        self.message_handler = Arc::new(handler);
        self
    }

    pub async fn connect_to(&self, peer: &PeerId) -> Result<(), String> {
        let mut state = self.state.write().await;
        if !state.connections.contains(peer) {
            state.connections.push(*peer);
        }
        
        self.network.add_connection(self.peer_id, *peer).await;
        Ok(())
    }

    pub async fn disconnect_from(&self, peer: &PeerId) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.connections.retain(|p| p != peer);
        
        self.network.remove_connection(self.peer_id, *peer).await;
        Ok(())
    }

    pub async fn send_message(&self, to: PeerId, payload: MessagePayload) -> Result<(), String> {
        let msg = Message {
            from: self.peer_id,
            to,
            payload,
            timestamp: std::time::Instant::now(),
        };

        self.network.route_message(msg).await
    }

    pub async fn broadcast(&self, payload: MessagePayload) -> Result<(), String> {
        let state = self.state.read().await;
        for peer in &state.connections {
            self.send_message(*peer, payload.clone()).await?;
        }
        Ok(())
    }

    pub async fn receive_message(&self, timeout: Duration) -> Option<Message> {
        tokio::time::timeout(timeout, async {
            self.inbox.lock().await.recv().await
        })
        .await
        .ok()
        .flatten()
    }

    pub async fn process_messages(&self) {
        while let Some(msg) = self.receive_message(Duration::from_millis(10)).await {
            if let Some(response) = self.message_handler.handle_message(msg, self).await {
                let _ = self.network.route_message(response).await;
            }
        }
    }

    pub async fn go_offline(&self) {
        let mut state = self.state.write().await;
        state.is_online = false;
        self.network.set_node_offline(self.peer_id).await;
    }

    pub async fn go_online(&self) {
        let mut state = self.state.write().await;
        state.is_online = true;
        self.network.set_node_online(self.peer_id).await;
    }
}

impl MockNetwork {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            topology: Arc::new(RwLock::new(NetworkTopology {
                connections: HashMap::new(),
            })),
            conditions: Arc::new(RwLock::new(NetworkConditions {
                latency: Duration::from_millis(10),
                packet_loss: 0.0,
                partitions: Vec::new(),
            })),
        }
    }

    async fn register_node(&self, peer_id: PeerId, sender: mpsc::Sender<Message>) {
        self.nodes.write().await.insert(peer_id, sender);
    }

    async fn add_connection(&self, from: PeerId, to: PeerId) {
        let mut topology = self.topology.write().await;
        topology.connections.entry(from).or_default().push(to);
        topology.connections.entry(to).or_default().push(from);
    }

    async fn remove_connection(&self, from: PeerId, to: PeerId) {
        let mut topology = self.topology.write().await;
        if let Some(connections) = topology.connections.get_mut(&from) {
            connections.retain(|p| p != &to);
        }
        if let Some(connections) = topology.connections.get_mut(&to) {
            connections.retain(|p| p != &from);
        }
    }

    async fn route_message(&self, msg: Message) -> Result<(), String> {
        let conditions = self.conditions.read().await;
        
        // Check if nodes are in different partitions
        for partition in &conditions.partitions {
            let from_in = partition.contains(&msg.from);
            let to_in = partition.contains(&msg.to);
            if from_in != to_in {
                return Err("Network partitioned".to_string());
            }
        }

        // Simulate packet loss
        if rand::random::<f64>() < conditions.packet_loss {
            return Ok(()); // Message lost
        }

        // Simulate latency
        let latency = conditions.latency;
        drop(conditions);
        
        tokio::time::sleep(latency).await;

        // Deliver message
        let nodes = self.nodes.read().await;
        if let Some(sender) = nodes.get(&msg.to) {
            let _ = sender.send(msg).await;
        }

        Ok(())
    }

    async fn set_node_offline(&self, peer_id: PeerId) {
        self.nodes.write().await.remove(&peer_id);
    }

    async fn set_node_online(&self, peer_id: PeerId) {
        // Node needs to re-register when coming back online
    }

    pub async fn set_conditions(&self, conditions: NetworkConditions) {
        *self.conditions.write().await = conditions;
    }

    pub async fn partition_network(&self, partitions: Vec<Vec<PeerId>>) {
        self.conditions.write().await.partitions = partitions;
    }

    pub async fn heal_partition(&self) {
        self.conditions.write().await.partitions.clear();
    }
}

/// Test scenario builder
pub struct ScenarioBuilder {
    network: Arc<MockNetwork>,
    nodes: Vec<MockNode>,
}

impl ScenarioBuilder {
    pub fn new() -> Self {
        Self {
            network: Arc::new(MockNetwork::new()),
            nodes: Vec::new(),
        }
    }

    pub async fn add_node(mut self) -> Self {
        let peer_id = PeerId::random();
        let address = format!("/ip4/127.0.0.1/tcp/{}", 8000 + self.nodes.len())
            .parse()
            .unwrap();
        
        let node = MockNode::new(peer_id, address, self.network.clone()).await;
        self.nodes.push(node);
        self
    }

    pub async fn add_nodes(mut self, count: usize) -> Self {
        for _ in 0..count {
            self = self.add_node().await;
        }
        self
    }

    pub async fn connect_all(self) -> Self {
        for i in 0..self.nodes.len() {
            for j in i + 1..self.nodes.len() {
                let _ = self.nodes[i].connect_to(&self.nodes[j].peer_id).await;
            }
        }
        self
    }

    pub async fn with_conditions(self, conditions: NetworkConditions) -> Self {
        self.network.set_conditions(conditions).await;
        self
    }

    pub fn build(self) -> (Arc<MockNetwork>, Vec<MockNode>) {
        (self.network, self.nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_node_creation() {
        let network = Arc::new(MockNetwork::new());
        let peer_id = PeerId::random();
        let address = "/ip4/127.0.0.1/tcp/8000".parse().unwrap();
        
        let node = MockNode::new(peer_id, address, network).await;
        let state = node.state.read().await;
        
        assert!(state.is_online);
        assert!(state.connections.is_empty());
    }

    #[tokio::test]
    async fn test_node_messaging() {
        let (network, nodes) = ScenarioBuilder::new()
            .add_nodes(2).await
            .connect_all().await
            .build();

        let msg_sent = nodes[0].send_message(
            nodes[1].peer_id,
            MessagePayload::Ping,
        ).await;
        
        assert!(msg_sent.is_ok());

        // Process messages
        tokio::spawn({
            let node = nodes[1].clone();
            async move {
                node.process_messages().await;
            }
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_network_partition() {
        let (network, nodes) = ScenarioBuilder::new()
            .add_nodes(4).await
            .connect_all().await
            .build();

        // Partition network into two groups
        network.partition_network(vec![
            vec![nodes[0].peer_id, nodes[1].peer_id],
            vec![nodes[2].peer_id, nodes[3].peer_id],
        ]).await;

        // Messages within partition should work
        let result1 = nodes[0].send_message(nodes[1].peer_id, MessagePayload::Ping).await;
        assert!(result1.is_ok());

        // Messages across partition should fail
        let result2 = nodes[0].send_message(nodes[2].peer_id, MessagePayload::Ping).await;
        assert!(result2.is_err());

        // Heal partition
        network.heal_partition().await;

        // Now cross-partition messages should work
        let result3 = nodes[0].send_message(nodes[2].peer_id, MessagePayload::Ping).await;
        assert!(result3.is_ok());
    }
}