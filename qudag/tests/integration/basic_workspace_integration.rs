//! Basic workspace integration tests to validate module compilation and basic functionality

#[cfg(test)]
mod tests {
    use qudag_crypto::{KEMError, KeyEncapsulation, ml_kem::MlKem768};
    use std::sync::Arc;

    #[test]
    fn test_crypto_module_basic_functionality() {
        // Test ML-KEM key generation
        let result = MlKem768::keygen();
        assert!(result.is_ok(), "ML-KEM key generation should succeed");
        
        let (pk, sk) = result.unwrap();
        assert!(!pk.as_ref().is_empty(), "Public key should not be empty");
        assert!(!sk.as_ref().is_empty(), "Secret key should not be empty");
    }

    #[test]
    fn test_ml_kem_integration() {
        // Test full ML-KEM workflow
        let (pk, sk) = MlKem768::keygen().expect("Key generation failed");
        
        let (ct, ss1) = MlKem768::encapsulate(&pk).expect("Encapsulation failed");
        assert!(!ct.as_ref().is_empty(), "Ciphertext should not be empty");
        assert!(!ss1.as_ref().is_empty(), "Shared secret should not be empty");
        
        let ss2 = MlKem768::decapsulate(&sk, &ct).expect("Decapsulation failed");
        assert_eq!(ss1.as_ref(), ss2.as_ref(), "Shared secrets should match");
    }

    #[test]
    fn test_crypto_error_handling() {
        use qudag_crypto::kem::{PublicKey, Ciphertext, SecretKey};
        
        // Test with invalid ciphertext
        let (_, sk) = MlKem768::keygen().unwrap();
        let invalid_ct = Ciphertext::from_bytes(&[0u8; 10]);
        
        match invalid_ct {
            Ok(ct) => {
                let result = MlKem768::decapsulate(&sk, &ct);
                // Should either succeed or fail gracefully
                match result {
                    Ok(_) => (), // Valid result
                    Err(KEMError::DecapsulationError) => (), // Expected error
                    Err(_) => panic!("Unexpected error type"),
                }
            }
            Err(_) => (), // Expected - invalid ciphertext format
        }
    }

    #[test]
    fn test_concurrent_crypto_operations() {
        use std::thread;
        
        let handles: Vec<_> = (0..4).map(|_| {
            thread::spawn(|| {
                let (pk, sk) = MlKem768::keygen().unwrap();
                let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
                let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                assert_eq!(ss1.as_ref(), ss2.as_ref());
            })
        }).collect();
        
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }
    }
}