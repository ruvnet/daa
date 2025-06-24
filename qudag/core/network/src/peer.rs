//! P2P network peer management implementation.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use thiserror::Error;

/// Errors that can occur during peer operations.
#[derive(Debug, Error)]
pub enum PeerError {
    /// Connection failed
    #[error("Connection failed")]
    ConnectionFailed,

    /// Peer not found
    #[error("Peer not found")]
    PeerNotFound,

    /// Invalid peer address
    #[error("Invalid peer address")]
    InvalidAddress,

    /// Handshake failed
    #[error("Handshake failed")]
    HandshakeFailed,
}

/// Unique peer identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(Vec<u8>);

impl PeerId {
    /// Create a new random peer ID
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        // For now, use timestamp as ID. In production, this should be more secure
        Self(timestamp.to_be_bytes().to_vec())
    }

    /// Create a peer ID from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of the peer ID
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Peer connection status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerStatus {
    /// Initial connection attempt
    Connecting,

    /// Connected and handshake complete
    Connected,

    /// Connection lost
    Disconnected,

    /// Banned or blacklisted
    Banned,
}

/// Network peer information.
#[derive(Debug, Clone)]
pub struct Peer {
    /// Unique peer identifier
    pub id: PeerId,

    /// Network address
    pub address: SocketAddr,

    /// Connection status
    pub status: PeerStatus,

    /// Protocol version
    pub version: u32,
}

/// Peer management trait defining the interface for peer operations.
pub trait PeerManager {
    /// Add a new peer to the network.
    fn add_peer(&mut self, address: SocketAddr) -> Result<PeerId, PeerError>;

    /// Remove a peer from the network.
    fn remove_peer(&mut self, peer_id: &PeerId) -> Result<(), PeerError>;

    /// Get information about a specific peer.
    fn get_peer(&self, peer_id: &PeerId) -> Result<Peer, PeerError>;

    /// Get list of all connected peers.
    fn get_peers(&self) -> Vec<Peer>;

    /// Ban a peer from the network.
    fn ban_peer(&mut self, peer_id: &PeerId) -> Result<(), PeerError>;

    /// Check if a peer is banned.
    fn is_banned(&self, peer_id: &PeerId) -> bool;
}
