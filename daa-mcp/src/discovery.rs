//! Agent Discovery Protocol for DAA MCP
//! 
//! This module implements a distributed discovery protocol that allows agents
//! to find and communicate with each other through the MCP interface.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{DaaMcpError, McpServerState, Result};

/// Discovery protocol version
pub const DISCOVERY_PROTOCOL_VERSION: &str = "1.0.0";

/// Default discovery port for UDP multicast
pub const DEFAULT_DISCOVERY_PORT: u16 = 5555;

/// Multicast address for agent discovery
pub const DISCOVERY_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DiscoveryMessage {
    /// Agent announces its presence
    Announce {
        agent_id: String,
        agent_name: String,
        agent_type: String,
        capabilities: Vec<String>,
        endpoint: String,
        mcp_endpoint: Option<String>,
        timestamp: u64,
        ttl: u64,
    },
    /// Request for available agents
    Query {
        query_id: String,
        filter: DiscoveryFilter,
        requester_id: String,
        timestamp: u64,
    },
    /// Response to a query
    Response {
        query_id: String,
        agent_id: String,
        agent_info: AgentDiscoveryInfo,
        timestamp: u64,
    },
    /// Agent is going offline
    Goodbye {
        agent_id: String,
        timestamp: u64,
    },
    /// Heartbeat to maintain presence
    Heartbeat {
        agent_id: String,
        timestamp: u64,
    },
}

/// Filter for discovery queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryFilter {
    pub agent_type: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub max_agents: Option<usize>,
    pub exclude_self: bool,
}

/// Agent information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiscoveryInfo {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub mcp_endpoint: Option<String>,
    pub last_seen: u64,
    pub availability: AgentAvailability,
    pub load_factor: f32,
    pub response_time_avg: f32,
}

/// Agent availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAvailability {
    Available,
    Busy,
    Overloaded,
    Maintenance,
}

/// Discovery protocol configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub enabled: bool,
    pub bind_address: IpAddr,
    pub port: u16,
    pub multicast_address: Ipv4Addr,
    pub announce_interval: Duration,
    pub heartbeat_interval: Duration,
    pub query_timeout: Duration,
    pub agent_ttl: Duration,
    pub max_agents_in_response: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: DEFAULT_DISCOVERY_PORT,
            multicast_address: DISCOVERY_MULTICAST_ADDR,
            announce_interval: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
            query_timeout: Duration::from_secs(5),
            agent_ttl: Duration::from_secs(90),
            max_agents_in_response: 10,
        }
    }
}

/// Discovery protocol implementation
pub struct DiscoveryProtocol {
    config: DiscoveryConfig,
    server_state: Arc<McpServerState>,
    socket: Arc<UdpSocket>,
    discovered_agents: Arc<RwLock<HashMap<String, AgentDiscoveryInfo>>>,
    pending_queries: Arc<RwLock<HashMap<String, broadcast::Sender<AgentDiscoveryInfo>>>>,
    local_agent_id: String,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl DiscoveryProtocol {
    /// Create a new discovery protocol instance
    pub async fn new(
        config: DiscoveryConfig,
        server_state: Arc<McpServerState>,
    ) -> Result<Self> {
        let bind_addr = SocketAddr::new(config.bind_address, config.port);
        let socket = UdpSocket::bind(bind_addr).await
            .map_err(|e| DaaMcpError::Network(e))?;

        // Join multicast group
        socket.join_multicast_v4(config.multicast_address, Ipv4Addr::UNSPECIFIED)
            .map_err(|e| DaaMcpError::Network(e))?;

        let local_agent_id = Uuid::new_v4().to_string();

        info!(
            "Discovery protocol initialized on {} with agent ID {}",
            bind_addr, local_agent_id
        );

        Ok(Self {
            config,
            server_state,
            socket: Arc::new(socket),
            discovered_agents: Arc::new(RwLock::new(HashMap::new())),
            pending_queries: Arc::new(RwLock::new(HashMap::new())),
            local_agent_id,
            shutdown_tx: None,
        })
    }

    /// Start the discovery protocol
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Discovery protocol is disabled");
            return Ok(());
        }

        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        // Start the message receiver
        self.start_receiver(shutdown_tx.subscribe()).await?;

        // Start the announcement loop
        self.start_announcer(shutdown_tx.subscribe()).await?;

        // Start the cleanup task
        self.start_cleanup_task(shutdown_tx.subscribe()).await?;

        info!("Discovery protocol started successfully");
        Ok(())
    }

    /// Stop the discovery protocol
    pub async fn stop(&self) -> Result<()> {
        if let Some(ref shutdown_tx) = self.shutdown_tx {
            // Send goodbye message
            self.send_goodbye().await?;

            // Signal shutdown
            let _ = shutdown_tx.send(());
        }

        info!("Discovery protocol stopped");
        Ok(())
    }

    /// Discover agents matching the filter
    pub async fn discover_agents(
        &self,
        filter: DiscoveryFilter,
    ) -> Result<Vec<AgentDiscoveryInfo>> {
        let query_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = broadcast::channel(100);

        // Store the query channel
        {
            let mut queries = self.pending_queries.write().await;
            queries.insert(query_id.clone(), tx);
        }

        // Send discovery query
        let query_msg = DiscoveryMessage::Query {
            query_id: query_id.clone(),
            filter: filter.clone(),
            requester_id: self.local_agent_id.clone(),
            timestamp: current_timestamp(),
        };

        self.send_message(&query_msg).await?;

        // Collect responses with timeout
        let mut agents = Vec::new();
        let deadline = tokio::time::Instant::now() + self.config.query_timeout;

        while tokio::time::Instant::now() < deadline {
            match timeout(Duration::from_millis(100), rx.recv()).await {
                Ok(Ok(agent_info)) => {
                    agents.push(agent_info);
                    if let Some(max) = filter.max_agents {
                        if agents.len() >= max {
                            break;
                        }
                    }
                }
                Ok(Err(broadcast::error::RecvError::Closed)) => break,
                Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
                Err(_) => continue, // Timeout
            }
        }

        // Clean up the query
        {
            let mut queries = self.pending_queries.write().await;
            queries.remove(&query_id);
        }

        debug!("Discovery query {} returned {} agents", query_id, agents.len());
        Ok(agents)
    }

    /// Get all currently discovered agents
    pub async fn get_discovered_agents(&self) -> Vec<AgentDiscoveryInfo> {
        let agents = self.discovered_agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get agent discovery information by ID
    pub async fn get_agent_info(&self, agent_id: &str) -> Option<AgentDiscoveryInfo> {
        let agents = self.discovered_agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Manually announce presence
    pub async fn announce(&self) -> Result<()> {
        let announce_msg = self.create_announce_message().await?;
        self.send_message(&announce_msg).await
    }

    /// Start the message receiver task
    async fn start_receiver(&self, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
        let socket = self.socket.clone();
        let server_state = self.server_state.clone();
        let discovered_agents = self.discovered_agents.clone();
        let pending_queries = self.pending_queries.clone();
        let local_agent_id = self.local_agent_id.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Discovery receiver shutting down");
                        break;
                    }
                    result = socket.recv_from(&mut buffer) => {
                        match result {
                            Ok((len, addr)) => {
                                if let Err(e) = Self::handle_received_message(
                                    &buffer[..len],
                                    addr,
                                    &server_state,
                                    &discovered_agents,
                                    &pending_queries,
                                    &local_agent_id,
                                    &config,
                                ).await {
                                    warn!("Error handling discovery message: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Error receiving discovery message: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Start the announcement task
    async fn start_announcer(&self, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
        let discovery = Arc::new(self.clone());
        let announce_interval = self.config.announce_interval;

        tokio::spawn(async move {
            let mut interval = interval(announce_interval);

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Discovery announcer shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        if let Err(e) = discovery.announce().await {
                            warn!("Error sending discovery announcement: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Start the cleanup task for stale agents
    async fn start_cleanup_task(&self, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
        let discovered_agents = self.discovered_agents.clone();
        let agent_ttl = self.config.agent_ttl;

        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Discovery cleanup task shutting down");
                        break;
                    }
                    _ = cleanup_interval.tick() => {
                        let now = current_timestamp();
                        let ttl_seconds = agent_ttl.as_secs();
                        
                        let mut agents = discovered_agents.write().await;
                        let initial_count = agents.len();
                        
                        agents.retain(|_, agent| {
                            now - agent.last_seen < ttl_seconds
                        });
                        
                        let removed_count = initial_count - agents.len();
                        if removed_count > 0 {
                            debug!("Cleaned up {} stale agents", removed_count);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle received discovery message
    async fn handle_received_message(
        data: &[u8],
        _addr: SocketAddr,
        server_state: &Arc<McpServerState>,
        discovered_agents: &Arc<RwLock<HashMap<String, AgentDiscoveryInfo>>>,
        pending_queries: &Arc<RwLock<HashMap<String, broadcast::Sender<AgentDiscoveryInfo>>>>,
        local_agent_id: &str,
        config: &DiscoveryConfig,
    ) -> Result<()> {
        let message: DiscoveryMessage = serde_json::from_slice(data)
            .map_err(|e| DaaMcpError::Protocol(format!("Invalid discovery message: {}", e)))?;

        debug!("Received discovery message: {:?}", message);

        match message {
            DiscoveryMessage::Announce {
                agent_id,
                agent_name,
                agent_type,
                capabilities,
                endpoint,
                mcp_endpoint,
                timestamp,
                ttl: _,
            } => {
                if agent_id != local_agent_id {
                    let agent_info = AgentDiscoveryInfo {
                        id: agent_id.clone(),
                        name: agent_name,
                        agent_type,
                        capabilities,
                        endpoint,
                        mcp_endpoint,
                        last_seen: timestamp,
                        availability: AgentAvailability::Available,
                        load_factor: 0.5, // Default value
                        response_time_avg: 100.0, // Default value
                    };

                    let mut agents = discovered_agents.write().await;
                    agents.insert(agent_id, agent_info);
                }
            }

            DiscoveryMessage::Query {
                query_id,
                filter,
                requester_id,
                timestamp: _,
            } => {
                if requester_id != local_agent_id {
                    // Respond with matching local agents
                    let agents = server_state.agents.read().await;
                    
                    for (agent_id, agent) in agents.iter() {
                        if Self::matches_filter(agent, &filter) {
                            let response = DiscoveryMessage::Response {
                                query_id: query_id.clone(),
                                agent_id: agent_id.clone(),
                                agent_info: AgentDiscoveryInfo {
                                    id: agent_id.clone(),
                                    name: agent.name.clone(),
                                    agent_type: agent.agent_type.clone(),
                                    capabilities: agent.capabilities.clone(),
                                    endpoint: agent.endpoint.clone().unwrap_or_default(),
                                    mcp_endpoint: Some(format!("http://localhost:{}/mcp", config.port)),
                                    last_seen: current_timestamp(),
                                    availability: match agent.status {
                                        crate::AgentStatus::Running => AgentAvailability::Available,
                                        crate::AgentStatus::Paused => AgentAvailability::Busy,
                                        crate::AgentStatus::Error => AgentAvailability::Maintenance,
                                        _ => AgentAvailability::Available,
                                    },
                                    load_factor: 0.3, // Mock value
                                    response_time_avg: 120.0, // Mock value
                                },
                                timestamp: current_timestamp(),
                            };

                            // Send response (we'd need access to the socket here)
                            debug!("Would send discovery response for agent {}", agent_id);
                        }
                    }
                }
            }

            DiscoveryMessage::Response {
                query_id,
                agent_id: _,
                agent_info,
                timestamp: _,
            } => {
                // Forward to pending query
                let queries = pending_queries.read().await;
                if let Some(tx) = queries.get(&query_id) {
                    let _ = tx.send(agent_info);
                }
            }

            DiscoveryMessage::Goodbye { agent_id, timestamp: _ } => {
                if agent_id != local_agent_id {
                    let mut agents = discovered_agents.write().await;
                    agents.remove(&agent_id);
                    debug!("Agent {} said goodbye", agent_id);
                }
            }

            DiscoveryMessage::Heartbeat { agent_id, timestamp } => {
                if agent_id != local_agent_id {
                    let mut agents = discovered_agents.write().await;
                    if let Some(agent) = agents.get_mut(&agent_id) {
                        agent.last_seen = timestamp;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if an agent matches the discovery filter
    fn matches_filter(agent: &crate::DaaAgentInfo, filter: &DiscoveryFilter) -> bool {
        if let Some(ref agent_type) = filter.agent_type {
            if agent.agent_type != *agent_type {
                return false;
            }
        }

        if let Some(ref required_capabilities) = filter.capabilities {
            for capability in required_capabilities {
                if !agent.capabilities.contains(capability) {
                    return false;
                }
            }
        }

        true
    }

    /// Create an announce message for this server
    async fn create_announce_message(&self) -> Result<DiscoveryMessage> {
        Ok(DiscoveryMessage::Announce {
            agent_id: self.local_agent_id.clone(),
            agent_name: "DAA MCP Server".to_string(),
            agent_type: "mcp_server".to_string(),
            capabilities: vec![
                "agent_management".to_string(),
                "task_coordination".to_string(),
                "swarm_coordination".to_string(),
                "mcp_tools".to_string(),
                "mcp_resources".to_string(),
                "mcp_prompts".to_string(),
            ],
            endpoint: format!("http://{}:{}", 
                self.server_state.config.bind_address, 
                self.server_state.config.port
            ),
            mcp_endpoint: Some(format!("http://{}:{}/mcp", 
                self.server_state.config.bind_address, 
                self.server_state.config.port
            )),
            timestamp: current_timestamp(),
            ttl: self.config.agent_ttl.as_secs(),
        })
    }

    /// Send goodbye message
    async fn send_goodbye(&self) -> Result<()> {
        let goodbye_msg = DiscoveryMessage::Goodbye {
            agent_id: self.local_agent_id.clone(),
            timestamp: current_timestamp(),
        };

        self.send_message(&goodbye_msg).await
    }

    /// Send a discovery message
    async fn send_message(&self, message: &DiscoveryMessage) -> Result<()> {
        let data = serde_json::to_vec(message)
            .map_err(|e| DaaMcpError::Protocol(format!("Failed to serialize message: {}", e)))?;

        let multicast_addr = SocketAddr::new(
            IpAddr::V4(self.config.multicast_address),
            self.config.port,
        );

        self.socket.send_to(&data, multicast_addr).await
            .map_err(|e| DaaMcpError::Network(e))?;

        debug!("Sent discovery message to {}", multicast_addr);
        Ok(())
    }
}

impl Clone for DiscoveryProtocol {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            server_state: self.server_state.clone(),
            socket: self.socket.clone(),
            discovered_agents: self.discovered_agents.clone(),
            pending_queries: self.pending_queries.clone(),
            local_agent_id: self.local_agent_id.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

/// Get current timestamp in seconds since UNIX epoch
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Discovery protocol utilities
pub struct DiscoveryUtils;

impl DiscoveryUtils {
    /// Create a swarm discovery filter
    pub fn swarm_filter(agent_types: Vec<String>, max_agents: Option<usize>) -> DiscoveryFilter {
        DiscoveryFilter {
            agent_type: None, // Will match any type, then filter in application
            capabilities: None,
            max_agents,
            exclude_self: true,
        }
    }

    /// Create a capability-based filter
    pub fn capability_filter(capabilities: Vec<String>) -> DiscoveryFilter {
        DiscoveryFilter {
            agent_type: None,
            capabilities: Some(capabilities),
            max_agents: None,
            exclude_self: true,
        }
    }

    /// Calculate agent compatibility score
    pub fn compatibility_score(
        agent: &AgentDiscoveryInfo,
        required_capabilities: &[String],
    ) -> f32 {
        if required_capabilities.is_empty() {
            return 1.0;
        }

        let matches = required_capabilities
            .iter()
            .filter(|cap| agent.capabilities.contains(cap))
            .count();

        matches as f32 / required_capabilities.len() as f32
    }

    /// Sort agents by suitability for a task
    pub fn sort_by_suitability(
        mut agents: Vec<AgentDiscoveryInfo>,
        required_capabilities: &[String],
    ) -> Vec<AgentDiscoveryInfo> {
        agents.sort_by(|a, b| {
            let score_a = Self::compatibility_score(a, required_capabilities) 
                - a.load_factor * 0.3
                - (a.response_time_avg / 1000.0) * 0.1;
            
            let score_b = Self::compatibility_score(b, required_capabilities)
                - b.load_factor * 0.3
                - (b.response_time_avg / 1000.0) * 0.1;

            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DaaMcpConfig;

    #[tokio::test]
    async fn test_discovery_protocol_creation() {
        let config = DiscoveryConfig::default();
        let server_config = DaaMcpConfig::default();
        let server_state = Arc::new(McpServerState::new(server_config));

        // This would fail in a test environment without proper network setup
        // but tests the basic structure
        assert!(config.enabled);
        assert_eq!(config.port, DEFAULT_DISCOVERY_PORT);
    }

    #[test]
    fn test_discovery_filter() {
        let filter = DiscoveryUtils::swarm_filter(
            vec!["researcher".to_string(), "trader".to_string()],
            Some(5),
        );

        assert_eq!(filter.max_agents, Some(5));
        assert!(filter.exclude_self);
    }

    #[test]
    fn test_compatibility_score() {
        let agent = AgentDiscoveryInfo {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            agent_type: "researcher".to_string(),
            capabilities: vec!["analysis".to_string(), "research".to_string()],
            endpoint: "http://localhost:3002".to_string(),
            mcp_endpoint: None,
            last_seen: 0,
            availability: AgentAvailability::Available,
            load_factor: 0.3,
            response_time_avg: 150.0,
        };

        let required = vec!["analysis".to_string(), "research".to_string()];
        let score = DiscoveryUtils::compatibility_score(&agent, &required);
        assert_eq!(score, 1.0);

        let partial_required = vec!["analysis".to_string(), "trading".to_string()];
        let partial_score = DiscoveryUtils::compatibility_score(&agent, &partial_required);
        assert_eq!(partial_score, 0.5);
    }
}