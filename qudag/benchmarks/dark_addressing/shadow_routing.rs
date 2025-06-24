//! Shadow address routing benchmarks

use criterion::{black_box, Criterion, BenchmarkId, Throughput};
use rand::{thread_rng, RngCore, Rng};
use std::time::{Duration, Instant};
use super::BenchmarkConfig;

/// Mock shadow address handler for benchmarking
pub struct MockShadowRouter {
    routing_table: Vec<RouteEntry>,
}

struct RouteEntry {
    shadow_address: Vec<u8>,
    next_hops: Vec<Vec<u8>>,
    latency_us: u64,
}

impl MockShadowRouter {
    pub fn new() -> Self {
        Self {
            routing_table: Vec::new(),
        }
    }

    pub fn with_routes(count: usize) -> Self {
        let mut router = Self::new();
        
        for _ in 0..count {
            let mut shadow_address = vec![0u8; 96]; // View key + spend key + metadata
            thread_rng().fill_bytes(&mut shadow_address);
            
            let hop_count = thread_rng().gen_range(2..5);
            let mut next_hops = Vec::with_capacity(hop_count);
            
            for _ in 0..hop_count {
                let mut hop = vec![0u8; 32]; // Peer ID
                thread_rng().fill_bytes(&mut hop);
                next_hops.push(hop);
            }
            
            router.routing_table.push(RouteEntry {
                shadow_address,
                next_hops,
                latency_us: thread_rng().gen_range(100..1000),
            });
        }
        
        router
    }

    pub fn generate_shadow_address(&self) -> Vec<u8> {
        let mut addr = vec![0u8; 96];
        thread_rng().fill_bytes(&mut addr);
        addr
    }

    pub fn derive_shadow_address(&self, base: &[u8]) -> Vec<u8> {
        let mut derived = vec![0u8; base.len()];
        
        // Simulate cryptographic derivation
        for (i, &byte) in base.iter().enumerate() {
            derived[i] = byte.wrapping_add((i as u8).wrapping_mul(37));
        }
        
        // Replace some bytes with random data
        thread_rng().fill_bytes(&mut derived[32..64]);
        
        derived
    }

    pub fn route_message(&self, shadow: &[u8], message: &[u8]) -> Duration {
        // Find route (or generate one)
        let route = self.routing_table.iter()
            .find(|r| r.shadow_address == shadow)
            .or_else(|| self.routing_table.first());
        
        let base_latency = route.map(|r| r.latency_us).unwrap_or(500);
        let size_factor = (message.len() as u64) / 1024; // Per KB
        let total_latency = base_latency + size_factor * 10;
        
        Duration::from_micros(total_latency)
    }

    pub fn simulate_onion_routing(&self, shadow: &[u8], message: &[u8], layers: usize) -> Duration {
        let mut total_latency = Duration::from_micros(0);
        let mut current_message = message.to_vec();
        
        for layer in 0..layers {
            // Simulate encryption overhead
            let mut encrypted = vec![0u8; current_message.len() + 32]; // Add overhead
            thread_rng().fill_bytes(&mut encrypted[..32]);
            encrypted[32..].copy_from_slice(&current_message);
            
            // Simulate routing
            let hop_latency = self.route_message(shadow, &encrypted);
            total_latency += hop_latency;
            
            // Prepare for next layer
            current_message = encrypted;
        }
        
        total_latency
    }
}

/// Benchmark shadow address routing operations
pub fn benchmark_routing(c: &mut Criterion, config: &BenchmarkConfig) {
    let mut group = c.benchmark_group("shadow_address_routing");
    
    // Benchmark address generation
    benchmark_address_generation(&mut group);
    
    // Benchmark address derivation
    benchmark_address_derivation(&mut group);
    
    // Benchmark message routing
    benchmark_message_routing(&mut group, config);
    
    // Benchmark onion routing
    benchmark_onion_routing(&mut group);
    
    // Benchmark routing table scaling
    benchmark_routing_table_scaling(&mut group, config);
    
    group.finish();
}

fn benchmark_address_generation(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    let router = MockShadowRouter::new();
    
    group.bench_function("generate_shadow_address", |b| {
        b.iter(|| {
            black_box(router.generate_shadow_address());
        })
    });
    
    // Benchmark batch generation
    group.bench_function("generate_100_addresses_batch", |b| {
        b.iter(|| {
            let mut addresses = Vec::with_capacity(100);
            for _ in 0..100 {
                addresses.push(router.generate_shadow_address());
            }
            black_box(addresses);
        })
    });
}

fn benchmark_address_derivation(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    let router = MockShadowRouter::new();
    let base_address = router.generate_shadow_address();
    
    group.bench_function("derive_single_address", |b| {
        b.iter(|| {
            black_box(router.derive_shadow_address(&base_address));
        })
    });
    
    // Benchmark chain derivation
    group.bench_function("derive_address_chain_10", |b| {
        b.iter(|| {
            let mut current = base_address.clone();
            for _ in 0..10 {
                current = router.derive_shadow_address(&current);
            }
            black_box(current);
        })
    });
}

fn benchmark_message_routing(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, config: &BenchmarkConfig) {
    let router = MockShadowRouter::with_routes(1000);
    let shadow_address = router.generate_shadow_address();
    
    // Benchmark routing with different message sizes
    for &size in &config.message_sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("route_message_size", size),
            &size,
            |b, &size| {
                let message = vec![0u8; size];
                b.iter(|| {
                    black_box(router.route_message(&shadow_address, &message));
                })
            },
        );
    }
    
    // Benchmark routing latency distribution
    group.bench_function("route_message_latency_distribution", |b| {
        let message = vec![0u8; 1024];
        let mut latencies = Vec::with_capacity(1000);
        
        b.iter(|| {
            let start = Instant::now();
            let _latency = router.route_message(&shadow_address, &message);
            latencies.push(start.elapsed());
        });
        
        // In real benchmark, would analyze latency distribution
        black_box(latencies);
    });
}

fn benchmark_onion_routing(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    let router = MockShadowRouter::with_routes(100);
    let shadow_address = router.generate_shadow_address();
    let message = vec![0u8; 1024];
    
    // Benchmark different layer counts
    for layers in [1, 3, 5, 7] {
        group.bench_with_input(
            BenchmarkId::new("onion_routing_layers", layers),
            &layers,
            |b, &layers| {
                b.iter(|| {
                    black_box(router.simulate_onion_routing(&shadow_address, &message, layers));
                })
            },
        );
    }
    
    // Benchmark onion routing with payload encryption
    group.bench_function("onion_routing_with_encryption", |b| {
        b.iter(|| {
            // Simulate full onion encryption
            let mut encrypted_message = message.clone();
            
            for _ in 0..3 {
                // Simulate AES-GCM encryption
                let mut layer = vec![0u8; encrypted_message.len() + 16]; // Add auth tag
                thread_rng().fill_bytes(&mut layer[..16]);
                layer[16..].copy_from_slice(&encrypted_message);
                encrypted_message = layer;
            }
            
            black_box(router.simulate_onion_routing(&shadow_address, &encrypted_message, 3));
        })
    });
}

fn benchmark_routing_table_scaling(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, config: &BenchmarkConfig) {
    // Test routing performance with different table sizes
    for &count in &config.domain_counts {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("routing_table_size", count),
            &count,
            |b, &count| {
                let router = MockShadowRouter::with_routes(count);
                let addresses: Vec<_> = (0..100)
                    .map(|_| router.generate_shadow_address())
                    .collect();
                let message = vec![0u8; 1024];
                let mut idx = 0;
                
                b.iter(|| {
                    let shadow = &addresses[idx % addresses.len()];
                    idx += 1;
                    black_box(router.route_message(shadow, &message));
                })
            },
        );
    }
    
    // Benchmark concurrent routing
    use std::sync::Arc;
    use std::thread;
    
    let router = Arc::new(MockShadowRouter::with_routes(10000));
    
    group.bench_function("concurrent_routing_10_threads", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let router = Arc::clone(&router);
                    thread::spawn(move || {
                        let shadow = router.generate_shadow_address();
                        let message = vec![i as u8; 1024];
                        black_box(router.route_message(&shadow, &message));
                    })
                })
                .collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}