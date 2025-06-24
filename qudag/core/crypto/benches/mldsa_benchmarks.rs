use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qudag_crypto::ml_dsa::{MLDsa, SIGNATURE_LENGTH};
use std::alloc::{GlobalAlloc, Layout, System};
use std::time::Instant;

#[global_allocator]
static ALLOCATOR: MemoryTracker = MemoryTracker;

struct MemoryTracker;

static mut MEMORY_USAGE: usize = 0;

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        MEMORY_USAGE += layout.size();
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        MEMORY_USAGE -= layout.size();
        System.dealloc(ptr, layout)
    }
}

fn reset_memory_usage() {
    unsafe {
        MEMORY_USAGE = 0;
    }
}

fn get_memory_usage() -> usize {
    unsafe { MEMORY_USAGE }
}

fn benchmark_sign_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Sign/Verify");

    group.bench_function("keygen", |b| {
        b.iter(|| {
            black_box(MLDsa::keygen().expect("Key generation failed"));
        });
    });

    let (pk, sk) = MLDsa::keygen().expect("Key generation failed");
    let message = b"Test message for benchmarking";

    group.bench_function("sign", |b| {
        b.iter(|| {
            black_box(sk.sign(black_box(message)).expect("Signing failed"));
        });
    });

    let signature = sk.sign(message).expect("Signing failed");
    let pk_bytes = pk.public_key.expect("Missing public key");

    group.bench_function("verify", |b| {
        b.iter(|| {
            black_box(
                MLDsa::verify(
                    black_box(message),
                    black_box(&signature),
                    black_box(&pk_bytes),
                )
                .expect("Verification failed"),
            );
        });
    });

    group.finish();
}

fn benchmark_batch_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Batch Operations");

    let batch_sizes = [1, 10, 50, 100, 500];

    for size in batch_sizes.iter() {
        group.throughput(Throughput::Elements(*size as u64));

        let mut signatures = Vec::new();
        let mut messages = Vec::new();
        let mut public_keys = Vec::new();

        for i in 0..*size {
            let (pk, sk) = MLDsa::keygen().expect("Key generation failed");
            let message = format!("Test message {}", i).into_bytes();
            let signature = sk.sign(&message).expect("Signing failed");

            signatures.push(signature);
            messages.push(message);
            public_keys.push(pk.public_key.expect("Missing public key"));
        }

        group.bench_with_input(BenchmarkId::new("batch_verify", size), size, |b, _| {
            b.iter(|| {
                for i in 0..*size {
                    black_box(
                        MLDsa::verify(
                            black_box(&messages[i]),
                            black_box(&signatures[i]),
                            black_box(&public_keys[i]),
                        )
                        .expect("Verification failed"),
                    );
                }
            });
        });
    }
    group.finish();
}

fn benchmark_timing_consistency(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Timing Analysis");
    group.sample_size(1000); // Increased samples for timing analysis

    let (pk, sk) = MLDsa::keygen().expect("Key generation failed");
    let message = b"Test message for timing analysis";
    let signature = sk.sign(message).expect("Signing failed");
    let pk_bytes = pk.public_key.expect("Missing public key");

    // Measure sign timing variance
    group.bench_function("sign_timing", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);
            let mut timings = Vec::with_capacity(iters as usize);

            for _ in 0..iters {
                let start = Instant::now();
                black_box(sk.sign(black_box(message))).expect("Signing failed");
                let duration = start.elapsed();
                timings.push(duration);
                total_duration += duration;
            }

            total_duration
        });
    });

    // Measure verify timing variance
    group.bench_function("verify_timing", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);
            let mut timings = Vec::with_capacity(iters as usize);

            for _ in 0..iters {
                let start = Instant::now();
                black_box(MLDsa::verify(
                    black_box(message),
                    black_box(&signature),
                    black_box(&pk_bytes),
                ))
                .expect("Verification failed");
                let duration = start.elapsed();
                timings.push(duration);
                total_duration += duration;
            }

            total_duration
        });
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Memory Usage");

    group.bench_function("sign_memory", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);
            let (pk, sk) = MLDsa::keygen().expect("Key generation failed");
            let message = b"Test message for memory analysis";

            for _ in 0..iters {
                reset_memory_usage();
                let start = Instant::now();
                black_box(sk.sign(black_box(message))).expect("Signing failed");
                let duration = start.elapsed();
                let mem_used = get_memory_usage();
                println!("Sign Memory Usage: {} bytes", mem_used);
                total_duration += duration;
            }

            total_duration
        });
    });

    group.bench_function("verify_memory", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);
            let (pk, sk) = MLDsa::keygen().expect("Key generation failed");
            let message = b"Test message for memory analysis";
            let signature = sk.sign(message).expect("Signing failed");
            let pk_bytes = pk.public_key.expect("Missing public key");

            for _ in 0..iters {
                reset_memory_usage();
                let start = Instant::now();
                black_box(MLDsa::verify(
                    black_box(message),
                    black_box(&signature),
                    black_box(&pk_bytes),
                ))
                .expect("Verification failed");
                let duration = start.elapsed();
                let mem_used = get_memory_usage();
                println!("Verify Memory Usage: {} bytes", mem_used);
                total_duration += duration;
            }

            total_duration
        });
    });

    group.finish();
}

fn benchmark_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Throughput");

    let test_durations = [1, 5, 10]; // seconds

    for duration in test_durations.iter() {
        group.bench_with_input(
            BenchmarkId::new("sign_throughput", duration),
            duration,
            |b, &duration| {
                b.iter_custom(|_| {
                    let start = Instant::now();
                    let mut count = 0;
                    let (_, sk) = MLDsa::keygen().expect("Key generation failed");
                    let message = b"Test message for throughput";

                    while start.elapsed().as_secs() < duration {
                        black_box(sk.sign(black_box(message))).expect("Signing failed");
                        count += 1;
                    }

                    println!(
                        "Sign Throughput: {} ops/sec",
                        count as f64 / duration as f64
                    );
                    std::time::Duration::from_secs(duration)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("verify_throughput", duration),
            duration,
            |b, &duration| {
                b.iter_custom(|_| {
                    let start = Instant::now();
                    let mut count = 0;
                    let (pk, sk) = MLDsa::keygen().expect("Key generation failed");
                    let message = b"Test message for throughput";
                    let signature = sk.sign(message).expect("Signing failed");
                    let pk_bytes = pk.public_key.expect("Missing public key");

                    while start.elapsed().as_secs() < duration {
                        black_box(MLDsa::verify(
                            black_box(message),
                            black_box(&signature),
                            black_box(&pk_bytes),
                        ))
                        .expect("Verification failed");
                        count += 1;
                    }

                    println!(
                        "Verify Throughput: {} ops/sec",
                        count as f64 / duration as f64
                    );
                    std::time::Duration::from_secs(duration)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_sign_verify,
    benchmark_batch_verify,
    benchmark_timing_consistency,
    benchmark_memory_usage,
    benchmark_throughput
);
criterion_main!(benches);
