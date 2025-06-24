//! Benchmarks for QuDAG vault operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_vault_core::{utils::CharacterSet, Vault};
use tempfile::TempDir;

fn vault_creation(c: &mut Criterion) {
    c.bench_function("vault_create", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let vault_path = temp_dir.path().join("vault.qdag");
            let vault = Vault::create(black_box(&vault_path), black_box("test_password"));
            black_box(vault);
        });
    });
}

fn secret_operations(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().join("vault.qdag");
    let mut vault = Vault::create(&vault_path, "test_password").unwrap();

    c.bench_function("add_secret", |b| {
        let mut counter = 0;
        b.iter(|| {
            let label = format!("test/secret{}", counter);
            counter += 1;
            vault
                .add_secret(
                    black_box(&label),
                    black_box("user@example.com"),
                    black_box(Some("password123")),
                )
                .unwrap();
        });
    });
}

fn get_secret_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().join("vault.qdag");
    let mut vault = Vault::create(&vault_path, "test_password").unwrap();

    // Add some secrets
    for i in 0..100 {
        vault
            .add_secret(
                &format!("test/secret{}", i),
                "user@example.com",
                Some("password123"),
            )
            .unwrap();
    }

    c.bench_function("get_secret", |b| {
        b.iter(|| {
            let secret = vault.get_secret(black_box("test/secret50")).unwrap();
            black_box(secret);
        });
    });
}

fn list_secrets_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().join("vault.qdag");
    let mut vault = Vault::create(&vault_path, "test_password").unwrap();

    // Add many secrets in different categories
    for category in ["email", "social", "work", "personal", "banking"] {
        for i in 0..20 {
            vault
                .add_secret(
                    &format!("{}/entry{}", category, i),
                    "user@example.com",
                    Some("password123"),
                )
                .unwrap();
        }
    }

    c.bench_function("list_all_secrets", |b| {
        b.iter(|| {
            let secrets = vault.list_secrets(black_box(None)).unwrap();
            black_box(secrets);
        });
    });

    c.bench_function("list_category_secrets", |b| {
        b.iter(|| {
            let secrets = vault.list_secrets(black_box(Some("email"))).unwrap();
            black_box(secrets);
        });
    });
}

fn password_generation(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().join("vault.qdag");
    let vault = Vault::create(&vault_path, "test_password").unwrap();

    c.bench_function("generate_password_alphanumeric", |b| {
        b.iter(|| {
            let password =
                vault.generate_password(black_box(16), black_box(CharacterSet::Alphanumeric));
            black_box(password);
        });
    });

    c.bench_function("generate_password_all_chars", |b| {
        b.iter(|| {
            let password = vault.generate_password(black_box(24), black_box(CharacterSet::All));
            black_box(password);
        });
    });
}

criterion_group!(
    benches,
    vault_creation,
    secret_operations,
    get_secret_benchmark,
    list_secrets_benchmark,
    password_generation
);
criterion_main!(benches);
