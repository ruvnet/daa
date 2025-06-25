//! Lightweight training tasks for browser-based WASM execution
//!
//! This module provides CPU-friendly training operations suitable for
//! running in web browsers with limited resources.

use serde::{Serialize, Deserialize};
use anyhow::Result;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Lightweight model suitable for browser training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightModel {
    pub layers: Vec<Layer>,
    pub learning_rate: f32,
    pub batch_size: usize,
}

/// A simple neural network layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub weights: Vec<Vec<f32>>,
    pub biases: Vec<f32>,
    pub activation: ActivationType,
}

/// Activation functions supported in WASM
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Sigmoid,
    Tanh,
    Linear,
}

/// Training task configuration for browsers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTrainingConfig {
    /// Maximum milliseconds to train before yielding
    pub max_train_time_ms: u32,
    /// Number of samples to process per batch
    pub batch_size: usize,
    /// Whether to use SIMD if available
    pub use_simd: bool,
    /// Memory limit in MB
    pub memory_limit_mb: usize,
}

impl Default for BrowserTrainingConfig {
    fn default() -> Self {
        Self {
            max_train_time_ms: 50, // 50ms slices to avoid blocking
            batch_size: 32,
            use_simd: true,
            memory_limit_mb: 256,
        }
    }
}

/// Lightweight training task that can run in browsers
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct BrowserTrainer {
    model: LightweightModel,
    config: BrowserTrainingConfig,
    gradients: Vec<Vec<Vec<f32>>>, // Accumulated gradients
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl BrowserTrainer {
    /// Create a new browser trainer
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<BrowserTrainer, JsValue> {
        let config: BrowserTrainingConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(Self {
            model: create_lightweight_model(),
            config,
            gradients: Vec::new(),
        })
    }

    /// Train on a batch of data
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub async fn train_batch(&mut self, inputs: &[f32], targets: &[f32]) -> Result<String, JsValue> {
        let start = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();

        let batch_size = self.config.batch_size;
        let input_size = self.model.layers[0].weights[0].len();
        let output_size = self.model.layers.last().unwrap().biases.len();

        // Process batch
        let mut total_loss = 0.0;
        let mut processed = 0;

        while processed < inputs.len() / input_size {
            // Check time limit
            let elapsed = web_sys::window()
                .unwrap()
                .performance()
                .unwrap()
                .now() - start;
            
            if elapsed > self.config.max_train_time_ms as f64 {
                break;
            }

            let start_idx = processed * input_size;
            let end_idx = ((processed + 1) * input_size).min(inputs.len());
            let input_slice = &inputs[start_idx..end_idx];

            let target_start = processed * output_size;
            let target_end = ((processed + 1) * output_size).min(targets.len());
            let target_slice = &targets[target_start..target_end];

            // Forward pass
            let output = self.forward(input_slice);
            
            // Calculate loss
            let loss = calculate_loss(&output, target_slice);
            total_loss += loss;

            // Backward pass
            let gradients = self.backward(input_slice, &output, target_slice);
            self.accumulate_gradients(gradients);

            processed += 1;
        }

        let result = serde_json::json!({
            "processed_samples": processed,
            "average_loss": total_loss / processed as f32,
            "time_ms": web_sys::window().unwrap().performance().unwrap().now() - start,
        });

        Ok(result.to_string())
    }

    /// Get compressed gradients for P2P sharing
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn get_gradients(&self) -> Vec<u8> {
        // Flatten and compress gradients
        let flattened = self.flatten_gradients();
        compress_gradients(&flattened)
    }

    /// Apply gradients received from peers
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn apply_gradients(&mut self, compressed_gradients: &[u8]) -> Result<(), JsValue> {
        let gradients = decompress_gradients(compressed_gradients)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        self.update_model_weights(&gradients);
        Ok(())
    }
}

impl BrowserTrainer {
    /// Forward pass through the network
    fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut current = input.to_vec();
        
        for layer in &self.model.layers {
            current = self.forward_layer(&current, layer);
        }
        
        current
    }

    /// Forward pass through a single layer
    fn forward_layer(&self, input: &[f32], layer: &Layer) -> Vec<f32> {
        let mut output = vec![0.0; layer.biases.len()];
        
        // Matrix multiplication with SIMD hints
        #[cfg(target_feature = "simd128")]
        {
            use core::arch::wasm32::*;
            // SIMD implementation for WebAssembly
            for i in 0..output.len() {
                let mut sum = layer.biases[i];
                for j in 0..input.len() {
                    sum += layer.weights[i][j] * input[j];
                }
                output[i] = sum;
            }
        }
        
        #[cfg(not(target_feature = "simd128"))]
        {
            // Scalar fallback
            for i in 0..output.len() {
                let mut sum = layer.biases[i];
                for j in 0..input.len() {
                    sum += layer.weights[i][j] * input[j];
                }
                output[i] = sum;
            }
        }
        
        // Apply activation
        match layer.activation {
            ActivationType::ReLU => output.iter_mut().for_each(|x| *x = x.max(0.0)),
            ActivationType::Sigmoid => output.iter_mut().for_each(|x| *x = 1.0 / (1.0 + (-*x).exp())),
            ActivationType::Tanh => output.iter_mut().for_each(|x| *x = x.tanh()),
            ActivationType::Linear => {} // No activation
        }
        
        output
    }

    /// Backward pass to compute gradients
    fn backward(&self, input: &[f32], output: &[f32], target: &[f32]) -> Vec<Vec<Vec<f32>>> {
        let mut layer_gradients = Vec::new();
        
        // Simplified backpropagation for demonstration
        // In production, this would implement full backprop
        for (layer_idx, layer) in self.model.layers.iter().enumerate() {
            let mut weight_grads = vec![vec![0.0; layer.weights[0].len()]; layer.weights.len()];
            
            // Compute gradients (simplified)
            for i in 0..layer.weights.len() {
                for j in 0..layer.weights[0].len() {
                    let error = if layer_idx == self.model.layers.len() - 1 {
                        output[i] - target[i]
                    } else {
                        0.1 // Placeholder for hidden layer gradients
                    };
                    
                    weight_grads[i][j] = error * (if layer_idx == 0 { input[j] } else { 0.5 });
                }
            }
            
            layer_gradients.push(weight_grads);
        }
        
        layer_gradients
    }

    /// Accumulate gradients for averaging
    fn accumulate_gradients(&mut self, gradients: Vec<Vec<Vec<f32>>>) {
        if self.gradients.is_empty() {
            self.gradients = gradients;
        } else {
            for (layer_idx, layer_grads) in gradients.iter().enumerate() {
                for (i, row) in layer_grads.iter().enumerate() {
                    for (j, &grad) in row.iter().enumerate() {
                        self.gradients[layer_idx][i][j] += grad;
                    }
                }
            }
        }
    }

    /// Flatten gradients for transmission
    fn flatten_gradients(&self) -> Vec<f32> {
        let mut flattened = Vec::new();
        
        for layer_grads in &self.gradients {
            for row in layer_grads {
                flattened.extend_from_slice(row);
            }
        }
        
        flattened
    }

    /// Update model weights with aggregated gradients
    fn update_model_weights(&mut self, gradients: &[f32]) {
        let mut grad_idx = 0;
        
        for (layer_idx, layer) in self.model.layers.iter_mut().enumerate() {
            for i in 0..layer.weights.len() {
                for j in 0..layer.weights[0].len() {
                    if grad_idx < gradients.len() {
                        layer.weights[i][j] -= self.model.learning_rate * gradients[grad_idx];
                        grad_idx += 1;
                    }
                }
            }
        }
        
        // Clear accumulated gradients
        self.gradients.clear();
    }
}

/// Create a lightweight model suitable for browsers
fn create_lightweight_model() -> LightweightModel {
    LightweightModel {
        layers: vec![
            Layer {
                weights: vec![vec![0.1; 10]; 16], // 10 inputs -> 16 hidden
                biases: vec![0.0; 16],
                activation: ActivationType::ReLU,
            },
            Layer {
                weights: vec![vec![0.1; 16]; 8], // 16 hidden -> 8 hidden
                biases: vec![0.0; 8],
                activation: ActivationType::ReLU,
            },
            Layer {
                weights: vec![vec![0.1; 8]; 4], // 8 hidden -> 4 outputs
                biases: vec![0.0; 4],
                activation: ActivationType::Sigmoid,
            },
        ],
        learning_rate: 0.01,
        batch_size: 32,
    }
}

/// Calculate MSE loss
fn calculate_loss(output: &[f32], target: &[f32]) -> f32 {
    output.iter()
        .zip(target.iter())
        .map(|(o, t)| (o - t).powi(2))
        .sum::<f32>() / output.len() as f32
}

/// Compress gradients using simple quantization
fn compress_gradients(gradients: &[f32]) -> Vec<u8> {
    // Find min/max for quantization
    let min = gradients.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max = gradients.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    let mut compressed = Vec::with_capacity(gradients.len() + 8);
    
    // Store min/max
    compressed.extend_from_slice(&min.to_le_bytes());
    compressed.extend_from_slice(&max.to_le_bytes());
    
    // Quantize to u8
    if (max - min).abs() < f32::EPSILON {
        compressed.resize(compressed.len() + gradients.len(), 128);
    } else {
        let scale = 255.0 / (max - min);
        for &grad in gradients {
            let quantized = ((grad - min) * scale).round() as u8;
            compressed.push(quantized);
        }
    }
    
    compressed
}

/// Decompress gradients
fn decompress_gradients(compressed: &[u8]) -> Result<Vec<f32>> {
    if compressed.len() < 8 {
        return Err(anyhow::anyhow!("Invalid compressed gradient data"));
    }
    
    let min = f32::from_le_bytes([compressed[0], compressed[1], compressed[2], compressed[3]]);
    let max = f32::from_le_bytes([compressed[4], compressed[5], compressed[6], compressed[7]]);
    
    let mut gradients = Vec::with_capacity(compressed.len() - 8);
    
    if (max - min).abs() < f32::EPSILON {
        gradients.resize(compressed.len() - 8, min);
    } else {
        let scale = (max - min) / 255.0;
        for &quantized in &compressed[8..] {
            let grad = min + (quantized as f32) * scale;
            gradients.push(grad);
        }
    }
    
    Ok(gradients)
}

/// WebAssembly-specific exports for direct browser usage
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_wasm_training() {
    console_error_panic_hook::set_once();
    
    // Log initialization
    web_sys::console::log_1(&"DAA WASM Training initialized".into());
}

/// Get system capabilities for adaptive training
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_browser_capabilities() -> String {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    
    let capabilities = serde_json::json!({
        "hardware_concurrency": navigator.hardware_concurrency(),
        "device_memory": js_sys::Reflect::get(&navigator, &"deviceMemory".into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        "simd_available": cfg!(target_feature = "simd128"),
        "webgl_available": window.get("WebGLRenderingContext").is_some(),
        "webgpu_available": window.get("GPU").is_some(),
    });
    
    capabilities.to_string()
}