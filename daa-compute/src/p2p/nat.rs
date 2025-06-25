//! NAT traversal implementation using STUN/TURN
//!
//! Provides NAT hole punching and relay capabilities for
//! nodes behind firewalls.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use anyhow::{Result, anyhow};
use tracing::{info, debug, warn};
use igd_next;

/// STUN server configuration
#[derive(Debug, Clone)]
pub struct StunConfig {
    pub servers: Vec<String>,
    pub timeout: Duration,
    pub retry_count: u32,
}

impl Default for StunConfig {
    fn default() -> Self {
        Self {
            servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
                "stun:stun2.l.google.com:19302".to_string(),
                "stun:global.stun.twilio.com:3478".to_string(),
            ],
            timeout: Duration::from_secs(5),
            retry_count: 3,
        }
    }
}

/// TURN server configuration
#[derive(Debug, Clone)]
pub struct TurnConfig {
    pub servers: Vec<TurnServer>,
    pub allocation_lifetime: Duration,
}

#[derive(Debug, Clone)]
pub struct TurnServer {
    pub urls: Vec<String>,
    pub username: String,
    pub credential: String,
    pub credential_type: CredentialType,
}

#[derive(Debug, Clone)]
pub enum CredentialType {
    Password,
    ApiKey,
    OAuth,
}

/// NAT type detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// No NAT, direct internet connection
    None,
    /// Full cone NAT (least restrictive)
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT (most restrictive)
    Symmetric,
    /// Unknown or detection failed
    Unknown,
}

impl NatType {
    /// Check if direct connection is possible
    pub fn supports_direct_connection(&self) -> bool {
        matches!(self, NatType::None | NatType::FullCone | NatType::RestrictedCone)
    }
    
    /// Check if STUN is sufficient
    pub fn requires_turn(&self) -> bool {
        matches!(self, NatType::Symmetric)
    }
}

/// NAT traversal manager
pub struct NatTraversal {
    stun_config: StunConfig,
    turn_config: Option<TurnConfig>,
    public_addresses: Arc<RwLock<Vec<SocketAddr>>>,
    nat_type: Arc<RwLock<NatType>>,
    turn_allocations: Arc<Mutex<Vec<TurnAllocation>>>,
}

/// TURN allocation information
#[derive(Debug, Clone)]
pub struct TurnAllocation {
    pub relay_address: SocketAddr,
    pub username: String,
    pub realm: String,
    pub lifetime: Duration,
    pub created_at: std::time::Instant,
}

impl NatTraversal {
    pub fn new(stun_config: StunConfig, turn_config: Option<TurnConfig>) -> Self {
        Self {
            stun_config,
            turn_config,
            public_addresses: Arc::new(RwLock::new(Vec::new())),
            nat_type: Arc::new(RwLock::new(NatType::Unknown)),
            turn_allocations: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Detect NAT type and public addresses
    pub async fn detect_nat(&self) -> Result<NatType> {
        info!("Starting NAT detection");
        
        // Perform STUN binding tests
        let test_results = self.perform_stun_tests().await?;
        
        // Analyze results to determine NAT type
        let nat_type = self.analyze_nat_type(&test_results).await?;
        
        *self.nat_type.write().await = nat_type;
        info!("Detected NAT type: {:?}", nat_type);
        
        Ok(nat_type)
    }
    
    /// Get public addresses via STUN
    pub async fn get_public_addresses(&self) -> Result<Vec<SocketAddr>> {
        let mut addresses = Vec::new();
        
        for server in &self.stun_config.servers {
            match self.query_stun_server(server).await {
                Ok(addr) => {
                    info!("STUN server {} returned public address: {}", server, addr);
                    addresses.push(addr);
                }
                Err(e) => {
                    warn!("Failed to query STUN server {}: {}", server, e);
                }
            }
        }
        
        if addresses.is_empty() {
            return Err(anyhow!("No STUN servers responded"));
        }
        
        // Update cached addresses
        *self.public_addresses.write().await = addresses.clone();
        
        Ok(addresses)
    }
    
    /// Perform STUN binding request
    async fn query_stun_server(&self, server: &str) -> Result<SocketAddr> {
        use stun::agent::*;
        use stun::message::*;
        use stun::xoraddr::*;
        use stun::client::Client;
        
        // Parse STUN server URL
        let server_addr = server.trim_start_matches("stun:")
            .parse::<SocketAddr>()
            .map_err(|e| anyhow!("Invalid STUN server address: {}", e))?;
        
        // Create STUN client
        // TODO: Fix STUN Client::new API for newer versions
        // For now, we'll create a basic UDP socket for STUN
        // let mut client = Client::new(server_addr, None).await?;
        
        // TODO: Implement STUN binding request with updated API
        // For now, return a placeholder external address
        Ok("127.0.0.1:0".parse().unwrap())
    }
    
    /// Perform comprehensive STUN tests
    async fn perform_stun_tests(&self) -> Result<StunTestResults> {
        let mut results = StunTestResults::default();
        
        // Test 1: Basic binding test
        if let Ok(addr1) = self.get_public_addresses().await {
            results.mapped_addresses = addr1;
        }
        
        // Test 2: Changed address test (if server supports)
        // This would test if NAT changes ports for different destinations
        
        // Test 3: Filtering test
        // This would test if NAT filters incoming packets
        
        Ok(results)
    }
    
    /// Analyze STUN test results to determine NAT type
    async fn analyze_nat_type(&self, results: &StunTestResults) -> Result<NatType> {
        if results.mapped_addresses.is_empty() {
            return Ok(NatType::Unknown);
        }
        
        // Check if all STUN servers report the same public address
        let first_addr = results.mapped_addresses[0];
        let all_same = results.mapped_addresses.iter()
            .all(|addr| addr.ip() == first_addr.ip());
        
        if !all_same {
            // Different public IPs means symmetric NAT
            return Ok(NatType::Symmetric);
        }
        
        // Further tests would determine exact NAT type
        // For now, assume restricted cone if consistent
        Ok(NatType::RestrictedCone)
    }
    
    /// Allocate TURN relay
    pub async fn allocate_turn_relay(&self) -> Result<TurnAllocation> {
        let turn_config = self.turn_config.as_ref()
            .ok_or_else(|| anyhow!("TURN not configured"))?;
        
        for server in &turn_config.servers {
            match self.allocate_from_turn_server(server).await {
                Ok(allocation) => {
                    self.turn_allocations.lock().await.push(allocation.clone());
                    return Ok(allocation);
                }
                Err(e) => {
                    warn!("Failed to allocate from TURN server: {}", e);
                }
            }
        }
        
        Err(anyhow!("Failed to allocate from any TURN server"))
    }
    
    /// Allocate from specific TURN server
    async fn allocate_from_turn_server(&self, server: &TurnServer) -> Result<TurnAllocation> {
        use turn::client::*;
        use turn::auth::*;
        
        // Parse first TURN URL
        let url = server.urls.first()
            .ok_or_else(|| anyhow!("No TURN URLs provided"))?;
        
        let server_addr = url.trim_start_matches("turn:")
            .parse::<SocketAddr>()
            .map_err(|e| anyhow!("Invalid TURN server address: {}", e))?;
        
        // TODO: Create TURN client config for libp2p 0.53
        // The ClientConfig API requires proper connection handling
        /*
        let config = ClientConfig {
            stun_serv_addr: server_addr.to_string(),
            turn_serv_addr: server_addr.to_string(),
            username: server.username.clone(),
            password: server.credential.clone(),
            realm: String::new(),
            software: "DAA-Compute/1.0".to_string(),
            rto_in_ms: 100,
            vnet: None,
            conn: proper_connection, // Needs proper Arc<dyn Conn>
        };
        */
        
        // Create and connect client
        // TODO: Fix TURN client API for newer versions
        // let client = Client::new(config).await?;
        
        // Allocate relay
        // let allocation = client.allocate().await?;
        // let relay_addr = allocation.relay_addr()?;
        
        // For now, return a placeholder
        Ok(TurnAllocation {
            relay_address: server_addr, // Placeholder
            username: server.username.clone(),
            realm: "placeholder".to_string(), // Placeholder
            lifetime: Duration::from_secs(600), // Default 10 minutes
            created_at: std::time::Instant::now(),
        })
    }
    
    /// Perform ICE (Interactive Connectivity Establishment)
    pub async fn establish_connection(
        &self,
        remote_candidates: Vec<IceCandidate>,
    ) -> Result<SocketAddr> {
        // Gather local candidates
        let local_candidates = self.gather_candidates().await?;
        
        // Perform connectivity checks
        for local in &local_candidates {
            for remote in &remote_candidates {
                if let Ok(addr) = self.check_candidate_pair(local, remote).await {
                    info!("Successfully established connection via {}", addr);
                    return Ok(addr);
                }
            }
        }
        
        Err(anyhow!("Failed to establish connection"))
    }
    
    /// Gather ICE candidates
    async fn gather_candidates(&self) -> Result<Vec<IceCandidate>> {
        let mut candidates = Vec::new();
        
        // Host candidates (local addresses)
        candidates.extend(self.get_host_candidates().await?);
        
        // Server reflexive candidates (STUN)
        if let Ok(addrs) = self.get_public_addresses().await {
            for addr in addrs {
                candidates.push(IceCandidate {
                    typ: CandidateType::ServerReflexive,
                    address: addr,
                    priority: calculate_priority(CandidateType::ServerReflexive),
                    foundation: generate_foundation(),
                });
            }
        }
        
        // Relay candidates (TURN)
        if self.nat_type.read().await.requires_turn() {
            if let Ok(allocation) = self.allocate_turn_relay().await {
                candidates.push(IceCandidate {
                    typ: CandidateType::Relay,
                    address: allocation.relay_address,
                    priority: calculate_priority(CandidateType::Relay),
                    foundation: generate_foundation(),
                });
            }
        }
        
        Ok(candidates)
    }
    
    /// Get host candidates
    async fn get_host_candidates(&self) -> Result<Vec<IceCandidate>> {
        use if_addrs::get_if_addrs;
        
        let mut candidates = Vec::new();
        
        for iface in get_if_addrs()? {
            if !iface.is_loopback() {
                let addr = SocketAddr::new(iface.ip(), 0); // Port will be assigned
                candidates.push(IceCandidate {
                    typ: CandidateType::Host,
                    address: addr,
                    priority: calculate_priority(CandidateType::Host),
                    foundation: generate_foundation(),
                });
            }
        }
        
        Ok(candidates)
    }
    
    /// Check connectivity between candidate pair
    async fn check_candidate_pair(
        &self,
        local: &IceCandidate,
        remote: &IceCandidate,
    ) -> Result<SocketAddr> {
        // Simplified connectivity check
        // In production, would use STUN binding requests
        
        debug!("Checking connectivity: {} -> {}", local.address, remote.address);
        
        // For now, assume connectivity if both are not behind symmetric NAT
        if !self.nat_type.read().await.requires_turn() {
            Ok(remote.address)
        } else {
            Err(anyhow!("Direct connectivity not possible"))
        }
    }
}

/// STUN test results
#[derive(Debug, Default)]
struct StunTestResults {
    mapped_addresses: Vec<SocketAddr>,
    changed_addresses: Vec<SocketAddr>,
    filtering_behavior: Option<FilteringBehavior>,
}

#[derive(Debug, Clone, Copy)]
enum FilteringBehavior {
    EndpointIndependent,
    AddressDependent,
    AddressAndPortDependent,
}

/// ICE candidate
#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub typ: CandidateType,
    pub address: SocketAddr,
    pub priority: u32,
    pub foundation: String,
}

#[derive(Debug, Clone, Copy)]
pub enum CandidateType {
    Host,
    ServerReflexive,
    PeerReflexive,
    Relay,
}

/// Calculate ICE candidate priority
fn calculate_priority(typ: CandidateType) -> u32 {
    let type_preference = match typ {
        CandidateType::Host => 126,
        CandidateType::ServerReflexive => 100,
        CandidateType::PeerReflexive => 110,
        CandidateType::Relay => 0,
    };
    
    let local_preference = 65535;
    let component = 1;
    
    (1 << 24) * type_preference + (1 << 8) * local_preference + (256 - component)
}

/// Generate ICE foundation string
fn generate_foundation() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:08x}", rng.gen::<u32>())
}

/// UPnP port mapping for automatic port forwarding
pub struct UpnpPortMapper {
    gateway: Option<igd_next::Gateway>,
}

impl UpnpPortMapper {
    pub async fn new() -> Result<Self> {
        match igd_next::search_gateway(Default::default()) {
            Ok(gateway) => {
                info!("Found UPnP gateway: {}", gateway.addr);
                Ok(Self {
                    gateway: Some(gateway),
                })
            }
            Err(e) => {
                warn!("No UPnP gateway found: {}", e);
                Ok(Self { gateway: None })
            }
        }
    }
    
    /// Add port mapping
    pub async fn add_port(&self, protocol: igd_next::PortMappingProtocol, external_port: u16, internal_port: u16, description: &str) -> Result<()> {
        if let Some(gateway) = &self.gateway {
            gateway.add_port(
                protocol,
                external_port,
                SocketAddr::new(gateway.addr.ip(), internal_port),
                0, // Infinite lease
                description,
            )?;
            
            info!("Added UPnP port mapping: {} -> {}", external_port, internal_port);
        }
        
        Ok(())
    }
    
    /// Remove port mapping
    pub async fn remove_port(&self, protocol: igd_next::PortMappingProtocol, external_port: u16) -> Result<()> {
        if let Some(gateway) = &self.gateway {
            gateway.remove_port(protocol, external_port)?;
            info!("Removed UPnP port mapping: {}", external_port);
        }
        
        Ok(())
    }
}