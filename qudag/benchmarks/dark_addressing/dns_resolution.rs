//! DNS resolution benchmarks

use criterion::{black_box, Criterion, BenchmarkId, Throughput};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use super::BenchmarkConfig;

/// Mock DNS resolver for benchmarking
pub struct MockDnsResolver {
    cache: Arc<RwLock<HashMap<String, DnsRecord>>>,
    upstream_latency: Duration,
}

#[derive(Clone)]
struct DnsRecord {
    addresses: Vec<Vec<u8>>,
    ttl: u64,
    cached_at: Instant,
}

impl MockDnsResolver {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            upstream_latency: Duration::from_millis(50),
        }
    }

    pub fn with_latency(latency: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            upstream_latency: latency,
        }
    }

    pub fn with_preloaded_cache(entries: usize) -> Self {
        let resolver = Self::new();
        
        // Preload cache with entries
        for i in 0..entries {
            let domain = format!("cached{}.example.com", i);
            resolver.resolve(&domain);
        }
        
        resolver
    }

    pub fn resolve(&self, domain: &str) -> Vec<u8> {
        // Check cache first
        if let Some(record) = self.cache_lookup(domain) {
            return record.addresses[0].clone();
        }

        // Simulate upstream DNS query
        std::thread::sleep(self.upstream_latency);
        
        // Generate mock IP address
        let mut addr = vec![0u8; 16]; // IPv6
        thread_rng().fill(&mut addr[..]);
        
        // Cache the result
        self.cache_insert(domain, vec![addr.clone()], 300); // 5 minute TTL
        
        addr
    }

    pub fn resolve_all(&self, domain: &str) -> Vec<Vec<u8>> {
        // Check cache first
        if let Some(record) = self.cache_lookup(domain) {
            return record.addresses;
        }

        // Simulate upstream DNS query for multiple records
        std::thread::sleep(self.upstream_latency);
        
        // Generate multiple mock addresses
        let count = thread_rng().gen_range(1..=5);
        let mut addresses = Vec::with_capacity(count);
        
        for _ in 0..count {
            let mut addr = vec![0u8; 16];
            thread_rng().fill(&mut addr[..]);
            addresses.push(addr);
        }
        
        // Cache the result
        self.cache_insert(domain, addresses.clone(), 300);
        
        addresses
    }

    pub fn resolve_with_fallback(&self, domain: &str, fallback_servers: &[&str]) -> Option<Vec<u8>> {
        // Try primary resolution
        if let Ok(addr) = std::panic::catch_unwind(|| self.resolve(domain)) {
            return Some(addr);
        }

        // Try fallback servers
        for &server in fallback_servers {
            std::thread::sleep(Duration::from_millis(100)); // Fallback latency
            
            if thread_rng().gen_bool(0.8) { // 80% success rate
                let mut addr = vec![0u8; 4]; // IPv4 for fallback
                thread_rng().fill(&mut addr[..]);
                return Some(addr);
            }
        }

        None
    }

    fn cache_lookup(&self, domain: &str) -> Option<DnsRecord> {
        let cache = self.cache.read().unwrap();
        cache.get(domain).and_then(|record| {
            // Check TTL
            let age = record.cached_at.elapsed().as_secs();
            if age < record.ttl {
                Some(record.clone())
            } else {
                None
            }
        })
    }

    fn cache_insert(&self, domain: &str, addresses: Vec<Vec<u8>>, ttl: u64) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(domain.to_string(), DnsRecord {
            addresses,
            ttl,
            cached_at: Instant::now(),
        });
    }

    pub fn cache_size(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    pub fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
    }
}

/// Benchmark DNS resolution operations
pub fn benchmark_dns(c: &mut Criterion, config: &BenchmarkConfig) {
    let mut group = c.benchmark_group("dns_resolution");
    
    // Benchmark basic resolution
    benchmark_basic_resolution(&mut group);
    
    // Benchmark cache performance
    benchmark_cache_performance(&mut group, config);
    
    // Benchmark resolution with different latencies
    benchmark_latency_scenarios(&mut group);
    
    // Benchmark batch resolution
    benchmark_batch_resolution(&mut group);
    
    // Benchmark concurrent resolution
    benchmark_concurrent_resolution(&mut group);
    
    // Benchmark failover scenarios
    benchmark_failover_scenarios(&mut group);
    
    group.finish();
}

fn benchmark_basic_resolution(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    let resolver = MockDnsResolver::new();
    
    // Benchmark single domain resolution
    group.bench_function("resolve_single_domain", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let domain = format!("test{}.example.com", counter);
            black_box(resolver.resolve(&domain));
        })
    });
    
    // Benchmark resolution of multiple addresses
    group.bench_function("resolve_multiple_addresses", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let domain = format!("multi{}.example.com", counter);
            black_box(resolver.resolve_all(&domain));
        })
    });
}

fn benchmark_cache_performance(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>, config: &BenchmarkConfig) {
    // Benchmark cache hit vs miss
    let resolver = MockDnsResolver::with_preloaded_cache(1000);
    
    group.bench_function("resolve_cache_hit", |b| {
        b.iter(|| {
            let idx = thread_rng().gen_range(0..1000);
            let domain = format!("cached{}.example.com", idx);
            black_box(resolver.resolve(&domain));
        })
    });
    
    group.bench_function("resolve_cache_miss", |b| {
        let mut counter = 10000;
        b.iter(|| {
            counter += 1;
            let domain = format!("uncached{}.example.com", counter);
            black_box(resolver.resolve(&domain));
        })
    });
    
    // Benchmark with different cache sizes
    for &size in &config.domain_counts {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("cache_size_performance", size),
            &size,
            |b, &size| {
                let resolver = MockDnsResolver::with_preloaded_cache(size);
                
                b.iter(|| {
                    let idx = thread_rng().gen_range(0..size);
                    let domain = format!("cached{}.example.com", idx);
                    black_box(resolver.resolve(&domain));
                })
            },
        );
    }
    
    // Benchmark cache eviction performance
    group.bench_function("cache_with_eviction", |b| {
        let resolver = MockDnsResolver::new();
        let mut counter = 0;
        
        b.iter(|| {
            counter += 1;
            let domain = format!("evict{}.example.com", counter);
            resolver.resolve(&domain);
            
            // Simulate cache pressure
            if counter % 100 == 0 {
                resolver.clear_cache();
            }
            
            black_box(counter);
        })
    });
}

fn benchmark_latency_scenarios(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    // Benchmark with different upstream latencies
    let latencies = [
        Duration::from_micros(100),   // Local network
        Duration::from_millis(10),    // Fast internet
        Duration::from_millis(50),    // Normal internet
        Duration::from_millis(200),   // Slow internet
        Duration::from_millis(1000),  // Very slow/satellite
    ];
    
    for (i, &latency) in latencies.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("upstream_latency", i),
            &latency,
            |b, &latency| {
                let resolver = MockDnsResolver::with_latency(latency);
                let mut counter = 0;
                
                b.iter(|| {
                    counter += 1;
                    let domain = format!("latency{}.example.com", counter);
                    black_box(resolver.resolve(&domain));
                })
            },
        );
    }
}

fn benchmark_batch_resolution(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    let resolver = MockDnsResolver::new();
    
    // Benchmark resolving multiple domains in sequence
    for batch_size in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("batch_resolve_sequential", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(batch_size);
                    for i in 0..batch_size {
                        let domain = format!("batch{}.example.com", i);
                        results.push(resolver.resolve(&domain));
                    }
                    black_box(results);
                })
            },
        );
    }
    
    // Benchmark parallel resolution
    use std::sync::Arc;
    use std::thread;
    
    let resolver = Arc::new(MockDnsResolver::new());
    
    group.bench_function("batch_resolve_parallel", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let resolver = Arc::clone(&resolver);
                    thread::spawn(move || {
                        let domain = format!("parallel{}.example.com", i);
                        resolver.resolve(&domain)
                    })
                })
                .collect();
            
            let results: Vec<_> = handles.into_iter()
                .map(|h| h.join().unwrap())
                .collect();
            
            black_box(results);
        })
    });
}

fn benchmark_concurrent_resolution(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    use std::sync::Arc;
    use std::thread;
    
    let resolver = Arc::new(MockDnsResolver::with_preloaded_cache(1000));
    
    // Benchmark with different thread counts
    for thread_count in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_threads", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|i| {
                            let resolver = Arc::clone(&resolver);
                            thread::spawn(move || {
                                for j in 0..10 {
                                    let idx = (i * 10 + j) % 1000;
                                    let domain = format!("concurrent{}.example.com", idx);
                                    black_box(resolver.resolve(&domain));
                                }
                            })
                        })
                        .collect();
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                })
            },
        );
    }
    
    // Benchmark read/write contention
    group.bench_function("concurrent_cache_contention", |b| {
        let resolver = Arc::new(MockDnsResolver::new());
        
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let resolver = Arc::clone(&resolver);
                    thread::spawn(move || {
                        if i % 3 == 0 {
                            // Cache miss (write to cache)
                            let domain = format!("new{}-{}.example.com", i, thread_rng().gen::<u32>());
                            resolver.resolve(&domain);
                        } else {
                            // Attempt cache hit (read from cache)
                            let domain = format!("cached{}.example.com", i % 100);
                            resolver.resolve(&domain);
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

fn benchmark_failover_scenarios(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    let resolver = MockDnsResolver::new();
    let fallback_servers = ["8.8.8.8", "1.1.1.1", "208.67.222.222"];
    
    // Benchmark resolution with fallback
    group.bench_function("resolve_with_fallback", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let domain = format!("fallback{}.example.com", counter);
            black_box(resolver.resolve_with_fallback(&domain, &fallback_servers));
        })
    });
    
    // Benchmark cascading failures
    group.bench_function("cascading_failure_recovery", |b| {
        b.iter(|| {
            // Simulate primary failure, then fallback success
            let domain = "unreliable.example.com";
            
            // Try primary (simulate failure)
            let start = Instant::now();
            let result = resolver.resolve_with_fallback(domain, &fallback_servers);
            let duration = start.elapsed();
            
            black_box((result, duration));
        })
    });
    
    // Benchmark DNS-over-HTTPS simulation
    group.bench_function("dns_over_https_simulation", |b| {
        b.iter(|| {
            // Simulate DoH overhead
            std::thread::sleep(Duration::from_micros(500)); // TLS overhead
            
            let domain = "doh.example.com";
            let result = resolver.resolve(domain);
            
            black_box(result);
        })
    });
}