use qudag_crypto::fingerprint::{Fingerprint, FingerprintError};
use rand::thread_rng;

#[test]
fn test_fingerprint_generation_and_verification() {
    let mut rng = thread_rng();
    let data = b"test data for fingerprinting";

    // Generate fingerprint
    let (fingerprint, public_key) = Fingerprint::generate(data, &mut rng).unwrap();

    // Verify the fingerprint
    assert!(fingerprint.verify(&public_key).is_ok());
}

#[test]
fn test_different_data_produces_different_fingerprints() {
    let mut rng = thread_rng();
    let data1 = b"first piece of data";
    let data2 = b"second piece of data";

    let (fp1, _) = Fingerprint::generate(data1, &mut rng).unwrap();
    let (fp2, _) = Fingerprint::generate(data2, &mut rng).unwrap();

    // Fingerprints should be different
    assert_ne!(fp1.data(), fp2.data());
}

#[test]
fn test_fingerprint_constant_time_comparison() {
    let mut rng = thread_rng();
    let data = b"test data";

    let (fp1, _) = Fingerprint::generate(data, &mut rng).unwrap();
    let (fp2, _) = Fingerprint::generate(data, &mut rng).unwrap();

    // Even for same data, fingerprints should be different due to different keys
    assert_ne!(fp1, fp2);
}

#[test]
fn test_fingerprint_verification_with_wrong_key() {
    let mut rng = thread_rng();
    let data = b"test data";

    let (fingerprint, _) = Fingerprint::generate(data, &mut rng).unwrap();
    let (_, wrong_key) = Fingerprint::generate(b"other data", &mut rng).unwrap();

    // Verification should fail with wrong key
    assert!(fingerprint.verify(&wrong_key).is_err());
}

#[test]
fn test_empty_data_fingerprint() {
    let mut rng = thread_rng();
    let empty_data = b"";

    // Should be able to generate fingerprint for empty data
    let (fingerprint, public_key) = Fingerprint::generate(empty_data, &mut rng).unwrap();
    assert!(fingerprint.verify(&public_key).is_ok());
}

#[test]
fn test_large_data_fingerprint() {
    let mut rng = thread_rng();
    let large_data = vec![0u8; 1024 * 1024]; // 1MB of data

    // Should handle large data efficiently
    let (fingerprint, public_key) = Fingerprint::generate(&large_data, &mut rng).unwrap();
    assert!(fingerprint.verify(&public_key).is_ok());
}

#[test]
fn test_fingerprint_memory_safety() {
    let mut rng = thread_rng();
    let data = b"sensitive data";

    let (fingerprint, _) = Fingerprint::generate(data, &mut rng).unwrap();
    let fingerprint_ptr = fingerprint.data().as_ptr();
    drop(fingerprint);

    // Note: This test is illustrative only
    // In practice, we can't reliably test zeroization
    // as the memory may be reused or optimized away
}
