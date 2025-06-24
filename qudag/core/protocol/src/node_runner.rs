use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::select;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

// Import network components
use qudag_network::{
    p2p::{NetworkConfig as P2PNetworkConfig, P2PEvent, P2PNode, QuDagResponse},
    DarkResolver, P2PHandle,
};

// Import DAG components
use qudag_dag::{Dag, DagMessage, VertexId};

// Minimal RPC types for NodeRunner integration
#[derive(Debug, Clone)]
pub enum RpcTransport {
    Tcp(String),
    Unix(String),
}

#[derive(Debug, Clone)]
pub enum RpcCommand {
    Stop,
    GetStatus,
}

// Minimal RPC server placeholder
pub struct RpcServer {
    _transport: RpcTransport,
}

impl RpcServer {
    pub fn new_tcp(
        port: u16,
    ) -> (
        Self,
        tokio::sync::mpsc::Receiver<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    ) {
        let (_, rx) = tokio::sync::mpsc::channel(1);
        (
            Self {
                _transport: RpcTransport::Tcp(format!("127.0.0.1:{}", port)),
            },
            rx,
        )
    }

    pub fn new_unix(
        path: String,
    ) -> (
        Self,
        tokio::sync::mpsc::Receiver<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    ) {
        let (_, rx) = tokio::sync::mpsc::channel(1);
        (
            Self {
                _transport: RpcTransport::Unix(path),
            },
            rx,
        )
    }

    pub async fn start(&mut self) -> Result<(), String> {
        // Placeholder implementation
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        // Placeholder implementation
        Ok(())
    }
}

// Import protocol types
use crate::types::{ProtocolError, ProtocolEvent};

/// Errors that can occur during node operations
#[derive(Error, Debug)]
pub enum NodeRunnerError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("DAG error: {0}")]
    DagError(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),

    #[error("Node already started")]
    AlreadyStarted,

    #[error("Node not started")]
    NotStarted,

    #[error("Shutdown error: {0}")]
    ShutdownError(String),
}

/// Configuration for the NodeRunner
#[derive(Debug, Clone)]
pub struct NodeRunnerConfig {
    /// P2P network configuration
    pub p2p_config: P2PNetworkConfig,

    /// RPC server transport configuration
    pub rpc_transport: RpcTransport,

    /// Maximum concurrent DAG messages
    pub max_dag_concurrent: usize,

    /// Enable dark resolver
    pub enable_dark_resolver: bool,

    /// Node shutdown timeout
    pub shutdown_timeout: Duration,
}

impl Default for NodeRunnerConfig {
    fn default() -> Self {
        Self {
            p2p_config: P2PNetworkConfig::default(),
            rpc_transport: RpcTransport::Tcp("127.0.0.1:9090".to_string()),
            max_dag_concurrent: 100,
            enable_dark_resolver: true,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

/// The main node integration coordinator
pub struct NodeRunner {
    /// Configuration
    config: NodeRunnerConfig,

    /// P2P network handle
    p2p_handle: Option<P2PHandle>,

    /// P2P node task handle
    p2p_task_handle:
        Option<tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,

    /// DAG consensus
    dag: Arc<RwLock<Dag>>,

    /// RPC server
    rpc_server: Option<Arc<Mutex<RpcServer>>>,

    /// RPC command receiver
    #[allow(dead_code)]
    rpc_command_rx: Option<
        tokio::sync::mpsc::Receiver<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
    >,

    /// Dark resolver for .dark addresses
    dark_resolver: Option<Arc<RwLock<DarkResolver>>>,

    /// Event channel for protocol events
    #[allow(dead_code)]
    event_tx: mpsc::UnboundedSender<ProtocolEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<ProtocolEvent>>,

    /// Shutdown signal
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,

    /// Node state
    is_running: Arc<RwLock<bool>>,
}

impl NodeRunner {
    /// Create a new NodeRunner instance
    pub fn new(config: NodeRunnerConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Create DAG instance
        let dag = Arc::new(RwLock::new(Dag::new(config.max_dag_concurrent)));

        Self {
            config,
            p2p_handle: None,
            p2p_task_handle: None,
            dag,
            rpc_server: None,
            rpc_command_rx: None,
            dark_resolver: None,
            event_tx,
            event_rx: Some(event_rx),
            shutdown_tx: None,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize all components
    async fn initialize_components(&mut self) -> Result<(), NodeRunnerError> {
        info!("Initializing node components...");

        // Initialize P2P node
        let (mut p2p_node, p2p_handle) = P2PNode::new(self.config.p2p_config.clone())
            .await
            .map_err(|e| NodeRunnerError::NetworkError(e.to_string()))?;

        // Start the P2P node
        p2p_node
            .start()
            .await
            .map_err(|e| NodeRunnerError::NetworkError(e.to_string()))?;

        // Spawn the P2P node task
        let p2p_task_handle = tokio::spawn(async move {
            p2p_node
                .run()
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(std::io::Error::other(e.to_string()))
                })
        });

        self.p2p_handle = Some(p2p_handle);
        self.p2p_task_handle = Some(p2p_task_handle);

        // Initialize RPC server
        let (rpc_server, rpc_command_rx) = match &self.config.rpc_transport {
            RpcTransport::Tcp(addr) => {
                let port = addr
                    .split(':')
                    .next_back()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(9090);
                RpcServer::new_tcp(port)
            }
            RpcTransport::Unix(path) => RpcServer::new_unix(path.clone()),
        };
        self.rpc_server = Some(Arc::new(Mutex::new(rpc_server)));
        self.rpc_command_rx = Some(rpc_command_rx);

        // Initialize dark resolver if enabled
        if self.config.enable_dark_resolver {
            self.dark_resolver = Some(Arc::new(RwLock::new(DarkResolver::new())));
        }

        info!("All node components initialized successfully");
        Ok(())
    }

    /// Start the node and all its components
    pub async fn start(&mut self) -> Result<(), NodeRunnerError> {
        // Check if already running
        if *self.is_running.read().await {
            return Err(NodeRunnerError::AlreadyStarted);
        }

        info!("Starting QuDAG node...");

        // Initialize components if not already done
        if self.p2p_handle.is_none() {
            self.initialize_components().await?;
        }

        // Start RPC server
        if let Some(rpc_server) = &self.rpc_server {
            let mut server = rpc_server.lock().await;
            server
                .start()
                .await
                .map_err(|e| NodeRunnerError::RpcError(e.to_string()))?;
        }

        // Mark as running
        *self.is_running.write().await = true;

        info!("QuDAG node started successfully");
        Ok(())
    }

    /// Main event loop that bridges P2P messages to DAG
    pub async fn run(&mut self) -> Result<(), NodeRunnerError> {
        if !*self.is_running.read().await {
            return Err(NodeRunnerError::NotStarted);
        }

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let mut event_rx = self.event_rx.take().ok_or(NodeRunnerError::NotStarted)?;

        // Get P2P events from the handle if available
        let p2p_handle = self.p2p_handle.clone();

        info!("Node runner event loop started");

        loop {
            select! {
                // Handle P2P events (if P2P handle is available)
                p2p_event = async {
                    if let Some(ref handle) = p2p_handle {
                        handle.next_event().await
                    } else {
                        None::<P2PEvent>
                    }
                } => {
                    if let Some(event) = p2p_event {
                        if let Err(e) = self.handle_p2p_event(event).await {
                            error!("Error handling P2P event: {}", e);
                        }
                    }
                }

                // Handle protocol events
                Some(protocol_event) = event_rx.recv() => {
                    if let Err(e) = self.handle_protocol_event(protocol_event).await {
                        error!("Error handling protocol event: {}", e);
                    }
                }

                // Handle shutdown signal
                _ = &mut shutdown_rx => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        info!("Node runner event loop stopped");
        Ok(())
    }

    /// Handle P2P network events
    async fn handle_p2p_event(&self, event: P2PEvent) -> Result<(), NodeRunnerError> {
        match event {
            P2PEvent::MessageReceived {
                peer_id,
                topic,
                data,
            } => {
                debug!("Received message from peer {} on topic {}", peer_id, topic);

                // Convert P2P message to DAG message
                let dag_message = DagMessage {
                    id: VertexId::new(),
                    payload: data,
                    parents: Default::default(), // TODO: Extract parents from message
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                // Submit to DAG
                let dag = self.dag.write().await;
                dag.submit_message(dag_message)
                    .await
                    .map_err(|e| NodeRunnerError::DagError(e.to_string()))?;
            }

            P2PEvent::PeerConnected(peer_id) => {
                info!("Peer connected: {}", peer_id);
                // TODO: Update peer tracking
            }

            P2PEvent::PeerDisconnected(peer_id) => {
                info!("Peer disconnected: {}", peer_id);
                // TODO: Update peer tracking
            }

            P2PEvent::RequestReceived {
                peer_id,
                request,
                channel,
            } => {
                debug!("Received request from peer {}: {:?}", peer_id, request);
                // TODO: Handle custom requests
                let response = QuDagResponse {
                    request_id: request.request_id,
                    payload: vec![],
                };
                let _ = channel.send(response);
            }

            _ => {
                debug!("Unhandled P2P event: {:?}", event);
            }
        }

        Ok(())
    }

    /// Handle protocol events
    async fn handle_protocol_event(&self, event: ProtocolEvent) -> Result<(), NodeRunnerError> {
        match event {
            ProtocolEvent::MessageReceived { id, .. } => {
                debug!("Message received: {:?}", id);

                // Broadcast consensus result to network
                if let Some(p2p_handle) = &self.p2p_handle {
                    // TODO: Implement broadcast_consensus_result using p2p_handle
                    let _handle = p2p_handle;
                    // handle.publish("consensus", consensus_data).await?;
                }
            }

            ProtocolEvent::MessageFinalized { id, .. } => {
                info!("Message finalized: {:?}", id);
                // TODO: Handle finalization completion
            }

            _ => {
                debug!("Unhandled protocol event: {:?}", event);
            }
        }

        Ok(())
    }

    /// Handle RPC commands (placeholder implementation)
    async fn _handle_rpc_commands(
        mut _rx: tokio::sync::mpsc::Receiver<(
            RpcCommand,
            tokio::sync::oneshot::Sender<serde_json::Value>,
        )>,
        _event_tx: mpsc::UnboundedSender<ProtocolEvent>,
        _is_running: Arc<RwLock<bool>>,
    ) {
        // Placeholder implementation for RPC command handling
        // In a real implementation, this would process RPC commands
    }

    /// Gracefully stop the node and all its components
    pub async fn stop(&mut self) -> Result<(), NodeRunnerError> {
        if !*self.is_running.read().await {
            return Ok(());
        }

        info!("Stopping QuDAG node...");

        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Stop RPC server
        if let Some(rpc_server) = &self.rpc_server {
            let mut server = rpc_server.lock().await;
            server
                .stop()
                .await
                .map_err(|e| NodeRunnerError::RpcError(e.to_string()))?;
        }

        // Stop P2P node by canceling the task
        if let Some(task_handle) = self.p2p_task_handle.take() {
            task_handle.abort();
            if let Err(e) = task_handle.await {
                if !e.is_cancelled() {
                    warn!("P2P task shutdown error: {}", e);
                }
            }
        }

        // Drop P2P handle
        self.p2p_handle = None;

        // Mark as stopped
        *self.is_running.write().await = false;

        info!("QuDAG node stopped successfully");
        Ok(())
    }

    /// Get a reference to the P2P handle
    pub fn p2p_handle(&self) -> &Option<P2PHandle> {
        &self.p2p_handle
    }

    /// Get a reference to the DAG
    pub fn dag(&self) -> &Arc<RwLock<Dag>> {
        &self.dag
    }

    /// Get a reference to the RPC server
    pub fn rpc_server(&self) -> &Option<Arc<Mutex<RpcServer>>> {
        &self.rpc_server
    }

    /// Get a reference to the dark resolver
    pub fn dark_resolver(&self) -> &Option<Arc<RwLock<DarkResolver>>> {
        &self.dark_resolver
    }

    /// Get a reference to the running state
    pub fn is_running(&self) -> &Arc<RwLock<bool>> {
        &self.is_running
    }

    /// Get the node configuration
    pub fn config(&self) -> &NodeRunnerConfig {
        &self.config
    }

    /// Get the current node status
    pub async fn status(&self) -> Result<serde_json::Value, NodeRunnerError> {
        let is_running = *self.is_running.read().await;

        let dag_stats = {
            let dag = self.dag.read().await;
            let vertices = dag.vertices.read().await;
            serde_json::json!({
                "vertex_count": vertices.len(),
                "tips": 0, // TODO: Implement get_tips method
            })
        };

        let p2p_stats = if let Some(p2p_handle) = &self.p2p_handle {
            serde_json::json!({
                "peer_id": p2p_handle.local_peer_id().await.to_string(),
                "connected_peers": p2p_handle.connected_peers().await.len(),
            })
        } else {
            serde_json::Value::Null
        };

        Ok(serde_json::json!({
            "is_running": is_running,
            "dag": dag_stats,
            "p2p": p2p_stats,
            "dark_resolver_enabled": self.config.enable_dark_resolver,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_runner_creation() {
        let config = NodeRunnerConfig::default();
        let node_runner = NodeRunner::new(config);

        assert!(!*node_runner.is_running.read().await);
        assert!(node_runner.p2p_handle.is_none());
        assert!(node_runner.rpc_server.is_none());
    }

    #[tokio::test]
    async fn test_node_runner_start_stop() {
        let config = NodeRunnerConfig {
            rpc_transport: RpcTransport::Tcp("127.0.0.1:0".to_string()),
            ..Default::default()
        };

        let mut node_runner = NodeRunner::new(config);

        // Should be able to start
        assert!(node_runner.start().await.is_ok());
        assert!(*node_runner.is_running.read().await);

        // Should not be able to start again
        assert!(matches!(
            node_runner.start().await,
            Err(NodeRunnerError::AlreadyStarted)
        ));

        // Should be able to stop
        assert!(node_runner.stop().await.is_ok());
        assert!(!*node_runner.is_running.read().await);
    }

    #[tokio::test]
    async fn test_node_runner_status() {
        let config = NodeRunnerConfig::default();
        let node_runner = NodeRunner::new(config);

        let status = node_runner.status().await.unwrap();
        assert_eq!(status["is_running"], false);
        assert!(status["dag"].is_object());
        assert!(status["p2p"].is_null());
    }
}
