use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Elastic device mesh for dynamic node management
pub struct ElasticDeviceMesh {
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    topology: Arc<RwLock<MeshTopology>>,
    heartbeat_timeout: Duration,
    checkpoint_manager: Arc<CheckpointManager>,
    event_tx: mpsc::Sender<MeshEvent>,
    event_rx: Arc<RwLock<mpsc::Receiver<MeshEvent>>>,
}

#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub id: String,
    pub address: String,
    pub capabilities: NodeCapabilities,
    pub last_heartbeat: Instant,
    pub status: NodeStatus,
    pub reliability_score: f32,
}

#[derive(Clone, Debug)]
pub struct NodeCapabilities {
    pub compute_flops: f64,        // FLOPs available
    pub memory_gb: f32,            // Memory in GB
    pub bandwidth_mbps: f32,       // Network bandwidth
    pub has_gpu: bool,
    pub gpu_memory_gb: Option<f32>,
    pub node_type: NodeType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeType {
    CloudGPU,
    EdgeDevice,
    BrowserClient,
    Validator,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Active,
    Syncing,
    Inactive,
    Failed,
}

#[derive(Debug)]
struct MeshTopology {
    /// Direct connections between nodes
    connections: HashMap<String, HashSet<String>>,
    /// Regional groupings for hierarchical organization
    regions: HashMap<String, Vec<String>>,
    /// Bandwidth matrix between nodes
    bandwidth_map: HashMap<(String, String), f32>,
}

#[derive(Debug)]
pub enum MeshEvent {
    NodeJoined(String),
    NodeLeft(String),
    NodeFailed(String),
    TopologyChanged,
}

struct CheckpointManager {
    latest_checkpoint: RwLock<Option<ModelCheckpoint>>,
    checkpoint_servers: RwLock<Vec<String>>, // Nodes serving checkpoints
}

#[derive(Clone)]
struct ModelCheckpoint {
    version: u64,
    hash: String,
    size_bytes: u64,
    timestamp: Instant,
}

impl ElasticDeviceMesh {
    pub async fn new() -> anyhow::Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        let mesh = Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            topology: Arc::new(RwLock::new(MeshTopology {
                connections: HashMap::new(),
                regions: HashMap::new(),
                bandwidth_map: HashMap::new(),
            })),
            heartbeat_timeout: Duration::from_secs(6),
            checkpoint_manager: Arc::new(CheckpointManager {
                latest_checkpoint: RwLock::new(None),
                checkpoint_servers: RwLock::new(Vec::new()),
            }),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
        };

        // Start heartbeat monitor
        mesh.start_heartbeat_monitor();

        Ok(mesh)
    }

    /// Add a new node to the mesh
    pub async fn add_node(&mut self, node: NodeInfo) -> anyhow::Result<()> {
        let node_id = node.id.clone();
        info!("Adding node {} to elastic mesh", node_id);

        // Add to nodes map
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(node_id.clone(), node);
        }

        // Update topology
        self.update_topology_for_new_node(&node_id).await?;

        // Trigger checkpoint sync for new node
        self.initiate_checkpoint_sync(&node_id).await?;

        // Send event
        let _ = self.event_tx.send(MeshEvent::NodeJoined(node_id)).await;

        Ok(())
    }

    /// Remove a node from the mesh
    pub async fn remove_node(&mut self, node_id: &str) -> anyhow::Result<()> {
        info!("Removing node {} from elastic mesh", node_id);

        // Remove from nodes map
        {
            let mut nodes = self.nodes.write().await;
            nodes.remove(node_id);
        }

        // Update topology
        self.update_topology_for_removed_node(node_id).await?;

        // Remove from checkpoint servers if present
        {
            let mut servers = self.checkpoint_manager.checkpoint_servers.write().await;
            servers.retain(|id| id != node_id);
        }

        // Send event
        let _ = self.event_tx.send(MeshEvent::NodeLeft(node_id.to_string())).await;

        Ok(())
    }

    /// Check for new nodes attempting to join
    pub async fn check_new_nodes(&self) -> anyhow::Result<Vec<NodeInfo>> {
        // In a real implementation, this would check network discovery
        // For now, return empty (nodes added via add_node)
        Ok(vec![])
    }

    /// Check for failed nodes based on heartbeat timeout
    pub async fn check_failed_nodes(&self) -> anyhow::Result<Vec<String>> {
        let mut failed_nodes = Vec::new();
        let now = Instant::now();

        let mut nodes = self.nodes.write().await;
        for (node_id, node_info) in nodes.iter_mut() {
            if node_info.status == NodeStatus::Active 
                && now.duration_since(node_info.last_heartbeat) > self.heartbeat_timeout {
                
                warn!("Node {} failed heartbeat check", node_id);
                node_info.status = NodeStatus::Failed;
                failed_nodes.push(node_id.clone());
                
                // Reduce reliability score
                node_info.reliability_score *= 0.9;
            }
        }

        Ok(failed_nodes)
    }

    /// Update heartbeat for a node
    pub async fn update_heartbeat(&self, node_id: &str) -> anyhow::Result<()> {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            node.last_heartbeat = Instant::now();
            if node.status == NodeStatus::Failed {
                node.status = NodeStatus::Active;
                info!("Node {} recovered", node_id);
            }
        }
        Ok(())
    }

    /// Get active nodes for task assignment
    pub async fn get_active_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values()
            .filter(|n| n.status == NodeStatus::Active)
            .cloned()
            .collect()
    }

    /// Get nodes by capability
    pub async fn get_nodes_by_capability(&self, min_flops: f64, requires_gpu: bool) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values()
            .filter(|n| {
                n.status == NodeStatus::Active 
                && n.capabilities.compute_flops >= min_flops
                && (!requires_gpu || n.capabilities.has_gpu)
            })
            .cloned()
            .collect()
    }

    /// Update mesh topology for new node
    async fn update_topology_for_new_node(&self, node_id: &str) -> anyhow::Result<()> {
        let mut topology = self.topology.write().await;
        
        // Initialize connections for new node
        topology.connections.insert(node_id.to_string(), HashSet::new());
        
        // Connect to nearby nodes (simplified: connect to 3-5 random active nodes)
        let active_nodes = self.get_active_nodes().await;
        let num_connections = 3.min(active_nodes.len());
        
        for (i, node) in active_nodes.iter().take(num_connections).enumerate() {
            if node.id != node_id {
                // Add bidirectional connection
                topology.connections.get_mut(node_id).unwrap().insert(node.id.clone());
                topology.connections.entry(node.id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(node_id.to_string());
                
                // Estimate bandwidth (simplified)
                let bandwidth = 100.0 + (i as f32 * 50.0); // 100-250 Mbps
                topology.bandwidth_map.insert(
                    (node_id.to_string(), node.id.clone()),
                    bandwidth,
                );
                topology.bandwidth_map.insert(
                    (node.id.clone(), node_id.to_string()),
                    bandwidth,
                );
            }
        }
        
        let _ = self.event_tx.send(MeshEvent::TopologyChanged).await;
        Ok(())
    }

    /// Update topology when node is removed
    async fn update_topology_for_removed_node(&self, node_id: &str) -> anyhow::Result<()> {
        let mut topology = self.topology.write().await;
        
        // Remove node's connections
        if let Some(connections) = topology.connections.remove(node_id) {
            // Remove from other nodes' connection lists
            for connected_node in connections {
                if let Some(node_connections) = topology.connections.get_mut(&connected_node) {
                    node_connections.remove(node_id);
                }
            }
        }
        
        // Remove bandwidth entries
        topology.bandwidth_map.retain(|(from, to), _| {
            from != node_id && to != node_id
        });
        
        let _ = self.event_tx.send(MeshEvent::TopologyChanged).await;
        Ok(())
    }

    /// Initiate checkpoint sync for new node
    async fn initiate_checkpoint_sync(&self, node_id: &str) -> anyhow::Result<()> {
        info!("Initiating checkpoint sync for node {}", node_id);
        
        // Mark node as syncing
        {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(node_id) {
                node.status = NodeStatus::Syncing;
            }
        }
        
        // Find best checkpoint server (highest bandwidth)
        let checkpoint_server = self.find_best_checkpoint_server(node_id).await?;
        
        // In real implementation, would initiate P2P transfer
        // For now, simulate with a timer
        let nodes_clone = self.nodes.clone();
        let node_id_clone = node_id.to_string();
        tokio::spawn(async move {
            // Simulate checkpoint download time
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            // Mark as active after sync
            let mut nodes = nodes_clone.write().await;
            if let Some(node) = nodes.get_mut(&node_id_clone) {
                node.status = NodeStatus::Active;
                info!("Node {} completed checkpoint sync", node_id_clone);
            }
        });
        
        Ok(())
    }

    /// Find best checkpoint server based on bandwidth
    async fn find_best_checkpoint_server(&self, requesting_node: &str) -> anyhow::Result<String> {
        let servers = self.checkpoint_manager.checkpoint_servers.read().await;
        let topology = self.topology.read().await;
        
        let best_server = servers.iter()
            .max_by_key(|server| {
                topology.bandwidth_map
                    .get(&(server.to_string(), requesting_node.to_string()))
                    .unwrap_or(&0.0) as &f32
            })
            .cloned()
            .unwrap_or_else(|| "default-server".to_string());
        
        Ok(best_server)
    }

    /// Start heartbeat monitoring task
    fn start_heartbeat_monitor(&self) {
        let mesh_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(2));
            
            loop {
                interval.tick().await;
                
                match mesh_clone.check_failed_nodes().await {
                    Ok(failed) => {
                        for node_id in failed {
                            let _ = mesh_clone.event_tx.send(MeshEvent::NodeFailed(node_id)).await;
                        }
                    }
                    Err(e) => error!("Heartbeat monitor error: {}", e),
                }
            }
        });
    }

    /// Calculate optimal node assignment for a task
    pub async fn calculate_optimal_assignment(
        &self,
        task_size: f64,
        requires_gpu: bool,
    ) -> anyhow::Result<Vec<String>> {
        let suitable_nodes = self.get_nodes_by_capability(task_size, requires_gpu).await;
        
        // Sort by reliability and capability
        let mut ranked_nodes: Vec<_> = suitable_nodes.into_iter()
            .map(|n| (n.id.clone(), n.reliability_score * n.capabilities.compute_flops as f32))
            .collect();
        
        ranked_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Return top nodes
        Ok(ranked_nodes.into_iter()
            .take(5)
            .map(|(id, _)| id)
            .collect())
    }
}

impl Clone for ElasticDeviceMesh {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            topology: self.topology.clone(),
            heartbeat_timeout: self.heartbeat_timeout,
            checkpoint_manager: self.checkpoint_manager.clone(),
            event_tx: self.event_tx.clone(),
            event_rx: self.event_rx.clone(),
        }
    }
}