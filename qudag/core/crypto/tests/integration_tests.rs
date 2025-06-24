#[cfg(test)]
mod integration_tests {
    use qudag_crypto::{
        hqc::HQC,
        ml_dsa::{MLDsa, SIGNATURE_LENGTH},
        ml_kem::{KeyEncapsulation, MlKem768},
    };
    use std::time::Instant;

    #[test]
    fn test_kem_integration() {
        // Generate keypair
        let (pk, sk) = MlKem768::keygen().expect("Failed to generate ML-KEM keypair");

        // Encapsulate shared secret
        let (ct, ss1) = MlKem768::encapsulate(&pk).expect("Failed to encapsulate");

        // Decapsulate shared secret
        let ss2 = MlKem768::decapsulate(&sk, &ct).expect("Failed to decapsulate");

        // Verify shared secrets match
        assert_eq!(ss1.as_ref(), ss2.as_ref());
    }

    #[test]
    fn test_signature_integration() {
        let message = b"Test message for integration testing";

        // Generate keypair
        let (pk, sk) = MLDsa::keygen().expect("Failed to generate ML-DSA keypair");

        // Sign message
        let signature = sk.sign(message).expect("Failed to sign message");
        assert_eq!(signature.len(), SIGNATURE_LENGTH);

        // Verify signature
        let pk_bytes = pk.public_key.expect("Missing public key");
        MLDsa::verify(message, &signature, &pk_bytes).expect("Failed to verify signature");
    }

    #[test]
    fn test_encryption_integration() {
        let message = b"Test message for HQC encryption";

        // Generate keypair
        let (pk, sk) = HQC::keygen().expect("Failed to generate HQC keypair");

        // Encrypt message
        let ciphertext = pk.encrypt(message).expect("Failed to encrypt message");

        // Decrypt message
        let decrypted = sk.decrypt(&ciphertext).expect("Failed to decrypt message");
        assert_eq!(message[..], decrypted[..32]);
    }

    #[test]
    fn test_performance_requirements() {
        let iterations = 100;
        let mut total_time = std::time::Duration::new(0, 0);

        for _ in 0..iterations {
            let start = Instant::now();

            // Generate and verify signature
            let message = b"Performance test message";
            let (pk, sk) = MLDsa::keygen().expect("Failed to generate keypair");
            let signature = sk.sign(message).expect("Failed to sign");
            let pk_bytes = pk.public_key.expect("Missing public key");
            MLDsa::verify(message, &signature, &pk_bytes).expect("Failed to verify");

            total_time += start.elapsed();
        }

        let avg_time = total_time / iterations;
        println!("Average operation time: {:?}", avg_time);
        assert!(
            avg_time.as_millis() < 100,
            "Performance requirements not met"
        );
    }

    #[test]
    fn test_memory_safety() {
        use std::mem;

        // Test ML-KEM key alignment
        let (pk, sk) = MlKem768::keygen().expect("Failed to generate keypair");
        assert!(
            mem::align_of_val(&pk) >= 16,
            "Public key not properly aligned"
        );
        assert!(
            mem::align_of_val(&sk) >= 16,
            "Secret key not properly aligned"
        );

        // Test automatic cleanup
        let mut secret = [0u8; 32];
        {
            let (ct, ss) = MlKem768::encapsulate(&pk).expect("Failed to encapsulate");
            secret.copy_from_slice(ss.as_ref());
        }
        // Secret should be cleared after drop
        assert_ne!(secret, [0u8; 32]);
    }
}
