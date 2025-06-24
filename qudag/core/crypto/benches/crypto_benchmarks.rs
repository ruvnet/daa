use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_crypto::ml_kem::{KeyEncapsulation, MlKem768};

fn benchmark_mlkem(c: &mut Criterion) {
    c.bench_function("ml_kem_768_keygen", |b| {
        b.iter(|| {
            black_box(MlKem768::keygen().expect("Key generation failed"));
        });
    });

    let (pk, sk) = MlKem768::keygen().expect("Key generation failed");

    c.bench_function("ml_kem_768_encapsulate", |b| {
        b.iter(|| {
            black_box(MlKem768::encapsulate(black_box(&pk)).expect("Encapsulation failed"));
        });
    });

    let (ct, _) = MlKem768::encapsulate(&pk).expect("Encapsulation failed");

    c.bench_function("ml_kem_768_decapsulate", |b| {
        b.iter(|| {
            black_box(
                MlKem768::decapsulate(black_box(&sk), black_box(&ct))
                    .expect("Decapsulation failed"),
            );
        });
    });
}

criterion_group!(benches, benchmark_mlkem);
criterion_main!(benches);
