//! Gradient structures and operations for DiLoCo algorithm

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tch::{Device, Tensor};

/// Raw gradient tensor with metadata
#[derive(Debug, Clone)]
pub struct Gradient {
    /// Layer identifier
    pub layer_id: String,
    
    /// Gradient tensor
    pub tensor: Tensor,
    
    /// Original shape before any transformations
    pub original_shape: Vec<i64>,
    
    /// Device where gradient is stored
    pub device: Device,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Gradient {
    /// Create a new gradient
    pub fn new(layer_id: String, tensor: Tensor) -> Self {
        let original_shape = tensor.size();
        let device = tensor.device();
        
        Self {
            layer_id,
            tensor,
            original_shape,
            device,
            metadata: HashMap::new(),
        }
    }
    
    /// Get the number of elements in the gradient
    pub fn numel(&self) -> i64 {
        self.tensor.numel()
    }
    
    /// Get the gradient norm
    pub fn norm(&self) -> Result<f32> {
        Ok(self.tensor.norm().double_value(&[]) as f32)
    }
    
    /// Apply gradient clipping
    pub fn clip_norm(&mut self, max_norm: f32) -> Result<()> {
        let norm = self.norm()?;
        if norm > max_norm {
            self.tensor = &self.tensor * (max_norm / norm);
        }
        Ok(())
    }
    
    /// Convert to compressed representation
    pub fn compress(&self, algorithm: CompressionAlgorithm) -> Result<CompressedGradient> {
        match algorithm {
            CompressionAlgorithm::Int8Quantization => self.quantize_int8(),
            CompressionAlgorithm::None => self.to_uncompressed(),
        }
    }
    
    /// Quantize gradient to int8
    fn quantize_int8(&self) -> Result<CompressedGradient> {
        // Flatten tensor for quantization
        let flat_tensor = self.tensor.flatten(0, -1);
        
        // Calculate scale and zero point
        let min_val = flat_tensor.min().double_value(&[]) as f32;
        let max_val = flat_tensor.max().double_value(&[]) as f32;
        
        let scale = (max_val - min_val) / 255.0;
        let zero_point = (-min_val / scale).round() as i32;
        
        // Quantize
        let quantized = ((flat_tensor - min_val) / scale).round().to_kind(tch::Kind::Int8);
        
        // Convert to bytes
        let quantized_data = tensor_to_bytes(&quantized)?;
        
        Ok(CompressedGradient {
            layer_id: self.layer_id.clone(),
            shape: self.original_shape.clone(),
            quantized_data,
            scale,
            zero_point,
            algorithm: CompressionAlgorithm::Int8Quantization,
            original_size: self.tensor.numel() * 4, // Assuming float32
            compressed_size: quantized_data.len() as i64,
        })
    }
    
    /// Create uncompressed representation
    fn to_uncompressed(&self) -> Result<CompressedGradient> {
        let data = tensor_to_bytes(&self.tensor)?;
        let size = data.len() as i64;
        
        Ok(CompressedGradient {
            layer_id: self.layer_id.clone(),
            shape: self.original_shape.clone(),
            quantized_data: data,
            scale: 1.0,
            zero_point: 0,
            algorithm: CompressionAlgorithm::None,
            original_size: size,
            compressed_size: size,
        })
    }
}

/// Compressed gradient representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedGradient {
    /// Layer identifier
    pub layer_id: String,
    
    /// Original tensor shape
    pub shape: Vec<i64>,
    
    /// Compressed/quantized data
    pub quantized_data: Vec<u8>,
    
    /// Quantization scale
    pub scale: f32,
    
    /// Quantization zero point
    pub zero_point: i32,
    
    /// Compression algorithm used
    pub algorithm: CompressionAlgorithm,
    
    /// Original size in bytes
    pub original_size: i64,
    
    /// Compressed size in bytes
    pub compressed_size: i64,
}

impl CompressedGradient {
    /// Decompress to gradient tensor
    pub fn decompress(&self, device: Device) -> Result<Gradient> {
        match self.algorithm {
            CompressionAlgorithm::Int8Quantization => self.dequantize_int8(device),
            CompressionAlgorithm::None => self.from_uncompressed(device),
        }
    }
    
    /// Dequantize from int8
    fn dequantize_int8(&self, device: Device) -> Result<Gradient> {
        // Convert bytes back to tensor
        let quantized = bytes_to_tensor(&self.quantized_data, tch::Kind::Int8, &self.shape)?;
        
        // Dequantize
        let tensor = quantized.to_kind(tch::Kind::Float) * self.scale + (self.zero_point as f64 * self.scale as f64);
        let tensor = tensor.reshape(&self.shape).to_device(device);
        
        Ok(Gradient::new(self.layer_id.clone(), tensor))
    }
    
    /// Convert from uncompressed
    fn from_uncompressed(&self, device: Device) -> Result<Gradient> {
        let tensor = bytes_to_tensor(&self.quantized_data, tch::Kind::Float, &self.shape)?;
        let tensor = tensor.to_device(device);
        
        Ok(Gradient::new(self.layer_id.clone(), tensor))
    }
    
    /// Get compression ratio
    pub fn compression_ratio(&self) -> f32 {
        self.original_size as f32 / self.compressed_size as f32
    }
}

/// Compression algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Int8 quantization
    Int8Quantization,
}

/// Batch of gradients for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientBatch {
    /// Unique batch identifier
    pub batch_id: String,
    
    /// Global training step
    pub global_step: u64,
    
    /// Worker identifier
    pub worker_id: String,
    
    /// Compressed gradients
    pub gradients: Vec<CompressedGradient>,
    
    /// Timestamp
    pub timestamp: u64,
}

impl GradientBatch {
    /// Create a new gradient batch
    pub fn new(batch_id: String, global_step: u64, worker_id: String) -> Self {
        Self {
            batch_id,
            global_step,
            worker_id,
            gradients: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Add a gradient to the batch
    pub fn add_gradient(&mut self, gradient: CompressedGradient) {
        self.gradients.push(gradient);
    }
    
    /// Get total compressed size
    pub fn total_compressed_size(&self) -> i64 {
        self.gradients.iter().map(|g| g.compressed_size).sum()
    }
    
    /// Get average compression ratio
    pub fn average_compression_ratio(&self) -> f32 {
        if self.gradients.is_empty() {
            return 1.0;
        }
        
        let total_original: i64 = self.gradients.iter().map(|g| g.original_size).sum();
        let total_compressed: i64 = self.gradients.iter().map(|g| g.compressed_size).sum();
        
        total_original as f32 / total_compressed as f32
    }
}

/// Helper function to convert tensor to bytes
fn tensor_to_bytes(tensor: &Tensor) -> Result<Vec<u8>> {
    // Get tensor as contiguous array
    let tensor = tensor.contiguous();
    
    // Get raw data pointer and size
    let numel = tensor.numel() as usize;
    let element_size = tensor.element_size();
    let total_size = numel * element_size;
    
    // Create byte vector
    let mut bytes = vec![0u8; total_size];
    
    // Copy data
    unsafe {
        let src = tensor.data_ptr() as *const u8;
        std::ptr::copy_nonoverlapping(src, bytes.as_mut_ptr(), total_size);
    }
    
    Ok(bytes)
}

/// Helper function to convert bytes to tensor
fn bytes_to_tensor(bytes: &[u8], kind: tch::Kind, shape: &[i64]) -> Result<Tensor> {
    // Calculate expected size
    let numel: i64 = shape.iter().product();
    let element_size = kind.element_size();
    let expected_size = (numel as usize) * element_size;
    
    if bytes.len() != expected_size {
        return Err(Error::Gradient(format!(
            "Size mismatch: expected {} bytes, got {}",
            expected_size,
            bytes.len()
        )));
    }
    
    // Create tensor from bytes
    let tensor = unsafe {
        Tensor::from_blob(
            bytes.as_ptr() as *const std::ffi::c_void,
            shape,
            &[1; 7], // Dummy strides, will be recalculated
            kind,
            Device::Cpu,
        )
    };
    
    Ok(tensor.contiguous())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gradient_compression() {
        // Create a test tensor
        let tensor = Tensor::randn(&[10, 20], (tch::Kind::Float, Device::Cpu));
        let gradient = Gradient::new("test_layer".to_string(), tensor);
        
        // Test int8 quantization
        let compressed = gradient.compress(CompressionAlgorithm::Int8Quantization).unwrap();
        assert!(compressed.compression_ratio() > 1.0);
        
        // Test decompression
        let decompressed = compressed.decompress(Device::Cpu).unwrap();
        assert_eq!(decompressed.layer_id, "test_layer");
        assert_eq!(decompressed.original_shape, vec![10, 20]);
    }
    
    #[test]
    fn test_gradient_batch() {
        let mut batch = GradientBatch::new(
            "batch_1".to_string(),
            100,
            "worker_1".to_string(),
        );
        
        // Add some gradients
        for i in 0..3 {
            let tensor = Tensor::randn(&[5, 5], (tch::Kind::Float, Device::Cpu));
            let gradient = Gradient::new(format!("layer_{}", i), tensor);
            let compressed = gradient.compress(CompressionAlgorithm::Int8Quantization).unwrap();
            batch.add_gradient(compressed);
        }
        
        assert_eq!(batch.gradients.len(), 3);
        assert!(batch.average_compression_ratio() > 1.0);
    }
}