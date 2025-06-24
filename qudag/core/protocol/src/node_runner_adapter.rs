use crate::node_runner::NodeRunner;
use crate::rpc_server::{NetworkStats, NodeRunnerTrait, PeerInfo};
use libp2p::Multiaddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Adapter that implements NodeRunnerTrait for the actual NodeRunner
pub struct NodeRunnerAdapter {
    /// Reference to the actual NodeRunner
    node_runner: Arc<RwLock<NodeRunner>>,
    /// Start time for uptime calculation
    start_time: SystemTime,
}

impl NodeRunnerAdapter {
    pub fn new(node_runner: Arc<RwLock<NodeRunner>>) -> Self {
        Self {
            node_runner,
            start_time: SystemTime::now(),
        }
    }
}

impl std::fmt::Debug for NodeRunnerAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRunnerAdapter")
            .field("start_time", &self.start_time)
            .finish()
    }
}

impl NodeRunnerTrait for NodeRunnerAdapter {
    fn get_status(
        &self,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>,
                > + Send,
        >,
    > {
        let node_runner = self.node_runner.clone();
        Box::pin(async move {
            let runner = node_runner.read().await;
            runner.status().await.map_err(|e| e.into())
        })
    }

    fn get_connected_peers(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Vec<PeerInfo>> + Send>> {
        let node_runner = self.node_runner.clone();
        Box::pin(async move {
            let runner = node_runner.read().await;

            // Get P2P handle if available
            if let Some(p2p_handle) = runner.p2p_handle() {
                let peer_ids = p2p_handle.connected_peers().await;

                // Convert libp2p peer IDs to PeerInfo
                peer_ids
                    .into_iter()
                    .map(|peer_id| {
                        PeerInfo {
                            id: peer_id.to_string(),
                            address: "unknown".to_string(), // TODO: Get actual address
                            connected_duration: 0,          // TODO: Track connection time
                            messages_sent: 0,               // TODO: Get from metrics
                            messages_received: 0,           // TODO: Get from metrics
                            last_seen: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            status: "Connected".to_string(),
                            latency: None, // TODO: Get from ping
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        })
    }

    fn dial_peer(
        &self,
        address: String,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let node_runner = self.node_runner.clone();
        Box::pin(async move {
            let runner = node_runner.read().await;

            if let Some(p2p_handle) = runner.p2p_handle() {
                // Parse the address as Multiaddr
                let multiaddr: Multiaddr = address
                    .parse()
                    .map_err(|e| format!("Invalid multiaddr: {}", e))?;

                p2p_handle
                    .dial(multiaddr)
                    .await
                    .map_err(|e| format!("Failed to dial peer: {}", e))
            } else {
                Err("P2P handle not available".to_string())
            }
        })
    }

    fn disconnect_peer(
        &self,
        peer_id: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let peer_id = peer_id.to_string();
        Box::pin(async move {
            // libp2p doesn't have a direct "disconnect" method for individual peers
            // We would need to implement this by closing all connections to the peer
            // For now, return an error indicating this is not yet implemented
            Err(format!(
                "Disconnecting peer {} not yet implemented",
                peer_id
            ))
        })
    }

    fn get_network_stats(&self) -> Pin<Box<dyn std::future::Future<Output = NetworkStats> + Send>> {
        let node_runner = self.node_runner.clone();
        let start_time = self.start_time;
        Box::pin(async move {
            let runner = node_runner.read().await;

            if let Some(p2p_handle) = runner.p2p_handle() {
                let connected_peers = p2p_handle.connected_peers().await;

                NetworkStats {
                    total_connections: connected_peers.len(),
                    active_connections: connected_peers.len(),
                    messages_sent: 0,     // TODO: Get from metrics
                    messages_received: 0, // TODO: Get from metrics
                    bytes_sent: 0,        // TODO: Get from metrics
                    bytes_received: 0,    // TODO: Get from metrics
                    average_latency: 0.0, // TODO: Calculate from ping data
                    uptime: start_time.elapsed().unwrap_or_default().as_secs(),
                }
            } else {
                NetworkStats {
                    total_connections: 0,
                    active_connections: 0,
                    messages_sent: 0,
                    messages_received: 0,
                    bytes_sent: 0,
                    bytes_received: 0,
                    average_latency: 0.0,
                    uptime: start_time.elapsed().unwrap_or_default().as_secs(),
                }
            }
        })
    }

    fn shutdown(
        &self,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                + Send,
        >,
    > {
        let node_runner = self.node_runner.clone();
        Box::pin(async move {
            let mut runner = node_runner.write().await;
            runner.stop().await.map_err(|e| e.into())
        })
    }
}
