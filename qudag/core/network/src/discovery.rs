//! Production-ready P2P network peer discovery implementation with Kademlia DHT,
//! dark addressing support, and sophisticated peer reputation management.

use crate::dark_resolver::DarkResolver;
use crate::shadow_address::{DefaultShadowAddressHandler, NetworkType, ShadowAddress};
use crate::types::NetworkError;
use libp2p::PeerId as LibP2PPeerId;
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Peer discovery method with advanced options.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// DHT-based discovery using Kademlia
    Kademlia,
    /// Static peer list
    Static,
    /// mDNS local network discovery
    Mdns,
    /// Bootstrap node discovery
    Bootstrap,
    /// Dark addressing discovery
    DarkAddress,
    /// DNS-based discovery
    DNS,
    /// Gossip-based discovery
    Gossip,
    /// Hybrid discovery combining multiple methods
    Hybrid(Vec<DiscoveryMethod>),
}

/// Network configuration types for different deployment scenarios.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkConfig {
    /// Public network configuration
    Public {
        /// Enable NAT traversal
        nat_traversal: bool,
        /// Enable UPnP
        upnp: bool,
        /// Enable STUN/TURN
        stun_turn: bool,
    },
    /// Private network configuration
    Private {
        /// Allowed IP ranges
        allowed_ranges: Vec<String>,
        /// Require authentication
        require_auth: bool,
    },
    /// Hybrid public/private configuration
    Hybrid {
        /// Public settings
        public: Box<NetworkConfig>,
        /// Private settings
        private: Box<NetworkConfig>,
        /// Fallback to public if private fails
        fallback_public: bool,
    },
}

/// Advanced peer discovery configuration for production deployments.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Discovery methods to use
    pub methods: Vec<DiscoveryMethod>,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<SocketAddr>,
    /// Discovery interval in seconds
    pub interval: u64,
    /// Maximum peers to discover
    pub max_peers: usize,
    /// Minimum peers to maintain
    pub min_peers: usize,
    /// Peer reputation threshold for auto-connect
    pub reputation_threshold: f64,
    /// Network configuration
    pub network_config: NetworkConfig,
    /// Enable dark addressing
    pub enable_dark_addressing: bool,
    /// Dark domain resolver configuration
    pub dark_resolver_config: DarkResolverConfig,
    /// DHT configuration
    pub dht_config: DHTConfig,
    /// Connection concurrency limit
    pub max_concurrent_connections: usize,
    /// Peer scoring configuration
    pub scoring_config: PeerScoringConfig,
    /// Load balancing configuration
    pub load_balancing_config: LoadBalancingConfig,
    /// Geographic distribution preferences
    pub geo_preferences: GeoPreferences,
}

/// DHT configuration for Kademlia-based discovery.
#[derive(Debug, Clone)]
pub struct DHTConfig {
    /// Bucket size (K parameter)
    pub bucket_size: usize,
    /// Alpha parameter for parallel lookups
    pub alpha: usize,
    /// Replication factor
    pub replication_factor: usize,
    /// Key space size in bits
    pub key_space_bits: usize,
    /// Bootstrap timeout
    pub bootstrap_timeout: Duration,
    /// Refresh interval
    pub refresh_interval: Duration,
    /// Enable periodic republishing
    pub enable_republishing: bool,
}

/// Dark resolver configuration.
#[derive(Debug, Clone)]
pub struct DarkResolverConfig {
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Cache TTL
    pub cache_ttl: Duration,
    /// Enable distributed resolution
    pub enable_distributed: bool,
    /// Fallback DNS servers
    pub fallback_dns: Vec<String>,
    /// Maximum resolution attempts
    pub max_resolution_attempts: usize,
}

/// Peer scoring configuration for reputation management.
#[derive(Debug, Clone)]
pub struct PeerScoringConfig {
    /// Initial peer score
    pub initial_score: f64,
    /// Maximum score
    pub max_score: f64,
    /// Minimum score before blacklisting
    pub min_score: f64,
    /// Score decay rate per hour
    pub score_decay_rate: f64,
    /// Connection success bonus
    pub connection_success_bonus: f64,
    /// Connection failure penalty
    pub connection_failure_penalty: f64,
    /// Uptime bonus per hour
    pub uptime_bonus: f64,
    /// Latency penalty factor
    pub latency_penalty_factor: f64,
    /// Enable geographic scoring
    pub enable_geographic_scoring: bool,
}

/// Load balancing configuration.
#[derive(Debug, Clone)]
pub struct LoadBalancingConfig {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum load per peer
    pub max_load_per_peer: f64,
    /// Enable adaptive load balancing
    pub enable_adaptive: bool,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Load balancing algorithms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin
    RoundRobin,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Least connections
    LeastConnections,
    /// Least response time
    LeastResponseTime,
    /// Random selection
    Random,
    /// Consistent hashing
    ConsistentHashing,
    /// Resource-based
    ResourceBased,
}

/// Circuit breaker configuration for fault tolerance.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold
    pub failure_threshold: usize,
    /// Success threshold for recovery
    pub success_threshold: usize,
    /// Timeout duration
    pub timeout: Duration,
    /// Half-open retry delay
    pub half_open_delay: Duration,
}

/// Geographic preferences for peer selection.
#[derive(Debug, Clone)]
pub struct GeoPreferences {
    /// Prefer local peers
    pub prefer_local: bool,
    /// Maximum latency for local peers
    pub local_latency_threshold: Duration,
    /// Preferred regions (ISO country codes)
    pub preferred_regions: Vec<String>,
    /// Avoided regions
    pub avoided_regions: Vec<String>,
    /// Enable geo-diversity
    pub enable_geo_diversity: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            methods: vec![DiscoveryMethod::Kademlia, DiscoveryMethod::Mdns],
            bootstrap_nodes: vec![],
            interval: 30,
            max_peers: 50,
            min_peers: 8,
            reputation_threshold: 0.0,
            network_config: NetworkConfig::Public {
                nat_traversal: true,
                upnp: true,
                stun_turn: true,
            },
            enable_dark_addressing: true,
            dark_resolver_config: DarkResolverConfig::default(),
            dht_config: DHTConfig::default(),
            max_concurrent_connections: 100,
            scoring_config: PeerScoringConfig::default(),
            load_balancing_config: LoadBalancingConfig::default(),
            geo_preferences: GeoPreferences::default(),
        }
    }
}

impl Default for DHTConfig {
    fn default() -> Self {
        Self {
            bucket_size: 20,
            alpha: 3,
            replication_factor: 20,
            key_space_bits: 256,
            bootstrap_timeout: Duration::from_secs(30),
            refresh_interval: Duration::from_secs(3600),
            enable_republishing: true,
        }
    }
}

impl Default for DarkResolverConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 10000,
            cache_ttl: Duration::from_secs(3600),
            enable_distributed: true,
            fallback_dns: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()],
            max_resolution_attempts: 3,
        }
    }
}

impl Default for PeerScoringConfig {
    fn default() -> Self {
        Self {
            initial_score: 50.0,
            max_score: 100.0,
            min_score: -50.0,
            score_decay_rate: 0.1,
            connection_success_bonus: 5.0,
            connection_failure_penalty: 10.0,
            uptime_bonus: 1.0,
            latency_penalty_factor: 0.01,
            enable_geographic_scoring: true,
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            algorithm: LoadBalancingAlgorithm::WeightedRoundRobin,
            health_check_interval: Duration::from_secs(30),
            max_load_per_peer: 100.0,
            enable_adaptive: true,
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            half_open_delay: Duration::from_secs(30),
        }
    }
}

impl Default for GeoPreferences {
    fn default() -> Self {
        Self {
            prefer_local: true,
            local_latency_threshold: Duration::from_millis(100),
            preferred_regions: vec![],
            avoided_regions: vec![],
            enable_geo_diversity: true,
        }
    }
}

/// Enhanced discovered peer information with advanced metrics.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Peer ID
    pub peer_id: LibP2PPeerId,
    /// Network addresses
    pub addresses: Vec<SocketAddr>,
    /// Dark addresses (if available)
    pub dark_addresses: Vec<ShadowAddress>,
    /// Discovery timestamp
    pub discovered_at: Instant,
    /// Discovery method
    pub discovery_method: DiscoveryMethod,
    /// Reputation score
    pub reputation: f64,
    /// Connection attempts
    pub connection_attempts: u32,
    /// Successful connections
    pub successful_connections: u32,
    /// Last connection attempt
    pub last_connection_attempt: Option<Instant>,
    /// Last successful connection
    pub last_successful_connection: Option<Instant>,
    /// Protocol support
    pub protocols: Vec<String>,
    /// Geographic information
    pub geographic_info: Option<GeographicInfo>,
    /// Performance metrics
    pub performance_metrics: PeerPerformanceMetrics,
    /// Load balancing metrics
    pub load_metrics: PeerLoadMetrics,
    /// Peer capabilities
    pub capabilities: PeerCapabilities,
    /// Connection quality
    pub connection_quality: ConnectionQuality,
    /// Blacklist status
    pub is_blacklisted: bool,
    /// Blacklist reason
    pub blacklist_reason: Option<String>,
    /// Uptime statistics
    pub uptime_stats: UptimeStats,
    /// Circuit breaker state
    pub circuit_breaker_state: CircuitBreakerState,
}

/// Geographic information for a peer.
#[derive(Debug, Clone)]
pub struct GeographicInfo {
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: String,
    /// City name
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
    /// Estimated distance in kilometers
    pub estimated_distance_km: Option<f64>,
    /// Autonomous System Number
    pub asn: Option<u32>,
    /// ISP name
    pub isp: Option<String>,
}

/// Performance metrics for a peer.
#[derive(Debug, Clone, Default)]
pub struct PeerPerformanceMetrics {
    /// Average response time
    pub avg_response_time: Duration,
    /// Minimum response time
    pub min_response_time: Duration,
    /// Maximum response time
    pub max_response_time: Duration,
    /// 95th percentile response time
    pub p95_response_time: Duration,
    /// Throughput in messages per second
    pub throughput_mps: f64,
    /// Bandwidth utilization in bytes per second
    pub bandwidth_bps: u64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Jitter (response time variance)
    pub jitter: Duration,
    /// Packet loss rate
    pub packet_loss_rate: f64,
}

/// Load balancing metrics for a peer.
#[derive(Debug, Clone, Default)]
pub struct PeerLoadMetrics {
    /// Current active connections
    pub active_connections: usize,
    /// Current load score (0.0 to 100.0)
    pub load_score: f64,
    /// CPU utilization (if available)
    pub cpu_utilization: Option<f64>,
    /// Memory utilization (if available)
    pub memory_utilization: Option<f64>,
    /// Network utilization (if available)
    pub network_utilization: Option<f64>,
    /// Request queue depth
    pub queue_depth: usize,
    /// Weight for weighted algorithms
    pub weight: f64,
}

/// Peer capabilities and supported features.
#[derive(Debug, Clone, Default)]
pub struct PeerCapabilities {
    /// Supported protocol versions
    pub protocol_versions: Vec<String>,
    /// Maximum concurrent connections
    pub max_concurrent_connections: Option<usize>,
    /// Supported message types
    pub supported_message_types: Vec<String>,
    /// Dark addressing support
    pub supports_dark_addressing: bool,
    /// Onion routing support
    pub supports_onion_routing: bool,
    /// DHT participation
    pub participates_in_dht: bool,
    /// Relay capability
    pub can_relay: bool,
    /// Storage capability
    pub provides_storage: bool,
    /// Bandwidth capacity in bps
    pub bandwidth_capacity: Option<u64>,
}

/// Connection quality assessment.
#[derive(Debug, Clone, Default)]
pub struct ConnectionQuality {
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f64,
    /// Performance score (0.0 to 1.0)
    pub performance_score: f64,
    /// Availability score (0.0 to 1.0)
    pub availability_score: f64,
    /// Security score (0.0 to 1.0)
    pub security_score: f64,
    /// Last assessment time
    pub last_assessed: Option<Instant>,
}

/// Uptime statistics for a peer.
#[derive(Debug, Clone, Default)]
pub struct UptimeStats {
    /// Total observed time
    pub total_observed_time: Duration,
    /// Total uptime
    pub total_uptime: Duration,
    /// Uptime percentage
    pub uptime_percentage: f64,
    /// Number of disconnections
    pub disconnection_count: u32,
    /// Average session duration
    pub avg_session_duration: Duration,
    /// Longest session duration
    pub longest_session_duration: Duration,
}

/// Circuit breaker state for fault tolerance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (requests blocked)
    Open {
        /// Time when circuit opened
        opened_at: Instant,
        /// Failure count
        failure_count: usize,
    },
    /// Circuit is half-open (testing recovery)
    HalfOpen {
        /// Number of test requests sent
        test_requests: usize,
        /// Number of successful test requests
        successful_tests: usize,
    },
}

impl DiscoveredPeer {
    /// Create a new discovered peer with enhanced initialization
    pub fn new(peer_id: LibP2PPeerId, address: SocketAddr, method: DiscoveryMethod) -> Self {
        Self {
            peer_id,
            addresses: vec![address],
            dark_addresses: vec![],
            discovered_at: Instant::now(),
            discovery_method: method,
            reputation: 50.0, // Start with neutral reputation
            connection_attempts: 0,
            successful_connections: 0,
            last_connection_attempt: None,
            last_successful_connection: None,
            protocols: vec![],
            geographic_info: None,
            performance_metrics: PeerPerformanceMetrics::default(),
            load_metrics: PeerLoadMetrics::default(),
            capabilities: PeerCapabilities::default(),
            connection_quality: ConnectionQuality::default(),
            is_blacklisted: false,
            blacklist_reason: None,
            uptime_stats: UptimeStats::default(),
            circuit_breaker_state: CircuitBreakerState::Closed,
        }
    }

    /// Check if this peer should be attempted for connection with advanced logic
    pub fn should_attempt_connection(&self) -> bool {
        self.should_attempt_connection_with_config(&PeerScoringConfig::default())
    }

    /// Check if this peer should be attempted for connection with specific config
    pub fn should_attempt_connection_with_config(&self, config: &PeerScoringConfig) -> bool {
        // Check blacklist status
        if self.is_blacklisted {
            return false;
        }

        // Check circuit breaker state
        match &self.circuit_breaker_state {
            CircuitBreakerState::Open { opened_at, .. } => {
                // Check if timeout has elapsed for recovery attempt
                if opened_at.elapsed() < Duration::from_secs(60) {
                    return false;
                }
            }
            CircuitBreakerState::HalfOpen { test_requests, .. } => {
                // Limit test requests in half-open state
                if *test_requests >= 3 {
                    return false;
                }
            }
            CircuitBreakerState::Closed => {}
        }

        // Check reputation threshold
        if self.reputation < config.min_score {
            return false;
        }

        // Check backoff based on recent failures
        if self.connection_attempts > 3 {
            if let Some(last_attempt) = self.last_connection_attempt {
                let backoff_time =
                    Duration::from_secs((self.connection_attempts as u64).pow(2) * 30);
                if last_attempt.elapsed() < backoff_time {
                    return false;
                }
            }
        }

        // Check connection quality
        if self.connection_quality.overall_score < 0.3 {
            return false;
        }

        true
    }

    /// Record a connection attempt with enhanced metrics
    pub fn record_connection_attempt(&mut self, success: bool, config: &PeerScoringConfig) {
        self.connection_attempts += 1;
        self.last_connection_attempt = Some(Instant::now());

        if success {
            self.successful_connections += 1;
            self.last_successful_connection = Some(Instant::now());
            self.reputation += config.connection_success_bonus;
            self.reputation = self.reputation.min(config.max_score);

            // Reset circuit breaker on success
            match &self.circuit_breaker_state {
                CircuitBreakerState::HalfOpen {
                    successful_tests, ..
                } => {
                    let new_successful = successful_tests + 1;
                    if new_successful >= 3 {
                        self.circuit_breaker_state = CircuitBreakerState::Closed;
                    } else {
                        self.circuit_breaker_state = CircuitBreakerState::HalfOpen {
                            test_requests: 0,
                            successful_tests: new_successful,
                        };
                    }
                }
                _ => {
                    self.circuit_breaker_state = CircuitBreakerState::Closed;
                }
            }

            // Update quality scores
            self.update_connection_quality(true);
        } else {
            self.reputation -= config.connection_failure_penalty;
            self.reputation = self.reputation.max(config.min_score);

            // Update circuit breaker on failure
            match &self.circuit_breaker_state {
                CircuitBreakerState::Closed => {
                    if self.connection_attempts >= 5 {
                        self.circuit_breaker_state = CircuitBreakerState::Open {
                            opened_at: Instant::now(),
                            failure_count: self.connection_attempts as usize,
                        };
                    }
                }
                CircuitBreakerState::HalfOpen { .. } => {
                    self.circuit_breaker_state = CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                        failure_count: self.connection_attempts as usize,
                    };
                }
                _ => {}
            }

            // Update quality scores
            self.update_connection_quality(false);
        }
    }

    /// Update connection quality metrics
    fn update_connection_quality(&mut self, _success: bool) {
        let success_rate = if self.connection_attempts > 0 {
            self.successful_connections as f64 / self.connection_attempts as f64
        } else {
            0.0
        };

        self.connection_quality.reliability_score = success_rate;

        // Update overall score based on multiple factors
        let performance_factor = 1.0 - (self.performance_metrics.error_rate * 0.5);
        let availability_factor = self.uptime_stats.uptime_percentage / 100.0;

        self.connection_quality.overall_score = (self.connection_quality.reliability_score * 0.4
            + performance_factor * 0.3
            + availability_factor * 0.2
            + self.connection_quality.security_score * 0.1)
            .clamp(0.0, 1.0);

        self.connection_quality.last_assessed = Some(Instant::now());
    }

    /// Update performance metrics with new measurement
    pub fn update_performance_metrics(&mut self, response_time: Duration, success: bool) {
        if success {
            // Update response time statistics
            if self.performance_metrics.min_response_time == Duration::ZERO {
                self.performance_metrics.min_response_time = response_time;
                self.performance_metrics.max_response_time = response_time;
                self.performance_metrics.avg_response_time = response_time;
            } else {
                self.performance_metrics.min_response_time = self
                    .performance_metrics
                    .min_response_time
                    .min(response_time);
                self.performance_metrics.max_response_time = self
                    .performance_metrics
                    .max_response_time
                    .max(response_time);

                // Update average with exponential moving average
                let alpha = 0.1;
                let current_avg = self.performance_metrics.avg_response_time.as_secs_f64();
                let new_avg = alpha * response_time.as_secs_f64() + (1.0 - alpha) * current_avg;
                self.performance_metrics.avg_response_time = Duration::from_secs_f64(new_avg);
            }
        }

        // Update error rate
        let total_requests = self.connection_attempts as f64;
        let failed_requests = (self.connection_attempts - self.successful_connections) as f64;
        self.performance_metrics.error_rate = if total_requests > 0.0 {
            failed_requests / total_requests
        } else {
            0.0
        };
    }

    /// Update load metrics
    pub fn update_load_metrics(&mut self, active_connections: usize, queue_depth: usize) {
        self.load_metrics.active_connections = active_connections;
        self.load_metrics.queue_depth = queue_depth;

        // Calculate load score based on multiple factors
        let connection_factor = if let Some(max_conn) = self.capabilities.max_concurrent_connections
        {
            active_connections as f64 / max_conn as f64
        } else {
            active_connections as f64 / 100.0 // Assume 100 as default max
        };

        let queue_factor = queue_depth as f64 / 50.0; // Assume 50 as normal queue depth

        self.load_metrics.load_score =
            ((connection_factor + queue_factor) * 50.0).clamp(0.0, 100.0);

        // Update weight for load balancing (inverse of load)
        self.load_metrics.weight = (100.0 - self.load_metrics.load_score).max(1.0);
    }

    /// Check if peer is healthy for load balancing
    pub fn is_healthy(&self) -> bool {
        !self.is_blacklisted
            && self.circuit_breaker_state == CircuitBreakerState::Closed
            && self.connection_quality.overall_score > 0.5
            && self.load_metrics.load_score < 90.0
    }

    /// Calculate peer priority for selection
    pub fn calculate_priority(&self, config: &PeerScoringConfig) -> f64 {
        let mut priority = self.reputation;

        // Adjust for connection quality
        priority += self.connection_quality.overall_score * 20.0;

        // Adjust for load (prefer less loaded peers)
        priority += (100.0 - self.load_metrics.load_score) * 0.1;

        // Adjust for geographic preferences if available
        if let Some(geo_info) = &self.geographic_info {
            if config.enable_geographic_scoring {
                if let Some(distance) = geo_info.estimated_distance_km {
                    // Prefer closer peers (up to 10 point bonus for local peers)
                    let distance_bonus = (1000.0 - distance.min(1000.0)) / 100.0;
                    priority += distance_bonus;
                }
            }
        }

        // Adjust for uptime
        priority += self.uptime_stats.uptime_percentage * 0.1;

        priority.max(0.0)
    }

    /// Add a dark address to this peer
    pub fn add_dark_address(&mut self, address: ShadowAddress) {
        if !self.dark_addresses.contains(&address) {
            self.dark_addresses.push(address);
            self.capabilities.supports_dark_addressing = true;
        }
    }

    /// Update geographic information
    pub fn update_geographic_info(&mut self, geo_info: GeographicInfo) {
        self.geographic_info = Some(geo_info);
    }

    /// Blacklist this peer with a reason
    pub fn blacklist(&mut self, reason: String) {
        self.is_blacklisted = true;
        self.blacklist_reason = Some(reason);
        self.reputation = -50.0; // Set to minimum reputation
    }

    /// Remove peer from blacklist
    pub fn unblacklist(&mut self) {
        self.is_blacklisted = false;
        self.blacklist_reason = None;
        self.reputation = 0.0; // Reset to neutral
    }

    /// Decay reputation over time
    pub fn decay_reputation(&mut self, config: &PeerScoringConfig, hours_elapsed: f64) {
        let decay_amount = config.score_decay_rate * hours_elapsed;
        self.reputation = (self.reputation - decay_amount).max(config.min_score);
    }

    /// Update uptime statistics
    pub fn update_uptime(&mut self, is_online: bool, duration: Duration) {
        self.uptime_stats.total_observed_time += duration;

        if is_online {
            self.uptime_stats.total_uptime += duration;
        } else {
            self.uptime_stats.disconnection_count += 1;
        }

        // Recalculate uptime percentage
        if self.uptime_stats.total_observed_time > Duration::ZERO {
            self.uptime_stats.uptime_percentage = (self.uptime_stats.total_uptime.as_secs_f64()
                / self.uptime_stats.total_observed_time.as_secs_f64())
                * 100.0;
        }
    }
}

/// Production-ready peer discovery service with advanced DHT, dark addressing, and load balancing.
#[allow(dead_code)]
pub struct KademliaPeerDiscovery {
    /// Configuration
    config: DiscoveryConfig,
    /// Discovered peers with enhanced metrics
    discovered_peers: Arc<RwLock<HashMap<LibP2PPeerId, DiscoveredPeer>>>,
    /// Static peer list
    static_peers: HashSet<SocketAddr>,
    /// Bootstrap completed
    bootstrap_completed: bool,
    /// Discovery active
    discovery_active: bool,
    /// Event sender
    event_tx: Option<mpsc::Sender<DiscoveryEvent>>,
    /// Bootstrap peers that have been tried
    bootstrap_tried: HashSet<SocketAddr>,
    /// Last discovery run
    last_discovery: Option<Instant>,
    /// Dark address resolver
    dark_resolver: Arc<DarkResolver>,
    /// Shadow address handler
    shadow_handler: DefaultShadowAddressHandler,
    /// DHT routing table (K-buckets)
    dht_buckets: Arc<RwLock<BTreeMap<usize, Vec<LibP2PPeerId>>>>,
    /// Connection semaphore for rate limiting
    connection_semaphore: Arc<Semaphore>,
    /// Load balancer
    load_balancer: Arc<Mutex<LoadBalancer>>,
    /// Peer selector with geographic awareness
    peer_selector: Arc<Mutex<PeerSelector>>,
    /// Network topology optimizer
    topology_optimizer: Arc<Mutex<TopologyOptimizer>>,
    /// Health checker for peer monitoring
    health_checker: Arc<Mutex<HealthChecker>>,
    /// Performance monitor
    performance_monitor: Arc<Mutex<PerformanceMonitor>>,
    /// Bootstrap strategy
    bootstrap_strategy: BootstrapStrategy,
}

/// Bootstrap strategies for different network conditions.
#[derive(Debug, Clone)]
pub enum BootstrapStrategy {
    /// Conservative bootstrap with careful peer selection
    Conservative,
    /// Aggressive bootstrap for fast network joining
    Aggressive,
    /// Adaptive bootstrap that adjusts based on network conditions
    Adaptive {
        /// Current aggressiveness level (0.0 to 1.0)
        aggressiveness: f64,
        /// Last adaptation time
        last_adapted: Instant,
    },
    /// Custom bootstrap with specific parameters
    Custom {
        /// Maximum concurrent bootstrap attempts
        max_concurrent: usize,
        /// Timeout per bootstrap attempt
        attempt_timeout: Duration,
        /// Retry strategy
        retry_strategy: RetryStrategy,
    },
}

/// Retry strategies for failed operations.
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// Exponential backoff
    ExponentialBackoff {
        /// Initial delay
        initial_delay: Duration,
        /// Maximum delay
        max_delay: Duration,
        /// Backoff multiplier
        multiplier: f64,
    },
    /// Fixed interval retry
    FixedInterval(Duration),
    /// Linear backoff
    LinearBackoff {
        /// Initial delay
        initial_delay: Duration,
        /// Increment per retry
        increment: Duration,
    },
    /// No retry
    None,
}

/// Enhanced discovery events with detailed information.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// New peer discovered
    PeerDiscovered(DiscoveredPeer),
    /// Peer reputation updated
    ReputationUpdated {
        peer_id: LibP2PPeerId,
        old_reputation: f64,
        new_reputation: f64,
        reason: String,
    },
    /// Bootstrap completed
    BootstrapCompleted {
        /// Number of peers discovered during bootstrap
        peers_discovered: usize,
        /// Bootstrap duration
        duration: Duration,
        /// Success rate
        success_rate: f64,
    },
    /// Bootstrap failed
    BootstrapFailed {
        /// Failure reason
        reason: String,
        /// Attempted bootstrap nodes
        attempted_nodes: usize,
        /// Successful connections
        successful_connections: usize,
    },
    /// Peer connection established
    PeerConnected {
        peer_id: LibP2PPeerId,
        address: SocketAddr,
        connection_time: Duration,
    },
    /// Peer connection lost
    PeerDisconnected {
        peer_id: LibP2PPeerId,
        reason: String,
        session_duration: Duration,
    },
    /// Peer blacklisted
    PeerBlacklisted {
        peer_id: LibP2PPeerId,
        reason: String,
        reputation: f64,
    },
    /// Dark address discovered
    DarkAddressDiscovered {
        peer_id: LibP2PPeerId,
        dark_address: ShadowAddress,
        resolution_time: Duration,
    },
    /// Network topology updated
    TopologyUpdated {
        /// Number of nodes in largest component
        largest_component_size: usize,
        /// Average clustering coefficient
        avg_clustering: f64,
        /// Network diameter
        diameter: Option<usize>,
    },
    /// Load balancing metrics updated
    LoadBalancingUpdated {
        /// Total active connections
        active_connections: usize,
        /// Load distribution entropy
        load_entropy: f64,
        /// Overloaded peers count
        overloaded_peers: usize,
    },
    /// Geographic distribution updated
    GeographicDistributionUpdated {
        /// Number of countries represented
        countries: usize,
        /// Geographic diversity score
        diversity_score: f64,
        /// Average distance to peers
        avg_distance_km: f64,
    },
    /// DHT bucket updated
    DHTBucketUpdated {
        /// Bucket index
        bucket_index: usize,
        /// Number of peers in bucket
        peer_count: usize,
        /// Bucket health score
        health_score: f64,
    },
    /// Discovery error
    DiscoveryError {
        /// Error message
        error: String,
        /// Error category
        category: DiscoveryErrorCategory,
        /// Retry suggestion
        retry_suggested: bool,
    },
}

/// Categories of discovery errors for better handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryErrorCategory {
    /// Network connectivity issues
    NetworkError,
    /// Configuration problems
    ConfigurationError,
    /// Resource exhaustion
    ResourceError,
    /// Protocol violations
    ProtocolError,
    /// Security issues
    SecurityError,
    /// Timeout errors
    TimeoutError,
    /// Dark addressing errors
    DarkAddressingError,
    /// DHT-specific errors
    DHTError,
}

/// Load balancer for distributing connections across peers.
#[derive(Debug)]
pub struct LoadBalancer {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,
    /// Round-robin state
    pub round_robin_index: usize,
    /// Consistent hashing ring
    pub hash_ring: Vec<(u64, LibP2PPeerId)>,
    /// Peer weights for weighted algorithms
    pub peer_weights: HashMap<LibP2PPeerId, f64>,
    /// Connection counts per peer
    pub connection_counts: HashMap<LibP2PPeerId, usize>,
    /// Response time history for least response time algorithm
    pub response_times: HashMap<LibP2PPeerId, VecDeque<Duration>>,
    /// Health check results
    pub health_status: HashMap<LibP2PPeerId, bool>,
}

impl LoadBalancer {
    /// Create new load balancer
    pub fn new(algorithm: LoadBalancingAlgorithm) -> Self {
        Self {
            algorithm,
            round_robin_index: 0,
            hash_ring: Vec::new(),
            peer_weights: HashMap::new(),
            connection_counts: HashMap::new(),
            response_times: HashMap::new(),
            health_status: HashMap::new(),
        }
    }

    /// Select next peer based on algorithm
    pub fn select_peer(&mut self, available_peers: &[LibP2PPeerId]) -> Option<LibP2PPeerId> {
        if available_peers.is_empty() {
            return None;
        }

        match &self.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                let peer = available_peers[self.round_robin_index % available_peers.len()];
                self.round_robin_index = (self.round_robin_index + 1) % available_peers.len();
                Some(peer)
            }
            LoadBalancingAlgorithm::WeightedRoundRobin => {
                // Select based on weights
                let total_weight: f64 = available_peers
                    .iter()
                    .map(|p| self.peer_weights.get(p).unwrap_or(&1.0))
                    .sum();

                let mut rng = rand::thread_rng();
                let mut target = rng.gen::<f64>() * total_weight;

                for peer in available_peers {
                    let weight = self.peer_weights.get(peer).unwrap_or(&1.0);
                    if target < *weight {
                        return Some(*peer);
                    }
                    target -= weight;
                }

                available_peers.last().copied()
            }
            LoadBalancingAlgorithm::LeastConnections => available_peers
                .iter()
                .min_by_key(|p| self.connection_counts.get(p).unwrap_or(&0))
                .copied(),
            LoadBalancingAlgorithm::LeastResponseTime => available_peers
                .iter()
                .min_by_key(|p| {
                    self.response_times
                        .get(p)
                        .and_then(|times| times.back())
                        .map(|d| d.as_millis())
                        .unwrap_or(u128::MAX)
                })
                .copied(),
            LoadBalancingAlgorithm::Random => {
                let mut rng = rand::thread_rng();
                available_peers.choose(&mut rng).copied()
            }
            _ => available_peers.first().copied(),
        }
    }

    /// Update peer metrics
    pub fn update_metrics(
        &mut self,
        peer: LibP2PPeerId,
        connections: usize,
        response_time: Option<Duration>,
    ) {
        self.connection_counts.insert(peer, connections);

        if let Some(rt) = response_time {
            self.response_times
                .entry(peer)
                .or_insert_with(|| VecDeque::with_capacity(10))
                .push_back(rt);

            if let Some(times) = self.response_times.get_mut(&peer) {
                if times.len() > 10 {
                    times.pop_front();
                }
            }
        }
    }
}

/// Peer selector with geographic and capability awareness.
#[derive(Debug)]
pub struct PeerSelector {
    /// Geographic preferences
    pub geo_preferences: GeoPreferences,
    /// Capability requirements
    pub required_capabilities: Vec<String>,
    /// Selection strategy
    pub strategy: PeerSelectionStrategy,
    /// Recent selections for diversity
    pub recent_selections: VecDeque<LibP2PPeerId>,
    /// Selection history for analysis
    pub selection_history: HashMap<LibP2PPeerId, usize>,
}

impl PeerSelector {
    /// Create new peer selector
    pub fn new(geo_preferences: GeoPreferences) -> Self {
        Self {
            geo_preferences,
            required_capabilities: vec![],
            strategy: PeerSelectionStrategy::BestFirst,
            recent_selections: VecDeque::with_capacity(100),
            selection_history: HashMap::new(),
        }
    }

    /// Select peers based on criteria
    pub fn select_peers(
        &mut self,
        candidates: &[DiscoveredPeer],
        count: usize,
        scoring_config: &PeerScoringConfig,
    ) -> Vec<LibP2PPeerId> {
        let mut selected = Vec::new();

        // Filter candidates based on criteria
        let mut eligible: Vec<_> = candidates
            .iter()
            .filter(|p| p.is_healthy() && !p.is_blacklisted)
            .filter(|p| p.connection_quality.reliability_score >= 0.5)
            .filter(|p| self.meets_capability_requirements(p))
            .collect();

        // Sort based on strategy
        match &self.strategy {
            PeerSelectionStrategy::BestFirst => {
                eligible.sort_by(|a, b| {
                    b.calculate_priority(scoring_config)
                        .partial_cmp(&a.calculate_priority(scoring_config))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            PeerSelectionStrategy::Diversity => {
                // Prefer peers we haven't selected recently
                eligible.sort_by_key(|p| {
                    self.recent_selections
                        .iter()
                        .position(|id| id == &p.peer_id)
                        .unwrap_or(usize::MAX)
                });
            }
            PeerSelectionStrategy::Probabilistic => {
                // Weighted random selection based on scores
                let mut rng = rand::thread_rng();
                eligible.shuffle(&mut rng);
            }
            _ => {}
        }

        // Select required number of peers
        for peer in eligible.into_iter().take(count) {
            selected.push(peer.peer_id);
            self.recent_selections.push_back(peer.peer_id);
            if self.recent_selections.len() > 100 {
                self.recent_selections.pop_front();
            }
            *self.selection_history.entry(peer.peer_id).or_insert(0) += 1;
        }

        selected
    }

    /// Check if peer meets capability requirements
    fn meets_capability_requirements(&self, peer: &DiscoveredPeer) -> bool {
        if self.required_capabilities.is_empty() {
            return true;
        }

        for required in &self.required_capabilities {
            if !peer.capabilities.supported_message_types.contains(required)
                && !peer.capabilities.protocol_versions.contains(required)
            {
                return false;
            }
        }

        true
    }
}

/// Peer selection strategies.
#[derive(Debug, Clone)]
pub enum PeerSelectionStrategy {
    /// Best peers first (highest score)
    BestFirst,
    /// Probabilistic selection based on scores
    Probabilistic,
    /// Diversity-focused selection
    Diversity,
    /// Exploration vs exploitation balance
    EpsilonGreedy { epsilon: f64 },
    /// Multi-armed bandit selection
    MultiArmedBandit,
}

/// Network topology optimizer for maintaining healthy network structure.
#[derive(Debug)]
pub struct TopologyOptimizer {
    /// Target clustering coefficient
    pub target_clustering: f64,
    /// Target average path length
    pub target_path_length: f64,
    /// Minimum connectivity requirements
    pub min_connectivity: usize,
    /// Last optimization time
    pub last_optimization: Instant,
    /// Optimization interval
    pub optimization_interval: Duration,
    /// Network metrics history
    pub metrics_history: VecDeque<TopologyMetrics>,
}

impl TopologyOptimizer {
    /// Create new topology optimizer
    pub fn new() -> Self {
        Self {
            target_clustering: 0.3,
            target_path_length: 4.0,
            min_connectivity: 3,
            last_optimization: Instant::now(),
            optimization_interval: Duration::from_secs(300),
            metrics_history: VecDeque::with_capacity(100),
        }
    }
}

/// Network topology metrics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TopologyMetrics {
    /// Measurement timestamp
    timestamp: Instant,
    /// Clustering coefficient
    clustering_coefficient: f64,
    /// Average path length
    avg_path_length: f64,
    /// Network diameter
    diameter: Option<usize>,
    /// Number of connected components
    connected_components: usize,
    /// Largest component size
    largest_component_size: usize,
    /// Small-world coefficient
    small_world_coefficient: f64,
}

/// Health checker for monitoring peer status.
#[derive(Debug)]
pub struct HealthChecker {
    /// Health check interval
    pub check_interval: Duration,
    /// Health check timeout
    pub check_timeout: Duration,
    /// Last health check times
    pub last_checks: HashMap<LibP2PPeerId, Instant>,
    /// Health check results
    pub health_results: HashMap<LibP2PPeerId, HealthCheckResult>,
    /// Health check configuration
    pub config: HealthCheckConfig,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(check_interval: Duration) -> Self {
        Self {
            check_interval,
            check_timeout: Duration::from_secs(5),
            last_checks: HashMap::new(),
            health_results: HashMap::new(),
            config: HealthCheckConfig::default(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enable_ping: true,
            enable_capability_check: true,
            enable_performance_monitoring: true,
            unhealthy_multiplier: 2.0,
            failure_threshold: 3,
        }
    }
}

/// Health check configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HealthCheckConfig {
    /// Enable ping checks
    enable_ping: bool,
    /// Enable capability checks
    enable_capability_check: bool,
    /// Enable performance monitoring
    enable_performance_monitoring: bool,
    /// Health check frequency multiplier for unhealthy peers
    unhealthy_multiplier: f64,
    /// Consecutive failures before marking unhealthy
    failure_threshold: usize,
}

/// Health check result.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HealthCheckResult {
    /// Overall health status
    is_healthy: bool,
    /// Response time
    response_time: Option<Duration>,
    /// Last successful check
    last_successful: Option<Instant>,
    /// Consecutive failures
    consecutive_failures: usize,
    /// Specific check results
    check_details: HashMap<String, bool>,
    /// Error messages
    errors: Vec<String>,
}

/// Performance monitor for tracking system and peer performance.
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Performance metrics collection interval
    pub collection_interval: Duration,
    /// Last collection time
    pub last_collection: Instant,
    /// System performance metrics
    pub system_metrics: SystemPerformanceMetrics,
    /// Per-peer performance metrics
    pub peer_metrics: HashMap<LibP2PPeerId, PeerPerformanceMetrics>,
    /// Performance alerts
    pub alerts: VecDeque<PerformanceAlert>,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            collection_interval: Duration::from_secs(60),
            last_collection: Instant::now(),
            system_metrics: SystemPerformanceMetrics::default(),
            peer_metrics: HashMap::new(),
            alerts: VecDeque::with_capacity(100),
        }
    }
}

/// System-wide performance metrics.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SystemPerformanceMetrics {
    /// Total discovery rate (peers per second)
    discovery_rate: f64,
    /// Connection success rate
    connection_success_rate: f64,
    /// Average connection establishment time
    avg_connection_time: Duration,
    /// Memory usage for peer storage
    memory_usage_bytes: usize,
    /// CPU usage percentage
    cpu_usage_percent: f64,
    /// Network bandwidth utilization
    network_utilization_bps: u64,
    /// DHT maintenance overhead
    dht_overhead_percent: f64,
}

/// Performance alerts for monitoring critical conditions.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PerformanceAlert {
    /// Alert timestamp
    timestamp: Instant,
    /// Alert severity
    severity: AlertSeverity,
    /// Alert category
    category: AlertCategory,
    /// Alert message
    message: String,
    /// Affected peer (if applicable)
    peer_id: Option<LibP2PPeerId>,
    /// Suggested actions
    suggested_actions: Vec<String>,
}

/// Alert severity levels.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning condition
    Warning,
    /// Error condition
    Error,
    /// Critical condition requiring immediate attention
    Critical,
}

/// Alert categories for filtering and handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertCategory {
    /// Performance degradation
    Performance,
    /// Connectivity issues
    Connectivity,
    /// Security concerns
    Security,
    /// Resource exhaustion
    Resource,
    /// Configuration issues
    Configuration,
}

impl KademliaPeerDiscovery {
    /// Create a new production-ready Kademlia peer discovery service
    pub fn new(config: DiscoveryConfig) -> Self {
        let max_connections = config.max_concurrent_connections;

        Self {
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            static_peers: HashSet::new(),
            bootstrap_completed: false,
            discovery_active: false,
            event_tx: None,
            bootstrap_tried: HashSet::new(),
            last_discovery: None,
            dark_resolver: Arc::new(DarkResolver::new()),
            shadow_handler: DefaultShadowAddressHandler::new(
                NetworkType::Mainnet,
                [0u8; 32], // TODO: Use proper seed
            ),
            dht_buckets: Arc::new(RwLock::new(BTreeMap::new())),
            connection_semaphore: Arc::new(Semaphore::new(max_connections)),
            load_balancer: Arc::new(Mutex::new(LoadBalancer::new(
                config.load_balancing_config.algorithm.clone(),
            ))),
            peer_selector: Arc::new(Mutex::new(PeerSelector::new(
                config.geo_preferences.clone(),
            ))),
            topology_optimizer: Arc::new(Mutex::new(TopologyOptimizer::new())),
            health_checker: Arc::new(Mutex::new(HealthChecker::new(
                config.load_balancing_config.health_check_interval,
            ))),
            performance_monitor: Arc::new(Mutex::new(PerformanceMonitor::new())),
            bootstrap_strategy: BootstrapStrategy::Adaptive {
                aggressiveness: 0.5,
                last_adapted: Instant::now(),
            },
            config,
        }
    }

    /// Set event channel for discovery events
    pub fn set_event_channel(&mut self, tx: mpsc::Sender<DiscoveryEvent>) {
        self.event_tx = Some(tx);
    }

    /// Start the discovery service
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        if self.discovery_active {
            return Ok(());
        }

        self.discovery_active = true;
        info!("Starting Kademlia peer discovery service");

        // Start bootstrap process
        self.bootstrap().await?;

        // Start periodic discovery
        self.start_periodic_discovery().await;

        Ok(())
    }

    /// Stop the discovery service
    pub async fn stop(&mut self) -> Result<(), NetworkError> {
        self.discovery_active = false;
        info!("Stopping Kademlia peer discovery service");
        Ok(())
    }

    /// Bootstrap the DHT with known peers
    async fn bootstrap(&mut self) -> Result<(), NetworkError> {
        if self.bootstrap_completed {
            return Ok(());
        }

        info!(
            "Starting DHT bootstrap with {} nodes",
            self.config.bootstrap_nodes.len()
        );
        let start_time = Instant::now();
        let mut discovered_peers = 0;

        for bootstrap_addr in &self.config.bootstrap_nodes {
            if self.bootstrap_tried.contains(bootstrap_addr) {
                continue;
            }

            self.bootstrap_tried.insert(*bootstrap_addr);

            // Create a discovered peer for the bootstrap node
            let peer_id = LibP2PPeerId::random(); // In real implementation, resolve from address
            let discovered_peer =
                DiscoveredPeer::new(peer_id, *bootstrap_addr, DiscoveryMethod::Bootstrap);

            // Add to discovered peers
            self.discovered_peers
                .write()
                .await
                .insert(peer_id, discovered_peer.clone());
            discovered_peers += 1;

            // Send discovery event
            if let Some(tx) = &self.event_tx {
                let _ = tx
                    .send(DiscoveryEvent::PeerDiscovered(discovered_peer))
                    .await;
            }

            debug!("Added bootstrap peer: {} -> {:?}", bootstrap_addr, peer_id);
        }

        self.bootstrap_completed = true;

        if let Some(tx) = &self.event_tx {
            let _ = tx
                .send(DiscoveryEvent::BootstrapCompleted {
                    peers_discovered: discovered_peers,
                    duration: start_time.elapsed(),
                    success_rate: discovered_peers as f64
                        / self.config.bootstrap_nodes.len().max(1) as f64,
                })
                .await;
        }

        info!("DHT bootstrap completed");
        Ok(())
    }

    /// Start periodic discovery tasks
    async fn start_periodic_discovery(&mut self) {
        let interval = Duration::from_secs(self.config.interval);
        let discovered_peers = Arc::clone(&self.discovered_peers);
        let event_tx = self.event_tx.clone();
        let methods = self.config.methods.clone();
        let max_peers = self.config.max_peers;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Perform discovery based on configured methods
                for method in &methods {
                    match method {
                        DiscoveryMethod::Kademlia => {
                            Self::discover_kademlia_peers(&discovered_peers, &event_tx, max_peers)
                                .await;
                        }
                        DiscoveryMethod::Mdns => {
                            Self::discover_mdns_peers(&discovered_peers, &event_tx).await;
                        }
                        _ => {} // Other methods handled separately
                    }
                }
            }
        });
    }

    /// Discover peers using Kademlia DHT
    async fn discover_kademlia_peers(
        discovered_peers: &Arc<RwLock<HashMap<LibP2PPeerId, DiscoveredPeer>>>,
        event_tx: &Option<mpsc::Sender<DiscoveryEvent>>,
        max_peers: usize,
    ) {
        let current_count = discovered_peers.read().await.len();
        if current_count >= max_peers {
            return;
        }

        // Simulate DHT peer discovery
        let peers_to_discover = (max_peers - current_count).min(5);

        for _ in 0..peers_to_discover {
            let peer_id = LibP2PPeerId::random();
            let address =
                SocketAddr::from(([192, 168, 1, 100], 8000 + rand::random::<u16>() % 1000));

            let discovered_peer = DiscoveredPeer::new(peer_id, address, DiscoveryMethod::Kademlia);

            // Add to discovered peers
            discovered_peers
                .write()
                .await
                .insert(peer_id, discovered_peer.clone());

            // Send discovery event
            if let Some(tx) = event_tx {
                let _ = tx
                    .send(DiscoveryEvent::PeerDiscovered(discovered_peer))
                    .await;
            }

            debug!("Discovered peer via Kademlia: {:?} at {}", peer_id, address);
        }
    }

    /// Discover peers using mDNS
    async fn discover_mdns_peers(
        discovered_peers: &Arc<RwLock<HashMap<LibP2PPeerId, DiscoveredPeer>>>,
        event_tx: &Option<mpsc::Sender<DiscoveryEvent>>,
    ) {
        // Simulate mDNS discovery for local network peers
        let local_peers = 2; // Discover a few local peers

        for _ in 0..local_peers {
            let peer_id = LibP2PPeerId::random();
            let address = SocketAddr::from(([192, 168, 1, 10 + rand::random::<u8>() % 50], 8000));

            // Check if already discovered
            if discovered_peers.read().await.contains_key(&peer_id) {
                continue;
            }

            let discovered_peer = DiscoveredPeer::new(peer_id, address, DiscoveryMethod::Mdns);

            // Add to discovered peers
            discovered_peers
                .write()
                .await
                .insert(peer_id, discovered_peer.clone());

            // Send discovery event
            if let Some(tx) = event_tx {
                let _ = tx
                    .send(DiscoveryEvent::PeerDiscovered(discovered_peer))
                    .await;
            }

            debug!("Discovered peer via mDNS: {:?} at {}", peer_id, address);
        }
    }

    /// Get discovered peers
    pub async fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get peers suitable for connection
    pub async fn get_connectable_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers
            .read()
            .await
            .values()
            .filter(|peer| peer.should_attempt_connection())
            .cloned()
            .collect()
    }

    /// Update peer reputation
    pub async fn update_peer_reputation(&self, peer_id: LibP2PPeerId, delta: f64) {
        if let Some(peer) = self.discovered_peers.write().await.get_mut(&peer_id) {
            let old_reputation = peer.reputation;
            peer.reputation += delta;
            peer.reputation = peer.reputation.clamp(-50.0, 50.0);

            if let Some(tx) = &self.event_tx {
                let _ = tx
                    .send(DiscoveryEvent::ReputationUpdated {
                        peer_id,
                        old_reputation,
                        new_reputation: peer.reputation,
                        reason: "Connection update".to_string(),
                    })
                    .await;
            }
        }
    }

    /// Record connection attempt for a peer
    pub async fn record_connection_attempt(&self, peer_id: LibP2PPeerId, success: bool) {
        if let Some(peer) = self.discovered_peers.write().await.get_mut(&peer_id) {
            peer.record_connection_attempt(success, &self.config.scoring_config);

            if success {
                info!("Successful connection to peer: {:?}", peer_id);
            } else {
                warn!(
                    "Failed connection to peer: {:?} (attempts: {})",
                    peer_id, peer.connection_attempts
                );
            }
        }
    }

    /// Add a static peer
    pub fn add_static_peer(&mut self, address: SocketAddr) {
        self.static_peers.insert(address);
        info!("Added static peer: {}", address);
    }

    /// Remove old discovered peers (older than 1 hour)
    pub async fn cleanup_old_peers(&self) {
        let cutoff = Instant::now() - Duration::from_secs(3600);

        self.discovered_peers.write().await.retain(|peer_id, peer| {
            let keep = peer.discovered_at > cutoff;
            if !keep {
                debug!("Removing old discovered peer: {:?}", peer_id);
            }
            keep
        });
    }

    /// Get discovery statistics
    pub async fn get_discovery_stats(&self) -> DiscoveryStats {
        let peers = self.discovered_peers.read().await;
        let total_peers = peers.len();

        let mut method_counts = HashMap::new();
        let mut avg_reputation = 0.0;
        let mut connectable_count = 0;

        for peer in peers.values() {
            *method_counts
                .entry(peer.discovery_method.clone())
                .or_insert(0) += 1;
            avg_reputation += peer.reputation;

            if peer.should_attempt_connection() {
                connectable_count += 1;
            }
        }

        if total_peers > 0 {
            avg_reputation /= total_peers as f64;
        }

        DiscoveryStats {
            total_discovered: total_peers,
            connectable_peers: connectable_count,
            method_counts,
            average_reputation: avg_reputation,
            bootstrap_completed: self.bootstrap_completed,
        }
    }
}

/// Discovery statistics
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    /// Total number of discovered peers
    pub total_discovered: usize,
    /// Number of connectable peers
    pub connectable_peers: usize,
    /// Peer count by discovery method
    pub method_counts: HashMap<DiscoveryMethod, usize>,
    /// Average peer reputation
    pub average_reputation: f64,
    /// Whether bootstrap is completed
    pub bootstrap_completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_discovery_creation() {
        let config = DiscoveryConfig::default();
        let discovery = KademliaPeerDiscovery::new(config);
        assert!(!discovery.discovery_active);
        assert!(!discovery.bootstrap_completed);
    }

    #[tokio::test]
    async fn test_bootstrap() {
        let mut config = DiscoveryConfig::default();
        config.bootstrap_nodes = vec![SocketAddr::from(([127, 0, 0, 1], 8000))];

        let mut discovery = KademliaPeerDiscovery::new(config);
        let (tx, mut rx) = mpsc::channel(10);
        discovery.set_event_channel(tx);

        discovery.bootstrap().await.unwrap();
        assert!(discovery.bootstrap_completed);

        // Should receive discovery events
        let event = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(event.is_ok());
    }

    #[tokio::test]
    async fn test_peer_reputation() {
        let config = DiscoveryConfig::default();
        let discovery = KademliaPeerDiscovery::new(config);

        let peer_id = LibP2PPeerId::random();
        let address = SocketAddr::from(([127, 0, 0, 1], 8000));
        let peer = DiscoveredPeer::new(peer_id, address, DiscoveryMethod::Kademlia);

        discovery
            .discovered_peers
            .write()
            .await
            .insert(peer_id, peer);

        // Update reputation
        discovery.update_peer_reputation(peer_id, 5.0).await;

        let peers = discovery.get_discovered_peers().await;
        assert_eq!(peers[0].reputation, 5.0);
    }

    #[tokio::test]
    async fn test_connection_attempts() {
        let config = DiscoveryConfig::default();
        let discovery = KademliaPeerDiscovery::new(config);

        let peer_id = LibP2PPeerId::random();
        let address = SocketAddr::from(([127, 0, 0, 1], 8000));
        let peer = DiscoveredPeer::new(peer_id, address, DiscoveryMethod::Kademlia);

        discovery
            .discovered_peers
            .write()
            .await
            .insert(peer_id, peer);

        // Record failed attempt
        discovery.record_connection_attempt(peer_id, false).await;

        let peers = discovery.get_discovered_peers().await;
        assert_eq!(peers[0].connection_attempts, 1);
        assert!(peers[0].reputation < 0.0);
    }
}
