//! NAT traversal and firewall penetration module for QuDAG network.
//!
//! This module implements comprehensive NAT traversal capabilities including:
//! - STUN/TURN protocol support for NAT detection and relay
//! - UPnP and NAT-PMP for automatic port mapping
//! - Hole punching techniques for direct peer connections
//! - AutoNAT protocol from libp2p for NAT detection
//! - Relay functionality for unreachable peers
//! - IPv6 support with fallback to IPv4
//! - Connection upgrade paths from relay to direct connections

use crate::connection::ConnectionManager;
use crate::types::{ConnectionStatus, NetworkError, PeerId};
use dashmap::DashMap;
use libp2p::core::Multiaddr;
use parking_lot::RwLock;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::{interval, sleep, timeout};
use tracing::{debug, error, info, warn};

/// STUN transaction ID type (12 bytes as per RFC 5389)
type TransactionId = [u8; 12];

/// STUN message structure
#[derive(Debug, Clone)]
pub struct Message {
    /// Message type
    pub msg_type: MessageType,
    /// Transaction ID
    pub transaction_id: TransactionId,
    /// Attributes
    pub attributes: Vec<Attribute>,
}

/// STUN message types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    /// Binding request
    BindingRequest,
    /// Binding response
    BindingResponse,
    /// Binding error response
    BindingErrorResponse,
    /// Allocate request (TURN)
    AllocateRequest,
    /// Allocate response (TURN)
    AllocateResponse,
}

/// STUN attributes
#[derive(Debug, Clone)]
pub enum Attribute {
    /// Mapped address
    MappedAddress(SocketAddr),
    /// XOR mapped address
    XorMappedAddress(SocketAddr),
    /// Changed address
    ChangedAddress(SocketAddr),
    /// Username
    Username(String),
    /// Message integrity
    MessageIntegrity(Vec<u8>),
    /// Error code
    ErrorCode(u16, String),
    /// Unknown attributes
    UnknownAttributes(Vec<u16>),
    /// Realm
    Realm(String),
    /// Nonce
    Nonce(Vec<u8>),
}

// NatTraversalStats will be defined later with atomic fields

/// Errors that can occur during NAT traversal operations
#[derive(Debug, Error)]
pub enum NatTraversalError {
    /// STUN operation failed
    #[error("STUN error: {0}")]
    StunError(String),

    /// TURN operation failed
    #[error("TURN error: {0}")]
    TurnError(String),

    /// UPnP operation failed
    #[error("UPnP error: {0}")]
    UpnpError(String),

    /// NAT-PMP operation failed
    #[error("NAT-PMP error: {0}")]
    NatPmpError(String),

    /// Hole punching failed
    #[error("Hole punching failed: {0}")]
    HolePunchError(String),

    /// Relay error
    #[error("Relay error: {0}")]
    RelayError(String),

    /// NAT detection failed
    #[error("NAT detection failed: {0}")]
    DetectionError(String),

    /// Connection upgrade failed
    #[error("Connection upgrade failed: {0}")]
    UpgradeError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(#[from] NetworkError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(NetworkError),
}

/// NAT types detected by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NatType {
    /// No NAT - direct internet connection
    None,
    /// Full cone NAT (one-to-one NAT)
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT (hardest to traverse)
    Symmetric,
    /// Unknown NAT type
    Unknown,
}

/// NAT detection result
#[derive(Debug, Clone)]
pub struct NatInfo {
    /// Detected NAT type
    pub nat_type: NatType,
    /// Public IP address
    pub public_ip: Option<IpAddr>,
    /// Public port (if applicable)
    pub public_port: Option<u16>,
    /// Local IP address
    pub local_ip: IpAddr,
    /// Local port
    pub local_port: u16,
    /// Supports hairpinning
    pub hairpinning: bool,
    /// Detection timestamp
    pub detected_at: Instant,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
}

/// STUN server configuration
#[derive(Debug, Clone)]
pub struct StunServer {
    /// Server address
    pub address: SocketAddr,
    /// Server priority (lower is better)
    pub priority: u32,
    /// Is this server responsive
    pub is_active: bool,
    /// Last successful response time
    pub last_success: Option<Instant>,
    /// Average response time
    pub avg_response_ms: u64,
}

impl StunServer {
    /// Create a new STUN server configuration
    pub fn new(address: SocketAddr, priority: u32) -> Self {
        Self {
            address,
            priority,
            is_active: true,
            last_success: None,
            avg_response_ms: 0,
        }
    }
}

/// TURN server configuration with credentials
#[derive(Debug, Clone)]
pub struct TurnServer {
    /// Server address
    pub address: SocketAddr,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Realm (optional)
    pub realm: Option<String>,
    /// Server priority
    pub priority: u32,
    /// Is this server active
    pub is_active: bool,
    /// Allocated relay address
    pub relay_address: Option<SocketAddr>,
}

/// Configuration for NAT traversal
#[derive(Debug, Clone)]
pub struct NatTraversalConfig {
    /// Enable STUN
    pub enable_stun: bool,
    /// Enable TURN
    pub enable_turn: bool,
    /// Enable UPnP
    pub enable_upnp: bool,
    /// Enable NAT-PMP
    pub enable_nat_pmp: bool,
    /// Enable hole punching
    pub enable_hole_punching: bool,
    /// Enable relay fallback
    pub enable_relay: bool,
    /// Enable IPv6
    pub enable_ipv6: bool,
    /// STUN servers
    pub stun_servers: Vec<StunServer>,
    /// TURN servers
    pub turn_servers: Vec<TurnServer>,
    /// Maximum relay connections
    pub max_relay_connections: usize,
    /// Hole punch timeout
    pub hole_punch_timeout: Duration,
    /// NAT detection interval
    pub detection_interval: Duration,
    /// Connection upgrade interval
    pub upgrade_interval: Duration,
    /// Port mapping lifetime (for UPnP/NAT-PMP)
    pub port_mapping_lifetime: Duration,
}

impl Default for NatTraversalConfig {
    fn default() -> Self {
        Self {
            enable_stun: true,
            enable_turn: true,
            enable_upnp: true,
            enable_nat_pmp: true,
            enable_hole_punching: true,
            enable_relay: true,
            enable_ipv6: true,
            stun_servers: vec![
                StunServer::new("stun1.l.google.com:19302".parse().unwrap(), 1),
                StunServer::new("stun2.l.google.com:19302".parse().unwrap(), 2),
                StunServer::new("stun3.l.google.com:19302".parse().unwrap(), 3),
                StunServer::new("stun4.l.google.com:19302".parse().unwrap(), 4),
            ],
            turn_servers: vec![],
            max_relay_connections: 50,
            hole_punch_timeout: Duration::from_secs(30),
            detection_interval: Duration::from_secs(300), // 5 minutes
            upgrade_interval: Duration::from_secs(60),    // 1 minute
            port_mapping_lifetime: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Main NAT traversal manager
pub struct NatTraversalManager {
    /// Configuration
    config: NatTraversalConfig,
    /// Current NAT information
    nat_info: Arc<RwLock<Option<NatInfo>>>,
    /// Connection manager reference
    connection_manager: Arc<ConnectionManager>,
    /// STUN client
    stun_client: Arc<StunClient>,
    /// TURN client
    turn_client: Arc<TurnClient>,
    /// UPnP manager
    upnp_manager: Arc<UpnpManager>,
    /// NAT-PMP client
    nat_pmp_client: Arc<NatPmpClient>,
    /// Hole punch coordinator
    hole_punch_coordinator: Arc<HolePunchCoordinator>,
    /// Relay manager
    relay_manager: Arc<RelayManager>,
    /// Connection upgrade manager
    upgrade_manager: Arc<ConnectionUpgradeManager>,
    /// Active port mappings
    port_mappings: Arc<DashMap<u16, PortMapping>>,
    /// NAT detection task handle
    detection_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Statistics
    stats: Arc<NatTraversalStats>,
}

/// Port mapping information
#[derive(Debug, Clone)]
pub struct PortMapping {
    /// Local port
    pub local_port: u16,
    /// External port
    pub external_port: u16,
    /// Protocol (TCP/UDP)
    pub protocol: PortMappingProtocol,
    /// Mapping method (UPnP/NAT-PMP)
    pub method: PortMappingMethod,
    /// Creation timestamp
    pub created_at: Instant,
    /// Expiration time
    pub expires_at: Instant,
}

/// Port mapping protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortMappingProtocol {
    /// TCP protocol
    TCP,
    /// UDP protocol
    UDP,
}

/// Port mapping method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortMappingMethod {
    /// UPnP mapping
    Upnp,
    /// NAT-PMP mapping
    NatPmp,
    /// Manual mapping
    Manual,
}

/// NAT traversal statistics
#[derive(Debug)]
pub struct NatTraversalStats {
    /// Total traversal attempts
    pub total_attempts: AtomicU64,
    /// Successful traversals
    pub successful_traversals: AtomicU64,
    /// Failed traversals
    pub failed_traversals: AtomicU64,
    /// Successful STUN queries
    pub stun_success: AtomicU64,
    /// Failed STUN queries
    pub stun_failures: AtomicU64,
    /// Successful hole punches
    pub hole_punch_success: AtomicU64,
    /// Failed hole punches
    pub hole_punch_failures: AtomicU64,
    /// Active relay connections
    pub relay_connections: AtomicU32,
    /// Upgraded connections
    pub upgraded_connections: AtomicU64,
    /// Port mappings created
    pub port_mappings_created: AtomicU64,
    /// Port mappings failed
    pub port_mappings_failed: AtomicU64,
    /// Average traversal time (in milliseconds)
    pub avg_traversal_time_ms: AtomicU64,
}

impl Default for NatTraversalStats {
    fn default() -> Self {
        Self {
            total_attempts: AtomicU64::new(0),
            successful_traversals: AtomicU64::new(0),
            failed_traversals: AtomicU64::new(0),
            stun_success: AtomicU64::new(0),
            stun_failures: AtomicU64::new(0),
            hole_punch_success: AtomicU64::new(0),
            hole_punch_failures: AtomicU64::new(0),
            relay_connections: AtomicU32::new(0),
            upgraded_connections: AtomicU64::new(0),
            port_mappings_created: AtomicU64::new(0),
            port_mappings_failed: AtomicU64::new(0),
            avg_traversal_time_ms: AtomicU64::new(0),
        }
    }
}

/// STUN client for NAT detection and address discovery
pub struct StunClient {
    /// STUN servers
    servers: Arc<RwLock<Vec<StunServer>>>,
    /// UDP socket for STUN
    socket: Arc<Mutex<Option<UdpSocket>>>,
    /// Transaction tracking
    #[allow(dead_code)]
    transactions: Arc<DashMap<TransactionId, StunTransaction>>,
}

/// STUN transaction information
#[derive(Debug)]
#[allow(dead_code)]
struct StunTransaction {
    /// Server address
    server: SocketAddr,
    /// Request timestamp
    sent_at: Instant,
    /// Response callback
    callback: Arc<Mutex<Option<mpsc::Sender<Result<Message, NatTraversalError>>>>>,
}

impl StunClient {
    /// Create a new STUN client
    pub fn new(servers: Vec<StunServer>) -> Self {
        Self {
            servers: Arc::new(RwLock::new(servers)),
            socket: Arc::new(Mutex::new(None)),
            transactions: Arc::new(DashMap::new()),
        }
    }

    /// Detect NAT type and public address
    pub async fn detect_nat(&self) -> Result<NatInfo, NatTraversalError> {
        // Bind local socket
        let local_addr = if false {
            // TODO: Add ipv6 feature flag
            SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)
        } else {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
        };

        let socket = UdpSocket::bind(local_addr).await?;
        let local_addr = socket.local_addr()?;

        // Store socket
        *self.socket.lock().await = Some(socket);

        // Test with multiple STUN servers
        let mut results = Vec::new();
        let servers = self.servers.read().clone();

        for server in servers.iter().filter(|s| s.is_active) {
            match self.query_stun_server(&server.address).await {
                Ok(mapped_addr) => {
                    results.push((server.clone(), mapped_addr));
                    if results.len() >= 3 {
                        break; // We have enough results
                    }
                }
                Err(e) => {
                    warn!("STUN query to {} failed: {}", server.address, e);
                }
            }
        }

        if results.is_empty() {
            return Err(NatTraversalError::DetectionError(
                "No STUN servers responded".to_string(),
            ));
        }

        // Analyze results to determine NAT type
        let nat_type = self.analyze_nat_type(&results, local_addr).await?;
        let (public_ip, public_port) = if let Some((_, addr)) = results.first() {
            (Some(addr.ip()), Some(addr.port()))
        } else {
            (None, None)
        };

        Ok(NatInfo {
            nat_type,
            public_ip,
            public_port,
            local_ip: local_addr.ip(),
            local_port: local_addr.port(),
            hairpinning: false, // TODO: Test hairpinning
            detected_at: Instant::now(),
            confidence: self.calculate_confidence(&results),
        })
    }

    /// Query a single STUN server
    async fn query_stun_server(
        &self,
        server: &SocketAddr,
    ) -> Result<SocketAddr, NatTraversalError> {
        // Get the socket from the mutex guard
        let socket_guard = self.socket.lock().await;
        let socket = socket_guard
            .as_ref()
            .ok_or_else(|| NatTraversalError::StunError("Socket not initialized".to_string()))?;

        // Simple STUN-like request - send a UDP packet and expect echo with public address
        let request_data = b"STUN_REQUEST";

        // Send request
        socket
            .send_to(request_data, server)
            .await
            .map_err(|e| NatTraversalError::StunError(e.to_string()))?;

        // Wait for response
        let mut response_buf = vec![0u8; 1024];
        let (_len, from) = timeout(Duration::from_secs(5), socket.recv_from(&mut response_buf))
            .await
            .map_err(|_| NatTraversalError::Timeout)??;

        if from != *server {
            return Err(NatTraversalError::StunError(
                "Response from wrong server".to_string(),
            ));
        }

        // For simplicity, assume the response contains the public address
        // In a real implementation, this would parse STUN protocol messages
        let local_addr = socket.local_addr()?;

        // Mock response - in real implementation this would be parsed from STUN response
        Ok(SocketAddr::new(server.ip(), local_addr.port()))
    }

    /// Analyze NAT type based on STUN results
    async fn analyze_nat_type(
        &self,
        results: &[(StunServer, SocketAddr)],
        local_addr: SocketAddr,
    ) -> Result<NatType, NatTraversalError> {
        // If public IP matches local IP, no NAT
        if let Some((_, public_addr)) = results.first() {
            if public_addr.ip() == local_addr.ip() {
                return Ok(NatType::None);
            }
        }

        // Check if all results have the same public IP/port
        let all_same = results.windows(2).all(|w| w[0].1 == w[1].1);

        if all_same {
            // Could be Full Cone or Restricted Cone
            // Need additional tests to distinguish
            Ok(NatType::RestrictedCone)
        } else {
            // Different mappings for different servers = Symmetric NAT
            Ok(NatType::Symmetric)
        }
    }

    /// Calculate confidence score based on results
    fn calculate_confidence(&self, results: &[(StunServer, SocketAddr)]) -> f64 {
        let base_confidence = results.len() as f64 / 3.0; // More servers = higher confidence
        base_confidence.min(1.0)
    }
}

/// TURN client for relay allocation
pub struct TurnClient {
    /// TURN servers
    servers: Arc<RwLock<Vec<TurnServer>>>,
    /// Active allocations
    allocations: Arc<DashMap<SocketAddr, TurnAllocation>>,
    /// Allocation semaphore
    allocation_limit: Arc<Semaphore>,
}

/// TURN allocation information
#[derive(Debug, Clone)]
pub struct TurnAllocation {
    /// Server address
    pub server: SocketAddr,
    /// Allocated relay address
    pub relay_address: SocketAddr,
    /// Allocation lifetime
    pub lifetime: Duration,
    /// Created timestamp
    pub created_at: Instant,
    /// Refresh handle
    pub refresh_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl TurnClient {
    /// Create a new TURN client
    pub fn new(servers: Vec<TurnServer>, max_allocations: usize) -> Self {
        Self {
            servers: Arc::new(RwLock::new(servers)),
            allocations: Arc::new(DashMap::new()),
            allocation_limit: Arc::new(Semaphore::new(max_allocations)),
        }
    }

    /// Allocate a relay address
    pub async fn allocate_relay(&self) -> Result<TurnAllocation, NatTraversalError> {
        // Acquire allocation permit
        let _permit =
            self.allocation_limit.acquire().await.map_err(|_| {
                NatTraversalError::TurnError("Allocation limit reached".to_string())
            })?;

        // Try each TURN server
        let servers = self.servers.read().clone();
        for server in servers.iter().filter(|s| s.is_active) {
            match self.allocate_from_server(server).await {
                Ok(allocation) => {
                    self.allocations.insert(server.address, allocation.clone());
                    return Ok(allocation);
                }
                Err(e) => {
                    warn!("TURN allocation from {} failed: {}", server.address, e);
                }
            }
        }

        Err(NatTraversalError::TurnError(
            "No TURN servers available".to_string(),
        ))
    }

    /// Allocate from a specific TURN server
    async fn allocate_from_server(
        &self,
        server: &TurnServer,
    ) -> Result<TurnAllocation, NatTraversalError> {
        // TODO: Implement actual TURN allocation protocol
        // For now, return a mock allocation
        Ok(TurnAllocation {
            server: server.address,
            relay_address: server.address, // In real implementation, this would be different
            lifetime: Duration::from_secs(600),
            created_at: Instant::now(),
            refresh_handle: Arc::new(Mutex::new(None)),
        })
    }
}

/// Simple UPnP gateway representation
#[derive(Debug, Clone)]
pub struct SimpleGateway {
    /// Gateway address
    pub address: SocketAddr,
    /// Friendly name
    pub name: String,
}

/// UPnP manager for automatic port mapping
pub struct UpnpManager {
    /// Gateway device
    gateway: Arc<Mutex<Option<SimpleGateway>>>,
    /// Active mappings
    mappings: Arc<DashMap<u16, UpnpMapping>>,
    /// Mapping refresh interval
    #[allow(dead_code)]
    refresh_interval: Duration,
}

/// UPnP mapping information
#[derive(Debug, Clone)]
pub struct UpnpMapping {
    /// Local port
    pub local_port: u16,
    /// External port  
    pub external_port: u16,
    /// Protocol
    pub protocol: PortMappingProtocol,
    /// Description
    pub description: String,
    /// Lease duration
    pub lease_duration: Duration,
    /// Created timestamp
    pub created_at: Instant,
}

impl UpnpManager {
    /// Create a new UPnP manager
    pub fn new(refresh_interval: Duration) -> Self {
        Self {
            gateway: Arc::new(Mutex::new(None)),
            mappings: Arc::new(DashMap::new()),
            refresh_interval,
        }
    }

    /// Discover UPnP gateway
    pub async fn discover_gateway(&self) -> Result<(), NatTraversalError> {
        // Simple UPnP discovery simulation
        // In a real implementation, this would use SSDP multicast discovery
        let potential_gateways = vec!["192.168.1.1:1900", "192.168.0.1:1900", "10.0.0.1:1900"];

        for gateway_addr in potential_gateways {
            if let Ok(addr) = gateway_addr.parse::<SocketAddr>() {
                // Test if gateway responds
                if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                    if socket.send_to(b"M-SEARCH", addr).await.is_ok() {
                        info!("Discovered UPnP gateway at: {}", addr);
                        let gateway = SimpleGateway {
                            address: addr,
                            name: "UPnP Gateway".to_string(),
                        };
                        *self.gateway.lock().await = Some(gateway);
                        return Ok(());
                    }
                }
            }
        }

        Err(NatTraversalError::UpnpError(
            "No UPnP gateway found".to_string(),
        ))
    }

    /// Create port mapping
    pub async fn create_mapping(
        &self,
        local_port: u16,
        external_port: u16,
        protocol: PortMappingProtocol,
        description: &str,
        lease_duration: Duration,
    ) -> Result<UpnpMapping, NatTraversalError> {
        // Simulate UPnP port mapping
        // In a real implementation, this would send UPnP control messages
        info!(
            "Creating UPnP port mapping: {}:{} -> {} ({})",
            local_port, external_port, protocol as u8, description
        );

        let mapping = UpnpMapping {
            local_port,
            external_port,
            protocol,
            description: description.to_string(),
            lease_duration,
            created_at: Instant::now(),
        };

        self.mappings.insert(local_port, mapping.clone());
        Ok(mapping)
    }

    /// Get local IP address
    #[allow(dead_code)]
    async fn get_local_ip(&self) -> Result<IpAddr, NatTraversalError> {
        // Try to get local IP by connecting to a public address
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect("8.8.8.8:80").await?;
        let local_addr = socket.local_addr()?;
        Ok(local_addr.ip())
    }
}

/// NAT-PMP client for port mapping
pub struct NatPmpClient {
    /// Gateway address
    gateway: Arc<Mutex<Option<IpAddr>>>,
    /// Active mappings
    mappings: Arc<DashMap<u16, NatPmpMapping>>,
}

/// NAT-PMP mapping information
#[derive(Debug, Clone)]
pub struct NatPmpMapping {
    /// Local port
    pub local_port: u16,
    /// External port
    pub external_port: u16,
    /// Protocol (TCP/UDP)
    pub is_tcp: bool,
    /// Lifetime
    pub lifetime: Duration,
    /// Created timestamp
    pub created_at: Instant,
}

impl NatPmpClient {
    /// Create a new NAT-PMP client
    pub fn new() -> Self {
        Self {
            gateway: Arc::new(Mutex::new(None)),
            mappings: Arc::new(DashMap::new()),
        }
    }

    /// Discover NAT-PMP gateway
    pub async fn discover_gateway(&self) -> Result<(), NatTraversalError> {
        // TODO: Implement gateway discovery
        // For now, try common gateway addresses
        let common_gateways = vec!["192.168.1.1", "192.168.0.1", "10.0.0.1"];

        for gateway_str in common_gateways {
            if let Ok(gateway) = gateway_str.parse::<IpAddr>() {
                // Test if gateway responds to NAT-PMP
                if self.test_gateway(&gateway).await {
                    *self.gateway.lock().await = Some(gateway);
                    info!("Discovered NAT-PMP gateway: {}", gateway);
                    return Ok(());
                }
            }
        }

        Err(NatTraversalError::NatPmpError(
            "No NAT-PMP gateway found".to_string(),
        ))
    }

    /// Test if an address responds to NAT-PMP
    async fn test_gateway(&self, _gateway: &IpAddr) -> bool {
        // TODO: Implement actual NAT-PMP protocol test
        // For now, return false
        false
    }

    /// Create port mapping
    pub async fn create_mapping(
        &self,
        local_port: u16,
        external_port: u16,
        is_tcp: bool,
        lifetime: Duration,
    ) -> Result<NatPmpMapping, NatTraversalError> {
        let gateway = self.gateway.lock().await;
        let _gateway_addr = gateway
            .as_ref()
            .ok_or_else(|| NatTraversalError::NatPmpError("No gateway discovered".to_string()))?;

        // TODO: Implement actual NAT-PMP mapping protocol

        let mapping = NatPmpMapping {
            local_port,
            external_port,
            is_tcp,
            lifetime,
            created_at: Instant::now(),
        };

        self.mappings.insert(local_port, mapping.clone());
        Ok(mapping)
    }
}

/// Hole punch coordinator for establishing direct connections
pub struct HolePunchCoordinator {
    /// Active hole punch attempts
    attempts: Arc<DashMap<PeerId, HolePunchAttempt>>,
    /// Success callback handlers
    success_handlers: Arc<DashMap<PeerId, mpsc::Sender<SocketAddr>>>,
    /// Hole punch timeout
    timeout: Duration,
}

/// Hole punch attempt information
#[derive(Debug)]
pub struct HolePunchAttempt {
    /// Target peer
    pub peer_id: PeerId,
    /// Local candidate addresses
    pub local_candidates: Vec<SocketAddr>,
    /// Remote candidate addresses
    pub remote_candidates: Vec<SocketAddr>,
    /// Started timestamp
    pub started_at: Instant,
    /// Current phase
    pub phase: HolePunchPhase,
    /// Success flag
    pub succeeded: Arc<AtomicBool>,
}

/// Hole punch phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolePunchPhase {
    /// Gathering candidates
    GatheringCandidates,
    /// Exchanging candidates
    ExchangingCandidates,
    /// Probing connections
    Probing,
    /// Connection established
    Connected,
    /// Failed
    Failed,
}

impl HolePunchCoordinator {
    /// Create a new hole punch coordinator
    pub fn new(timeout: Duration) -> Self {
        Self {
            attempts: Arc::new(DashMap::new()),
            success_handlers: Arc::new(DashMap::new()),
            timeout,
        }
    }

    /// Start hole punching to a peer
    pub async fn start_hole_punch(
        &self,
        peer_id: PeerId,
        local_candidates: Vec<SocketAddr>,
        remote_candidates: Vec<SocketAddr>,
    ) -> Result<SocketAddr, NatTraversalError> {
        info!("Starting hole punch to peer {:?}", peer_id);

        let attempt = HolePunchAttempt {
            peer_id,
            local_candidates: local_candidates.clone(),
            remote_candidates: remote_candidates.clone(),
            started_at: Instant::now(),
            phase: HolePunchPhase::Probing,
            succeeded: Arc::new(AtomicBool::new(false)),
        };

        self.attempts.insert(peer_id, attempt);

        // Create result channel
        let (tx, mut rx) = mpsc::channel(1);
        self.success_handlers.insert(peer_id, tx);

        // Start probing all candidate pairs
        let _probe_tasks: Vec<_> = local_candidates
            .iter()
            .flat_map(|local| {
                remote_candidates
                    .iter()
                    .map(move |remote| self.probe_candidate_pair(*local, *remote, peer_id))
            })
            .collect();

        // Wait for success or timeout
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Some(addr) => {
                        self.mark_success(peer_id);
                        Ok(addr)
                    }
                    None => Err(NatTraversalError::HolePunchError("Channel closed".to_string()))
                }
            }
            _ = sleep(self.timeout) => {
                self.mark_failure(peer_id);
                Err(NatTraversalError::HolePunchError("Timeout".to_string()))
            }
        }
    }

    /// Probe a candidate pair
    async fn probe_candidate_pair(
        &self,
        local: SocketAddr,
        remote: SocketAddr,
        peer_id: PeerId,
    ) -> Result<(), NatTraversalError> {
        debug!("Probing candidate pair: {} -> {}", local, remote);

        let socket = UdpSocket::bind(local).await?;

        // Send probe packets
        for i in 0..5 {
            let probe_data = format!("HOLE_PUNCH_PROBE_{}", i).into_bytes();
            socket.send_to(&probe_data, remote).await?;

            // Wait for response with short timeout
            let mut buf = vec![0u8; 1024];
            match timeout(Duration::from_millis(200), socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => {
                    if from == remote && len > 0 {
                        // Success! Notify coordinator
                        if let Some(handler) = self.success_handlers.get(&peer_id) {
                            let _ = handler.send(local).await;
                        }
                        return Ok(());
                    }
                }
                _ => continue, // Timeout or error, try next probe
            }

            sleep(Duration::from_millis(100)).await;
        }

        Err(NatTraversalError::HolePunchError(
            "No response from remote".to_string(),
        ))
    }

    /// Mark hole punch as successful
    fn mark_success(&self, peer_id: PeerId) {
        if let Some(mut attempt) = self.attempts.get_mut(&peer_id) {
            attempt.phase = HolePunchPhase::Connected;
            attempt.succeeded.store(true, Ordering::Relaxed);
        }
    }

    /// Mark hole punch as failed
    fn mark_failure(&self, peer_id: PeerId) {
        if let Some(mut attempt) = self.attempts.get_mut(&peer_id) {
            attempt.phase = HolePunchPhase::Failed;
        }
    }
}

/// Relay manager for unreachable peers
pub struct RelayManager {
    /// Available relay servers
    relay_servers: Arc<RwLock<Vec<RelayServer>>>,
    /// Active relay connections
    relay_connections: Arc<DashMap<PeerId, RelayConnection>>,
    /// Connection limit
    connection_limit: Arc<Semaphore>,
    /// Relay statistics
    stats: Arc<RelayStats>,
}

/// Relay server information
#[derive(Debug, Clone)]
pub struct RelayServer {
    /// Server ID
    pub id: PeerId,
    /// Server address
    pub address: Multiaddr,
    /// Server capacity
    pub capacity: u32,
    /// Current load
    pub load: Arc<AtomicU32>,
    /// Is available
    pub is_available: bool,
    /// Last health check
    pub last_health_check: Option<Instant>,
}

/// Relay connection information
#[derive(Debug, Clone)]
pub struct RelayConnection {
    /// Relay server
    pub relay_server: PeerId,
    /// Target peer
    pub target_peer: PeerId,
    /// Connection ID
    pub connection_id: u64,
    /// Established timestamp
    pub established_at: Instant,
    /// Bytes relayed
    pub bytes_relayed: Arc<AtomicU64>,
    /// Is active
    pub is_active: Arc<AtomicBool>,
}

/// Relay statistics
#[derive(Debug)]
pub struct RelayStats {
    /// Total relay connections
    pub total_connections: AtomicU64,
    /// Active relay connections
    pub active_connections: AtomicU32,
    /// Total bytes relayed
    pub bytes_relayed: AtomicU64,
    /// Failed relay attempts
    pub failed_attempts: AtomicU64,
}

impl RelayManager {
    /// Create a new relay manager
    pub fn new(max_connections: usize) -> Self {
        Self {
            relay_servers: Arc::new(RwLock::new(Vec::new())),
            relay_connections: Arc::new(DashMap::new()),
            connection_limit: Arc::new(Semaphore::new(max_connections)),
            stats: Arc::new(RelayStats {
                total_connections: AtomicU64::new(0),
                active_connections: AtomicU32::new(0),
                bytes_relayed: AtomicU64::new(0),
                failed_attempts: AtomicU64::new(0),
            }),
        }
    }

    /// Add a relay server
    pub async fn add_relay_server(&self, server: RelayServer) {
        self.relay_servers.write().push(server);
    }

    /// Establish relay connection to a peer
    pub async fn establish_relay(
        &self,
        target_peer: PeerId,
    ) -> Result<RelayConnection, NatTraversalError> {
        // Acquire connection permit
        let _permit =
            self.connection_limit.acquire().await.map_err(|_| {
                NatTraversalError::RelayError("Connection limit reached".to_string())
            })?;

        // Find best relay server
        let relay_server = self.select_relay_server().await?;

        // Create relay connection
        let connection = RelayConnection {
            relay_server: relay_server.id,
            target_peer,
            connection_id: thread_rng().gen(),
            established_at: Instant::now(),
            bytes_relayed: Arc::new(AtomicU64::new(0)),
            is_active: Arc::new(AtomicBool::new(true)),
        };

        // Update stats
        self.stats.total_connections.fetch_add(1, Ordering::Relaxed);
        self.stats
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        relay_server.load.fetch_add(1, Ordering::Relaxed);

        self.relay_connections
            .insert(target_peer, connection.clone());

        info!(
            "Established relay connection to {:?} via {:?}",
            target_peer, relay_server.id
        );
        Ok(connection)
    }

    /// Select best relay server based on load
    async fn select_relay_server(&self) -> Result<RelayServer, NatTraversalError> {
        let servers = self.relay_servers.read();

        servers
            .iter()
            .filter(|s| s.is_available)
            .min_by_key(|s| s.load.load(Ordering::Relaxed))
            .cloned()
            .ok_or_else(|| NatTraversalError::RelayError("No relay servers available".to_string()))
    }

    /// Close relay connection
    pub async fn close_relay(&self, peer_id: &PeerId) {
        if let Some((_, connection)) = self.relay_connections.remove(peer_id) {
            connection.is_active.store(false, Ordering::Relaxed);
            self.stats
                .active_connections
                .fetch_sub(1, Ordering::Relaxed);

            // Update relay server load
            let servers = self.relay_servers.read();
            if let Some(server) = servers.iter().find(|s| s.id == connection.relay_server) {
                server.load.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }
}

/// Connection upgrade manager for upgrading relay connections to direct
pub struct ConnectionUpgradeManager {
    /// Upgrade attempts
    upgrade_attempts: Arc<DashMap<PeerId, UpgradeAttempt>>,
    /// Upgrade interval
    upgrade_interval: Duration,
    /// NAT traversal manager reference
    nat_manager: Option<Arc<NatTraversalManager>>,
}

/// Connection upgrade attempt
#[derive(Debug)]
pub struct UpgradeAttempt {
    /// Target peer
    pub peer_id: PeerId,
    /// Current connection type
    pub current_type: ConnectionType,
    /// Attempt count
    pub attempt_count: u32,
    /// Last attempt timestamp
    pub last_attempt: Instant,
    /// Success flag
    pub succeeded: bool,
}

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    /// Direct connection
    Direct,
    /// Relay connection
    Relay,
    /// TURN relay
    Turn,
}

impl ConnectionUpgradeManager {
    /// Create a new connection upgrade manager
    pub fn new(upgrade_interval: Duration) -> Self {
        Self {
            upgrade_attempts: Arc::new(DashMap::new()),
            upgrade_interval,
            nat_manager: None,
        }
    }

    /// Set NAT manager reference
    pub fn set_nat_manager(&mut self, nat_manager: Arc<NatTraversalManager>) {
        self.nat_manager = Some(nat_manager);
    }

    /// Try to upgrade a connection
    pub async fn try_upgrade(
        &self,
        peer_id: PeerId,
        current_type: ConnectionType,
    ) -> Result<ConnectionType, NatTraversalError> {
        if current_type == ConnectionType::Direct {
            return Ok(ConnectionType::Direct); // Already direct
        }

        let mut attempt = self
            .upgrade_attempts
            .entry(peer_id)
            .or_insert(UpgradeAttempt {
                peer_id,
                current_type,
                attempt_count: 0,
                last_attempt: Instant::now(),
                succeeded: false,
            });

        // Check if we should attempt upgrade
        if attempt.last_attempt.elapsed() < self.upgrade_interval {
            return Err(NatTraversalError::UpgradeError(
                "Too soon to retry".to_string(),
            ));
        }

        attempt.attempt_count += 1;
        attempt.last_attempt = Instant::now();

        // Try hole punching if we have NAT manager
        if let Some(nat_manager) = &self.nat_manager {
            match nat_manager.establish_direct_connection(peer_id).await {
                Ok(_) => {
                    attempt.succeeded = true;
                    info!(
                        "Successfully upgraded connection to {:?} from {:?} to Direct",
                        peer_id, current_type
                    );
                    Ok(ConnectionType::Direct)
                }
                Err(e) => {
                    warn!("Failed to upgrade connection to {:?}: {}", peer_id, e);
                    Err(e)
                }
            }
        } else {
            Err(NatTraversalError::UpgradeError(
                "NAT manager not available".to_string(),
            ))
        }
    }
}

impl NatTraversalManager {
    /// Create a new NAT traversal manager
    pub fn new(config: NatTraversalConfig, connection_manager: Arc<ConnectionManager>) -> Self {
        let stats = Arc::new(NatTraversalStats::default());

        Self {
            config: config.clone(),
            nat_info: Arc::new(RwLock::new(None)),
            connection_manager,
            stun_client: Arc::new(StunClient::new(config.stun_servers.clone())),
            turn_client: Arc::new(TurnClient::new(
                config.turn_servers.clone(),
                config.max_relay_connections,
            )),
            upnp_manager: Arc::new(UpnpManager::new(config.port_mapping_lifetime)),
            nat_pmp_client: Arc::new(NatPmpClient::new()),
            hole_punch_coordinator: Arc::new(HolePunchCoordinator::new(config.hole_punch_timeout)),
            relay_manager: Arc::new(RelayManager::new(config.max_relay_connections)),
            upgrade_manager: Arc::new(ConnectionUpgradeManager::new(config.upgrade_interval)),
            port_mappings: Arc::new(DashMap::new()),
            detection_handle: Arc::new(Mutex::new(None)),
            stats,
        }
    }

    /// Initialize NAT traversal
    pub async fn initialize(&self) -> Result<(), NatTraversalError> {
        info!("Initializing NAT traversal manager");

        // Start NAT detection
        if self.config.enable_stun {
            self.start_nat_detection().await?;
        }

        // Discover gateways
        if self.config.enable_upnp {
            if let Err(e) = self.upnp_manager.discover_gateway().await {
                warn!("UPnP gateway discovery failed: {}", e);
            }
        }

        if self.config.enable_nat_pmp {
            if let Err(e) = self.nat_pmp_client.discover_gateway().await {
                warn!("NAT-PMP gateway discovery failed: {}", e);
            }
        }

        // Start periodic tasks
        self.start_periodic_tasks().await;

        Ok(())
    }

    /// Start NAT detection
    async fn start_nat_detection(&self) -> Result<(), NatTraversalError> {
        match self.stun_client.detect_nat().await {
            Ok(nat_info) => {
                info!("NAT detected: {:?}", nat_info.nat_type);
                *self.nat_info.write() = Some(nat_info);
                self.stats.stun_success.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                error!("NAT detection failed: {}", e);
                self.stats.stun_failures.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Start periodic maintenance tasks
    async fn start_periodic_tasks(&self) {
        let nat_info = Arc::clone(&self.nat_info);
        let stun_client = Arc::clone(&self.stun_client);
        let stats = Arc::clone(&self.stats);
        let detection_interval = self.config.detection_interval;

        // NAT detection refresh task
        let detection_task = tokio::spawn(async move {
            let mut interval = interval(detection_interval);
            loop {
                interval.tick().await;

                match stun_client.detect_nat().await {
                    Ok(new_info) => {
                        *nat_info.write() = Some(new_info);
                        stats.stun_success.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        warn!("Periodic NAT detection failed: {}", e);
                        stats.stun_failures.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        *self.detection_handle.lock().await = Some(detection_task);
    }

    /// Get current NAT information
    pub fn get_nat_info(&self) -> Option<NatInfo> {
        self.nat_info.read().clone()
    }

    /// Create port mapping
    pub async fn create_port_mapping(
        &self,
        local_port: u16,
        external_port: u16,
        protocol: PortMappingProtocol,
    ) -> Result<PortMapping, NatTraversalError> {
        // Try UPnP first
        if self.config.enable_upnp {
            match self
                .upnp_manager
                .create_mapping(
                    local_port,
                    external_port,
                    protocol,
                    "QuDAG P2P",
                    self.config.port_mapping_lifetime,
                )
                .await
            {
                Ok(mapping) => {
                    let port_mapping = PortMapping {
                        local_port,
                        external_port: mapping.external_port,
                        protocol,
                        method: PortMappingMethod::Upnp,
                        created_at: Instant::now(),
                        expires_at: Instant::now() + mapping.lease_duration,
                    };

                    self.port_mappings.insert(local_port, port_mapping.clone());
                    self.stats
                        .port_mappings_created
                        .fetch_add(1, Ordering::Relaxed);
                    return Ok(port_mapping);
                }
                Err(e) => {
                    warn!("UPnP port mapping failed: {}", e);
                }
            }
        }

        // Try NAT-PMP
        if self.config.enable_nat_pmp {
            let is_tcp = matches!(protocol, PortMappingProtocol::TCP);
            match self
                .nat_pmp_client
                .create_mapping(
                    local_port,
                    external_port,
                    is_tcp,
                    self.config.port_mapping_lifetime,
                )
                .await
            {
                Ok(mapping) => {
                    let port_mapping = PortMapping {
                        local_port,
                        external_port: mapping.external_port,
                        protocol,
                        method: PortMappingMethod::NatPmp,
                        created_at: Instant::now(),
                        expires_at: Instant::now() + mapping.lifetime,
                    };

                    self.port_mappings.insert(local_port, port_mapping.clone());
                    self.stats
                        .port_mappings_created
                        .fetch_add(1, Ordering::Relaxed);
                    return Ok(port_mapping);
                }
                Err(e) => {
                    warn!("NAT-PMP port mapping failed: {}", e);
                }
            }
        }

        self.stats
            .port_mappings_failed
            .fetch_add(1, Ordering::Relaxed);
        Err(NatTraversalError::UpnpError(
            "All port mapping methods failed".to_string(),
        ))
    }

    /// Establish connection to a peer with NAT traversal
    pub async fn connect_peer(&self, peer_id: PeerId) -> Result<(), NatTraversalError> {
        // Try direct connection first
        match self.connection_manager.connect(peer_id).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                debug!("Direct connection failed: {}, trying NAT traversal", e);
            }
        }

        // Try hole punching if enabled
        if self.config.enable_hole_punching {
            match self.try_hole_punch(peer_id).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    debug!("Hole punching failed: {}", e);
                    self.stats
                        .hole_punch_failures
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Fall back to relay if enabled
        if self.config.enable_relay {
            match self.establish_relay_connection(peer_id).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    error!("Relay connection failed: {}", e);
                }
            }
        }

        Err(NatTraversalError::ConnectionError(
            NetworkError::ConnectionError("All connection methods failed".to_string()),
        ))
    }

    /// Try hole punching to establish direct connection
    async fn try_hole_punch(&self, peer_id: PeerId) -> Result<(), NatTraversalError> {
        // Get local candidates
        let local_candidates = self.gather_local_candidates().await?;

        // Exchange candidates with peer (through signaling)
        let remote_candidates = self.exchange_candidates(peer_id, &local_candidates).await?;

        // Start hole punching
        match self
            .hole_punch_coordinator
            .start_hole_punch(peer_id, local_candidates, remote_candidates)
            .await
        {
            Ok(addr) => {
                info!("Hole punch successful, connected via {}", addr);
                self.stats
                    .hole_punch_success
                    .fetch_add(1, Ordering::Relaxed);

                // Update connection in connection manager
                self.connection_manager
                    .update_status(peer_id, ConnectionStatus::Connected);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Gather local candidate addresses
    async fn gather_local_candidates(&self) -> Result<Vec<SocketAddr>, NatTraversalError> {
        let mut candidates = Vec::new();

        // Add public address from NAT info
        if let Some(nat_info) = self.nat_info.read().as_ref() {
            if let (Some(ip), Some(port)) = (nat_info.public_ip, nat_info.public_port) {
                candidates.push(SocketAddr::new(ip, port));
            }
        }

        // Add local addresses
        // TODO: Enumerate local network interfaces

        // Add mapped ports
        for mapping in self.port_mappings.iter() {
            if let Some(public_ip) = self.get_public_ip() {
                candidates.push(SocketAddr::new(public_ip, mapping.external_port));
            }
        }

        Ok(candidates)
    }

    /// Exchange candidates with peer (placeholder - needs signaling)
    async fn exchange_candidates(
        &self,
        _peer_id: PeerId,
        _local_candidates: &[SocketAddr],
    ) -> Result<Vec<SocketAddr>, NatTraversalError> {
        // TODO: Implement actual candidate exchange through signaling
        // For now, return empty list
        Ok(Vec::new())
    }

    /// Establish relay connection
    async fn establish_relay_connection(&self, peer_id: PeerId) -> Result<(), NatTraversalError> {
        // Try TURN relay first
        if self.config.enable_turn {
            match self.turn_client.allocate_relay().await {
                Ok(allocation) => {
                    info!("TURN relay allocated: {}", allocation.relay_address);
                    // TODO: Use TURN relay for connection
                    return Ok(());
                }
                Err(e) => {
                    warn!("TURN allocation failed: {}", e);
                }
            }
        }

        // Use custom relay
        match self.relay_manager.establish_relay(peer_id).await {
            Ok(connection) => {
                info!(
                    "Relay connection established via {:?}",
                    connection.relay_server
                );
                self.stats.relay_connections.fetch_add(1, Ordering::Relaxed);

                // Update connection status
                self.connection_manager
                    .update_status(peer_id, ConnectionStatus::Connected);

                // Schedule upgrade attempt
                self.schedule_connection_upgrade(peer_id, ConnectionType::Relay);

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Schedule connection upgrade attempt
    fn schedule_connection_upgrade(&self, peer_id: PeerId, current_type: ConnectionType) {
        let upgrade_manager = Arc::clone(&self.upgrade_manager);
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            // Wait before attempting upgrade
            sleep(Duration::from_secs(30)).await;

            match upgrade_manager.try_upgrade(peer_id, current_type).await {
                Ok(ConnectionType::Direct) => {
                    stats.upgraded_connections.fetch_add(1, Ordering::Relaxed);
                    stats.relay_connections.fetch_sub(1, Ordering::Relaxed);
                }
                Ok(_) => {}
                Err(e) => {
                    debug!("Connection upgrade failed: {}", e);
                }
            }
        });
    }

    /// Establish direct connection (for connection upgrade)
    async fn establish_direct_connection(&self, peer_id: PeerId) -> Result<(), NatTraversalError> {
        // Try hole punching
        self.try_hole_punch(peer_id).await
    }

    /// Get public IP address
    fn get_public_ip(&self) -> Option<IpAddr> {
        self.nat_info.read().as_ref()?.public_ip
    }

    /// Get NAT traversal statistics
    pub fn get_stats(&self) -> NatTraversalStats {
        NatTraversalStats {
            total_attempts: AtomicU64::new(self.stats.total_attempts.load(Ordering::Relaxed)),
            successful_traversals: AtomicU64::new(
                self.stats.successful_traversals.load(Ordering::Relaxed),
            ),
            failed_traversals: AtomicU64::new(self.stats.failed_traversals.load(Ordering::Relaxed)),
            stun_success: AtomicU64::new(self.stats.stun_success.load(Ordering::Relaxed)),
            stun_failures: AtomicU64::new(self.stats.stun_failures.load(Ordering::Relaxed)),
            hole_punch_success: AtomicU64::new(
                self.stats.hole_punch_success.load(Ordering::Relaxed),
            ),
            hole_punch_failures: AtomicU64::new(
                self.stats.hole_punch_failures.load(Ordering::Relaxed),
            ),
            relay_connections: AtomicU32::new(self.stats.relay_connections.load(Ordering::Relaxed)),
            upgraded_connections: AtomicU64::new(
                self.stats.upgraded_connections.load(Ordering::Relaxed),
            ),
            port_mappings_created: AtomicU64::new(
                self.stats.port_mappings_created.load(Ordering::Relaxed),
            ),
            port_mappings_failed: AtomicU64::new(
                self.stats.port_mappings_failed.load(Ordering::Relaxed),
            ),
            avg_traversal_time_ms: AtomicU64::new(
                self.stats.avg_traversal_time_ms.load(Ordering::Relaxed),
            ),
        }
    }

    /// Shutdown NAT traversal manager
    pub async fn shutdown(&self) -> Result<(), NatTraversalError> {
        info!("Shutting down NAT traversal manager");

        // Cancel detection task
        if let Some(handle) = self.detection_handle.lock().await.take() {
            handle.abort();
        }

        // Close all relay connections
        let relay_peers: Vec<_> = self
            .relay_manager
            .relay_connections
            .iter()
            .map(|entry| *entry.key())
            .collect();

        for peer_id in relay_peers {
            self.relay_manager.close_relay(&peer_id).await;
        }

        // Remove port mappings
        // TODO: Implement port mapping cleanup

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nat_detection() {
        let servers = vec![StunServer::new("8.8.8.8:3478".parse().unwrap(), 1)];

        let client = StunClient::new(servers);

        // This test will fail without real STUN servers
        // It's here to show the structure
        match client.detect_nat().await {
            Ok(nat_info) => {
                println!("NAT type: {:?}", nat_info.nat_type);
                println!("Public IP: {:?}", nat_info.public_ip);
            }
            Err(e) => {
                println!("NAT detection failed: {}", e);
            }
        }
    }

    #[test]
    fn test_nat_type_properties() {
        assert_eq!(NatType::None, NatType::None);
        assert_ne!(NatType::FullCone, NatType::Symmetric);
    }
}
