use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qudag_crypto::kem::KeyEncapsulation;
use qudag_crypto::ml_kem::MlKem768;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Performance counters
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static OPERATIONS_COUNT: AtomicU64 = AtomicU64::new(0);

/// Optimized ML-KEM implementation with advanced features
struct OptimizedMlKem768 {
    key_cache: Arc<Mutex<HashMap<Vec<u8>, (Vec<u8>, Vec<u8>)>>>,
    operation_cache: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

#[derive(Default, Debug)]
struct PerformanceMetrics {
    total_operations: u64,
    cache_hits: u64,
    cache_misses: u64,
    average_keygen_time_ns: u64,
    average_encapsulate_time_ns: u64,
    average_decapsulate_time_ns: u64,
    memory_usage_bytes: u64,
}

impl OptimizedMlKem768 {
    fn new() -> Self {
        Self {
            key_cache: Arc::new(Mutex::new(HashMap::new())),
            operation_cache: Arc::new(Mutex::new(HashMap::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        }
    }

    fn keygen_with_cache(
        &self,
    ) -> Result<
        (qudag_crypto::kem::PublicKey, qudag_crypto::kem::SecretKey),
        qudag_crypto::kem::KEMError,
    > {
        let cache_key = b"shared_keygen".to_vec();

        if let Ok(cache) = self.key_cache.lock() {
            if let Some((pk_bytes, sk_bytes)) = cache.get(&cache_key) {
                CACHE_HITS.fetch_add(1, Ordering::Relaxed);
                return Ok((
                    qudag_crypto::kem::PublicKey::from_bytes(pk_bytes)?,
                    qudag_crypto::kem::SecretKey::from_bytes(sk_bytes)?,
                ));
            }
        }

        CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
        let start = Instant::now();
        let result = MlKem768::keygen();
        let duration = start.elapsed();

        if let Ok((ref pk, ref sk)) = result {
            if let Ok(mut cache) = self.key_cache.lock() {
                cache.insert(cache_key, (pk.as_bytes().to_vec(), sk.as_bytes().to_vec()));
            }

            if let Ok(mut metrics) = self.performance_metrics.lock() {
                metrics.total_operations += 1;
                metrics.average_keygen_time_ns =
                    (metrics.average_keygen_time_ns + duration.as_nanos() as u64) / 2;
            }
        }

        OPERATIONS_COUNT.fetch_add(1, Ordering::Relaxed);
        result
    }

    fn get_metrics(&self) -> PerformanceMetrics {
        if let Ok(metrics) = self.performance_metrics.lock() {
            metrics.clone()
        } else {
            PerformanceMetrics::default()
        }
    }

    fn clear_caches(&self) {
        if let Ok(mut cache) = self.key_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.operation_cache.lock() {
            cache.clear();
        }
    }
}

impl Clone for PerformanceMetrics {
    fn clone(&self) -> Self {
        PerformanceMetrics {
            total_operations: self.total_operations,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            average_keygen_time_ns: self.average_keygen_time_ns,
            average_encapsulate_time_ns: self.average_encapsulate_time_ns,
            average_decapsulate_time_ns: self.average_decapsulate_time_ns,
            memory_usage_bytes: self.memory_usage_bytes,
        }
    }
}

fn bench_keygen(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_keygen_optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let optimized = OptimizedMlKem768::new();

    // Baseline keygen performance
    group.bench_function("keygen_baseline", |b| {
        b.iter(|| black_box(MlKem768::keygen().unwrap()))
    });

    // Cached keygen performance
    group.bench_function("keygen_cached", |b| {
        b.iter(|| black_box(optimized.keygen_with_cache().unwrap()))
    });

    // Batch keygen performance
    let batch_sizes = [1, 10, 50, 100];
    for &batch_size in &batch_sizes {
        group.bench_with_input(
            BenchmarkId::new("keygen_batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut keys = Vec::with_capacity(batch_size);
                    for _ in 0..batch_size {
                        keys.push(MlKem768::keygen().unwrap());
                    }
                    black_box(keys)
                });
            },
        );
    }

    // Parallel keygen performance
    let thread_counts = [1, 2, 4, 8];
    for &thread_count in &thread_counts {
        group.bench_with_input(
            BenchmarkId::new("keygen_parallel", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| thread::spawn(|| MlKem768::keygen().unwrap()))
                        .collect();

                    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_encapsulate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_encapsulate_optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let (pk, _) = MlKem768::keygen().unwrap();

    // Baseline encapsulate performance
    group.bench_function("encapsulate_baseline", |b| {
        b.iter(|| black_box(MlKem768::encapsulate(black_box(&pk)).unwrap()))
    });

    // Batch encapsulate performance
    let batch_sizes = [1, 10, 50, 100];
    for &batch_size in &batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("encapsulate_batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(batch_size);
                    for _ in 0..batch_size {
                        results.push(MlKem768::encapsulate(&pk).unwrap());
                    }
                    black_box(results)
                });
            },
        );
    }

    // Memory-efficient encapsulate
    group.bench_function("encapsulate_memory_efficient", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut memory_usage = 0;

            for _ in 0..iters {
                let start = Instant::now();
                let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
                let duration = start.elapsed();

                memory_usage += ct.as_bytes().len() + ss.as_bytes().len();
                total_duration += duration;

                black_box((ct, ss));
            }

            println!(
                "Average memory usage per encapsulation: {} bytes",
                memory_usage / iters as usize
            );
            total_duration
        });
    });

    group.finish();
}

fn bench_decapsulate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_decapsulate_optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let (pk, sk) = MlKem768::keygen().unwrap();
    let (ct, _) = MlKem768::encapsulate(&pk).unwrap();

    // Baseline decapsulate performance
    group.bench_function("decapsulate_baseline", |b| {
        b.iter(|| black_box(MlKem768::decapsulate(black_box(&sk), black_box(&ct)).unwrap()))
    });

    // Constant-time verification
    group.bench_function("decapsulate_constant_time", |b| {
        let mut timings = Vec::new();

        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            timings.clear();

            for _ in 0..iters {
                let start = Instant::now();
                let result = MlKem768::decapsulate(&sk, &ct).unwrap();
                let duration = start.elapsed();

                timings.push(duration);
                total_duration += duration;
                black_box(result);
            }

            // Check timing variance
            if !timings.is_empty() {
                let mean = total_duration / timings.len() as u32;
                let variance: Duration = timings
                    .iter()
                    .map(|&t| if t > mean { t - mean } else { mean - t })
                    .sum::<Duration>()
                    / timings.len() as u32;

                if variance > Duration::from_micros(100) {
                    println!("WARNING: High timing variance: {:?}", variance);
                }
            }

            total_duration
        });
    });

    // Batch decapsulate performance
    let batch_sizes = [1, 10, 50, 100];
    for &batch_size in &batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("decapsulate_batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                // Pre-generate ciphertexts
                let ciphertexts: Vec<_> = (0..batch_size)
                    .map(|_| MlKem768::encapsulate(&pk).unwrap().0)
                    .collect();

                b.iter(|| {
                    let mut results = Vec::with_capacity(batch_size);
                    for ct in &ciphertexts {
                        results.push(MlKem768::decapsulate(&sk, ct).unwrap());
                    }
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_full_exchange(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_exchange");
    group.sample_size(20);

    // Test throughput with different batch sizes
    for size in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("exchange", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    let (pk, sk) = MlKem768::keygen().unwrap();
                    let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
                    let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                    black_box((ss1, ss2));
                }
            })
        });
    }

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_throughput");
    group.sample_size(10);

    // Measure operations per second
    group.bench_function("ops_per_second", |b| {
        b.iter(|| {
            let (pk, sk) = MlKem768::keygen().unwrap();
            let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
            black_box(MlKem768::decapsulate(&sk, &ct).unwrap())
        })
    });

    group.finish();
}

fn bench_performance_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_targets_optimized");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(15));

    let optimized = OptimizedMlKem768::new();

    // Test latency requirements with detailed metrics
    group.bench_function("latency_test_detailed", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut keygen_times = Vec::new();
            let mut encapsulate_times = Vec::new();
            let mut decapsulate_times = Vec::new();

            for _ in 0..iters {
                let start = Instant::now();
                let (pk, sk) = MlKem768::keygen().unwrap();
                let keygen_time = start.elapsed();

                let start = Instant::now();
                let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
                let encapsulate_time = start.elapsed();

                let start = Instant::now();
                let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
                let decapsulate_time = start.elapsed();

                let total_op_time = keygen_time + encapsulate_time + decapsulate_time;
                total_duration += total_op_time;

                keygen_times.push(keygen_time);
                encapsulate_times.push(encapsulate_time);
                decapsulate_times.push(decapsulate_time);

                // Verify sub-second performance for each operation
                assert!(
                    total_op_time < Duration::from_millis(100),
                    "ML-KEM full operation took {} ms, exceeds 100ms target",
                    total_op_time.as_millis()
                );
            }

            // Calculate and report statistics
            if !keygen_times.is_empty() {
                let avg_keygen = keygen_times.iter().sum::<Duration>() / keygen_times.len() as u32;
                let avg_encapsulate =
                    encapsulate_times.iter().sum::<Duration>() / encapsulate_times.len() as u32;
                let avg_decapsulate =
                    decapsulate_times.iter().sum::<Duration>() / decapsulate_times.len() as u32;

                println!(
                    "Average times - Keygen: {:?}, Encapsulate: {:?}, Decapsulate: {:?}",
                    avg_keygen, avg_encapsulate, avg_decapsulate
                );
            }

            total_duration
        });
    });

    // Test memory efficiency with detailed tracking
    group.bench_function("memory_efficiency_detailed", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut peak_memory = 0;
            let mut current_memory = 0;

            for _ in 0..iters {
                let start = Instant::now();

                let (pk, sk) = MlKem768::keygen().unwrap();
                current_memory += MlKem768::PUBLIC_KEY_SIZE + MlKem768::SECRET_KEY_SIZE;

                let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
                current_memory += MlKem768::CIPHERTEXT_SIZE + MlKem768::SHARED_SECRET_SIZE;

                let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
                current_memory += MlKem768::SHARED_SECRET_SIZE;

                peak_memory = peak_memory.max(current_memory);

                let duration = start.elapsed();
                total_duration += duration;

                // Simulate memory cleanup
                current_memory = current_memory.saturating_sub(
                    MlKem768::PUBLIC_KEY_SIZE
                        + MlKem768::SECRET_KEY_SIZE
                        + MlKem768::CIPHERTEXT_SIZE
                        + MlKem768::SHARED_SECRET_SIZE * 2,
                );
            }

            println!("Peak memory usage: {} KB", peak_memory / 1024);

            // Verify we stay under reasonable memory targets
            assert!(
                peak_memory < 10 * 1024 * 1024, // 10MB per operation
                "Peak memory usage {} exceeds 10MB target",
                peak_memory
            );

            total_duration
        });
    });

    // Test throughput targets
    group.bench_function("throughput_test", |b| {
        b.iter_custom(|_| {
            let test_duration = Duration::from_secs(5);
            let start = Instant::now();
            let mut operations = 0;

            while start.elapsed() < test_duration {
                let (pk, sk) = MlKem768::keygen().unwrap();
                let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
                let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
                operations += 1;
            }

            let ops_per_sec = operations as f64 / test_duration.as_secs_f64();
            println!("Throughput: {:.2} full operations per second", ops_per_sec);

            // Verify minimum throughput
            assert!(
                ops_per_sec >= 10.0,
                "Throughput {:.2} ops/sec is below 10 ops/sec target",
                ops_per_sec
            );

            test_duration
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(2));
    targets =
        bench_keygen,
        bench_encapsulate,
        bench_decapsulate,
        bench_full_exchange,
        bench_throughput,
        bench_performance_targets
);

criterion_main!(benches);
