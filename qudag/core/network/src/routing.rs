use crate::discovery::{
    DiscoveredPeer, GeoPreferences, LoadBalancer, LoadBalancingAlgorithm, PeerScoringConfig,
    PeerSelector,
};
use crate::shadow_address::{ShadowAddress, ShadowAddressError, ShadowAddressResolver};
use libp2p::PeerId as LibP2PPeerId;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

/// Errors that can occur during routing operations
#[derive(Error, Debug)]
pub enum RoutingError {
    #[error("No route available to destination")]
    NoRoute,
    #[error("Message too large: {size} bytes exceeds limit of {limit} bytes")]
    MessageTooLarge { size: usize, limit: usize },
    #[error("Channel send error")]
    ChannelError,
    #[error("Shadow address error: {0}")]
    ShadowAddressError(#[from] ShadowAddressError),
    #[error("Load balancer error: {0}")]
    LoadBalancerError(String),
    #[error("Peer selection error: {0}")]
    PeerSelectionError(String),
    #[error("Route optimization failed: {0}")]
    RouteOptimizationError(String),
    #[error("Dark addressing not available")]
    DarkAddressingUnavailable,
    #[error("Circuit breaker is open for peer {peer_id}")]
    CircuitBreakerOpen { peer_id: String },
    #[error("All peers are overloaded")]
    AllPeersOverloaded,
    #[error("Geographic constraints cannot be satisfied")]
    GeographicConstraintsUnsatisfied,
    #[error("Network topology insufficient for routing")]
    TopologyInsufficient,
}

/// Message destination type
#[derive(Debug, Clone)]
pub enum Destination {
    /// Direct peer routing
    Peer(LibP2PPeerId),
    /// Shadow address routing
    Shadow(ShadowAddress),
}

impl From<LibP2PPeerId> for Destination {
    fn from(peer_id: LibP2PPeerId) -> Self {
        Destination::Peer(peer_id)
    }
}

impl From<ShadowAddress> for Destination {
    fn from(addr: ShadowAddress) -> Self {
        Destination::Shadow(addr)
    }
}

/// Enhanced route path with comprehensive metrics and optimization
#[derive(Clone, Debug)]
pub struct RoutePath {
    /// Sequence of peer hops
    hops: Vec<LibP2PPeerId>,
    /// Expected end-to-end latency
    latency: Duration,
    /// Route reliability score (0.0 to 1.0)
    reliability: f64,
    /// Bandwidth capacity in bytes per second
    bandwidth_capacity: Option<u64>,
    /// Load factor for this route (0.0 to 1.0)
    #[allow(dead_code)]
    load_factor: f64,
    /// Geographic diversity score
    #[allow(dead_code)]
    geographic_diversity: f64,
    /// Security level (based on encryption and peer reputation)
    #[allow(dead_code)]
    security_level: SecurityLevel,
    /// Route cost (for optimization algorithms)
    #[allow(dead_code)]
    cost: f64,
    /// Route creation timestamp
    created_at: Instant,
    /// Route last used timestamp
    #[allow(dead_code)]
    last_used: Option<Instant>,
    /// Usage count
    #[allow(dead_code)]
    usage_count: u64,
    /// Success rate for this route
    #[allow(dead_code)]
    success_rate: f64,
    /// Dark addressing support
    supports_dark_addressing: bool,
    /// Onion routing capability
    supports_onion_routing: bool,
}

/// Security levels for routing paths
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    /// Basic security (standard encryption)
    Basic,
    /// Enhanced security (quantum-resistant)
    Enhanced,
    /// Maximum security (onion routing + dark addressing)
    Maximum,
}

/// Route selection criteria for different use cases
#[derive(Debug, Clone)]
pub struct RouteSelectionCriteria {
    /// Maximum acceptable latency
    max_latency: Option<Duration>,
    /// Minimum reliability requirement
    min_reliability: f64,
    /// Required security level
    #[allow(dead_code)]
    required_security: SecurityLevel,
    /// Bandwidth requirements in bps
    min_bandwidth: Option<u64>,
    /// Geographic constraints
    #[allow(dead_code)]
    geographic_constraints: GeographicConstraints,
    /// Load balancing preferences
    #[allow(dead_code)]
    load_balancing_preference: LoadBalancingPreference,
    /// Redundancy requirements
    redundancy_level: RedundancyLevel,
    /// Dark addressing requirement
    require_dark_addressing: bool,
    /// Onion routing requirement
    require_onion_routing: bool,
}

/// Geographic constraints for routing
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GeographicConstraints {
    /// Preferred regions (ISO country codes)
    preferred_regions: Vec<String>,
    /// Excluded regions
    excluded_regions: Vec<String>,
    /// Maximum distance from source in kilometers
    max_distance_km: Option<f64>,
    /// Require geographic diversity in multi-hop routes
    require_diversity: bool,
}

/// Load balancing preferences for route selection
#[derive(Debug, Clone)]
pub enum LoadBalancingPreference {
    /// Prefer routes with lowest load
    LowLoad,
    /// Distribute load evenly across available routes
    EvenDistribution,
    /// Use weighted distribution based on peer capacity
    WeightedCapacity,
    /// Adaptive based on current network conditions
    Adaptive,
}

/// Redundancy levels for fault tolerance
#[derive(Debug, Clone)]
pub enum RedundancyLevel {
    /// No redundancy required
    None,
    /// Basic redundancy (backup route)
    Basic,
    /// High redundancy (multiple disjoint paths)
    High,
    /// Maximum redundancy (full mesh of paths)
    Maximum,
}

impl RedundancyLevel {
    /// Get the number of paths required for this redundancy level
    pub fn path_count(&self) -> usize {
        match self {
            RedundancyLevel::None => 1,
            RedundancyLevel::Basic => 2,
            RedundancyLevel::High => 3,
            RedundancyLevel::Maximum => 5,
        }
    }
}

/// Production-ready multi-path router with advanced load balancing and dark addressing
pub struct Router {
    /// Simple peer connectivity graph
    peer_connections: Arc<RwLock<HashMap<LibP2PPeerId, HashSet<LibP2PPeerId>>>>,
    /// Discovered peers with comprehensive metrics
    peers: Arc<RwLock<HashMap<LibP2PPeerId, DiscoveredPeer>>>,
    /// Route cache with performance metrics
    route_cache: Arc<RwLock<HashMap<LibP2PPeerId, Vec<RoutePath>>>>,
    /// Message channel for routing
    message_tx: mpsc::Sender<Vec<u8>>,
    /// Shadow address resolver
    shadow_resolver: Option<Box<dyn ShadowAddressResolver + Send + Sync>>,
    /// Load balancer for peer selection
    load_balancer: Arc<Mutex<LoadBalancer>>,
    /// Peer selector with geographic awareness
    peer_selector: Arc<Mutex<PeerSelector>>,
    /// Scoring configuration
    scoring_config: PeerScoringConfig,
    /// Route optimization settings
    optimization_config: RouteOptimizationConfig,
    /// Dark addressing configuration
    dark_addressing_config: DarkAddressingConfig,
    /// Performance monitoring
    performance_metrics: Arc<Mutex<RouterPerformanceMetrics>>,
    /// Route usage statistics
    route_stats: Arc<Mutex<HashMap<LibP2PPeerId, RouteStatistics>>>,
    /// Maximum message size
    max_message_size: usize,
    /// Circuit breaker states
    circuit_breakers: Arc<RwLock<HashMap<LibP2PPeerId, CircuitBreakerState>>>,
}

/// Route optimization configuration
#[derive(Debug, Clone)]
pub struct RouteOptimizationConfig {
    /// Enable automatic route optimization
    #[allow(dead_code)]
    enable_optimization: bool,
    /// Optimization interval
    #[allow(dead_code)]
    optimization_interval: Duration,
    /// Route cache size
    #[allow(dead_code)]
    cache_size: usize,
    /// Route cache TTL
    cache_ttl: Duration,
    /// Prefer shorter paths
    #[allow(dead_code)]
    prefer_shorter_paths: bool,
    /// Weight factors for route selection
    weight_factors: RouteWeightFactors,
    /// Enable adaptive routing
    #[allow(dead_code)]
    enable_adaptive_routing: bool,
}

impl Default for RouteOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_optimization: true,
            optimization_interval: Duration::from_secs(60),
            cache_size: 1000,
            cache_ttl: Duration::from_secs(300),
            prefer_shorter_paths: true,
            weight_factors: RouteWeightFactors::default(),
            enable_adaptive_routing: true,
        }
    }
}

/// Weight factors for route scoring
#[derive(Debug, Clone)]
pub struct RouteWeightFactors {
    /// Latency weight (0.0 to 1.0)
    latency_weight: f64,
    /// Reliability weight (0.0 to 1.0)
    reliability_weight: f64,
    /// Load weight (0.0 to 1.0)
    load_weight: f64,
    /// Security weight (0.0 to 1.0)
    #[allow(dead_code)]
    security_weight: f64,
    /// Geographic diversity weight (0.0 to 1.0)
    #[allow(dead_code)]
    diversity_weight: f64,
}

impl Default for RouteWeightFactors {
    fn default() -> Self {
        Self {
            latency_weight: 0.3,
            reliability_weight: 0.3,
            load_weight: 0.2,
            security_weight: 0.1,
            diversity_weight: 0.1,
        }
    }
}

/// Dark addressing configuration for routing
#[derive(Debug, Clone)]
pub struct DarkAddressingConfig {
    /// Enable dark addressing support
    enabled: bool,
    /// Preferred dark address resolution timeout
    #[allow(dead_code)]
    resolution_timeout: Duration,
    /// Maximum resolution attempts
    #[allow(dead_code)]
    max_resolution_attempts: usize,
    /// Cache resolved addresses
    #[allow(dead_code)]
    enable_caching: bool,
    /// Dark address cache TTL
    #[allow(dead_code)]
    cache_ttl: Duration,
}

impl Default for DarkAddressingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            resolution_timeout: Duration::from_secs(10),
            max_resolution_attempts: 3,
            enable_caching: true,
            cache_ttl: Duration::from_secs(600),
        }
    }
}

/// Router performance metrics
#[derive(Debug, Clone, Default)]
pub struct RouterPerformanceMetrics {
    /// Total messages routed
    total_messages: u64,
    /// Successful routings
    successful_routings: u64,
    /// Failed routings
    #[allow(dead_code)]
    failed_routings: u64,
    /// Average routing latency
    #[allow(dead_code)]
    avg_routing_latency: Duration,
    /// Route cache hit rate
    cache_hit_rate: f64,
    /// Load balancing effectiveness
    #[allow(dead_code)]
    load_balancing_score: f64,
    /// Dark addressing usage rate
    #[allow(dead_code)]
    dark_addressing_usage: f64,
}

/// Route usage statistics for optimization
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RouteStatistics {
    /// Route usage count
    usage_count: u64,
    /// Success count
    success_count: u64,
    /// Failure count
    failure_count: u64,
    /// Average latency
    avg_latency: Duration,
    /// Last used timestamp
    last_used: Option<Instant>,
    /// Bandwidth utilization
    bandwidth_utilization: f64,
}

/// Circuit breaker state for fault tolerance
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// Circuit closed (normal operation)
    Closed,
    /// Circuit open (failures detected)
    Open {
        opened_at: Instant,
        failure_count: usize,
    },
    /// Circuit half-open (testing recovery)
    HalfOpen {
        test_count: usize,
        success_count: usize,
    },
}

impl Router {
    /// Creates a new router instance with default configuration
    pub fn new(message_tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self::new_with_config(
            message_tx,
            PeerScoringConfig::default(),
            RouteOptimizationConfig::default(),
            DarkAddressingConfig::default(),
            16 * 1024 * 1024, // 16MB default
        )
    }

    /// Creates a new production-ready router instance with custom configuration
    pub fn new_with_config(
        message_tx: mpsc::Sender<Vec<u8>>,
        scoring_config: PeerScoringConfig,
        optimization_config: RouteOptimizationConfig,
        dark_addressing_config: DarkAddressingConfig,
        max_message_size: usize,
    ) -> Self {
        Self {
            peer_connections: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            route_cache: Arc::new(RwLock::new(HashMap::new())),
            message_tx,
            shadow_resolver: None,
            load_balancer: Arc::new(Mutex::new(LoadBalancer::new(
                LoadBalancingAlgorithm::WeightedRoundRobin,
            ))),
            peer_selector: Arc::new(Mutex::new(PeerSelector::new(GeoPreferences::default()))),
            scoring_config,
            optimization_config,
            dark_addressing_config,
            performance_metrics: Arc::new(Mutex::new(RouterPerformanceMetrics::default())),
            route_stats: Arc::new(Mutex::new(HashMap::new())),
            max_message_size,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the shadow address resolver
    pub fn set_shadow_resolver(&mut self, resolver: Box<dyn ShadowAddressResolver + Send + Sync>) {
        self.shadow_resolver = Some(resolver);
    }

    /// Find paths for a shadow address
    async fn find_shadow_paths(
        &self,
        addr: &ShadowAddress,
    ) -> Result<Vec<RoutePath>, RoutingError> {
        // Resolve shadow address to onetime address
        let _resolved = if let Some(resolver) = &self.shadow_resolver {
            resolver.resolve_address(addr)?
        } else {
            return Err(RoutingError::DarkAddressingUnavailable);
        };

        // Get available peers
        let peers = self.peers.read().await;
        let mut available_peers: Vec<_> = peers.keys().cloned().collect();

        if available_peers.is_empty() {
            return Err(RoutingError::NoRoute);
        }

        // Find random set of peers to use as intermediaries
        let mut rng = thread_rng();
        available_peers.shuffle(&mut rng);

        let peer_count = 3; // Use 3 intermediate hops
        let selected_peers: Vec<_> = available_peers.into_iter().take(peer_count).collect();

        if selected_peers.len() < peer_count {
            return Err(RoutingError::NoRoute);
        }

        // Create path through selected peers with dark addressing support
        Ok(vec![RoutePath {
            hops: selected_peers,
            latency: Duration::from_millis(50),
            reliability: 0.95,
            bandwidth_capacity: None,
            load_factor: 0.5,
            geographic_diversity: 1.0,
            security_level: SecurityLevel::Maximum,
            cost: 1.0,
            created_at: Instant::now(),
            last_used: None,
            usage_count: 0,
            success_rate: 0.0,
            supports_dark_addressing: true,
            supports_onion_routing: true,
        }])
    }

    /// Adds a peer connection to the routing table
    pub fn add_peer_connection(&mut self, from: LibP2PPeerId, to: LibP2PPeerId) {
        let mut connections = self.peer_connections.blocking_write();
        connections
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    /// Removes a peer connection from the routing table
    pub fn remove_peer_connection(&mut self, from: LibP2PPeerId, to: LibP2PPeerId) {
        let mut connections = self.peer_connections.blocking_write();
        if let Some(peer_connections) = connections.get_mut(&from) {
            peer_connections.remove(&to);
            if peer_connections.is_empty() {
                connections.remove(&from);
            }
        }
    }

    /// Adds a discovered peer with full metrics
    pub async fn add_discovered_peer(&self, peer_id: LibP2PPeerId, peer: DiscoveredPeer) {
        self.peers.write().await.insert(peer_id, peer);
    }

    /// Removes a discovered peer
    pub async fn remove_discovered_peer(&self, peer_id: LibP2PPeerId) {
        self.peers.write().await.remove(&peer_id);
    }

    /// Updates path metrics for a peer
    pub async fn update_path_metrics(&self, peer_id: LibP2PPeerId, path: RoutePath) {
        let mut cache = self.route_cache.write().await;
        cache.entry(peer_id).or_insert_with(Vec::new).push(path);

        // Keep only the most recent paths
        if let Some(paths) = cache.get_mut(&peer_id) {
            if paths.len() > 10 {
                paths.remove(0);
            }
        }
    }

    /// Finds multiple disjoint paths to a destination
    pub async fn find_paths(
        &self,
        destination: LibP2PPeerId,
        criteria: &RouteSelectionCriteria,
    ) -> Result<Vec<RoutePath>, RoutingError> {
        // Check cache first
        let cache = self.route_cache.read().await;
        if let Some(cached_paths) = cache.get(&destination) {
            let valid_paths: Vec<_> = cached_paths
                .iter()
                .filter(|p| p.created_at.elapsed() < self.optimization_config.cache_ttl)
                .filter(|p| self.meets_criteria(p, criteria))
                .cloned()
                .collect();

            if !valid_paths.is_empty() {
                let mut metrics = self.performance_metrics.lock().unwrap();
                metrics.cache_hit_rate = (metrics.cache_hit_rate + 1.0) / 2.0;
                return Ok(valid_paths);
            }
        }
        drop(cache);

        // Find new paths
        let peers = self.peers.read().await;
        let available_peers: Vec<_> = peers.values().filter(|p| p.is_healthy()).collect();

        if available_peers.is_empty() {
            return Err(RoutingError::NoRoute);
        }

        // Use peer selector to find suitable peers
        let mut peer_selector = self.peer_selector.lock().unwrap();
        let candidates: Vec<DiscoveredPeer> = available_peers.into_iter().cloned().collect();
        let selected_peer_ids = peer_selector.select_peers(
            &candidates,
            criteria.redundancy_level.path_count(),
            &self.scoring_config,
        );

        // Build paths based on redundancy level
        let mut paths = Vec::new();
        for peer_id in selected_peer_ids {
            let path = self.build_path_to_peer(peer_id, &peers, criteria).await?;
            paths.push(path);
        }

        // Cache the paths
        let mut cache = self.route_cache.write().await;
        cache.insert(destination, paths.clone());

        // Update metrics
        let mut metrics = self.performance_metrics.lock().unwrap();
        metrics.total_messages += 1;

        Ok(paths)
    }

    /// Build a path to a specific peer
    async fn build_path_to_peer(
        &self,
        destination: LibP2PPeerId,
        peers: &HashMap<LibP2PPeerId, DiscoveredPeer>,
        criteria: &RouteSelectionCriteria,
    ) -> Result<RoutePath, RoutingError> {
        let hops = if criteria.require_onion_routing {
            // Build multi-hop path for onion routing
            self.select_onion_hops(destination, peers, 3)?
        } else {
            // Direct path
            vec![destination]
        };

        let latency = self.calculate_path_latency(&hops, peers);
        let reliability = self.calculate_path_reliability(&hops, peers);
        let bandwidth = self.calculate_path_bandwidth(&hops, peers);
        let load_factor = self.calculate_path_load(&hops, peers);
        let security_level = if criteria.require_onion_routing {
            SecurityLevel::Maximum
        } else if criteria.require_dark_addressing {
            SecurityLevel::Enhanced
        } else {
            SecurityLevel::Basic
        };

        Ok(RoutePath {
            hops,
            latency,
            reliability,
            bandwidth_capacity: bandwidth,
            load_factor,
            geographic_diversity: 1.0, // TODO: Calculate actual diversity
            security_level,
            cost: self.calculate_path_cost(latency, reliability, load_factor),
            created_at: Instant::now(),
            last_used: None,
            usage_count: 0,
            success_rate: 0.0,
            supports_dark_addressing: criteria.require_dark_addressing,
            supports_onion_routing: criteria.require_onion_routing,
        })
    }

    /// Select hops for onion routing
    fn select_onion_hops(
        &self,
        destination: LibP2PPeerId,
        peers: &HashMap<LibP2PPeerId, DiscoveredPeer>,
        hop_count: usize,
    ) -> Result<Vec<LibP2PPeerId>, RoutingError> {
        let mut available: Vec<_> = peers
            .iter()
            .filter(|(id, p)| **id != destination && p.capabilities.can_relay)
            .map(|(id, _)| *id)
            .collect();

        if available.len() < hop_count {
            return Err(RoutingError::TopologyInsufficient);
        }

        let mut rng = thread_rng();
        available.shuffle(&mut rng);

        let mut hops = available.into_iter().take(hop_count).collect::<Vec<_>>();
        hops.push(destination);

        Ok(hops)
    }

    /// Check if a path meets the selection criteria
    fn meets_criteria(&self, path: &RoutePath, criteria: &RouteSelectionCriteria) -> bool {
        if let Some(max_latency) = criteria.max_latency {
            if path.latency > max_latency {
                return false;
            }
        }

        if path.reliability < criteria.min_reliability {
            return false;
        }

        if let Some(min_bandwidth) = criteria.min_bandwidth {
            if let Some(bandwidth) = path.bandwidth_capacity {
                if bandwidth < min_bandwidth {
                    return false;
                }
            } else {
                return false;
            }
        }

        if criteria.require_dark_addressing && !path.supports_dark_addressing {
            return false;
        }

        if criteria.require_onion_routing && !path.supports_onion_routing {
            return false;
        }

        true
    }

    /// Calculate latency for a path
    fn calculate_path_latency(
        &self,
        hops: &[LibP2PPeerId],
        peers: &HashMap<LibP2PPeerId, DiscoveredPeer>,
    ) -> Duration {
        let mut total_latency = Duration::ZERO;

        for hop in hops {
            if let Some(peer) = peers.get(hop) {
                total_latency += peer.performance_metrics.avg_response_time;
            } else {
                total_latency += Duration::from_millis(50); // Default estimate
            }
        }

        total_latency
    }

    /// Calculate reliability for a path
    fn calculate_path_reliability(
        &self,
        hops: &[LibP2PPeerId],
        peers: &HashMap<LibP2PPeerId, DiscoveredPeer>,
    ) -> f64 {
        let mut reliability = 1.0;

        for hop in hops {
            if let Some(peer) = peers.get(hop) {
                reliability *= peer.connection_quality.reliability_score;
            } else {
                reliability *= 0.9; // Default estimate
            }
        }

        reliability
    }

    /// Calculate bandwidth for a path
    fn calculate_path_bandwidth(
        &self,
        hops: &[LibP2PPeerId],
        peers: &HashMap<LibP2PPeerId, DiscoveredPeer>,
    ) -> Option<u64> {
        let mut min_bandwidth = u64::MAX;

        for hop in hops {
            if let Some(peer) = peers.get(hop) {
                if let Some(bw) = peer.capabilities.bandwidth_capacity {
                    min_bandwidth = min_bandwidth.min(bw);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        if min_bandwidth == u64::MAX {
            None
        } else {
            Some(min_bandwidth)
        }
    }

    /// Calculate load factor for a path
    fn calculate_path_load(
        &self,
        hops: &[LibP2PPeerId],
        peers: &HashMap<LibP2PPeerId, DiscoveredPeer>,
    ) -> f64 {
        let mut total_load = 0.0;

        for hop in hops {
            if let Some(peer) = peers.get(hop) {
                total_load += peer.load_metrics.load_score / 100.0;
            } else {
                total_load += 0.5; // Default estimate
            }
        }

        total_load / hops.len() as f64
    }

    /// Calculate path cost
    fn calculate_path_cost(&self, latency: Duration, reliability: f64, load_factor: f64) -> f64 {
        let weights = &self.optimization_config.weight_factors;

        let latency_cost = latency.as_millis() as f64 / 1000.0; // Convert to seconds
        let reliability_cost = 1.0 - reliability;
        let load_cost = load_factor;

        latency_cost * weights.latency_weight
            + reliability_cost * weights.reliability_weight
            + load_cost * weights.load_weight
    }

    /// Routes a message through multiple paths using either PeerId or ShadowAddress
    pub async fn route_message(
        &self,
        destination: impl Into<Destination>,
        message: Vec<u8>,
    ) -> Result<(), RoutingError> {
        // Check message size
        if message.len() > self.max_message_size {
            return Err(RoutingError::MessageTooLarge {
                size: message.len(),
                limit: self.max_message_size,
            });
        }

        let dest = destination.into();
        let criteria = RouteSelectionCriteria {
            max_latency: None,
            min_reliability: 0.5,
            required_security: SecurityLevel::Basic,
            min_bandwidth: None,
            geographic_constraints: GeographicConstraints {
                preferred_regions: vec![],
                excluded_regions: vec![],
                max_distance_km: None,
                require_diversity: false,
            },
            load_balancing_preference: LoadBalancingPreference::Adaptive,
            redundancy_level: RedundancyLevel::Basic,
            require_dark_addressing: false,
            require_onion_routing: false,
        };

        // Get routing paths based on destination type
        let paths = match dest {
            Destination::Peer(peer_id) => {
                // Check circuit breaker
                let breakers = self.circuit_breakers.read().await;
                if let Some(state) = breakers.get(&peer_id) {
                    if matches!(state, CircuitBreakerState::Open { .. }) {
                        return Err(RoutingError::CircuitBreakerOpen {
                            peer_id: peer_id.to_string(),
                        });
                    }
                }
                drop(breakers);

                self.find_paths(peer_id, &criteria).await?
            }
            Destination::Shadow(shadow_addr) => {
                if !self.dark_addressing_config.enabled {
                    return Err(RoutingError::DarkAddressingUnavailable);
                }
                self.find_shadow_paths(&shadow_addr).await?
            }
        };

        if paths.is_empty() {
            return Err(RoutingError::NoRoute);
        }

        // Update performance metrics
        let mut metrics = self.performance_metrics.lock().unwrap();
        metrics.total_messages += 1;

        // Use load balancer to select best path
        let mut load_balancer = self.load_balancer.lock().unwrap();
        let selected_path = if paths.len() == 1 {
            &paths[0]
        } else {
            // Get peer IDs from paths
            let peer_ids: Vec<_> = paths
                .iter()
                .filter_map(|p| p.hops.first())
                .copied()
                .collect();

            if let Some(selected_peer) = load_balancer.select_peer(&peer_ids) {
                paths
                    .iter()
                    .find(|p| p.hops.first() == Some(&selected_peer))
                    .unwrap_or(&paths[0])
            } else {
                &paths[0]
            }
        };

        // Update route statistics
        if let Some(first_hop) = selected_path.hops.first() {
            let mut route_stats = self.route_stats.lock().unwrap();
            let stats = route_stats.entry(*first_hop).or_default();
            stats.usage_count += 1;
            stats.last_used = Some(Instant::now());
        }

        // Build routing header
        let mut routed_message = Vec::new();

        // Header format:
        // - Path length (4 bytes)
        // - Path hops (variable)
        // - Message data

        routed_message.extend_from_slice(&(selected_path.hops.len() as u32).to_le_bytes());

        for hop in &selected_path.hops {
            routed_message.extend_from_slice(hop.to_bytes().as_slice());
        }

        routed_message.extend_from_slice(&message);

        // Send through channel
        self.message_tx
            .send(routed_message)
            .await
            .map_err(|_| RoutingError::ChannelError)?;

        metrics.successful_routings += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadow_address::{NetworkType, ShadowMetadata};
    use tokio::sync::mpsc;

    // Mock shadow address resolver for testing
    struct MockResolver;

    impl ShadowAddressResolver for MockResolver {
        fn resolve_address(&self, _: &ShadowAddress) -> Result<Vec<u8>, ShadowAddressError> {
            Ok(vec![1, 2, 3, 4])
        }

        fn check_address(
            &self,
            _: &ShadowAddress,
            onetime: &[u8],
        ) -> Result<bool, ShadowAddressError> {
            Ok(onetime == &[1, 2, 3, 4])
        }
    }

    fn setup_test_router() -> (Router, mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(128);
        let mut router = Router::new_with_config(
            tx,
            PeerScoringConfig::default(),
            RouteOptimizationConfig::default(),
            DarkAddressingConfig::default(),
            1024 * 1024, // 1MB max message size
        );
        router.set_shadow_resolver(Box::new(MockResolver));
        (router, rx)
    }

    fn create_test_shadow_address() -> ShadowAddress {
        ShadowAddress {
            view_key: vec![1, 2, 3, 4],
            spend_key: vec![5, 6, 7, 8],
            payment_id: None,
            metadata: ShadowMetadata {
                version: 1,
                network: NetworkType::Testnet,
                expires_at: None,
                flags: 0,
            },
        }
    }

    #[tokio::test]
    async fn test_add_remove_peer() {
        let (router, _) = setup_test_router();
        let peer1 = LibP2PPeerId::random();
        let peer2 = DiscoveredPeer::new(
            LibP2PPeerId::random(),
            "127.0.0.1:8000".parse().unwrap(),
            DiscoveryMethod::Static,
        );

        router.add_peer_connection(peer1, peer2.clone()).await;
        assert!(router.peers.read().await.contains_key(&peer1));

        router.remove_peer_connection(peer1).await;
        assert!(!router.peers.read().await.contains_key(&peer1));
    }

    #[tokio::test]
    async fn test_route_message() {
        let (router, mut rx) = setup_test_router();
        let peer1 = LibP2PPeerId::random();
        let peer2 = LibP2PPeerId::random();
        let peer3 = LibP2PPeerId::random();

        // Set up peers
        let discovered_peer1 = DiscoveredPeer::new(
            peer1,
            "127.0.0.1:8001".parse().unwrap(),
            DiscoveryMethod::Static,
        );
        let discovered_peer2 = DiscoveredPeer::new(
            peer2,
            "127.0.0.1:8002".parse().unwrap(),
            DiscoveryMethod::Static,
        );
        let discovered_peer3 = DiscoveredPeer::new(
            peer3,
            "127.0.0.1:8003".parse().unwrap(),
            DiscoveryMethod::Static,
        );

        router.add_peer_connection(peer1, discovered_peer1).await;
        router.add_peer_connection(peer2, discovered_peer2).await;
        router.add_peer_connection(peer3, discovered_peer3).await;

        let test_msg = vec![1, 2, 3, 4];
        router.route_message(peer3, test_msg.clone()).await.unwrap();

        // Verify message was sent
        let received = rx.recv().await.unwrap();
        assert!(!received.is_empty());
    }

    #[tokio::test]
    async fn test_find_paths() {
        let (router, _) = setup_test_router();
        let peer1 = LibP2PPeerId::random();
        let peer2 = LibP2PPeerId::random();
        let peer3 = LibP2PPeerId::random();

        // Set up peers
        let discovered_peer1 = DiscoveredPeer::new(
            peer1,
            "127.0.0.1:8001".parse().unwrap(),
            DiscoveryMethod::Static,
        );
        let discovered_peer2 = DiscoveredPeer::new(
            peer2,
            "127.0.0.1:8002".parse().unwrap(),
            DiscoveryMethod::Static,
        );
        let discovered_peer3 = DiscoveredPeer::new(
            peer3,
            "127.0.0.1:8003".parse().unwrap(),
            DiscoveryMethod::Static,
        );

        router.add_peer_connection(peer1, discovered_peer1).await;
        router.add_peer_connection(peer2, discovered_peer2).await;
        router.add_peer_connection(peer3, discovered_peer3).await;

        let criteria = RouteSelectionCriteria {
            max_latency: None,
            min_reliability: 0.5,
            required_security: SecurityLevel::Basic,
            min_bandwidth: None,
            geographic_constraints: GeographicConstraints {
                preferred_regions: vec![],
                excluded_regions: vec![],
                max_distance_km: None,
                require_diversity: false,
            },
            load_balancing_preference: LoadBalancingPreference::Adaptive,
            redundancy_level: RedundancyLevel::Basic,
            require_dark_addressing: false,
            require_onion_routing: false,
        };
        let paths = router.find_paths(peer3, &criteria).await.unwrap();
        assert!(!paths.is_empty());
    }

    #[tokio::test]
    async fn test_route_shadow_message() {
        let (router, mut rx) = setup_test_router();
        let peer1 = LibP2PPeerId::random();
        let peer2 = LibP2PPeerId::random();
        let peer3 = LibP2PPeerId::random();

        // Set up some peers
        let discovered_peer1 = DiscoveredPeer::new(
            peer1,
            "127.0.0.1:8001".parse().unwrap(),
            DiscoveryMethod::Static,
        );
        let discovered_peer2 = DiscoveredPeer::new(
            peer2,
            "127.0.0.1:8002".parse().unwrap(),
            DiscoveryMethod::Static,
        );
        let discovered_peer3 = DiscoveredPeer::new(
            peer3,
            "127.0.0.1:8003".parse().unwrap(),
            DiscoveryMethod::Static,
        );

        router.add_peer_connection(peer1, discovered_peer1).await;
        router.add_peer_connection(peer2, discovered_peer2).await;
        router.add_peer_connection(peer3, discovered_peer3).await;

        // Try routing to a shadow address
        let shadow_addr = create_test_shadow_address();
        let test_msg = vec![1, 2, 3, 4];
        router
            .route_message(shadow_addr, test_msg.clone())
            .await
            .unwrap();

        // Verify message was sent
        let received = rx.recv().await.unwrap();
        assert!(!received.is_empty());
    }
}
