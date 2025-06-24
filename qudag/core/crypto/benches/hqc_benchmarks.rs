use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_crypto::hqc::{Hqc, Hqc256, SecurityParameter};
use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn benchmark_hqc_keygen(c: &mut Criterion) {
    c.bench_function("hqc128_keygen", |b| {
        b.iter(|| {
            let mut rng = ChaCha20Rng::from_entropy();
            let hqc = Hqc::new(SecurityParameter::Hqc128);
            black_box(
                hqc.generate_keypair(&mut rng)
                    .expect("Key generation failed"),
            );
        });
    });

    c.bench_function("hqc192_keygen", |b| {
        b.iter(|| {
            let mut rng = ChaCha20Rng::from_entropy();
            let hqc = Hqc::new(SecurityParameter::Hqc192);
            black_box(
                hqc.generate_keypair(&mut rng)
                    .expect("Key generation failed"),
            );
        });
    });

    c.bench_function("hqc256_keygen", |b| {
        b.iter(|| {
            let mut rng = ChaCha20Rng::from_entropy();
            let hqc = Hqc::new(SecurityParameter::Hqc256);
            black_box(
                hqc.generate_keypair(&mut rng)
                    .expect("Key generation failed"),
            );
        });
    });

    // Benchmark using the compatibility interface
    c.bench_function("hqc256_keygen_compat", |b| {
        b.iter(|| {
            black_box(Hqc256::keygen().expect("Key generation failed"));
        });
    });
}

fn benchmark_hqc_encryption(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_entropy();

    // HQC128 benchmarks
    let hqc128 = Hqc::new(SecurityParameter::Hqc128);
    let (pk128, sk128) = hqc128
        .generate_keypair(&mut rng)
        .expect("Key generation failed");
    let message128 = vec![0x42u8; 16]; // 128-bit message

    c.bench_function("hqc128_encrypt", |b| {
        b.iter(|| {
            let mut rng = ChaCha20Rng::from_entropy();
            black_box(
                hqc128
                    .encrypt(black_box(&message128), &pk128, &mut rng)
                    .expect("Encryption failed"),
            );
        });
    });

    let ct128 = hqc128
        .encrypt(&message128, &pk128, &mut rng)
        .expect("Encryption failed");
    c.bench_function("hqc128_decrypt", |b| {
        b.iter(|| {
            black_box(
                hqc128
                    .decrypt(black_box(&ct128), &sk128)
                    .expect("Decryption failed"),
            );
        });
    });

    // HQC256 benchmarks
    let hqc256 = Hqc::new(SecurityParameter::Hqc256);
    let (pk256, sk256) = hqc256
        .generate_keypair(&mut rng)
        .expect("Key generation failed");
    let message256 = vec![0x42u8; 32]; // 256-bit message

    c.bench_function("hqc256_encrypt", |b| {
        b.iter(|| {
            let mut rng = ChaCha20Rng::from_entropy();
            black_box(
                hqc256
                    .encrypt(black_box(&message256), &pk256, &mut rng)
                    .expect("Encryption failed"),
            );
        });
    });

    let ct256 = hqc256
        .encrypt(&message256, &pk256, &mut rng)
        .expect("Encryption failed");
    c.bench_function("hqc256_decrypt", |b| {
        b.iter(|| {
            black_box(
                hqc256
                    .decrypt(black_box(&ct256), &sk256)
                    .expect("Decryption failed"),
            );
        });
    });
}

fn benchmark_hqc_compatibility(c: &mut Criterion) {
    // Test the compatibility interface with existing tests
    let (pk, sk) = Hqc256::keygen().expect("Key generation failed");
    let message =
        b"Test message for HQC256 benchmarking - this is a longer message to test performance";

    c.bench_function("hqc256_compat_encrypt", |b| {
        b.iter(|| {
            black_box(
                Hqc256::encrypt(black_box(&pk), black_box(message)).expect("Encryption failed"),
            );
        });
    });

    let ciphertext = Hqc256::encrypt(&pk, message).expect("Encryption failed");
    c.bench_function("hqc256_compat_decrypt", |b| {
        b.iter(|| {
            black_box(
                Hqc256::decrypt(black_box(&sk), black_box(&ciphertext)).expect("Decryption failed"),
            );
        });
    });
}

fn benchmark_hqc_polynomial_ops(c: &mut Criterion) {
    let hqc = Hqc::new(SecurityParameter::Hqc128);
    let byte_len = (hqc.params.n + 7) / 8;

    let a = vec![0xAA; byte_len];
    let b = vec![0x55; byte_len];
    let c_vec = vec![0xFF; byte_len];

    c.bench_function("hqc128_poly_mult_add", |b| {
        b.iter(|| {
            black_box(
                hqc.poly_mult_add(black_box(&a), black_box(&b), black_box(&c_vec))
                    .expect("Polynomial operation failed"),
            );
        });
    });

    c.bench_function("hqc128_bytes_to_bits", |b| {
        b.iter(|| {
            black_box(hqc.bytes_to_bits(black_box(&a)));
        });
    });

    let bits = hqc.bytes_to_bits(&a);
    c.bench_function("hqc128_bits_to_bytes", |b| {
        b.iter(|| {
            black_box(hqc.bits_to_bytes(black_box(&bits)));
        });
    });
}

criterion_group!(
    benches,
    benchmark_hqc_keygen,
    benchmark_hqc_encryption,
    benchmark_hqc_compatibility,
    benchmark_hqc_polynomial_ops
);
criterion_main!(benches);
