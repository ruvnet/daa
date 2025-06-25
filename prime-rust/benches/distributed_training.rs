//! Benchmarks for distributed training performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use prime_core::types::*;
use prime_dht::{Dht, DhtConfig};
use libp2p::PeerId;
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark gradient aggregation performance
fn bench_gradient_aggregation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("gradient_aggregation");
    
    // Test different numbers of nodes
    for node_count in [5, 10, 20, 50].iter() {
        group.throughput(Throughput::Elements(*node_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("federated_averaging", node_count),
            node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async {
                    let gradients = generate_gradients(node_count);
                    let aggregated = federated_averaging(black_box(gradients));
                    black_box(aggregated)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("trimmed_mean", node_count),
            node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async {
                    let gradients = generate_gradients(node_count);
                    let aggregated = trimmed_mean(black_box(gradients), 0.1);
                    black_box(aggregated)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark DHT operations
fn bench_dht_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("dht_operations");
    
    // Test different data sizes
    for size in [1024, 10240, 102400].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("put_operation", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let dht = Dht::new(PeerId::random(), DhtConfig::default());
                    let key = b"benchmark_key".to_vec();
                    let value = vec![0u8; size];
                    
                    dht.put(black_box(key), black_box(value)).await.unwrap();
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("get_operation", size),
            size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let dht = Dht::new(PeerId::random(), DhtConfig::default());
                    let key = b"benchmark_key".to_vec();
                    let value = vec![0u8; size];
                    
                    // Pre-populate
                    dht.put(key.clone(), value).await.unwrap();
                    
                    let result = dht.get(black_box(key)).await.unwrap();
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark message serialization
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    // Create test messages of different sizes
    let small_update = create_gradient_update(10);
    let medium_update = create_gradient_update(100);
    let large_update = create_gradient_update(1000);
    
    group.bench_function("serialize_small", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&small_update)).unwrap();
            black_box(serialized)
        })
    });
    
    group.bench_function("deserialize_small", |b| {
        let serialized = serde_json::to_string(&small_update).unwrap();
        b.iter(|| {
            let deserialized: GradientUpdate = serde_json::from_str(black_box(&serialized)).unwrap();
            black_box(deserialized)
        })
    });
    
    group.bench_function("serialize_medium", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&medium_update)).unwrap();
            black_box(serialized)
        })
    });
    
    group.bench_function("serialize_large", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&large_update)).unwrap();
            black_box(serialized)
        })
    });
    
    group.finish();
}

/// Benchmark network topology performance
fn bench_network_topologies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("network_topologies");
    
    for node_count in [10, 20, 50].iter() {
        group.throughput(Throughput::Elements(*node_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("full_mesh_broadcast", node_count),
            node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async {
                    let result = simulate_full_mesh_broadcast(black_box(node_count)).await;
                    black_box(result)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("star_topology_broadcast", node_count),
            node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async {
                    let result = simulate_star_topology_broadcast(black_box(node_count)).await;
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark consensus performance
fn bench_consensus(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consensus");
    
    for node_count in [4, 7, 10, 16].iter() {
        group.throughput(Throughput::Elements(*node_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("pbft_consensus", node_count),
            node_count,
            |b, &node_count| {
                b.to_async(&rt).iter(|| async {
                    let result = simulate_pbft_consensus(black_box(*node_count)).await;
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark training convergence
fn bench_training_convergence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("training_convergence");
    group.sample_size(10); // Fewer samples for long-running tests
    
    for strategy in ["federated_avg", "secure_agg", "krum"].iter() {
        group.bench_with_input(
            BenchmarkId::new("convergence_rate", strategy),
            strategy,
            |b, &strategy| {
                b.to_async(&rt).iter(|| async {
                    let epochs = simulate_training_convergence(black_box(strategy)).await;
                    black_box(epochs)
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for benchmarks

fn generate_gradients(node_count: usize) -> Vec<HashMap<String, Vec<f32>>> {
    (0..node_count)
        .map(|_| {
            HashMap::from([
                ("layer1".to_string(), vec![0.1, 0.2, 0.3, 0.4, 0.5]),
                ("layer2".to_string(), vec![0.6, 0.7, 0.8, 0.9, 1.0]),
                ("layer3".to_string(), vec![1.1, 1.2, 1.3, 1.4, 1.5]),
            ])
        })
        .collect()
}

fn federated_averaging(gradients: Vec<HashMap<String, Vec<f32>>>) -> HashMap<String, Vec<f32>> {
    let mut aggregated = HashMap::new();
    let node_count = gradients.len() as f32;
    
    for gradient_map in gradients {
        for (layer, grads) in gradient_map {
            let entry = aggregated.entry(layer).or_insert_with(Vec::new);
            if entry.is_empty() {
                *entry = grads;
            } else {
                for (i, grad) in grads.iter().enumerate() {
                    if i < entry.len() {
                        entry[i] += grad;
                    }
                }
            }
        }
    }
    
    // Average the gradients
    for grads in aggregated.values_mut() {
        for grad in grads {
            *grad /= node_count;
        }
    }
    
    aggregated
}

fn trimmed_mean(gradients: Vec<HashMap<String, Vec<f32>>>, trim_ratio: f32) -> HashMap<String, Vec<f32>> {
    // Simplified trimmed mean implementation
    let trim_count = (gradients.len() as f32 * trim_ratio) as usize;
    let keep_count = gradients.len() - 2 * trim_count;
    
    if keep_count == 0 {
        return HashMap::new();
    }
    
    federated_averaging(gradients.into_iter().take(keep_count).collect())
}

fn create_gradient_update(layer_size: usize) -> GradientUpdate {
    let mut gradients = HashMap::new();
    
    for i in 0..3 {
        let layer_name = format!("layer_{}", i);
        let grad_values: Vec<f32> = (0..layer_size).map(|j| j as f32 * 0.001).collect();
        gradients.insert(layer_name, grad_values);
    }
    
    GradientUpdate {
        node_id: NodeId::new("benchmark_node"),
        model_version: 1,
        round: 0,
        gradients,
        metrics: TrainingMetrics {
            loss: 0.5,
            accuracy: 0.85,
            samples_processed: 1000,
            computation_time_ms: 500,
        },
        timestamp: 1234567890,
    }
}

async fn simulate_full_mesh_broadcast(node_count: usize) -> usize {
    // Simulate broadcasting to all nodes in full mesh
    let mut total_messages = 0;
    
    for i in 0..node_count {
        for j in 0..node_count {
            if i != j {
                total_messages += 1;
                // Simulate network delay
                tokio::time::sleep(Duration::from_nanos(100)).await;
            }
        }
    }
    
    total_messages
}

async fn simulate_star_topology_broadcast(node_count: usize) -> usize {
    // Simulate broadcasting in star topology (hub + spokes)
    let total_messages = (node_count - 1) * 2; // Hub to all, all to hub
    
    for _ in 0..total_messages {
        // Simulate network delay
        tokio::time::sleep(Duration::from_nanos(50)).await;
    }
    
    total_messages
}

async fn simulate_pbft_consensus(node_count: usize) -> usize {
    // Simulate PBFT consensus rounds
    let f = (node_count - 1) / 3; // Byzantine fault tolerance
    let phases = 3; // Pre-prepare, prepare, commit
    let messages_per_phase = node_count * (node_count - 1);
    
    for _ in 0..phases {
        for _ in 0..messages_per_phase {
            // Simulate consensus message processing
            tokio::time::sleep(Duration::from_nanos(200)).await;
        }
    }
    
    phases * messages_per_phase
}

async fn simulate_training_convergence(strategy: &str) -> usize {
    let target_accuracy = 0.95;
    let mut current_accuracy = 0.5;
    let mut epochs = 0;
    
    while current_accuracy < target_accuracy && epochs < 100 {
        epochs += 1;
        
        // Different convergence rates for different strategies
        let improvement = match strategy {
            "federated_avg" => 0.01,
            "secure_agg" => 0.008,
            "krum" => 0.012,
            _ => 0.01,
        };
        
        current_accuracy += improvement * (1.0 - current_accuracy);
        
        // Simulate training time
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
    
    epochs
}

criterion_group!(
    benches,
    bench_gradient_aggregation,
    bench_dht_operations,
    bench_serialization,
    bench_network_topologies,
    bench_consensus,
    bench_training_convergence
);
criterion_main!(benches);