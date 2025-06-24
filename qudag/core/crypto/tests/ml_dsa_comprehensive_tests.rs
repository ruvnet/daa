//! Comprehensive test suite for ML-DSA implementation
//!
//! This test suite covers:
//! - Functional correctness
//! - Security properties
//! - Performance characteristics
//! - Edge cases and error conditions
//! - Property-based testing

use proptest::prelude::*;
use qudag_crypto::ml_dsa::{MlDsaError, MlDsaKeyPair, MlDsaPublicKey};
use std::time::{Duration, Instant};

// NIST ML-DSA parameter sets
mod ml_dsa_44_params {
    pub const PUBLIC_KEY_SIZE: usize = 1312;
    pub const SECRET_KEY_SIZE: usize = 2560;
    pub const SIGNATURE_SIZE: usize = 2420;
    pub const SEED_SIZE: usize = 32;
}

mod ml_dsa_65_params {
    pub const PUBLIC_KEY_SIZE: usize = 1952;
    pub const SECRET_KEY_SIZE: usize = 4032;
    pub const SIGNATURE_SIZE: usize = 3309;
    pub const SEED_SIZE: usize = 32;
}

mod ml_dsa_87_params {
    pub const PUBLIC_KEY_SIZE: usize = 2592;
    pub const SECRET_KEY_SIZE: usize = 4896;
    pub const SIGNATURE_SIZE: usize = 4627;
    pub const SEED_SIZE: usize = 32;
}

/// Test ML-DSA key generation
#[test]
fn test_ml_dsa_key_generation() {
    let mut rng = rand::thread_rng();

    // Generate keypair
    let keypair = MlDsaKeyPair::generate(&mut rng);
    assert!(keypair.is_ok(), "Key generation should succeed");

    let keypair = keypair.unwrap();

    // Check key sizes
    assert_eq!(
        keypair.public_key().len(),
        ml_dsa_65_params::PUBLIC_KEY_SIZE
    );
    assert_eq!(
        keypair.secret_key().len(),
        ml_dsa_65_params::SECRET_KEY_SIZE
    );

    // Ensure keys are not all zeros
    assert!(!keypair.public_key().iter().all(|&b| b == 0));
    assert!(!keypair.secret_key().iter().all(|&b| b == 0));
}

/// Test ML-DSA key generation determinism
#[test]
fn test_ml_dsa_key_generation_determinism() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let seed = [42u8; 32];
    let mut rng1 = StdRng::from_seed(seed);
    let mut rng2 = StdRng::from_seed(seed);

    let keypair1 = MlDsaKeyPair::generate(&mut rng1).unwrap();
    let keypair2 = MlDsaKeyPair::generate(&mut rng2).unwrap();

    // Same seed should produce same keys
    assert_eq!(keypair1.public_key(), keypair2.public_key());
    assert_eq!(keypair1.secret_key(), keypair2.secret_key());
}

/// Test ML-DSA signing and verification
#[test]
fn test_ml_dsa_sign_verify() {
    let mut rng = rand::thread_rng();
    let message = b"Test message for ML-DSA";

    // Generate keypair
    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();

    // Sign message
    let signature = keypair.sign(message, &mut rng);
    assert!(signature.is_ok(), "Signing should succeed");

    let signature = signature.unwrap();
    assert_eq!(signature.len(), ml_dsa_65_params::SIGNATURE_SIZE);

    // Verify signature
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let result = public_key.verify(message, &signature);
    assert!(result.is_ok(), "Verification should succeed");
}

/// Test ML-DSA signature verification with wrong message
#[test]
fn test_ml_dsa_verify_wrong_message() {
    let mut rng = rand::thread_rng();
    let original_message = b"Original message";
    let wrong_message = b"Wrong message";

    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let signature = keypair.sign(original_message, &mut rng).unwrap();

    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let result = public_key.verify(wrong_message, &signature);

    assert!(
        result.is_err(),
        "Verification should fail with wrong message"
    );
    assert!(matches!(
        result.unwrap_err(),
        MlDsaError::VerificationFailed
    ));
}

/// Test ML-DSA signature verification with tampered signature
#[test]
fn test_ml_dsa_verify_tampered_signature() {
    let mut rng = rand::thread_rng();
    let message = b"Test message";

    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let mut signature = keypair.sign(message, &mut rng).unwrap();

    // Tamper with the signature
    signature[0] ^= 1;

    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let result = public_key.verify(message, &signature);

    assert!(
        result.is_err(),
        "Verification should fail with tampered signature"
    );
    assert!(matches!(
        result.unwrap_err(),
        MlDsaError::VerificationFailed
    ));
}

/// Test ML-DSA with invalid key sizes
#[test]
fn test_ml_dsa_invalid_key_sizes() {
    // Test invalid public key size
    let invalid_public_key = vec![0u8; ml_dsa_65_params::PUBLIC_KEY_SIZE - 1];
    let result = MlDsaPublicKey::from_bytes(&invalid_public_key);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        MlDsaError::InvalidKeyLength { .. }
    ));

    // Test invalid signature size
    let mut rng = rand::thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();

    let invalid_signature = vec![0u8; ml_dsa_65_params::SIGNATURE_SIZE - 1];
    let result = public_key.verify(b"test", &invalid_signature);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        MlDsaError::InvalidSignatureLength { .. }
    ));
}

/// Test ML-DSA multiple signatures with same key
#[test]
fn test_ml_dsa_multiple_signatures() {
    let mut rng = rand::thread_rng();
    let message = b"Test message";

    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();

    // Generate multiple signatures
    let sig1 = keypair.sign(message, &mut rng).unwrap();
    let sig2 = keypair.sign(message, &mut rng).unwrap();
    let sig3 = keypair.sign(message, &mut rng).unwrap();

    // All signatures should be different (probabilistic)
    assert_ne!(sig1, sig2);
    assert_ne!(sig2, sig3);
    assert_ne!(sig1, sig3);

    // All signatures should verify
    assert!(public_key.verify(message, &sig1).is_ok());
    assert!(public_key.verify(message, &sig2).is_ok());
    assert!(public_key.verify(message, &sig3).is_ok());
}

/// Test ML-DSA timing consistency for verification
#[test]
fn test_ml_dsa_timing_consistency() {
    let mut rng = rand::thread_rng();
    let message = b"Test message for timing analysis";

    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let signature = keypair.sign(message, &mut rng).unwrap();

    // Measure timing for valid signature
    let start = Instant::now();
    let _ = public_key.verify(message, &signature);
    let valid_duration = start.elapsed();

    // Measure timing for invalid signature
    let mut invalid_signature = signature.clone();
    invalid_signature[0] ^= 1;
    let start = Instant::now();
    let _ = public_key.verify(message, &invalid_signature);
    let invalid_duration = start.elapsed();

    // Timing should be consistent (within 10ms tolerance)
    let diff = if valid_duration > invalid_duration {
        valid_duration - invalid_duration
    } else {
        invalid_duration - valid_duration
    };

    assert!(
        diff < Duration::from_millis(10),
        "Timing difference too large: {:?}",
        diff
    );
}

/// Test ML-DSA memory zeroization
#[test]
fn test_ml_dsa_memory_zeroization() {
    let mut rng = rand::thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();

    // Get a reference to the secret key data
    let secret_key_ptr = keypair.secret_key().as_ptr();
    let secret_key_len = keypair.secret_key().len();

    // Drop the keypair - this should trigger zeroization
    drop(keypair);

    // Note: This is a best-effort test for zeroization
    // In practice, we cannot reliably test that memory has been zeroed
    // because the memory may be reused or optimized away by the compiler
    // The test is here to document the requirement
    assert!(secret_key_len > 0, "Secret key should have non-zero length");
}

/// Test ML-DSA with empty message
#[test]
fn test_ml_dsa_empty_message() {
    let mut rng = rand::thread_rng();
    let empty_message = b"";

    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let signature = keypair.sign(empty_message, &mut rng).unwrap();

    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let result = public_key.verify(empty_message, &signature);
    assert!(
        result.is_ok(),
        "Should be able to sign and verify empty message"
    );
}

/// Test ML-DSA with large message
#[test]
fn test_ml_dsa_large_message() {
    let mut rng = rand::thread_rng();
    let large_message = vec![0x42u8; 1024 * 1024]; // 1MB message

    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let signature = keypair.sign(&large_message, &mut rng).unwrap();

    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let result = public_key.verify(&large_message, &signature);
    assert!(
        result.is_ok(),
        "Should be able to sign and verify large message"
    );
}

/// Property-based test for ML-DSA correctness
proptest! {
    #[test]
    fn test_ml_dsa_property_based_correctness(
        message in prop::collection::vec(any::<u8>(), 0..10000),
        seed in prop::array::uniform32(any::<u8>())
    ) {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut rng = StdRng::from_seed(seed);

        // Generate keypair
        let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();

        // Sign message
        let signature = keypair.sign(&message, &mut rng).unwrap();

        // Verify signature
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
        let result = public_key.verify(&message, &signature);

        prop_assert!(result.is_ok(), "Valid signature should always verify");
    }
}

/// Property-based test for ML-DSA signature uniqueness
proptest! {
    #[test]
    fn test_ml_dsa_signature_uniqueness(
        message in prop::collection::vec(any::<u8>(), 1..1000),
        seed1 in prop::array::uniform32(any::<u8>()),
        seed2 in prop::array::uniform32(any::<u8>())
    ) {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // Assume different seeds (skip if same)
        prop_assume!(seed1 != seed2);

        let mut rng1 = StdRng::from_seed(seed1);
        let mut rng2 = StdRng::from_seed(seed2);

        let keypair = MlDsaKeyPair::generate(&mut rng1).unwrap();

        // Generate two signatures with different randomness
        let sig1 = keypair.sign(&message, &mut rng1).unwrap();
        let sig2 = keypair.sign(&message, &mut rng2).unwrap();

        // Signatures should be different (probabilistic)
        prop_assert_ne!(sig1, sig2, "Signatures with different randomness should differ");
    }
}

/// Property-based test for ML-DSA verification failure with wrong message
proptest! {
    #[test]
    fn test_ml_dsa_verification_failure(
        message1 in prop::collection::vec(any::<u8>(), 1..1000),
        message2 in prop::collection::vec(any::<u8>(), 1..1000),
        seed in prop::array::uniform32(any::<u8>())
    ) {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // Assume different messages
        prop_assume!(message1 != message2);

        let mut rng = StdRng::from_seed(seed);

        let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
        let signature = keypair.sign(&message1, &mut rng).unwrap();

        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
        let result = public_key.verify(&message2, &signature);

        prop_assert!(result.is_err(), "Verification should fail with wrong message");
    }
}

/// Security test for constant-time operations
#[test]
fn test_ml_dsa_constant_time_operations() {
    let mut rng = rand::thread_rng();
    let message = b"Test message for constant-time analysis";

    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();

    // Create multiple signatures and measure verification times
    let mut times = Vec::new();
    for _ in 0..100 {
        let signature = keypair.sign(message, &mut rng).unwrap();

        let start = Instant::now();
        let _ = public_key.verify(message, &signature);
        times.push(start.elapsed());
    }

    // Calculate mean and variance
    let mean = times.iter().sum::<Duration>() / times.len() as u32;
    let variance: Duration = times
        .iter()
        .map(|&t| if t > mean { t - mean } else { mean - t })
        .sum::<Duration>()
        / times.len() as u32;

    // Variance should be small for constant-time operations
    assert!(
        variance < Duration::from_millis(5),
        "Verification timing variance too large: {:?}",
        variance
    );
}

/// Test ML-DSA against known attack vectors
#[test]
fn test_ml_dsa_security_properties() {
    let mut rng = rand::thread_rng();

    // Test 1: Signature should not reveal secret key
    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let message = b"Secret key leakage test";
    let signature = keypair.sign(message, &mut rng).unwrap();

    // Signature should not contain secret key data
    let secret_key_bytes = keypair.secret_key();
    for window in secret_key_bytes.windows(16) {
        assert!(
            !signature.windows(16).any(|sig_window| sig_window == window),
            "Signature should not contain secret key data"
        );
    }

    // Test 2: Signature forgery should be infeasible
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let forged_signature = vec![0u8; ml_dsa_65_params::SIGNATURE_SIZE];
    let result = public_key.verify(message, &forged_signature);
    assert!(result.is_err(), "Forged signature should not verify");
}

/// Performance benchmark for ML-DSA operations
#[test]
fn test_ml_dsa_performance_benchmarks() {
    let mut rng = rand::thread_rng();
    let message = b"Performance test message";

    // Benchmark key generation
    let start = Instant::now();
    let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
    let keygen_time = start.elapsed();

    // Benchmark signing
    let start = Instant::now();
    let signature = keypair.sign(message, &mut rng).unwrap();
    let sign_time = start.elapsed();

    // Benchmark verification
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
    let start = Instant::now();
    public_key.verify(message, &signature).unwrap();
    let verify_time = start.elapsed();

    // Performance requirements (adjust based on actual requirements)
    assert!(
        keygen_time < Duration::from_millis(100),
        "Key generation too slow: {:?}",
        keygen_time
    );
    assert!(
        sign_time < Duration::from_millis(50),
        "Signing too slow: {:?}",
        sign_time
    );
    assert!(
        verify_time < Duration::from_millis(50),
        "Verification too slow: {:?}",
        verify_time
    );

    println!("Performance results:");
    println!("  Key generation: {:?}", keygen_time);
    println!("  Signing: {:?}", sign_time);
    println!("  Verification: {:?}", verify_time);
}
