//! P2P Network Latency and Throughput Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use daa_compute::{P2PNetwork, SwarmConfig, GradientMessage};
use libp2p::{PeerId, Multiaddr};
use tokio::runtime::Runtime;
use std::time::Duration;
use rand::prelude::*;

/// Benchmark P2P message latency
fn benchmark_p2p_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("p2p_latency");
    group.measurement_time(Duration::from_secs(10));
    
    // Test different message sizes
    let message_sizes = vec![1_024, 10_240, 102_400, 1_048_576]; // 1KB, 10KB, 100KB, 1MB
    
    for size in message_sizes {
        group.throughput(Throughput::Bytes(size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("broadcast", size),
            &size,
            |b, &size| {
                b.to_async(&rt).iter(|| async move {
                    let config = SwarmConfig::default();
                    let mut network = P2PNetwork::new(config).await.unwrap();
                    
                    // Create gradient of specified size
                    let gradient: Vec<f32> = (0..size/4).map(|_| rand::random()).collect();
                    
                    // Measure broadcast latency
                    let start = std::time::Instant::now();
                    network.broadcast_gradient(gradient).await.unwrap();
                    black_box(start.elapsed())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark P2P network throughput
fn benchmark_p2p_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("p2p_throughput");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);
    
    // Test sustained throughput with different batch sizes
    let batch_sizes = vec![10, 50, 100, 500];
    let gradient_size = 100_000; // 100K parameters
    
    for batch_size in batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("batch_broadcast", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async move {
                    let config = SwarmConfig {
                        compression_level: 3,
                        ..Default::default()
                    };
                    let mut network = P2PNetwork::new(config).await.unwrap();
                    
                    // Broadcast multiple gradients to measure throughput
                    for _ in 0..batch_size {
                        let gradient: Vec<f32> = (0..gradient_size)
                            .map(|_| rand::random::<f32>() * 0.01)
                            .collect();
                        network.broadcast_gradient(gradient).await.unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark gradient compression impact on latency
fn benchmark_compression_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("compression_impact");
    group.measurement_time(Duration::from_secs(20));
    
    let gradient_size = 1_000_000; // 1M parameters
    let compression_levels = vec![0, 3, 6, 9];
    
    for level in compression_levels {
        group.bench_with_input(
            BenchmarkId::new("compression_level", level),
            &level,
            |b, &level| {
                b.to_async(&rt).iter(|| async move {
                    let config = SwarmConfig {
                        compression_level: level,
                        ..Default::default()
                    };
                    let mut network = P2PNetwork::new(config).await.unwrap();
                    
                    // Create realistic gradient (sparse, with many small values)
                    let mut gradient = vec![0.0f32; gradient_size];
                    let mut rng = rand::thread_rng();
                    
                    // Make 10% of values non-zero (typical sparsity)
                    for i in (0..gradient_size).step_by(10) {
                        gradient[i] = rng.gen_range(-0.01..0.01);
                    }
                    
                    let start = std::time::Instant::now();
                    network.broadcast_gradient(gradient).await.unwrap();
                    black_box(start.elapsed())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark NAT traversal overhead
fn benchmark_nat_traversal(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("nat_traversal");
    group.measurement_time(Duration::from_secs(15));
    
    group.bench_function("with_nat", |b| {
        b.to_async(&rt).iter(|| async {
            let config = SwarmConfig {
                enable_nat_traversal: true,
                enable_relay: true,
                ..Default::default()
            };
            let mut network = P2PNetwork::new(config).await.unwrap();
            
            // Connect to bootstrap node through NAT
            network.bootstrap().await.unwrap();
            
            // Send test gradient
            let gradient = vec![0.1f32; 10_000];
            network.broadcast_gradient(gradient).await.unwrap();
        });
    });
    
    group.bench_function("without_nat", |b| {
        b.to_async(&rt).iter(|| async {
            let config = SwarmConfig {
                enable_nat_traversal: false,
                enable_relay: false,
                ..Default::default()
            };
            let mut network = P2PNetwork::new(config).await.unwrap();
            
            // Direct connection (no NAT)
            network.bootstrap().await.unwrap();
            
            // Send test gradient
            let gradient = vec![0.1f32; 10_000];
            network.broadcast_gradient(gradient).await.unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark peer discovery time
fn benchmark_peer_discovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("peer_discovery");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);
    
    // Test discovery with different network sizes
    let network_sizes = vec![10, 50, 100, 500];
    
    for size in network_sizes {
        group.bench_with_input(
            BenchmarkId::new("network_size", size),
            &size,
            |b, &size| {
                b.to_async(&rt).iter(|| async move {
                    let config = SwarmConfig {
                        enable_mdns: true,
                        ..Default::default()
                    };
                    let mut network = P2PNetwork::new(config).await.unwrap();
                    
                    // Measure time to discover N peers
                    let start = std::time::Instant::now();
                    
                    // Simulate peer discovery
                    for _ in 0..size {
                        network.bootstrap().await.unwrap();
                    }
                    
                    black_box(start.elapsed())
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark gradient aggregation algorithms
fn benchmark_gradient_aggregation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("gradient_aggregation");
    group.measurement_time(Duration::from_secs(20));
    
    let gradient_sizes = vec![10_000, 100_000, 1_000_000];
    let peer_counts = vec![4, 8, 16, 32];
    
    for gradient_size in gradient_sizes {
        for peer_count in &peer_counts {
            group.throughput(Throughput::Elements((gradient_size * peer_count) as u64));
            
            group.bench_with_input(
                BenchmarkId::new(format!("size_{}_peers", gradient_size), peer_count),
                peer_count,
                |b, &peer_count| {
                    b.to_async(&rt).iter(|| async move {
                        // Simulate gradient aggregation
                        let mut gradients = Vec::new();
                        let mut rng = rand::thread_rng();
                        
                        for _ in 0..peer_count {
                            let gradient: Vec<f32> = (0..gradient_size)
                                .map(|_| rng.gen_range(-0.01..0.01))
                                .collect();
                            gradients.push(gradient);
                        }
                        
                        // All-reduce operation
                        let mut aggregated = vec![0.0f32; gradient_size];
                        for gradient in &gradients {
                            for (i, &val) in gradient.iter().enumerate() {
                                aggregated[i] += val;
                            }
                        }
                        
                        // Average
                        for val in &mut aggregated {
                            *val /= peer_count as f32;
                        }
                        
                        black_box(aggregated)
                    });
                },
            );
        }
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_p2p_latency,
    benchmark_p2p_throughput,
    benchmark_compression_impact,
    benchmark_nat_traversal,
    benchmark_peer_discovery,
    benchmark_gradient_aggregation
);
criterion_main!(benches);