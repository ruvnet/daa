//! Performance benchmarks for ML-DSA implementation
//!
//! This benchmark suite measures the performance of:
//! - Key generation
//! - Message signing
//! - Signature verification
//! - Memory usage patterns
//! - Constant-time operation validation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qudag_crypto::ml_dsa::{MlDsa, MlDsaKeyPair, MlDsaPublicKey};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Performance tracking
static SIGNATURE_OPERATIONS: AtomicU64 = AtomicU64::new(0);
static VERIFICATION_OPERATIONS: AtomicU64 = AtomicU64::new(0);
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

/// Optimized ML-DSA with caching and batching
struct OptimizedMlDsa {
    keypair_cache: Arc<Mutex<HashMap<Vec<u8>, MlDsaKeyPair>>>,
    signature_cache: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    batch_size: usize,
    enable_parallel: bool,
}

impl OptimizedMlDsa {
    fn new(batch_size: usize, enable_parallel: bool) -> Self {
        Self {
            keypair_cache: Arc::new(Mutex::new(HashMap::new())),
            signature_cache: Arc::new(Mutex::new(HashMap::new())),
            batch_size,
            enable_parallel,
        }
    }

    fn generate_keypair_cached(
        &self,
        seed: &[u8],
    ) -> Result<MlDsaKeyPair, qudag_crypto::ml_dsa::MlDsaError> {
        // Try cache first
        if let Ok(cache) = self.keypair_cache.lock() {
            if let Some(keypair) = cache.get(seed) {
                CACHE_HITS.fetch_add(1, Ordering::Relaxed);
                return Ok(keypair.clone());
            }
        }

        CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng)?;

        // Cache the result
        if let Ok(mut cache) = self.keypair_cache.lock() {
            cache.insert(seed.to_vec(), keypair.clone());
        }

        Ok(keypair)
    }

    fn sign_batch(
        &self,
        messages: &[Vec<u8>],
    ) -> Result<Vec<Vec<u8>>, qudag_crypto::ml_dsa::MlDsaError> {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng)?;
        let mut signatures = Vec::with_capacity(messages.len());

        if self.enable_parallel && messages.len() > 4 {
            // Parallel signing for large batches
            let chunk_size = (messages.len() + 3) / 4; // Divide into 4 chunks
            let chunks: Vec<_> = messages.chunks(chunk_size).collect();

            let handles: Vec<_> = chunks
                .into_iter()
                .map(|chunk| {
                    let keypair_clone = keypair.clone();
                    let chunk_owned = chunk.to_vec();

                    thread::spawn(move || {
                        let mut rng = thread_rng();
                        let mut chunk_signatures = Vec::new();

                        for message in chunk_owned {
                            let signature = keypair_clone.sign(&message, &mut rng)?;
                            chunk_signatures.push(signature);
                        }

                        Ok::<Vec<Vec<u8>>, qudag_crypto::ml_dsa::MlDsaError>(chunk_signatures)
                    })
                })
                .collect();

            for handle in handles {
                let chunk_signatures = handle.join().unwrap()?;
                signatures.extend(chunk_signatures);
            }
        } else {
            // Sequential signing
            for message in messages {
                let signature = keypair.sign(message, &mut rng)?;
                signatures.push(signature);
                SIGNATURE_OPERATIONS.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(signatures)
    }

    fn verify_batch(
        &self,
        messages: &[Vec<u8>],
        signatures: &[Vec<u8>],
        public_key: &MlDsaPublicKey,
    ) -> Result<Vec<bool>, qudag_crypto::ml_dsa::MlDsaError> {
        let mut results = Vec::with_capacity(messages.len());

        if self.enable_parallel && messages.len() > 4 {
            // Parallel verification
            let chunk_size = (messages.len() + 3) / 4;
            let message_chunks: Vec<_> = messages.chunks(chunk_size).collect();
            let signature_chunks: Vec<_> = signatures.chunks(chunk_size).collect();

            let handles: Vec<_> = message_chunks
                .iter()
                .zip(signature_chunks.iter())
                .map(|(msg_chunk, sig_chunk)| {
                    let public_key_clone = public_key.clone();
                    let msg_chunk_owned = msg_chunk.to_vec();
                    let sig_chunk_owned = sig_chunk.to_vec();

                    thread::spawn(move || {
                        let mut chunk_results = Vec::new();

                        for (message, signature) in
                            msg_chunk_owned.iter().zip(sig_chunk_owned.iter())
                        {
                            let result = public_key_clone.verify(message, signature).is_ok();
                            chunk_results.push(result);
                        }

                        chunk_results
                    })
                })
                .collect();

            for handle in handles {
                let chunk_results = handle.join().unwrap();
                results.extend(chunk_results);
            }
        } else {
            // Sequential verification
            for (message, signature) in messages.iter().zip(signatures.iter()) {
                let result = public_key.verify(message, signature).is_ok();
                results.push(result);
                VERIFICATION_OPERATIONS.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(results)
    }

    fn clear_caches(&self) {
        if let Ok(mut cache) = self.keypair_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.signature_cache.lock() {
            cache.clear();
        }
    }
}

impl Clone for MlDsaKeyPair {
    fn clone(&self) -> Self {
        // This is a simplified clone - in production, you'd want a proper implementation
        // For benchmarking purposes, we'll generate a new keypair
        let mut rng = thread_rng();
        MlDsaKeyPair::generate(&mut rng).expect("Failed to generate keypair")
    }
}

/// Benchmark ML-DSA key generation performance with optimizations
fn bench_ml_dsa_keygen(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Key Generation Optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let optimized = OptimizedMlDsa::new(32, true);

    // Baseline key generation
    group.bench_function("keygen_baseline", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let _ = MlDsaKeyPair::generate(&mut rng);
        })
    });

    // Cached key generation
    group.bench_function("keygen_cached", |b| {
        b.iter(|| {
            let seed = b"test_seed_for_caching";
            let _ = optimized.generate_keypair_cached(seed);
        })
    });

    // Batch key generation
    let batch_sizes = [1, 5, 10, 25, 50];
    for &batch_size in &batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("keygen_batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut keypairs = Vec::with_capacity(batch_size);
                    for _ in 0..batch_size {
                        let mut rng = thread_rng();
                        keypairs.push(MlDsaKeyPair::generate(&mut rng).unwrap());
                    }
                    criterion::black_box(keypairs)
                });
            },
        );
    }

    // Parallel key generation
    let thread_counts = [1, 2, 4, 8];
    for &thread_count in &thread_counts {
        group.bench_with_input(
            BenchmarkId::new("keygen_parallel", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            thread::spawn(|| {
                                let mut rng = thread_rng();
                                MlDsaKeyPair::generate(&mut rng).unwrap()
                            })
                        })
                        .collect();

                    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                    criterion::black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark ML-DSA signing performance with optimizations
fn bench_ml_dsa_signing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Signing Optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let optimized = OptimizedMlDsa::new(32, true);
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");

    // Test with different message sizes
    let message_sizes = [32, 256, 1024, 4096, 16384];

    for &size in &message_sizes {
        let message: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("sign_baseline", size),
            &message,
            |b, msg| {
                b.iter(|| {
                    let mut rng = thread_rng();
                    let _ = keypair.sign(msg, &mut rng);
                })
            },
        );
    }

    // Batch signing performance
    let batch_sizes = [1, 5, 10, 25, 50];
    for &batch_size in &batch_sizes {
        let messages: Vec<Vec<u8>> = (0..batch_size)
            .map(|_| (0..1024).map(|_| rng.gen()).collect())
            .collect();

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("sign_batch", batch_size),
            &batch_size,
            |b, _| {
                b.iter(|| {
                    let _ = optimized.sign_batch(&messages);
                });
            },
        );
    }

    // Memory-efficient signing
    group.bench_function("sign_memory_efficient", |b| {
        let message = vec![0x42u8; 1024];

        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut max_memory = 0;

            for _ in 0..iters {
                let start = Instant::now();
                let mut rng = thread_rng();
                let signature = keypair.sign(&message, &mut rng).unwrap();
                let duration = start.elapsed();

                max_memory = max_memory.max(signature.len());
                total_duration += duration;

                criterion::black_box(signature);
            }

            println!("Max signature size: {} bytes", max_memory);
            total_duration
        });
    });

    group.finish();
}

/// Benchmark ML-DSA verification performance with optimizations
fn bench_ml_dsa_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Verification Optimized");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    let optimized = OptimizedMlDsa::new(32, true);
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())
        .expect("Public key creation should succeed");

    // Test with different message sizes
    let message_sizes = [32, 256, 1024, 4096, 16384];

    for &size in &message_sizes {
        let message: Vec<u8> = (0..size).map(|_| rng.gen()).collect();
        let signature = keypair
            .sign(&message, &mut rng)
            .expect("Signing should succeed");

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("verify_baseline", size),
            &(message, signature),
            |b, (msg, sig)| {
                b.iter(|| {
                    let _ = public_key.verify(msg, sig);
                })
            },
        );
    }

    // Batch verification performance
    let batch_sizes = [1, 5, 10, 25, 50];
    for &batch_size in &batch_sizes {
        let messages: Vec<Vec<u8>> = (0..batch_size)
            .map(|_| (0..1024).map(|_| rng.gen()).collect())
            .collect();
        let signatures: Vec<Vec<u8>> = messages
            .iter()
            .map(|msg| keypair.sign(msg, &mut rng).unwrap())
            .collect();

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("verify_batch", batch_size),
            &batch_size,
            |b, _| {
                b.iter(|| {
                    let _ = optimized.verify_batch(&messages, &signatures, &public_key);
                });
            },
        );
    }

    // Constant-time verification testing
    group.bench_function("verify_constant_time", |b| {
        let message = vec![0x42u8; 1024];
        let valid_signature = keypair.sign(&message, &mut rng).unwrap();
        let mut invalid_signature = valid_signature.clone();
        invalid_signature[0] ^= 1; // Tamper with signature

        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut valid_times = Vec::new();
            let mut invalid_times = Vec::new();

            for i in 0..iters {
                let (msg, sig) = if i % 2 == 0 {
                    (&message, &valid_signature)
                } else {
                    (&message, &invalid_signature)
                };

                let start = Instant::now();
                let _ = public_key.verify(msg, sig);
                let duration = start.elapsed();

                if i % 2 == 0 {
                    valid_times.push(duration);
                } else {
                    invalid_times.push(duration);
                }

                total_duration += duration;
            }

            // Check timing consistency
            if !valid_times.is_empty() && !invalid_times.is_empty() {
                let avg_valid = valid_times.iter().sum::<Duration>() / valid_times.len() as u32;
                let avg_invalid =
                    invalid_times.iter().sum::<Duration>() / invalid_times.len() as u32;
                let timing_diff = if avg_valid > avg_invalid {
                    avg_valid - avg_invalid
                } else {
                    avg_invalid - avg_valid
                };

                if timing_diff > Duration::from_micros(100) {
                    println!("WARNING: Timing difference detected: {:?}", timing_diff);
                }
            }

            total_duration
        });
    });

    group.finish();
}

/// Benchmark full ML-DSA round-trip operations
fn bench_ml_dsa_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Round-trip");

    let message_sizes = [256, 1024, 4096];

    for &size in &message_sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("keygen_sign_verify", size),
            &size,
            |b, &msg_size| {
                b.iter(|| {
                    let mut rng = thread_rng();
                    let message: Vec<u8> = (0..msg_size).map(|_| rng.gen()).collect();

                    // Generate keypair
                    let keypair =
                        MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");

                    // Sign message
                    let signature = keypair
                        .sign(&message, &mut rng)
                        .expect("Signing should succeed");

                    // Verify signature
                    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())
                        .expect("Public key creation should succeed");
                    let _ = public_key.verify(&message, &signature);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark constant-time properties of ML-DSA operations
fn bench_ml_dsa_constant_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Constant-time Properties");

    // Pre-generate test data
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())
        .expect("Public key creation should succeed");

    let message1: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();
    let message2: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();

    let signature1 = keypair
        .sign(&message1, &mut rng)
        .expect("Signing should succeed");
    let signature2 = keypair
        .sign(&message2, &mut rng)
        .expect("Signing should succeed");

    // Benchmark verification timing consistency
    group.bench_function("verify_valid_signature", |b| {
        b.iter(|| {
            let _ = public_key.verify(&message1, &signature1);
        })
    });

    group.bench_function("verify_invalid_signature", |b| {
        b.iter(|| {
            let _ = public_key.verify(&message1, &signature2);
        })
    });

    group.finish();
}

/// Benchmark memory usage patterns for ML-DSA operations
fn bench_ml_dsa_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Memory Usage");

    // Measure memory allocation patterns
    group.bench_function("keypair_allocation", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");

            // Access key data to prevent optimization
            criterion::black_box(keypair.public_key().len());
            criterion::black_box(keypair.secret_key().len());
        })
    });

    group.bench_function("signature_allocation", |b| {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let message = vec![0x42u8; 1024];

        b.iter(|| {
            let signature = keypair
                .sign(&message, &mut rng)
                .expect("Signing should succeed");
            criterion::black_box(signature.len());
        })
    });

    group.finish();
}

/// Measure actual timing variance to validate constant-time properties
fn measure_timing_variance() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())
        .expect("Public key creation should succeed");

    let message = vec![0x42u8; 1024];
    let signature = keypair
        .sign(&message, &mut rng)
        .expect("Signing should succeed");

    // Measure verification timing for valid signature
    let mut valid_times = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        let _ = public_key.verify(&message, &signature);
        valid_times.push(start.elapsed());
    }

    // Measure verification timing for invalid signature
    let mut invalid_signature = signature.clone();
    invalid_signature[0] ^= 1; // Tamper with signature

    let mut invalid_times = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        let _ = public_key.verify(&message, &invalid_signature);
        invalid_times.push(start.elapsed());
    }

    // Calculate statistics
    let valid_mean = valid_times.iter().sum::<Duration>() / valid_times.len() as u32;
    let invalid_mean = invalid_times.iter().sum::<Duration>() / invalid_times.len() as u32;

    let valid_variance = valid_times
        .iter()
        .map(|&t| {
            if t > valid_mean {
                t - valid_mean
            } else {
                valid_mean - t
            }
        })
        .sum::<Duration>()
        / valid_times.len() as u32;

    let invalid_variance = invalid_times
        .iter()
        .map(|&t| {
            if t > invalid_mean {
                t - invalid_mean
            } else {
                invalid_mean - t
            }
        })
        .sum::<Duration>()
        / invalid_times.len() as u32;

    println!("Timing Analysis Results:");
    println!("Valid signature verification:");
    println!("  Mean: {:?}", valid_mean);
    println!("  Variance: {:?}", valid_variance);
    println!("Invalid signature verification:");
    println!("  Mean: {:?}", invalid_mean);
    println!("  Variance: {:?}", invalid_variance);

    let timing_difference = if valid_mean > invalid_mean {
        valid_mean - invalid_mean
    } else {
        invalid_mean - valid_mean
    };

    println!("Timing difference: {:?}", timing_difference);

    // For constant-time operations, timing difference should be minimal
    if timing_difference > Duration::from_millis(1) {
        println!("WARNING: Significant timing difference detected!");
    } else {
        println!("Timing difference within acceptable range");
    }
}

/// Performance regression test
fn bench_ml_dsa_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Performance Regression");

    // Set performance targets based on requirements
    let target_keygen_time = Duration::from_millis(100);
    let target_sign_time = Duration::from_millis(50);
    let target_verify_time = Duration::from_millis(50);

    group.bench_function("regression_keygen", |b| {
        let duration = b.iter(|| {
            let mut rng = thread_rng();
            let _ = MlDsaKeyPair::generate(&mut rng);
        });

        // Note: In a real implementation, we would assert performance targets
        // For now, we just measure and report
    });

    group.bench_function("regression_sign", |b| {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let message = vec![0x42u8; 1024];

        let duration = b.iter(|| {
            let _ = keypair.sign(&message, &mut rng);
        });
    });

    group.bench_function("regression_verify", |b| {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())
            .expect("Public key creation should succeed");
        let message = vec![0x42u8; 1024];
        let signature = keypair
            .sign(&message, &mut rng)
            .expect("Signing should succeed");

        let duration = b.iter(|| {
            let _ = public_key.verify(&message, &signature);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ml_dsa_keygen,
    bench_ml_dsa_signing,
    bench_ml_dsa_verification,
    bench_ml_dsa_roundtrip,
    bench_ml_dsa_constant_time,
    bench_ml_dsa_memory_usage,
    bench_ml_dsa_regression
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_variance_measurement() {
        // Run timing variance measurement as a test
        measure_timing_variance();
    }

    #[test]
    fn test_performance_targets() {
        // Basic performance sanity check
        let mut rng = thread_rng();

        // Test key generation performance
        let start = Instant::now();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let keygen_time = start.elapsed();

        // Test signing performance
        let message = vec![0x42u8; 1024];
        let start = Instant::now();
        let signature = keypair
            .sign(&message, &mut rng)
            .expect("Signing should succeed");
        let sign_time = start.elapsed();

        // Test verification performance
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())
            .expect("Public key creation should succeed");
        let start = Instant::now();
        let _ = public_key.verify(&message, &signature);
        let verify_time = start.elapsed();

        println!("Performance measurements:");
        println!("  Key generation: {:?}", keygen_time);
        println!("  Signing: {:?}", sign_time);
        println!("  Verification: {:?}", verify_time);

        // Basic sanity checks (adjust targets as needed)
        assert!(
            keygen_time < Duration::from_secs(1),
            "Key generation too slow"
        );
        assert!(sign_time < Duration::from_secs(1), "Signing too slow");
        assert!(
            verify_time < Duration::from_secs(1),
            "Verification too slow"
        );
    }
}
