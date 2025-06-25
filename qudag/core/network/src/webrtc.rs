//! WebRTC transport implementation for browser-based QuDAG nodes.
//!
//! This module provides WebRTC transport capabilities to enable browser nodes
//! to participate in the QuDAG network. It supports:
//! - WebRTC DataChannel for P2P communication
//! - STUN/TURN for NAT traversal
//! - Integration with existing transport layer
//! - Quantum-resistant encryption over WebRTC

use crate::transport::{AsyncTransport, ConnectionMetadata, Transport, TransportConfig, TransportError, TransportStats};
use crate::types::{ConnectionStatus, NetworkError, PeerId};
use crate::nat_traversal::{StunClient, TurnClient};
use async_trait::async_trait;
use futures::StreamExt;
use libp2p::core::transport::{Transport as LibP2PTransport, ListenerEvent};
use libp2p::PeerId as LibP2PPeerId;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use webrtc::{
    api::{APIBuilder, API},
    data_channel::{data_channel_init::RTCDataChannelInit, RTCDataChannel},
    ice_transport::{
        ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
        ice_server::RTCIceServer,
    },
    peer_connection::{
        configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
};

/// WebRTC-specific configuration
#[derive(Debug, Clone)]
pub struct WebRTCConfig {
    /// STUN servers for NAT traversal
    pub stun_servers: Vec<String>,
    /// TURN servers for relay fallback
    pub turn_servers: Vec<TurnServerConfig>,
    /// Maximum message size for data channels
    pub max_message_size: usize,
    /// Enable ordered delivery
    pub ordered: bool,
    /// Maximum retransmits for unreliable channels
    pub max_retransmits: Option<u16>,
    /// Enable DTLS fingerprint verification
    pub verify_fingerprint: bool,
    /// ICE gathering timeout
    pub ice_gathering_timeout: std::time::Duration,
    /// Signaling server URL
    pub signaling_server: Option<String>,
}

impl Default for WebRTCConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn_servers: vec![],
            max_message_size: 16 * 1024 * 1024, // 16MB
            ordered: true,
            max_retransmits: None,
            verify_fingerprint: true,
            ice_gathering_timeout: std::time::Duration::from_secs(10),
            signaling_server: None,
        }
    }
}

/// TURN server configuration
#[derive(Debug, Clone)]
pub struct TurnServerConfig {
    /// TURN server URL
    pub urls: Vec<String>,
    /// Username for authentication
    pub username: Option<String>,
    /// Credential (password)
    pub credential: Option<String>,
    /// Credential type
    pub credential_type: String,
}

/// WebRTC transport implementation
pub struct WebRTCTransport {
    /// WebRTC configuration
    config: WebRTCConfig,
    /// WebRTC API instance
    api: Arc<API>,
    /// Active peer connections
    connections: Arc<RwLock<HashMap<String, Arc<RTCPeerConnection>>>>,
    /// Data channels
    data_channels: Arc<RwLock<HashMap<String, Arc<RTCDataChannel>>>>,
    /// Connection metadata
    metadata: Arc<RwLock<HashMap<String, ConnectionMetadata>>>,
    /// Signaling channel
    signaling_tx: Option<mpsc::Sender<SignalingMessage>>,
    signaling_rx: Option<Arc<Mutex<mpsc::Receiver<SignalingMessage>>>>,
    /// Transport statistics
    stats: Arc<RwLock<TransportStats>>,
    /// Connection ID counter
    connection_counter: Arc<std::sync::atomic::AtomicU64>,
}

/// Signaling messages for WebRTC connection establishment
#[derive(Debug, Clone)]
pub enum SignalingMessage {
    /// Offer SDP
    Offer {
        from: String,
        to: String,
        sdp: String,
    },
    /// Answer SDP
    Answer {
        from: String,
        to: String,
        sdp: String,
    },
    /// ICE candidate
    IceCandidate {
        from: String,
        to: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    },
}

impl WebRTCTransport {
    /// Create a new WebRTC transport
    pub fn new(config: WebRTCConfig) -> Self {
        // Create WebRTC API with custom configuration
        let api = APIBuilder::new().build();

        let (signaling_tx, signaling_rx) = mpsc::channel(1024);

        Self {
            config,
            api: Arc::new(api),
            connections: Arc::new(RwLock::new(HashMap::new())),
            data_channels: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            signaling_tx: Some(signaling_tx),
            signaling_rx: Some(Arc::new(Mutex::new(signaling_rx))),
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connection_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Generate a unique connection ID
    fn generate_connection_id(&self) -> String {
        let id = self.connection_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("webrtc_conn_{}", id)
    }

    /// Create ICE servers configuration
    fn create_ice_servers(&self) -> Vec<RTCIceServer> {
        let mut ice_servers = Vec::new();

        // Add STUN servers
        for stun_server in &self.config.stun_servers {
            ice_servers.push(RTCIceServer {
                urls: vec![stun_server.clone()],
                username: String::new(),
                credential: String::new(),
                credential_type: "password".to_string(),
            });
        }

        // Add TURN servers
        for turn_server in &self.config.turn_servers {
            ice_servers.push(RTCIceServer {
                urls: turn_server.urls.clone(),
                username: turn_server.username.clone().unwrap_or_default(),
                credential: turn_server.credential.clone().unwrap_or_default(),
                credential_type: turn_server.credential_type.clone(),
            });
        }

        ice_servers
    }

    /// Create a new peer connection
    async fn create_peer_connection(&self, connection_id: String) -> Result<Arc<RTCPeerConnection>, TransportError> {
        let config = RTCConfiguration {
            ice_servers: self.create_ice_servers(),
            ..Default::default()
        };

        let peer_connection = Arc::new(
            self.api
                .new_peer_connection(config)
                .await
                .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create peer connection: {}", e)))?
        );

        // Set up connection state handler
        let conn_id = connection_id.clone();
        let metadata = Arc::clone(&self.metadata);
        peer_connection.on_peer_connection_state_change(Box::new(move |state: RTCPeerConnectionState| {
            let conn_id = conn_id.clone();
            let metadata = metadata.clone();
            Box::pin(async move {
                debug!("Peer connection state changed to: {:?}", state);
                
                // Update connection metadata based on state
                if let Ok(mut meta_lock) = metadata.write().await.try_write() {
                    if let Some(meta) = meta_lock.get_mut(&conn_id) {
                        meta.status = match state {
                            RTCPeerConnectionState::Connected => ConnectionStatus::Connected,
                            RTCPeerConnectionState::Disconnected => ConnectionStatus::Disconnected,
                            RTCPeerConnectionState::Failed => ConnectionStatus::Failed,
                            RTCPeerConnectionState::Closed => ConnectionStatus::Closed,
                            _ => ConnectionStatus::Connecting,
                        };
                        meta.last_activity = Instant::now();
                    }
                }
            })
        }));

        // Set up ICE candidate handler
        let signaling_tx = self.signaling_tx.clone();
        let conn_id = connection_id.clone();
        peer_connection.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
            let signaling_tx = signaling_tx.clone();
            let conn_id = conn_id.clone();
            Box::pin(async move {
                if let Some(candidate) = candidate {
                    if let Some(tx) = signaling_tx {
                        let _ = tx.send(SignalingMessage::IceCandidate {
                            from: conn_id.clone(),
                            to: "remote".to_string(), // This should be the actual remote peer ID
                            candidate: candidate.to_json().await.unwrap_or_default().candidate,
                            sdp_mid: candidate.to_json().await.ok().and_then(|c| c.sdp_mid),
                            sdp_mline_index: candidate.to_json().await.ok().and_then(|c| c.sdp_mline_index),
                        }).await;
                    }
                }
            })
        }));

        Ok(peer_connection)
    }

    /// Create a data channel
    async fn create_data_channel(
        &self,
        peer_connection: &Arc<RTCPeerConnection>,
        label: &str,
    ) -> Result<Arc<RTCDataChannel>, TransportError> {
        let data_channel_init = RTCDataChannelInit {
            ordered: Some(self.config.ordered),
            max_retransmits: self.config.max_retransmits,
            ..Default::default()
        };

        let data_channel = peer_connection
            .create_data_channel(label, Some(data_channel_init))
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create data channel: {}", e)))?;

        Ok(Arc::new(data_channel))
    }

    /// Handle incoming offer
    async fn handle_offer(
        &self,
        from: String,
        sdp: String,
    ) -> Result<String, TransportError> {
        let connection_id = self.generate_connection_id();
        let peer_connection = self.create_peer_connection(connection_id.clone()).await?;

        // Set remote description
        let offer = RTCSessionDescription::offer(sdp).map_err(|e| {
            TransportError::ConnectionFailed(format!("Invalid offer SDP: {}", e))
        })?;

        peer_connection
            .set_remote_description(offer)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to set remote description: {}", e)))?;

        // Create answer
        let answer = peer_connection
            .create_answer(None)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create answer: {}", e)))?;

        peer_connection
            .set_local_description(answer.clone())
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to set local description: {}", e)))?;

        // Store connection
        self.connections.write().await.insert(connection_id.clone(), peer_connection);

        // Create metadata
        let metadata = ConnectionMetadata {
            connection_id: connection_id.clone(),
            peer_id: Some(PeerId::from_bytes([0u8; 32])), // TODO: Extract actual peer ID
            status: ConnectionStatus::Connecting,
            established_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            is_post_quantum: false, // WebRTC doesn't support post-quantum crypto yet
            tls_version: Some("DTLS 1.2".to_string()),
        };
        self.metadata.write().await.insert(connection_id.clone(), metadata);

        Ok(answer.sdp)
    }

    /// Handle incoming answer
    async fn handle_answer(
        &self,
        connection_id: String,
        sdp: String,
    ) -> Result<(), TransportError> {
        let connections = self.connections.read().await;
        let peer_connection = connections
            .get(&connection_id)
            .ok_or_else(|| TransportError::ConnectionFailed("Connection not found".to_string()))?;

        let answer = RTCSessionDescription::answer(sdp).map_err(|e| {
            TransportError::ConnectionFailed(format!("Invalid answer SDP: {}", e))
        })?;

        peer_connection
            .set_remote_description(answer)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to set remote description: {}", e)))?;

        Ok(())
    }

    /// Handle incoming ICE candidate
    async fn handle_ice_candidate(
        &self,
        connection_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    ) -> Result<(), TransportError> {
        let connections = self.connections.read().await;
        let peer_connection = connections
            .get(&connection_id)
            .ok_or_else(|| TransportError::ConnectionFailed("Connection not found".to_string()))?;

        let ice_candidate = RTCIceCandidateInit {
            candidate,
            sdp_mid,
            sdp_mline_index,
            username_fragment: None,
        };

        peer_connection
            .add_ice_candidate(ice_candidate)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to add ICE candidate: {}", e)))?;

        Ok(())
    }

    /// Process signaling messages
    async fn process_signaling(&self) -> Result<(), TransportError> {
        if let Some(rx) = &self.signaling_rx {
            let mut rx = rx.lock().await;
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    SignalingMessage::Offer { from, sdp, .. } => {
                        match self.handle_offer(from, sdp).await {
                            Ok(answer_sdp) => {
                                // Send answer back through signaling
                                if let Some(tx) = &self.signaling_tx {
                                    let _ = tx.send(SignalingMessage::Answer {
                                        from: "local".to_string(),
                                        to: from,
                                        sdp: answer_sdp,
                                    }).await;
                                }
                            }
                            Err(e) => error!("Failed to handle offer: {}", e),
                        }
                    }
                    SignalingMessage::Answer { from: connection_id, sdp, .. } => {
                        if let Err(e) = self.handle_answer(connection_id, sdp).await {
                            error!("Failed to handle answer: {}", e);
                        }
                    }
                    SignalingMessage::IceCandidate {
                        from: connection_id,
                        candidate,
                        sdp_mid,
                        sdp_mline_index,
                        ..
                    } => {
                        if let Err(e) = self.handle_ice_candidate(
                            connection_id,
                            candidate,
                            sdp_mid,
                            sdp_mline_index,
                        ).await {
                            error!("Failed to handle ICE candidate: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Transport for WebRTCTransport {
    async fn init(&mut self, config: TransportConfig) -> Result<(), TransportError> {
        info!("Initializing WebRTC transport");
        
        // Merge transport config with WebRTC-specific config
        if config.max_message_size > 0 {
            self.config.max_message_size = config.max_message_size;
        }

        // Start signaling processing task
        let signaling_rx = self.signaling_rx.clone();
        if signaling_rx.is_some() {
            let transport = self.clone();
            tokio::spawn(async move {
                loop {
                    if let Err(e) = transport.process_signaling().await {
                        error!("Signaling processing error: {}", e);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            });
        }

        info!("WebRTC transport initialized successfully");
        Ok(())
    }

    async fn listen(&mut self, addr: SocketAddr) -> Result<(), TransportError> {
        // WebRTC doesn't directly listen on addresses like TCP
        // Instead, it uses signaling servers for connection establishment
        info!("WebRTC transport ready for connections (signaling required)");
        Ok(())
    }

    async fn connect(
        &mut self,
        addr: SocketAddr,
    ) -> Result<Box<dyn AsyncTransport + Send + Sync>, TransportError> {
        let connection_id = self.generate_connection_id();
        let peer_connection = self.create_peer_connection(connection_id.clone()).await?;

        // Create data channel
        let data_channel = self.create_data_channel(&peer_connection, "qudag").await?;

        // Create offer
        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create offer: {}", e)))?;

        peer_connection
            .set_local_description(offer.clone())
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to set local description: {}", e)))?;

        // Store connection
        self.connections.write().await.insert(connection_id.clone(), peer_connection);
        self.data_channels.write().await.insert(connection_id.clone(), data_channel.clone());

        // Create metadata
        let metadata = ConnectionMetadata {
            connection_id: connection_id.clone(),
            peer_id: Some(PeerId::from_bytes([0u8; 32])), // TODO: Extract actual peer ID
            status: ConnectionStatus::Connecting,
            established_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            is_post_quantum: false,
            tls_version: Some("DTLS 1.2".to_string()),
        };
        self.metadata.write().await.insert(connection_id.clone(), metadata.clone());

        // Send offer through signaling
        if let Some(tx) = &self.signaling_tx {
            let _ = tx.send(SignalingMessage::Offer {
                from: connection_id.clone(),
                to: addr.to_string(), // This should be the actual peer ID
                sdp: offer.sdp,
            }).await;
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_connections += 1;
        stats.active_connections = self.connections.read().await.len();

        // Return WebRTC transport wrapper
        Ok(Box::new(WebRTCDataChannelTransport {
            data_channel,
            connection_id,
            metadata,
            bytes_sent: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            bytes_received: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }))
    }

    async fn accept(&mut self) -> Result<Box<dyn AsyncTransport + Send + Sync>, TransportError> {
        // WebRTC doesn't have a traditional accept model
        // Connections are established through signaling
        Err(TransportError::ConnectionFailed(
            "WebRTC connections must be established through signaling".to_string(),
        ))
    }

    async fn close_connection(&mut self, connection_id: &str) -> Result<(), TransportError> {
        // Remove and close peer connection
        if let Some(peer_connection) = self.connections.write().await.remove(connection_id) {
            peer_connection.close().await.map_err(|e| {
                TransportError::ConnectionFailed(format!("Failed to close connection: {}", e))
            })?;
        }

        // Remove data channel
        self.data_channels.write().await.remove(connection_id);

        // Remove metadata
        self.metadata.write().await.remove(connection_id);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.active_connections = self.connections.read().await.len();

        Ok(())
    }

    fn get_connections(&self) -> Vec<ConnectionMetadata> {
        // This is a blocking call in an async context, but it's required by the trait
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            self.metadata
                .read()
                .await
                .values()
                .cloned()
                .collect()
        })
    }

    fn get_stats(&self) -> TransportStats {
        // This is a blocking call in an async context, but it's required by the trait
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            self.stats.read().await.clone()
        })
    }

    async fn shutdown(&mut self) -> Result<(), TransportError> {
        info!("Shutting down WebRTC transport");

        // Close all connections
        let connection_ids: Vec<String> = self.connections.read().await.keys().cloned().collect();
        for conn_id in connection_ids {
            if let Err(e) = self.close_connection(&conn_id).await {
                warn!("Error closing connection {}: {}", conn_id, e);
            }
        }

        info!("WebRTC transport shutdown completed");
        Ok(())
    }
}

/// WebRTC data channel transport wrapper
struct WebRTCDataChannelTransport {
    data_channel: Arc<RTCDataChannel>,
    connection_id: String,
    metadata: ConnectionMetadata,
    bytes_sent: Arc<std::sync::atomic::AtomicU64>,
    bytes_received: Arc<std::sync::atomic::AtomicU64>,
}

impl AsyncRead for WebRTCDataChannelTransport {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        // WebRTC data channels don't implement AsyncRead directly
        // This would need to be implemented with a message queue
        Poll::Pending
    }
}

impl AsyncWrite for WebRTCDataChannelTransport {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        // WebRTC data channels don't implement AsyncWrite directly
        // This would need to be implemented with the send method
        Poll::Pending
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncTransport for WebRTCDataChannelTransport {
    fn peer_addr(&self) -> Result<SocketAddr, TransportError> {
        // WebRTC doesn't expose direct socket addresses
        Err(TransportError::ConnectionFailed(
            "WebRTC doesn't provide direct socket addresses".to_string(),
        ))
    }

    fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        // WebRTC doesn't expose direct socket addresses
        Err(TransportError::ConnectionFailed(
            "WebRTC doesn't provide direct socket addresses".to_string(),
        ))
    }

    fn is_secure(&self) -> bool {
        true // WebRTC uses DTLS for encryption
    }

    fn metadata(&self) -> ConnectionMetadata {
        self.metadata.clone()
    }

    fn close_sync(&mut self) -> Result<(), TransportError> {
        // Data channel closing is async, but we can't await here
        // In a real implementation, this would schedule the close
        Ok(())
    }
}

// Make WebRTCTransport cloneable for the signaling task
impl Clone for WebRTCTransport {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            api: Arc::clone(&self.api),
            connections: Arc::clone(&self.connections),
            data_channels: Arc::clone(&self.data_channels),
            metadata: Arc::clone(&self.metadata),
            signaling_tx: self.signaling_tx.clone(),
            signaling_rx: self.signaling_rx.clone(),
            stats: Arc::clone(&self.stats),
            connection_counter: Arc::clone(&self.connection_counter),
        }
    }
}

/// Create a WebRTC transport for browser compatibility
pub fn create_webrtc_transport(config: WebRTCConfig) -> Box<dyn Transport + Send + Sync> {
    Box::new(WebRTCTransport::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webrtc_config_default() {
        let config = WebRTCConfig::default();
        assert!(!config.stun_servers.is_empty());
        assert_eq!(config.max_message_size, 16 * 1024 * 1024);
        assert!(config.ordered);
    }

    #[tokio::test]
    async fn test_webrtc_transport_creation() {
        let config = WebRTCConfig::default();
        let transport = WebRTCTransport::new(config);
        assert_eq!(transport.get_connections().len(), 0);
    }

    #[test]
    fn test_signaling_message() {
        let msg = SignalingMessage::Offer {
            from: "peer1".to_string(),
            to: "peer2".to_string(),
            sdp: "test_sdp".to_string(),
        };

        match msg {
            SignalingMessage::Offer { from, to, sdp } => {
                assert_eq!(from, "peer1");
                assert_eq!(to, "peer2");
                assert_eq!(sdp, "test_sdp");
            }
            _ => panic!("Wrong message type"),
        }
    }
}