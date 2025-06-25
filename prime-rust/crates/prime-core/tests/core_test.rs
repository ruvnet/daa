use prime_core::{ModelMetadata, NodeIdentity, TrainingConfig};

#[tokio::test]
async fn test_model_metadata_serialization() {
    let metadata = ModelMetadata {
        id: "model-001".to_string(),
        version: "1.0.0".to_string(),
        parameters: 1_000_000,
        architecture: "transformer".to_string(),
    };
    
    let serialized = serde_json::to_string(&metadata).unwrap();
    let deserialized: ModelMetadata = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(metadata, deserialized);
}

#[tokio::test]
async fn test_node_identity_creation() {
    let node = NodeIdentity::new("node-001");
    assert_eq!(node.id(), "node-001");
    assert!(node.is_active());
}

#[tokio::test]
async fn test_training_config_validation() {
    let config = TrainingConfig {
        batch_size: 32,
        learning_rate: 0.001,
        epochs: 10,
        gradient_accumulation_steps: 4,
    };
    
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_invalid_training_config() {
    let config = TrainingConfig {
        batch_size: 0, // Invalid
        learning_rate: 0.001,
        epochs: 10,
        gradient_accumulation_steps: 4,
    };
    
    assert!(config.validate().is_err());
}