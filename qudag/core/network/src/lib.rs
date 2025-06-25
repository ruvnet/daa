#![allow(missing_docs)]

//! P2P networking layer with anonymous routing.
//!
//! This module provides the networking layer for the QuDAG protocol,
//! implementing anonymous routing, P2P communication, and traffic obfuscation.

pub mod circuit_breaker;
pub mod connection;
pub mod connection_pool;
pub mod dag_consensus;
pub mod dark_resolver;
pub mod discovery;
pub mod dns;
pub mod kademlia;
pub mod message;
pub mod metrics;
pub mod nat_traversal;
pub mod onion;
// Optimization features disabled for initial release
// pub mod optimized;
pub mod p2p;
pub mod peer;
pub mod quantum_crypto;
pub mod router;
pub mod routing;
pub mod shadow_address;
pub mod traffic_obfuscation;
pub mod transport;
pub mod types;
pub mod webrtc;

pub use dark_resolver::{DarkDomainRecord, DarkResolver, DarkResolverError};
pub use discovery::{
    DiscoveredPeer, DiscoveryConfig, DiscoveryEvent, DiscoveryMethod, DiscoveryStats,
    KademliaPeerDiscovery,
};
pub use dns::{CloudflareClient, CloudflareConfig, DnsError, DnsManager, DnsRecord, RecordType};
pub use kademlia::{BootstrapConfig, ContentRoutingConfig, KademliaDHT, PeerReputation};
pub use message::MessageEnvelope;
pub use nat_traversal::{
    ConnectionType, ConnectionUpgradeManager, HolePunchCoordinator, HolePunchPhase, NatInfo,
    NatPmpClient, NatTraversalConfig, NatTraversalError, NatTraversalManager, NatTraversalStats,
    NatType, PortMapping, PortMappingMethod, PortMappingProtocol, RelayConnection, RelayManager,
    RelayServer, StunClient, StunServer, TurnClient, TurnServer, UpgradeAttempt,
};
pub use onion::{
    Circuit, CircuitManager, CircuitState, CircuitStats, DirectoryClient, HopMetadata, LayerFlags,
    MLKEMOnionRouter, MetadataConfig, MetadataProtector, MixConfig, MixMessage, MixMessageType,
    MixNode, MixNodeStats, NodeFlags, NodeInfo, OnionError, OnionLayer, OnionRouter,
    ProtectedMetadata, TrafficAnalysisConfig, TrafficAnalysisResistance,
};
pub use p2p::{
    NetworkConfig as P2PNetworkConfig, P2PCommand, P2PEvent, P2PHandle, P2PNode, QuDagRequest,
    QuDagResponse,
};
pub use quantum_crypto::{
    MlKemCiphertext, MlKemPublicKey, MlKemSecretKey, MlKemSecurityLevel, QuantumKeyExchange,
    SharedSecret,
};
pub use router::{HopInfo, Router};
pub use shadow_address::{
    DefaultShadowAddressHandler, NetworkType, RotationPolicies, ShadowAddress, ShadowAddressError,
    ShadowAddressGenerator, ShadowAddressManager, ShadowAddressMixer, ShadowAddressPool,
    ShadowAddressResolver, ShadowFeatures, ShadowMetadata,
};
pub use traffic_obfuscation::{
    ObfuscationPattern, ObfuscationStats, TrafficObfuscationConfig, TrafficObfuscator,
    DEFAULT_MESSAGE_SIZE, STANDARD_MESSAGE_SIZES,
};
pub use transport::{AsyncTransport, Transport, TransportConfig, TransportError};
pub use types::{
    ConnectionStatus, LatencyMetrics, MessagePriority, NetworkAddress, NetworkError,
    NetworkMessage, PeerId, QueueMetrics, RoutingStrategy, ThroughputMetrics,
};
pub use webrtc::{
    WebRTCConfig, WebRTCTransport, TurnServerConfig, SignalingMessage, create_webrtc_transport,
};
pub use dag_consensus::{
    ConsensusMessage, ConsensusNetworkConfig, ConsensusNetworkEvent, ConsensusStats,
    DagConsensusNetwork, SerializedVertex, create_dag_consensus_network,
};

use libp2p::PeerId as LibP2PPeerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Comprehensive network manager for P2P operations
pub struct NetworkManager {
    /// Local peer ID
    local_peer_id: Option<LibP2PPeerId>,
    /// Connected peers
    connected_peers: Arc<RwLock<HashMap<LibP2PPeerId, PeerMetadata>>>,
    /// Message channel for inter-component communication
    message_tx: Option<mpsc::Sender<NetworkEvent>>,
    /// Network configuration
    config: NetworkConfig,
    /// Connection manager instance
    connection_manager: Option<Arc<ConnectionManager>>,
    /// Peer discovery service
    discovery_service: Option<Arc<dyn PeerDiscoveryService>>,
    /// Reputation manager
    reputation_manager: Arc<RwLock<ReputationManager>>,
    /// NAT traversal manager
    nat_traversal_manager: Option<Arc<NatTraversalManager>>,
}

/// Network configuration for the manager
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Maximum number of connections
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
    /// Discovery interval
    pub discovery_interval: std::time::Duration,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
    /// Enable DHT
    pub enable_dht: bool,
    /// Quantum-resistant mode
    pub quantum_resistant: bool,
    /// Enable NAT traversal
    pub enable_nat_traversal: bool,
    /// NAT traversal configuration
    pub nat_traversal_config: Option<NatTraversalConfig>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 50,
            connection_timeout: std::time::Duration::from_secs(30),
            discovery_interval: std::time::Duration::from_secs(60),
            bootstrap_peers: vec![],
            enable_dht: true,
            quantum_resistant: true,
            enable_nat_traversal: true,
            nat_traversal_config: None,
        }
    }
}

/// Peer metadata for tracking
#[derive(Debug, Clone)]
pub struct PeerMetadata {
    /// Peer address information
    pub address: String,
    /// Connection timestamp
    pub connected_at: std::time::Instant,
    /// Last activity timestamp
    pub last_activity: std::time::Instant,
    /// Reputation score
    pub reputation: f64,
    /// Protocol version
    pub protocol_version: u32,
    /// Connection quality metrics
    pub latency_ms: u64,
}

/// Network events for inter-component communication
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Peer connected
    PeerConnected(LibP2PPeerId),
    /// Peer disconnected
    PeerDisconnected(LibP2PPeerId),
    /// Message received
    MessageReceived { from: LibP2PPeerId, data: Vec<u8> },
    /// Discovery update
    DiscoveryUpdate(Vec<LibP2PPeerId>),
    /// Network error
    NetworkError(String),
}

/// Trait for peer discovery services
pub trait PeerDiscoveryService: Send + Sync {
    /// Start discovery service
    fn start(&self) -> Result<(), NetworkError>;
    /// Stop discovery service
    fn stop(&self) -> Result<(), NetworkError>;
    /// Get discovered peers
    fn get_peers(&self) -> Vec<LibP2PPeerId>;
    /// Add bootstrap peer
    fn add_bootstrap_peer(&mut self, peer: String) -> Result<(), NetworkError>;
}

/// Reputation management for peers
#[derive(Debug)]
pub struct ReputationManager {
    /// Peer reputation scores
    scores: HashMap<LibP2PPeerId, f64>,
    /// Blacklisted peers
    blacklist: HashMap<LibP2PPeerId, std::time::Instant>,
    /// Trusted peers
    trusted: HashMap<LibP2PPeerId, std::time::Instant>,
}

impl Default for ReputationManager {
    fn default() -> Self {
        Self {
            scores: HashMap::new(),
            blacklist: HashMap::new(),
            trusted: HashMap::new(),
        }
    }
}

impl ReputationManager {
    /// Get reputation score for a peer
    pub fn get_reputation(&self, peer_id: &LibP2PPeerId) -> f64 {
        self.scores.get(peer_id).copied().unwrap_or(0.0)
    }

    /// Update reputation score
    pub fn update_reputation(&mut self, peer_id: LibP2PPeerId, delta: f64) {
        let current = self.scores.get(&peer_id).copied().unwrap_or(0.0);
        let new_score = (current + delta).clamp(-100.0, 100.0);
        self.scores.insert(peer_id, new_score);

        // Auto-blacklist peers with very low reputation
        if new_score < -50.0 {
            self.blacklist.insert(peer_id, std::time::Instant::now());
            warn!(
                "Auto-blacklisted peer {:?} due to low reputation: {}",
                peer_id, new_score
            );
        }
    }

    /// Check if peer is blacklisted
    pub fn is_blacklisted(&self, peer_id: &LibP2PPeerId) -> bool {
        self.blacklist.contains_key(peer_id)
    }

    /// Add peer to trusted list
    pub fn add_trusted(&mut self, peer_id: LibP2PPeerId) {
        self.trusted.insert(peer_id, std::time::Instant::now());
        // Set high reputation for trusted peers
        self.scores.insert(peer_id, 75.0);
    }

    /// Check if peer is trusted
    pub fn is_trusted(&self, peer_id: &LibP2PPeerId) -> bool {
        self.trusted.contains_key(peer_id)
    }

    /// Remove expired blacklist entries (24 hours)
    pub fn cleanup_expired(&mut self) {
        let now = std::time::Instant::now();
        let expire_time = std::time::Duration::from_secs(24 * 60 * 60);

        self.blacklist
            .retain(|_, timestamp| now.duration_since(*timestamp) < expire_time);
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkManager {
    /// Create new network manager with default configuration
    pub fn new() -> Self {
        Self::with_config(NetworkConfig::default())
    }

    /// Create new network manager with custom configuration
    pub fn with_config(config: NetworkConfig) -> Self {
        Self {
            local_peer_id: None,
            connected_peers: Arc::new(RwLock::new(HashMap::new())),
            message_tx: None,
            config,
            connection_manager: None,
            discovery_service: None,
            reputation_manager: Arc::new(RwLock::new(ReputationManager::default())),
            nat_traversal_manager: None,
        }
    }

    /// Initialize the network manager
    pub async fn initialize(&mut self) -> Result<(), NetworkError> {
        // Generate or load peer identity
        self.local_peer_id = Some(LibP2PPeerId::random());

        // Set up message channel
        let (tx, mut rx) = mpsc::channel(1024);
        self.message_tx = Some(tx);

        // Initialize connection manager
        let connection_manager = Arc::new(ConnectionManager::new(self.config.max_connections));
        self.connection_manager = Some(connection_manager.clone());

        // Initialize NAT traversal if enabled
        if self.config.enable_nat_traversal {
            let nat_config = self.config.nat_traversal_config.clone().unwrap_or_default();
            let nat_manager = Arc::new(NatTraversalManager::new(
                nat_config,
                connection_manager.clone(),
            ));

            if let Err(e) = nat_manager.initialize().await {
                warn!("NAT traversal initialization failed: {}", e);
            } else {
                info!("NAT traversal initialized successfully");
            }

            self.nat_traversal_manager = Some(nat_manager);
        }

        // Start background event processing
        let connected_peers = Arc::clone(&self.connected_peers);
        let reputation_manager = Arc::clone(&self.reputation_manager);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                Self::handle_network_event(event, &connected_peers, &reputation_manager).await;
            }
        });

        info!(
            "NetworkManager initialized with peer ID: {:?}",
            self.local_peer_id
        );
        Ok(())
    }

    /// Handle network events in background task
    async fn handle_network_event(
        event: NetworkEvent,
        connected_peers: &Arc<RwLock<HashMap<LibP2PPeerId, PeerMetadata>>>,
        reputation_manager: &Arc<RwLock<ReputationManager>>,
    ) {
        match event {
            NetworkEvent::PeerConnected(peer_id) => {
                debug!("Handling peer connection: {:?}", peer_id);
                let metadata = PeerMetadata {
                    address: "unknown".to_string(),
                    connected_at: std::time::Instant::now(),
                    last_activity: std::time::Instant::now(),
                    reputation: 0.0,
                    protocol_version: 1,
                    latency_ms: 0,
                };
                connected_peers.write().await.insert(peer_id, metadata);
            }
            NetworkEvent::PeerDisconnected(peer_id) => {
                debug!("Handling peer disconnection: {:?}", peer_id);
                connected_peers.write().await.remove(&peer_id);
            }
            NetworkEvent::MessageReceived { from, data: _ } => {
                // Update last activity and reputation
                if let Some(metadata) = connected_peers.write().await.get_mut(&from) {
                    metadata.last_activity = std::time::Instant::now();
                }
                reputation_manager
                    .write()
                    .await
                    .update_reputation(from, 0.1);
            }
            NetworkEvent::NetworkError(error) => {
                error!("Network error: {}", error);
            }
            NetworkEvent::DiscoveryUpdate(peers) => {
                debug!("Discovery update: {} new peers found", peers.len());
            }
        }
    }

    /// Connect to a peer
    pub async fn connect_peer(&self, _peer_address: &str) -> Result<LibP2PPeerId, NetworkError> {
        let peer_id = LibP2PPeerId::random(); // In real implementation, derive from address

        // Check if peer is blacklisted
        if self
            .reputation_manager
            .read()
            .await
            .is_blacklisted(&peer_id)
        {
            return Err(NetworkError::ConnectionError(
                "Peer is blacklisted".to_string(),
            ));
        }

        // Convert LibP2PPeerId to our PeerId type for connection manager
        let peer_bytes = peer_id.to_bytes();
        let mut bytes_array = [0u8; 32];
        let len = peer_bytes.len().min(32);
        bytes_array[..len].copy_from_slice(&peer_bytes[..len]);
        let our_peer_id = crate::types::PeerId::from_bytes(bytes_array);

        // Try NAT traversal if available
        if let Some(nat_manager) = &self.nat_traversal_manager {
            match nat_manager.connect_peer(our_peer_id).await {
                Ok(()) => {
                    info!("Connected to peer {:?} via NAT traversal", peer_id);
                }
                Err(e) => {
                    warn!("NAT traversal failed for peer {:?}: {}", peer_id, e);
                    // Fall back to direct connection
                    if let Some(conn_mgr) = &self.connection_manager {
                        conn_mgr.connect(our_peer_id).await?;
                    }
                }
            }
        } else if let Some(conn_mgr) = &self.connection_manager {
            // Use regular connection manager
            conn_mgr.connect(our_peer_id).await?;
        }

        // Notify about new connection
        if let Some(tx) = &self.message_tx {
            let _ = tx.send(NetworkEvent::PeerConnected(peer_id)).await;
        }

        info!("Successfully connected to peer: {:?}", peer_id);
        Ok(peer_id)
    }

    /// Disconnect from a peer
    pub async fn disconnect_peer(&self, peer_id: &LibP2PPeerId) -> Result<(), NetworkError> {
        // Use connection manager to close connection
        if let Some(conn_mgr) = &self.connection_manager {
            let peer_bytes = peer_id.to_bytes();
            let mut bytes_array = [0u8; 32];
            let len = peer_bytes.len().min(32);
            bytes_array[..len].copy_from_slice(&peer_bytes[..len]);
            let our_peer_id = crate::types::PeerId::from_bytes(bytes_array);
            conn_mgr.disconnect(&our_peer_id);
        }

        // Notify about disconnection
        if let Some(tx) = &self.message_tx {
            let _ = tx.send(NetworkEvent::PeerDisconnected(*peer_id)).await;
        }

        info!("Disconnected from peer: {:?}", peer_id);
        Ok(())
    }

    /// Send message to a peer
    pub async fn send_message(
        &self,
        peer_id: &LibP2PPeerId,
        data: Vec<u8>,
    ) -> Result<(), NetworkError> {
        // Check if peer is connected
        if !self.connected_peers.read().await.contains_key(peer_id) {
            return Err(NetworkError::ConnectionError(
                "Peer not connected".to_string(),
            ));
        }

        // In real implementation, this would use the actual transport layer
        debug!("Sending {} bytes to peer {:?}", data.len(), peer_id);

        // Update peer activity
        if let Some(metadata) = self.connected_peers.write().await.get_mut(peer_id) {
            metadata.last_activity = std::time::Instant::now();
        }

        Ok(())
    }

    /// Get list of connected peers
    pub async fn get_connected_peers(&self) -> Vec<LibP2PPeerId> {
        self.connected_peers.read().await.keys().cloned().collect()
    }

    /// Get peer metadata
    pub async fn get_peer_metadata(&self, peer_id: &LibP2PPeerId) -> Option<PeerMetadata> {
        self.connected_peers.read().await.get(peer_id).cloned()
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let connected_count = self.connected_peers.read().await.len();
        let reputation_scores: Vec<f64> = {
            let rep_mgr = self.reputation_manager.read().await;
            rep_mgr.scores.values().cloned().collect()
        };

        let avg_reputation = if reputation_scores.is_empty() {
            0.0
        } else {
            reputation_scores.iter().sum::<f64>() / reputation_scores.len() as f64
        };

        NetworkStats {
            connected_peers: connected_count,
            average_reputation: avg_reputation,
            blacklisted_peers: self.reputation_manager.read().await.blacklist.len(),
            trusted_peers: self.reputation_manager.read().await.trusted.len(),
        }
    }

    /// Add peer to trusted list
    pub async fn add_trusted_peer(&self, peer_id: LibP2PPeerId) {
        self.reputation_manager.write().await.add_trusted(peer_id);
        info!("Added trusted peer: {:?}", peer_id);
    }

    /// Blacklist a peer
    pub async fn blacklist_peer(&self, peer_id: LibP2PPeerId) {
        self.reputation_manager
            .write()
            .await
            .update_reputation(peer_id, -100.0);

        // Disconnect if currently connected
        let _ = self.disconnect_peer(&peer_id).await;

        warn!("Blacklisted peer: {:?}", peer_id);
    }

    /// Start peer discovery
    pub async fn start_discovery(&mut self) -> Result<(), NetworkError> {
        // TODO: Initialize and start discovery service
        info!("Starting peer discovery");
        Ok(())
    }

    /// Stop the network manager
    pub async fn shutdown(&mut self) -> Result<(), NetworkError> {
        info!("Shutting down NetworkManager");

        // Disconnect all peers
        let peers: Vec<_> = self.get_connected_peers().await;
        for peer_id in peers {
            let _ = self.disconnect_peer(&peer_id).await;
        }

        // Stop discovery service
        if let Some(discovery) = &self.discovery_service {
            discovery.stop()?;
        }

        // Shutdown NAT traversal
        if let Some(nat_manager) = &self.nat_traversal_manager {
            if let Err(e) = nat_manager.shutdown().await {
                warn!("NAT traversal shutdown error: {}", e);
            }
        }

        Ok(())
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> Option<LibP2PPeerId> {
        self.local_peer_id
    }

    /// Perform maintenance tasks
    pub async fn maintenance(&self) {
        // Cleanup expired blacklist entries
        self.reputation_manager.write().await.cleanup_expired();

        // Remove inactive peers (older than 5 minutes with no activity)
        let now = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(300);

        let mut to_disconnect = Vec::new();
        {
            let peers = self.connected_peers.read().await;
            for (peer_id, metadata) in peers.iter() {
                if now.duration_since(metadata.last_activity) > timeout {
                    to_disconnect.push(*peer_id);
                }
            }
        }

        for peer_id in to_disconnect {
            warn!("Disconnecting inactive peer: {:?}", peer_id);
            let _ = self.disconnect_peer(&peer_id).await;
        }
    }

    /// Get NAT information
    pub fn get_nat_info(&self) -> Option<NatInfo> {
        self.nat_traversal_manager.as_ref()?.get_nat_info()
    }

    /// Create port mapping
    pub async fn create_port_mapping(
        &self,
        local_port: u16,
        external_port: u16,
        protocol: crate::nat_traversal::PortMappingProtocol,
    ) -> Result<PortMapping, NetworkError> {
        if let Some(nat_manager) = &self.nat_traversal_manager {
            nat_manager
                .create_port_mapping(local_port, external_port, protocol)
                .await
                .map_err(|e| NetworkError::ConnectionError(e.to_string()))
        } else {
            Err(NetworkError::ConnectionError(
                "NAT traversal not enabled".to_string(),
            ))
        }
    }

    /// Get NAT traversal statistics
    pub fn get_nat_traversal_stats(&self) -> Option<NatTraversalStats> {
        self.nat_traversal_manager.as_ref().map(|m| m.get_stats())
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    /// Number of connected peers
    pub connected_peers: usize,
    /// Average reputation score
    pub average_reputation: f64,
    /// Number of blacklisted peers
    pub blacklisted_peers: usize,
    /// Number of trusted peers
    pub trusted_peers: usize,
}
pub use circuit_breaker::{CircuitBreaker, CircuitState as CircuitBreakerState};
pub use connection::{
    ConnectionInfo, ConnectionManager, HealthStatistics, SecureConfig, SecureConnection,
    TransportKeys, UnhealthyConnectionInfo,
};
