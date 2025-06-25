//! Core types used throughout the Prime system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node identifier in the network
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub version: u64,
    pub architecture: String,
    pub parameters_count: usize,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Training parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub batch_size: usize,
    pub learning_rate: f32,
    pub epochs: usize,
    pub optimizer: OptimizerType,
    pub aggregation_strategy: AggregationStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizerType {
    Sgd { momentum: f32 },
    Adam { beta1: f32, beta2: f32 },
    AdamW { beta1: f32, beta2: f32, weight_decay: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    FederatedAveraging,
    SecureAggregation,
    TrimmedMean { trim_ratio: f32 },
    Krum { selection_count: usize },
}

/// Gradient update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientUpdate {
    pub node_id: NodeId,
    pub model_version: u64,
    pub round: u64,
    pub gradients: HashMap<String, Vec<f32>>,
    pub metrics: TrainingMetrics,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    pub loss: f32,
    pub accuracy: f32,
    pub samples_processed: usize,
    pub computation_time_ms: u64,
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    // Training messages
    GradientUpdate(GradientUpdate),
    ModelSync { version: u64, parameters: Vec<u8> },
    TrainingRequest(TrainingConfig),
    
    // Consensus messages
    ConsensusProposal { round: u64, value: Vec<u8> },
    ConsensusVote { round: u64, accept: bool },
    ConsensusCommit { round: u64, value: Vec<u8> },
    
    // DHT messages
    DhtPut { key: Vec<u8>, value: Vec<u8> },
    DhtGet { key: Vec<u8> },
    DhtResponse { key: Vec<u8>, value: Option<Vec<u8>> },
    
    // Control messages
    Ping,
    Pong,
    JoinRequest { capabilities: Vec<String> },
    JoinResponse { accepted: bool },
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use test_case::test_case;

    #[test]
    fn test_node_id_creation() {
        let id = NodeId::new("node1");
        assert_eq!(id.0, "node1");
    }

    #[test_case(10, 0.01, 5 ; "small batch")]
    #[test_case(32, 0.001, 10 ; "medium batch")]
    #[test_case(128, 0.0001, 100 ; "large batch")]
    fn test_training_config(batch_size: usize, lr: f32, epochs: usize) {
        let config = TrainingConfig {
            batch_size,
            learning_rate: lr,
            epochs,
            optimizer: OptimizerType::Adam { beta1: 0.9, beta2: 0.999 },
            aggregation_strategy: AggregationStrategy::FederatedAveraging,
        };
        
        assert_eq!(config.batch_size, batch_size);
        assert_eq!(config.learning_rate, lr);
        assert_eq!(config.epochs, epochs);
    }

    proptest! {
        #[test]
        fn test_gradient_update_serialization(
            node_id in "[a-zA-Z0-9]{5,20}",
            version in 0u64..1000u64,
            round in 0u64..1000u64,
            loss in 0.0f32..100.0f32,
            accuracy in 0.0f32..1.0f32,
        ) {
            let update = GradientUpdate {
                node_id: NodeId::new(node_id),
                model_version: version,
                round,
                gradients: HashMap::new(),
                metrics: TrainingMetrics {
                    loss,
                    accuracy,
                    samples_processed: 1000,
                    computation_time_ms: 100,
                },
                timestamp: 1234567890,
            };

            let serialized = serde_json::to_string(&update).unwrap();
            let deserialized: GradientUpdate = serde_json::from_str(&serialized).unwrap();
            
            assert_eq!(update.node_id, deserialized.node_id);
            assert_eq!(update.model_version, deserialized.model_version);
            assert_eq!(update.round, deserialized.round);
        }

        #[test]
        fn test_message_type_variants(msg_type in 0..10) {
            let message = match msg_type % 10 {
                0 => MessageType::Ping,
                1 => MessageType::Pong,
                2 => MessageType::DhtGet { key: vec![1, 2, 3] },
                3 => MessageType::DhtPut { key: vec![1, 2, 3], value: vec![4, 5, 6] },
                4 => MessageType::ConsensusProposal { round: 1, value: vec![7, 8, 9] },
                5 => MessageType::ConsensusVote { round: 1, accept: true },
                6 => MessageType::ConsensusCommit { round: 1, value: vec![10, 11, 12] },
                7 => MessageType::ModelSync { version: 1, parameters: vec![13, 14, 15] },
                8 => MessageType::JoinRequest { capabilities: vec!["compute".to_string()] },
                _ => MessageType::JoinResponse { accepted: true },
            };

            let serialized = serde_json::to_string(&message).unwrap();
            let deserialized: MessageType = serde_json::from_str(&serialized).unwrap();
            
            // Basic check that serialization works
            assert!(!serialized.is_empty());
        }
    }
}