# QuDAG Node Operations Implementation Plan

## Overview

This document outlines the implementation plan for QuDAG node operations commands, with a primary focus on the `qudag status` command that provides comprehensive node monitoring capabilities.

## 1. Node Status Command Implementation

### 1.1 Architecture Overview

The node status command will follow a modular architecture:

```
CLI Layer (qudag status)
    ↓
RPC Client
    ↓  
RPC Server (Node Process)
    ↓
Status Collector
    ↓
[Consensus] [DAG] [Network] [Metrics]
```

### 1.2 Status Data Structure

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct NodeStatus {
    // Basic Information
    pub node_id: String,
    pub version: String,
    pub uptime: Duration,
    pub start_time: SystemTime,
    
    // Consensus State
    pub consensus_state: ConsensusState,
    
    // DAG Statistics
    pub dag_stats: DagStatistics,
    
    // Network Connectivity
    pub network_info: NetworkInfo,
    
    // Resource Usage
    pub resource_usage: ResourceMetrics,
    
    // Health Status
    pub health: HealthStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConsensusState {
    pub status: ConsensusStatus,
    pub round: u64,
    pub finalized_vertices: u64,
    pub pending_vertices: u64,
    pub voting_power: f64,
    pub last_finalized_time: SystemTime,
    pub average_finality_time: Duration,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DagStatistics {
    pub total_vertices: u64,
    pub total_edges: u64,
    pub tip_count: usize,
    pub depth: u64,
    pub branching_factor: f64,
    pub growth_rate: f64, // vertices per second
    pub storage_size: u64, // bytes
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInfo {
    pub peer_count: usize,
    pub active_connections: usize,
    pub inbound_connections: usize,
    pub outbound_connections: usize,
    pub bandwidth_in: u64, // bytes/sec
    pub bandwidth_out: u64, // bytes/sec
    pub average_latency: Duration,
    pub network_health: NetworkHealth,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceMetrics {
    pub cpu_usage: f64, // percentage
    pub memory_usage: MemoryUsage,
    pub disk_usage: DiskUsage,
    pub thread_count: usize,
    pub open_file_descriptors: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryUsage {
    pub resident: u64, // bytes
    pub virtual_memory: u64, // bytes
    pub heap_allocated: u64, // bytes
    pub heap_used: u64, // bytes
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiskUsage {
    pub database_size: u64, // bytes
    pub log_size: u64, // bytes
    pub available_space: u64, // bytes
    pub iops_read: u64,
    pub iops_write: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HealthStatus {
    Healthy,
    Degraded(Vec<HealthIssue>),
    Unhealthy(Vec<HealthIssue>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthIssue {
    pub component: String,
    pub severity: Severity,
    pub message: String,
    pub since: SystemTime,
}
```

### 1.3 Implementation Phases

#### Phase 1: Basic Status Infrastructure
- Implement `StatusCollector` trait
- Create basic RPC endpoints for status retrieval
- Implement node ID and version reporting

#### Phase 2: Consensus Integration
- Connect to consensus module
- Report consensus round and status
- Track finalized/pending vertices
- Calculate voting power

#### Phase 3: DAG Statistics
- Integrate with DAG module
- Calculate graph metrics (vertices, edges, tips)
- Measure growth rate
- Track storage usage

#### Phase 4: Network Monitoring
- Connect to network layer
- Track peer connections
- Measure bandwidth usage
- Calculate network latency

#### Phase 5: Resource Monitoring
- Implement system resource tracking
- CPU and memory monitoring
- Disk usage tracking
- Thread and file descriptor counting

## 2. Health Check Mechanisms

### 2.1 Component Health Checks

```rust
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> ComponentHealth;
    fn component_name(&self) -> &str;
}

pub struct ComponentHealth {
    pub status: HealthStatus,
    pub last_check: SystemTime,
    pub check_duration: Duration,
    pub details: HashMap<String, String>,
}
```

### 2.2 Health Check Components

1. **Consensus Health**
   - Check if consensus is making progress
   - Verify voting participation
   - Monitor finality times
   - Detect stalled rounds

2. **DAG Health**
   - Verify DAG consistency
   - Check for orphaned vertices
   - Monitor tip selection performance
   - Detect excessive branching

3. **Network Health**
   - Verify minimum peer connections
   - Check message propagation
   - Monitor connection stability
   - Detect network partitions

4. **Storage Health**
   - Check database connectivity
   - Monitor disk space
   - Verify write performance
   - Check for corruption

5. **Resource Health**
   - Monitor memory pressure
   - Check CPU throttling
   - Verify file descriptor limits
   - Monitor thread pool saturation

### 2.3 Health Check Scheduling

```rust
pub struct HealthMonitor {
    checks: Vec<Box<dyn HealthCheck>>,
    interval: Duration,
    results: Arc<RwLock<HashMap<String, ComponentHealth>>>,
}

impl HealthMonitor {
    pub fn new(interval: Duration) -> Self {
        Self {
            checks: Vec::new(),
            interval,
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }
    
    pub async fn start(&self) {
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;
            self.run_checks().await;
        }
    }
    
    async fn run_checks(&self) {
        let futures: Vec<_> = self.checks.iter()
            .map(|check| async move {
                let name = check.component_name().to_string();
                let health = check.check().await;
                (name, health)
            })
            .collect();
        
        let results = futures::future::join_all(futures).await;
        
        let mut health_results = self.results.write().await;
        for (name, health) in results {
            health_results.insert(name, health);
        }
    }
}
```

## 3. Metrics Collection and Reporting

### 3.1 Metrics Framework

```rust
pub struct MetricsCollector {
    registry: Arc<RwLock<MetricsRegistry>>,
    exporters: Vec<Box<dyn MetricsExporter>>,
}

#[async_trait]
pub trait MetricsExporter: Send + Sync {
    async fn export(&self, metrics: &MetricsSnapshot) -> Result<()>;
}

pub struct MetricsRegistry {
    counters: HashMap<String, AtomicU64>,
    gauges: HashMap<String, AtomicI64>,
    histograms: HashMap<String, Histogram>,
    labels: HashMap<String, HashMap<String, String>>,
}
```

### 3.2 Key Metrics

1. **Consensus Metrics**
   - `consensus_round_counter`: Current consensus round
   - `consensus_finalized_vertices`: Total finalized vertices
   - `consensus_finality_time_histogram`: Finality time distribution
   - `consensus_voting_power_gauge`: Current voting power

2. **DAG Metrics**
   - `dag_vertex_counter`: Total vertices in DAG
   - `dag_edge_counter`: Total edges in DAG
   - `dag_tip_gauge`: Current number of tips
   - `dag_depth_gauge`: Current DAG depth
   - `dag_growth_rate_gauge`: Vertices per second

3. **Network Metrics**
   - `network_peer_gauge`: Current peer count
   - `network_messages_sent_counter`: Total messages sent
   - `network_messages_received_counter`: Total messages received
   - `network_bytes_sent_counter`: Total bytes sent
   - `network_bytes_received_counter`: Total bytes received
   - `network_latency_histogram`: Peer latency distribution

4. **Resource Metrics**
   - `process_cpu_usage_gauge`: CPU usage percentage
   - `process_memory_resident_gauge`: Resident memory in bytes
   - `process_thread_count_gauge`: Number of threads
   - `process_open_fds_gauge`: Open file descriptors

### 3.3 Metrics Export Formats

1. **Prometheus Format**
```
# HELP consensus_round_counter Current consensus round
# TYPE consensus_round_counter counter
consensus_round_counter 12345

# HELP dag_vertex_counter Total vertices in DAG
# TYPE dag_vertex_counter counter
dag_vertex_counter{type="finalized"} 98765
dag_vertex_counter{type="pending"} 123
```

2. **JSON Format**
```json
{
  "timestamp": "2024-01-20T10:30:00Z",
  "metrics": {
    "consensus": {
      "round": 12345,
      "finalized_vertices": 98765,
      "pending_vertices": 123
    },
    "dag": {
      "total_vertices": 98888,
      "total_edges": 197776,
      "tips": 5
    }
  }
}
```

## 4. Integration with Consensus and DAG Modules

### 4.1 Consensus Integration

```rust
impl Node {
    pub async fn get_consensus_status(&self) -> ConsensusState {
        let consensus = self.consensus.read().await;
        
        ConsensusState {
            status: consensus.status(),
            round: consensus.current_round(),
            finalized_vertices: consensus.finalized_count(),
            pending_vertices: consensus.pending_count(),
            voting_power: consensus.voting_power(),
            last_finalized_time: consensus.last_finalized_time(),
            average_finality_time: consensus.average_finality_time(),
        }
    }
}
```

### 4.2 DAG Integration

```rust
impl Node {
    pub async fn get_dag_statistics(&self) -> DagStatistics {
        let dag = self.dag.read().await;
        let vertices = dag.vertices.read().await;
        
        DagStatistics {
            total_vertices: vertices.len() as u64,
            total_edges: dag.edge_count(),
            tip_count: dag.get_tips().await.len(),
            depth: dag.depth(),
            branching_factor: dag.calculate_branching_factor(),
            growth_rate: dag.growth_rate(),
            storage_size: dag.storage_size(),
        }
    }
}
```

### 4.3 Network Integration

```rust
impl Node {
    pub async fn get_network_info(&self) -> NetworkInfo {
        let network = self.network.read().await;
        
        NetworkInfo {
            peer_count: network.peer_count(),
            active_connections: network.active_connections(),
            inbound_connections: network.inbound_connections(),
            outbound_connections: network.outbound_connections(),
            bandwidth_in: network.bandwidth_in(),
            bandwidth_out: network.bandwidth_out(),
            average_latency: network.average_latency(),
            network_health: network.health_status(),
        }
    }
}
```

## 5. Real-time Status Updates

### 5.1 WebSocket Server

```rust
pub struct StatusWebSocketServer {
    node: Arc<Node>,
    update_interval: Duration,
}

impl StatusWebSocketServer {
    pub async fn handle_connection(&self, ws: WebSocket) {
        let (tx, rx) = ws.split();
        let tx = Arc::new(Mutex::new(tx));
        
        // Start update loop
        let update_task = self.start_updates(tx.clone());
        
        // Handle incoming messages
        let message_task = self.handle_messages(rx);
        
        tokio::select! {
            _ = update_task => {},
            _ = message_task => {},
        }
    }
    
    async fn start_updates(&self, tx: Arc<Mutex<SplitSink<WebSocket, Message>>>) {
        let mut interval = tokio::time::interval(self.update_interval);
        
        loop {
            interval.tick().await;
            
            let status = self.node.get_status().await;
            let message = serde_json::to_string(&status).unwrap();
            
            let mut tx = tx.lock().await;
            if tx.send(Message::text(message)).await.is_err() {
                break;
            }
        }
    }
}
```

### 5.2 Server-Sent Events (SSE)

```rust
pub async fn status_stream(node: Arc<Node>) -> impl Stream<Item = Result<Event, Infallible>> {
    let interval = Duration::from_secs(1);
    let mut interval = tokio::time::interval(interval);
    
    stream! {
        loop {
            interval.tick().await;
            
            let status = node.get_status().await;
            let data = serde_json::to_string(&status).unwrap();
            
            yield Ok(Event::default().data(data));
        }
    }
}
```

### 5.3 Status Change Notifications

```rust
pub enum StatusChangeEvent {
    ConsensusStateChanged(ConsensusStatus),
    NetworkPeerCountChanged(usize),
    HealthStatusChanged(HealthStatus),
    ResourceThresholdExceeded(ResourceAlert),
}

pub struct StatusNotifier {
    subscribers: Arc<RwLock<Vec<mpsc::Sender<StatusChangeEvent>>>>,
}

impl StatusNotifier {
    pub async fn subscribe(&self) -> mpsc::Receiver<StatusChangeEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers.write().await.push(tx);
        rx
    }
    
    pub async fn notify(&self, event: StatusChangeEvent) {
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            let _ = subscriber.send(event.clone()).await;
        }
    }
}
```

## 6. Test Scenarios and Edge Cases

### 6.1 Unit Tests

1. **Status Collection Tests**
   - Test each status component individually
   - Verify correct data aggregation
   - Test error handling in collectors

2. **Health Check Tests**
   - Test health check thresholds
   - Verify health state transitions
   - Test component failure detection

3. **Metrics Tests**
   - Test metric recording accuracy
   - Verify histogram percentiles
   - Test counter atomicity

### 6.2 Integration Tests

1. **Full Status Retrieval**
   ```rust
   #[tokio::test]
   async fn test_full_status_retrieval() {
       let node = create_test_node().await;
       node.start().await;
       
       // Let node run for a bit
       tokio::time::sleep(Duration::from_secs(2)).await;
       
       let status = node.get_status().await;
       
       assert!(!status.node_id.is_empty());
       assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
       assert!(status.uptime.as_secs() >= 2);
       assert_matches!(status.health, HealthStatus::Healthy);
   }
   ```

2. **Consensus Integration**
   ```rust
   #[tokio::test]
   async fn test_consensus_status_updates() {
       let node = create_test_node().await;
       node.start().await;
       
       // Submit some vertices
       for i in 0..10 {
           node.submit_vertex(create_test_vertex(i)).await;
       }
       
       // Wait for consensus
       tokio::time::sleep(Duration::from_secs(1)).await;
       
       let status = node.get_status().await;
       assert!(status.consensus_state.finalized_vertices > 0);
       assert!(status.dag_stats.total_vertices >= 10);
   }
   ```

3. **Network Status**
   ```rust
   #[tokio::test]
   async fn test_network_peer_tracking() {
       let node1 = create_test_node().await;
       let node2 = create_test_node().await;
       
       node1.start().await;
       node2.start().await;
       
       // Connect nodes
       node2.connect_to_peer(node1.address()).await;
       
       tokio::time::sleep(Duration::from_secs(1)).await;
       
       let status1 = node1.get_status().await;
       let status2 = node2.get_status().await;
       
       assert_eq!(status1.network_info.peer_count, 1);
       assert_eq!(status2.network_info.peer_count, 1);
   }
   ```

### 6.3 Edge Cases

1. **Resource Exhaustion**
   - Test behavior under memory pressure
   - Test with maxed out file descriptors
   - Test with full disk

2. **Network Partitions**
   - Test status reporting during network splits
   - Verify health detection of isolated nodes
   - Test recovery after partition heals

3. **Consensus Stalls**
   - Test detection of consensus deadlock
   - Verify health status degradation
   - Test automatic recovery mechanisms

4. **Data Corruption**
   - Test detection of corrupted DAG data
   - Verify health check failures
   - Test graceful degradation

### 6.4 Performance Tests

1. **Status Query Performance**
   ```rust
   #[tokio::test]
   async fn test_status_query_performance() {
       let node = create_large_test_node().await; // Node with lots of data
       
       let start = Instant::now();
       let status = node.get_status().await;
       let duration = start.elapsed();
       
       assert!(duration < Duration::from_millis(100));
   }
   ```

2. **Concurrent Status Queries**
   ```rust
   #[tokio::test]
   async fn test_concurrent_status_queries() {
       let node = Arc::new(create_test_node().await);
       
       let handles: Vec<_> = (0..100)
           .map(|_| {
               let node = node.clone();
               tokio::spawn(async move {
                   node.get_status().await
               })
           })
           .collect();
       
       let results = futures::future::join_all(handles).await;
       assert!(results.iter().all(|r| r.is_ok()));
   }
   ```

3. **Real-time Update Performance**
   ```rust
   #[tokio::test]
   async fn test_websocket_update_rate() {
       let node = create_test_node().await;
       let ws_server = StatusWebSocketServer::new(node, Duration::from_millis(100));
       
       // Connect client and measure update rate
       let updates_received = Arc::new(AtomicU64::new(0));
       // ... WebSocket client setup ...
       
       tokio::time::sleep(Duration::from_secs(10)).await;
       
       let count = updates_received.load(Ordering::Relaxed);
       assert!(count >= 90 && count <= 110); // Allow some variance
   }
   ```

## 7. CLI Output Formatting

### 7.1 Default Format

```
QuDAG Node Status
═════════════════════════════════════════════════════════════════

Node Information:
  ID:           7f3a9b2c-d1e4-4a6f-8c5d-2e1f0a9b8c7d
  Version:      0.1.0
  Uptime:       2 days, 14:32:15
  Started:      2024-01-18 10:30:45 UTC

Consensus State:
  Status:       Active
  Round:        12,345
  Finalized:    98,765 vertices
  Pending:      123 vertices
  Voting Power: 0.85
  Avg Finality: 845ms

DAG Statistics:
  Vertices:     98,888
  Edges:        197,776
  Tips:         5
  Depth:        1,234
  Growth Rate:  12.5 vertices/sec
  Storage:      1.2 GB

Network:
  Peers:        42 (15 in, 27 out)
  Bandwidth:    ↓ 125 KB/s  ↑ 89 KB/s
  Avg Latency:  23ms
  Health:       Healthy

Resources:
  CPU:          12.5%
  Memory:       256 MB / 8 GB (3.1%)
  Disk:         1.5 GB / 100 GB (1.5%)
  Threads:      24
  FDs:          156 / 1024

Health:       ✓ Healthy
```

### 7.2 JSON Format

```bash
qudag status --json
```

### 7.3 Compact Format

```bash
qudag status --compact
```
```
Node: 7f3a9b2c | v0.1.0 | Up: 2d 14h | Consensus: Active/12345 | DAG: 98888v/5t | Net: 42p/23ms | CPU: 12.5% | Mem: 256MB | ✓ Healthy
```

### 7.4 Watch Mode

```bash
qudag status --watch
```

Updates the display every second with color-coded changes.

## 8. Error Handling

### 8.1 Connection Errors

```rust
pub enum StatusError {
    NodeNotRunning,
    RpcConnectionFailed(String),
    RpcTimeout,
    SerializationError(String),
    InternalError(String),
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusError::NodeNotRunning => {
                write!(f, "Node is not running. Start with 'qudag start'")
            }
            StatusError::RpcConnectionFailed(addr) => {
                write!(f, "Failed to connect to node at {}. Is the node running?", addr)
            }
            StatusError::RpcTimeout => {
                write!(f, "Request timed out. Node may be unresponsive")
            }
            StatusError::SerializationError(e) => {
                write!(f, "Failed to parse response: {}", e)
            }
            StatusError::InternalError(e) => {
                write!(f, "Internal error: {}", e)
            }
        }
    }
}
```

### 8.2 Graceful Degradation

When certain components fail, the status command should still return partial information:

```rust
impl NodeStatus {
    pub fn with_partial_data() -> Self {
        Self {
            node_id: "unknown".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            consensus_state: ConsensusState::unavailable(),
            dag_stats: DagStatistics::unavailable(),
            network_info: NetworkInfo::unavailable(),
            resource_usage: ResourceMetrics::current_process_only(),
            health: HealthStatus::Degraded(vec![
                HealthIssue {
                    component: "consensus".to_string(),
                    severity: Severity::Warning,
                    message: "Unable to retrieve consensus state".to_string(),
                    since: SystemTime::now(),
                }
            ]),
            ..Default::default()
        }
    }
}
```

## 9. Future Enhancements

### 9.1 Historical Data

- Store status snapshots for trend analysis
- Provide status history queries
- Generate performance reports

### 9.2 Alerting

- Configure thresholds for alerts
- Send notifications on status changes
- Integrate with monitoring systems

### 9.3 Diagnostics

- Automatic problem detection
- Suggested remediation steps
- Performance optimization hints

### 9.4 Multi-node Support

- Query status of remote nodes
- Aggregate cluster statistics
- Network-wide health monitoring

## 10. Implementation Timeline

### Week 1: Foundation
- Implement basic status data structures
- Create RPC endpoints
- Basic CLI integration

### Week 2: Component Integration
- Integrate consensus module
- Integrate DAG module
- Integrate network module

### Week 3: Health & Metrics
- Implement health checks
- Add metrics collection
- Resource monitoring

### Week 4: Real-time & Polish
- WebSocket/SSE support
- Output formatting
- Error handling
- Documentation

### Week 5: Testing & Optimization
- Comprehensive test suite
- Performance optimization
- Edge case handling
- Integration testing

## Conclusion

This implementation plan provides a comprehensive approach to building the QuDAG node operations commands, with a focus on the `qudag status` command. The modular design allows for incremental development while ensuring that each component is thoroughly tested and integrated properly. The real-time update capabilities and comprehensive health checking ensure that operators have full visibility into node operations.