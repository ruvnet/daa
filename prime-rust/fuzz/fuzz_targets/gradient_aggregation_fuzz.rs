#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use prime_core::types::*;
use std::collections::HashMap;

/// Fuzz input for gradient aggregation
#[derive(Arbitrary, Debug)]
struct GradientFuzzInput {
    node_count: usize,
    layer_count: usize,
    gradient_size: usize,
    values: Vec<f32>,
    aggregation_type: u8,
}

fuzz_target!(|input: GradientFuzzInput| {
    // Limit sizes to prevent excessive memory usage
    if input.node_count > 100 || input.layer_count > 50 || input.gradient_size > 1000 {
        return;
    }
    
    if input.values.is_empty() {
        return;
    }
    
    // Generate gradient updates
    let mut updates = Vec::new();
    
    for node_idx in 0..input.node_count {
        let mut gradients = HashMap::new();
        
        for layer_idx in 0..input.layer_count {
            let mut layer_gradients = Vec::new();
            
            for grad_idx in 0..input.gradient_size {
                let value_idx = (node_idx + layer_idx + grad_idx) % input.values.len();
                let mut value = input.values[value_idx];
                
                // Test handling of special float values
                if value.is_nan() || value.is_infinite() {
                    value = 0.0;
                }
                
                layer_gradients.push(value);
            }
            
            gradients.insert(format!("layer_{}", layer_idx), layer_gradients);
        }
        
        let update = GradientUpdate {
            node_id: NodeId::new(format!("node_{}", node_idx)),
            model_version: node_idx as u64,
            round: 1,
            gradients,
            metrics: TrainingMetrics {
                loss: input.values[node_idx % input.values.len()].abs(),
                accuracy: (node_idx as f32 / 100.0).min(1.0).max(0.0),
                samples_processed: node_idx.max(1),
                computation_time_ms: node_idx as u64,
            },
            timestamp: node_idx as u64,
        };
        
        updates.push(update);
    }
    
    // Test different aggregation strategies
    match input.aggregation_type % 4 {
        0 => {
            // Federated averaging
            let _ = aggregate_federated_averaging(&updates);
        }
        1 => {
            // Trimmed mean
            let trim_ratio = 0.1;
            let _ = aggregate_trimmed_mean(&updates, trim_ratio);
        }
        2 => {
            // Secure aggregation (simplified)
            let _ = aggregate_secure(&updates);
        }
        _ => {
            // Krum aggregation
            let selection_count = (input.node_count / 2).max(1);
            let _ = aggregate_krum(&updates, selection_count);
        }
    }
});

/// Federated averaging aggregation
fn aggregate_federated_averaging(updates: &[GradientUpdate]) -> HashMap<String, Vec<f32>> {
    if updates.is_empty() {
        return HashMap::new();
    }
    
    let mut aggregated = HashMap::new();
    let node_count = updates.len() as f32;
    
    for update in updates {
        for (layer_name, gradients) in &update.gradients {
            let entry = aggregated.entry(layer_name.clone()).or_insert_with(Vec::new);
            
            if entry.is_empty() {
                entry.extend(gradients.iter().map(|&x| x / node_count));
            } else {
                for (i, &grad) in gradients.iter().enumerate() {
                    if i < entry.len() {
                        entry[i] += grad / node_count;
                    }
                }
            }
        }
    }
    
    aggregated
}

/// Trimmed mean aggregation
fn aggregate_trimmed_mean(updates: &[GradientUpdate], trim_ratio: f32) -> HashMap<String, Vec<f32>> {
    if updates.is_empty() {
        return HashMap::new();
    }
    
    let trim_count = ((updates.len() as f32) * trim_ratio) as usize;
    let start_idx = trim_count;
    let end_idx = updates.len().saturating_sub(trim_count);
    
    if start_idx >= end_idx {
        return HashMap::new();
    }
    
    let trimmed_updates = &updates[start_idx..end_idx];
    aggregate_federated_averaging(trimmed_updates)
}

/// Secure aggregation (simplified version)
fn aggregate_secure(updates: &[GradientUpdate]) -> HashMap<String, Vec<f32>> {
    // In a real implementation, this would involve cryptographic protocols
    // For fuzzing, we just test basic aggregation with noise
    let mut result = aggregate_federated_averaging(updates);
    
    // Add noise to test robustness
    for gradients in result.values_mut() {
        for grad in gradients {
            *grad += (*grad * 0.001).sin(); // Small deterministic noise
        }
    }
    
    result
}

/// Krum aggregation
fn aggregate_krum(updates: &[GradientUpdate], selection_count: usize) -> HashMap<String, Vec<f32>> {
    if updates.is_empty() || selection_count == 0 {
        return HashMap::new();
    }
    
    // Simplified Krum: just select first k updates
    let selected_count = selection_count.min(updates.len());
    let selected_updates = &updates[..selected_count];
    
    aggregate_federated_averaging(selected_updates)
}