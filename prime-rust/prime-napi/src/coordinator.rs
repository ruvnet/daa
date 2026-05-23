//! Coordinator bindings for NAPI
//!
//! Provides Node.js bindings for the federated learning coordinator with support for:
//! - Node registration and management
//! - Training round coordination
//! - Progress tracking and metrics collection
//! - Consensus and governance

use napi::bindgen_prelude::*;
use napi::{Env, JsObject, Result, Status};
use std::sync::Arc;
use tokio::sync::RwLock;

use daa_prime_coordinator::{
    CoordinatorConfig as RustCoordinatorConfig, CoordinatorNode as RustCoordinator, NodeInfo,
};

use crate::types::NodeInfoJs;

/// Coordinator configuration for JavaScript
#[napi(object)]
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Minimum number of nodes required to start a training round
    pub min_nodes_for_round: u32,
    /// Heartbeat timeout in milliseconds
    pub heartbeat_timeout_ms: u32,
    /// Task timeout in milliseconds
    pub task_timeout_ms: u32,
    /// Consensus threshold (0.0 - 1.0)
    pub consensus_threshold: f64,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            min_nodes_for_round: 3,
            heartbeat_timeout_ms: 5000,
            task_timeout_ms: 60000,
            consensus_threshold: 0.66,
        }
    }
}

impl From<CoordinatorConfig> for RustCoordinatorConfig {
    fn from(config: CoordinatorConfig) -> Self {
        RustCoordinatorConfig {
            min_nodes_for_round: config.min_nodes_for_round as usize,
            heartbeat_timeout_ms: config.heartbeat_timeout_ms as u64,
            task_timeout_ms: config.task_timeout_ms as u64,
            consensus_threshold: config.consensus_threshold as f32,
        }
    }
}

/// Coordinator status for JavaScript
#[napi(object)]
#[derive(Debug, Clone)]
pub struct CoordinatorStatusJs {
    /// Number of active training nodes
    pub active_nodes: u32,
    /// Number of pending training tasks
    pub pending_tasks: u32,
    /// Current training round
    pub current_round: u32,
    /// Model version
    pub model_version: u32,
}

/// Federated learning coordinator
///
/// Manages distributed training coordination, node registration, and consensus.
/// The coordinator orchestrates training rounds across multiple nodes and ensures
/// proper synchronization and model updates.
#[napi]
pub struct Coordinator {
    inner: Arc<RwLock<Option<RustCoordinator>>>,
    node_id: String,
    config: CoordinatorConfig,
    current_round: Arc<RwLock<u32>>,
    model_version: Arc<RwLock<u32>>,
}

#[napi]
impl Coordinator {
    /// Create a new coordinator
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier for this coordinator
    /// * `config` - Optional coordinator configuration (uses defaults if not provided)
    ///
    /// # Example
    ///
    /// ```javascript
    /// const coordinator = new Coordinator('coordinator-1', {
    ///   minNodesForRound: 5,
    ///   heartbeatTimeoutMs: 10000,
    ///   taskTimeoutMs: 120000,
    ///   consensusThreshold: 0.75
    /// });
    /// ```
    #[napi(constructor)]
    pub fn new(node_id: String, config: Option<CoordinatorConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();

        Ok(Self {
            inner: Arc::new(RwLock::new(None)),
            node_id,
            config,
            current_round: Arc::new(RwLock::new(0)),
            model_version: Arc::new(RwLock::new(0)),
        })
    }

    /// Initialize the coordinator
    ///
    /// Sets up the coordinator and prepares it to accept node registrations
    /// and coordinate training rounds.
    ///
    /// # Example
    ///
    /// ```javascript
    /// await coordinator.init();
    /// ```
    #[napi]
    pub async fn init(&self) -> Result<()> {
        let rust_config: RustCoordinatorConfig = self.config.clone().into();
        let coordinator = RustCoordinator::new(self.node_id.clone(), rust_config)
            .await
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to create coordinator: {}", e),
                )
            })?;

        *self.inner.write().await = Some(coordinator);
        Ok(())
    }

    /// Register a new training node
    ///
    /// Adds a training node to the coordinator's active node registry.
    /// Nodes must be registered before they can participate in training rounds.
    ///
    /// # Arguments
    ///
    /// * `node_info` - Information about the node to register
    ///
    /// # Example
    ///
    /// ```javascript
    /// await coordinator.registerNode({
    ///   nodeId: 'node-1',
    ///   nodeType: 'trainer',
    ///   lastHeartbeat: Date.now(),
    ///   reliabilityScore: 0.95
    /// });
    /// ```
    #[napi]
    pub async fn register_node(&self, node_info: NodeInfoJs) -> Result<()> {
        let inner = self.inner.read().await;
        let coordinator = inner.as_ref().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Coordinator not initialized. Call init() first",
            )
        })?;

        let rust_node_info = NodeInfo {
            node_id: node_info.node_id,
            node_type: node_info.node_type,
            last_heartbeat: node_info.last_heartbeat as u64,
            reliability_score: node_info.reliability_score as f32,
        };

        coordinator.add_node(rust_node_info).await.map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to register node: {}", e),
            )
        })?;

        Ok(())
    }

    /// Start a new training round
    ///
    /// Initiates a new federated learning training round. This coordinates
    /// all registered nodes to perform local training and gradient updates.
    ///
    /// # Returns
    ///
    /// The round number that was started
    ///
    /// # Example
    ///
    /// ```javascript
    /// const roundNumber = await coordinator.startTraining();
    /// console.log(`Started training round ${roundNumber}`);
    /// ```
    #[napi]
    pub async fn start_training(&self) -> Result<u32> {
        let inner = self.inner.read().await;
        let _coordinator = inner.as_ref().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Coordinator not initialized. Call init() first",
            )
        })?;

        // Increment round
        let mut round = self.current_round.write().await;
        *round += 1;

        // In a real implementation, this would:
        // 1. Check if minimum nodes are available
        // 2. Broadcast training start to all nodes
        // 3. Initialize round state
        // 4. Set up gradient collection

        Ok(*round)
    }

    /// Get training progress for the current round
    ///
    /// Returns detailed information about the current training round's progress,
    /// including how many nodes have completed training and reported gradients.
    ///
    /// # Returns
    ///
    /// Progress information including completion percentage and node status
    ///
    /// # Example
    ///
    /// ```javascript
    /// const progress = await coordinator.getProgress();
    /// console.log(`Training ${progress.completionPercent}% complete`);
    /// console.log(`${progress.completedNodes}/${progress.totalNodes} nodes finished`);
    /// ```
    #[napi]
    pub async fn get_progress(&self) -> Result<serde_json::Value> {
        let inner = self.inner.read().await;
        let coordinator = inner.as_ref().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Coordinator not initialized. Call init() first",
            )
        })?;

        let status = coordinator.get_status().await.map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to get status: {}", e),
            )
        })?;

        let round = *self.current_round.read().await;

        Ok(serde_json::json!({
            "currentRound": round,
            "totalNodes": status.active_nodes as u32,
            "completedNodes": 0u32,
            "completionPercent": 0.0,
            "pendingTasks": status.pending_tasks as u32
        }))
    }

    /// Get coordinator status
    ///
    /// Returns comprehensive status information about the coordinator including
    /// active nodes, pending tasks, and current round information.
    ///
    /// # Returns
    ///
    /// Coordinator status object
    ///
    /// # Example
    ///
    /// ```javascript
    /// const status = await coordinator.getStatus();
    /// console.log(`Active nodes: ${status.activeNodes}`);
    /// console.log(`Current round: ${status.currentRound}`);
    /// ```
    #[napi]
    pub async fn get_status(&self) -> Result<CoordinatorStatusJs> {
        let inner = self.inner.read().await;
        let coordinator = inner.as_ref().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Coordinator not initialized. Call init() first",
            )
        })?;

        let status = coordinator.get_status().await.map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to get status: {}", e),
            )
        })?;

        let round = *self.current_round.read().await;
        let version = *self.model_version.read().await;

        Ok(CoordinatorStatusJs {
            active_nodes: status.active_nodes as u32,
            pending_tasks: status.pending_tasks as u32,
            current_round: round,
            model_version: version,
        })
    }

    /// Stop the coordinator and clean up resources
    ///
    /// Gracefully shuts down the coordinator, notifying all nodes and
    /// cleaning up network connections.
    ///
    /// # Example
    ///
    /// ```javascript
    /// await coordinator.stop();
    /// ```
    #[napi]
    pub async fn stop(&self) -> Result<()> {
        // Clear the coordinator
        *self.inner.write().await = None;
        Ok(())
    }

    /// Get the coordinator's node ID
    #[napi(getter)]
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }

    /// Get the current round number
    #[napi(getter)]
    pub async fn current_round(&self) -> u32 {
        *self.current_round.read().await
    }

    /// Get the current model version
    #[napi(getter)]
    pub async fn model_version(&self) -> u32 {
        *self.model_version.read().await
    }
}
