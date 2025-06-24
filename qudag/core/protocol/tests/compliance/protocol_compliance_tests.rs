//! Protocol compliance tests against QuDAG specifications.

use std::time::Duration;
use tokio::time::timeout;

use qudag_protocol::{
    Message, MessageType, ProtocolVersion, MessageFactory,
    HandshakeCoordinator, HandshakeConfig, HandshakeKeys,
    ProtocolStateMachine, ProtocolState, StateMachineConfig,
    VersionManager, VersionRegistry, CompatibilityAdapter,
};
use qudag_crypto::{MlDsa, MlKem768, KeyPair as KemKeyPair};

/// Test suite for protocol compliance
#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    /// Test message structure compliance
    #[tokio::test]
    async fn test_message_structure_compliance() {
        // Create a test message
        let message = MessageFactory::create_ping().unwrap();
        
        // Verify message structure requirements
        assert!(!message.id.is_nil(), "Message must have a unique ID");
        assert_eq!(message.version, ProtocolVersion::CURRENT, "Message must use current protocol version");
        assert!(message.timestamp > 0, "Message must have a valid timestamp");
        assert!(!message.payload.is_empty(), "Message must have a payload");
        
        // Test message serialization/deserialization
        let serialized = message.to_bytes().unwrap();
        let deserialized = Message::from_bytes(&serialized).unwrap();
        
        assert_eq!(message.id, deserialized.id, "Message ID must be preserved");
        assert_eq!(message.payload, deserialized.payload, "Message payload must be preserved");
        assert_eq!(message.timestamp, deserialized.timestamp, "Message timestamp must be preserved");
    }

    /// Test protocol version compatibility
    #[tokio::test]
    async fn test_protocol_version_compatibility() {
        let registry = VersionRegistry::new();
        
        // Test supported versions
        let v1_0_0 = ProtocolVersion { major: 1, minor: 0, patch: 0, features: vec![] };
        let v1_1_0 = ProtocolVersion { major: 1, minor: 1, patch: 0, features: vec![] };
        
        assert!(registry.is_supported(&v1_0_0), "Version 1.0.0 must be supported");
        assert!(registry.is_supported(&v1_1_0), "Version 1.1.0 must be supported");
        
        // Test version compatibility
        assert!(registry.are_compatible(&v1_0_0, &v1_1_0), "Versions 1.0.0 and 1.1.0 must be compatible");
        assert!(registry.are_compatible(&v1_1_0, &v1_0_0), "Compatibility must be bidirectional");
        
        // Test feature support
        assert!(registry.is_feature_supported(&v1_0_0, "basic-messaging"), "Basic messaging must be supported in 1.0.0");
        assert!(registry.is_feature_supported(&v1_1_0, "dark-addressing"), "Dark addressing must be supported in 1.1.0");
        assert!(!registry.is_feature_supported(&v1_0_0, "dark-addressing"), "Dark addressing must not be supported in 1.0.0");
    }

    /// Test handshake procedure compliance
    #[tokio::test]
    async fn test_handshake_procedure_compliance() {
        let mut rng = thread_rng();
        
        // Generate identity keys
        let ml_dsa = MlDsa::new();
        let signature_keypair = ml_dsa.generate_keypair().unwrap();
        let ml_kem = MlKem768::new();
        let kem_keypair = ml_kem.generate_keypair().unwrap();
        
        let identity_keys = HandshakeKeys {
            signature_keypair,
            kem_keypair,
        };
        
        // Create handshake coordinator
        let config = HandshakeConfig::default();
        let state_machine = ProtocolStateMachine::new(ProtocolVersion::CURRENT);
        let mut coordinator = HandshakeCoordinator::new(config, identity_keys, state_machine);
        
        // Test handshake initiation
        let (session_id, init_message) = coordinator.initiate_handshake(None).unwrap();
        
        // Verify handshake init message structure
        assert!(matches!(init_message.msg_type, MessageType::Handshake(_)), "Must be a handshake message");
        assert!(init_message.signature.is_some(), "Handshake message must be signed");
        assert!(!init_message.payload.is_empty(), "Handshake message must have payload");
        
        // Verify session creation
        let session = coordinator.get_session(&session_id).unwrap();
        assert_eq!(session.session_id, session_id, "Session ID must match");
        assert!(session.started_at.elapsed().unwrap() < Duration::from_secs(1), "Session must be recently created");
    }

    /// Test state machine compliance
    #[tokio::test]
    async fn test_state_machine_compliance() {
        let mut state_machine = ProtocolStateMachine::new(ProtocolVersion::CURRENT);
        
        // Test initial state
        assert_eq!(state_machine.current_state(), &ProtocolState::Initial, "Must start in Initial state");
        
        // Test valid state transitions
        state_machine.transition_to(
            ProtocolState::Handshake(qudag_protocol::state::HandshakeState::Waiting),
            "Starting handshake".to_string(),
        ).unwrap();
        
        assert!(matches!(
            state_machine.current_state(),
            ProtocolState::Handshake(qudag_protocol::state::HandshakeState::Waiting)
        ), "Must transition to Handshake/Waiting state");
        
        // Test invalid state transitions
        let result = state_machine.transition_to(
            ProtocolState::Active(qudag_protocol::state::ActiveState::Normal),
            "Invalid transition".to_string(),
        );
        
        assert!(result.is_err(), "Invalid state transitions must be rejected");
        
        // Test session management
        let session_id = state_machine.create_session(
            vec![1, 2, 3, 4],
            ProtocolVersion::CURRENT,
            vec!["basic-messaging".to_string()],
        ).unwrap();
        
        assert_eq!(state_machine.active_sessions(), 1, "Session count must be accurate");
        
        let session = state_machine.get_session(&session_id).unwrap();
        assert_eq!(session.peer_id, vec![1, 2, 3, 4], "Session peer ID must match");
    }

    /// Test message encryption and signing compliance
    #[tokio::test]
    async fn test_message_security_compliance() {
        let mut rng = thread_rng();
        
        // Generate keys
        let ml_dsa = MlDsa::new();
        let keypair = ml_dsa.generate_keypair().unwrap();
        let public_key = qudag_crypto::MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();
        
        // Create and sign message
        let mut message = MessageFactory::create_ping().unwrap();
        message.sign(&keypair).unwrap();
        
        // Verify signature compliance
        assert!(message.signature.is_some(), "Message must be signed");
        assert!(message.sender_key_hash.is_some(), "Message must include sender key hash");
        
        // Verify signature verification
        assert!(message.verify(&public_key).unwrap(), "Signature verification must succeed");
        
        // Test message validation
        message.validate().unwrap();
        
        // Test TTL compliance
        let mut ttl_message = MessageFactory::create_ping().unwrap().with_ttl(1); // 1ms TTL
        std::thread::sleep(Duration::from_millis(2));
        
        assert!(ttl_message.is_expired(), "Message must expire after TTL");
        assert!(ttl_message.validate().is_err(), "Expired message validation must fail");
    }

    /// Test performance requirements compliance
    #[tokio::test]
    async fn test_performance_compliance() {
        let start = std::time::Instant::now();
        
        // Test message processing speed
        for _ in 0..1000 {
            let message = MessageFactory::create_ping().unwrap();
            let _serialized = message.to_bytes().unwrap();
            let _deserialized = Message::from_bytes(&_serialized).unwrap();
        }
        
        let elapsed = start.elapsed();
        let messages_per_second = 1000.0 / elapsed.as_secs_f64();
        
        assert!(messages_per_second > 10000.0, "Must process at least 10,000 messages per second");
        
        // Test memory usage compliance
        let initial_memory = qudag_protocol::get_memory_usage();
        
        let mut messages = Vec::new();
        for _ in 0..1000 {
            messages.push(MessageFactory::create_ping().unwrap());
        }
        
        let memory_used = qudag_protocol::get_memory_usage() - initial_memory;
        let memory_per_message = memory_used / 1000;
        
        assert!(memory_per_message < 1024, "Memory usage per message must be less than 1KB");
        
        // Clean up
        drop(messages);
    }

    /// Test quantum resistance compliance
    #[tokio::test]
    async fn test_quantum_resistance_compliance() {
        let mut rng = thread_rng();
        
        // Verify ML-DSA usage
        let ml_dsa = MlDsa::new();
        let keypair = ml_dsa.generate_keypair().unwrap();
        
        assert_eq!(keypair.public_key().len(), qudag_crypto::ml_dsa::ML_DSA_PUBLIC_KEY_SIZE, "Must use correct ML-DSA key size");
        assert_eq!(keypair.secret_key().len(), qudag_crypto::ml_dsa::ML_DSA_SECRET_KEY_SIZE, "Must use correct ML-DSA secret key size");
        
        // Verify ML-KEM usage
        let ml_kem = MlKem768::new();
        let kem_keypair = ml_kem.generate_keypair().unwrap();
        
        let (shared_secret, ciphertext) = ml_kem.encapsulate(kem_keypair.public_key()).unwrap();
        let decrypted_secret = ml_kem.decapsulate(&ciphertext, kem_keypair.secret_key()).unwrap();
        
        assert_eq!(shared_secret.as_bytes(), decrypted_secret.as_bytes(), "KEM encapsulation/decapsulation must work correctly");
        
        // Test that only quantum-resistant algorithms are used
        let registry = VersionRegistry::new();
        let version_info = registry.get_version_info(&ProtocolVersion::CURRENT).unwrap();
        
        assert!(version_info.security_requirements.quantum_resistant, "Protocol must be quantum resistant");
        assert!(version_info.security_requirements.required_algorithms.contains(&"ML-DSA".to_string()), "ML-DSA must be required");
        assert!(version_info.security_requirements.required_algorithms.contains(&"ML-KEM-768".to_string()), "ML-KEM-768 must be required");
        assert!(version_info.security_requirements.forbidden_algorithms.contains(&"RSA".to_string()), "RSA must be forbidden");
        assert!(version_info.security_requirements.forbidden_algorithms.contains(&"ECDSA".to_string()), "ECDSA must be forbidden");
    }

    /// Test backward compatibility compliance
    #[tokio::test]
    async fn test_backward_compatibility_compliance() {
        let version_manager = VersionManager::new(ProtocolVersion::CURRENT);
        let adapter = CompatibilityAdapter::new(version_manager);
        
        // Test version compatibility
        let v1_0_0 = ProtocolVersion { major: 1, minor: 0, patch: 0, features: vec![] };
        let v1_1_0 = ProtocolVersion { major: 1, minor: 1, patch: 0, features: vec![] };
        
        let compatibility_notes = adapter.check_compatibility(&v1_0_0, &v1_1_0).unwrap();
        assert!(!compatibility_notes.is_empty(), "Compatibility check must provide information");
        
        // Test message transformation
        let mut message = MessageFactory::create_ping().unwrap();
        message.version = v1_1_0.clone();
        
        let transformed = adapter.transform_message(&message, &v1_0_0).unwrap();
        assert_eq!(transformed.version, v1_0_0, "Message version must be transformed");
        
        // Test legacy format conversion
        let legacy = adapter.to_legacy_format(&message).unwrap();
        let restored = adapter.from_legacy_format(&legacy).unwrap();
        
        assert_eq!(message.payload, restored.payload, "Legacy conversion must preserve payload");
        assert_eq!(message.timestamp, restored.timestamp, "Legacy conversion must preserve timestamp");
    }

    /// Test network compliance
    #[tokio::test]
    async fn test_network_compliance() {
        // Test message size limits
        let large_payload = vec![0u8; 2 * 1024 * 1024]; // 2MB
        let large_message = Message::new(
            MessageType::Control(qudag_protocol::message::ControlMessageType::Ping),
            large_payload,
        );
        
        assert!(large_message.validate().is_err(), "Messages larger than 1MB must be rejected");
        
        // Test timeout compliance
        let timeout_duration = Duration::from_millis(100);
        let start = std::time::Instant::now();
        
        let result = timeout(timeout_duration, async {
            // Simulate long-running operation
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<(), ()>(())
        }).await;
        
        assert!(result.is_err(), "Operations must respect timeout limits");
        assert!(start.elapsed() <= Duration::from_millis(150), "Timeout must be enforced promptly");
    }

    /// Test concurrency and thread safety compliance
    #[tokio::test]
    async fn test_concurrency_compliance() {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        
        let state_machine = Arc::new(Mutex::new(ProtocolStateMachine::new(ProtocolVersion::CURRENT)));
        let mut handles = Vec::new();
        
        // Test concurrent session creation
        for i in 0..10 {
            let state_machine = state_machine.clone();
            let handle = tokio::spawn(async move {
                let mut sm = state_machine.lock().await;
                sm.create_session(
                    vec![i],
                    ProtocolVersion::CURRENT,
                    vec!["basic-messaging".to_string()],
                ).unwrap()
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let session_ids: Vec<_> = futures::future::join_all(handles).await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        
        assert_eq!(session_ids.len(), 10, "All concurrent operations must succeed");
        
        let sm = state_machine.lock().await;
        assert_eq!(sm.active_sessions(), 10, "All sessions must be created");
        
        // Verify all session IDs are unique
        let mut unique_ids = session_ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(unique_ids.len(), session_ids.len(), "All session IDs must be unique");
    }

    /// Test error handling compliance
    #[tokio::test]
    async fn test_error_handling_compliance() {
        // Test invalid protocol version
        let invalid_version = ProtocolVersion { major: 99, minor: 99, patch: 99, features: vec![] };
        let registry = VersionRegistry::new();
        
        assert!(!registry.is_supported(&invalid_version), "Invalid versions must not be supported");
        
        // Test malformed message handling
        let invalid_data = vec![0xFF; 100];
        let result = Message::from_bytes(&invalid_data);
        
        assert!(result.is_err(), "Malformed messages must be rejected");
        
        // Test resource limits
        let mut state_machine = ProtocolStateMachine::new(ProtocolVersion::CURRENT);
        let config = StateMachineConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let mut limited_state_machine = ProtocolStateMachine::with_config(ProtocolVersion::CURRENT, config);
        
        // Create maximum sessions
        for i in 0..2 {
            limited_state_machine.create_session(
                vec![i],
                ProtocolVersion::CURRENT,
                vec!["basic-messaging".to_string()],
            ).unwrap();
        }
        
        // Try to create one more (should fail)
        let result = limited_state_machine.create_session(
            vec![99],
            ProtocolVersion::CURRENT,
            vec!["basic-messaging".to_string()],
        );
        
        assert!(result.is_err(), "Session creation beyond limits must fail");
    }

    /// Test compliance with QuDAG-specific requirements
    #[tokio::test]
    async fn test_qudag_specific_compliance() {
        // Test DAG consensus message support
        let vertex_id = [1u8; 32]; // Mock vertex ID
        let vertex_data = vec![1, 2, 3, 4];
        let parent_vertices = vec![[0u8; 32], [2u8; 32]]; // Mock parent vertex IDs
        
        let consensus_message = MessageFactory::create_vertex_proposal(
            vertex_id,
            vertex_data.clone(),
            parent_vertices.clone(),
        ).unwrap();
        
        assert!(matches!(
            consensus_message.msg_type,
            MessageType::Consensus(qudag_protocol::message::ConsensusMessageType::VertexProposal)
        ), "Must support DAG consensus messages");
        
        // Test anonymous routing support
        let anonymous_message = Message::new(
            MessageType::Anonymous(qudag_protocol::message::AnonymousMessageType::Data),
            vec![1, 2, 3, 4],
        );
        
        assert!(matches!(
            anonymous_message.msg_type,
            MessageType::Anonymous(_)
        ), "Must support anonymous messaging");
        
        // Test dark addressing feature flag
        let registry = VersionRegistry::new();
        let v1_1_0 = ProtocolVersion { major: 1, minor: 1, patch: 0, features: vec![] };
        
        assert!(registry.is_feature_supported(&v1_1_0, "dark-addressing"), "Dark addressing must be supported in 1.1.0");
        
        // Test protocol versioning for QuDAG features
        assert!(registry.is_feature_supported(&ProtocolVersion::CURRENT, "dag-consensus"), "DAG consensus must be supported");
        assert!(registry.is_feature_supported(&ProtocolVersion::CURRENT, "quantum-resistant-crypto"), "Quantum resistance must be supported");
    }
}