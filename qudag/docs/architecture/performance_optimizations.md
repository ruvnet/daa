# QuDAG Performance Optimizations

This document describes the performance optimizations integrated into the QuDAG protocol, their configuration, and usage.

## Overview

QuDAG includes several performance optimizations across three main modules:
- **Network**: Message chunking and adaptive batching
- **DAG**: Validation caching and traversal indexing  
- **Swarm**: Asynchronous coordination

## Network Optimizations

### Message Chunking

Large messages are automatically split into smaller chunks for efficient transmission and reassembly.

**Features:**
- Automatic chunking for messages > 64KB
- Compression support using zstd
- Concurrent chunk transmission
- Out-of-order reassembly
- Integrity verification via blake3 hashing

**Configuration:**
```toml
[network.message_chunking]
enabled = true
max_chunk_size = 65536  # 64KB
max_chunks = 10000
chunk_timeout = 30  # seconds
enable_compression = true
compression_threshold = 1024  # bytes
compression_level = 3
cache_size = 1000
```

**Usage:**
```rust
use qudag_network::optimized::{MessageChunker, ChunkerConfig};

let config = ChunkerConfig {
    max_chunk_size: 65536,
    enable_compression: true,
    ..Default::default()
};
let chunker = MessageChunker::new(config);

// Chunk a large message
let chunks = chunker.chunk_message(&network_message).await?;

// Process incoming chunks
if let Some(reassembled) = chunker.process_chunk(chunk).await? {
    // Message fully reassembled
}
```

### Adaptive Batching

Messages are intelligently batched to optimize network throughput.

**Configuration:**
```toml
[network.adaptive_batching]
enabled = true
max_batch_size = 100
batch_timeout = 50  # milliseconds
algorithm = "exponential_backoff"
```

## DAG Optimizations

### Validation Cache

Vertex validation results are cached to avoid redundant cryptographic operations.

**Features:**
- LRU cache with configurable size
- TTL-based expiration
- Parallel batch validation
- Hot cache for frequently accessed vertices
- Bloom filter for quick negative lookups

**Configuration:**
```toml
[dag.validation_cache]
enabled = true
max_entries = 100000
ttl = 3600  # 1 hour
enable_batch_validation = true
batch_size = 100
cache_parent_validation = true
```

**Usage:**
```rust
use qudag_dag::optimized::{ValidationCache, ValidationResult};

let cache = ValidationCache::new(Default::default());

// Validate with caching
let result = cache.validate(&vertex)?;
if result.is_valid {
    // Vertex is valid
}

// Batch validation
let results = cache.batch_validate(&vertices);
```

### Traversal Index

Accelerates DAG traversal operations with pre-computed indexes.

**Features:**
- Ancestor/descendant indexing
- Depth tracking
- Common ancestor caching
- Path finding with caching
- Graph algorithm support via petgraph

**Configuration:**
```toml
[dag.traversal_index]
enabled = true
common_ancestor_cache_size = 10000
path_cache_size = 1000
enable_graph_algorithms = true
```

**Usage:**
```rust
use qudag_dag::optimized::{TraversalIndex, IndexedDAG};

let index = TraversalIndex::new();
index.add_vertex(&vertex);

// Fast ancestor queries
let ancestors = index.get_ancestors(&vertex_id)?;

// Find common ancestor
let common = index.find_common_ancestor(&id1, &id2)?;

// Shortest path
let path = index.find_path(&from, &to)?;
```

## Swarm Optimizations

### Async Coordination

Hierarchical swarm coordination with work stealing and load balancing.

**Features:**
- Hierarchical agent organization
- Multiple distribution strategies
- Work stealing between agents
- Async task execution
- Real-time monitoring

**Configuration:**
```toml
[swarm.async_coordination]
enabled = true
max_agents_per_coordinator = 10
max_hierarchy_depth = 3
communication_timeout = 5  # seconds
distribution_strategy = "load_balanced"
enable_work_stealing = true
heartbeat_interval = 10  # seconds
```

**Usage:**
```rust
use qudag_swarm::{HierarchicalSwarm, SwarmConfig, Task, TaskPriority};

let config = SwarmConfig::default();
let swarm = HierarchicalSwarm::new(config);

// Add agents
swarm.add_agent(agent).await?;

// Submit tasks
let task = Task {
    id: "task_1".to_string(),
    payload: data,
    priority: TaskPriority::Normal,
    timeout: Duration::from_secs(30),
};
swarm.submit_task(task).await?;

// Get statistics
let stats = swarm.get_stats();
```

## Configuration Management

### Loading Configuration

```rust
use qudag_protocol::OptimizationConfig;

// Load from file
let config = OptimizationConfig::from_file("config/optimizations.toml")?;

// Use defaults
let config = OptimizationConfig::default();

// Check if optimization is enabled
if config.is_enabled("message_chunking") {
    // Use message chunking
}
```

### Environment Variables

All settings can be overridden via environment variables:

```bash
# Global
export QUDAG_ENABLE_OPTIMIZATIONS=true

# Network
export QUDAG_NETWORK_MESSAGE_CHUNKING_ENABLED=true
export QUDAG_NETWORK_MESSAGE_CHUNKING_MAX_CHUNK_SIZE=131072

# DAG
export QUDAG_DAG_VALIDATION_CACHE_ENABLED=true
export QUDAG_DAG_VALIDATION_CACHE_MAX_ENTRIES=200000

# Swarm
export QUDAG_SWARM_ASYNC_COORDINATION_ENABLED=true
export QUDAG_SWARM_ASYNC_COORDINATION_MAX_AGENTS=20
```

## Feature Flags

Optimizations can be controlled at compile time using Cargo features:

```toml
[dependencies]
qudag-network = { version = "0.1", features = ["message-chunking"] }
qudag-dag = { version = "0.1", features = ["validation-cache", "traversal-index"] }
qudag-swarm = { version = "0.1", features = ["async-coordination"] }
```

Or enable all optimizations:

```toml
[dependencies]
qudag-network = { version = "0.1", features = ["full-optimizations"] }
qudag-dag = { version = "0.1", features = ["full-optimizations"] }
qudag-swarm = { version = "0.1", features = ["full-optimizations"] }
```

## Performance Metrics

Monitor optimization effectiveness:

```rust
// Message chunking stats
let stats = chunker.get_stats();
println!("Active reassemblies: {}", stats.active_reassemblies);
println!("Cache size: {}", stats.cache_size);

// Validation cache stats
let stats = cache.get_stats();
println!("Cache hits: {}", stats.cache_hits);
println!("Cache misses: {}", stats.cache_misses);
println!("Hit rate: {:.2}%", (stats.cache_hits as f64 / stats.total_validations as f64) * 100.0);

// Swarm stats
let stats = swarm.get_stats();
println!("Active agents: {}", stats.active_agents);
println!("Average execution time: {}μs", stats.avg_execution_time_us);
```

## Best Practices

1. **Message Chunking**
   - Enable compression for messages > 1KB
   - Adjust chunk size based on network conditions
   - Monitor reassembly timeouts

2. **Validation Cache**
   - Size cache based on active vertex count
   - Use batch validation for bulk operations
   - Clear cache periodically to free memory

3. **Traversal Index**
   - Rebuild index after major DAG reorganizations
   - Use cached operations for hot paths
   - Monitor cache hit rates

4. **Swarm Coordination**
   - Balance hierarchy depth vs coordinator load
   - Enable work stealing for uneven workloads
   - Monitor agent health via heartbeats

## Troubleshooting

### High Memory Usage

If memory usage is too high:
- Reduce cache sizes in configuration
- Decrease TTL values
- Enable more aggressive garbage collection

### Poor Cache Performance

If cache hit rates are low:
- Increase cache size limits
- Adjust TTL based on usage patterns
- Enable parent validation caching

### Network Congestion

If experiencing network issues:
- Reduce max chunk size
- Increase batch timeouts
- Adjust compression thresholds

## Future Optimizations

Planned optimizations include:
- Zero-copy networking
- NUMA-aware memory allocation
- Lock-free data structures
- SIMD acceleration for crypto operations
- GPU acceleration for batch validation