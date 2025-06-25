#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use prime_core::types::*;
use std::collections::HashMap;

/// Fuzz input for serialization testing
#[derive(Arbitrary, Debug)]
struct SerializationFuzzInput {
    message_types: Vec<FuzzMessageType>,
}

#[derive(Arbitrary, Debug)]
enum FuzzMessageType {
    GradientUpdate(FuzzGradientUpdate),
    TrainingConfig(FuzzTrainingConfig),
    ModelMetadata(FuzzModelMetadata),
    TrainingMetrics(FuzzTrainingMetrics),
}

#[derive(Arbitrary, Debug)]
struct FuzzGradientUpdate {
    node_id: String,
    model_version: u64,
    round: u64,
    gradient_data: Vec<(String, Vec<f32>)>,
    loss: f32,
    accuracy: f32,
    samples: usize,
    time_ms: u64,
    timestamp: u64,
}

#[derive(Arbitrary, Debug)]
struct FuzzTrainingConfig {
    batch_size: usize,
    learning_rate: f32,
    epochs: usize,
    optimizer_type: u8,
    aggregation_type: u8,
}

#[derive(Arbitrary, Debug)]
struct FuzzModelMetadata {
    id: String,
    version: u64,
    architecture: String,
    parameters_count: usize,
    created_at: u64,
    updated_at: u64,
}

#[derive(Arbitrary, Debug)]
struct FuzzTrainingMetrics {
    loss: f32,
    accuracy: f32,
    samples_processed: usize,
    computation_time_ms: u64,
}

fuzz_target!(|input: SerializationFuzzInput| {
    // Limit number of messages to prevent timeout
    if input.message_types.len() > 100 {
        return;
    }
    
    for fuzz_msg in input.message_types {
        match fuzz_msg {
            FuzzMessageType::GradientUpdate(fuzz_update) => {
                test_gradient_update_serialization(fuzz_update);
            }
            FuzzMessageType::TrainingConfig(fuzz_config) => {
                test_training_config_serialization(fuzz_config);
            }
            FuzzMessageType::ModelMetadata(fuzz_metadata) => {
                test_model_metadata_serialization(fuzz_metadata);
            }
            FuzzMessageType::TrainingMetrics(fuzz_metrics) => {
                test_training_metrics_serialization(fuzz_metrics);
            }
        }
    }
});

fn test_gradient_update_serialization(fuzz_update: FuzzGradientUpdate) {
    // Limit string lengths and collection sizes
    if fuzz_update.node_id.len() > 1000 || fuzz_update.gradient_data.len() > 100 {
        return;
    }
    
    let mut gradients = HashMap::new();
    for (key, values) in fuzz_update.gradient_data {
        if key.len() > 100 || values.len() > 1000 {
            continue;
        }
        
        // Handle special float values
        let clean_values: Vec<f32> = values
            .into_iter()
            .map(|v| {
                if v.is_nan() || v.is_infinite() {
                    0.0
                } else {
                    v
                }
            })
            .collect();
        
        gradients.insert(key, clean_values);
    }
    
    let update = GradientUpdate {
        node_id: NodeId::new(fuzz_update.node_id),
        model_version: fuzz_update.model_version,
        round: fuzz_update.round,
        gradients,
        metrics: TrainingMetrics {
            loss: if fuzz_update.loss.is_finite() { fuzz_update.loss } else { 0.0 },
            accuracy: if fuzz_update.accuracy.is_finite() { 
                fuzz_update.accuracy.max(0.0).min(1.0)
            } else { 
                0.5 
            },
            samples_processed: fuzz_update.samples,
            computation_time_ms: fuzz_update.time_ms,
        },
        timestamp: fuzz_update.timestamp,
    };
    
    // Test JSON serialization
    if let Ok(json) = serde_json::to_string(&update) {
        // Test deserialization
        let _: Result<GradientUpdate, _> = serde_json::from_str(&json);
    }
    
    // Test MessagePack serialization if available
    #[cfg(feature = "msgpack")]
    {
        if let Ok(msgpack) = rmp_serde::to_vec(&update) {
            let _: Result<GradientUpdate, _> = rmp_serde::from_slice(&msgpack);
        }
    }
}

fn test_training_config_serialization(fuzz_config: FuzzTrainingConfig) {
    // Validate and clamp values
    let batch_size = fuzz_config.batch_size.max(1).min(10000);
    let learning_rate = if fuzz_config.learning_rate.is_finite() && fuzz_config.learning_rate > 0.0 {
        fuzz_config.learning_rate.min(10.0)
    } else {
        0.001
    };
    let epochs = fuzz_config.epochs.max(1).min(10000);
    
    let optimizer = match fuzz_config.optimizer_type % 3 {
        0 => OptimizerType::Sgd { momentum: 0.9 },
        1 => OptimizerType::Adam { beta1: 0.9, beta2: 0.999 },
        _ => OptimizerType::AdamW { beta1: 0.9, beta2: 0.999, weight_decay: 0.01 },
    };
    
    let aggregation = match fuzz_config.aggregation_type % 4 {
        0 => AggregationStrategy::FederatedAveraging,
        1 => AggregationStrategy::SecureAggregation,
        2 => AggregationStrategy::TrimmedMean { trim_ratio: 0.1 },
        _ => AggregationStrategy::Krum { selection_count: 5 },
    };
    
    let config = TrainingConfig {
        batch_size,
        learning_rate,
        epochs,
        optimizer,
        aggregation_strategy: aggregation,
    };
    
    // Test serialization
    if let Ok(json) = serde_json::to_string(&config) {
        let _: Result<TrainingConfig, _> = serde_json::from_str(&json);
    }
}

fn test_model_metadata_serialization(fuzz_metadata: FuzzModelMetadata) {
    // Limit string lengths
    if fuzz_metadata.id.len() > 1000 || fuzz_metadata.architecture.len() > 1000 {
        return;
    }
    
    let metadata = ModelMetadata {
        id: fuzz_metadata.id,
        version: fuzz_metadata.version,
        architecture: fuzz_metadata.architecture,
        parameters_count: fuzz_metadata.parameters_count,
        created_at: fuzz_metadata.created_at,
        updated_at: fuzz_metadata.updated_at,
    };
    
    // Test serialization
    if let Ok(json) = serde_json::to_string(&metadata) {
        let _: Result<ModelMetadata, _> = serde_json::from_str(&json);
    }
}

fn test_training_metrics_serialization(fuzz_metrics: FuzzTrainingMetrics) {
    let metrics = TrainingMetrics {
        loss: if fuzz_metrics.loss.is_finite() { fuzz_metrics.loss } else { 0.0 },
        accuracy: if fuzz_metrics.accuracy.is_finite() { 
            fuzz_metrics.accuracy.max(0.0).min(1.0)
        } else { 
            0.5 
        },
        samples_processed: fuzz_metrics.samples_processed,
        computation_time_ms: fuzz_metrics.computation_time_ms,
    };
    
    // Test serialization
    if let Ok(json) = serde_json::to_string(&metrics) {
        let _: Result<TrainingMetrics, _> = serde_json::from_str(&json);
    }
    
    // Test that metrics are within valid ranges after deserialization
    if let Ok(json) = serde_json::to_string(&metrics) {
        if let Ok(deserialized) = serde_json::from_str::<TrainingMetrics>(&json) {
            assert!(deserialized.accuracy >= 0.0 && deserialized.accuracy <= 1.0);
            assert!(!deserialized.loss.is_nan());
        }
    }
}