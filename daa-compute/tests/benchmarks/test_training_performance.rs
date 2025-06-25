//! Performance benchmarks for distributed training components

use daa_compute::{P2PNetwork, SwarmConfig};
use daa_compute::protocols::aggregation::{GradientAggregator, AggregationStrategy};
use daa_compute::training::{Gradient, ModelParameters, TrainingMetrics};
use daa_compute::p2p::compression::{CompressionMethod, GradientCompressor};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio;
use anyhow::Result;

#[cfg(test)]
mod benchmarks {
    use super::*;

    fn create_test_gradient(node_id: &str, size: usize, round: u64) -> Gradient {
        let values: Vec<f32> = (0..size).map(|i| (i as f32).sin()).collect();
        Gradient {
            values,
            node_id: node_id.to_string(),
            round,
            compressed: false,
        }
    }

    fn create_large_gradients(num_gradients: usize, gradient_size: usize) -> Vec<Gradient> {
        (0..num_gradients)
            .map(|i| create_test_gradient(&format!("node_{}", i), gradient_size, 1))
            .collect()
    }

    #[tokio::test]
    async fn benchmark_gradient_aggregation_small() -> Result<()> {
        let aggregator = GradientAggregator::new(0).await?;
        let gradients = create_large_gradients(10, 1000); // 10 nodes, 1K parameters each
        
        let start = Instant::now();
        let (result, _) = aggregator.aggregate(gradients, 1).await?;
        let duration = start.elapsed();
        
        println!("Small gradient aggregation (10 nodes, 1K params): {:?}", duration);
        
        // Should complete in reasonable time
        assert!(duration < Duration::from_millis(100));
        assert_eq!(result.values.len(), 1000);
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_gradient_aggregation_medium() -> Result<()> {
        let aggregator = GradientAggregator::new(0).await?;
        let gradients = create_large_gradients(50, 10000); // 50 nodes, 10K parameters each
        
        let start = Instant::now();
        let (result, _) = aggregator.aggregate(gradients, 1).await?;
        let duration = start.elapsed();
        
        println!("Medium gradient aggregation (50 nodes, 10K params): {:?}", duration);
        
        // Should complete in reasonable time
        assert!(duration < Duration::from_secs(1));
        assert_eq!(result.values.len(), 10000);
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_gradient_aggregation_large() -> Result<()> {
        let aggregator = GradientAggregator::new(0).await?;
        let gradients = create_large_gradients(100, 100000); // 100 nodes, 100K parameters each
        
        let start = Instant::now();
        let (result, _) = aggregator.aggregate(gradients, 1).await?;
        let duration = start.elapsed();
        
        println!("Large gradient aggregation (100 nodes, 100K params): {:?}", duration);
        
        // Should complete in reasonable time even for large scale
        assert!(duration < Duration::from_secs(10));
        assert_eq!(result.values.len(), 100000);
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_aggregation_strategies() -> Result<()> {
        let strategies = vec![
            ("Average", AggregationStrategy::Average),
            ("TrimmedMean", AggregationStrategy::TrimmedMean(0.1)),
            ("Median", AggregationStrategy::Median),
            ("Krum", AggregationStrategy::Krum(5)),
        ];
        
        let gradients = create_large_gradients(20, 5000);
        
        for (name, strategy) in strategies {
            let mut aggregator = GradientAggregator::new(0).await?;
            aggregator.set_strategy(strategy);
            
            let start = Instant::now();
            let (result, _) = aggregator.aggregate(gradients.clone(), 1).await?;
            let duration = start.elapsed();
            
            println!("{} aggregation (20 nodes, 5K params): {:?}", name, duration);
            
            // All strategies should complete reasonably quickly
            assert!(duration < Duration::from_secs(5));
            assert_eq!(result.values.len(), 5000);
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_compression_methods() -> Result<()> {
        let methods = vec![
            ("Zstd-1", CompressionMethod::Zstd { level: 1 }),
            ("Zstd-3", CompressionMethod::Zstd { level: 3 }),
            ("Zstd-9", CompressionMethod::Zstd { level: 9 }),
            ("TopK-100", CompressionMethod::TopK { k: 100 }),
            ("TopK-1000", CompressionMethod::TopK { k: 1000 }),
            ("Quantization-4", CompressionMethod::Quantization { bits: 4 }),
            ("Quantization-8", CompressionMethod::Quantization { bits: 8 }),
        ];
        
        let gradient_data = vec![0.1_f32; 50000]; // 50K parameters
        
        for (name, method) in methods {
            let compressor = GradientCompressor::new(method);
            
            // Benchmark compression
            let start = Instant::now();
            let compressed = compressor.compress_sparse(&gradient_data)?;
            let compress_time = start.elapsed();
            
            // Benchmark decompression
            let start = Instant::now();
            let decompressed = compressor.decompress_sparse(&compressed)?;
            let decompress_time = start.elapsed();
            
            let compression_ratio = compressed.len() as f32 / (gradient_data.len() * 4) as f32;
            
            println!("{}: compress={:?}, decompress={:?}, ratio={:.3}", 
                    name, compress_time, decompress_time, compression_ratio);
            
            // Basic sanity checks
            assert!(compress_time < Duration::from_secs(10));
            assert!(decompress_time < Duration::from_secs(10));
            assert_eq!(decompressed.len(), gradient_data.len());
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_parallel_aggregation() -> Result<()> {
        let aggregator = GradientAggregator::new(0).await?;
        
        // Create large batches for parallel processing
        let batch_size = 25;
        let num_batches = 8;
        let gradient_size = 10000;
        
        let mut batches = Vec::new();
        for batch_idx in 0..num_batches {
            let batch = (0..batch_size)
                .map(|i| create_test_gradient(
                    &format!("batch{}_node{}", batch_idx, i), 
                    gradient_size, 
                    1
                ))
                .collect();
            batches.push(batch);
        }
        
        let start = Instant::now();
        let result = aggregator.parallel_aggregate(batches, 1).await?;
        let duration = start.elapsed();
        
        println!("Parallel aggregation ({} batches, {} nodes each, {}K params): {:?}", 
                num_batches, batch_size, gradient_size / 1000, duration);
        
        // Should be faster than sequential processing
        assert!(duration < Duration::from_secs(5));
        assert_eq!(result.values.len(), gradient_size);
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_network_throughput() -> Result<()> {
        let mut config1 = SwarmConfig::default();
        config1.listen_addresses = vec!["/ip4/127.0.0.1/tcp/6000".parse()?];
        
        let mut config2 = SwarmConfig::default();
        config2.listen_addresses = vec!["/ip4/127.0.0.1/tcp/6001".parse()?];
        
        let network1 = P2PNetwork::new(config1).await?;
        let network2 = P2PNetwork::new(config2).await?;
        
        // Connect networks
        let addr1 = network1.get_listen_addresses().await?;
        let peer_id1 = network1.local_peer_id();
        network2.connect_to_peer(peer_id1, addr1[0].clone()).await?;
        
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Benchmark gradient broadcasting
        let num_gradients = 100;
        let gradient_size = 5000;
        
        let start = Instant::now();
        
        for i in 0..num_gradients {
            let gradient = create_test_gradient(&format!("throughput_{}", i), gradient_size, i);
            network2.broadcast_gradient(gradient).await?;
        }
        
        let send_duration = start.elapsed();
        
        // Wait for all messages to be received
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        let received = network1.get_received_gradients().await?;
        
        let throughput = num_gradients as f64 / send_duration.as_secs_f64();
        let data_rate = (num_gradients * gradient_size * 4) as f64 / send_duration.as_secs_f64() / 1024.0 / 1024.0; // MB/s
        
        println!("Network throughput: {:.1} gradients/sec, {:.1} MB/s", throughput, data_rate);
        
        // Should achieve reasonable throughput
        assert!(throughput > 10.0); // At least 10 gradients per second
        assert!(received.len() > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_byzantine_resilience() -> Result<()> {
        let mut aggregator = GradientAggregator::new(0).await?;
        aggregator.set_strategy(AggregationStrategy::Krum(10)); // Tolerate 10 Byzantine nodes
        
        let honest_nodes = 50;
        let byzantine_nodes = 10;
        let gradient_size = 10000;
        
        let mut gradients = Vec::new();
        
        // Add honest gradients
        for i in 0..honest_nodes {
            let gradient = create_test_gradient(&format!("honest_{}", i), gradient_size, 1);
            gradients.push(gradient);
        }
        
        // Add Byzantine gradients
        for i in 0..byzantine_nodes {
            let mut byzantine_values = vec![1e6_f32; gradient_size]; // Extremely large values
            if i % 2 == 0 {
                byzantine_values = vec![f32::NAN; gradient_size]; // NaN values
            }
            
            let byzantine_gradient = Gradient {
                values: byzantine_values,
                node_id: format!("byzantine_{}", i),
                round: 1,
                compressed: false,
            };
            gradients.push(byzantine_gradient);
        }
        
        let start = Instant::now();
        let (result, _) = aggregator.aggregate(gradients, 1).await?;
        let duration = start.elapsed();
        
        println!("Byzantine-resilient aggregation ({} honest, {} Byzantine, {}K params): {:?}", 
                honest_nodes, byzantine_nodes, gradient_size / 1000, duration);
        
        // Should complete and produce reasonable results despite Byzantine nodes
        assert!(duration < Duration::from_secs(10));
        assert_eq!(result.values.len(), gradient_size);
        assert!(result.values.iter().all(|v| v.is_finite()));
        assert!(result.values.iter().all(|v| v.abs() < 1e3)); // Should not be influenced by Byzantine values
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_memory_usage() -> Result<()> {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        // This is a simplified memory tracking - in practice you'd use more sophisticated tools
        
        let large_gradient_size = 1000000; // 1M parameters
        let num_nodes = 100;
        
        let gradients = create_large_gradients(num_nodes, large_gradient_size);
        
        // Estimate memory usage
        let gradient_memory = gradients.len() * large_gradient_size * 4; // f32 = 4 bytes
        println!("Estimated gradient memory: {:.1} MB", gradient_memory as f64 / 1024.0 / 1024.0);
        
        let aggregator = GradientAggregator::new(0).await?;
        
        let start = Instant::now();
        let (result, _) = aggregator.aggregate(gradients, 1).await?;
        let duration = start.elapsed();
        
        println!("Large-scale aggregation memory test ({} nodes, {}M params): {:?}", 
                num_nodes, large_gradient_size / 1000000, duration);
        
        // Should handle large datasets efficiently
        assert!(duration < Duration::from_secs(30));
        assert_eq!(result.values.len(), large_gradient_size);
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_concurrent_aggregation() -> Result<()> {
        let num_concurrent = 10;
        let gradients_per_task = 20;
        let gradient_size = 5000;
        
        let mut handles = Vec::new();
        
        let start = Instant::now();
        
        for task_id in 0..num_concurrent {
            let handle = tokio::spawn(async move {
                let aggregator = GradientAggregator::new(0).await.unwrap();
                let gradients = create_large_gradients(gradients_per_task, gradient_size);
                
                let task_start = Instant::now();
                let (result, _) = aggregator.aggregate(gradients, task_id as u64).await.unwrap();
                let task_duration = task_start.elapsed();
                
                (result.values.len(), task_duration)
            });
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        
        let total_duration = start.elapsed();
        
        println!("Concurrent aggregation ({} tasks, {} nodes each, {}K params): {:?}", 
                num_concurrent, gradients_per_task, gradient_size / 1000, total_duration);
        
        // All tasks should complete successfully
        assert_eq!(results.len(), num_concurrent);
        assert!(results.iter().all(|(size, _)| *size == gradient_size));
        
        // Should benefit from concurrency
        let avg_task_duration: Duration = results.iter()
            .map(|(_, duration)| *duration)
            .sum::<Duration>() / num_concurrent as u32;
        
        println!("Average task duration: {:?}", avg_task_duration);
        
        // Total time should be less than sum of all task times (due to concurrency)
        assert!(total_duration < avg_task_duration * num_concurrent as u32);
        
        Ok(())
    }

    #[tokio::test]
    async fn benchmark_compression_accuracy() -> Result<()> {
        let original_gradient = vec![0.1, 0.2, 0.3, -0.1, -0.2, 0.05, 0.15, -0.05];
        
        let compression_methods = vec![
            CompressionMethod::Zstd { level: 3 },
            CompressionMethod::TopK { k: 6 }, // Keep top 6 out of 8
            CompressionMethod::Quantization { bits: 8 },
        ];
        
        for method in compression_methods {
            let compressor = GradientCompressor::new(method);
            
            let start = Instant::now();
            let compressed = compressor.compress_sparse(&original_gradient)?;
            let decompressed = compressor.decompress_sparse(&compressed)?;
            let duration = start.elapsed();
            
            // Calculate accuracy metrics
            let mse: f32 = original_gradient.iter()
                .zip(decompressed.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>() / original_gradient.len() as f32;
            
            let compression_ratio = compressed.len() as f32 / (original_gradient.len() * 4) as f32;
            
            println!("Compression accuracy - MSE: {:.6}, Ratio: {:.3}, Time: {:?}", 
                    mse, compression_ratio, duration);
            
            // Should maintain reasonable accuracy
            assert!(mse < 1.0); // MSE should be reasonable
            assert_eq!(decompressed.len(), original_gradient.len());
        }
        
        Ok(())
    }
}