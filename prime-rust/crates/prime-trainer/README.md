# DAA Prime Trainer

[![Crates.io](https://img.shields.io/crates/v/daa-prime-trainer.svg)](https://crates.io/crates/daa-prime-trainer)
[![Documentation](https://docs.rs/daa-prime-trainer/badge.svg)](https://docs.rs/daa-prime-trainer)
[![License](https://img.shields.io/crates/l/daa-prime-trainer.svg)](https://github.com/yourusername/daa/blob/main/LICENSE)

Distributed SGD/FSDP trainer implementation for the Prime distributed machine learning framework. Provides fault-tolerant, scalable training coordination with built-in incentive mechanisms through DAA ecosystem integration.

## Overview

DAA Prime Trainer implements a robust distributed training node that participates in federated learning and distributed training protocols. It provides:

- **Distributed SGD**: Scalable stochastic gradient descent across multiple nodes
- **FSDP Support**: Fully Sharded Data Parallel training for large models
- **Fault Tolerance**: Automatic recovery from node failures and network partitions
- **DAA Integration**: Token-based incentives and governance participation
- **Flexible Architecture**: Pluggable optimizers and aggregation strategies

## Features

- 🚀 **High Performance**: Optimized gradient computation and communication
- 🔄 **Fault Tolerant**: Automatic failure detection and recovery
- 🏆 **Incentivized**: Token rewards for quality contributions
- 📊 **Comprehensive Metrics**: Detailed training and performance monitoring
- 🔒 **Secure**: Cryptographic verification of gradient updates
- 🌐 **Network Agnostic**: Works with any transport layer

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
daa-prime-trainer = "0.2.1"
daa-prime-core = "0.2.1"
daa-prime-dht = "0.2.1"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

### Basic Training Node

```rust
use daa_prime_trainer::{TrainerNode, TrainingConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a trainer node
    let trainer = TrainerNode::new("trainer-001".to_string()).await?;
    
    // Start training process
    trainer.start_training().await?;
    
    // Monitor training status
    let status = trainer.get_status().await?;
    println!("Training status: {:?}", status);
    
    Ok(())
}
```

### Custom Training Configuration

```rust
use daa_prime_trainer::{TrainerNode, TrainingConfig};

let config = TrainingConfig {
    batch_size: 64,
    learning_rate: 0.001,
    max_epochs: 100,
};

// Create trainer with custom configuration
let trainer = TrainerNode::with_config("trainer-001".to_string(), config).await?;
```

### Running as Binary

The crate also provides a standalone binary for direct execution:

```bash
# Start a trainer node
cargo run --bin prime-trainer

# Or install and run
cargo install daa-prime-trainer
prime-trainer --node-id trainer-001 --config config.json
```

## Core Concepts

### Training Lifecycle

```rust
use daa_prime_trainer::{TrainerNode, TrainingPhase};

let trainer = TrainerNode::new("trainer-001".to_string()).await?;

// Training goes through several phases:
// 1. Initialization - Set up local model and data
// 2. Gradient Computation - Compute local gradients
// 3. Communication - Share gradients with coordinators
// 4. Aggregation - Receive aggregated updates
// 5. Model Update - Apply updates to local model

trainer.start_training().await?;
```

### Distributed Gradient Computation

```rust
use daa_prime_trainer::gradient::{GradientComputer, LocalGradients};
use daa_prime_core::GradientUpdate;

// Compute gradients on local data
let computer = GradientComputer::new();
let local_gradients = computer.compute_batch_gradients(&data_batch).await?;

// Create gradient update for sharing
let update = GradientUpdate {
    node_id: trainer.node_id(),
    model_version: trainer.current_model_version(),
    round: trainer.current_round(),
    gradients: local_gradients.into_hashmap(),
    metrics: trainer.get_training_metrics(),
    timestamp: current_timestamp(),
};
```

### Fault Tolerance

```rust
use daa_prime_trainer::{TrainerNode, FaultToleranceConfig};

let fault_config = FaultToleranceConfig {
    checkpoint_interval: Duration::from_secs(300), // 5 minutes
    max_retries: 3,
    timeout: Duration::from_secs(30),
    recovery_strategy: RecoveryStrategy::RestoreFromCheckpoint,
};

let trainer = TrainerNode::with_fault_tolerance(
    "trainer-001".to_string(),
    fault_config
).await?;
```

## Advanced Usage

### Custom Data Loading

```rust
use daa_prime_trainer::{TrainerNode, DataLoader};
use async_trait::async_trait;

// Implement custom data loader
struct CustomDataLoader {
    data_path: String,
}

#[async_trait]
impl DataLoader for CustomDataLoader {
    type Item = (Vec<f32>, Vec<f32>); // (features, labels)
    
    async fn load_batch(&self, batch_size: usize) -> Result<Vec<Self::Item>> {
        // Load data from custom source (database, API, etc.)
        // This is a simplified example
        let mut batch = Vec::new();
        for _ in 0..batch_size {
            let features = vec![0.5; 784]; // Mock MNIST features
            let labels = vec![1.0, 0.0, 0.0]; // One-hot encoded label
            batch.push((features, labels));
        }
        Ok(batch)
    }
    
    async fn size(&self) -> Result<usize> {
        // Return total dataset size
        Ok(60000) // Mock MNIST size
    }
}

// Use custom data loader with trainer
let data_loader = CustomDataLoader {
    data_path: "/path/to/data".to_string(),
};
let trainer = TrainerNode::with_data_loader("trainer-001".to_string(), data_loader).await?;
```

### Federated Learning

```rust
use daa_prime_trainer::{TrainerNode, FederatedConfig};
use daa_prime_core::{AggregationStrategy, OptimizerType};

// Configure for federated learning
let fed_config = FederatedConfig {
    aggregation_strategy: AggregationStrategy::FederatedAveraging,
    local_epochs: 5,          // Local training epochs per round
    participation_rate: 0.8,   // Fraction of clients per round
    min_clients: 10,          // Minimum clients for aggregation
};

let trainer = TrainerNode::with_federated_config(
    "trainer-001".to_string(),
    fed_config
).await?;

// Participate in federated training rounds
trainer.join_federated_round(42).await?;
```

### Model Checkpointing

```rust
use daa_prime_trainer::{TrainerNode, CheckpointManager};

let trainer = TrainerNode::new("trainer-001".to_string()).await?;

// Save checkpoint
trainer.save_checkpoint("checkpoint_epoch_10").await?;

// Load checkpoint
trainer.load_checkpoint("checkpoint_epoch_10").await?;

// List available checkpoints
let checkpoints = trainer.list_checkpoints().await?;
for checkpoint in checkpoints {
    println!("Checkpoint: {} (epoch {})", checkpoint.name, checkpoint.epoch);
}
```

### Performance Monitoring

```rust
use daa_prime_trainer::{TrainerNode, MetricsCollector};

let trainer = TrainerNode::new("trainer-001".to_string()).await?;

// Start metrics collection
let metrics_collector = MetricsCollector::new();
trainer.set_metrics_collector(metrics_collector).await?;

// Get real-time metrics
loop {
    let metrics = trainer.get_training_metrics().await?;
    println!("Loss: {:.4}, Accuracy: {:.4}, Throughput: {:.1} samples/s",
        metrics.loss,
        metrics.accuracy,
        metrics.samples_per_second
    );
    
    tokio::time::sleep(Duration::from_secs(10)).await;
}
```

### Integration with DHT

```rust
use daa_prime_trainer::TrainerNode;
use daa_prime_dht::{Dht, DhtConfig};

// Create trainer with DHT integration
let dht = Dht::new(peer_id, DhtConfig::default());
let trainer = TrainerNode::with_dht("trainer-001".to_string(), dht).await?;

// Trainer will automatically:
// - Store training checkpoints in DHT
// - Retrieve model updates from DHT
// - Share gradient updates via DHT
// - Discover other training nodes
```

## Configuration

### Training Parameters

```rust
use daa_prime_trainer::TrainingConfig;

let config = TrainingConfig {
    // Basic training parameters
    batch_size: 32,
    learning_rate: 0.001,
    max_epochs: 100,
    
    // Advanced parameters
    optimizer: OptimizerType::AdamW {
        beta1: 0.9,
        beta2: 0.999,
        weight_decay: 0.01,
    },
    scheduler: SchedulerType::CosineAnnealing {
        t_max: 100,
        eta_min: 0.0001,
    },
    
    // Distributed training
    gradient_clipping: Some(1.0),
    aggregation_strategy: AggregationStrategy::FederatedAveraging,
    
    // Fault tolerance
    checkpoint_interval: Duration::from_secs(300),
    max_retries: 3,
};
```

### Network Configuration

```rust
use daa_prime_trainer::NetworkConfig;

let network_config = NetworkConfig {
    // Communication settings
    max_message_size: 1024 * 1024, // 1MB
    timeout: Duration::from_secs(30),
    retry_interval: Duration::from_secs(5),
    
    // Compression
    enable_compression: true,
    compression_level: 6,
    
    // Security
    enable_encryption: true,
    verify_signatures: true,
};
```

## Command Line Interface

The trainer can be run as a standalone binary:

```bash
# Basic usage
prime-trainer --node-id trainer-001

# With custom configuration
prime-trainer --node-id trainer-001 --config config.json

# Specify coordinator endpoints
prime-trainer --node-id trainer-001 --coordinators coord1:8080,coord2:8080

# Enable verbose logging
prime-trainer --node-id trainer-001 --log-level debug

# Set data directory
prime-trainer --node-id trainer-001 --data-dir /path/to/data

# Join specific training round
prime-trainer --node-id trainer-001 --round 42
```

### Configuration File

```json
{
  "training": {
    "batch_size": 64,
    "learning_rate": 0.001,
    "max_epochs": 100,
    "optimizer": {
      "type": "AdamW",
      "beta1": 0.9,
      "beta2": 0.999,
      "weight_decay": 0.01
    }
  },
  "network": {
    "timeout": 30,
    "max_message_size": 1048576,
    "enable_compression": true
  },
  "fault_tolerance": {
    "checkpoint_interval": 300,
    "max_retries": 3
  }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use daa_prime_trainer::test_utils::*;
    
    #[tokio::test]
    async fn test_trainer_creation() {
        let trainer = TrainerNode::new("test-trainer".to_string()).await.unwrap();
        let status = trainer.get_status().await.unwrap();
        
        assert_eq!(status.node_id, "test-trainer");
        assert_eq!(status.current_epoch, 0);
        assert!(!status.is_training);
    }
    
    #[tokio::test]
    async fn test_gradient_computation() {
        let trainer = create_test_trainer().await;
        let mock_data = create_mock_dataset(100);
        
        let gradients = trainer.compute_gradients(&mock_data).await.unwrap();
        assert!(!gradients.is_empty());
        
        // Verify gradient properties
        for (layer_name, gradient) in gradients {
            assert!(!layer_name.is_empty());
            assert!(!gradient.is_empty());
            assert!(gradient.iter().all(|&x| x.is_finite()));
        }
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use daa_prime_coordinator::CoordinatorNode;
    
    #[tokio::test]
    async fn test_distributed_training() {
        // Start coordinator
        let coordinator = CoordinatorNode::new(
            "test-coordinator".to_string(),
            CoordinatorConfig::default()
        ).await.unwrap();
        
        // Start multiple trainers
        let mut trainers = Vec::new();
        for i in 0..3 {
            let trainer = TrainerNode::new(format!("trainer-{}", i)).await.unwrap();
            trainers.push(trainer);
        }
        
        // Run training round
        for trainer in &trainers {
            trainer.start_training().await.unwrap();
        }
        
        // Verify convergence
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        for trainer in &trainers {
            let metrics = trainer.get_training_metrics().await.unwrap();
            assert!(metrics.loss < 1.0); // Should have decreased
        }
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_gradient_aggregation_properties(
        gradients in prop::collection::vec(
            prop::collection::hash_map(
                "[a-zA-Z0-9_]+", 
                prop::collection::vec(-1.0f32..1.0f32, 1..100),
                1..10
            ),
            2..10
        )
    ) {
        let aggregated = aggregate_gradients(&gradients).unwrap();
        
        // Properties to verify:
        // 1. All gradient keys are preserved
        // 2. Aggregated values are finite
        // 3. Aggregated dimensions match input dimensions
        
        for (key, values) in &aggregated {
            assert!(!key.is_empty());
            assert!(!values.is_empty());
            assert!(values.iter().all(|&x| x.is_finite()));
        }
    }
}
```

## Performance Optimization

### Memory Management

```rust
use daa_prime_trainer::{TrainerNode, MemoryConfig};

let memory_config = MemoryConfig {
    gradient_buffer_size: 1024 * 1024, // 1MB
    model_cache_size: 512 * 1024 * 1024, // 512MB
    enable_gradient_compression: true,
    compression_ratio: 0.1, // 10x compression
};

let trainer = TrainerNode::with_memory_config(
    "trainer-001".to_string(),
    memory_config
).await?;
```

### Parallel Processing

```rust
use daa_prime_trainer::{TrainerNode, ParallelConfig};
use rayon::prelude::*;

let parallel_config = ParallelConfig {
    gradient_computation_threads: num_cpus::get(),
    data_loading_threads: 4,
    network_io_threads: 2,
};

// Parallel gradient computation
let gradients: Vec<_> = data_batches
    .par_iter()
    .map(|batch| compute_batch_gradients(batch))
    .collect();
```

### Benchmarking

```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use daa_prime_trainer::*;
    
    fn bench_gradient_computation(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let trainer = rt.block_on(TrainerNode::new("bench-trainer".to_string())).unwrap();
        
        c.bench_function("gradient_computation_1k_samples", |b| {
            b.to_async(&rt).iter(|| async {
                let data = create_mock_dataset(1000);
                let gradients = trainer.compute_gradients(black_box(&data)).await.unwrap();
                black_box(gradients);
            });
        });
    }
    
    criterion_group!(benches, bench_gradient_computation);
    criterion_main!(benches);
}
```

## Troubleshooting

### Common Issues

1. **Training Divergence**
   ```rust
   // Monitor gradient norms
   let grad_norm = calculate_gradient_norm(&gradients);
   if grad_norm > 10.0 {
       eprintln!("Warning: Large gradient norm detected: {}", grad_norm);
       // Apply gradient clipping
       clip_gradients(&mut gradients, 1.0);
   }
   ```

2. **Memory Issues**
   ```rust
   // Enable gradient checkpointing for large models
   let config = TrainingConfig {
       enable_gradient_checkpointing: true,
       ..Default::default()
   };
   ```

3. **Network Timeouts**
   ```rust
   // Increase timeout for slow networks
   let network_config = NetworkConfig {
       timeout: Duration::from_secs(120), // 2 minutes
       retry_attempts: 5,
       ..Default::default()
   };
   ```

## Roadmap

- [ ] GPU acceleration support
- [ ] Model parallel training (tensor parallelism)
- [ ] Advanced aggregation algorithms (Byzantine fault tolerance)
- [ ] Differential privacy integration
- [ ] Automated hyperparameter tuning
- [ ] Real-time model serving integration

## Contributing

Contributions are welcome! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Related Crates

- [`daa-prime-core`](../prime-core): Core types and protocol definitions
- [`daa-prime-dht`](../prime-dht): Distributed hash table for model storage
- [`daa-prime-coordinator`](../prime-coordinator): Training coordination and governance
- [`daa-prime-cli`](../prime-cli): Command-line interface

## References

- [Federated Learning](https://arxiv.org/abs/1602.05629) - McMahan et al.
- [FSDP Paper](https://arxiv.org/abs/2101.01234) - Fully Sharded Data Parallel
- [Byzantine ML](https://arxiv.org/abs/1703.02757) - Byzantine-robust distributed learning

## Support

- 📖 [Documentation](https://docs.rs/daa-prime-trainer)
- 🐛 [Issue Tracker](https://github.com/yourusername/daa/issues)  
- 💬 [Discussions](https://github.com/yourusername/daa/discussions)