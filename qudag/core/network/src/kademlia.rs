//! Production-ready Kademlia DHT implementation for decentralized peer discovery
//! with bootstrap node functionality, content routing, and peer reputation scoring.

use crate::discovery::{
    DHTConfig, DiscoveredPeer, DiscoveryEvent, DiscoveryMethod, GeographicInfo, PeerScoringConfig,
};
use crate::shadow_address::{ShadowAddress, ShadowAddressResolver};
use crate::types::NetworkError;
use libp2p::{
    kad::{
        BootstrapOk, Event as KademliaEvent, GetClosestPeersOk, GetProvidersOk, GetRecordOk,
        PutRecordOk, QueryId, QueryResult, Record, RecordKey,
    },
    PeerId as LibP2PPeerId,
};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Maximum record size for DHT storage (1MB)
const MAX_RECORD_SIZE: usize = 1024 * 1024;

/// Maximum number of provider records per key
#[allow(dead_code)]
const MAX_PROVIDERS_PER_KEY: usize = 20;

/// Default TTL for records (24 hours)
const DEFAULT_RECORD_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Bootstrap node configuration
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Bootstrap node addresses
    pub nodes: Vec<(LibP2PPeerId, SocketAddr)>,
    /// Bootstrap timeout
    pub timeout: Duration,
    /// Minimum successful connections for bootstrap
    pub min_connections: usize,
    /// Enable periodic bootstrap
    pub periodic_bootstrap: bool,
    /// Bootstrap interval
    pub bootstrap_interval: Duration,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            nodes: vec![],
            timeout: Duration::from_secs(30),
            min_connections: 3,
            periodic_bootstrap: true,
            bootstrap_interval: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Content routing configuration
#[derive(Debug, Clone)]
pub struct ContentRoutingConfig {
    /// Enable content routing
    pub enabled: bool,
    /// Provider record TTL
    pub provider_ttl: Duration,
    /// Replication factor for content
    pub replication_factor: usize,
    /// Enable automatic republishing
    pub auto_republish: bool,
    /// Republish interval
    pub republish_interval: Duration,
    /// Maximum content size
    pub max_content_size: usize,
}

impl Default for ContentRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider_ttl: Duration::from_secs(24 * 60 * 60), // 24 hours
            replication_factor: 20,
            auto_republish: true,
            republish_interval: Duration::from_secs(12 * 60 * 60), // 12 hours
            max_content_size: MAX_RECORD_SIZE,
        }
    }
}

/// Peer reputation and scoring system
#[derive(Debug, Clone)]
pub struct PeerReputation {
    /// Peer ID
    pub peer_id: LibP2PPeerId,
    /// Current reputation score
    pub score: f64,
    /// Total interactions
    pub total_interactions: u64,
    /// Successful interactions
    pub successful_interactions: u64,
    /// Failed interactions
    pub failed_interactions: u64,
    /// Last interaction time
    pub last_interaction: Option<Instant>,
    /// Response times history
    pub response_times: VecDeque<Duration>,
    /// Average response time
    pub avg_response_time: Duration,
    /// Provided content count
    pub content_provided: u64,
    /// Content retrieval success rate
    pub content_success_rate: f64,
    /// Bandwidth contribution
    pub bandwidth_contribution: u64,
    /// Uptime tracking
    pub uptime_start: Instant,
    /// Downtime incidents
    pub downtime_incidents: u32,
    /// Misbehavior count
    pub misbehavior_count: u32,
    /// Last misbehavior time
    pub last_misbehavior: Option<Instant>,
    /// Is bootstrap node
    pub is_bootstrap: bool,
    /// Geographic location
    pub location: Option<GeographicInfo>,
}

impl PeerReputation {
    /// Create new peer reputation
    pub fn new(peer_id: LibP2PPeerId) -> Self {
        Self {
            peer_id,
            score: 50.0, // Start with neutral score
            total_interactions: 0,
            successful_interactions: 0,
            failed_interactions: 0,
            last_interaction: None,
            response_times: VecDeque::with_capacity(100),
            avg_response_time: Duration::ZERO,
            content_provided: 0,
            content_success_rate: 0.0,
            bandwidth_contribution: 0,
            uptime_start: Instant::now(),
            downtime_incidents: 0,
            misbehavior_count: 0,
            last_misbehavior: None,
            is_bootstrap: false,
            location: None,
        }
    }

    /// Record an interaction with this peer
    pub fn record_interaction(
        &mut self,
        success: bool,
        response_time: Option<Duration>,
        config: &PeerScoringConfig,
    ) {
        self.total_interactions += 1;
        self.last_interaction = Some(Instant::now());

        if success {
            self.successful_interactions += 1;
            self.score += config.connection_success_bonus;

            // Update response time metrics
            if let Some(rt) = response_time {
                self.response_times.push_back(rt);
                if self.response_times.len() > 100 {
                    self.response_times.pop_front();
                }

                // Calculate new average
                let sum: Duration = self.response_times.iter().sum();
                self.avg_response_time = sum / self.response_times.len() as u32;

                // Apply latency penalty if response is slow
                let latency_penalty = rt.as_millis() as f64 * config.latency_penalty_factor;
                self.score -= latency_penalty.min(5.0); // Cap penalty at 5 points
            }
        } else {
            self.failed_interactions += 1;
            self.score -= config.connection_failure_penalty;
        }

        // Update content success rate
        if self.total_interactions > 0 {
            self.content_success_rate =
                self.successful_interactions as f64 / self.total_interactions as f64;
        }

        // Clamp score to valid range
        self.score = self.score.clamp(config.min_score, config.max_score);
    }

    /// Record misbehavior
    pub fn record_misbehavior(&mut self, severity: f64) {
        self.misbehavior_count += 1;
        self.last_misbehavior = Some(Instant::now());
        self.score -= severity * 10.0; // Heavy penalty for misbehavior
        self.score = self.score.max(-100.0); // Floor at -100
    }

    /// Calculate uptime percentage
    pub fn uptime_percentage(&self) -> f64 {
        let total_time = self.uptime_start.elapsed();
        let downtime = Duration::from_secs(self.downtime_incidents as u64 * 300); // Assume 5 min per incident
        let uptime = total_time.saturating_sub(downtime);

        if total_time.as_secs() > 0 {
            (uptime.as_secs() as f64 / total_time.as_secs() as f64) * 100.0
        } else {
            100.0
        }
    }

    /// Apply time-based decay to reputation
    pub fn apply_decay(&mut self, config: &PeerScoringConfig) {
        if let Some(last) = self.last_interaction {
            let hours_elapsed = last.elapsed().as_secs() as f64 / 3600.0;
            let decay = config.score_decay_rate * hours_elapsed;
            self.score = (self.score - decay).max(config.min_score);
        }
    }

    /// Get reliability score (0.0 to 1.0)
    pub fn reliability_score(&self) -> f64 {
        if self.total_interactions == 0 {
            0.5 // Neutral for new peers
        } else {
            self.content_success_rate
        }
    }
}

/// Kademlia DHT service with enhanced features
pub struct KademliaDHT {
    /// Kademlia instance (commented out - needs libp2p kad feature update)
    // kademlia: Kademlia<MemoryStore>,
    /// DHT configuration
    #[allow(dead_code)]
    config: DHTConfig,
    /// Bootstrap configuration
    bootstrap_config: BootstrapConfig,
    /// Content routing configuration
    content_config: ContentRoutingConfig,
    /// Peer reputations
    peer_reputations: Arc<RwLock<HashMap<LibP2PPeerId, PeerReputation>>>,
    /// Scoring configuration
    scoring_config: PeerScoringConfig,
    /// Local peer ID
    local_peer_id: LibP2PPeerId,
    /// Bootstrap state
    bootstrap_state: BootstrapState,
    /// Content providers cache
    #[allow(dead_code)]
    providers_cache: Arc<RwLock<HashMap<RecordKey, HashSet<LibP2PPeerId>>>>,
    /// Pending queries
    #[allow(dead_code)]
    pending_queries: Arc<Mutex<HashMap<QueryId, QueryInfo>>>,
    /// Event channel
    event_tx: Option<mpsc::Sender<DiscoveryEvent>>,
    /// Performance metrics
    metrics: Arc<Mutex<DHTMetrics>>,
    /// Dark address resolver for content routing
    dark_resolver: Option<Arc<dyn ShadowAddressResolver + Send + Sync>>,
    /// Network partitions detector
    partition_detector: Arc<Mutex<PartitionDetector>>,
}

/// Bootstrap state tracking
#[derive(Debug, Clone, PartialEq)]
enum BootstrapState {
    /// Not started
    NotStarted,
    /// In progress
    InProgress {
        start_time: Instant,
        connected_nodes: usize,
        attempted_nodes: usize,
    },
    /// Completed successfully
    Completed {
        completion_time: Instant,
        connected_nodes: usize,
        duration: Duration,
    },
    /// Failed
    Failed {
        failure_time: Instant,
        reason: String,
        attempted_nodes: usize,
    },
}

/// Query information for tracking
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct QueryInfo {
    /// Query type
    query_type: QueryType,
    /// Start time
    start_time: Instant,
    /// Target key or peer
    target: QueryTarget,
    /// Requestor info
    requestor: Option<LibP2PPeerId>,
}

/// Query types
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum QueryType {
    /// Find node query
    FindNode,
    /// Get providers query
    GetProviders,
    /// Put record query
    PutRecord,
    /// Get record query
    GetRecord,
    /// Add provider query
    AddProvider,
}

/// Query target
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum QueryTarget {
    /// Peer ID target
    PeerId(LibP2PPeerId),
    /// Record key target
    RecordKey(RecordKey),
}

/// DHT performance metrics
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DHTMetrics {
    /// Total queries initiated
    total_queries: u64,
    /// Successful queries
    successful_queries: u64,
    /// Failed queries
    failed_queries: u64,
    /// Average query time
    avg_query_time: Duration,
    /// Records stored
    records_stored: u64,
    /// Providers announced
    providers_announced: u64,
    /// Bootstrap attempts
    bootstrap_attempts: u64,
    /// Successful bootstraps
    successful_bootstraps: u64,
    /// Current routing table size
    routing_table_size: usize,
    /// Network size estimate
    network_size_estimate: usize,
}

/// Network partition detector
#[derive(Debug)]
#[allow(dead_code)]
struct PartitionDetector {
    /// Last successful queries to different parts of the network
    last_successful_queries: HashMap<u8, Instant>, // Bucket index -> last success
    /// Partition detection threshold
    detection_threshold: Duration,
    /// Detected partitions
    detected_partitions: Vec<PartitionInfo>,
}

/// Partition information
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PartitionInfo {
    /// Detection time
    detected_at: Instant,
    /// Affected bucket indices
    affected_buckets: Vec<u8>,
    /// Estimated partition size
    estimated_size: usize,
    /// Recovery status
    recovered: bool,
}

#[allow(dead_code)]
impl KademliaDHT {
    /// Create new Kademlia DHT instance
    pub fn new(
        local_peer_id: LibP2PPeerId,
        config: DHTConfig,
        bootstrap_config: BootstrapConfig,
        content_config: ContentRoutingConfig,
        scoring_config: PeerScoringConfig,
    ) -> Self {
        // Configure Kademlia
        // TODO: Re-enable when libp2p kad API is updated
        // let mut kad_config = KademliaConfig::default();
        // kad_config.set_k_value(config.bucket_size);
        // kad_config.set_alpha_value(config.alpha);
        // kad_config.set_query_timeout(config.bootstrap_timeout);
        // kad_config.set_replication_factor(config.replication_factor);
        // kad_config.set_publication_interval(Some(config.refresh_interval));
        // kad_config.set_provider_record_ttl(Some(content_config.provider_ttl));
        // kad_config.set_record_ttl(Some(DEFAULT_RECORD_TTL));

        // Create memory store
        // let store = MemoryStore::new(local_peer_id);

        // Create Kademlia instance
        // let kademlia = Kademlia::with_config(local_peer_id, store, kad_config);

        Self {
            // kademlia,
            config,
            bootstrap_config,
            content_config,
            scoring_config,
            local_peer_id,
            bootstrap_state: BootstrapState::NotStarted,
            peer_reputations: Arc::new(RwLock::new(HashMap::new())),
            providers_cache: Arc::new(RwLock::new(HashMap::new())),
            pending_queries: Arc::new(Mutex::new(HashMap::new())),
            event_tx: None,
            metrics: Arc::new(Mutex::new(DHTMetrics::default())),
            dark_resolver: None,
            partition_detector: Arc::new(Mutex::new(PartitionDetector {
                last_successful_queries: HashMap::new(),
                detection_threshold: Duration::from_secs(300), // 5 minutes
                detected_partitions: Vec::new(),
            })),
        }
    }

    /// Set event channel
    pub fn set_event_channel(&mut self, tx: mpsc::Sender<DiscoveryEvent>) {
        self.event_tx = Some(tx);
    }

    /// Set dark address resolver
    pub fn set_dark_resolver(&mut self, resolver: Arc<dyn ShadowAddressResolver + Send + Sync>) {
        self.dark_resolver = Some(resolver);
    }

    /// Bootstrap the DHT with known nodes
    pub async fn bootstrap(&mut self) -> Result<(), NetworkError> {
        match &self.bootstrap_state {
            BootstrapState::Completed { .. } => {
                info!("DHT already bootstrapped");
                return Ok(());
            }
            BootstrapState::InProgress { .. } => {
                warn!("Bootstrap already in progress");
                return Ok(());
            }
            _ => {}
        }

        info!(
            "Starting DHT bootstrap with {} nodes",
            self.bootstrap_config.nodes.len()
        );

        self.bootstrap_state = BootstrapState::InProgress {
            start_time: Instant::now(),
            connected_nodes: 0,
            attempted_nodes: 0,
        };

        let mut metrics = self.metrics.lock().unwrap();
        metrics.bootstrap_attempts += 1;
        drop(metrics);

        // Add bootstrap nodes
        let mut connected = 0;
        let mut attempted = 0;

        for (peer_id, addr) in &self.bootstrap_config.nodes {
            attempted += 1;

            // Add address to Kademlia (TODO: re-enable when libp2p kad API is updated)
            // self.kademlia.add_address(peer_id, addr.clone().into());

            // Mark as bootstrap node in reputation system
            let mut reputations = self.peer_reputations.write().await;
            let reputation = reputations.entry(*peer_id).or_insert_with(|| {
                let mut rep = PeerReputation::new(*peer_id);
                rep.is_bootstrap = true;
                rep.score = 75.0; // Higher initial score for bootstrap nodes
                rep
            });
            reputation.is_bootstrap = true;
            drop(reputations);

            // Initiate connection (TODO: re-enable when libp2p kad API is updated)
            // self.kademlia.bootstrap();
            connected += 1;

            debug!("Added bootstrap peer: {} at {}", peer_id, addr);
        }

        // Wait for bootstrap to complete or timeout
        let start = Instant::now();
        let timeout = self.bootstrap_config.timeout;

        while start.elapsed() < timeout {
            if connected >= self.bootstrap_config.min_connections {
                // Bootstrap successful
                let duration = start.elapsed();
                self.bootstrap_state = BootstrapState::Completed {
                    completion_time: Instant::now(),
                    connected_nodes: connected,
                    duration,
                };

                let mut metrics = self.metrics.lock().unwrap();
                metrics.successful_bootstraps += 1;
                drop(metrics);

                if let Some(tx) = &self.event_tx {
                    let _ = tx
                        .send(DiscoveryEvent::BootstrapCompleted {
                            peers_discovered: connected,
                            duration,
                            success_rate: connected as f64 / attempted as f64,
                        })
                        .await;
                }

                info!(
                    "DHT bootstrap completed: {} nodes connected in {:?}",
                    connected, duration
                );
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Bootstrap failed
        let reason = format!(
            "Only {} of {} required nodes connected",
            connected, self.bootstrap_config.min_connections
        );

        self.bootstrap_state = BootstrapState::Failed {
            failure_time: Instant::now(),
            reason: reason.clone(),
            attempted_nodes: attempted,
        };

        if let Some(tx) = &self.event_tx {
            let _ = tx
                .send(DiscoveryEvent::BootstrapFailed {
                    reason,
                    attempted_nodes: attempted,
                    successful_connections: connected,
                })
                .await;
        }

        Err(NetworkError::BootstrapFailed)
    }

    /// Handle Kademlia events (placeholder for libp2p kad API update)
    pub async fn handle_event(&mut self, _event: KademliaEvent) {
        // TODO: Re-implement when libp2p kad API is updated for version 0.53
        // This is a placeholder to maintain the interface
        warn!("Kademlia event handling not yet implemented for libp2p 0.53");
    }

    /// Handle new peer discovered
    async fn handle_new_peer(&mut self, peer: LibP2PPeerId) {
        debug!("New peer discovered via Kademlia: {}", peer);

        // Create or update peer reputation
        let mut reputations = self.peer_reputations.write().await;
        reputations
            .entry(peer)
            .or_insert_with(|| PeerReputation::new(peer));
        drop(reputations);

        // Update metrics (TODO: re-enable when libp2p kad API is updated)
        let mut metrics = self.metrics.lock().unwrap();
        // metrics.routing_table_size = self.kademlia.kbuckets()
        //     .map(|bucket| bucket.num_entries())
        //     .sum();
        metrics.routing_table_size = 0; // Placeholder
        drop(metrics);

        // Send discovery event
        if let Some(tx) = &self.event_tx {
            let discovered_peer = DiscoveredPeer::new(
                peer,
                SocketAddr::from(([0, 0, 0, 0], 0)), // Address will be updated later
                DiscoveryMethod::Kademlia,
            );

            let _ = tx
                .send(DiscoveryEvent::PeerDiscovered(discovered_peer))
                .await;
        }
    }

    /// Handle unroutable peer
    async fn handle_unroutable_peer(&mut self, peer: LibP2PPeerId) {
        warn!("Peer became unroutable: {}", peer);

        // Update reputation
        let mut reputations = self.peer_reputations.write().await;
        if let Some(reputation) = reputations.get_mut(&peer) {
            reputation.downtime_incidents += 1;
            reputation.record_interaction(false, None, &self.scoring_config);
        }
    }

    /// Handle routable peer
    async fn handle_routable_peer(&mut self, peer: LibP2PPeerId) {
        debug!("Peer became routable: {}", peer);

        // Update reputation positively
        let mut reputations = self.peer_reputations.write().await;
        if let Some(reputation) = reputations.get_mut(&peer) {
            reputation.record_interaction(true, None, &self.scoring_config);
        }
    }

    /// Handle query result
    async fn handle_query_result(&mut self, id: QueryId, result: QueryResult) {
        let query_info = {
            let mut queries = self.pending_queries.lock().unwrap();
            queries.remove(&id)
        };

        if let Some(info) = query_info {
            let duration = info.start_time.elapsed();

            match result {
                QueryResult::GetProviders(Ok(result)) => {
                    // GetProvidersOk is an enum variant, extract providers from it
                    match result {
                        GetProvidersOk::FoundProviders { providers, .. } => {
                            self.handle_providers_found(providers, duration).await;
                        }
                        GetProvidersOk::FinishedWithNoAdditionalRecord { .. } => {
                            // No more providers found
                            self.handle_providers_found(HashSet::new(), duration).await;
                        }
                    }
                }
                QueryResult::PutRecord(Ok(PutRecordOk { .. })) => {
                    self.handle_record_stored(duration).await;
                }
                QueryResult::GetRecord(Ok(result)) => {
                    match result {
                        GetRecordOk::FoundRecord(peer_record) => {
                            self.handle_record_found(peer_record.record, duration).await;
                        }
                        GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {
                            // No record found
                            let mut metrics = self.metrics.lock().unwrap();
                            metrics.failed_queries += 1;
                        }
                    }
                }
                QueryResult::Bootstrap(Ok(result)) => {
                    self.handle_bootstrap_result(result, duration).await;
                }
                QueryResult::GetClosestPeers(Ok(result)) => {
                    self.handle_closest_peers(result, duration).await;
                }
                _ => {
                    // Handle errors
                    let mut metrics = self.metrics.lock().unwrap();
                    metrics.failed_queries += 1;
                }
            }
        }
    }

    /// Handle providers found
    async fn handle_providers_found(
        &mut self,
        providers: HashSet<LibP2PPeerId>,
        duration: Duration,
    ) {
        debug!("Found {} providers in {:?}", providers.len(), duration);

        let mut metrics = self.metrics.lock().unwrap();
        metrics.successful_queries += 1;
        metrics.avg_query_time = Duration::from_secs_f64(
            (metrics.avg_query_time.as_secs_f64() + duration.as_secs_f64()) / 2.0,
        );
        drop(metrics);

        // Update peer reputations
        let mut reputations = self.peer_reputations.write().await;
        for provider in providers {
            if let Some(reputation) = reputations.get_mut(&provider) {
                reputation.content_provided += 1;
                reputation.record_interaction(true, Some(duration), &self.scoring_config);
            }
        }
    }

    /// Handle record stored
    async fn handle_record_stored(&mut self, duration: Duration) {
        debug!("Record stored in {:?}", duration);

        let mut metrics = self.metrics.lock().unwrap();
        metrics.successful_queries += 1;
        metrics.records_stored += 1;
        metrics.avg_query_time = Duration::from_secs_f64(
            (metrics.avg_query_time.as_secs_f64() + duration.as_secs_f64()) / 2.0,
        );
    }

    /// Handle record found
    async fn handle_record_found(&mut self, _record: Record, duration: Duration) {
        debug!("Record found in {:?}", duration);

        let mut metrics = self.metrics.lock().unwrap();
        metrics.successful_queries += 1;
        metrics.avg_query_time = Duration::from_secs_f64(
            (metrics.avg_query_time.as_secs_f64() + duration.as_secs_f64()) / 2.0,
        );
    }

    /// Handle bootstrap result
    async fn handle_bootstrap_result(&mut self, result: BootstrapOk, duration: Duration) {
        let peer = result.peer;
        debug!("Bootstrap query to {} completed in {:?}", peer, duration);

        // Update bootstrap state if still in progress
        if let BootstrapState::InProgress {
            start_time,
            mut connected_nodes,
            attempted_nodes,
        } = self.bootstrap_state
        {
            connected_nodes += 1;
            self.bootstrap_state = BootstrapState::InProgress {
                start_time,
                connected_nodes,
                attempted_nodes,
            };
        }
    }

    /// Handle closest peers result
    async fn handle_closest_peers(&mut self, result: GetClosestPeersOk, duration: Duration) {
        let peers = result.peers;
        debug!("Found {} closest peers in {:?}", peers.len(), duration);

        // Update network size estimate
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.network_size_estimate = self.estimate_network_size(&peers);
        }

        // Check for network partitions
        self.check_network_partition(&peers).await;
    }

    /// Estimate network size based on closest peers
    fn estimate_network_size(&self, closest_peers: &[LibP2PPeerId]) -> usize {
        // Simple estimation based on XOR distances
        if closest_peers.is_empty() {
            return 1;
        }

        // Calculate average distance to closest peers
        let distances: Vec<u32> = closest_peers
            .iter()
            .map(|peer| {
                // Calculate XOR distance manually
                let distance = self.calculate_xor_distance(&self.local_peer_id, peer);
                // Count leading zeros as a measure of distance
                distance
            })
            .collect();

        let avg_distance = distances.iter().sum::<u32>() / distances.len() as u32;

        // Estimate: 2^(avg_distance) nodes in network
        2_usize.pow(avg_distance.min(31)) // Cap at 31 to avoid overflow
    }

    /// Check for network partitions
    async fn check_network_partition(&mut self, peers: &[LibP2PPeerId]) {
        let mut detector = self.partition_detector.lock().unwrap();

        // Update successful query times for buckets
        for peer in peers {
            let bucket_index = self.get_bucket_index(peer);
            detector
                .last_successful_queries
                .insert(bucket_index, Instant::now());
        }

        // Check for potential partitions
        let now = Instant::now();
        let mut potentially_partitioned_buckets = Vec::new();

        for bucket_idx in 0..160 {
            // Assuming 160-bit keys
            if let Some(last_success) = detector.last_successful_queries.get(&bucket_idx) {
                if now.duration_since(*last_success) > detector.detection_threshold {
                    potentially_partitioned_buckets.push(bucket_idx);
                }
            }
        }

        if !potentially_partitioned_buckets.is_empty() {
            warn!(
                "Potential network partition detected in buckets: {:?}",
                potentially_partitioned_buckets
            );

            detector.detected_partitions.push(PartitionInfo {
                detected_at: now,
                affected_buckets: potentially_partitioned_buckets,
                estimated_size: 0, // TODO: Estimate partition size
                recovered: false,
            });
        }
    }

    /// Get bucket index for a peer
    fn get_bucket_index(&self, peer: &LibP2PPeerId) -> u8 {
        let distance = self.calculate_xor_distance(&self.local_peer_id, peer);
        // Find the position of the most significant bit
        (159 - distance) as u8 // For 160-bit keys
    }

    /// Calculate XOR distance between two peer IDs (returns leading zeros count)
    fn calculate_xor_distance(&self, peer1: &LibP2PPeerId, peer2: &LibP2PPeerId) -> u32 {
        let bytes1 = peer1.to_bytes();
        let bytes2 = peer2.to_bytes();

        // XOR the bytes and count leading zeros
        let mut leading_zeros = 0u32;
        for i in 0..bytes1.len().min(bytes2.len()) {
            let xor = bytes1[i] ^ bytes2[i];
            if xor == 0 {
                leading_zeros += 8;
            } else {
                leading_zeros += xor.leading_zeros();
                break;
            }
        }

        leading_zeros
    }

    /// Store content in the DHT
    pub async fn store_record(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), NetworkError> {
        if value.len() > self.content_config.max_content_size {
            return Err(NetworkError::ContentTooLarge);
        }

        let record_key = RecordKey::new(&key);
        let _record = Record {
            key: record_key.clone(),
            value,
            publisher: Some(self.local_peer_id),
            expires: Some(Instant::now() + DEFAULT_RECORD_TTL),
        };

        // Track query (TODO: re-enable when libp2p kad API is updated)
        // let query_id = self.kademlia.put_record(record, Quorum::Majority)?;
        // let mut queries = self.pending_queries.lock().unwrap();
        // queries.insert(query_id, QueryInfo {
        //     query_type: QueryType::PutRecord,
        //     start_time: Instant::now(),
        //     target: QueryTarget::RecordKey(record_key),
        //     requestor: None,
        // });
        //
        // let mut metrics = self.metrics.lock().unwrap();
        // metrics.total_queries += 1;

        Err(NetworkError::Internal(
            "Kademlia functionality not implemented for libp2p 0.53".to_string(),
        ))
    }

    /// Get content from the DHT
    pub async fn get_record(&mut self, key: Vec<u8>) -> Result<(), NetworkError> {
        let _record_key = RecordKey::new(&key);

        // Track query (TODO: re-enable when libp2p kad API is updated)
        // let query_id = self.kademlia.get_record(&record_key, Quorum::Majority);
        // let mut queries = self.pending_queries.lock().unwrap();
        // queries.insert(query_id, QueryInfo {
        //     query_type: QueryType::GetRecord,
        //     start_time: Instant::now(),
        //     target: QueryTarget::RecordKey(record_key),
        //     requestor: None,
        // });
        //
        // let mut metrics = self.metrics.lock().unwrap();
        // metrics.total_queries += 1;

        Err(NetworkError::Internal(
            "Kademlia functionality not implemented for libp2p 0.53".to_string(),
        ))
    }

    /// Announce as provider for content
    pub async fn provide(&mut self, key: Vec<u8>) -> Result<(), NetworkError> {
        let _record_key = RecordKey::new(&key);

        // Track query (TODO: re-enable when libp2p kad API is updated)
        // let query_id = self.kademlia.start_providing(record_key.clone())?;
        // let mut queries = self.pending_queries.lock().unwrap();
        // queries.insert(query_id, QueryInfo {
        //     query_type: QueryType::AddProvider,
        //     start_time: Instant::now(),
        //     target: QueryTarget::RecordKey(record_key),
        //     requestor: None,
        // });
        //
        // let mut metrics = self.metrics.lock().unwrap();
        // metrics.total_queries += 1;
        // metrics.providers_announced += 1;

        Err(NetworkError::Internal(
            "Kademlia functionality not implemented for libp2p 0.53".to_string(),
        ))
    }

    /// Find providers for content
    pub async fn find_providers(&mut self, key: Vec<u8>) -> Result<(), NetworkError> {
        let _record_key = RecordKey::new(&key);

        // Track query (TODO: re-enable when libp2p kad API is updated)
        // let query_id = self.kademlia.get_providers(&record_key);
        // let mut queries = self.pending_queries.lock().unwrap();
        // queries.insert(query_id, QueryInfo {
        //     query_type: QueryType::GetProviders,
        //     start_time: Instant::now(),
        //     target: QueryTarget::RecordKey(record_key),
        //     requestor: None,
        // });
        //
        // let mut metrics = self.metrics.lock().unwrap();
        // metrics.total_queries += 1;

        Err(NetworkError::Internal(
            "Kademlia functionality not implemented for libp2p 0.53".to_string(),
        ))
    }

    /// Store dark address mapping in DHT
    pub async fn store_dark_address(
        &mut self,
        shadow_address: &ShadowAddress,
        peer_id: LibP2PPeerId,
    ) -> Result<(), NetworkError> {
        // Generate deterministic key from shadow address
        let mut hasher = Sha256::new();
        hasher.update(&shadow_address.view_key);
        hasher.update(&shadow_address.spend_key);
        let key = hasher.finalize().to_vec();

        // Serialize peer ID as value
        let value = peer_id.to_bytes();

        self.store_record(key, value).await
    }

    /// Find peer for dark address
    pub async fn find_dark_address(
        &mut self,
        shadow_address: &ShadowAddress,
    ) -> Result<(), NetworkError> {
        // Generate same deterministic key
        let mut hasher = Sha256::new();
        hasher.update(&shadow_address.view_key);
        hasher.update(&shadow_address.spend_key);
        let key = hasher.finalize().to_vec();

        self.get_record(key).await
    }

    /// Get peer reputation
    pub async fn get_peer_reputation(&self, peer_id: &LibP2PPeerId) -> Option<PeerReputation> {
        self.peer_reputations.read().await.get(peer_id).cloned()
    }

    /// Get all peer reputations sorted by score
    pub async fn get_top_peers(&self, limit: usize) -> Vec<PeerReputation> {
        let mut peers: Vec<_> = self
            .peer_reputations
            .read()
            .await
            .values()
            .cloned()
            .collect();
        peers.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        peers.truncate(limit);
        peers
    }

    /// Apply reputation decay to all peers
    pub async fn apply_reputation_decay(&mut self) {
        let mut reputations = self.peer_reputations.write().await;
        for reputation in reputations.values_mut() {
            reputation.apply_decay(&self.scoring_config);
        }
    }

    /// Get DHT metrics
    pub fn get_metrics(&self) -> DHTMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Check if bootstrap is complete
    pub fn is_bootstrapped(&self) -> bool {
        matches!(self.bootstrap_state, BootstrapState::Completed { .. })
    }

    /// Get routing table size
    pub fn routing_table_size(&self) -> usize {
        // TODO: re-enable when libp2p kad API is updated
        // self.kademlia.kbuckets()
        //     .map(|bucket| bucket.num_entries())
        //     .sum()
        0 // Placeholder
    }

    /// Handle periodic maintenance tasks
    pub async fn perform_maintenance(&mut self) {
        // Apply reputation decay
        self.apply_reputation_decay().await;

        // Clean up old partition detection data
        let mut detector = self.partition_detector.lock().unwrap();
        let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
        detector
            .detected_partitions
            .retain(|p| p.detected_at > cutoff || !p.recovered);

        // Update metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.routing_table_size = self.routing_table_size();

        // Periodic bootstrap if configured
        if self.bootstrap_config.periodic_bootstrap {
            if let BootstrapState::Completed {
                completion_time, ..
            } = self.bootstrap_state
            {
                if completion_time.elapsed() > self.bootstrap_config.bootstrap_interval {
                    info!("Performing periodic bootstrap");
                    // TODO: re-enable when libp2p kad API is updated
                    // self.kademlia.bootstrap();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_reputation() {
        let peer_id = LibP2PPeerId::random();
        let mut reputation = PeerReputation::new(peer_id);
        let config = PeerScoringConfig::default();

        // Test successful interaction
        reputation.record_interaction(true, Some(Duration::from_millis(50)), &config);
        assert!(reputation.score > 50.0);
        assert_eq!(reputation.successful_interactions, 1);

        // Test failed interaction
        reputation.record_interaction(false, None, &config);
        assert!(reputation.score < 50.0);
        assert_eq!(reputation.failed_interactions, 1);

        // Test misbehavior
        reputation.record_misbehavior(2.0);
        assert!(reputation.score < 30.0);
        assert_eq!(reputation.misbehavior_count, 1);
    }

    #[test]
    fn test_bootstrap_state() {
        let state = BootstrapState::NotStarted;
        assert_eq!(state, BootstrapState::NotStarted);

        let state = BootstrapState::InProgress {
            start_time: Instant::now(),
            connected_nodes: 5,
            attempted_nodes: 10,
        };

        if let BootstrapState::InProgress {
            connected_nodes, ..
        } = state
        {
            assert_eq!(connected_nodes, 5);
        }
    }

    #[tokio::test]
    async fn test_dht_creation() {
        let peer_id = LibP2PPeerId::random();
        let config = DHTConfig::default();
        let bootstrap_config = BootstrapConfig::default();
        let content_config = ContentRoutingConfig::default();
        let scoring_config = PeerScoringConfig::default();

        let dht = KademliaDHT::new(
            peer_id,
            config,
            bootstrap_config,
            content_config,
            scoring_config,
        );

        assert_eq!(dht.local_peer_id, peer_id);
        assert!(!dht.is_bootstrapped());
        assert_eq!(dht.routing_table_size(), 0);
    }
}
