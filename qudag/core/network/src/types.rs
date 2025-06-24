use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use thiserror::Error;

/// Network address combining IP and port
///
/// # Examples
///
/// ```rust
/// use qudag_network::types::NetworkAddress;
/// use std::net::{IpAddr, Ipv4Addr};
///
/// // Create address from IP parts
/// let addr1 = NetworkAddress::new([127, 0, 0, 1], 8080);
///
/// // Create address from IP and port
/// let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
/// let addr2 = NetworkAddress::from_ip_port(ip, 3000);
///
/// // Get socket address string
/// let socket_str = addr1.to_socket_addr();
/// assert_eq!(socket_str, "127.0.0.1:8080");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkAddress {
    /// IP address
    pub ip: IpAddr,
    /// Port number
    pub port: u16,
}

impl NetworkAddress {
    /// Create a new network address from IPv4 address parts and port
    pub fn new(ip_parts: [u8; 4], port: u16) -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(
                ip_parts[0],
                ip_parts[1],
                ip_parts[2],
                ip_parts[3],
            )),
            port,
        }
    }

    /// Create a new network address from IP and port
    pub fn from_ip_port(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    /// Get the socket address as a string
    pub fn to_socket_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

/// Network errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Message handling failed: {0}")]
    MessageError(String),

    #[error("Routing failed: {0}")]
    RoutingError(String),

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Bootstrap failed")]
    BootstrapFailed,

    #[error("Content too large")]
    ContentTooLarge,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    /// High priority messages
    High,
    /// Normal priority messages
    Normal,
    /// Low priority messages
    Low,
}

/// Message routing strategy
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    /// Direct to peer
    Direct(Vec<u8>),
    /// Flood to all peers
    Flood,
    /// Random subset of peers
    RandomSubset(usize),
    /// Anonymous routing
    Anonymous {
        /// Number of hops
        hops: usize,
    },
}

/// Routing layer for onion routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingLayer {
    /// Next hop
    pub next_hop: Vec<u8>,
    /// Encrypted payload
    pub payload: Vec<u8>,
    /// Layer metadata
    pub metadata: LayerMetadata,
}

/// Routing layer metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetadata {
    /// Time-to-live
    pub ttl: Duration,
    /// Flags
    pub flags: u32,
    /// Layer ID
    pub id: String,
}

/// Network metrics
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    /// Messages per second
    pub messages_per_second: f64,
    /// Current connections
    pub connections: usize,
    /// Active connections
    pub active_connections: usize,
    /// Average message latency
    pub avg_latency: Duration,
    /// Memory usage in bytes
    pub memory_usage: usize,
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Handshake message
    Handshake {
        /// Protocol version
        version: u32,
        /// Node ID
        node_id: Vec<u8>,
    },
    /// Data message
    Data {
        /// Message ID
        id: String,
        /// Payload
        payload: Vec<u8>,
        /// Priority
        priority: MessagePriority,
    },
    /// Control message
    Control {
        /// Command
        command: String,
        /// Parameters
        params: Vec<String>,
    },
}

/// Network message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Message identifier
    pub id: String,
    /// Source node identifier
    pub source: Vec<u8>,
    /// Destination node identifier
    pub destination: Vec<u8>,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message priority
    pub priority: MessagePriority,
    /// Time to live
    pub ttl: Duration,
}

/// Peer identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId([u8; 32]);

impl PeerId {
    /// Generate a random peer ID
    pub fn random() -> Self {
        use rand::RngCore;
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);
        Self(id)
    }

    /// Create a peer ID from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the peer ID as bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// Get the peer ID as a slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format as truncated hex string for readability (first 8 bytes)
        for byte in &self.0[..8] {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Connection status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Connection is being established
    Connecting,
    /// Connection is active
    Connected,
    /// Connection is being closed
    Disconnecting,
    /// Connection is closed
    Disconnected,
    /// Connection failed
    Failed(String),
}

/// Queue performance metrics
#[derive(Debug, Clone, Default)]
pub struct QueueMetrics {
    /// Current queue size
    pub current_size: usize,
    /// Maximum queue size
    pub max_size: usize,
    /// Queue utilization (0.0 to 1.0)
    pub utilization: f64,
    /// High water mark
    pub high_water_mark: usize,
    /// Messages processed per second
    pub messages_per_second: f64,
}

/// Latency performance metrics
#[derive(Debug, Clone, Default)]
pub struct LatencyMetrics {
    /// Average message latency
    pub avg_latency: Duration,
    /// Peak message latency
    pub peak_latency: Duration,
    /// 95th percentile latency
    pub p95_latency: Duration,
    /// 99th percentile latency
    pub p99_latency: Duration,
}

/// Throughput performance metrics
#[derive(Debug, Clone, Default)]
pub struct ThroughputMetrics {
    /// Messages per second
    pub messages_per_second: f64,
    /// Bytes per second
    pub bytes_per_second: f64,
    /// Peak throughput
    pub peak_throughput: f64,
    /// Average throughput
    pub avg_throughput: f64,
    /// Total messages processed
    pub total_messages: u64,
}
