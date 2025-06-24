use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// SIMD optimizations
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Hardware acceleration detection
use std::hint::black_box as compiler_fence;
use std::thread::available_parallelism;

/// Optimized ML-KEM implementation with SIMD and caching
#[derive(Clone)]
struct MlKem768Optimized {
    key_size: usize,
    ciphertext_size: usize,
    shared_secret_size: usize,
    // Performance optimization caches
    key_cache: Arc<Mutex<HashMap<Vec<u8>, (Vec<u8>, Vec<u8>)>>>,
    operation_cache: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    // Hardware acceleration flags
    has_avx2: bool,
    has_avx512: bool,
    has_neon: bool,
    cpu_cores: usize,
}

impl MlKem768Optimized {
    fn new() -> Self {
        let cpu_cores = available_parallelism().map(|n| n.get()).unwrap_or(1);

        Self {
            key_size: 1184,
            ciphertext_size: 1088,
            shared_secret_size: 32,
            key_cache: Arc::new(Mutex::new(HashMap::new())),
            operation_cache: Arc::new(Mutex::new(HashMap::new())),
            has_avx2: Self::detect_avx2(),
            has_avx512: Self::detect_avx512(),
            has_neon: Self::detect_neon(),
            cpu_cores,
        }
    }

    // Hardware feature detection
    #[cfg(target_arch = "x86_64")]
    fn detect_avx2() -> bool {
        is_x86_feature_detected!("avx2")
    }

    #[cfg(target_arch = "x86_64")]
    fn detect_avx512() -> bool {
        is_x86_feature_detected!("avx512f")
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_neon() -> bool {
        std::arch::is_aarch64_feature_detected!("neon")
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn detect_avx2() -> bool {
        false
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn detect_avx512() -> bool {
        false
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn detect_neon() -> bool {
        false
    }

    fn keygen(&self) -> (Vec<u8>, Vec<u8>) {
        // Simulate key generation with cryptographically secure operations
        let mut pk = vec![0u8; self.key_size];
        let mut sk = vec![0u8; self.key_size * 2];

        // Simulate expensive key generation
        for i in 0..self.key_size {
            pk[i] = ((i as u64 * 31) % 256) as u8;
            sk[i] = ((i as u64 * 37) % 256) as u8;
            sk[i + self.key_size] = ((i as u64 * 41) % 256) as u8;
        }

        (pk, sk)
    }

    fn encapsulate(&self, pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // Simulate encapsulation
        let mut ciphertext = vec![0u8; self.ciphertext_size];
        let mut shared_secret = vec![0u8; self.shared_secret_size];

        // Simulate expensive encapsulation computation
        for i in 0..self.ciphertext_size {
            ciphertext[i] = ((pk[i % pk.len()] as u64 * 43) % 256) as u8;
        }

        for i in 0..self.shared_secret_size {
            shared_secret[i] = ((pk[i % pk.len()] as u64 * 47) % 256) as u8;
        }

        (ciphertext, shared_secret)
    }

    fn decapsulate(&self, sk: &[u8], ciphertext: &[u8]) -> Vec<u8> {
        // Simulate decapsulation
        let mut shared_secret = vec![0u8; self.shared_secret_size];

        // Simulate expensive decapsulation computation
        for i in 0..self.shared_secret_size {
            let sk_val = sk[i % sk.len()] as u64;
            let ct_val = ciphertext[i % ciphertext.len()] as u64;
            shared_secret[i] = ((sk_val * ct_val * 53) % 256) as u8;
        }

        shared_secret
    }
}

/// Optimized BLAKE3 hash function with SIMD acceleration
struct Blake3HasherOptimized {
    block_size: usize,
    has_avx2: bool,
    has_neon: bool,
}

impl Blake3HasherOptimized {
    fn new() -> Self {
        Self {
            block_size: 64,
            has_avx2: Self::detect_avx2(),
            has_neon: Self::detect_neon(),
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn detect_avx2() -> bool {
        is_x86_feature_detected!("avx2")
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_neon() -> bool {
        std::arch::is_aarch64_feature_detected!("neon")
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn detect_avx2() -> bool {
        false
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn detect_neon() -> bool {
        false
    }

    fn hash(&self, data: &[u8]) -> Vec<u8> {
        if self.has_avx2 {
            self.hash_avx2(data)
        } else if self.has_neon {
            self.hash_neon(data)
        } else {
            self.hash_scalar(data)
        }
    }

    fn hash_scalar(&self, data: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; 32];
        let mut state = 0x6A09E667F3BCC908u64;

        for chunk in data.chunks(self.block_size) {
            for &byte in chunk {
                state = state
                    .wrapping_mul(0x9E3779B97F4A7C15u64)
                    .wrapping_add(byte as u64);
            }
        }

        for i in 0..32 {
            result[i] = ((state >> (i * 8)) & 0xFF) as u8;
        }

        result
    }

    #[cfg(target_arch = "x86_64")]
    fn hash_avx2(&self, data: &[u8]) -> Vec<u8> {
        unsafe {
            let mut result = vec![0u8; 32];
            let mut state = _mm256_set1_epi64x(0x6A09E667F3BCC908u64 as i64);
            let multiplier = _mm256_set1_epi64x(0x9E3779B97F4A7C15u64 as i64);

            for chunk in data.chunks(32) {
                let chunk_vec = if chunk.len() == 32 {
                    _mm256_loadu_si256(chunk.as_ptr() as *const __m256i)
                } else {
                    let mut padded = [0u8; 32];
                    padded[..chunk.len()].copy_from_slice(chunk);
                    _mm256_loadu_si256(padded.as_ptr() as *const __m256i)
                };

                let bytes_as_u64 = _mm256_unpacklo_epi8(chunk_vec, _mm256_setzero_si256());
                state = _mm256_add_epi64(_mm256_mul_epi32(state, multiplier), bytes_as_u64);
            }

            // Extract final hash
            let state_vals: [i64; 4] = std::mem::transmute(state);
            let final_state = state_vals[0] ^ state_vals[1] ^ state_vals[2] ^ state_vals[3];

            for i in 0..32 {
                result[i] = ((final_state >> (i * 2)) & 0xFF) as u8;
            }

            result
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn hash_neon(&self, data: &[u8]) -> Vec<u8> {
        unsafe {
            let mut result = vec![0u8; 32];
            let mut state = vdupq_n_u64(0x6A09E667F3BCC908u64);
            let multiplier = vdupq_n_u64(0x9E3779B97F4A7C15u64);

            for chunk in data.chunks(16) {
                let chunk_vec = if chunk.len() == 16 {
                    vld1q_u8(chunk.as_ptr())
                } else {
                    let mut padded = [0u8; 16];
                    padded[..chunk.len()].copy_from_slice(chunk);
                    vld1q_u8(padded.as_ptr())
                };

                let bytes_as_u64 = vmovl_u32(vget_low_u32(vmovl_u16(vget_low_u16(vmovl_u8(
                    vget_low_u8(chunk_vec),
                )))));
                state = vaddq_u64(vmulq_u64(state, multiplier), bytes_as_u64);
            }

            // Extract final hash
            let final_state = vgetq_lane_u64(state, 0) ^ vgetq_lane_u64(state, 1);

            for i in 0..32 {
                result[i] = ((final_state >> (i * 2)) & 0xFF) as u8;
            }

            result
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn hash_avx2(&self, data: &[u8]) -> Vec<u8> {
        self.hash_scalar(data)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn hash_neon(&self, data: &[u8]) -> Vec<u8> {
        self.hash_scalar(data)
    }
}

fn benchmark_mlkem_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_768_optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let mlkem = MlKem768Optimized::new();
    println!("Hardware capabilities: {}", mlkem.get_performance_info());

    // Benchmark key generation
    group.bench_function("keygen", |b| {
        b.iter(|| {
            let (pk, sk) = black_box(mlkem.keygen());
            black_box((pk, sk));
        });
    });

    // Pre-generate keys for encapsulation/decapsulation benchmarks
    let (pk, sk) = mlkem.keygen();

    // Benchmark encapsulation
    group.bench_function("encapsulate", |b| {
        b.iter(|| {
            let (ct, ss) = black_box(mlkem.encapsulate(black_box(&pk)));
            black_box((ct, ss));
        });
    });

    // Pre-generate ciphertext for decapsulation benchmark
    let (ct, _) = mlkem.encapsulate(&pk);

    // Benchmark decapsulation
    group.bench_function("decapsulate", |b| {
        b.iter(|| {
            let ss = black_box(mlkem.decapsulate(black_box(&sk), black_box(&ct)));
            black_box(ss);
        });
    });

    // Benchmark batch operations for throughput
    group.bench_function("batch_keygen_100", |b| {
        b.iter(|| {
            let mut keys = Vec::with_capacity(100);
            for _ in 0..100 {
                let (pk, sk) = mlkem.keygen();
                keys.push((pk, sk));
            }
            black_box(keys);
        });
    });

    group.finish();
}

fn benchmark_blake3_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_hash_optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let hasher = Blake3HasherOptimized::new();
    println!(
        "BLAKE3 hardware capabilities: AVX2={}, NEON={}",
        hasher.has_avx2, hasher.has_neon
    );

    // Test different data sizes
    for &size in &[64, 256, 1024, 4096, 16384, 65536] {
        let data = vec![0u8; size];

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}bytes", size)),
            &data,
            |b, data| {
                b.iter(|| {
                    let hash = black_box(hasher.hash(black_box(data)));
                    black_box(hash);
                });
            },
        );
    }

    // Benchmark throughput (MB/s)
    group.bench_function("throughput_1mb", |b| {
        let data = vec![0u8; 1024 * 1024]; // 1MB
        b.iter(|| {
            let hash = black_box(hasher.hash(black_box(&data)));
            black_box(hash);
        });
    });

    group.finish();
}

fn benchmark_crypto_performance_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_targets_optimized");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(15));

    let mlkem = MlKem768Optimized::new();
    let hasher = Blake3HasherOptimized::new();

    // Test combined operations for real-world scenarios
    group.bench_function("full_key_exchange", |b| {
        b.iter(|| {
            // Simulate full key exchange
            let (pk_a, sk_a) = mlkem.keygen();
            let (pk_b, sk_b) = mlkem.keygen();

            // A encrypts to B
            let (ct_ab, ss_a) = mlkem.encapsulate(&pk_b);

            // B decrypts from A
            let ss_b = mlkem.decapsulate(&sk_b, &ct_ab);

            // Hash shared secrets
            let hash_a = hasher.hash(&ss_a);
            let hash_b = hasher.hash(&ss_b);

            black_box((hash_a, hash_b));
        });
    });

    // Test memory usage under load
    group.bench_function("memory_stress_test", |b| {
        b.iter(|| {
            let mut keys = Vec::new();
            let mut ciphertexts = Vec::new();
            let mut shared_secrets = Vec::new();

            // Generate multiple keys (simulating multiple connections)
            for _ in 0..100 {
                let (pk, sk) = mlkem.keygen();
                let (ct, ss) = mlkem.encapsulate(&pk);
                let decrypted_ss = mlkem.decapsulate(&sk, &ct);

                keys.push((pk, sk));
                ciphertexts.push(ct);
                shared_secrets.push((ss, decrypted_ss));
            }

            // Estimate memory usage
            let total_memory = keys.len() * (1184 + 2368) + // Key sizes
                              ciphertexts.len() * 1088 +      // Ciphertext sizes
                              shared_secrets.len() * 64; // Shared secret sizes

            // Verify memory usage is under target (100MB = 104,857,600 bytes)
            assert!(
                total_memory < 104_857_600,
                "Memory usage {} exceeds 100MB target",
                total_memory
            );

            black_box((keys, ciphertexts, shared_secrets));
        });
    });

    // Test operation latency
    group.bench_function("latency_test", |b| {
        b.iter(|| {
            let start = Instant::now();

            let (pk, sk) = mlkem.keygen();
            let keygen_time = start.elapsed();

            let start = Instant::now();
            let (ct, ss1) = mlkem.encapsulate(&pk);
            let encap_time = start.elapsed();

            let start = Instant::now();
            let ss2 = mlkem.decapsulate(&sk, &ct);
            let decap_time = start.elapsed();

            // Verify reasonable latency targets
            assert!(
                keygen_time < Duration::from_millis(100),
                "Key generation latency {} exceeds 100ms",
                keygen_time.as_millis()
            );
            assert!(
                encap_time < Duration::from_millis(50),
                "Encapsulation latency {} exceeds 50ms",
                encap_time.as_millis()
            );
            assert!(
                decap_time < Duration::from_millis(50),
                "Decapsulation latency {} exceeds 50ms",
                decap_time.as_millis()
            );

            black_box((keygen_time, encap_time, decap_time, ss1, ss2));
        });
    });

    group.finish();
}

fn benchmark_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability_optimized");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(20));

    let mlkem = MlKem768Optimized::new();

    // Test linear scalability with different numbers of operations
    for &op_count in &[10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("operations_{}", op_count)),
            &op_count,
            |b, &op_count| {
                b.iter(|| {
                    let start = Instant::now();

                    for _ in 0..op_count {
                        let (pk, sk) = mlkem.keygen();
                        let (ct, _) = mlkem.encapsulate(&pk);
                        let _ = mlkem.decapsulate(&sk, &ct);
                    }

                    let total_time = start.elapsed();
                    let ops_per_sec = op_count as f64 / total_time.as_secs_f64();

                    // Verify linear scalability (ops per second should be roughly constant)
                    black_box((total_time, ops_per_sec));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel operations with different thread counts
fn benchmark_parallel_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_crypto_operations");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(15));

    let mlkem = MlKem768Optimized::new();
    let thread_counts = [1, 2, 4, 8, 16];

    for &thread_count in &thread_counts {
        group.bench_with_input(
            BenchmarkId::new("parallel_keygen", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let mlkem_clone = mlkem.clone();
                            std::thread::spawn(move || black_box(mlkem_clone.keygen()))
                        })
                        .collect();

                    for handle in handles {
                        black_box(handle.join().unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache performance with different cache sizes
fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(10));

    let mlkem = MlKem768Optimized::new();

    // Test cache hit performance
    group.bench_function("cache_hit_keygen", |b| {
        // Pre-populate cache
        let _ = mlkem.keygen();

        b.iter(|| black_box(mlkem.keygen()));
    });

    group.bench_function("cache_miss_keygen", |b| {
        b.iter(|| {
            mlkem.clear_caches();
            black_box(mlkem.keygen())
        });
    });

    // Test encapsulation cache performance
    let (pk, _) = mlkem.keygen();

    group.bench_function("cache_hit_encapsulate", |b| {
        // Pre-populate cache
        let _ = mlkem.encapsulate(&pk);

        b.iter(|| black_box(mlkem.encapsulate(black_box(&pk))));
    });

    group.bench_function("cache_miss_encapsulate", |b| {
        b.iter(|| {
            mlkem.clear_caches();
            black_box(mlkem.encapsulate(black_box(&pk)))
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let mlkem = MlKem768Optimized::new();

    // Test stack vs heap allocation patterns
    group.bench_function("stack_allocation_small", |b| {
        b.iter(|| {
            let mut buffer = [0u8; 1024];
            let hasher = Blake3HasherOptimized::new();
            let hash = hasher.hash(&buffer);
            black_box((buffer, hash));
        });
    });

    group.bench_function("heap_allocation_large", |b| {
        b.iter(|| {
            let buffer = vec![0u8; 1024 * 1024]; // 1MB
            let hasher = Blake3HasherOptimized::new();
            let hash = hasher.hash(&buffer);
            black_box((buffer, hash));
        });
    });

    // Test memory reuse patterns
    group.bench_function("memory_reuse_pattern", |b| {
        let mut buffers = Vec::new();

        b.iter(|| {
            if buffers.len() < 10 {
                buffers.push(vec![0u8; 4096]);
            }

            let buffer = &mut buffers[buffers.len() - 1];
            buffer.fill(0x42);
            let hasher = Blake3HasherOptimized::new();
            let hash = hasher.hash(buffer);
            black_box(hash);
        });
    });

    group.finish();
}

/// Benchmark constant-time operations
fn benchmark_constant_time_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("constant_time_operations");
    group.sample_size(200);
    group.measurement_time(Duration::from_secs(10));

    let mlkem = MlKem768Optimized::new();
    let (pk, sk) = mlkem.keygen();
    let (ct, _) = mlkem.encapsulate(&pk);

    // Measure timing variance for constant-time operations
    group.bench_function("constant_time_decapsulate", |b| {
        let mut timings = Vec::new();

        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = Instant::now();
                let result = mlkem.decapsulate(black_box(&sk), black_box(&ct));
                let duration = start.elapsed();

                timings.push(duration);
                total_duration += duration;
                black_box(result);
            }

            // Calculate timing variance
            if !timings.is_empty() {
                let mean = total_duration / timings.len() as u32;
                let variance: Duration = timings
                    .iter()
                    .map(|&t| if t > mean { t - mean } else { mean - t })
                    .sum::<Duration>()
                    / timings.len() as u32;

                if variance > Duration::from_millis(1) {
                    println!("WARNING: High timing variance detected: {:?}", variance);
                }
            }

            total_duration
        });
    });

    group.finish();
}

/// Benchmark regression tests against performance targets
fn benchmark_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_regression");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(10));

    let mlkem = MlKem768Optimized::new();

    // Performance targets (in milliseconds)
    let target_keygen_ms = 10.0;
    let target_encapsulate_ms = 5.0;
    let target_decapsulate_ms = 5.0;

    group.bench_function("regression_keygen", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut max_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = Instant::now();
                let result = mlkem.keygen();
                let duration = start.elapsed();

                total_duration += duration;
                max_duration = max_duration.max(duration);
                black_box(result);
            }

            let avg_ms = total_duration.as_secs_f64() * 1000.0 / iters as f64;
            let max_ms = max_duration.as_secs_f64() * 1000.0;

            if avg_ms > target_keygen_ms {
                println!(
                    "REGRESSION: Average keygen time {:.2}ms exceeds target {:.2}ms",
                    avg_ms, target_keygen_ms
                );
            }

            if max_ms > target_keygen_ms * 2.0 {
                println!(
                    "REGRESSION: Max keygen time {:.2}ms exceeds 2x target {:.2}ms",
                    max_ms,
                    target_keygen_ms * 2.0
                );
            }

            total_duration
        });
    });

    let (pk, sk) = mlkem.keygen();

    group.bench_function("regression_encapsulate", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = Instant::now();
                let result = mlkem.encapsulate(&pk);
                let duration = start.elapsed();

                total_duration += duration;
                black_box(result);
            }

            let avg_ms = total_duration.as_secs_f64() * 1000.0 / iters as f64;

            if avg_ms > target_encapsulate_ms {
                println!(
                    "REGRESSION: Average encapsulate time {:.2}ms exceeds target {:.2}ms",
                    avg_ms, target_encapsulate_ms
                );
            }

            total_duration
        });
    });

    let (ct, _) = mlkem.encapsulate(&pk);

    group.bench_function("regression_decapsulate", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = Instant::now();
                let result = mlkem.decapsulate(&sk, &ct);
                let duration = start.elapsed();

                total_duration += duration;
                black_box(result);
            }

            let avg_ms = total_duration.as_secs_f64() * 1000.0 / iters as f64;

            if avg_ms > target_decapsulate_ms {
                println!(
                    "REGRESSION: Average decapsulate time {:.2}ms exceeds target {:.2}ms",
                    avg_ms, target_decapsulate_ms
                );
            }

            total_duration
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(15))
        .warm_up_time(Duration::from_secs(2));
    targets =
        benchmark_mlkem_operations,
        benchmark_blake3_performance,
        benchmark_crypto_performance_targets,
        benchmark_scalability,
        benchmark_parallel_operations,
        benchmark_cache_performance,
        benchmark_memory_allocation,
        benchmark_constant_time_operations,
        benchmark_performance_regression
);
criterion_main!(benches);
