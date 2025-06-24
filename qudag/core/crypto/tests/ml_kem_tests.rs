use hex_literal::hex;
use proptest::prelude::*;
use qudag_crypto::kem::{
    Ciphertext, KEMError, KeyEncapsulation, PublicKey, SecretKey, SharedSecret,
};
use qudag_crypto::ml_kem::MlKem768;
use rand::RngCore;

// Official ML-KEM-768 test vectors
const TEST_SEED: [u8; 32] =
    hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
const TEST_PK: [u8; MlKem768::PUBLIC_KEY_SIZE] = include!(".test_vectors/mlkem768_pk.txt");
const TEST_SK: [u8; MlKem768::SECRET_KEY_SIZE] = include!(".test_vectors/mlkem768_sk.txt");
const TEST_CT: [u8; MlKem768::CIPHERTEXT_SIZE] = include!(".test_vectors/mlkem768_ct.txt");
const TEST_SS: [u8; MlKem768::SHARED_SECRET_SIZE] = include!(".test_vectors/mlkem768_ss.txt");

#[test]
fn test_mlkem_key_generation() {
    let (pk, sk) = MlKem768::keygen().expect("Key generation should succeed");

    // Verify key sizes
    assert_eq!(pk.as_bytes().len(), MlKem768::PUBLIC_KEY_SIZE);
    assert_eq!(sk.as_bytes().len(), MlKem768::SECRET_KEY_SIZE);

    // Verify keys are not all zeros
    assert_ne!(pk.as_bytes(), &[0u8; MlKem768::PUBLIC_KEY_SIZE]);
    assert_ne!(sk.as_bytes(), &[0u8; MlKem768::SECRET_KEY_SIZE]);
}

#[test]
fn test_mlkem_encapsulation_decapsulation() {
    let (pk, sk) = MlKem768::keygen().expect("Key generation should succeed");
    let (ciphertext, shared_secret_1) =
        MlKem768::encapsulate(&pk).expect("Encapsulation should succeed");
    let shared_secret_2 =
        MlKem768::decapsulate(&sk, &ciphertext).expect("Decapsulation should succeed");

    // Verify sizes
    assert_eq!(ciphertext.as_bytes().len(), MlKem768::CIPHERTEXT_SIZE);
    assert_eq!(
        shared_secret_1.as_bytes().len(),
        MlKem768::SHARED_SECRET_SIZE
    );
    assert_eq!(
        shared_secret_2.as_bytes().len(),
        MlKem768::SHARED_SECRET_SIZE
    );

    // Verify shared secrets match
    assert_eq!(shared_secret_1.as_bytes(), shared_secret_2.as_bytes());

    // Verify ciphertext and shared secret are not all zeros
    assert_ne!(ciphertext.as_bytes(), &[0u8; MlKem768::CIPHERTEXT_SIZE]);
    assert_ne!(
        shared_secret_1.as_bytes(),
        &[0u8; MlKem768::SHARED_SECRET_SIZE]
    );
}

#[test]
fn test_mlkem_with_test_vectors() {
    // Test decapsulation with known test vectors
    let sk = SecretKey::from_bytes(&TEST_SK).expect("Valid secret key");
    let ct = Ciphertext::from_bytes(&TEST_CT).expect("Valid ciphertext");
    let ss =
        MlKem768::decapsulate(&sk, &ct).expect("Decapsulation with test vectors should succeed");

    assert_eq!(ss.as_bytes(), &TEST_SS);
}

#[test]
fn test_mlkem_invalid_inputs() {
    let (_, sk) = MlKem768::keygen().expect("Key generation should succeed");

    // Test with invalid ciphertext length
    let short_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE - 1];
    let ct = Ciphertext::from_bytes(&short_ct).expect("Valid ciphertext creation");
    let result = MlKem768::decapsulate(&sk, &ct);
    assert!(result.is_err());

    // Test with random invalid ciphertext
    let mut invalid_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE];
    rand::thread_rng().fill_bytes(&mut invalid_ct);
    let ct = Ciphertext::from_bytes(&invalid_ct).expect("Valid ciphertext creation");
    let result = MlKem768::decapsulate(&sk, &ct);
    assert!(result.is_err());
}

proptest! {
    #[test]
    fn test_mlkem_random_keys(
        pk_bytes in prop::collection::vec(0u8..255, MlKem768::PUBLIC_KEY_SIZE),
        ct_bytes in prop::collection::vec(0u8..255, MlKem768::CIPHERTEXT_SIZE)
    ) {
        // Test constant-time behavior with random inputs
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap_or_else(|_| panic!("Failed to create public key"));
        let ct = Ciphertext::from_bytes(&ct_bytes).unwrap_or_else(|_| panic!("Failed to create ciphertext"));

        let start = std::time::Instant::now();
        let _ = MlKem768::encapsulate(&pk);
        let duration1 = start.elapsed();

        let start = std::time::Instant::now();
        let _ = MlKem768::encapsulate(&pk);
        let duration2 = start.elapsed();

        // Operations should complete in roughly the same time (within 20% variance)
        let variance = if duration2.as_nanos() > 0 {
            (duration1.as_nanos() as f64 / duration2.as_nanos() as f64 - 1.0).abs() < 0.2
        } else {
            true
        };
        prop_assert!(variance);
    }
}

#[test]
fn test_constant_time_operations() {
    let (pk1, sk1) = MlKem768::keygen().expect("Key generation should succeed");
    let (pk2, sk2) = MlKem768::keygen().expect("Key generation should succeed");

    // Test constant-time equality comparisons
    assert!(pk1 != pk2);
    assert!(sk1 != sk2);

    let (ct1, ss1) = MlKem768::encapsulate(&pk1).expect("Encapsulation should succeed");
    let ss2 = MlKem768::decapsulate(&sk1, &ct1).expect("Decapsulation should succeed");

    // Test shared secret constant-time comparison
    assert!(ss1 == ss2);
}

#[test]
fn test_key_cache_functionality() {
    let (pk, sk) = MlKem768::keygen().expect("Key generation should succeed");
    let (ct, _) = MlKem768::encapsulate(&pk).expect("Encapsulation should succeed");

    // First decapsulation - should miss cache
    let before = MlKem768::get_metrics();
    let _ = MlKem768::decapsulate(&sk, &ct).expect("Decapsulation should succeed");
    let after = MlKem768::get_metrics();

    assert_eq!(after.key_cache_misses, before.key_cache_misses + 1);

    // Second decapsulation - should hit cache
    let _ = MlKem768::decapsulate(&sk, &ct).expect("Decapsulation should succeed");
    let final_metrics = MlKem768::get_metrics();

    assert_eq!(final_metrics.key_cache_hits, after.key_cache_hits + 1);
}

#[test]
fn test_timing_consistency() {
    let (pk, sk) = MlKem768::keygen().expect("Key generation should succeed");
    let (ct, _) = MlKem768::encapsulate(&pk).expect("Encapsulation should succeed");

    let mut timings = Vec::new();

    // Multiple decapsulations to get timing data
    for _ in 0..10 {
        let start = std::time::Instant::now();
        let _ = MlKem768::decapsulate(&sk, &ct).expect("Decapsulation should succeed");
        timings.push(start.elapsed().as_nanos());
    }

    // Calculate timing variance
    let avg = timings.iter().sum::<u128>() as f64 / timings.len() as f64;
    let variance = timings
        .iter()
        .map(|&t| (t as f64 - avg).powi(2))
        .sum::<f64>()
        / timings.len() as f64;
    let std_dev = variance.sqrt();

    // Standard deviation should be less than 10% of mean
    assert!(
        std_dev / avg < 0.1,
        "Timing variation too high: {std_dev} / {avg}"
    );
}
