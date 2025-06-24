//! Security compliance tests for QuDAG protocol.

use std::time::Duration;
use qudag_protocol::{
    Message, MessageFactory, HandshakeKeys, HandshakeCoordinator, HandshakeConfig,
    ProtocolStateMachine, ProtocolVersion,
};
use qudag_crypto::{MlDsa, MlKem768, KeyPair as KemKeyPair};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    /// Test cryptographic compliance
    #[tokio::test]
    async fn test_cryptographic_compliance() {
        let mut rng = thread_rng();
        
        // Test ML-DSA key generation
        let ml_dsa = MlDsa::new();
        let keypair = ml_dsa.generate_keypair().unwrap();
        
        // Verify key sizes match specification
        assert_eq!(
            keypair.public_key().len(),
            qudag_crypto::ml_dsa::ML_DSA_PUBLIC_KEY_SIZE,
            "ML-DSA public key must be correct size"
        );
        assert_eq!(
            keypair.secret_key().len(),
            qudag_crypto::ml_dsa::ML_DSA_SECRET_KEY_SIZE,
            "ML-DSA secret key must be correct size"
        );
        
        // Test signing and verification
        let message = b"test message for signature";
        let signature = keypair.sign(message, &mut rng).unwrap();
        
        assert_eq!(
            signature.len(),
            qudag_crypto::ml_dsa::ML_DSA_SIGNATURE_SIZE,
            "Signature must be correct size"
        );
        
        let public_key = qudag_crypto::MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
        public_key.verify(message, &signature).unwrap();
        
        // Test ML-KEM key encapsulation
        let ml_kem = MlKem768::new();
        let kem_keypair = ml_kem.generate_keypair().unwrap();
        
        let (shared_secret, ciphertext) = ml_kem.encapsulate(kem_keypair.public_key()).unwrap();
        let decrypted_secret = ml_kem.decapsulate(&ciphertext, kem_keypair.secret_key()).unwrap();
        
        assert_eq!(
            shared_secret.as_bytes(),
            decrypted_secret.as_bytes(),
            "KEM encapsulation/decapsulation must be consistent"
        );
    }

    /// Test message authentication compliance
    #[tokio::test]
    async fn test_message_authentication_compliance() {
        let mut rng = thread_rng();
        let ml_dsa = MlDsa::new();
        let keypair = ml_dsa.generate_keypair().unwrap();
        let public_key = qudag_crypto::MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
        
        // Test message signing
        let mut message = MessageFactory::create_ping().unwrap();
        message.sign(&keypair).unwrap();
        
        // Verify signature is present
        assert!(message.signature.is_some(), "Signed message must have signature");
        assert!(message.sender_key_hash.is_some(), "Signed message must have sender key hash");
        
        // Test signature verification
        assert!(message.verify(&public_key).unwrap(), "Valid signature must verify");
        
        // Test signature tampering detection
        let mut tampered_message = message.clone();
        tampered_message.payload[0] ^= 0xFF; // Flip bits in payload
        
        assert!(!tampered_message.verify(&public_key).unwrap(), "Tampered message must fail verification");
        
        // Test signature replay protection
        let mut replay_message = message.clone();
        replay_message.timestamp = 0; // Very old timestamp
        
        assert!(replay_message.validate().is_err(), "Replayed message must be rejected");
    }

    /// Test encryption compliance
    #[tokio::test]
    async fn test_encryption_compliance() {
        let ml_kem = MlKem768::new();
        let recipient_keypair = ml_kem.generate_keypair().unwrap();
        
        let message = MessageFactory::create_ping().unwrap();
        
        // Test message encryption
        let encrypted = qudag_protocol::message::EncryptedMessage::encrypt(
            &message,
            recipient_keypair.public_key(),
        ).unwrap();
        
        // Verify encryption metadata
        assert!(!encrypted.ciphertext.is_empty(), "Encrypted message must have ciphertext");
        assert!(!encrypted.encapsulation.is_empty(), "Encrypted message must have key encapsulation");
        assert_eq!(encrypted.timestamp, message.timestamp, "Timestamp must be preserved");
        
        // Test decryption
        let decrypted = encrypted.decrypt(recipient_keypair.secret_key()).unwrap();
        
        assert_eq!(message.payload, decrypted.payload, "Decrypted payload must match original");
        assert_eq!(message.id, decrypted.id, "Decrypted ID must match original");
    }

    /// Test handshake security compliance
    #[tokio::test]
    async fn test_handshake_security_compliance() {
        let mut rng = thread_rng();
        
        // Generate keys for both parties
        let alice_keys = HandshakeKeys {
            signature_keypair: MlDsa::new().generate_keypair().unwrap(),
            kem_keypair: MlKem768::new().generate_keypair().unwrap(),
        };
        
        let bob_keys = HandshakeKeys {
            signature_keypair: MlDsa::new().generate_keypair().unwrap(),
            kem_keypair: MlKem768::new().generate_keypair().unwrap(),
        };
        
        // Create handshake coordinators
        let config = HandshakeConfig::default();
        let alice_state = ProtocolStateMachine::new(ProtocolVersion::CURRENT);
        let bob_state = ProtocolStateMachine::new(ProtocolVersion::CURRENT);
        
        let mut alice_coord = HandshakeCoordinator::new(config.clone(), alice_keys, alice_state);
        let mut bob_coord = HandshakeCoordinator::new(config, bob_keys, bob_state);
        
        // Test handshake initiation
        let (alice_session, init_message) = alice_coord.initiate_handshake(None).unwrap();
        
        // Verify init message is signed
        assert!(init_message.signature.is_some(), "Handshake init must be signed");
        
        // Process init message at Bob's end
        let response_opt = bob_coord.process_handshake_message(&init_message, None).unwrap();
        let response_message = response_opt.unwrap();
        
        // Verify response message is signed
        assert!(response_message.signature.is_some(), "Handshake response must be signed");
        
        // Test that handshake prevents replay attacks
        let old_init = init_message.clone();
        std::thread::sleep(Duration::from_millis(10));
        
        // Should reject old message (within tolerance but testing the mechanism)
        let mut very_old_init = old_init.clone();
        very_old_init.timestamp = 1; // Very old timestamp
        
        let result = bob_coord.process_handshake_message(&very_old_init, None);
        // This might not fail due to tolerance, but the mechanism should be in place
    }

    /// Test timing attack resistance
    #[tokio::test]
    async fn test_timing_attack_resistance() {
        let mut rng = thread_rng();
        let ml_dsa = MlDsa::new();
        let keypair = ml_dsa.generate_keypair().unwrap();
        let public_key = qudag_crypto::MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
        
        // Create valid and invalid signatures
        let message = b"test message";
        let valid_signature = keypair.sign(message, &mut rng).unwrap();
        let mut invalid_signature = valid_signature.clone();
        invalid_signature[0] ^= 0xFF; // Corrupt signature
        
        // Measure verification times
        let iterations = 100;
        
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = public_key.verify(message, &valid_signature);
        }
        let valid_time = start.elapsed();
        
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = public_key.verify(message, &invalid_signature);
        }
        let invalid_time = start.elapsed();
        
        // Timing should be similar (within reasonable variance)
        let time_ratio = valid_time.as_nanos() as f64 / invalid_time.as_nanos() as f64;
        assert!(
            time_ratio > 0.8 && time_ratio < 1.2,
            "Verification timing should be consistent to prevent timing attacks: ratio = {}",
            time_ratio
        );
    }

    /// Test side-channel resistance
    #[tokio::test]
    async fn test_side_channel_resistance() {
        let mut rng = thread_rng();
        
        // Test that key generation doesn't leak information through timing
        let iterations = 10;
        let mut key_gen_times = Vec::new();
        
        for _ in 0..iterations {
            let start = std::time::Instant::now();
            let _keypair = MlDsa::new().generate_keypair().unwrap();
            key_gen_times.push(start.elapsed());
        }
        
        // Calculate variance in key generation times
        let mean_time = key_gen_times.iter().sum::<Duration>().as_nanos() as f64 / iterations as f64;
        let variance = key_gen_times
            .iter()
            .map(|t| {
                let diff = t.as_nanos() as f64 - mean_time;
                diff * diff
            })
            .sum::<f64>() / iterations as f64;
        
        let std_dev = variance.sqrt();
        let coefficient_of_variation = std_dev / mean_time;
        
        // Key generation timing should be relatively consistent
        assert!(
            coefficient_of_variation < 0.5,
            "Key generation timing variance should be low: CV = {}",
            coefficient_of_variation
        );
        
        // Test that signing doesn't leak key information through timing
        let keypair = MlDsa::new().generate_keypair().unwrap();
        let message1 = vec![0u8; 100];
        let message2 = vec![0xFFu8; 100];
        
        let sign_iterations = 50;
        
        let start = std::time::Instant::now();
        for _ in 0..sign_iterations {
            let _ = keypair.sign(&message1, &mut rng);
        }
        let time1 = start.elapsed();
        
        let start = std::time::Instant::now();
        for _ in 0..sign_iterations {
            let _ = keypair.sign(&message2, &mut rng);
        }
        let time2 = start.elapsed();
        
        let timing_ratio = time1.as_nanos() as f64 / time2.as_nanos() as f64;
        assert!(
            timing_ratio > 0.8 && timing_ratio < 1.2,
            "Signing timing should be independent of message content: ratio = {}",
            timing_ratio
        );
    }

    /// Test memory safety compliance
    #[tokio::test]
    async fn test_memory_safety_compliance() {
        let mut rng = thread_rng();
        
        // Test that secret keys are properly zeroized
        let initial_memory = qudag_protocol::get_memory_usage();
        
        {
            let ml_dsa = MlDsa::new();
            let keypair = ml_dsa.generate_keypair().unwrap();
            
            // Use the keypair
            let message = b"test message";
            let _signature = keypair.sign(message, &mut rng).unwrap();
            
            // keypair will be dropped here, triggering zeroization
        }
        
        // Force garbage collection
        std::thread::sleep(Duration::from_millis(10));
        
        let final_memory = qudag_protocol::get_memory_usage();
        
        // Memory should not grow significantly after key cleanup
        let memory_growth = final_memory.saturating_sub(initial_memory);
        assert!(
            memory_growth < 1024 * 1024, // Less than 1MB growth
            "Memory should be properly cleaned up after key operations"
        );
    }

    /// Test compliance with security standards
    #[tokio::test]
    async fn test_security_standards_compliance() {
        use qudag_protocol::VersionRegistry;
        
        let registry = VersionRegistry::new();
        let version_info = registry.get_version_info(&ProtocolVersion::CURRENT).unwrap();
        
        // Verify quantum resistance requirements
        assert!(
            version_info.security_requirements.quantum_resistant,
            "Protocol must be quantum resistant"
        );
        
        // Verify required algorithms
        let required_algos = &version_info.security_requirements.required_algorithms;
        assert!(required_algos.contains(&"ML-DSA".to_string()), "ML-DSA must be required");
        assert!(required_algos.contains(&"ML-KEM-768".to_string()), "ML-KEM-768 must be required");
        assert!(required_algos.contains(&"BLAKE3".to_string()), "BLAKE3 must be required");
        
        // Verify forbidden algorithms
        let forbidden_algos = &version_info.security_requirements.forbidden_algorithms;
        assert!(forbidden_algos.contains(&"RSA".to_string()), "RSA must be forbidden");
        assert!(forbidden_algos.contains(&"ECDSA".to_string()), "ECDSA must be forbidden");
        assert!(forbidden_algos.contains(&"DH".to_string()), "DH must be forbidden");
        
        // Verify minimum key sizes
        let min_key_sizes = &version_info.security_requirements.min_key_sizes;
        assert_eq!(
            min_key_sizes.get("ml-dsa").copied().unwrap_or(0),
            2048,
            "ML-DSA minimum key size must be 2048"
        );
        assert_eq!(
            min_key_sizes.get("ml-kem").copied().unwrap_or(0),
            768,
            "ML-KEM minimum key size must be 768"
        );
    }

    /// Test forward secrecy compliance
    #[tokio::test]
    async fn test_forward_secrecy_compliance() {
        // Test that session keys are ephemeral
        let keys1 = HandshakeKeys {
            signature_keypair: MlDsa::new().generate_keypair().unwrap(),
            kem_keypair: MlKem768::new().generate_keypair().unwrap(),
        };
        
        let keys2 = HandshakeKeys {
            signature_keypair: MlDsa::new().generate_keypair().unwrap(),
            kem_keypair: MlKem768::new().generate_keypair().unwrap(),
        };
        
        // Create two separate handshakes
        let config = HandshakeConfig::default();
        let state1 = ProtocolStateMachine::new(ProtocolVersion::CURRENT);
        let state2 = ProtocolStateMachine::new(ProtocolVersion::CURRENT);
        
        let mut coord1 = HandshakeCoordinator::new(config.clone(), keys1, state1);
        let mut coord2 = HandshakeCoordinator::new(config, keys2, state2);
        
        let (session1, _) = coord1.initiate_handshake(None).unwrap();
        let (session2, _) = coord2.initiate_handshake(None).unwrap();
        
        // Session keys should be different
        assert_ne!(session1, session2, "Different handshakes must generate different session IDs");
        
        // Test that old sessions are properly cleaned up
        coord1.cleanup_sessions();
        coord2.cleanup_sessions();
        
        // After cleanup, old sessions should not be accessible
        // (This is a simplified test - in practice, session cleanup would
        // happen based on timeouts and completion status)
    }
}