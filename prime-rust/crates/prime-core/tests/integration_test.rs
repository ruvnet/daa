//! Integration tests for prime-core

use prime_core::{*, protocol::*, types::*};
use std::collections::HashMap;
use tokio;
use proptest::prelude::*;

#[tokio::test]
async fn test_protocol_message_flow() {
    let node1 = NodeId::new("node1");
    let node2 = NodeId::new("node2");
    
    // Create a gradient update
    let gradient_update = GradientUpdate {
        node_id: node1.clone(),
        model_version: 1,
        round: 10,
        gradients: HashMap::from([
            ("layer1".to_string(), vec![0.1, 0.2, 0.3]),
            ("layer2".to_string(), vec![0.4, 0.5, 0.6]),
        ]),
        metrics: TrainingMetrics {
            loss: 0.5,
            accuracy: 0.85,
            samples_processed: 1000,
            computation_time_ms: 500,
        },
        timestamp: 1234567890,
    };
    
    // Wrap in protocol message
    let msg = ProtocolMessage::new(
        node1.clone(),
        MessageType::GradientUpdate(gradient_update),
    ).with_recipient(node2.clone());
    
    // Serialize and deserialize
    let serialized = serde_json::to_string(&msg).unwrap();
    let deserialized: ProtocolMessage = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(msg.sender, deserialized.sender);
    assert_eq!(msg.recipient, deserialized.recipient);
    
    if let MessageType::GradientUpdate(update) = deserialized.message_type {
        assert_eq!(update.round, 10);
        assert_eq!(update.metrics.accuracy, 0.85);
        assert_eq!(update.gradients.len(), 2);
    } else {
        panic!("Wrong message type");
    }
}

#[tokio::test]
async fn test_training_configuration_validation() {
    let valid_configs = vec![
        TrainingConfig {
            batch_size: 32,
            learning_rate: 0.001,
            epochs: 10,
            optimizer: OptimizerType::Adam { beta1: 0.9, beta2: 0.999 },
            aggregation_strategy: AggregationStrategy::FederatedAveraging,
        },
        TrainingConfig {
            batch_size: 64,
            learning_rate: 0.01,
            epochs: 100,
            optimizer: OptimizerType::Sgd { momentum: 0.9 },
            aggregation_strategy: AggregationStrategy::TrimmedMean { trim_ratio: 0.1 },
        },
        TrainingConfig {
            batch_size: 128,
            learning_rate: 0.0001,
            epochs: 50,
            optimizer: OptimizerType::AdamW { 
                beta1: 0.9, 
                beta2: 0.999, 
                weight_decay: 0.01 
            },
            aggregation_strategy: AggregationStrategy::Krum { selection_count: 5 },
        },
    ];
    
    for config in valid_configs {
        assert!(config.batch_size > 0);
        assert!(config.learning_rate > 0.0);
        assert!(config.epochs > 0);
        
        // Test serialization
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: TrainingConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.batch_size, deserialized.batch_size);
    }
}

#[test]
fn test_model_metadata_lifecycle() {
    let mut metadata = ModelMetadata {
        id: "model_001".to_string(),
        version: 1,
        architecture: "ResNet50".to_string(),
        parameters_count: 25_000_000,
        created_at: 1000,
        updated_at: 1000,
    };
    
    // Simulate updates
    for i in 2..=10 {
        metadata.version = i;
        metadata.updated_at = 1000 + i * 100;
        
        assert!(metadata.updated_at > metadata.created_at);
        assert_eq!(metadata.version, i);
    }
}

// Property-based tests
proptest! {
    #[test]
    fn test_node_id_invariants(
        id in "[a-zA-Z0-9_-]{1,50}"
    ) {
        let node_id = NodeId::new(&id);
        assert_eq!(node_id.0, id);
        
        // Test equality
        let node_id2 = NodeId::new(&id);
        assert_eq!(node_id, node_id2);
        
        // Test hashing
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher1 = DefaultHasher::new();
        node_id.hash(&mut hasher1);
        let hash1 = hasher1.finish();
        
        let mut hasher2 = DefaultHasher::new();
        node_id2.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_training_metrics_bounds(
        loss in 0.0f32..1000.0f32,
        accuracy in 0.0f32..1.0f32,
        samples in 1usize..1_000_000usize,
        time_ms in 1u64..3_600_000u64,
    ) {
        let metrics = TrainingMetrics {
            loss,
            accuracy,
            samples_processed: samples,
            computation_time_ms: time_ms,
        };
        
        // Verify bounds
        assert!(metrics.loss >= 0.0);
        assert!(metrics.accuracy >= 0.0 && metrics.accuracy <= 1.0);
        assert!(metrics.samples_processed > 0);
        assert!(metrics.computation_time_ms > 0);
        
        // Test serialization
        let json = serde_json::to_string(&metrics).unwrap();
        let decoded: TrainingMetrics = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metrics.loss, decoded.loss);
        assert_eq!(metrics.accuracy, decoded.accuracy);
        assert_eq!(metrics.samples_processed, decoded.samples_processed);
        assert_eq!(metrics.computation_time_ms, decoded.computation_time_ms);
    }
    
    #[test]
    fn test_gradient_update_consistency(
        rounds in prop::collection::vec(1u64..1000u64, 10..50)
    ) {
        let node_id = NodeId::new("test_node");
        let mut prev_round = 0u64;
        
        for round in rounds {
            let update = GradientUpdate {
                node_id: node_id.clone(),
                model_version: 1,
                round,
                gradients: HashMap::new(),
                metrics: TrainingMetrics {
                    loss: 1.0,
                    accuracy: 0.5,
                    samples_processed: 100,
                    computation_time_ms: 50,
                },
                timestamp: round * 1000,
            };
            
            // Rounds should be increasing
            if prev_round > 0 {
                assert!(round != prev_round, "Duplicate rounds not allowed");
            }
            prev_round = round;
            
            // Timestamp should correlate with round
            assert_eq!(update.timestamp, round * 1000);
        }
    }
}

// Fuzzing-like property test for message handling
proptest! {
    #[test]
    fn test_message_type_exhaustive_handling(
        msg_variant in 0..20,
        round in 0u64..1000u64,
        accept in prop::bool::ANY,
        key_len in 1usize..100usize,
        value_len in 0usize..1000usize,
    ) {
        let message = match msg_variant % 10 {
            0 => MessageType::Ping,
            1 => MessageType::Pong,
            2 => MessageType::DhtGet { 
                key: vec![0u8; key_len] 
            },
            3 => MessageType::DhtPut { 
                key: vec![0u8; key_len], 
                value: vec![1u8; value_len] 
            },
            4 => MessageType::DhtResponse { 
                key: vec![0u8; key_len], 
                value: if value_len > 0 { Some(vec![1u8; value_len]) } else { None }
            },
            5 => MessageType::ConsensusProposal { 
                round, 
                value: vec![2u8; value_len.min(100)] 
            },
            6 => MessageType::ConsensusVote { round, accept },
            7 => MessageType::ConsensusCommit { 
                round, 
                value: vec![3u8; value_len.min(100)] 
            },
            8 => MessageType::JoinRequest { 
                capabilities: vec!["compute".to_string(), "storage".to_string()] 
            },
            _ => MessageType::JoinResponse { accepted: accept },
        };
        
        // Ensure all message types can be serialized and deserialized
        let json = serde_json::to_string(&message).unwrap();
        let decoded: MessageType = serde_json::from_str(&json).unwrap();
        
        // Basic structural check
        match (message, decoded) {
            (MessageType::Ping, MessageType::Ping) => {},
            (MessageType::Pong, MessageType::Pong) => {},
            (MessageType::DhtGet { key: k1 }, MessageType::DhtGet { key: k2 }) => {
                assert_eq!(k1, k2);
            },
            (MessageType::ConsensusVote { round: r1, accept: a1 }, 
             MessageType::ConsensusVote { round: r2, accept: a2 }) => {
                assert_eq!(r1, r2);
                assert_eq!(a1, a2);
            },
            _ => {
                // For other types, just ensure they deserialize to same variant
                assert_eq!(
                    std::mem::discriminant(&message), 
                    std::mem::discriminant(&decoded)
                );
            }
        }
    }
}