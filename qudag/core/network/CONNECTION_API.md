# QuDAG Network Connection Management API

## ConnectionManager

The `ConnectionManager` provides high-performance connection management with built-in connection pooling, status tracking, and performance monitoring.

### Public API

```rust
pub struct ConnectionManager {
    pub fn new(max_connections: usize) -> Self
    pub fn with_pool_timeout(max_connections: usize, pool_timeout: Duration) -> Self
    pub async fn connect(&self, peer_id: PeerId) -> Result<()>
    pub async fn disconnect(&self, peer_id: &PeerId)
    pub async fn update_status(&self, peer_id: PeerId, status: ConnectionStatus)
    pub async fn get_status(&self, peer_id: &PeerId) -> Option<ConnectionStatus>
    pub async fn get_metrics(&self) -> NetworkMetrics
}
```

### Usage Patterns

1. Basic Connection Management:
```rust
let manager = ConnectionManager::new(100); // Support up to 100 concurrent connections
await manager.connect(peer_id)?;
```

2. Custom Pool Configuration:
```rust
let manager = ConnectionManager::with_pool_timeout(
    100,                                    // Max connections 
    Duration::from_secs(300)               // 5 minute pool timeout
);
```

3. Status Tracking:
```rust 
await manager.update_status(peer_id, ConnectionStatus::Connected);
if let Some(status) = await manager.get_status(&peer_id) {
    // Handle connection status
}
```

### Connection Status Tracking

The `ConnectionStatus` enum provides comprehensive status tracking:

```rust
pub enum ConnectionStatus {
    Connected,    // Connection is active
    Connecting,   // Connection being established
    Disconnected, // Connection was lost
    Failed,       // Connection failed
}
```

All status changes are atomic and thread-safe. The manager maintains real-time status for all connections.

### Connection Pool Management

The connection pool provides efficient connection reuse with:

- Automatic pooling of idle connections
- Configurable TTL for pooled connections
- Automatic cleanup of expired connections 
- Thread-safe concurrent access
- Memory-efficient storage

### Performance Metrics

Real-time metrics tracking via `NetworkMetrics`:

```rust
pub struct NetworkMetrics {
    pub messages_per_second: f64,    // Message throughput
    pub connections: usize,          // Active connection count
    pub avg_latency: Duration,       // Average message latency
    pub memory_usage: usize,         // Memory utilization
}
```

### Implementation Details

1. Thread Safety
- Uses `dashmap` for concurrent connection storage
- Atomic status updates
- Lock-free metrics tracking
- Thread-safe connection pooling

2. Performance Optimizations  
- Connection pooling reduces setup overhead
- Batched status updates
- Efficient concurrent data structures
- Minimized lock contention

3. Resource Management
- Automatic cleanup of expired connections
- Configurable connection limits
- Memory-efficient storage
- Proper resource cleanup

### Usage Recommendations

1. Connection Pooling
- Configure pool timeout based on typical connection patterns
- Monitor pool utilization via metrics
- Adjust pool size based on usage patterns

2. Status Tracking  
- Subscribe to status changes for important peers
- Handle disconnections gracefully
- Implement reconnection logic as needed

3. Performance Monitoring
- Monitor metrics regularly 
- Set up alerts for anomalies
- Track connection pool efficiency

4. Resource Management
- Set appropriate connection limits
- Configure reasonable pool timeouts
- Implement proper cleanup