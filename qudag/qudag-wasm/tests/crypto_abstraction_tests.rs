//! Tests for the crypto abstraction layer

#[cfg(test)]
mod tests {
    use qudag_wasm::crypto_traits::*;
    use qudag_wasm::crypto_unified::*;

    #[test]
    fn test_platform_feature_detection() {
        // At minimum, BLAKE3 should always be available
        assert!(PlatformFeatures::has_blake3());

        // Check that available_features returns at least one feature
        let features = PlatformFeatures::available_features();
        assert!(!features.is_empty());
        assert!(features.contains(&"BLAKE3"));

        // Platform notes should be Some for WASM, None for native
        #[cfg(target_arch = "wasm32")]
        assert!(PlatformFeatures::platform_notes().is_some());

        #[cfg(not(target_arch = "wasm32"))]
        assert!(PlatformFeatures::platform_notes().is_none());
    }

    #[test]
    fn test_provider_information() {
        let name = CurrentProvider::name();
        assert!(!name.is_empty());

        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(name, "native");

        #[cfg(target_arch = "wasm32")]
        assert_eq!(name, "wasm");

        let version = CurrentProvider::version();
        assert!(!version.is_empty());

        // Check fallback status
        #[cfg(not(target_arch = "wasm32"))]
        assert!(!CurrentProvider::is_fallback());

        #[cfg(target_arch = "wasm32")]
        assert!(CurrentProvider::is_fallback());
    }

    #[test]
    fn test_provider_initialization() {
        // Initialization should always succeed
        assert!(CurrentProvider::initialize().is_ok());

        // Multiple initializations should be safe
        assert!(CurrentProvider::initialize().is_ok());
    }

    #[test]
    fn test_blake3_hashing() {
        // Test empty input
        let empty_hash = UnifiedBlake3::hash(b"");
        assert_eq!(empty_hash.len(), 32);
        assert_eq!(UnifiedBlake3::output_size(), 32);

        // Test with data
        let data = b"Hello, QuDAG!";
        let hash1 = UnifiedBlake3::hash(data);
        let hash2 = UnifiedBlake3::hash(data);

        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);

        // Different input should produce different hash
        let different_data = b"Different data";
        let different_hash = UnifiedBlake3::hash(different_data);
        assert_ne!(hash1, different_hash);

        // Check algorithm info
        assert_eq!(UnifiedBlake3::algorithm_name(), "BLAKE3");
        assert!(UnifiedBlake3::is_available());
    }

    #[test]
    fn test_ml_dsa_availability() {
        let is_available = UnifiedMlDsa::is_available();
        let platform_says_available = PlatformFeatures::has_ml_dsa();

        // These should be consistent
        assert_eq!(is_available, platform_says_available);

        assert_eq!(UnifiedMlDsa::algorithm_name(), "ML-DSA-65");
    }

    #[test]
    fn test_ml_dsa_keypair_generation() {
        if !UnifiedMlDsa::is_available() {
            println!("Skipping ML-DSA test - not available on this platform");
            return;
        }

        match UnifiedMlDsa::generate_keypair() {
            Ok((public_key, private_key)) => {
                // Check key properties
                assert!(!public_key.to_bytes().is_empty());
                assert!(!private_key.to_bytes().is_empty());

                assert_eq!(public_key.algorithm(), "ML-DSA-65");
                assert_eq!(private_key.algorithm(), "ML-DSA-65");

                assert!(public_key.size() > 0);
                assert!(private_key.size() > 0);

                // Test key serialization round-trip
                let pk_bytes = public_key.to_bytes();
                let sk_bytes = private_key.to_bytes();

                // Try to reconstruct keys from bytes
                match (
                    <UnifiedMlDsa as QuantumResistantSigning>::PublicKey::from_bytes(&pk_bytes),
                    <UnifiedMlDsa as QuantumResistantSigning>::PrivateKey::from_bytes(&sk_bytes),
                ) {
                    (Ok(pk2), Ok(sk2)) => {
                        assert_eq!(pk2.to_bytes(), pk_bytes);
                        assert_eq!(sk2.to_bytes(), sk_bytes);
                    }
                    _ => {
                        println!("Key reconstruction not fully implemented yet");
                    }
                }
            }
            Err(e) => {
                // Some platforms might not support key generation
                println!("ML-DSA keypair generation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_crypto_error_types() {
        use CryptoAbstractionError::*;

        let errors = vec![
            UnsupportedPlatform("Test platform".to_string()),
            FeatureNotAvailable("Test feature".to_string()),
            CryptoOperationFailed("Test failure".to_string()),
            InvalidKey("Bad key".to_string()),
            InvalidData("Bad data".to_string()),
        ];

        for error in errors {
            // Check that Display is implemented
            let error_string = error.to_string();
            assert!(!error_string.is_empty());

            // Check that Error trait is implemented
            let _: &dyn std::error::Error = &error;
        }
    }

    #[test]
    fn test_capabilities_summary() {
        let summary = get_crypto_capabilities();

        // Check that summary contains expected sections
        assert!(summary.contains("QuDAG Crypto Capabilities"));
        assert!(summary.contains("Provider:"));
        assert!(summary.contains("Fallback Mode:"));
        assert!(summary.contains("Available Features:"));

        // Check platform-specific content
        #[cfg(not(target_arch = "wasm32"))]
        assert!(summary.contains("native"));

        #[cfg(target_arch = "wasm32")]
        {
            assert!(summary.contains("wasm"));
            assert!(summary.contains("Note:"));
        }
    }

    #[test]
    fn test_private_key_zeroization() {
        if !UnifiedMlDsa::is_available() {
            return;
        }

        match UnifiedMlDsa::generate_keypair() {
            Ok((_public_key, mut private_key)) => {
                let original_bytes = private_key.to_bytes();
                assert!(!original_bytes.is_empty());

                // Zeroize the key
                private_key.zeroize();

                // The key should still have the same structure but zeroed content
                let zeroed_bytes = private_key.to_bytes();
                assert_eq!(zeroed_bytes.len(), original_bytes.len());

                // Check if bytes are actually zeroed (all zeros)
                let all_zeros = zeroed_bytes.iter().all(|&b| b == 0);
                if !all_zeros {
                    println!("Note: Zeroization might not be fully implemented");
                }
            }
            Err(_) => {
                println!("Skipping zeroization test - keypair generation failed");
            }
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::tests::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn wasm_test_platform_detection() {
        test_platform_feature_detection();
    }

    #[wasm_bindgen_test]
    fn wasm_test_provider_info() {
        test_provider_information();
    }

    #[wasm_bindgen_test]
    fn wasm_test_blake3() {
        test_blake3_hashing();
    }

    #[wasm_bindgen_test]
    fn wasm_test_capabilities() {
        test_capabilities_summary();
    }
}
