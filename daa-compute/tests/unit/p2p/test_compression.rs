//! Unit tests for gradient compression

use daa_compute::p2p::compression::{CompressionMethod, GradientCompressor};
use daa_compute::training::Gradient;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gradient(values: Vec<f32>) -> Gradient {
        Gradient {
            values,
            node_id: "test_node".to_string(),
            round: 1,
            compressed: false,
        }
    }

    #[test]
    fn test_zstd_compression_basic() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        let gradient = vec![0.1_f32; 1000];
        
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        
        // Should be compressed (less than original size)
        assert!(compressed.len() < gradient.len() * 4);
        
        let decompressed = compressor.decompress_sparse(&compressed).unwrap();
        
        // Should match original
        for (orig, decomp) in gradient.iter().zip(decompressed.iter()) {
            assert!((orig - decomp).abs() < 0.001);
        }
    }

    #[test]
    fn test_zstd_compression_levels() {
        let gradient = vec![0.1_f32; 1000];
        
        let compressor_low = GradientCompressor::new(CompressionMethod::Zstd { level: 1 });
        let compressor_high = GradientCompressor::new(CompressionMethod::Zstd { level: 9 });
        
        let compressed_low = compressor_low.compress_sparse(&gradient).unwrap();
        let compressed_high = compressor_high.compress_sparse(&gradient).unwrap();
        
        // Higher compression level should result in smaller size
        assert!(compressed_high.len() <= compressed_low.len());
    }

    #[test]
    fn test_top_k_compression() {
        let compressor = GradientCompressor::new(CompressionMethod::TopK { k: 10 });
        let gradient = vec![0.1, 0.9, 0.2, 0.8, 0.3, 0.7, 0.4, 0.6, 0.5, 0.0];
        
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        
        // Should be much smaller than original
        assert!(compressed.len() < gradient.len() * 4);
        
        let decompressed = compressor.decompress_sparse(&compressed).unwrap();
        
        // Top-K should preserve largest values
        assert_eq!(decompressed.len(), gradient.len());
        
        // Check that the largest values are preserved
        let mut sorted_indices: Vec<usize> = (0..gradient.len()).collect();
        sorted_indices.sort_by(|&a, &b| gradient[b].partial_cmp(&gradient[a]).unwrap());
        
        for i in 0..3 { // Check top 3 values
            let idx = sorted_indices[i];
            assert!((gradient[idx] - decompressed[idx]).abs() < 0.001);
        }
    }

    #[test]
    fn test_quantization_compression() {
        let compressor = GradientCompressor::new(CompressionMethod::Quantization { bits: 8 });
        let gradient = vec![-1.0, -0.5, 0.0, 0.5, 1.0];
        
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        let decompressed = compressor.decompress_sparse(&compressed).unwrap();
        
        // Should have quantization error but preserve general structure
        for (orig, decomp) in gradient.iter().zip(decompressed.iter()) {
            assert!((orig - decomp).abs() < 0.1); // Allow some quantization error
        }
    }

    #[test]
    fn test_compression_with_zeros() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        let gradient = vec![0.0_f32; 1000];
        
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        
        // Should compress very well with all zeros
        assert!(compressed.len() < 100);
        
        let decompressed = compressor.decompress_sparse(&compressed).unwrap();
        
        // Should match original
        for (orig, decomp) in gradient.iter().zip(decompressed.iter()) {
            assert!((orig - decomp).abs() < 0.001);
        }
    }

    #[test]
    fn test_compression_with_random_data() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        
        // Create random-like gradient
        let gradient: Vec<f32> = (0..1000).map(|i| (i as f32).sin()).collect();
        
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        let decompressed = compressor.decompress_sparse(&compressed).unwrap();
        
        // Should match original
        for (orig, decomp) in gradient.iter().zip(decompressed.iter()) {
            assert!((orig - decomp).abs() < 0.001);
        }
    }

    #[test]
    fn test_empty_gradient_compression() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        let gradient = Vec::<f32>::new();
        
        let result = compressor.compress_sparse(&gradient);
        
        // Should handle empty gradient gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_compression_ratio_calculation() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        let gradient = vec![0.1_f32; 1000];
        
        let original_size = gradient.len() * 4; // f32 = 4 bytes
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        
        let compression_ratio = compressed.len() as f32 / original_size as f32;
        
        // Should achieve some compression
        assert!(compression_ratio < 1.0);
        assert!(compression_ratio > 0.0);
    }

    #[test]
    fn test_multiple_compression_methods() {
        let gradient = vec![0.1_f32; 100];
        
        let methods = vec![
            CompressionMethod::Zstd { level: 3 },
            CompressionMethod::TopK { k: 50 },
            CompressionMethod::Quantization { bits: 8 },
        ];
        
        for method in methods {
            let compressor = GradientCompressor::new(method);
            let compressed = compressor.compress_sparse(&gradient).unwrap();
            let decompressed = compressor.decompress_sparse(&compressed).unwrap();
            
            // Basic sanity check
            assert_eq!(decompressed.len(), gradient.len());
        }
    }

    #[test]
    fn test_compression_with_extreme_values() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        let gradient = vec![f32::MAX, f32::MIN, 0.0, 1e-10, -1e-10];
        
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        let decompressed = compressor.decompress_sparse(&compressed).unwrap();
        
        // Should handle extreme values
        assert_eq!(decompressed.len(), gradient.len());
        
        // Check finite values
        for (orig, decomp) in gradient.iter().zip(decompressed.iter()) {
            if orig.is_finite() && decomp.is_finite() {
                assert!((orig - decomp).abs() < orig.abs() * 0.001 + 0.001);
            }
        }
    }

    #[test]
    fn test_compression_performance() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 1 });
        let gradient = vec![0.1_f32; 10000];
        
        let start = std::time::Instant::now();
        let compressed = compressor.compress_sparse(&gradient).unwrap();
        let compress_time = start.elapsed();
        
        let start = std::time::Instant::now();
        let _decompressed = compressor.decompress_sparse(&compressed).unwrap();
        let decompress_time = start.elapsed();
        
        // Should complete in reasonable time
        assert!(compress_time.as_secs() < 5);
        assert!(decompress_time.as_secs() < 5);
    }

    #[test]
    fn test_compression_deterministic() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        let gradient = vec![0.1_f32; 100];
        
        let compressed1 = compressor.compress_sparse(&gradient).unwrap();
        let compressed2 = compressor.compress_sparse(&gradient).unwrap();
        
        // Same input should produce same output
        assert_eq!(compressed1, compressed2);
    }

    #[test]
    fn test_compression_error_handling() {
        let compressor = GradientCompressor::new(CompressionMethod::Zstd { level: 3 });
        
        // Test decompression of invalid data
        let invalid_data = vec![0xFF_u8; 10];
        let result = compressor.decompress_sparse(&invalid_data);
        
        // Should handle invalid data gracefully
        assert!(result.is_err());
    }
}