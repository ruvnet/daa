//! Compression utilities for bandwidth optimization
//!
//! Provides multiple compression algorithms optimized for gradient data.

use std::io::{Read, Write};
use anyhow::{Result, anyhow};

/// Compression method for gradient data
#[derive(Debug, Clone, Copy)]
pub enum CompressionMethod {
    /// No compression
    None,
    /// Zstandard compression (best ratio)
    Zstd { level: i32 },
    /// LZ4 compression (fastest)
    Lz4 { level: u32 },
    /// Snappy compression (balanced)
    Snappy,
}

impl CompressionMethod {
    /// Compress data using the selected method
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self {
            CompressionMethod::None => Ok(data.to_vec()),
            
            CompressionMethod::Zstd { level } => {
                let mut encoder = zstd::Encoder::new(Vec::new(), *level)?;
                encoder.write_all(data)?;
                Ok(encoder.finish()?)
            }
            
            CompressionMethod::Lz4 { level } => {
                let mut compressed = Vec::new();
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(*level)
                    .build(&mut compressed)?;
                encoder.write_all(data)?;
                let (_output, result) = encoder.finish();
                result?;
                Ok(compressed)
            }
            
            CompressionMethod::Snappy => {
                let mut encoder = snap::write::FrameEncoder::new(Vec::new());
                encoder.write_all(data)?;
                Ok(encoder.into_inner()?)
            }
        }
    }
    
    /// Decompress data using the selected method
    pub fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        match self {
            CompressionMethod::None => Ok(compressed.to_vec()),
            
            CompressionMethod::Zstd { .. } => {
                let mut decoder = zstd::Decoder::new(compressed)?;
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            
            CompressionMethod::Lz4 { .. } => {
                let mut decoder = lz4::Decoder::new(compressed)?;
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            
            CompressionMethod::Snappy => {
                let mut decoder = snap::read::FrameDecoder::new(compressed);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
        }
    }
    
    /// Get compression ratio estimate
    pub fn compression_ratio(&self) -> f32 {
        match self {
            CompressionMethod::None => 1.0,
            CompressionMethod::Zstd { level } => {
                // Zstd typically achieves 3-5x compression on gradient data
                match level {
                    1..=3 => 3.0,
                    4..=9 => 4.0,
                    10..=19 => 4.5,
                    _ => 5.0,
                }
            }
            CompressionMethod::Lz4 { .. } => 2.5, // LZ4 is faster but lower ratio
            CompressionMethod::Snappy => 2.0, // Snappy is balanced
        }
    }
}

/// Gradient-specific compression optimizations
pub struct GradientCompressor {
    method: CompressionMethod,
    sparsity_threshold: f32,
}

impl GradientCompressor {
    pub fn new(method: CompressionMethod) -> Self {
        Self {
            method,
            sparsity_threshold: 0.01, // Values below this are considered zero
        }
    }
    
    /// Compress gradient with sparsity optimization
    pub fn compress_sparse(&self, gradient: &[f32]) -> Result<Vec<u8>> {
        // Count non-zero elements
        let non_zero_count = gradient.iter()
            .filter(|&&v| v.abs() > self.sparsity_threshold)
            .count();
        
        let sparsity = 1.0 - (non_zero_count as f32 / gradient.len() as f32);
        
        if sparsity > 0.5 {
            // Use sparse representation
            self.compress_sparse_format(gradient)
        } else {
            // Use dense representation with quantization
            let quantized = super::gradient::quantize_gradient(gradient)?;
            self.method.compress(&quantized)
        }
    }
    
    /// Compress using sparse format (index, value) pairs
    fn compress_sparse_format(&self, gradient: &[f32]) -> Result<Vec<u8>> {
        let mut sparse_data = Vec::new();
        
        // Write gradient length
        sparse_data.extend_from_slice(&(gradient.len() as u32).to_le_bytes());
        
        // Write non-zero count
        let non_zero_indices: Vec<(u32, f32)> = gradient.iter()
            .enumerate()
            .filter(|(_, &v)| v.abs() > self.sparsity_threshold)
            .map(|(i, &v)| (i as u32, v))
            .collect();
        
        sparse_data.extend_from_slice(&(non_zero_indices.len() as u32).to_le_bytes());
        
        // Write (index, value) pairs
        for (idx, value) in non_zero_indices {
            sparse_data.extend_from_slice(&idx.to_le_bytes());
            sparse_data.extend_from_slice(&value.to_le_bytes());
        }
        
        // Compress the sparse representation
        self.method.compress(&sparse_data)
    }
    
    /// Decompress sparse format back to dense gradient
    pub fn decompress_sparse(&self, compressed: &[u8]) -> Result<Vec<f32>> {
        let sparse_data = self.method.decompress(compressed)?;
        
        if sparse_data.len() < 8 {
            return Err(anyhow!("Invalid sparse gradient data"));
        }
        
        // Read gradient length
        let gradient_len = u32::from_le_bytes([
            sparse_data[0], sparse_data[1], sparse_data[2], sparse_data[3]
        ]) as usize;
        
        // Read non-zero count
        let non_zero_count = u32::from_le_bytes([
            sparse_data[4], sparse_data[5], sparse_data[6], sparse_data[7]
        ]) as usize;
        
        // Initialize gradient with zeros
        let mut gradient = vec![0.0; gradient_len];
        
        // Read (index, value) pairs
        let mut offset = 8;
        for _ in 0..non_zero_count {
            if offset + 8 > sparse_data.len() {
                return Err(anyhow!("Truncated sparse gradient data"));
            }
            
            let idx = u32::from_le_bytes([
                sparse_data[offset], sparse_data[offset + 1],
                sparse_data[offset + 2], sparse_data[offset + 3]
            ]) as usize;
            
            let value = f32::from_le_bytes([
                sparse_data[offset + 4], sparse_data[offset + 5],
                sparse_data[offset + 6], sparse_data[offset + 7]
            ]);
            
            if idx < gradient_len {
                gradient[idx] = value;
            }
            
            offset += 8;
        }
        
        Ok(gradient)
    }
}

/// Delta compression for sequential gradient updates
pub struct DeltaCompressor {
    base_method: CompressionMethod,
    previous_gradient: Option<Vec<f32>>,
}

impl DeltaCompressor {
    pub fn new(method: CompressionMethod) -> Self {
        Self {
            base_method: method,
            previous_gradient: None,
        }
    }
    
    /// Compress gradient as delta from previous
    pub fn compress_delta(&mut self, gradient: &[f32]) -> Result<Vec<u8>> {
        let delta = if let Some(prev) = &self.previous_gradient {
            if prev.len() != gradient.len() {
                return Err(anyhow!("Gradient size mismatch"));
            }
            
            // Compute delta
            gradient.iter()
                .zip(prev.iter())
                .map(|(&curr, &prev)| curr - prev)
                .collect()
        } else {
            // First gradient, no delta
            gradient.to_vec()
        };
        
        // Update previous gradient
        self.previous_gradient = Some(gradient.to_vec());
        
        // Compress delta
        let quantized = super::gradient::quantize_gradient(&delta)?;
        self.base_method.compress(&quantized)
    }
    
    /// Decompress delta and apply to previous gradient
    pub fn decompress_delta(&mut self, compressed: &[u8]) -> Result<Vec<f32>> {
        let quantized = self.base_method.decompress(compressed)?;
        let delta = super::gradient::dequantize_gradient(&quantized)?;
        
        let gradient = if let Some(prev) = &self.previous_gradient {
            if prev.len() != delta.len() {
                return Err(anyhow!("Delta size mismatch"));
            }
            
            // Apply delta
            delta.iter()
                .zip(prev.iter())
                .map(|(&d, &p)| p + d)
                .collect()
        } else {
            // First gradient
            delta
        };
        
        // Update previous gradient
        self.previous_gradient = Some(gradient.clone());
        
        Ok(gradient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression_methods() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let quantized = super::super::gradient::quantize_gradient(&data).unwrap();
        
        for method in &[
            CompressionMethod::None,
            CompressionMethod::Zstd { level: 3 },
            CompressionMethod::Lz4 { level: 4 },
            CompressionMethod::Snappy,
        ] {
            let compressed = method.compress(&quantized).unwrap();
            let decompressed = method.decompress(&compressed).unwrap();
            assert_eq!(quantized, decompressed);
        }
    }
    
    #[test]
    fn test_sparse_compression() {
        let mut gradient = vec![0.0; 1000];
        gradient[10] = 1.0;
        gradient[100] = 2.0;
        gradient[500] = 3.0;
        
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        let decompressed = compressor.decompress_sparse(&compressed).unwrap();
        
        assert_eq!(gradient.len(), decompressed.len());
        assert_eq!(gradient[10], decompressed[10]);
        assert_eq!(gradient[100], decompressed[100]);
        assert_eq!(gradient[500], decompressed[500]);
    }
}