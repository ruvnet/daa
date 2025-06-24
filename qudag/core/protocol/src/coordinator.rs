//! Protocol coordinator implementation.
//!
//! The coordinator provides a high-level interface for managing protocol operations,
//! integrating crypto, network, and DAG components.

use crate::{
    config::Config as ProtocolConfig,
    message::Message,
    node::{Node, NodeConfig},
    types::{ProtocolError, ProtocolEvent, ProtocolState},
};
use qudag_crypto::KeyPair;
use qudag_dag::QrDag;
use qudag_network::NetworkManager;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

/// Protocol coordinator
pub struct Coordinator {
    /// Internal node instance
    node: Node,
    /// Protocol configuration  
    #[allow(dead_code)]
    config: ProtocolConfig,
    /// Current state
    state: Arc<RwLock<ProtocolState>>,
    /// Event channels
    #[allow(dead_code)]
    events: CoordinatorEvents,
    /// Crypto manager
    crypto: Option<KeyPair>,
    /// Network manager
    network: Option<NetworkManager>,
    /// DAG manager
    dag: Option<QrDag>,
}

/// Coordinator event channels
struct CoordinatorEvents {
    /// Event sender
    #[allow(dead_code)]
    tx: mpsc::Sender<ProtocolEvent>,
    /// Event receiver  
    #[allow(dead_code)]
    rx: mpsc::Receiver<ProtocolEvent>,
}

impl Coordinator {
    /// Create new coordinator
    pub async fn new(config: ProtocolConfig) -> Result<Self, ProtocolError> {
        let node_config = NodeConfig {
            data_dir: config.node.data_dir.clone(),
            network_port: config.network.port,
            max_peers: config.network.max_peers,
            initial_peers: Vec::new(),
        };

        let node = Node::new(node_config).await?;
        let (tx, rx) = mpsc::channel(1000);

        Ok(Self {
            node,
            config,
            state: Arc::new(RwLock::new(ProtocolState::Initial)),
            events: CoordinatorEvents { tx, rx },
            crypto: None,
            network: None,
            dag: None,
        })
    }

    /// Start coordinator
    pub async fn start(&mut self) -> Result<(), ProtocolError> {
        info!("Starting protocol coordinator...");

        // Initialize components
        self.init_crypto().await?;
        self.init_network().await?;
        self.init_dag().await?;

        // Start internal node
        self.node.start().await?;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ProtocolState::Running;
        }

        info!("Protocol coordinator started successfully");
        Ok(())
    }

    /// Stop coordinator
    pub async fn stop(&mut self) -> Result<(), ProtocolError> {
        info!("Stopping protocol coordinator...");

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ProtocolState::Stopping;
        }

        // Stop internal node
        self.node.stop().await?;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ProtocolState::Stopped;
        }

        info!("Protocol coordinator stopped successfully");
        Ok(())
    }

    /// Get current state
    pub async fn state(&self) -> ProtocolState {
        self.state.read().await.clone()
    }

    /// Check if coordinator is initialized
    pub fn is_initialized(&self) -> bool {
        self.crypto.is_some() && self.network.is_some() && self.dag.is_some()
    }

    /// Broadcast message
    pub async fn broadcast_message(&mut self, message: Vec<u8>) -> Result<(), ProtocolError> {
        debug!("Broadcasting message of {} bytes", message.len());

        // Create protocol message
        let proto_message =
            Message::new(crate::message::MessageType::Data(message.clone()), vec![]);

        // Sign message if crypto is available
        if let Some(ref _crypto) = self.crypto {
            // TODO: Use proper keypair for signing
            // proto_message.sign(&proper_keypair).map_err(|e| ProtocolError::CryptoError(e.to_string()))?;
        }

        // Handle message through node
        self.node
            .handle_message(proto_message)
            .await
            .map_err(|e| ProtocolError::Internal(e.to_string()))?;

        // Add to DAG if available
        if let Some(ref mut dag) = self.dag {
            dag.add_message(message)
                .map_err(|e| ProtocolError::Internal(e.to_string()))?;
        }

        Ok(())
    }

    /// Get crypto manager
    pub fn crypto_manager(&self) -> Option<&KeyPair> {
        self.crypto.as_ref()
    }

    /// Get network manager
    pub fn network_manager(&self) -> Option<&NetworkManager> {
        self.network.as_ref()
    }

    /// Get DAG manager
    pub fn dag_manager(&self) -> Option<&QrDag> {
        self.dag.as_ref()
    }

    // Initialize crypto components
    async fn init_crypto(&mut self) -> Result<(), ProtocolError> {
        // TODO: Initialize proper crypto manager
        // For now, create a placeholder
        self.crypto = Some(KeyPair::new());
        Ok(())
    }

    // Initialize network components
    async fn init_network(&mut self) -> Result<(), ProtocolError> {
        // TODO: Initialize proper network manager
        // For now, create a placeholder
        self.network = Some(NetworkManager::new());
        Ok(())
    }

    // Initialize DAG components
    async fn init_dag(&mut self) -> Result<(), ProtocolError> {
        // TODO: Initialize proper DAG manager
        // For now, create a placeholder
        self.dag = Some(QrDag::new());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_lifecycle() {
        let config = ProtocolConfig::default();
        let mut coordinator = Coordinator::new(config).await.unwrap();

        assert_eq!(coordinator.state().await, ProtocolState::Initial);

        coordinator.start().await.unwrap();
        assert_eq!(coordinator.state().await, ProtocolState::Running);

        coordinator.stop().await.unwrap();
        assert_eq!(coordinator.state().await, ProtocolState::Stopped);
    }

    #[tokio::test]
    async fn test_coordinator_initialization() {
        let config = ProtocolConfig::default();
        let coordinator = Coordinator::new(config).await.unwrap();

        // Initially not initialized
        assert!(!coordinator.is_initialized());
    }
}
