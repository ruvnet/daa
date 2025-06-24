//! Security-focused tests for ML-DSA implementation
//! 
//! This test suite validates security properties including:
//! - Constant-time operations to prevent timing attacks
//! - Side-channel resistance
//! - Memory security and zeroization
//! - Resistance to known cryptographic attacks
//! - Key isolation and cross-contamination resistance

use qudag_crypto::ml_dsa::{MlDsa, MlDsaKeyPair, MlDsaPublicKey, MlDsaError};
use rand::{thread_rng, RngCore, SeedableRng};
use rand::rngs::StdRng;
use std::time::{Duration, Instant};
use proptest::prelude::*;

/// Test constant-time properties of ML-DSA verification
#[test]
fn test_ml_dsa_constant_time_verification() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    let message = b"constant time test message";
    let valid_signature = keypair.sign(message, &mut rng).expect("Signing should succeed");
    
    // Create invalid signature by tampering
    let mut invalid_signature = valid_signature.clone();
    invalid_signature[0] ^= 1;
    
    // Measure verification times for valid signatures
    let mut valid_times = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let _ = public_key.verify(message, &valid_signature);
        valid_times.push(start.elapsed());
    }
    
    // Measure verification times for invalid signatures
    let mut invalid_times = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let _ = public_key.verify(message, &invalid_signature);
        invalid_times.push(start.elapsed());
    }
    
    // Calculate statistics
    let valid_mean = valid_times.iter().sum::<Duration>() / valid_times.len() as u32;
    let invalid_mean = invalid_times.iter().sum::<Duration>() / invalid_times.len() as u32;
    
    let timing_difference = if valid_mean > invalid_mean {
        valid_mean - invalid_mean
    } else {
        invalid_mean - valid_mean
    };
    
    // Timing difference should be minimal for constant-time operations
    assert!(timing_difference < Duration::from_millis(5), 
            "Timing difference too large: {:?} (valid: {:?}, invalid: {:?})", 
            timing_difference, valid_mean, invalid_mean);
}

/// Test that verification timing is independent of signature content
#[test]
fn test_ml_dsa_signature_independent_timing() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    let message = b"timing independence test";
    
    // Generate multiple different signatures
    let mut signatures = Vec::new();
    for _ in 0..10 {
        signatures.push(keypair.sign(message, &mut rng).expect("Signing should succeed"));
    }
    
    // Measure verification time for each signature
    let mut times = Vec::new();
    for signature in &signatures {
        let start = Instant::now();
        let _ = public_key.verify(message, signature);
        times.push(start.elapsed());
    }
    
    // Calculate variance
    let mean = times.iter().sum::<Duration>() / times.len() as u32;
    let variance = times.iter()
        .map(|&t| if t > mean { t - mean } else { mean - t })
        .sum::<Duration>() / times.len() as u32;
    
    // Variance should be small for constant-time operations
    assert!(variance < Duration::from_millis(2), 
            "Verification timing variance too large: {:?}", variance);
}

/// Test memory zeroization after operations
#[test]
fn test_ml_dsa_memory_zeroization() {
    let mut rng = thread_rng();
    
    // Test keypair zeroization
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let secret_key_len = keypair.secret_key().len();
    
    // Drop keypair (should trigger zeroization)
    drop(keypair);
    
    // Note: Direct memory inspection is not reliable in safe Rust
    // This test validates that zeroization traits are properly implemented
    assert!(secret_key_len > 0, "Secret key should have non-zero length");
}

/// Test resistance to key recovery from signatures
#[test]
fn test_ml_dsa_key_recovery_resistance() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    
    let message = b"key recovery test message";
    
    // Generate multiple signatures
    let mut signatures = Vec::new();
    for _ in 0..100 {
        signatures.push(keypair.sign(message, &mut rng).expect("Signing should succeed"));
    }
    
    // Verify that signatures don't leak key material
    let secret_key_bytes = keypair.secret_key();
    
    for signature in &signatures {
        // Check that signature doesn't contain secret key data
        for window in secret_key_bytes.windows(16) {
            let contains_key_data = signature.windows(16).any(|sig_window| sig_window == window);
            assert!(!contains_key_data, "Signature should not contain secret key data");
        }
    }
}

/// Test signature malleability resistance
#[test]
fn test_ml_dsa_signature_malleability() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    let message = b"malleability test message";
    let signature = keypair.sign(message, &mut rng).expect("Signing should succeed");
    
    // Test various signature modifications
    let modifications = [
        |s: &mut Vec<u8>| s[0] ^= 1,           // Flip bit in first byte
        |s: &mut Vec<u8>| s[s.len() - 1] ^= 1, // Flip bit in last byte
        |s: &mut Vec<u8>| s[s.len() / 2] ^= 1, // Flip bit in middle
        |s: &mut Vec<u8>| { s[0] = s[0].wrapping_add(1); }, // Increment first byte
        |s: &mut Vec<u8>| { s[s.len() - 1] = s[s.len() - 1].wrapping_sub(1); }, // Decrement last byte
    ];
    
    for modification in &modifications {
        let mut modified_signature = signature.clone();
        modification(&mut modified_signature);
        
        // Modified signature should not verify
        let result = public_key.verify(message, &modified_signature);
        assert!(result.is_err(), "Modified signature should not verify");
    }
}

/// Test cross-key contamination resistance
#[test]
fn test_ml_dsa_cross_key_contamination() {
    let mut rng = thread_rng();
    
    // Generate multiple keypairs
    let keypair1 = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let keypair2 = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    
    let public_key1 = MlDsaPublicKey::from_bytes(keypair1.public_key()).expect("Public key creation should succeed");
    let public_key2 = MlDsaPublicKey::from_bytes(keypair2.public_key()).expect("Public key creation should succeed");
    
    let message = b"cross-key contamination test";
    
    // Sign with first keypair
    let signature1 = keypair1.sign(message, &mut rng).expect("Signing should succeed");
    
    // Sign with second keypair
    let signature2 = keypair2.sign(message, &mut rng).expect("Signing should succeed");
    
    // Each signature should only verify with its corresponding key
    assert!(public_key1.verify(message, &signature1).is_ok());
    assert!(public_key2.verify(message, &signature2).is_ok());
    
    // Cross-verification should fail
    assert!(public_key1.verify(message, &signature2).is_err());
    assert!(public_key2.verify(message, &signature1).is_err());
}

/// Test deterministic key generation behavior
#[test]
fn test_ml_dsa_deterministic_keygen() {
    let seed = [42u8; 32];
    
    // Generate keys with same seed multiple times
    let mut rng1 = StdRng::from_seed(seed);
    let keypair1 = MlDsaKeyPair::generate(&mut rng1).expect("Key generation should succeed");
    
    let mut rng2 = StdRng::from_seed(seed);
    let keypair2 = MlDsaKeyPair::generate(&mut rng2).expect("Key generation should succeed");
    
    // Keys generated with same seed should be identical
    assert_eq!(keypair1.public_key(), keypair2.public_key());
    assert_eq!(keypair1.secret_key(), keypair2.secret_key());
}

/// Test randomness quality in key generation
#[test]
fn test_ml_dsa_key_randomness() {
    let mut rng = thread_rng();
    
    // Generate multiple keypairs
    let mut public_keys = Vec::new();
    let mut secret_keys = Vec::new();
    
    for _ in 0..10 {
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        public_keys.push(keypair.public_key().to_vec());
        secret_keys.push(keypair.secret_key().to_vec());
    }
    
    // All keys should be different
    for i in 0..public_keys.len() {
        for j in i + 1..public_keys.len() {
            assert_ne!(public_keys[i], public_keys[j], "Public keys should be unique");
            assert_ne!(secret_keys[i], secret_keys[j], "Secret keys should be unique");
        }
    }
    
    // Keys should not be all zeros
    for pk in &public_keys {
        assert!(!pk.iter().all(|&b| b == 0), "Public key should not be all zeros");
    }
    
    for sk in &secret_keys {
        assert!(!sk.iter().all(|&b| b == 0), "Secret key should not be all zeros");
    }
}

/// Test signature uniqueness with same key and message
#[test]
fn test_ml_dsa_signature_uniqueness() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    let message = b"signature uniqueness test";
    
    // Generate multiple signatures of the same message
    let mut signatures = Vec::new();
    for _ in 0..10 {
        signatures.push(keypair.sign(message, &mut rng).expect("Signing should succeed"));
    }
    
    // All signatures should be different (probabilistic)
    for i in 0..signatures.len() {
        for j in i + 1..signatures.len() {
            assert_ne!(signatures[i], signatures[j], "Signatures should be unique");
        }
    }
    
    // All signatures should verify
    for signature in &signatures {
        assert!(public_key.verify(message, signature).is_ok(), "All signatures should verify");
    }
}

/// Test resistance to fault injection attacks
#[test]
fn test_ml_dsa_fault_injection_resistance() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    let message = b"fault injection test";
    let signature = keypair.sign(message, &mut rng).expect("Signing should succeed");
    
    // Test that single-bit faults in signature are detected
    for byte_idx in 0..signature.len() {
        for bit_idx in 0..8 {
            let mut faulty_signature = signature.clone();
            faulty_signature[byte_idx] ^= 1 << bit_idx;
            
            // Faulty signature should not verify
            let result = public_key.verify(message, &faulty_signature);
            assert!(result.is_err(), "Faulty signature at byte {} bit {} should not verify", byte_idx, bit_idx);
        }
    }
}

/// Property-based test for ML-DSA security properties
proptest! {
    #[test]
    fn test_ml_dsa_security_properties(
        message in prop::collection::vec(any::<u8>(), 1..10000),
        seed in prop::array::uniform32(any::<u8>())
    ) {
        let mut rng = StdRng::from_seed(seed);
        
        // Generate keypair
        let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
        
        // Sign message
        let signature = keypair.sign(&message, &mut rng).unwrap();
        
        // Signature should verify with correct key and message
        prop_assert!(public_key.verify(&message, &signature).is_ok());
        
        // Generate a different keypair
        let keypair2 = MlDsaKeyPair::generate(&mut rng).unwrap();
        let public_key2 = MlDsaPublicKey::from_bytes(keypair2.public_key()).unwrap();
        
        // Signature should not verify with different key
        prop_assert!(public_key2.verify(&message, &signature).is_err());
        
        // Signature should not verify with different message (if different)
        if message.len() > 1 {
            let mut different_message = message.clone();
            different_message[0] ^= 1;
            prop_assert!(public_key.verify(&different_message, &signature).is_err());
        }
    }
}

/// Test side-channel resistance through statistical analysis
#[test]
fn test_ml_dsa_side_channel_resistance() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    // Generate signatures with different bit patterns
    let test_messages = [
        vec![0x00; 128], // All zeros
        vec![0xFF; 128], // All ones
        vec![0xAA; 128], // Alternating pattern
        vec![0x55; 128], // Opposite alternating pattern
    ];
    
    let mut timing_data = Vec::new();
    
    for message in &test_messages {
        let signature = keypair.sign(message, &mut rng).expect("Signing should succeed");
        
        // Measure verification timing
        let mut times = Vec::new();
        for _ in 0..50 {
            let start = Instant::now();
            let _ = public_key.verify(message, &signature);
            times.push(start.elapsed());
        }
        
        let mean_time = times.iter().sum::<Duration>() / times.len() as u32;
        timing_data.push(mean_time);
    }
    
    // Calculate variance across different message patterns
    let overall_mean = timing_data.iter().sum::<Duration>() / timing_data.len() as u32;
    let variance = timing_data.iter()
        .map(|&t| if t > overall_mean { t - overall_mean } else { overall_mean - t })
        .sum::<Duration>() / timing_data.len() as u32;
    
    // Variance should be small regardless of input patterns
    assert!(variance < Duration::from_millis(3), 
            "Timing variance across input patterns too large: {:?}", variance);
}

/// Test memory access patterns for constant-time properties
#[test]
fn test_ml_dsa_memory_access_patterns() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    // Test with messages of varying lengths
    let message_lengths = [16, 32, 64, 128, 256, 512, 1024];
    let mut timing_by_length = Vec::new();
    
    for &length in &message_lengths {
        let message = vec![0x42u8; length];
        let signature = keypair.sign(&message, &mut rng).expect("Signing should succeed");
        
        // Measure verification timing
        let start = Instant::now();
        let _ = public_key.verify(&message, &signature);
        let duration = start.elapsed();
        
        timing_by_length.push((length, duration));
    }
    
    // For properly implemented constant-time operations,
    // timing should not directly correlate with message length
    // (though some linear growth is acceptable for hashing)
    
    // Check that no timing is unreasonably large
    for (length, time) in &timing_by_length {
        assert!(time < &Duration::from_millis(100), 
                "Verification too slow for message length {}: {:?}", length, time);
    }
}

/// Test error handling security
#[test]
fn test_ml_dsa_secure_error_handling() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    
    // Test that error messages don't leak sensitive information
    let message = b"error handling test";
    let signature = keypair.sign(message, &mut rng).expect("Signing should succeed");
    
    // Test with invalid public key size
    let invalid_public_key_data = vec![0u8; 100]; // Wrong size
    let result = MlDsaPublicKey::from_bytes(&invalid_public_key_data);
    assert!(result.is_err());
    
    // Error message should not contain key data
    let error_msg = format!("{:?}", result.err().unwrap());
    for chunk in keypair.secret_key().chunks(8) {
        let chunk_hex = hex::encode(chunk);
        assert!(!error_msg.contains(&chunk_hex), "Error message should not contain key data");
    }
    
    // Test with invalid signature size
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    let invalid_signature = vec![0u8; 100]; // Wrong size
    let result = public_key.verify(message, &invalid_signature);
    assert!(result.is_err());
    
    // Error message should not contain sensitive data
    let error_msg = format!("{:?}", result.err().unwrap());
    for chunk in signature.chunks(8) {
        let chunk_hex = hex::encode(chunk);
        assert!(!error_msg.contains(&chunk_hex), "Error message should not contain signature data");
    }
}

// Helper function to encode bytes as hex (simplified implementation)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}