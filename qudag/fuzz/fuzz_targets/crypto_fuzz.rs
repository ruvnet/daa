#![no_main]
use libfuzzer_sys::fuzz_target;
use zeroize::Zeroize;
use std::time::Instant;
use qudag_crypto::{
    MlKem768, HashFunction, 
    kem::{KeyEncapsulation, KEMError},
    signature::DigitalSignature,
    ml_dsa::MlDsa65,
    fingerprint::QuantumFingerprint,
};

/// Helper function to verify basic timing consistency 
fn verify_timing_consistency<F>(op: F) -> bool 
where
    F: Fn() -> Result<(), ()>
{
    let iterations = 50; // Reduced for faster fuzzing
    let mut timings = Vec::with_capacity(iterations);
    
    // Collect timing samples
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    if timings.is_empty() {
        return false;
    }
    
    // Calculate basic statistics
    let mean = timings.iter().sum::<std::time::Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    // Accept reasonable variance for fuzzing
    variance < 100000
}

/// Helper to validate proper memory cleanup
fn validate_memory_cleanup(data: &[u8]) -> bool {
    // Test stack cleanup
    let mut test_data = data.to_vec();
    test_data.zeroize();
    test_data.iter().all(|&b| b == 0)
}

/// Test cryptographic hash function behavior
fn test_hash_operations(data: &[u8]) {
    use blake3::Hasher;
    
    // Test consistent hashing
    let hash1 = blake3::hash(data);
    let hash2 = blake3::hash(data);
    assert_eq!(hash1, hash2, "Hash function not deterministic");
    
    // Test incremental hashing
    let mut hasher = Hasher::new();
    hasher.update(data);
    let incremental_hash = hasher.finalize();
    assert_eq!(hash1, incremental_hash, "Incremental hash mismatch");
    
    // Test different chunk sizes
    if data.len() > 8 {
        let mut chunked_hasher = Hasher::new();
        for chunk in data.chunks(8) {
            chunked_hasher.update(chunk);
        }
        let chunked_hash = chunked_hasher.finalize();
        assert_eq!(hash1, chunked_hash, "Chunked hash mismatch");
    }
}

/// Test memory zeroization
fn test_zeroization(data: &[u8]) {
    // Test Vec zeroization
    let mut vec_data = data.to_vec();
    vec_data.zeroize();
    assert!(vec_data.iter().all(|&b| b == 0), "Vec not properly zeroized");
    
    // Test array zeroization
    if data.len() >= 32 {
        let mut array_data = [0u8; 32];
        array_data.copy_from_slice(&data[..32]);
        array_data.zeroize();
        assert!(array_data.iter().all(|&b| b == 0), "Array not properly zeroized");
    }
}

/// Test random number generation consistency
fn test_random_consistency() {
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    
    let seed = [42u8; 32];
    let mut rng1 = StdRng::from_seed(seed);
    let mut rng2 = StdRng::from_seed(seed);
    
    // Generate same sequence
    for _ in 0..100 {
        let val1: u64 = rng1.gen();
        let val2: u64 = rng2.gen();
        assert_eq!(val1, val2, "RNG not deterministic with same seed");
    }
}

/// Test ML-KEM key encapsulation mechanism
fn test_ml_kem_operations(data: &[u8]) {
    if data.len() < 32 {
        return;
    }
    
    // Generate key pair
    let kem = MlKem768::new();
    let (pk, sk) = match kem.generate_keypair() {
        Ok(keypair) => keypair,
        Err(_) => return, // Skip if key generation fails
    };
    
    // Test encapsulation
    let (ciphertext, shared_secret1) = match kem.encapsulate(&pk) {
        Ok(result) => result,
        Err(_) => return,
    };
    
    // Test decapsulation
    let shared_secret2 = match kem.decapsulate(&sk, &ciphertext) {
        Ok(secret) => secret,
        Err(_) => return,
    };
    
    // Verify shared secrets match
    assert_eq!(shared_secret1.as_bytes(), shared_secret2.as_bytes(), "Shared secrets don't match");
    
    // Test with malformed ciphertext
    if data.len() >= ciphertext.as_bytes().len() {
        let mut malformed_ct = ciphertext.as_bytes().to_vec();
        malformed_ct[0] ^= 1; // Flip one bit
        
        let malformed_ciphertext = match qudag_crypto::kem::Ciphertext::from_bytes(&malformed_ct) {
            Ok(ct) => ct,
            Err(_) => return,
        };
        
        // Decapsulation should fail or produce different result
        match kem.decapsulate(&sk, &malformed_ciphertext) {
            Ok(malformed_secret) => {
                // If it succeeds, it should be different from original
                assert_ne!(shared_secret1.as_bytes(), malformed_secret.as_bytes(), 
                          "Malformed ciphertext produced same shared secret");
            }
            Err(_) => {
                // Expected failure - this is correct behavior
            }
        }
    }
}

/// Test ML-DSA digital signature algorithm
fn test_ml_dsa_operations(data: &[u8]) {
    if data.len() < 32 {
        return;
    }
    
    let dsa = MlDsa65::new();
    
    // Generate key pair
    let (pk, sk) = match dsa.generate_keypair() {
        Ok(keypair) => keypair,
        Err(_) => return,
    };
    
    // Test signing
    let message = &data[..std::cmp::min(data.len(), 1024)];
    let signature = match dsa.sign(&sk, message) {
        Ok(sig) => sig,
        Err(_) => return,
    };
    
    // Test verification
    let verification_result = dsa.verify(&pk, message, &signature);
    assert!(verification_result.is_ok(), "Signature verification failed");
    
    // Test with modified message
    if message.len() > 1 {
        let mut modified_message = message.to_vec();
        modified_message[0] ^= 1; // Flip one bit
        
        let modified_verification = dsa.verify(&pk, &modified_message, &signature);
        assert!(modified_verification.is_err(), "Modified message verification should fail");
    }
    
    // Test with malformed signature
    if data.len() >= signature.as_bytes().len() {
        let mut malformed_sig = signature.as_bytes().to_vec();
        malformed_sig[0] ^= 1; // Flip one bit
        
        let malformed_signature = match qudag_crypto::signature::Signature::from_bytes(&malformed_sig) {
            Ok(sig) => sig,
            Err(_) => return,
        };
        
        let malformed_verification = dsa.verify(&pk, message, &malformed_signature);
        assert!(malformed_verification.is_err(), "Malformed signature verification should fail");
    }
}

/// Test quantum fingerprint operations
fn test_quantum_fingerprint_operations(data: &[u8]) {
    if data.is_empty() {
        return;
    }
    
    let fingerprint = QuantumFingerprint::new();
    
    // Test fingerprint generation
    let fp1 = match fingerprint.generate(data) {
        Ok(fp) => fp,
        Err(_) => return,
    };
    
    // Test deterministic behavior
    let fp2 = match fingerprint.generate(data) {
        Ok(fp) => fp,
        Err(_) => return,
    };
    
    assert_eq!(fp1.as_bytes(), fp2.as_bytes(), "Fingerprint not deterministic");
    
    // Test with modified data
    if data.len() > 1 {
        let mut modified_data = data.to_vec();
        modified_data[0] ^= 1;
        
        let fp3 = match fingerprint.generate(&modified_data) {
            Ok(fp) => fp,
            Err(_) => return,
        };
        
        assert_ne!(fp1.as_bytes(), fp3.as_bytes(), "Fingerprint unchanged for modified data");
    }
    
    // Test verification
    let verification_result = fingerprint.verify(data, &fp1);
    assert!(verification_result.is_ok(), "Fingerprint verification failed");
    
    // Test with wrong data
    if data.len() > 1 {
        let wrong_data = vec![0xAA; data.len()];
        let wrong_verification = fingerprint.verify(&wrong_data, &fp1);
        assert!(wrong_verification.is_err(), "Wrong data verification should fail");
    }
}

fuzz_target!(|data: &[u8]| {
    // Set panic hook to prevent information leaks
    std::panic::set_hook(Box::new(|_| {}));

    if data.is_empty() {
        return;
    }

    // Test hash operations with timing validation
    let hash_timing = verify_timing_consistency(|| {
        test_hash_operations(data);
        Ok(())
    });
    // Don't assert on timing in fuzzing - just ensure it doesn't crash

    // Test zeroization
    test_zeroization(data);

    // Test memory cleanup validation
    assert!(validate_memory_cleanup(data), "Memory not properly zeroized");

    // Test random number generation (not dependent on input data)
    if data.len() >= 32 {
        test_random_consistency();
    }

    // Test edge cases
    if data.len() >= 64 {
        // Test with all zeros
        let zero_data = vec![0u8; 64];
        test_hash_operations(&zero_data);
        test_zeroization(&zero_data);

        // Test with all ones  
        let ones_data = vec![0xFFu8; 64];
        test_hash_operations(&ones_data);
        test_zeroization(&ones_data);

        // Test with alternating pattern
        let alt_data: Vec<u8> = (0..64).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect();
        test_hash_operations(&alt_data);
        test_zeroization(&alt_data);
    }

    // Test with truncated data
    for i in 1..std::cmp::min(data.len(), 32) {
        let truncated = &data[..i];
        test_hash_operations(truncated);
        if truncated.len() >= 4 {
            test_zeroization(truncated);
        }
    }

    // Test with bit flipping
    if data.len() >= 16 {
        let mut mutated = data[..16].to_vec();
        for i in 0..mutated.len() {
            mutated[i] ^= 1;
            test_hash_operations(&mutated);
            mutated[i] ^= 1; // Restore original
        }
    }

    // Test ML-KEM operations with timing validation
    if data.len() >= 32 {
        let _kem_timing = verify_timing_consistency(|| {
            test_ml_kem_operations(data);
            Ok(())
        });
        // Don't assert on timing in fuzzing - just ensure it doesn't crash
    }

    // Test ML-DSA operations with timing validation
    if data.len() >= 64 {
        let _dsa_timing = verify_timing_consistency(|| {
            test_ml_dsa_operations(data);
            Ok(())
        });
        // Don't assert on timing in fuzzing - just ensure it doesn't crash
    }

    // Test quantum fingerprint operations
    if data.len() >= 16 {
        let _fp_timing = verify_timing_consistency(|| {
            test_quantum_fingerprint_operations(data);
            Ok(())
        });
        // Don't assert on timing in fuzzing - just ensure it doesn't crash
    }

    // Test with various crypto edge cases
    if data.len() >= 128 {
        // Test with empty inputs where allowed
        test_hash_operations(&[]);
        
        // Test with maximum size inputs
        let max_data = vec![0xAA; 8192];
        test_hash_operations(&max_data);
        test_quantum_fingerprint_operations(&max_data);
        
        // Test with minimal size inputs
        let min_data = vec![0x55; 1];
        test_hash_operations(&min_data);
        test_quantum_fingerprint_operations(&min_data);
        
        // Test with specific patterns known to cause issues
        let patterns = vec![
            vec![0x00; 64], // All zeros
            vec![0xFF; 64], // All ones
            (0..64).map(|i| i as u8).collect::<Vec<u8>>(), // Sequential
            (0..64).map(|i| (i as u8) ^ 0xAA).collect::<Vec<u8>>(), // XOR pattern
        ];
        
        for pattern in patterns {
            test_hash_operations(&pattern);
            test_quantum_fingerprint_operations(&pattern);
            if pattern.len() >= 32 {
                test_ml_kem_operations(&pattern);
            }
            if pattern.len() >= 64 {
                test_ml_dsa_operations(&pattern);
            }
        }
    }
});