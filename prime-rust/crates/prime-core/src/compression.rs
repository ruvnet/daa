//! Gradient compression algorithms for DiLoCo

use crate::error::{Error, Result};
use crate::gradient::{Gradient, CompressedGradient, CompressionAlgorithm};
use byteorder::{ByteOrder, LittleEndian};
use tch::Tensor;

/// Gradient compressor trait
pub trait GradientCompressor {
    /// Compress a gradient
    fn compress(&self, gradient: &Gradient) -> Result<CompressedGradient>;
    
    /// Decompress a gradient
    fn decompress(&self, compressed: &CompressedGradient, device: tch::Device) -> Result<Gradient>;
    
    /// Get compression algorithm
    fn algorithm(&self) -> CompressionAlgorithm;
}

/// Int8 quantization compressor
pub struct Int8Compressor {
    /// Whether to use symmetric quantization
    pub symmetric: bool,
}

impl Default for Int8Compressor {
    fn default() -> Self {
        Self { symmetric: false }
    }
}

impl GradientCompressor for Int8Compressor {
    fn compress(&self, gradient: &Gradient) -> Result<CompressedGradient> {
        let tensor = &gradient.tensor;
        let flat_tensor = tensor.flatten(0, -1);
        
        // Calculate quantization parameters
        let (scale, zero_point) = if self.symmetric {
            calculate_symmetric_quantization_params(&flat_tensor)?
        } else {
            calculate_asymmetric_quantization_params(&flat_tensor)?
        };
        
        // Quantize
        let quantized = quantize_tensor(&flat_tensor, scale, zero_point)?;
        
        // Convert to bytes
        let quantized_data = tensor_to_int8_bytes(&quantized)?;
        
        // Calculate sizes
        let original_size = gradient.numel() * 4; // Assuming float32
        let compressed_size = quantized_data.len() as i64;
        
        Ok(CompressedGradient {
            layer_id: gradient.layer_id.clone(),
            shape: gradient.original_shape.clone(),
            quantized_data,
            scale,
            zero_point,
            algorithm: CompressionAlgorithm::Int8Quantization,
            original_size,
            compressed_size,
        })
    }
    
    fn decompress(&self, compressed: &CompressedGradient, device: tch::Device) -> Result<Gradient> {
        // Convert bytes back to tensor
        let quantized = int8_bytes_to_tensor(&compressed.quantized_data, &compressed.shape)?;
        
        // Dequantize
        let tensor = dequantize_tensor(&quantized, compressed.scale, compressed.zero_point)?;
        let tensor = tensor.reshape(&compressed.shape).to_device(device);
        
        Ok(Gradient::new(compressed.layer_id.clone(), tensor))
    }
    
    fn algorithm(&self) -> CompressionAlgorithm {
        CompressionAlgorithm::Int8Quantization
    }
}

/// Calculate symmetric quantization parameters
fn calculate_symmetric_quantization_params(tensor: &Tensor) -> Result<(f32, i32)> {
    let abs_max = tensor.abs().max().double_value(&[]) as f32;
    let scale = abs_max / 127.0;
    let zero_point = 0;
    
    Ok((scale, zero_point))
}

/// Calculate asymmetric quantization parameters
fn calculate_asymmetric_quantization_params(tensor: &Tensor) -> Result<(f32, i32)> {
    let min_val = tensor.min().double_value(&[]) as f32;
    let max_val = tensor.max().double_value(&[]) as f32;
    
    let scale = (max_val - min_val) / 255.0;
    let zero_point = (-min_val / scale).round() as i32;
    
    Ok((scale, zero_point))
}

/// Quantize tensor to int8
fn quantize_tensor(tensor: &Tensor, scale: f32, zero_point: i32) -> Result<Tensor> {
    let quantized = (tensor / scale + zero_point as f64)
        .clamp(-128.0, 127.0)
        .round()
        .to_kind(tch::Kind::Int8);
    
    Ok(quantized)
}

/// Dequantize tensor from int8
fn dequantize_tensor(quantized: &Tensor, scale: f32, zero_point: i32) -> Result<Tensor> {
    let tensor = (quantized.to_kind(tch::Kind::Float) - zero_point as f64) * scale as f64;
    Ok(tensor)
}

/// Convert tensor to int8 bytes
fn tensor_to_int8_bytes(tensor: &Tensor) -> Result<Vec<u8>> {
    let tensor = tensor.contiguous();
    let numel = tensor.numel() as usize;
    
    // For int8, each element is 1 byte
    let mut bytes = vec![0u8; numel];
    
    unsafe {
        let src = tensor.data_ptr() as *const i8;
        std::ptr::copy_nonoverlapping(src, bytes.as_mut_ptr() as *mut i8, numel);
    }
    
    Ok(bytes)
}

/// Convert int8 bytes to tensor
fn int8_bytes_to_tensor(bytes: &[u8], shape: &[i64]) -> Result<Tensor> {
    let numel: i64 = shape.iter().product();
    
    if bytes.len() != numel as usize {
        return Err(Error::Compression(format!(
            "Size mismatch: expected {} bytes, got {}",
            numel, bytes.len()
        )));
    }
    
    // Create tensor from bytes
    let tensor = unsafe {
        Tensor::from_blob(
            bytes.as_ptr() as *const std::ffi::c_void,
            shape,
            &[1; 7], // Dummy strides
            tch::Kind::Int8,
            tch::Device::Cpu,
        )
    };
    
    Ok(tensor.contiguous())
}

/// Adaptive compression based on gradient properties
pub struct AdaptiveCompressor {
    /// Threshold for using compression
    pub compression_threshold: f32,
    
    /// Int8 compressor
    int8_compressor: Int8Compressor,
}

impl Default for AdaptiveCompressor {
    fn default() -> Self {
        Self {
            compression_threshold: 0.01, // Compress if gradient norm > 0.01
            int8_compressor: Int8Compressor::default(),
        }
    }
}

impl GradientCompressor for AdaptiveCompressor {
    fn compress(&self, gradient: &Gradient) -> Result<CompressedGradient> {
        let norm = gradient.norm()?;
        
        if norm > self.compression_threshold {
            // Use int8 compression for large gradients
            self.int8_compressor.compress(gradient)
        } else {
            // Skip compression for small gradients
            gradient.compress(CompressionAlgorithm::None)
        }
    }
    
    fn decompress(&self, compressed: &CompressedGradient, device: tch::Device) -> Result<Gradient> {
        compressed.decompress(device)
    }
    
    fn algorithm(&self) -> CompressionAlgorithm {
        CompressionAlgorithm::Int8Quantization
    }
}

/// Batch compression for multiple gradients
pub struct BatchCompressor<C: GradientCompressor> {
    /// Underlying compressor
    compressor: C,
    
    /// Whether to compress in parallel
    parallel: bool,
}

impl<C: GradientCompressor> BatchCompressor<C> {
    /// Create a new batch compressor
    pub fn new(compressor: C, parallel: bool) -> Self {
        Self { compressor, parallel }
    }
    
    /// Compress multiple gradients
    pub fn compress_batch(&self, gradients: &[Gradient]) -> Result<Vec<CompressedGradient>> {
        if self.parallel {
            // TODO: Implement parallel compression
            gradients.iter()
                .map(|g| self.compressor.compress(g))
                .collect()
        } else {
            gradients.iter()
                .map(|g| self.compressor.compress(g))
                .collect()
        }
    }
    
    /// Decompress multiple gradients
    pub fn decompress_batch(
        &self,
        compressed: &[CompressedGradient],
        device: tch::Device,
    ) -> Result<Vec<Gradient>> {
        if self.parallel {
            // TODO: Implement parallel decompression
            compressed.iter()
                .map(|c| self.compressor.decompress(c, device))
                .collect()
        } else {
            compressed.iter()
                .map(|c| self.compressor.decompress(c, device))
                .collect()
        }
    }
}

/// Compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Total original size
    pub total_original_size: i64,
    
    /// Total compressed size
    pub total_compressed_size: i64,
    
    /// Number of gradients compressed
    pub num_gradients: usize,
    
    /// Average compression ratio
    pub avg_compression_ratio: f32,
    
    /// Compression time in milliseconds
    pub compression_time_ms: f32,
}

impl CompressionStats {
    /// Update statistics with a new compressed gradient
    pub fn update(&mut self, compressed: &CompressedGradient, time_ms: f32) {
        self.total_original_size += compressed.original_size;
        self.total_compressed_size += compressed.compressed_size;
        self.num_gradients += 1;
        self.compression_time_ms += time_ms;
        
        self.avg_compression_ratio = 
            self.total_original_size as f32 / self.total_compressed_size as f32;
    }
    
    /// Merge with another statistics object
    pub fn merge(&mut self, other: &CompressionStats) {
        self.total_original_size += other.total_original_size;
        self.total_compressed_size += other.total_compressed_size;
        self.num_gradients += other.num_gradients;
        self.compression_time_ms += other.compression_time_ms;
        
        if self.total_compressed_size > 0 {
            self.avg_compression_ratio = 
                self.total_original_size as f32 / self.total_compressed_size as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tch::Device;
    
    #[test]
    fn test_int8_compression() {
        let compressor = Int8Compressor::default();
        
        // Create test gradient
        let tensor = Tensor::randn(&[10, 20], (tch::Kind::Float, Device::Cpu));
        let gradient = Gradient::new("test_layer".to_string(), tensor);
        
        // Compress
        let compressed = compressor.compress(&gradient).unwrap();
        assert_eq!(compressed.algorithm, CompressionAlgorithm::Int8Quantization);
        assert!(compressed.compression_ratio() > 3.0); // Should achieve ~4x compression
        
        // Decompress
        let decompressed = compressor.decompress(&compressed, Device::Cpu).unwrap();
        assert_eq!(decompressed.layer_id, "test_layer");
        assert_eq!(decompressed.original_shape, vec![10, 20]);
    }
    
    #[test]
    fn test_quantization_accuracy() {
        let compressor = Int8Compressor::default();
        
        // Create gradient with known values
        let values = vec![0.0, 0.5, 1.0, -0.5, -1.0];
        let tensor = Tensor::of_slice(&values);
        let gradient = Gradient::new("test".to_string(), tensor.clone());
        
        // Compress and decompress
        let compressed = compressor.compress(&gradient).unwrap();
        let decompressed = compressor.decompress(&compressed, Device::Cpu).unwrap();
        
        // Check that values are approximately preserved
        let original = tensor.to_kind(tch::Kind::Float);
        let recovered = decompressed.tensor;
        let diff = (original - recovered).abs().max().double_value(&[]) as f32;
        
        // Quantization error should be small
        assert!(diff < 0.01);
    }
    
    #[test]
    fn test_adaptive_compression() {
        let compressor = AdaptiveCompressor::default();
        
        // Large gradient should be compressed
        let large_tensor = Tensor::randn(&[100, 100], (tch::Kind::Float, Device::Cpu)) * 10.0;
        let large_gradient = Gradient::new("large".to_string(), large_tensor);
        
        let compressed = compressor.compress(&large_gradient).unwrap();
        assert!(compressed.compression_ratio() > 1.0);
        
        // Small gradient might not be compressed
        let small_tensor = Tensor::randn(&[10, 10], (tch::Kind::Float, Device::Cpu)) * 0.001;
        let small_gradient = Gradient::new("small".to_string(), small_tensor);
        
        let compressed = compressor.compress(&small_gradient).unwrap();
        // Compression decision depends on gradient norm
    }
}