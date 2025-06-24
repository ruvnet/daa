# QuDAG Connection Management & NAT Traversal Strategy

## Overview

This document outlines the comprehensive connection management system for QuDAG, focusing on robust NAT traversal, connection pooling, health monitoring, and fault tolerance. The system is designed to handle diverse network conditions while maintaining high availability and performance.

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                 Connection Management System                 │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │ Connection Pool │  │ Health Monitor  │  │Circuit Breaker││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │ NAT Traversal   │  │ Relay Manager   │  │ Load Balancer││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │Bandwidth Manager│  │ Metrics Collector│ │ Maintenance  ││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## 1. Connection Pool Management

### 1.1 Pool Architecture

```rust
pub struct ConnectionPool {
    /// Active connections indexed by peer
    connections: Arc<DashMap<PeerId, PooledConnection>>,
    
    /// Pool configuration
    config: PoolConfig,
    
    /// Connection factory for creating new connections
    factory: Arc<dyn ConnectionFactory>,
    
    /// Health monitor
    health_monitor: Arc<HealthMonitor>,
    
    /// Metrics collector
    metrics: Arc<PoolMetrics>,
    
    /// Cleanup scheduler
    cleanup_scheduler: Arc<CleanupScheduler>,
}

pub struct PoolConfig {
    /// Maximum connections per peer
    max_connections_per_peer: usize,
    
    /// Global connection limit
    max_total_connections: usize,
    
    /// Idle timeout for connections
    idle_timeout: Duration,
    
    /// Connection keep-alive interval
    keep_alive_interval: Duration,
    
    /// Health check interval
    health_check_interval: Duration,
    
    /// Connection retry configuration
    retry_config: RetryConfig,
}

pub struct PooledConnection {
    /// The underlying connection
    connection: Box<dyn NetworkConnection>,
    
    /// Connection metadata
    metadata: ConnectionMetadata,
    
    /// Usage statistics
    stats: ConnectionStats,
    
    /// Current state
    state: ConnectionState,
    
    /// Reference counter for active usage
    ref_count: AtomicUsize,
}

pub struct ConnectionMetadata {
    /// Peer identifier
    peer_id: PeerId,
    
    /// Connection established timestamp
    established_at: Instant,
    
    /// Last activity timestamp
    last_activity: Instant,
    
    /// Connection properties
    properties: ConnectionProperties,
    
    /// Transport type used
    transport_type: TransportType,
}

pub struct ConnectionProperties {
    /// Connection latency
    latency: Duration,
    
    /// Available bandwidth
    bandwidth: Option<u64>,
    
    /// Connection security level
    security_level: SecurityLevel,
    
    /// NAT type detected
    nat_type: Option<NATType>,
    
    /// Connection quality score
    quality_score: f64,
}

impl ConnectionPool {
    /// Get or create connection to peer
    pub async fn get_connection(&self, peer_id: &PeerId) -> Result<PooledConnectionRef> {
        // Check existing connections
        if let Some(pooled) = self.connections.get(peer_id) {
            if pooled.is_healthy() {
                // Increment reference count
                pooled.ref_count.fetch_add(1, Ordering::Relaxed);
                return Ok(PooledConnectionRef::new(pooled.value().clone()));
            }
        }
        
        // Check pool limits
        if self.connections.len() >= self.config.max_total_connections {
            // Try to evict idle connections
            self.evict_idle_connections().await;
            
            if self.connections.len() >= self.config.max_total_connections {
                return Err(ConnectionError::PoolFull);
            }
        }
        
        // Create new connection
        let connection = self.create_new_connection(peer_id).await?;
        
        // Add to pool
        let pooled = PooledConnection {
            connection,
            metadata: ConnectionMetadata {
                peer_id: *peer_id,
                established_at: Instant::now(),
                last_activity: Instant::now(),
                properties: ConnectionProperties::default(),
                transport_type: TransportType::QUIC,
            },
            stats: ConnectionStats::default(),
            state: ConnectionState::Active,
            ref_count: AtomicUsize::new(1),
        };
        
        self.connections.insert(*peer_id, pooled.clone());
        
        // Update metrics
        self.metrics.record_connection_created();
        
        Ok(PooledConnectionRef::new(pooled))
    }
    
    /// Create new connection with NAT traversal
    async fn create_new_connection(&self, peer_id: &PeerId) -> Result<Box<dyn NetworkConnection>> {
        let mut attempts = 0;
        let max_attempts = self.config.retry_config.max_attempts;
        
        while attempts < max_attempts {
            match self.attempt_connection(peer_id).await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(e);
                    }
                    
                    // Exponential backoff
                    let delay = self.config.retry_config.base_delay * 2_u32.pow(attempts - 1);
                    sleep(delay).await;
                }
            }
        }
        
        Err(ConnectionError::MaxRetriesExceeded)
    }
    
    /// Background maintenance task
    pub async fn maintenance_loop(&self) {
        let mut interval = interval(self.config.health_check_interval);
        
        loop {
            interval.tick().await;
            
            // Health check all connections
            self.health_check_connections().await;
            
            // Cleanup stale connections
            self.cleanup_stale_connections().await;
            
            // Update connection metrics
            self.update_metrics().await;
        }
    }
}
```

### 1.2 Connection Reference Management

```rust
/// Smart pointer for pooled connections
pub struct PooledConnectionRef {
    connection: Arc<PooledConnection>,
    pool: Weak<ConnectionPool>,
}

impl PooledConnectionRef {
    pub fn new(connection: PooledConnection) -> Self {
        Self {
            connection: Arc::new(connection),
            pool: Weak::new(),
        }
    }
    
    /// Send data through connection
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        // Update activity timestamp
        self.connection.metadata.last_activity = Instant::now();
        
        // Update statistics
        self.connection.stats.bytes_sent.fetch_add(data.len() as u64, Ordering::Relaxed);
        self.connection.stats.messages_sent.fetch_add(1, Ordering::Relaxed);
        
        // Send through underlying connection
        self.connection.connection.send(data).await
    }
    
    /// Receive data from connection
    pub async fn receive(&self) -> Result<Vec<u8>> {
        let data = self.connection.connection.receive().await?;
        
        // Update activity and stats
        self.connection.metadata.last_activity = Instant::now();
        self.connection.stats.bytes_received.fetch_add(data.len() as u64, Ordering::Relaxed);
        self.connection.stats.messages_received.fetch_add(1, Ordering::Relaxed);
        
        Ok(data)
    }
}

impl Drop for PooledConnectionRef {
    fn drop(&mut self) {
        // Decrement reference count
        self.connection.ref_count.fetch_sub(1, Ordering::Relaxed);
        
        // If this was the last reference and connection is idle, mark for cleanup
        if self.connection.ref_count.load(Ordering::Relaxed) == 0 {
            if let Some(pool) = self.pool.upgrade() {
                pool.schedule_cleanup(&self.connection.metadata.peer_id);
            }
        }
    }
}
```

## 2. NAT Traversal Implementation

### 2.1 Multi-Strategy NAT Traversal

```rust
pub struct NATTraversalManager {
    /// Available traversal strategies
    strategies: Vec<Box<dyn NATStrategy>>,
    
    /// STUN client for address discovery
    stun_client: Arc<StunClient>,
    
    /// UPnP client for port mapping
    upnp_client: Option<Arc<UpnpClient>>,
    
    /// Relay manager for fallback
    relay_manager: Arc<RelayManager>,
    
    /// NAT type detector
    nat_detector: Arc<NATDetector>,
    
    /// Strategy selector
    strategy_selector: Arc<StrategySelector>,
}

pub trait NATStrategy: Send + Sync {
    /// Strategy name
    fn name(&self) -> &'static str;
    
    /// Attempt to establish connection
    async fn attempt_connection(&self, peer_info: &PeerInfo) -> Result<Connection>;
    
    /// Get strategy priority for given NAT types
    fn priority(&self, local_nat: NATType, remote_nat: NATType) -> u8;
    
    /// Strategy success rate
    fn success_rate(&self) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NATType {
    /// No NAT - direct connection possible
    None,
    
    /// Full cone NAT
    FullCone,
    
    /// Restricted cone NAT
    RestrictedCone,
    
    /// Port restricted cone NAT
    PortRestrictedCone,
    
    /// Symmetric NAT
    Symmetric,
    
    /// Unknown NAT type
    Unknown,
}

impl NATTraversalManager {
    /// Establish connection using best available strategy
    pub async fn establish_connection(&self, peer_info: &PeerInfo) -> Result<Connection> {
        // Detect local NAT type
        let local_nat = self.nat_detector.detect_nat_type().await?;
        
        // Get peer's NAT type from discovery
        let remote_nat = peer_info.nat_type.unwrap_or(NATType::Unknown);
        
        // Select best strategy
        let strategy = self.strategy_selector.select_strategy(local_nat, remote_nat);
        
        info!("Attempting connection with strategy: {} (Local NAT: {:?}, Remote NAT: {:?})", 
              strategy.name(), local_nat, remote_nat);
        
        // Try selected strategy first
        match strategy.attempt_connection(peer_info).await {
            Ok(conn) => {
                info!("Connection established using strategy: {}", strategy.name());
                return Ok(conn);
            }
            Err(e) => {
                warn!("Primary strategy {} failed: {:?}", strategy.name(), e);
            }
        }
        
        // Try other strategies in order of priority
        let mut strategies = self.strategies.clone();
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority(local_nat, remote_nat)));
        
        for strategy in strategies {
            if strategy.name() == strategy.name() {
                continue; // Skip already tried strategy
            }
            
            match strategy.attempt_connection(peer_info).await {
                Ok(conn) => {
                    info!("Connection established using fallback strategy: {}", strategy.name());
                    return Ok(conn);
                }
                Err(e) => {
                    warn!("Strategy {} failed: {:?}", strategy.name(), e);
                }
            }
        }
        
        Err(ConnectionError::AllStrategiesFailed)
    }
}
```

### 2.2 STUN/TURN Implementation

```rust
pub struct StunClient {
    /// STUN server endpoints
    stun_servers: Vec<SocketAddr>,
    
    /// Current server index
    current_server: AtomicUsize,
    
    /// Client socket
    socket: Arc<UdpSocket>,
    
    /// Transaction manager
    transactions: Arc<DashMap<TransactionId, PendingTransaction>>,
}

impl StunClient {
    /// Discover external address
    pub async fn discover_external_address(&self) -> Result<SocketAddr> {
        let server = self.get_next_server();
        
        // Create STUN binding request
        let transaction_id = TransactionId::generate();
        let request = StunMessage::new_request(BINDING, transaction_id);
        
        // Send request
        self.socket.send_to(&request.encode(), server).await?;
        
        // Wait for response
        let response = self.wait_for_response(transaction_id).await?;
        
        // Extract mapped address
        response.get_mapped_address()
            .ok_or(StunError::NoMappedAddress)
    }
    
    /// Check for NAT behavior
    pub async fn check_nat_behavior(&self) -> Result<NATBehavior> {
        // Perform multiple STUN requests to different servers
        let mut results = Vec::new();
        
        for &server in &self.stun_servers {
            if let Ok(addr) = self.discover_external_address_from(server).await {
                results.push(addr);
            }
        }
        
        // Analyze results to determine NAT behavior
        self.analyze_nat_behavior(results)
    }
}

pub struct TurnClient {
    /// TURN server configuration
    turn_config: TurnConfig,
    
    /// Authentication credentials
    credentials: TurnCredentials,
    
    /// Allocated relay addresses
    relay_addresses: Arc<RwLock<HashMap<SocketAddr, RelayAllocation>>>,
    
    /// Permission manager
    permission_manager: Arc<PermissionManager>,
}

impl TurnClient {
    /// Allocate relay address
    pub async fn allocate_relay(&self) -> Result<SocketAddr> {
        let request = TurnMessage::allocate_request(&self.credentials);
        let response = self.send_turn_request(request).await?;
        
        let relay_addr = response.get_relay_address()?;
        
        // Store allocation
        let allocation = RelayAllocation {
            relay_address: relay_addr,
            allocated_at: Instant::now(),
            lifetime: response.get_lifetime()?,
            permissions: HashSet::new(),
        };
        
        self.relay_addresses.write().await.insert(relay_addr, allocation);
        
        Ok(relay_addr)
    }
    
    /// Create permission for peer
    pub async fn create_permission(&self, peer_addr: SocketAddr) -> Result<()> {
        let request = TurnMessage::create_permission_request(peer_addr, &self.credentials);
        self.send_turn_request(request).await?;
        
        // Update permissions
        for allocation in self.relay_addresses.write().await.values_mut() {
            allocation.permissions.insert(peer_addr);
        }
        
        Ok(())
    }
}
```

### 2.3 UPnP Implementation

```rust
pub struct UpnpClient {
    /// UPnP device
    device: Arc<Mutex<Option<Device>>>,
    
    /// Current port mappings
    mappings: Arc<RwLock<HashMap<u16, PortMapping>>>,
    
    /// Discovery timeout
    discovery_timeout: Duration,
}

pub struct PortMapping {
    /// External port
    external_port: u16,
    
    /// Internal port
    internal_port: u16,
    
    /// Protocol (TCP/UDP)
    protocol: Protocol,
    
    /// Mapping description
    description: String,
    
    /// Lease duration
    lease_duration: Duration,
    
    /// Created timestamp
    created_at: Instant,
}

impl UpnpClient {
    /// Discover UPnP gateway
    pub async fn discover_gateway(&self) -> Result<()> {
        let discovery_future = async {
            // Search for IGD devices
            let search_target = "urn:schemas-upnp-org:device:InternetGatewayDevice:1";
            let responses = ssdp::search(search_target, Duration::from_secs(3)).await?;
            
            for response in responses {
                if let Ok(device) = Device::from_url(&response.location).await {
                    *self.device.lock().await = Some(device);
                    return Ok(());
                }
            }
            
            Err(UpnpError::NoGatewayFound)
        };
        
        timeout(self.discovery_timeout, discovery_future).await?
    }
    
    /// Create port mapping
    pub async fn create_port_mapping(
        &self,
        external_port: u16,
        internal_port: u16,
        protocol: Protocol,
        description: &str,
        lease_duration: Duration,
    ) -> Result<PortMapping> {
        let device = self.device.lock().await;
        let device = device.as_ref().ok_or(UpnpError::NoDevice)?;
        
        // Add port mapping
        device.add_port_mapping(
            external_port,
            internal_port,
            protocol,
            &description,
            lease_duration.as_secs() as u32,
        ).await?;
        
        let mapping = PortMapping {
            external_port,
            internal_port,
            protocol,
            description: description.to_string(),
            lease_duration,
            created_at: Instant::now(),
        };
        
        self.mappings.write().await.insert(external_port, mapping.clone());
        
        Ok(mapping)
    }
    
    /// Cleanup expired mappings
    pub async fn cleanup_expired_mappings(&self) {
        let now = Instant::now();
        let mut mappings = self.mappings.write().await;
        
        mappings.retain(|_, mapping| {
            if now.duration_since(mapping.created_at) > mapping.lease_duration {
                // Remove mapping from gateway
                if let Some(device) = self.device.lock().await.as_ref() {
                    let _ = device.remove_port_mapping(mapping.external_port, mapping.protocol);
                }
                false
            } else {
                true
            }
        });
    }
}
```

### 2.4 Hole Punching Implementation

```rust
pub struct HolePunchingManager {
    /// Signaling server for coordination
    signaling_server: Arc<SignalingClient>,
    
    /// Local socket for punching
    local_socket: Arc<UdpSocket>,
    
    /// Pending punch attempts
    pending_punches: Arc<DashMap<PeerId, PunchAttempt>>,
    
    /// Success statistics
    success_stats: Arc<Mutex<HolePunchingStats>>,
}

pub struct PunchAttempt {
    /// Target peer
    peer_id: PeerId,
    
    /// Target address
    target_addr: SocketAddr,
    
    /// Our external address
    our_external_addr: SocketAddr,
    
    /// Attempt start time
    started_at: Instant,
    
    /// Completion channel
    completion_tx: oneshot::Sender<Result<UdpSocket>>,
}

impl HolePunchingManager {
    /// Attempt hole punching to peer
    pub async fn punch_hole(&self, peer_id: PeerId) -> Result<UdpSocket> {
        // Exchange addresses via signaling server
        let (our_addr, peer_addr) = self.exchange_addresses(peer_id).await?;
        
        info!("Starting hole punching: {} -> {}", our_addr, peer_addr);
        
        // Create punch attempt
        let (completion_tx, completion_rx) = oneshot::channel();
        let attempt = PunchAttempt {
            peer_id,
            target_addr: peer_addr,
            our_external_addr: our_addr,
            started_at: Instant::now(),
            completion_tx,
        };
        
        self.pending_punches.insert(peer_id, attempt);
        
        // Start punching process
        self.perform_hole_punch(peer_id, our_addr, peer_addr).await;
        
        // Wait for completion
        completion_rx.await?
    }
    
    async fn perform_hole_punch(&self, peer_id: PeerId, our_addr: SocketAddr, peer_addr: SocketAddr) {
        let socket = &self.local_socket;
        
        // Send punch packets
        for i in 0..10 {
            let punch_data = format!("PUNCH-{}-{}", peer_id, i);
            
            if let Err(e) = socket.send_to(punch_data.as_bytes(), peer_addr).await {
                warn!("Punch packet {} failed: {:?}", i, e);
            }
            
            // Check for response
            if let Ok(Some(response_socket)) = self.check_for_response(peer_id).await {
                // Success!
                if let Some(attempt) = self.pending_punches.remove(&peer_id) {
                    let _ = attempt.1.completion_tx.send(Ok(response_socket));
                    
                    // Update statistics
                    self.success_stats.lock().await.record_success(
                        Instant::now().duration_since(attempt.1.started_at)
                    );
                }
                return;
            }
            
            // Exponential backoff with jitter
            let delay = Duration::from_millis(100 * 2_u64.pow(i.min(5)) + thread_rng().gen_range(0..50));
            sleep(delay).await;
        }
        
        // Timeout - punch failed
        if let Some(attempt) = self.pending_punches.remove(&peer_id) {
            let _ = attempt.1.completion_tx.send(Err(ConnectionError::HolePunchFailed));
            self.success_stats.lock().await.record_failure();
        }
    }
    
    async fn check_for_response(&self, peer_id: PeerId) -> Result<Option<UdpSocket>> {
        // Set up listener for punch responses
        let mut buf = [0u8; 1024];
        
        match timeout(Duration::from_millis(500), self.local_socket.recv_from(&mut buf)).await {
            Ok(Ok((len, addr))) => {
                let data = String::from_utf8_lossy(&buf[..len]);
                
                if data.starts_with(&format!("PUNCH-{}", peer_id)) {
                    // Create connected socket
                    let connected_socket = UdpSocket::bind("0.0.0.0:0").await?;
                    connected_socket.connect(addr).await?;
                    
                    info!("Hole punch successful to {}", addr);
                    return Ok(Some(connected_socket));
                }
            }
            _ => {}
        }
        
        Ok(None)
    }
}
```

## 3. Health Monitoring and Circuit Breakers

### 3.1 Health Monitor Implementation

```rust
pub struct HealthMonitor {
    /// Monitored connections
    connections: Arc<DashMap<PeerId, ConnectionHealth>>,
    
    /// Health check configuration
    config: HealthConfig,
    
    /// Circuit breakers
    circuit_breakers: Arc<DashMap<PeerId, CircuitBreaker>>,
    
    /// Health metrics
    metrics: Arc<HealthMetrics>,
}

pub struct ConnectionHealth {
    /// Health status
    status: HealthStatus,
    
    /// Last health check
    last_check: Instant,
    
    /// Response time history
    response_times: VecDeque<Duration>,
    
    /// Error count
    error_count: u32,
    
    /// Success count
    success_count: u32,
    
    /// Last error
    last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthMonitor {
    /// Perform health check on connection
    pub async fn health_check(&self, peer_id: &PeerId) -> HealthStatus {
        let start = Instant::now();
        
        // Send ping message
        match self.send_ping(peer_id).await {
            Ok(response_time) => {
                self.record_success(peer_id, response_time).await;
                self.calculate_health_status(peer_id)
            }
            Err(e) => {
                self.record_error(peer_id, e).await;
                HealthStatus::Unhealthy
            }
        }
    }
    
    async fn record_success(&self, peer_id: &PeerId, response_time: Duration) {
        let mut health = self.connections.entry(*peer_id).or_insert_with(ConnectionHealth::new);
        
        health.success_count += 1;
        health.response_times.push_back(response_time);
        health.last_check = Instant::now();
        
        // Keep only recent response times
        while health.response_times.len() > self.config.max_response_history {
            health.response_times.pop_front();
        }
        
        // Update circuit breaker
        if let Some(mut breaker) = self.circuit_breakers.get_mut(peer_id) {
            breaker.record_success();
        }
    }
    
    async fn record_error(&self, peer_id: &PeerId, error: ConnectionError) {
        let mut health = self.connections.entry(*peer_id).or_insert_with(ConnectionHealth::new);
        
        health.error_count += 1;
        health.last_error = Some(error.to_string());
        health.last_check = Instant::now();
        
        // Update circuit breaker
        if let Some(mut breaker) = self.circuit_breakers.get_mut(peer_id) {
            breaker.record_failure();
        }
    }
    
    fn calculate_health_status(&self, peer_id: &PeerId) -> HealthStatus {
        if let Some(health) = self.connections.get(peer_id) {
            let total_checks = health.success_count + health.error_count;
            
            if total_checks == 0 {
                return HealthStatus::Unknown;
            }
            
            let success_rate = health.success_count as f64 / total_checks as f64;
            let avg_response_time = health.response_times.iter().sum::<Duration>() 
                / health.response_times.len() as u32;
            
            match (success_rate, avg_response_time) {
                (rate, time) if rate >= 0.95 && time < self.config.healthy_response_threshold => {
                    HealthStatus::Healthy
                }
                (rate, time) if rate >= 0.80 && time < self.config.degraded_response_threshold => {
                    HealthStatus::Degraded
                }
                _ => HealthStatus::Unhealthy,
            }
        } else {
            HealthStatus::Unknown
        }
    }
}
```

### 3.2 Circuit Breaker Implementation

```rust
pub struct CircuitBreaker {
    /// Current state
    state: CircuitBreakerState,
    
    /// Configuration
    config: CircuitBreakerConfig,
    
    /// Failure count in current window
    failure_count: u32,
    
    /// Success count in current window
    success_count: u32,
    
    /// Last state change
    last_state_change: Instant,
    
    /// Next allowed attempt (for half-open state)
    next_attempt: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,  // Normal operation
    Open,    // Failing fast
    HalfOpen, // Testing if service recovered
}

pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    failure_threshold: u32,
    
    /// Success threshold to close circuit
    success_threshold: u32,
    
    /// Timeout before trying half-open
    timeout: Duration,
    
    /// Time window for counting failures
    window_duration: Duration,
}

impl CircuitBreaker {
    /// Record successful operation
    pub fn record_success(&mut self) {
        self.success_count += 1;
        
        match self.state {
            CircuitBreakerState::HalfOpen => {
                if self.success_count >= self.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            _ => {}
        }
    }
    
    /// Record failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        
        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.transition_to_open();
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open goes back to open
                self.transition_to_open();
            }
            _ => {}
        }
    }
    
    /// Check if request should be allowed
    pub fn should_allow_request(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout expired
                if self.last_state_change.elapsed() >= self.config.timeout {
                    self.transition_to_half_open();
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests in half-open state
                if let Some(next) = self.next_attempt {
                    if Instant::now() >= next {
                        self.next_attempt = Some(Instant::now() + Duration::from_millis(100));
                        true
                    } else {
                        false
                    }
                } else {
                    self.next_attempt = Some(Instant::now() + Duration::from_millis(100));
                    true
                }
            }
        }
    }
    
    fn transition_to_open(&mut self) {
        info!("Circuit breaker opening due to failures");
        self.state = CircuitBreakerState::Open;
        self.last_state_change = Instant::now();
        self.reset_counters();
    }
    
    fn transition_to_half_open(&mut self) {
        info!("Circuit breaker transitioning to half-open");
        self.state = CircuitBreakerState::HalfOpen;
        self.last_state_change = Instant::now();
        self.next_attempt = Some(Instant::now());
        self.reset_counters();
    }
    
    fn transition_to_closed(&mut self) {
        info!("Circuit breaker closing - service recovered");
        self.state = CircuitBreakerState::Closed;
        self.last_state_change = Instant::now();
        self.next_attempt = None;
        self.reset_counters();
    }
    
    fn reset_counters(&mut self) {
        self.failure_count = 0;
        self.success_count = 0;
    }
}
```

## 4. Bandwidth Management

### 4.1 Bandwidth Limiter

```rust
pub struct BandwidthLimiter {
    /// Upload rate limiter
    upload_limiter: Arc<RateLimiter>,
    
    /// Download rate limiter
    download_limiter: Arc<RateLimiter>,
    
    /// Per-peer limits
    peer_limits: Arc<DashMap<PeerId, PeerBandwidthLimiter>>,
    
    /// Global bandwidth configuration
    config: BandwidthConfig,
    
    /// Usage statistics
    stats: Arc<BandwidthStats>,
}

pub struct BandwidthConfig {
    /// Global upload limit (bytes/sec)
    global_upload_limit: u64,
    
    /// Global download limit (bytes/sec)
    global_download_limit: u64,
    
    /// Per-peer upload limit (bytes/sec)
    per_peer_upload_limit: u64,
    
    /// Per-peer download limit (bytes/sec)
    per_peer_download_limit: u64,
    
    /// Burst allowance
    burst_allowance: u64,
    
    /// Priority levels
    priority_weights: HashMap<Priority, f64>,
}

pub struct PeerBandwidthLimiter {
    /// Upload rate limiter
    upload_limiter: RateLimiter,
    
    /// Download rate limiter
    download_limiter: RateLimiter,
    
    /// Usage tracking
    usage: BandwidthUsage,
}

impl BandwidthLimiter {
    /// Check if can send data
    pub async fn can_send(&self, peer_id: &PeerId, size: usize, priority: Priority) -> bool {
        // Check global limits
        if !self.upload_limiter.check_and_consume(size).await {
            return false;
        }
        
        // Check per-peer limits
        if let Some(peer_limiter) = self.peer_limits.get(peer_id) {
            if !peer_limiter.upload_limiter.check_and_consume(size).await {
                // Refund global consumption
                self.upload_limiter.refund(size).await;
                return false;
            }
        }
        
        true
    }
    
    /// Consume bandwidth for sending
    pub async fn consume_send(&self, peer_id: &PeerId, size: usize, priority: Priority) {
        // Update global stats
        self.stats.record_upload(size, priority).await;
        
        // Update per-peer stats
        if let Some(mut peer_limiter) = self.peer_limits.get_mut(peer_id) {
            peer_limiter.usage.upload_bytes += size as u64;
            peer_limiter.usage.last_upload = Instant::now();
        }
    }
    
    /// Adaptive bandwidth allocation
    pub async fn adjust_limits(&self) {
        let current_usage = self.stats.get_current_usage().await;
        let utilization = current_usage.upload_rate / self.config.global_upload_limit as f64;
        
        // Adjust per-peer limits based on overall utilization
        if utilization > 0.8 {
            // High utilization - reduce per-peer limits
            self.reduce_peer_limits(0.8).await;
        } else if utilization < 0.5 {
            // Low utilization - increase per-peer limits
            self.increase_peer_limits(1.2).await;
        }
    }
}
```

## 5. Relay Management

### 5.1 Relay Node Selection

```rust
pub struct RelayManager {
    /// Available relay nodes
    relay_nodes: Arc<RwLock<Vec<RelayNode>>>,
    
    /// Relay selection strategy
    selection_strategy: RelaySelectionStrategy,
    
    /// Relay performance tracker
    performance_tracker: Arc<RelayPerformanceTracker>,
    
    /// Active relay circuits
    active_circuits: Arc<DashMap<CircuitId, RelayCircuit>>,
}

pub struct RelayNode {
    /// Node identifier
    peer_id: PeerId,
    
    /// Node address
    address: NetworkAddress,
    
    /// Node capabilities
    capabilities: RelayCapabilities,
    
    /// Performance metrics
    performance: RelayPerformance,
    
    /// Reputation score
    reputation: f64,
    
    /// Geographic location
    location: Option<GeoLocation>,
}

pub struct RelayCapabilities {
    /// Maximum bandwidth
    max_bandwidth: u64,
    
    /// Supported protocols
    supported_protocols: HashSet<Protocol>,
    
    /// Exit node capability
    is_exit_node: bool,
    
    /// Bridge capability
    is_bridge: bool,
    
    /// Onion routing support
    supports_onion_routing: bool,
}

impl RelayManager {
    /// Select best relay for connection
    pub async fn select_relay(&self, criteria: &RelaySelectionCriteria) -> Result<RelayNode> {
        let relay_nodes = self.relay_nodes.read().await;
        
        // Filter nodes based on criteria
        let candidates: Vec<_> = relay_nodes
            .iter()
            .filter(|node| self.meets_criteria(node, criteria))
            .collect();
        
        if candidates.is_empty() {
            return Err(RelayError::NoSuitableRelay);
        }
        
        // Apply selection strategy
        match self.selection_strategy {
            RelaySelectionStrategy::BestPerformance => {
                self.select_best_performance(&candidates)
            }
            RelaySelectionStrategy::Random => {
                self.select_random(&candidates)
            }
            RelaySelectionStrategy::GeographicDiversity => {
                self.select_geographically_diverse(&candidates, criteria)
            }
            RelaySelectionStrategy::LoadBalanced => {
                self.select_load_balanced(&candidates)
            }
        }
    }
    
    /// Create relay circuit
    pub async fn create_relay_circuit(
        &self,
        source: PeerId,
        destination: PeerId,
        relay: RelayNode,
    ) -> Result<RelayCircuit> {
        // Negotiate with relay
        let circuit_id = CircuitId::generate();
        let relay_params = self.negotiate_relay_params(&relay, &circuit_id).await?;
        
        // Create circuit
        let circuit = RelayCircuit {
            circuit_id,
            source,
            destination,
            relay: relay.peer_id,
            established_at: Instant::now(),
            params: relay_params,
            stats: RelayCircuitStats::default(),
        };
        
        self.active_circuits.insert(circuit_id, circuit.clone());
        
        Ok(circuit)
    }
    
    /// Background task for relay maintenance
    pub async fn maintenance_loop(&self) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Update relay performance metrics
            self.update_relay_performance().await;
            
            // Clean up stale circuits
            self.cleanup_stale_circuits().await;
            
            // Refresh relay list
            self.refresh_relay_list().await;
        }
    }
}
```

## 6. Integration with Network Stack

### 6.1 Connection Manager Integration

```rust
impl NetworkManager {
    /// Enhanced connection establishment with full NAT traversal
    pub async fn enhanced_connect(&self, peer_id: &PeerId) -> Result<ConnectionRef> {
        // Check circuit breaker
        if !self.circuit_breaker.should_allow_request(peer_id).await {
            return Err(NetworkError::CircuitBreakerOpen);
        }
        
        // Try to get existing connection from pool
        if let Ok(conn) = self.connection_pool.get_connection(peer_id).await {
            return Ok(conn);
        }
        
        // Get peer info for NAT traversal
        let peer_info = self.peer_discovery.get_peer_info(peer_id).await?;
        
        // Attempt NAT traversal
        let connection = self.nat_traversal.establish_connection(&peer_info).await?;
        
        // Wrap with bandwidth limiting
        let limited_connection = self.bandwidth_limiter.wrap_connection(connection);
        
        // Add to pool
        self.connection_pool.add_connection(*peer_id, limited_connection).await?;
        
        Ok(self.connection_pool.get_connection(peer_id).await?)
    }
}
```

## 7. Performance Optimizations

### 7.1 Connection Multiplexing

```rust
pub struct ConnectionMultiplexer {
    /// Underlying connection
    connection: Box<dyn NetworkConnection>,
    
    /// Active streams
    streams: Arc<DashMap<StreamId, MultiplexedStream>>,
    
    /// Stream allocation
    next_stream_id: AtomicU32,
    
    /// Frame processor
    frame_processor: Arc<FrameProcessor>,
}

impl ConnectionMultiplexer {
    /// Create new stream
    pub async fn create_stream(&self) -> Result<MultiplexedStream> {
        let stream_id = StreamId(self.next_stream_id.fetch_add(1, Ordering::Relaxed));
        
        let stream = MultiplexedStream::new(stream_id, self.frame_processor.clone());
        self.streams.insert(stream_id, stream.clone());
        
        // Send stream open frame
        let open_frame = Frame::new(stream_id, FrameType::StreamOpen, vec![]);
        self.send_frame(open_frame).await?;
        
        Ok(stream)
    }
    
    /// Process incoming frames
    async fn process_incoming_frames(&self) {
        while let Ok(frame) = self.receive_frame().await {
            if let Some(stream) = self.streams.get(&frame.stream_id) {
                stream.handle_frame(frame).await;
            }
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new_test();
        let peer_id = PeerId::random();
        
        // First connection should create new
        let conn1 = pool.get_connection(&peer_id).await.unwrap();
        assert_eq!(pool.connection_count(), 1);
        
        // Second request should reuse existing
        let conn2 = pool.get_connection(&peer_id).await.unwrap();
        assert_eq!(pool.connection_count(), 1);
    }
    
    #[tokio::test]
    async fn test_nat_traversal() {
        let traversal = NATTraversalManager::new_test();
        let peer_info = create_test_peer_info();
        
        let connection = traversal.establish_connection(&peer_info).await.unwrap();
        assert!(connection.is_connected());
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        
        // Record failures to open circuit
        for _ in 0..5 {
            breaker.record_failure();
        }
        
        assert!(!breaker.should_allow_request());
        assert_eq!(breaker.state, CircuitBreakerState::Open);
    }
}
```

## Conclusion

This connection management system provides:
- Robust NAT traversal with multiple strategies
- Intelligent connection pooling and reuse
- Health monitoring with circuit breaker protection
- Bandwidth management and QoS
- Relay fallback for challenging network conditions
- Integration with the broader QuDAG network stack

The modular design allows for easy testing and adaptation to different network environments while maintaining high reliability and performance.