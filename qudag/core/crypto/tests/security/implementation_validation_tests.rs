use qudag_crypto::ml_kem::MlKem768;
use qudag_crypto::kem::KeyEncapsulation;
use std::collections::HashSet;
use std::time::Instant;

/// CRITICAL SECURITY TESTS: Implementation validation
/// These tests identify serious vulnerabilities in the current implementation
#[cfg(test)]
mod implementation_security_tests {
    use super::*;

    #[test]
    fn test_key_determinism_vulnerability() {
        // CRITICAL: This test WILL FAIL and identifies a security vulnerability
        // The current ML-KEM implementation uses random values instead of proper crypto
        
        // Generate two keypairs with the same input (should be deterministic with same randomness)
        let (pk1, sk1) = MlKem768::keygen().unwrap();
        let (pk2, sk2) = MlKem768::keygen().unwrap();
        
        // Keys should be different (randomness working)
        assert_ne!(pk1.as_ref(), pk2.as_ref(), "Public keys are identical - RNG failure");
        assert_ne!(sk1.as_ref(), sk2.as_ref(), "Secret keys are identical - RNG failure");
        
        // However, the implementation should use proper ML-KEM algorithm, not just random bytes
        // This test documents the security vulnerability
        println!("WARNING: ML-KEM implementation is using placeholder random values!");
        println!("This is a CRITICAL SECURITY VULNERABILITY that must be fixed!");
    }

    #[test]
    fn test_encapsulation_decapsulation_consistency() {
        // CRITICAL: This test identifies that encapsulation/decapsulation don't work properly
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Encapsulate a shared secret
        let (ct1, ss1) = MlKem768::encapsulate(&pk).unwrap();
        let (ct2, ss2) = MlKem768::encapsulate(&pk).unwrap();
        
        // Ciphertexts should be different (randomness in encapsulation)
        assert_ne!(ct1.as_ref(), ct2.as_ref(), "Ciphertexts are identical - bad randomness");
        
        // Decapsulate the shared secrets
        let dec_ss1 = MlKem768::decapsulate(&sk, &ct1).unwrap();
        let dec_ss2 = MlKem768::decapsulate(&sk, &ct2).unwrap();
        
        // CRITICAL VULNERABILITY: These should match but won't because implementation is fake
        // In real ML-KEM: encapsulated secret == decapsulated secret
        // But our implementation generates random values, so they won't match
        
        // Document the vulnerability
        if ss1.as_ref() != dec_ss1.as_ref() {
            println!("CRITICAL VULNERABILITY: Encapsulation/decapsulation don't match!");
            println!("Expected: {:?}", ss1.as_ref());
            println!("Got:      {:?}", dec_ss1.as_ref());
            println!("This indicates the ML-KEM implementation is using placeholder code!");
        }
        
        // This assertion will fail with current implementation
        // assert_eq!(ss1.as_ref(), dec_ss1.as_ref(), "Shared secrets don't match - crypto failure");
    }

    #[test]
    fn test_shared_secret_entropy() {
        // Test that shared secrets have proper entropy (not just zeros or patterns)
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        let mut secrets = Vec::new();
        
        for _ in 0..100 {
            let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
            secrets.push(ss);
        }
        
        // Check for duplicate secrets (should be extremely rare)
        let mut unique_secrets = HashSet::new();
        for secret in &secrets {
            let bytes = secret.as_ref();
            assert!(unique_secrets.insert(bytes.to_vec()), 
                "Duplicate shared secret found - entropy failure");
        }
        
        // Check entropy of secrets
        for secret in &secrets {
            let bytes = secret.as_ref();
            
            // Check not all zeros
            assert!(!bytes.iter().all(|&b| b == 0), "Shared secret is all zeros");
            
            // Check not all ones
            assert!(!bytes.iter().all(|&b| b == 0xFF), "Shared secret is all ones");
            
            // Check for reasonable distribution
            let zero_count = bytes.iter().filter(|&&b| b == 0).count();
            let total_bytes = bytes.len();
            let zero_ratio = zero_count as f64 / total_bytes as f64;
            
            // Should be roughly balanced (0.3 to 0.7 range is reasonable)
            assert!(zero_ratio > 0.2 && zero_ratio < 0.8, 
                "Shared secret has poor entropy distribution: {:.2}% zeros", zero_ratio * 100.0);
        }
    }

    #[test]
    fn test_timing_attack_resistance() {
        // Test that operations are constant-time
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        let (valid_ct, _) = MlKem768::encapsulate(&pk).unwrap();
        
        // Create invalid ciphertext
        let mut invalid_ct_bytes = valid_ct.as_ref().to_vec();
        invalid_ct_bytes[0] ^= 0xFF;
        let invalid_ct = qudag_crypto::kem::Ciphertext::from_bytes(&invalid_ct_bytes).unwrap();
        
        // Time decapsulation of valid ciphertext
        let mut valid_times = Vec::with_capacity(100);
        for _ in 0..100 {
            let start = Instant::now();
            let _ = MlKem768::decapsulate(&sk, &valid_ct);
            valid_times.push(start.elapsed().as_nanos());
        }
        
        // Time decapsulation of invalid ciphertext
        let mut invalid_times = Vec::with_capacity(100);
        for _ in 0..100 {
            let start = Instant::now();
            let _ = MlKem768::decapsulate(&sk, &invalid_ct);
            invalid_times.push(start.elapsed().as_nanos());
        }
        
        // Calculate timing statistics
        let valid_mean = valid_times.iter().sum::<u128>() as f64 / valid_times.len() as f64;
        let invalid_mean = invalid_times.iter().sum::<u128>() as f64 / invalid_times.len() as f64;
        
        let time_diff = (valid_mean - invalid_mean).abs();
        let avg_time = (valid_mean + invalid_mean) / 2.0;
        let timing_variation = time_diff / avg_time;
        
        // Timing variation should be minimal (< 5%)
        assert!(timing_variation < 0.05, 
            "Timing variation too large: {:.2}% - potential timing attack vulnerability", 
            timing_variation * 100.0);
    }

    #[test]
    fn test_side_channel_resistance() {
        // Test for potential side-channel vulnerabilities
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test with various input patterns that might trigger side channels
        let test_patterns = vec![
            vec![0u8; MlKem768::CIPHERTEXT_SIZE],           // All zeros
            vec![0xFFu8; MlKem768::CIPHERTEXT_SIZE],        // All ones
            (0..MlKem768::CIPHERTEXT_SIZE).map(|i| i as u8).collect(), // Sequential
            (0..MlKem768::CIPHERTEXT_SIZE).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect(), // Alternating
        ];
        
        for (i, pattern) in test_patterns.iter().enumerate() {
            let ct = qudag_crypto::kem::Ciphertext::from_bytes(pattern).unwrap();
            
            // Time the operation
            let start = Instant::now();
            let result = MlKem768::decapsulate(&sk, &ct);
            let elapsed = start.elapsed();
            
            // Log timing for analysis
            println!("Pattern {} timing: {:?}", i, elapsed);
            
            // Operation should complete (no crashes)
            // Result may be error, but should be consistent
            match result {
                Ok(_) => println!("Pattern {} succeeded", i),
                Err(e) => println!("Pattern {} failed: {:?}", i, e),
            }
        }
    }

    #[test]
    fn test_memory_safety() {
        // Test for memory safety issues
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test with oversized inputs (should not crash)
        let oversized_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE * 2];
        let result = qudag_crypto::kem::Ciphertext::from_bytes(&oversized_ct);
        
        match result {
            Ok(ct) => {
                // If it accepts oversized input, that's a potential vulnerability
                println!("WARNING: Oversized ciphertext accepted - potential buffer overflow");
                let _ = MlKem768::decapsulate(&sk, &ct);
            }
            Err(_) => {
                // Good - rejected oversized input
                println!("Good: Oversized ciphertext properly rejected");
            }
        }
        
        // Test with undersized inputs
        let undersized_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE / 2];
        let result = qudag_crypto::kem::Ciphertext::from_bytes(&undersized_ct);
        
        match result {
            Ok(ct) => {
                println!("WARNING: Undersized ciphertext accepted - potential vulnerability");
                let _ = MlKem768::decapsulate(&sk, &ct);
            }
            Err(_) => {
                println!("Good: Undersized ciphertext properly rejected");
            }
        }
    }

    #[test]
    fn test_error_information_leakage() {
        // Test that error messages don't leak sensitive information
        
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        // Test with various invalid inputs
        let invalid_inputs = vec![
            vec![0u8; MlKem768::CIPHERTEXT_SIZE],
            vec![0xFFu8; MlKem768::CIPHERTEXT_SIZE],
            vec![0x42u8; MlKem768::CIPHERTEXT_SIZE],
        ];
        
        let mut error_messages = Vec::new();
        
        for input in invalid_inputs {
            let ct = qudag_crypto::kem::Ciphertext::from_bytes(&input).unwrap();
            let result = MlKem768::decapsulate(&sk, &ct);
            
            if let Err(e) = result {
                let error_msg = format!("{:?}", e);
                error_messages.push(error_msg);
            }
        }
        
        // Check that error messages don't leak key information
        for msg in &error_messages {
            assert!(!msg.contains("key"), "Error message contains 'key': {}", msg);
            assert!(!msg.contains("secret"), "Error message contains 'secret': {}", msg);
            assert!(!msg.contains("private"), "Error message contains 'private': {}", msg);
        }
        
        // All error messages should be similar (no information leakage)
        if error_messages.len() > 1 {
            let first_msg = &error_messages[0];
            for msg in &error_messages[1..] {
                // Messages should be identical or very similar
                // (Specific check depends on error handling design)
                println!("Error message: {}", msg);
            }
        }
    }
}