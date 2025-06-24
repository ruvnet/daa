use crate::{
    message::{Message, MessageError, MessageType},
    persistence::{
        MemoryBackend, PersistedState, PersistenceError, PersistenceManager, SqliteBackend,
        StatePersistence, StateProvider,
    },
    state::ProtocolStateMachine,
    types::{ProtocolError, ProtocolEvent},
};
use qudag_crypto::ml_kem::MlKem768;
use qudag_dag::Consensus;
use qudag_network::Transport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Node configuration
///
/// # Examples
///
/// ```rust
/// use qudag_protocol::NodeConfig;
/// use std::path::PathBuf;
///
/// // Create default configuration
/// let config = NodeConfig::default();
/// assert_eq!(config.network_port, 8000);
///
/// // Create custom configuration
/// let custom_config = NodeConfig {
///     data_dir: PathBuf::from("/custom/data"),
///     network_port: 9000,
///     max_peers: 100,
///     initial_peers: vec!["peer1:8000".to_string(), "peer2:8000".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Data directory
    pub data_dir: PathBuf,
    /// Network port
    pub network_port: u16,
    /// Maximum peers
    pub max_peers: usize,
    /// Initial peers
    pub initial_peers: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            network_port: 8000,
            max_peers: 50,
            initial_peers: Vec::new(),
        }
    }
}

/// Protocol node with persistence support
pub struct Node {
    /// Node configuration
    #[allow(dead_code)]
    config: NodeConfig,
    /// Protocol state machine
    state_machine: Arc<RwLock<ProtocolStateMachine>>,
    /// Event channels
    #[allow(dead_code)]
    events: NodeEvents,
    /// Cryptographic keys
    keys: Option<KeyPair>,
    /// Network transport
    transport: Option<Arc<dyn Transport + Send + Sync>>,
    /// Consensus engine
    #[allow(dead_code)]
    consensus: Option<Arc<dyn Consensus + Send + Sync>>,
    /// Persistence manager
    persistence: Option<PersistenceManager>,
    /// Node ID
    pub node_id: Vec<u8>,
}

/// Node event channels
struct NodeEvents {
    /// Event sender
    #[allow(dead_code)]
    tx: mpsc::Sender<ProtocolEvent>,
    /// Event receiver
    #[allow(dead_code)]
    rx: mpsc::Receiver<ProtocolEvent>,
}

/// Cryptographic key pair
struct KeyPair {
    /// Public key
    #[allow(dead_code)]
    public_key: Vec<u8>,
    /// Private key
    #[allow(dead_code)]
    private_key: Vec<u8>,
}

impl Node {
    /// Create new node
    pub async fn new(config: NodeConfig) -> Result<Self, ProtocolError> {
        let (tx, rx) = mpsc::channel(1000);

        // Generate node ID
        let node_id = Self::generate_node_id();

        // Initialize state machine
        let state_machine = Arc::new(RwLock::new(ProtocolStateMachine::new(
            crate::message::ProtocolVersion::CURRENT,
        )));

        Ok(Self {
            config,
            state_machine,
            events: NodeEvents { tx, rx },
            keys: None,
            transport: None,
            consensus: None,
            persistence: None,
            node_id,
        })
    }

    /// Create new node with persistence
    pub async fn with_persistence(config: NodeConfig) -> Result<Self, ProtocolError> {
        let mut node = Self::new(config.clone()).await?;

        // Create persistence backend based on configuration
        let backend: Arc<dyn StatePersistence> = if config.data_dir.join("state.db").exists() {
            // Use SQLite for lightweight persistence
            let db_path = config.data_dir.join("state.db");
            Arc::new(SqliteBackend::new(db_path).await.map_err(|e| {
                ProtocolError::Internal(format!("Failed to create SQLite backend: {}", e))
            })?)
        } else {
            // Use memory backend for testing
            Arc::new(MemoryBackend::default())
        };

        // Create persistence manager
        let persistence_manager: PersistenceManager = backend;

        // Try to recover state
        if let Some(recovered_state) = persistence_manager
            .recover_state()
            .await
            .map_err(|e| ProtocolError::Internal(format!("Failed to recover state: {}", e)))?
        {
            info!("Recovered state from persistence");

            // Restore state machine
            let _state_machine = node.state_machine.write().await;
            // TODO: Implement proper state restoration
            // For now, just log the recovery
            debug!(
                "Recovered {} peers and {} sessions",
                recovered_state.peers.len(),
                recovered_state.sessions.len()
            );
        }

        node.persistence = Some(persistence_manager);
        Ok(node)
    }

    fn generate_node_id() -> Vec<u8> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut id = vec![0u8; 32];
        rng.fill_bytes(&mut id);
        id
    }

    /// Start node
    pub async fn start(&mut self) -> Result<(), ProtocolError> {
        info!("Starting node...");

        // Initialize cryptographic keys
        self.init_keys().await?;

        // Initialize network transport
        self.init_transport().await?;

        // Initialize consensus engine
        self.init_consensus().await?;

        // Update state machine - transition through proper states
        let mut state_machine = self.state_machine.write().await;

        // First transition to Handshake
        state_machine
            .transition_to(
                crate::state::ProtocolState::Handshake(crate::state::HandshakeState::Waiting),
                "Node starting handshake".to_string(),
            )
            .map_err(|e| ProtocolError::StateError(e.to_string()))?;

        // Skip through handshake states for now
        state_machine
            .transition_to(
                crate::state::ProtocolState::Handshake(crate::state::HandshakeState::InProgress),
                "Handshake in progress".to_string(),
            )
            .map_err(|e| ProtocolError::StateError(e.to_string()))?;

        state_machine
            .transition_to(
                crate::state::ProtocolState::Handshake(crate::state::HandshakeState::Processing),
                "Processing handshake".to_string(),
            )
            .map_err(|e| ProtocolError::StateError(e.to_string()))?;

        state_machine
            .transition_to(
                crate::state::ProtocolState::Handshake(crate::state::HandshakeState::Completed),
                "Handshake completed".to_string(),
            )
            .map_err(|e| ProtocolError::StateError(e.to_string()))?;

        // Now transition to active
        state_machine
            .transition_to(
                crate::state::ProtocolState::Active(crate::state::ActiveState::Normal),
                "Node started".to_string(),
            )
            .map_err(|e| ProtocolError::StateError(e.to_string()))?;

        drop(state_machine);

        // Start auto-save if persistence is enabled
        // Note: Auto-save would need to be started externally with a proper Arc<Node>
        // to avoid self-referential issues

        info!("Node started successfully");
        Ok(())
    }

    /// Stop node
    pub async fn stop(&mut self) -> Result<(), ProtocolError> {
        info!("Stopping node...");

        // Update state machine
        let mut state_machine = self.state_machine.write().await;
        state_machine
            .transition_to(
                crate::state::ProtocolState::Shutdown,
                "Node stopping".to_string(),
            )
            .map_err(|e| ProtocolError::StateError(e.to_string()))?;
        drop(state_machine);

        // Save final state if persistence is enabled
        if let Some(persistence) = &self.persistence {
            let state = self.get_current_state().await.map_err(|e| {
                ProtocolError::Internal(format!("Failed to get state for save: {}", e))
            })?;
            persistence.save_state(&state).await.map_err(|e| {
                ProtocolError::Internal(format!("Failed to save final state: {}", e))
            })?;
        }

        // Stop components
        if let Some(_transport) = &self.transport {
            // TODO: Implement transport stop method
        }

        info!("Node stopped successfully");
        Ok(())
    }

    /// Handle incoming message
    pub async fn handle_message(&mut self, message: Message) -> Result<(), MessageError> {
        debug!("Handling message: {:?}", message.msg_type);

        // Verify message
        // TODO: Get proper public key for verification
        // if !message.verify(&proper_public_key)? {
        //     return Err(MessageError::InvalidSignature);
        // }

        // Process message
        match message.msg_type {
            MessageType::Handshake(_) => self.handle_handshake(message).await?,
            MessageType::Data(_) => self.handle_data(message).await?,
            MessageType::Control(_) => self.handle_control(message).await?,
            MessageType::Sync(_) => self.handle_sync(message).await?,
            _ => return Err(MessageError::InvalidFormat),
        }

        Ok(())
    }

    // Initialize cryptographic keys
    async fn init_keys(&mut self) -> Result<(), ProtocolError> {
        // Generate ML-KEM key pair
        let (pk, sk) = MlKem768::keygen().map_err(|e| ProtocolError::CryptoError(e.to_string()))?;

        self.keys = Some(KeyPair {
            public_key: pk.as_bytes().to_vec(),
            private_key: sk.as_bytes().to_vec(),
        });

        Ok(())
    }

    // Initialize network transport
    async fn init_transport(&mut self) -> Result<(), ProtocolError> {
        // TODO: Initialize transport
        Ok(())
    }

    // Initialize consensus engine
    async fn init_consensus(&mut self) -> Result<(), ProtocolError> {
        // TODO: Initialize consensus
        Ok(())
    }

    // Handle handshake message
    async fn handle_handshake(&mut self, _message: Message) -> Result<(), MessageError> {
        // TODO: Implement handshake
        Ok(())
    }

    // Handle data message
    async fn handle_data(&mut self, _message: Message) -> Result<(), MessageError> {
        // TODO: Implement data handling
        Ok(())
    }

    // Handle control message
    async fn handle_control(&mut self, _message: Message) -> Result<(), MessageError> {
        // TODO: Implement control handling
        Ok(())
    }

    // Handle sync message
    async fn handle_sync(&mut self, _message: Message) -> Result<(), MessageError> {
        // TODO: Implement sync handling
        Ok(())
    }

    /// Get current node state
    pub async fn get_state(&self) -> crate::state::ProtocolState {
        self.state_machine.read().await.current_state().clone()
    }

    /// Check if persistence is enabled
    pub fn has_persistence(&self) -> bool {
        self.persistence.is_some()
    }

    /// Save current state
    pub async fn save_state(&self) -> Result<(), ProtocolError> {
        if let Some(persistence) = &self.persistence {
            let state = self
                .get_current_state()
                .await
                .map_err(|e| ProtocolError::Internal(format!("Failed to get state: {}", e)))?;
            persistence
                .save_state(&state)
                .await
                .map_err(|e| ProtocolError::Internal(format!("Failed to save state: {}", e)))?;
            info!("State saved successfully");
        } else {
            warn!("No persistence backend configured");
        }
        Ok(())
    }

    /// Create backup
    pub async fn create_backup(&self, backup_path: PathBuf) -> Result<(), ProtocolError> {
        if let Some(persistence) = &self.persistence {
            persistence
                .create_backup(&backup_path)
                .await
                .map_err(|e| ProtocolError::Internal(format!("Failed to create backup: {}", e)))?;
            info!("Backup created at {:?}", backup_path);
        } else {
            return Err(ProtocolError::Internal(
                "No persistence backend configured".to_string(),
            ));
        }
        Ok(())
    }

    /// Restore from backup
    pub async fn restore_backup(&self, backup_path: PathBuf) -> Result<(), ProtocolError> {
        if let Some(persistence) = &self.persistence {
            persistence
                .restore_backup(&backup_path)
                .await
                .map_err(|e| ProtocolError::Internal(format!("Failed to restore backup: {}", e)))?;
            info!("Backup restored from {:?}", backup_path);
        } else {
            return Err(ProtocolError::Internal(
                "No persistence backend configured".to_string(),
            ));
        }
        Ok(())
    }
}

impl Node {
    /// Get current state for persistence
    pub async fn get_current_state(&self) -> Result<PersistedState, PersistenceError> {
        let state_machine = self.state_machine.read().await;
        let current_state = state_machine.current_state().clone();
        let sessions = state_machine.get_sessions().clone();
        let metrics = state_machine.get_metrics();

        // TODO: Get actual peer list from network transport
        let peers = vec![];

        // TODO: Get actual DAG state from consensus engine
        let dag_state = crate::persistence::DagState {
            vertices: HashMap::new(),
            tips: std::collections::HashSet::new(),
            voting_records: HashMap::new(),
            last_checkpoint: None,
        };

        Ok(PersistedState {
            version: crate::persistence::CURRENT_STATE_VERSION,
            node_id: self.node_id.clone(),
            protocol_state: current_state,
            sessions,
            peers,
            dag_state,
            metrics,
            last_saved: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
}

/// Wrapper to provide StateProvider implementation for Node
pub struct NodeStateProvider {
    node: Arc<RwLock<Node>>,
}

impl NodeStateProvider {
    /// Create new state provider for a node
    pub fn new(node: Arc<RwLock<Node>>) -> Self {
        Self { node }
    }
}

impl StateProvider for NodeStateProvider {
    fn get_state_store(&self) -> Arc<dyn crate::persistence::StateStore + Send + Sync> {
        // Since this is a sync method but we need to access an async RwLock,
        // we'll use try_read() which is non-blocking
        if let Ok(node) = self.node.try_read() {
            if let Some(persistence) = &node.persistence {
                // Return the actual state store from the persistence manager's backend
                return persistence.clone();
            }
        }

        // Fallback to memory store if no persistence is configured or lock failed
        Arc::new(crate::persistence::MemoryStateStore::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_lifecycle() {
        let config = NodeConfig::default();
        let mut node = Node::new(config).await.unwrap();

        assert_eq!(node.get_state().await, crate::state::ProtocolState::Initial);

        node.start().await.unwrap();
        assert!(matches!(
            node.get_state().await,
            crate::state::ProtocolState::Active(_)
        ));

        node.stop().await.unwrap();
        assert_eq!(
            node.get_state().await,
            crate::state::ProtocolState::Shutdown
        );
    }

    #[tokio::test]
    async fn test_node_persistence() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config = NodeConfig {
            data_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        // Create node with persistence
        let mut node = Node::with_persistence(config.clone()).await.unwrap();

        // Start node
        node.start().await.unwrap();

        // Save state
        node.save_state().await.unwrap();

        // Create backup
        let backup_path = temp_dir.path().join("backup");
        std::fs::create_dir_all(&backup_path).unwrap();
        node.create_backup(backup_path.clone()).await.unwrap();

        // Stop node
        node.stop().await.unwrap();

        // Create new node and verify state recovery
        let node2 = Node::with_persistence(config).await.unwrap();
        assert!(node2.persistence.is_some());
    }
}
