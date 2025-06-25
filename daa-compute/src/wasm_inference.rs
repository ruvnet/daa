//! Browser-compatible inference engine for WASM
//!
//! Provides lightweight model inference capabilities optimized for
//! web browser execution with minimal memory footprint.

use serde::{Serialize, Deserialize};
use anyhow::Result;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Inference engine for browser-based model execution
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct BrowserInference {
    model_cache: ModelCache,
    config: InferenceConfig,
}

/// Configuration for browser inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Maximum batch size for inference
    pub max_batch_size: usize,
    /// Use WebGL acceleration if available
    pub use_webgl: bool,
    /// Use WebGPU acceleration if available
    pub use_webgpu: bool,
    /// Cache model weights in IndexedDB
    pub cache_in_indexeddb: bool,
    /// Maximum inference time per request (ms)
    pub max_inference_time_ms: u32,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 64,
            use_webgl: true,
            use_webgpu: true,
            cache_in_indexeddb: true,
            max_inference_time_ms: 100,
        }
    }
}

/// Model cache for efficient inference
#[derive(Debug, Clone)]
struct ModelCache {
    models: std::collections::HashMap<String, CachedModel>,
    max_cache_size_mb: usize,
    current_size_bytes: usize,
}

/// Cached model data
#[derive(Debug, Clone)]
struct CachedModel {
    id: String,
    weights: Vec<Vec<f32>>,
    metadata: ModelMetadata,
    last_used: std::time::SystemTime,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub quantization: QuantizationType,
}

/// Quantization types supported
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QuantizationType {
    None,
    Int8,
    Int4,
    Dynamic,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl BrowserInference {
    /// Create a new inference engine
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: Option<String>) -> Result<BrowserInference, JsValue> {
        let config = if let Some(json) = config_json {
            serde_json::from_str(&json).map_err(|e| JsValue::from_str(&e.to_string()))?
        } else {
            InferenceConfig::default()
        };

        Ok(Self {
            model_cache: ModelCache {
                models: std::collections::HashMap::new(),
                max_cache_size_mb: 100,
                current_size_bytes: 0,
            },
            config,
        })
    }

    /// Load a model for inference
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub async fn load_model(&mut self, model_data: &[u8], metadata_json: &str) -> Result<String, JsValue> {
        let metadata: ModelMetadata = serde_json::from_str(metadata_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Decompress and deserialize model
        let weights = self.deserialize_model(model_data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let model_id = format!("{}-{}", metadata.name, metadata.version);
        
        // Cache the model
        self.cache_model(model_id.clone(), weights, metadata);

        // Store in IndexedDB if configured
        if self.config.cache_in_indexeddb {
            self.store_in_indexeddb(&model_id, model_data).await?;
        }

        Ok(model_id)
    }

    /// Run inference on input data
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub async fn infer(&mut self, model_id: &str, input_data: &[f32]) -> Result<Vec<f32>, JsValue> {
        let start = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();

        // Get model from cache
        let model = self.model_cache.models.get(model_id)
            .ok_or_else(|| JsValue::from_str("Model not found in cache"))?;

        // Validate input shape
        let expected_size: usize = model.metadata.input_shape.iter().product();
        if input_data.len() != expected_size {
            return Err(JsValue::from_str(&format!(
                "Input size mismatch. Expected {}, got {}",
                expected_size,
                input_data.len()
            )));
        }

        // Run inference based on available acceleration
        let output = if self.config.use_webgpu && self.is_webgpu_available() {
            self.infer_webgpu(model, input_data).await?
        } else if self.config.use_webgl && self.is_webgl_available() {
            self.infer_webgl(model, input_data)?
        } else {
            self.infer_cpu(model, input_data)?
        };

        // Check time constraint
        let elapsed = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now() - start;

        if elapsed > self.config.max_inference_time_ms as f64 {
            web_sys::console::warn_1(&format!("Inference took {}ms, exceeding limit", elapsed).into());
        }

        Ok(output)
    }

    /// Get model information
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn get_model_info(&self, model_id: &str) -> Result<String, JsValue> {
        let model = self.model_cache.models.get(model_id)
            .ok_or_else(|| JsValue::from_str("Model not found"))?;

        let info = serde_json::json!({
            "metadata": model.metadata,
            "cache_size_bytes": self.estimate_model_size(model),
            "last_used": model.last_used.elapsed().unwrap().as_secs(),
        });

        Ok(info.to_string())
    }

    /// Clear model cache
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn clear_cache(&mut self) {
        self.model_cache.models.clear();
        self.model_cache.current_size_bytes = 0;
    }
}

impl BrowserInference {
    /// CPU-based inference
    fn infer_cpu(&self, model: &CachedModel, input: &[f32]) -> Result<Vec<f32>, JsValue> {
        let mut current = input.to_vec();

        // Simple feedforward for demonstration
        for weights in &model.weights {
            current = self.apply_layer(&current, weights);
        }

        // Apply final activation based on model type
        self.apply_final_activation(&mut current);

        Ok(current)
    }

    /// WebGL-accelerated inference
    fn infer_webgl(&self, model: &CachedModel, input: &[f32]) -> Result<Vec<f32>, JsValue> {
        // WebGL implementation would go here
        // For now, fallback to CPU
        web_sys::console::log_1(&"WebGL inference not yet implemented, using CPU".into());
        self.infer_cpu(model, input)
    }

    /// WebGPU-accelerated inference
    async fn infer_webgpu(&self, model: &CachedModel, input: &[f32]) -> Result<Vec<f32>, JsValue> {
        // WebGPU implementation would go here
        // For now, fallback to CPU
        web_sys::console::log_1(&"WebGPU inference not yet implemented, using CPU".into());
        self.infer_cpu(model, input)
    }

    /// Apply a single layer transformation
    fn apply_layer(&self, input: &[f32], weights: &[f32]) -> Vec<f32> {
        // Simplified matrix multiplication
        // In production, this would handle proper shapes
        let output_size = (weights.len() / input.len()).max(1);
        let mut output = vec![0.0; output_size];

        for i in 0..output_size {
            for j in 0..input.len() {
                if i * input.len() + j < weights.len() {
                    output[i] += input[j] * weights[i * input.len() + j];
                }
            }
            // ReLU activation
            output[i] = output[i].max(0.0);
        }

        output
    }

    /// Apply final activation function
    fn apply_final_activation(&self, output: &mut [f32]) {
        // Softmax for classification
        let max = output.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f32 = output.iter().map(|&x| (x - max).exp()).sum();
        
        for x in output.iter_mut() {
            *x = (*x - max).exp() / sum;
        }
    }

    /// Cache a model
    fn cache_model(&mut self, id: String, weights: Vec<Vec<f32>>, metadata: ModelMetadata) {
        let model = CachedModel {
            id: id.clone(),
            weights,
            metadata,
            last_used: std::time::SystemTime::now(),
        };

        let size = self.estimate_model_size(&model);
        
        // Evict old models if needed
        while self.model_cache.current_size_bytes + size > self.model_cache.max_cache_size_mb * 1024 * 1024 {
            self.evict_oldest_model();
        }

        self.model_cache.current_size_bytes += size;
        self.model_cache.models.insert(id, model);
    }

    /// Estimate model size in bytes
    fn estimate_model_size(&self, model: &CachedModel) -> usize {
        model.weights.iter()
            .map(|layer| layer.len() * std::mem::size_of::<f32>())
            .sum()
    }

    /// Evict the oldest model from cache
    fn evict_oldest_model(&mut self) {
        if let Some((oldest_id, _)) = self.model_cache.models.iter()
            .min_by_key(|(_, model)| model.last_used) {
            let oldest_id = oldest_id.clone();
            if let Some(model) = self.model_cache.models.remove(&oldest_id) {
                self.model_cache.current_size_bytes -= self.estimate_model_size(&model);
            }
        }
    }

    /// Deserialize model from bytes
    fn deserialize_model(&self, data: &[u8]) -> Result<Vec<Vec<f32>>> {
        // Simple deserialization - in production would handle various formats
        let decompressed = self.decompress_model_data(data)?;
        Ok(vec![decompressed]) // Simplified - would parse actual layer structure
    }

    /// Decompress model data
    fn decompress_model_data(&self, data: &[u8]) -> Result<Vec<f32>> {
        // Simple decompression - would use proper compression in production
        if data.len() < 8 {
            return Err(anyhow::anyhow!("Invalid model data"));
        }

        let num_weights = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let mut weights = Vec::with_capacity(num_weights);

        for chunk in data[4..].chunks(4) {
            if chunk.len() == 4 {
                let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                weights.push(value);
            }
        }

        Ok(weights)
    }

    /// Store model in IndexedDB
    #[cfg(target_arch = "wasm32")]
    async fn store_in_indexeddb(&self, model_id: &str, data: &[u8]) -> Result<(), JsValue> {
        // IndexedDB implementation would go here
        web_sys::console::log_1(&format!("Would store {} bytes for model {} in IndexedDB", data.len(), model_id).into());
        Ok(())
    }

    /// Check if WebGL is available
    #[cfg(target_arch = "wasm32")]
    fn is_webgl_available(&self) -> bool {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.create_element("canvas").ok())
            .and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .and_then(|canvas| canvas.get_context("webgl").ok())
            .is_some()
    }

    /// Check if WebGPU is available
    #[cfg(target_arch = "wasm32")]
    fn is_webgpu_available(&self) -> bool {
        web_sys::window()
            .and_then(|w| js_sys::Reflect::get(&w, &"GPU".into()).ok())
            .is_some()
    }
}

/// Utility functions for WASM
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_inference_capabilities() -> String {
    let window = web_sys::window().unwrap();
    
    let caps = serde_json::json!({
        "webgl": window.document()
            .and_then(|d| d.create_element("canvas").ok())
            .and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .and_then(|canvas| canvas.get_context("webgl").ok())
            .is_some(),
        "webgl2": window.document()
            .and_then(|d| d.create_element("canvas").ok())
            .and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .and_then(|canvas| canvas.get_context("webgl2").ok())
            .is_some(),
        "webgpu": js_sys::Reflect::get(&window, &"GPU".into()).is_ok(),
        "wasm_simd": cfg!(target_feature = "simd128"),
        "indexeddb": js_sys::Reflect::get(&window, &"indexedDB".into()).is_ok(),
    });

    caps.to_string()
}

/// Benchmark inference performance
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn benchmark_inference(size: usize) -> String {
    let input = vec![0.5; size];
    let weights = vec![vec![0.1; size]; 10];
    
    let start = web_sys::window().unwrap().performance().unwrap().now();
    
    // Run simple inference
    let mut current = input;
    for _ in 0..10 {
        let mut output = vec![0.0; 10];
        for i in 0..10 {
            for j in 0..current.len() {
                output[i] += current[j] * weights[i][j];
            }
        }
        current = output;
    }
    
    let elapsed = web_sys::window().unwrap().performance().unwrap().now() - start;
    
    serde_json::json!({
        "input_size": size,
        "layers": 10,
        "time_ms": elapsed,
        "throughput_ops_per_sec": (size * 10 * 10) as f64 / (elapsed / 1000.0),
    }).to_string()
}