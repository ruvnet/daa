//! Mock implementations for testing CLI commands
//!
//! This module provides configurable mock implementations of various components
//! used by the CLI, allowing comprehensive testing without requiring actual
//! network connections or node instances.

use crate::rpc::{
    DagStats, MemoryStats, NetworkStats, NetworkTestResult, NodeStatus, PeerInfo, RpcError,
    RpcRequest, RpcResponse, WalletInfo,
};
use anyhow::{anyhow, Result};
use qudag_network::{NetworkAddress, PeerId};
use qudag_protocol::{Node, NodeConfig, ProtocolState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Mock behavior configuration
#[derive(Debug, Clone)]
pub struct MockBehavior {
    /// Should operations succeed
    pub should_succeed: bool,
    /// Latency to simulate (milliseconds)
    pub latency_ms: u64,
    /// Error message if should_succeed is false
    pub error_message: String,
    /// Custom response data
    pub custom_response: Option<serde_json::Value>,
}

impl Default for MockBehavior {
    fn default() -> Self {
        Self {
            should_succeed: true,
            latency_ms: 10,
            error_message: "Mock error".to_string(),
            custom_response: None,
        }
    }
}

/// Mock node for testing
pub struct MockNode {
    /// Node ID
    pub id: String,
    /// Current state
    pub state: Arc<RwLock<NodeState>>,
    /// Configured behaviors
    pub behaviors: Arc<RwLock<HashMap<String, MockBehavior>>>,
    /// Peers
    pub peers: Arc<RwLock<Vec<MockPeer>>>,
    /// Network stats
    pub network_stats: Arc<RwLock<NetworkStats>>,
    /// DAG stats
    pub dag_stats: Arc<RwLock<DagStats>>,
    /// Memory stats
    pub memory_stats: Arc<RwLock<MemoryStats>>,
    /// Start time
    pub start_time: SystemTime,
}

/// Node state
#[derive(Debug, Clone)]
pub enum NodeState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

/// Mock peer representation
#[derive(Debug, Clone)]
pub struct MockPeer {
    pub id: String,
    pub address: String,
    pub connected_at: SystemTime,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub last_seen: SystemTime,
}

impl MockNode {
    /// Create new mock node
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: Arc::new(RwLock::new(NodeState::Stopped)),
            behaviors: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(Vec::new())),
            network_stats: Arc::new(RwLock::new(NetworkStats {
                total_connections: 0,
                active_connections: 0,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                average_latency: 0.0,
                uptime: 0,
            })),
            dag_stats: Arc::new(RwLock::new(DagStats {
                vertex_count: 0,
                edge_count: 0,
                tip_count: 0,
                finalized_height: 0,
                pending_transactions: 0,
            })),
            memory_stats: Arc::new(RwLock::new(MemoryStats {
                total_allocated: 0,
                current_usage: 0,
                peak_usage: 0,
            })),
            start_time: SystemTime::now(),
        }
    }

    /// Set behavior for a specific method
    pub async fn set_behavior(&self, method: &str, behavior: MockBehavior) {
        self.behaviors
            .write()
            .await
            .insert(method.to_string(), behavior);
    }

    /// Get current status
    pub async fn get_status(&self) -> NodeStatus {
        let state = self.state.read().await;
        let peers = self.peers.read().await;
        let network_stats = self.network_stats.read().await;
        let dag_stats = self.dag_stats.read().await;
        let memory_stats = self.memory_stats.read().await;

        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let peer_info: Vec<PeerInfo> = peers
            .iter()
            .map(|p| PeerInfo {
                id: p.id.clone(),
                address: p.address.clone(),
                connected_duration: SystemTime::now()
                    .duration_since(p.connected_at)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs(),
                messages_sent: p.messages_sent,
                messages_received: p.messages_received,
                last_seen: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                status: "Connected".to_string(),
                latency: None,
            })
            .collect();

        NodeStatus {
            node_id: self.id.clone(),
            state: format!("{:?}", *state),
            uptime,
            peers: peer_info,
            network_stats: network_stats.clone(),
            dag_stats: dag_stats.clone(),
            memory_usage: memory_stats.clone(),
        }
    }

    /// Start the node
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.write().await;
        match &*state {
            NodeState::Stopped => {
                *state = NodeState::Starting;
                // Simulate startup delay
                tokio::time::sleep(Duration::from_millis(100)).await;
                *state = NodeState::Running;

                // Initialize some mock data
                let mut dag_stats = self.dag_stats.write().await;
                dag_stats.vertex_count = 10;
                dag_stats.edge_count = 15;
                dag_stats.tip_count = 3;

                Ok(())
            }
            _ => Err(anyhow!("Node is already running or in transition")),
        }
    }

    /// Stop the node
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().await;
        match &*state {
            NodeState::Running => {
                *state = NodeState::Stopping;
                // Simulate shutdown delay
                tokio::time::sleep(Duration::from_millis(50)).await;
                *state = NodeState::Stopped;

                // Clear peers
                self.peers.write().await.clear();

                Ok(())
            }
            _ => Err(anyhow!("Node is not running")),
        }
    }

    /// Add a peer
    pub async fn add_peer(&self, address: String) -> Result<()> {
        let mut peers = self.peers.write().await;

        // Check if peer already exists
        if peers.iter().any(|p| p.address == address) {
            return Err(anyhow!("Peer already connected"));
        }

        let peer = MockPeer {
            id: format!("peer-{}", uuid::Uuid::new_v4()),
            address,
            connected_at: SystemTime::now(),
            messages_sent: 0,
            messages_received: 0,
            last_seen: SystemTime::now(),
        };

        peers.push(peer);

        // Update network stats
        let mut stats = self.network_stats.write().await;
        stats.total_connections += 1;
        stats.active_connections += 1;

        Ok(())
    }

    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &str) -> Result<()> {
        let mut peers = self.peers.write().await;

        let initial_count = peers.len();
        peers.retain(|p| p.id != peer_id);

        if peers.len() == initial_count {
            return Err(anyhow!("Peer not found"));
        }

        // Update network stats
        let mut stats = self.network_stats.write().await;
        stats.active_connections = stats.active_connections.saturating_sub(1);

        Ok(())
    }

    /// Simulate network activity
    pub async fn simulate_activity(&self) {
        let mut peers = self.peers.write().await;
        let mut stats = self.network_stats.write().await;

        for peer in peers.iter_mut() {
            // Simulate some messages
            peer.messages_sent += rand::random::<u64>() % 10;
            peer.messages_received += rand::random::<u64>() % 10;
            peer.last_seen = SystemTime::now();

            stats.messages_sent += peer.messages_sent;
            stats.messages_received += peer.messages_received;
        }

        // Update bytes
        stats.bytes_sent = stats.messages_sent * 256; // Assume 256 bytes per message
        stats.bytes_received = stats.messages_received * 256;

        // Update latency
        stats.average_latency = 20.0 + (rand::random::<f64>() * 10.0);

        // Update DAG stats
        let mut dag = self.dag_stats.write().await;
        dag.vertex_count += rand::random::<usize>() % 5;
        dag.edge_count += rand::random::<usize>() % 8;
        dag.pending_transactions = rand::random::<usize>() % 20;
    }
}

/// Mock peer manager for testing peer operations
pub struct MockPeerManager {
    /// Known peers
    pub peers: Arc<RwLock<HashMap<String, MockPeer>>>,
    /// Connection attempts
    pub connection_attempts: Arc<Mutex<Vec<ConnectionAttempt>>>,
    /// Configured behaviors
    pub behaviors: Arc<RwLock<HashMap<String, MockBehavior>>>,
}

#[derive(Debug, Clone)]
pub struct ConnectionAttempt {
    pub address: String,
    pub timestamp: SystemTime,
    pub success: bool,
    pub error: Option<String>,
}

impl MockPeerManager {
    /// Create new mock peer manager
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            connection_attempts: Arc::new(Mutex::new(Vec::new())),
            behaviors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a known peer
    pub async fn add_known_peer(&self, peer: MockPeer) {
        self.peers.write().await.insert(peer.id.clone(), peer);
    }

    /// Attempt to connect to a peer
    pub async fn connect_to_peer(&self, address: String) -> Result<String> {
        let behaviors = self.behaviors.read().await;
        let behavior = behaviors.get("connect").cloned().unwrap_or_default();

        // Simulate latency
        if behavior.latency_ms > 0 {
            tokio::time::sleep(Duration::from_millis(behavior.latency_ms)).await;
        }

        let attempt = ConnectionAttempt {
            address: address.clone(),
            timestamp: SystemTime::now(),
            success: behavior.should_succeed,
            error: if behavior.should_succeed {
                None
            } else {
                Some(behavior.error_message.clone())
            },
        };

        self.connection_attempts
            .lock()
            .unwrap()
            .push(attempt.clone());

        if behavior.should_succeed {
            let peer_id = format!("peer-{}", uuid::Uuid::new_v4());
            let peer = MockPeer {
                id: peer_id.clone(),
                address,
                connected_at: SystemTime::now(),
                messages_sent: 0,
                messages_received: 0,
                last_seen: SystemTime::now(),
            };

            self.peers.write().await.insert(peer_id.clone(), peer);
            Ok(peer_id)
        } else {
            Err(anyhow!(behavior.error_message))
        }
    }

    /// Get connection history
    pub fn get_connection_attempts(&self) -> Vec<ConnectionAttempt> {
        self.connection_attempts.lock().unwrap().clone()
    }
}

/// Mock network statistics collector
pub struct MockNetworkStats {
    /// Current stats
    pub stats: Arc<RwLock<NetworkStats>>,
    /// Historical data points
    pub history: Arc<Mutex<Vec<NetworkStatsSnapshot>>>,
    /// Update interval
    pub update_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct NetworkStatsSnapshot {
    pub timestamp: SystemTime,
    pub stats: NetworkStats,
}

impl MockNetworkStats {
    /// Create new mock network stats
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(NetworkStats {
                total_connections: 0,
                active_connections: 0,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                average_latency: 0.0,
                uptime: 0,
            })),
            history: Arc::new(Mutex::new(Vec::new())),
            update_interval: Duration::from_secs(1),
        }
    }

    /// Update statistics with simulated data
    pub async fn update(&self) {
        let mut stats = self.stats.write().await;

        // Simulate network activity
        stats.messages_sent += rand::random::<u64>() % 100;
        stats.messages_received += rand::random::<u64>() % 100;
        stats.bytes_sent += stats.messages_sent * 256;
        stats.bytes_received += stats.messages_received * 256;
        stats.average_latency = 15.0 + (rand::random::<f64>() * 20.0);

        // Store snapshot
        let snapshot = NetworkStatsSnapshot {
            timestamp: SystemTime::now(),
            stats: stats.clone(),
        };

        self.history.lock().unwrap().push(snapshot);
    }

    /// Get current statistics
    pub async fn get_current(&self) -> NetworkStats {
        self.stats.read().await.clone()
    }

    /// Get historical data
    pub fn get_history(&self) -> Vec<NetworkStatsSnapshot> {
        self.history.lock().unwrap().clone()
    }

    /// Reset statistics
    pub async fn reset(&self) {
        let mut stats = self.stats.write().await;
        *stats = NetworkStats {
            total_connections: stats.total_connections,
            active_connections: stats.active_connections,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            average_latency: 0.0,
            uptime: stats.uptime,
        };

        self.history.lock().unwrap().clear();
    }
}

/// Mock RPC client for testing
pub struct MockRpcClient {
    /// Mock node instance
    pub node: Arc<MockNode>,
    /// Response behaviors
    pub behaviors: Arc<RwLock<HashMap<String, MockBehavior>>>,
    /// Request history
    pub request_history: Arc<Mutex<Vec<RpcRequest>>>,
}

impl MockRpcClient {
    /// Create new mock RPC client
    pub fn new(node: Arc<MockNode>) -> Self {
        Self {
            node,
            behaviors: Arc::new(RwLock::new(HashMap::new())),
            request_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Set behavior for a specific RPC method
    pub async fn set_behavior(&self, method: &str, behavior: MockBehavior) {
        self.behaviors
            .write()
            .await
            .insert(method.to_string(), behavior);
    }

    /// Process RPC request
    pub async fn process_request(&self, request: RpcRequest) -> RpcResponse {
        // Store request in history
        self.request_history.lock().unwrap().push(request.clone());

        // Get behavior for this method
        let behaviors = self.behaviors.read().await;
        let behavior = behaviors.get(&request.method).cloned().unwrap_or_default();

        // Simulate latency
        if behavior.latency_ms > 0 {
            tokio::time::sleep(Duration::from_millis(behavior.latency_ms)).await;
        }

        // Return custom response if configured
        if let Some(custom) = behavior.custom_response {
            return RpcResponse {
                id: request.id,
                result: Some(custom),
                error: None,
            };
        }

        // Handle request based on method
        if behavior.should_succeed {
            let result = match request.method.as_str() {
                "get_status" => {
                    let status = self.node.get_status().await;
                    serde_json::to_value(status).ok()
                }
                "start" => match self.node.start().await {
                    Ok(_) => Some(serde_json::Value::Bool(true)),
                    Err(e) => {
                        return RpcResponse {
                            id: request.id,
                            result: None,
                            error: Some(RpcError {
                                code: -1,
                                message: e.to_string(),
                                data: None,
                            }),
                        }
                    }
                },
                "stop" => match self.node.stop().await {
                    Ok(_) => Some(serde_json::Value::Bool(true)),
                    Err(e) => {
                        return RpcResponse {
                            id: request.id,
                            result: None,
                            error: Some(RpcError {
                                code: -1,
                                message: e.to_string(),
                                data: None,
                            }),
                        }
                    }
                },
                "list_peers" => {
                    let peers = self.node.peers.read().await;
                    let peer_info: Vec<PeerInfo> = peers
                        .iter()
                        .map(|p| PeerInfo {
                            id: p.id.clone(),
                            address: p.address.clone(),
                            connected_duration: SystemTime::now()
                                .duration_since(p.connected_at)
                                .unwrap_or(Duration::from_secs(0))
                                .as_secs(),
                            messages_sent: p.messages_sent,
                            messages_received: p.messages_received,
                            last_seen: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            status: "Connected".to_string(),
                            latency: None,
                        })
                        .collect();
                    serde_json::to_value(peer_info).ok()
                }
                "add_peer" => {
                    if let Some(params) = request.params.as_object() {
                        if let Some(address) = params.get("address").and_then(|v| v.as_str()) {
                            match self.node.add_peer(address.to_string()).await {
                                Ok(_) => Some(serde_json::Value::Bool(true)),
                                Err(e) => {
                                    return RpcResponse {
                                        id: request.id,
                                        result: None,
                                        error: Some(RpcError {
                                            code: -1,
                                            message: e.to_string(),
                                            data: None,
                                        }),
                                    }
                                }
                            }
                        } else {
                            return RpcResponse {
                                id: request.id,
                                result: None,
                                error: Some(RpcError {
                                    code: -32602,
                                    message: "Invalid params: missing address".to_string(),
                                    data: None,
                                }),
                            };
                        }
                    } else {
                        return RpcResponse {
                            id: request.id,
                            result: None,
                            error: Some(RpcError {
                                code: -32602,
                                message: "Invalid params".to_string(),
                                data: None,
                            }),
                        };
                    }
                }
                "get_network_stats" => {
                    let stats = self.node.network_stats.read().await;
                    serde_json::to_value(&*stats).ok()
                }
                "test_network" => {
                    // Simulate network test results
                    let results = vec![
                        NetworkTestResult {
                            peer_id: "peer-1".to_string(),
                            address: "192.168.1.10:8080".to_string(),
                            reachable: true,
                            latency: Some(15.5),
                            error: None,
                        },
                        NetworkTestResult {
                            peer_id: "peer-2".to_string(),
                            address: "192.168.1.11:8080".to_string(),
                            reachable: true,
                            latency: Some(22.3),
                            error: None,
                        },
                        NetworkTestResult {
                            peer_id: "peer-3".to_string(),
                            address: "192.168.1.12:8080".to_string(),
                            reachable: false,
                            latency: None,
                            error: Some("Connection timeout".to_string()),
                        },
                    ];
                    serde_json::to_value(results).ok()
                }
                _ => Some(serde_json::json!({
                    "message": format!("Mock response for {}", request.method)
                })),
            };

            RpcResponse {
                id: request.id,
                result,
                error: None,
            }
        } else {
            RpcResponse {
                id: request.id,
                result: None,
                error: Some(RpcError {
                    code: -1,
                    message: behavior.error_message,
                    data: None,
                }),
            }
        }
    }

    /// Get request history
    pub fn get_request_history(&self) -> Vec<RpcRequest> {
        self.request_history.lock().unwrap().clone()
    }

    /// Clear request history
    pub fn clear_history(&self) {
        self.request_history.lock().unwrap().clear();
    }
}

/// Test scenario builder for creating complex test setups
pub struct TestScenarioBuilder {
    /// Nodes in the scenario
    pub nodes: HashMap<String, Arc<MockNode>>,
    /// Network topology
    pub topology: NetworkTopology,
    /// Global behaviors
    pub global_behaviors: HashMap<String, MockBehavior>,
}

#[derive(Debug, Clone)]
pub enum NetworkTopology {
    /// All nodes connected to all other nodes
    FullMesh,
    /// Nodes connected in a ring
    Ring,
    /// Star topology with one central node
    Star { center: String },
    /// Custom topology with specific connections
    Custom { connections: Vec<(String, String)> },
}

impl TestScenarioBuilder {
    /// Create new test scenario builder
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            topology: NetworkTopology::FullMesh,
            global_behaviors: HashMap::new(),
        }
    }

    /// Add a node to the scenario
    pub fn add_node(mut self, id: String) -> Self {
        let node = Arc::new(MockNode::new(id.clone()));
        self.nodes.insert(id, node);
        self
    }

    /// Set network topology
    pub fn with_topology(mut self, topology: NetworkTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Set global behavior for all nodes
    pub fn with_global_behavior(mut self, method: String, behavior: MockBehavior) -> Self {
        self.global_behaviors.insert(method, behavior);
        self
    }

    /// Build and configure the scenario
    pub async fn build(self) -> TestScenario {
        let mut scenario = TestScenario {
            nodes: self.nodes,
            start_time: SystemTime::now(),
        };

        // Apply global behaviors to all nodes
        for (method, behavior) in self.global_behaviors {
            for node in scenario.nodes.values() {
                node.set_behavior(&method, behavior.clone()).await;
            }
        }

        // Configure network topology
        match self.topology {
            NetworkTopology::FullMesh => {
                scenario.configure_full_mesh().await;
            }
            NetworkTopology::Ring => {
                scenario.configure_ring().await;
            }
            NetworkTopology::Star { center } => {
                scenario.configure_star(&center).await;
            }
            NetworkTopology::Custom { connections } => {
                scenario.configure_custom(connections).await;
            }
        }

        scenario
    }
}

/// Configured test scenario
pub struct TestScenario {
    /// Nodes in the scenario
    pub nodes: HashMap<String, Arc<MockNode>>,
    /// Scenario start time
    pub start_time: SystemTime,
}

impl TestScenario {
    /// Configure full mesh topology
    async fn configure_full_mesh(&self) {
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();

        for (i, node_id) in node_ids.iter().enumerate() {
            if let Some(node) = self.nodes.get(node_id) {
                for (j, peer_id) in node_ids.iter().enumerate() {
                    if i != j {
                        let address = format!("{}:8080", peer_id);
                        let _ = node.add_peer(address).await;
                    }
                }
            }
        }
    }

    /// Configure ring topology
    async fn configure_ring(&self) {
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        let n = node_ids.len();

        for (i, node_id) in node_ids.iter().enumerate() {
            if let Some(node) = self.nodes.get(node_id) {
                // Connect to next node in ring
                let next_idx = (i + 1) % n;
                let next_id = &node_ids[next_idx];
                let address = format!("{}:8080", next_id);
                let _ = node.add_peer(address).await;

                // Connect to previous node in ring
                let prev_idx = if i == 0 { n - 1 } else { i - 1 };
                let prev_id = &node_ids[prev_idx];
                let address = format!("{}:8080", prev_id);
                let _ = node.add_peer(address).await;
            }
        }
    }

    /// Configure star topology
    async fn configure_star(&self, center: &str) {
        if let Some(center_node) = self.nodes.get(center) {
            for (node_id, node) in &self.nodes {
                if node_id != center {
                    // Connect peripheral node to center
                    let center_address = format!("{}:8080", center);
                    let _ = node.add_peer(center_address).await;

                    // Connect center to peripheral node
                    let node_address = format!("{}:8080", node_id);
                    let _ = center_node.add_peer(node_address).await;
                }
            }
        }
    }

    /// Configure custom topology
    async fn configure_custom(&self, connections: Vec<(String, String)>) {
        for (from, to) in connections {
            if let Some(from_node) = self.nodes.get(&from) {
                let to_address = format!("{}:8080", to);
                let _ = from_node.add_peer(to_address).await;
            }
        }
    }

    /// Start all nodes in the scenario
    pub async fn start_all_nodes(&self) -> Result<()> {
        for node in self.nodes.values() {
            node.start().await?;
        }
        Ok(())
    }

    /// Stop all nodes in the scenario
    pub async fn stop_all_nodes(&self) -> Result<()> {
        for node in self.nodes.values() {
            node.stop().await?;
        }
        Ok(())
    }

    /// Simulate network activity across all nodes
    pub async fn simulate_activity(&self, duration: Duration) {
        let end_time = SystemTime::now() + duration;

        while SystemTime::now() < end_time {
            for node in self.nodes.values() {
                node.simulate_activity().await;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get aggregated network statistics
    pub async fn get_aggregate_stats(&self) -> NetworkStats {
        let mut total_stats = NetworkStats {
            total_connections: 0,
            active_connections: 0,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            average_latency: 0.0,
            uptime: 0,
        };

        let mut latency_sum = 0.0;
        let mut latency_count = 0;

        for node in self.nodes.values() {
            let stats = node.network_stats.read().await;
            total_stats.total_connections += stats.total_connections;
            total_stats.active_connections += stats.active_connections;
            total_stats.messages_sent += stats.messages_sent;
            total_stats.messages_received += stats.messages_received;
            total_stats.bytes_sent += stats.bytes_sent;
            total_stats.bytes_received += stats.bytes_received;

            if stats.average_latency > 0.0 {
                latency_sum += stats.average_latency;
                latency_count += 1;
            }
        }

        if latency_count > 0 {
            total_stats.average_latency = latency_sum / latency_count as f64;
        }

        total_stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_node_lifecycle() {
        let node = MockNode::new("test-node".to_string());

        // Test initial state
        assert!(matches!(*node.state.read().await, NodeState::Stopped));

        // Test start
        assert!(node.start().await.is_ok());
        assert!(matches!(*node.state.read().await, NodeState::Running));

        // Test double start
        assert!(node.start().await.is_err());

        // Test stop
        assert!(node.stop().await.is_ok());
        assert!(matches!(*node.state.read().await, NodeState::Stopped));
    }

    #[tokio::test]
    async fn test_mock_peer_management() {
        let node = MockNode::new("test-node".to_string());

        // Add peer
        assert!(node.add_peer("192.168.1.10:8080".to_string()).await.is_ok());
        assert_eq!(node.peers.read().await.len(), 1);

        // Add duplicate peer
        assert!(node
            .add_peer("192.168.1.10:8080".to_string())
            .await
            .is_err());

        // Remove peer
        let peer_id = node.peers.read().await[0].id.clone();
        assert!(node.remove_peer(&peer_id).await.is_ok());
        assert_eq!(node.peers.read().await.len(), 0);

        // Remove non-existent peer
        assert!(node.remove_peer("non-existent").await.is_err());
    }

    #[tokio::test]
    async fn test_mock_rpc_client() {
        let node = Arc::new(MockNode::new("test-node".to_string()));
        let rpc = MockRpcClient::new(node);

        // Test successful request
        let request = RpcRequest {
            id: Uuid::new_v4(),
            method: "get_status".to_string(),
            params: serde_json::Value::Null,
        };

        let response = rpc.process_request(request.clone()).await;
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Test request with error behavior
        rpc.set_behavior(
            "test_error",
            MockBehavior {
                should_succeed: false,
                error_message: "Test error".to_string(),
                ..Default::default()
            },
        )
        .await;

        let error_request = RpcRequest {
            id: Uuid::new_v4(),
            method: "test_error".to_string(),
            params: serde_json::Value::Null,
        };

        let error_response = rpc.process_request(error_request).await;
        assert!(error_response.result.is_none());
        assert!(error_response.error.is_some());
    }

    #[tokio::test]
    async fn test_scenario_builder() {
        let scenario = TestScenarioBuilder::new()
            .add_node("node1".to_string())
            .add_node("node2".to_string())
            .add_node("node3".to_string())
            .with_topology(NetworkTopology::Ring)
            .build()
            .await;

        // Start all nodes
        assert!(scenario.start_all_nodes().await.is_ok());

        // Check ring topology
        for node in scenario.nodes.values() {
            assert_eq!(node.peers.read().await.len(), 2); // Each node should have 2 peers in a ring
        }

        // Stop all nodes
        assert!(scenario.stop_all_nodes().await.is_ok());
    }
}
