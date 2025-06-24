//! Dark domain resolution benchmarks

use criterion::{black_box, Criterion, BenchmarkId, Throughput};
use rand::{thread_rng, RngCore};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::BenchmarkConfig;

/// Mock dark domain resolver for benchmarking
pub struct MockDarkResolver {
    domains: Arc<RwLock<HashMap<String, DomainRecord>>>,
}

struct DomainRecord {
    encrypted_address: Vec<u8>,
    public_key: Vec<u8>,
    registered_at: u64,
}

impl MockDarkResolver {
    pub fn new() -> Self {
        Self {
            domains: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_domains(count: usize) -> Self {
        let resolver = Self::new();
        for i in 0..count {
            resolver.register_domain(&format!("test{}.dark", i));
        }
        resolver
    }

    pub fn register_domain(&self, domain: &str) {
        let mut domains = self.domains.write().unwrap();
        
        // Simulate ML-KEM-768 encryption
        let mut encrypted_address = vec![0u8; 2432]; // ML-KEM-768 ciphertext
        let mut public_key = vec![0u8; 1184]; // ML-KEM-768 public key
        thread_rng().fill_bytes(&mut encrypted_address);
        thread_rng().fill_bytes(&mut public_key);
        
        domains.insert(domain.to_string(), DomainRecord {
            encrypted_address,
            public_key,
            registered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
    }

    pub fn lookup_domain(&self, domain: &str) -> Option<Vec<u8>> {
        let domains = self.domains.read().unwrap();
        domains.get(domain).map(|r| r.encrypted_address.clone())
    }

    pub fn resolve_address(&self, domain: &str, _secret_key: &[u8]) -> Option<Vec<u8>> {
        self.lookup_domain(domain).map(|encrypted| {
            // Simulate ML-KEM decapsulation
            let mut address = vec![0u8; 16]; // IPv6 address
            for (i, &byte) in encrypted.iter().take(16).enumerate() {
                address[i] = byte ^ 0xFF; // Simple mock decryption
            }
            address
        })
    }
}

/// Benchmark dark domain resolution operations
pub fn benchmark_resolution(c: &mut Criterion, config: &BenchmarkConfig) {
    let mut group = c.benchmark_group("dark_domain_resolution");
    
    // Benchmark domain registration
    benchmark_registration(&mut group, config);
    
    // Benchmark domain lookup
    benchmark_lookup(&mut group, config);
    
    // Benchmark full resolution with decryption
    benchmark_full_resolution(&mut group, config);
    
    // Benchmark scaling with domain count
    benchmark_domain_scaling(&mut group, config);
    
    // Benchmark concurrent access
    benchmark_concurrent_access(&mut group);
    
    group.finish();
}

fn benchmark_registration(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, _config: &BenchmarkConfig) {
    let resolver = MockDarkResolver::new();
    let mut counter = 0;
    
    group.bench_function("register_single_domain", |b| {
        b.iter(|| {
            counter += 1;
            let domain = format!("bench{}.dark", counter);
            black_box(resolver.register_domain(&domain));
        })
    });
    
    // Benchmark batch registration
    group.bench_function("register_100_domains_batch", |b| {
        b.iter(|| {
            let resolver = MockDarkResolver::new();
            for i in 0..100 {
                let domain = format!("batch{}.dark", i);
                resolver.register_domain(&domain);
            }
            black_box(resolver);
        })
    });
}

fn benchmark_lookup(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, config: &BenchmarkConfig) {
    // Benchmark lookup with different cache states
    for &domain_count in &config.domain_counts {
        let resolver = MockDarkResolver::with_domains(domain_count);
        
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("lookup_with_n_domains", domain_count),
            &domain_count,
            |b, _| {
                b.iter(|| {
                    let idx = thread_rng().next_u64() % domain_count as u64;
                    let domain = format!("test{}.dark", idx);
                    black_box(resolver.lookup_domain(&domain));
                })
            },
        );
    }
    
    // Benchmark cache miss
    let resolver = MockDarkResolver::with_domains(1000);
    group.bench_function("lookup_nonexistent_domain", |b| {
        let mut counter = 10000;
        b.iter(|| {
            counter += 1;
            let domain = format!("missing{}.dark", counter);
            black_box(resolver.lookup_domain(&domain));
        })
    });
}

fn benchmark_full_resolution(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, _config: &BenchmarkConfig) {
    let resolver = MockDarkResolver::with_domains(1000);
    let secret_key = vec![0u8; 2400]; // ML-KEM-768 secret key
    
    group.bench_function("resolve_with_decryption", |b| {
        b.iter(|| {
            let idx = thread_rng().next_u64() % 1000;
            let domain = format!("test{}.dark", idx);
            black_box(resolver.resolve_address(&domain, &secret_key));
        })
    });
    
    // Benchmark resolution with caching simulation
    group.bench_function("resolve_with_cache_hit", |b| {
        let domain = "test500.dark";
        // Warm up cache
        resolver.resolve_address(domain, &secret_key);
        
        b.iter(|| {
            black_box(resolver.resolve_address(domain, &secret_key));
        })
    });
}

fn benchmark_domain_scaling(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, config: &BenchmarkConfig) {
    // Test how performance scales with number of registered domains
    for &count in &config.domain_counts {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("scaling_lookup_ops_per_sec", count),
            &count,
            |b, &count| {
                let resolver = MockDarkResolver::with_domains(count);
                let domains: Vec<String> = (0..100)
                    .map(|i| format!("test{}.dark", i % count))
                    .collect();
                let mut idx = 0;
                
                b.iter(|| {
                    let domain = &domains[idx % domains.len()];
                    idx += 1;
                    black_box(resolver.lookup_domain(domain));
                })
            },
        );
    }
}

fn benchmark_concurrent_access(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    use std::thread;
    
    let resolver = Arc::new(MockDarkResolver::with_domains(10000));
    
    // Benchmark concurrent reads
    group.bench_function("concurrent_10_readers", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let resolver = Arc::clone(&resolver);
                    thread::spawn(move || {
                        let domain = format!("test{}.dark", i * 100);
                        black_box(resolver.lookup_domain(&domain));
                    })
                })
                .collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
    
    // Benchmark mixed read/write
    group.bench_function("concurrent_mixed_ops", |b| {
        let counter = Arc::new(RwLock::new(20000));
        
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let resolver = Arc::clone(&resolver);
                    let counter = Arc::clone(&counter);
                    
                    thread::spawn(move || {
                        if i % 3 == 0 {
                            // Write operation
                            let mut c = counter.write().unwrap();
                            *c += 1;
                            let domain = format!("new{}.dark", *c);
                            resolver.register_domain(&domain);
                        } else {
                            // Read operation
                            let domain = format!("test{}.dark", i * 100);
                            black_box(resolver.lookup_domain(&domain));
                        }
                    })
                })
                .collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}