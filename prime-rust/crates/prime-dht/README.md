# DAA Prime DHT

[![Crates.io](https://img.shields.io/crates/v/daa-prime-dht.svg)](https://crates.io/crates/daa-prime-dht)
[![Documentation](https://docs.rs/daa-prime-dht/badge.svg)](https://docs.rs/daa-prime-dht)
[![License](https://img.shields.io/crates/l/daa-prime-dht.svg)](https://github.com/yourusername/daa/blob/main/LICENSE)

High-performance Kademlia-based distributed hash table implementation for the Prime distributed machine learning framework. Provides decentralized storage and discovery for ML models, gradients, and metadata.

## Overview

DAA Prime DHT implements a robust distributed hash table based on the Kademlia protocol, specifically optimized for machine learning workloads. It provides:

- **Distributed Storage**: Scalable key-value storage across network nodes
- **Efficient Routing**: Kademlia routing table for O(log N) lookups  
- **Fault Tolerance**: Built-in replication and failure recovery
- **ML Optimization**: Specialized for storing ML models and gradients
- **libp2p Integration**: Seamless networking with modern P2P stack

## Features

- 🔄 **Kademlia Protocol**: Industry-standard DHT with proven scalability
- 🚀 **High Performance**: Optimized for ML data patterns and sizes
- 🛡️ **Fault Tolerant**: Configurable replication and self-healing
- 🔍 **Efficient Discovery**: Fast peer and content discovery
- 📊 **Property Testing**: Comprehensive test coverage with QuickCheck/PropTest
- 🌐 **Network Agnostic**: Works with any libp2p transport

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
daa-prime-dht = "0.2.1"
daa-prime-core = "0.2.1"
libp2p = "0.53"
```

## Quick Start

### Basic DHT Operations

```rust
use daa_prime_dht::{Dht, DhtConfig};
use libp2p::PeerId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DHT with default configuration
    let peer_id = PeerId::random();
    let config = DhtConfig::default();
    let dht = Dht::new(peer_id, config);
    
    // Store a value
    let key = b"model_weights_v1".to_vec();
    let value = b"serialized_model_data".to_vec();
    dht.put(key.clone(), value.clone()).await?;
    
    // Retrieve the value
    if let Some(retrieved) = dht.get(key).await? {
        println!("Retrieved: {:?}", String::from_utf8_lossy(&retrieved));
    }
    
    Ok(())
}
```

### Custom Configuration

```rust
use daa_prime_dht::DhtConfig;
use std::time::Duration;

let config = DhtConfig {
    k_bucket_size: 20,           // Nodes per bucket
    alpha: 3,                    // Parallel query parameter
    replication_factor: 3,       // Number of replicas
    refresh_interval: Duration::from_secs(3600), // Bucket refresh
    ttl: Duration::from_secs(86400),             // Data expiration
};
```

## Core Concepts

### Kademlia Routing

The DHT uses Kademlia's XOR metric for routing:

```rust
use daa_prime_dht::routing::RoutingTable;
use libp2p::PeerId;

let peer_id = PeerId::random();
let routing_table = RoutingTable::new(peer_id, 20);

// Find closest nodes to a target
let target = PeerId::random();
let closest_nodes = routing_table.find_closest(&target, 3);
```

### Storage Management

```rust
use daa_prime_dht::storage::Storage;

let mut storage = Storage::new();

// Store with automatic expiration
storage.put(b"key".to_vec(), b"value".to_vec());

// Retrieve if still valid
if let Some(value) = storage.get(&b"key".to_vec()) {
    println!("Found: {:?}", value);
}
```

### Peer Discovery

```rust
use daa_prime_dht::discovery::Discovery;

let discovery = Discovery::new();

// Start discovery process
discovery.start_discovery().await?;

// Get discovered peers
let peers = discovery.get_peers().await;
```

## Advanced Usage

### ML Model Storage

```rust
use daa_prime_dht::Dht;
use daa_prime_core::ModelMetadata;

// Store ML model with metadata
async fn store_model(
    dht: &Dht,
    model_id: &str,
    model_data: Vec<u8>,
    metadata: ModelMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    // Store model data
    let model_key = format!("model:{}", model_id).into_bytes();
    dht.put(model_key, model_data).await?;
    
    // Store metadata separately
    let meta_key = format!("meta:{}", model_id).into_bytes();
    let meta_data = serde_json::to_vec(&metadata)?;
    dht.put(meta_key, meta_data).await?;
    
    Ok(())
}

// Retrieve model with metadata
async fn load_model(
    dht: &Dht,
    model_id: &str,
) -> Result<Option<(Vec<u8>, ModelMetadata)>, Box<dyn std::error::Error>> {
    let model_key = format!("model:{}", model_id).into_bytes();
    let meta_key = format!("meta:{}", model_id).into_bytes();
    
    let model_data = dht.get(model_key).await?;
    let meta_data = dht.get(meta_key).await?;
    
    match (model_data, meta_data) {
        (Some(model), Some(meta)) => {
            let metadata: ModelMetadata = serde_json::from_slice(&meta)?;
            Ok(Some((model, metadata)))
        },
        _ => Ok(None),
    }
}
```

### Gradient Aggregation

```rust
use daa_prime_dht::Dht;
use daa_prime_core::{GradientUpdate, NodeId};

// Store gradient updates from multiple nodes
async fn store_gradients(
    dht: &Dht,
    round: u64,
    updates: Vec<GradientUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    for update in updates {
        let key = format!("gradient:{}:{}", round, update.node_id.0).into_bytes();
        let data = serde_json::to_vec(&update)?;
        dht.put(key, data).await?;
    }
    Ok(())
}

// Retrieve all gradients for a training round
async fn collect_gradients(
    dht: &Dht,
    round: u64,
    node_ids: Vec<NodeId>,
) -> Result<Vec<GradientUpdate>, Box<dyn std::error::Error>> {
    let mut gradients = Vec::new();
    
    for node_id in node_ids {
        let key = format!("gradient:{}:{}", round, node_id.0).into_bytes();
        if let Some(data) = dht.get(key).await? {
            let update: GradientUpdate = serde_json::from_slice(&data)?;
            gradients.push(update);
        }
    }
    
    Ok(gradients)
}
```

### Custom Storage Backends

```rust
use daa_prime_dht::storage::{Storage, StorageBackend};
use async_trait::async_trait;

// Implement custom storage backend
struct RedisStorage {
    client: redis::Client,
}

#[async_trait]
impl StorageBackend for RedisStorage {
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        // Store in Redis
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
    
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let mut conn = self.client.get_async_connection().await?;
        let result: Option<Vec<u8>> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }
}
```

## Configuration Options

### DHT Parameters

```rust
use daa_prime_dht::DhtConfig;
use std::time::Duration;

let config = DhtConfig {
    // Routing table parameters
    k_bucket_size: 20,           // Maximum nodes per k-bucket
    alpha: 3,                    // Concurrency parameter for lookups
    
    // Storage parameters  
    replication_factor: 3,       // Number of nodes to store each value
    ttl: Duration::from_secs(86400), // Time-to-live for stored values
    
    // Maintenance parameters
    refresh_interval: Duration::from_secs(3600), // Bucket refresh frequency
};
```

### Performance Tuning

```rust
// High-performance configuration for large networks
let high_perf_config = DhtConfig {
    k_bucket_size: 32,           // Larger buckets for better connectivity
    alpha: 8,                    // Higher concurrency for faster lookups
    replication_factor: 5,       // More replicas for reliability
    refresh_interval: Duration::from_secs(1800), // More frequent refresh
    ttl: Duration::from_secs(172800), // Longer TTL for ML models
};

// Low-latency configuration for small networks
let low_latency_config = DhtConfig {
    k_bucket_size: 10,           // Smaller buckets for faster updates
    alpha: 2,                    // Lower concurrency to reduce overhead
    replication_factor: 2,       // Fewer replicas for speed
    refresh_interval: Duration::from_secs(7200), // Less frequent refresh
    ttl: Duration::from_secs(43200), // Shorter TTL for rapid iteration
};
```

## Testing

The crate includes comprehensive property-based testing:

```rust
use daa_prime_dht::{Dht, DhtConfig};
use proptest::prelude::*;
use quickcheck_macros::quickcheck;

proptest! {
    #[test]
    fn test_dht_config_validation(
        k_bucket in 5usize..100usize,
        alpha in 1usize..10usize,
        replication in 1usize..20usize,
    ) {
        let config = DhtConfig {
            k_bucket_size: k_bucket,
            alpha,
            replication_factor: replication,
            refresh_interval: Duration::from_secs(3600),
            ttl: Duration::from_secs(86400),
        };
        
        // Validate configuration constraints
        assert!(config.alpha <= config.k_bucket_size);
        assert!(config.replication_factor >= 1);
    }
}

#[quickcheck]
async fn test_put_get_consistency(key: Vec<u8>, value: Vec<u8>) -> bool {
    if key.is_empty() || value.is_empty() {
        return true;
    }
    
    let peer_id = PeerId::random();
    let dht = Dht::new(peer_id, DhtConfig::default());
    
    // Put should succeed and get should return same value
    dht.put(key.clone(), value.clone()).await.is_ok() &&
    dht.get(key).await.unwrap() == Some(value)
}
```

## Performance Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use daa_prime_dht::{Dht, DhtConfig};
    
    fn bench_put_operations(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let peer_id = PeerId::random();
        let dht = Dht::new(peer_id, DhtConfig::default());
        
        c.bench_function("dht_put_1kb", |b| {
            b.to_async(&rt).iter(|| async {
                let key = black_box(format!("key_{}", fastrand::u64(..)).into_bytes());
                let value = black_box(vec![0u8; 1024]);
                dht.put(key, value).await.unwrap();
            });
        });
    }
    
    criterion_group!(benches, bench_put_operations);
    criterion_main!(benches);
}
```

## Monitoring and Metrics

```rust
use daa_prime_dht::Dht;

// Monitor DHT health
async fn monitor_dht_health(dht: &Dht) {
    let routing_table = dht.routing_table.read().await;
    let storage = dht.storage.read().await;
    
    println!("DHT Health Report:");
    println!("  Active peers: {}", routing_table.len());
    println!("  Stored keys: {}", storage.len());
    println!("  Bucket distribution: {:?}", routing_table.bucket_sizes());
}
```

## Integration Examples

### With DAA Orchestrator

```rust
use daa_prime_dht::Dht;
// use daa_orchestrator::Orchestrator;

async fn setup_orchestrated_dht() -> Result<(), Box<dyn std::error::Error>> {
    let peer_id = PeerId::random();
    let dht = Dht::new(peer_id, DhtConfig::default());
    
    // Register DHT with orchestrator
    // let orchestrator = Orchestrator::new().await?;
    // orchestrator.register_dht(dht).await?;
    
    Ok(())
}
```

### With Prime Trainer

```rust
use daa_prime_dht::Dht;
use daa_prime_trainer::TrainerNode;

async fn trainer_with_dht() -> Result<(), Box<dyn std::error::Error>> {
    let peer_id = PeerId::random();
    let dht = Dht::new(peer_id, DhtConfig::default());
    
    // Trainer can use DHT for model sharing
    let trainer = TrainerNode::new("trainer-001".to_string()).await?;
    
    // Store training checkpoints in DHT
    let checkpoint_key = b"checkpoint_epoch_10".to_vec();
    let checkpoint_data = b"model_state_dict".to_vec();
    dht.put(checkpoint_key, checkpoint_data).await?;
    
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **Connection Failures**
   ```rust
   // Ensure proper libp2p transport configuration
   use libp2p::{tcp, noise, yamux, Transport};
   
   let transport = tcp::tokio::Transport::default()
       .upgrade(noise::Config::new(&keypair)?)
       .multiplex(yamux::Config::default())
       .boxed();
   ```

2. **High Memory Usage**
   ```rust
   // Implement periodic cleanup
   async fn cleanup_expired_data(dht: &Dht) {
       let mut storage = dht.storage.write().await;
       storage.cleanup_expired();
   }
   ```

3. **Slow Lookups**
   ```rust
   // Increase alpha parameter for higher concurrency
   let config = DhtConfig {
       alpha: 8, // Higher concurrency
       ..Default::default()
   };
   ```

## Roadmap

- [ ] IPFS compatibility layer
- [ ] Advanced replication strategies
- [ ] Built-in data compression
- [ ] Metrics and observability
- [ ] Byzantine fault tolerance
- [ ] Integration with DAA consensus

## Contributing

Contributions are welcome! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Related Crates

- [`daa-prime-core`](../prime-core): Core types and protocol definitions
- [`daa-prime-trainer`](../prime-trainer): Training node implementation
- [`daa-prime-coordinator`](../prime-coordinator): Coordination and governance  
- [`daa-prime-cli`](../prime-cli): Command-line interface

## References

- [Kademlia Paper](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf) - Original Kademlia protocol
- [libp2p Specification](https://github.com/libp2p/specs) - Modern P2P networking
- [DHT Security](https://docs.libp2p.io/concepts/security/) - Security considerations

## Support

- 📖 [Documentation](https://docs.rs/daa-prime-dht)  
- 🐛 [Issue Tracker](https://github.com/yourusername/daa/issues)
- 💬 [Discussions](https://github.com/yourusername/daa/discussions)