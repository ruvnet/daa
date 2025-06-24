use anyhow::Result;
use qudag_protocol::config::Config as ProtocolConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::info;

/// Network simulator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    /// Number of nodes to simulate
    pub node_count: usize,
    /// Network latency in milliseconds
    pub latency_ms: u64,
    /// Message drop rate (0.0-1.0)
    pub drop_rate: f64,
    /// Network partition probability
    pub partition_prob: f64,
}

/// Network simulator for testing protocol behavior
pub struct NetworkSimulator {
    config: SimulatorConfig,
    nodes: Vec<NodeHandle>,
    events_tx: mpsc::Sender<SimulatorEvent>,
}

/// Handle to a simulated node
struct NodeHandle {
    id: String,
    #[allow(dead_code)] // May be used in future node implementations
    config: ProtocolConfig,
    #[allow(dead_code)] // May be used in future message routing implementations
    msg_tx: mpsc::Sender<Vec<u8>>,
}

/// Events emitted by the simulator
#[derive(Debug)]
pub enum SimulatorEvent {
    /// Node joined the network
    NodeJoined(String),
    /// Node left the network
    NodeLeft(String),
    /// Network partition occurred
    Partition {
        /// List of nodes in the partition
        nodes: Vec<String>,
    },
    /// Network healed
    Heal,
}

impl NetworkSimulator {
    /// Create a new network simulator
    pub fn new(config: SimulatorConfig) -> (Self, mpsc::Receiver<SimulatorEvent>) {
        let (events_tx, events_rx) = mpsc::channel(1000);

        let simulator = Self {
            config,
            nodes: Vec::new(),
            events_tx,
        };

        (simulator, events_rx)
    }

    /// Add a new node to the network
    pub async fn add_node(&mut self, config: ProtocolConfig) -> Result<()> {
        let id = format!("node-{}", self.nodes.len());
        let (msg_tx, _msg_rx) = mpsc::channel(1000);

        self.nodes.push(NodeHandle {
            id: id.clone(),
            config,
            msg_tx,
        });

        self.events_tx.send(SimulatorEvent::NodeJoined(id)).await?;
        Ok(())
    }

    /// Remove a node from the network
    pub async fn remove_node(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.nodes.iter().position(|n| n.id == id) {
            self.nodes.remove(pos);
            self.events_tx
                .send(SimulatorEvent::NodeLeft(id.to_string()))
                .await?;
        }
        Ok(())
    }

    /// Simulate a network partition
    pub async fn create_partition(&mut self) -> Result<()> {
        let partition_size = (self.nodes.len() as f64 * self.config.partition_prob) as usize;
        let partitioned: Vec<_> = self.nodes[..partition_size]
            .iter()
            .map(|n| n.id.clone())
            .collect();

        self.events_tx
            .send(SimulatorEvent::Partition {
                nodes: partitioned.clone(),
            })
            .await?;

        info!("Created network partition with {} nodes", partition_size);
        Ok(())
    }

    /// Heal network partition
    pub async fn heal_partition(&mut self) -> Result<()> {
        self.events_tx.send(SimulatorEvent::Heal).await?;
        info!("Healed network partition");
        Ok(())
    }
}
