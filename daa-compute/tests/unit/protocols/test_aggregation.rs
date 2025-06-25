//! Unit tests for gradient aggregation protocols with Byzantine fault tolerance

use daa_compute::protocols::aggregation::{GradientAggregator, AggregationStrategy};
use daa_compute::training::Gradient;
use std::collections::HashMap;
use tokio;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gradient(node_id: &str, values: Vec<f32>, round: u64) -> Gradient {
        Gradient {
            values,
            node_id: node_id.to_string(),
            round,
            compressed: false,
        }
    }

    #[tokio::test]
    async fn test_aggregator_creation() {
        let aggregator = GradientAggregator::new(3).await.unwrap();
        assert_eq!(aggregator.compression_level, 3);
    }

    #[tokio::test]
    async fn test_average_aggregation() {
        let mut aggregator = GradientAggregator::new(0).await.unwrap();
        aggregator.set_strategy(AggregationStrategy::Average);
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0, 2.0, 3.0], 1),
            create_test_gradient("node2", vec![4.0, 5.0, 6.0], 1),
            create_test_gradient("node3", vec![7.0, 8.0, 9.0], 1),
        ];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Check averaging: (1+4+7)/3=4, (2+5+8)/3=5, (3+6+9)/3=6
        assert_eq!(result.values, vec![4.0, 5.0, 6.0]);
        assert_eq!(result.node_id, "aggregator");
        assert_eq!(result.round, 1);
    }

    #[tokio::test]
    async fn test_weighted_average_aggregation() {
        let mut aggregator = GradientAggregator::new(0).await.unwrap();
        
        let mut weights = HashMap::new();
        weights.insert("node1".to_string(), 0.5);
        weights.insert("node2".to_string(), 0.3);
        weights.insert("node3".to_string(), 0.2);
        
        aggregator.set_strategy(AggregationStrategy::WeightedAverage(weights));
        
        let gradients = vec![
            create_test_gradient("node1", vec![10.0, 20.0], 1),
            create_test_gradient("node2", vec![30.0, 40.0], 1),
            create_test_gradient("node3", vec![50.0, 60.0], 1),
        ];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Weighted average: (10*0.5 + 30*0.3 + 50*0.2) = 24
        assert_eq!(result.values[0], 24.0);
        assert_eq!(result.values[1], 34.0); // (20*0.5 + 40*0.3 + 60*0.2)
    }

    #[tokio::test]
    async fn test_trimmed_mean_aggregation() {
        let mut aggregator = GradientAggregator::new(0).await.unwrap();
        aggregator.set_strategy(AggregationStrategy::TrimmedMean(0.2)); // Trim 20%
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0], 1),
            create_test_gradient("node2", vec![2.0], 1),
            create_test_gradient("node3", vec![3.0], 1),
            create_test_gradient("node4", vec![4.0], 1),
            create_test_gradient("node5", vec![100.0], 1), // Outlier
        ];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Should trim extremes and average the middle values
        // With 20% trimming on 5 values, trim 1 from each end
        // Remaining: [2.0, 3.0, 4.0], average = 3.0
        assert_eq!(result.values[0], 3.0);
    }

    #[tokio::test]
    async fn test_median_aggregation() {
        let mut aggregator = GradientAggregator::new(0).await.unwrap();
        aggregator.set_strategy(AggregationStrategy::Median);
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0, 10.0], 1),
            create_test_gradient("node2", vec![2.0, 20.0], 1),
            create_test_gradient("node3", vec![3.0, 30.0], 1),
            create_test_gradient("node4", vec![4.0, 40.0], 1),
            create_test_gradient("node5", vec![5.0, 50.0], 1),
        ];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Median of [1,2,3,4,5] = 3, median of [10,20,30,40,50] = 30
        assert_eq!(result.values, vec![3.0, 30.0]);
    }

    #[tokio::test]
    async fn test_krum_aggregation_basic() {
        let mut aggregator = GradientAggregator::new(0).await.unwrap();
        aggregator.set_strategy(AggregationStrategy::Krum(1)); // Tolerate 1 Byzantine node
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0, 1.0], 1),
            create_test_gradient("node2", vec![1.1, 1.1], 1),
            create_test_gradient("node3", vec![1.2, 1.2], 1),
            create_test_gradient("node4", vec![100.0, 100.0], 1), // Byzantine
        ];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Should select one of the honest gradients (not the Byzantine one)
        assert!(result.values[0] < 10.0);
        assert!(result.values[1] < 10.0);
    }

    #[tokio::test]
    async fn test_krum_insufficient_nodes() {
        let mut aggregator = GradientAggregator::new(0).await.unwrap();
        aggregator.set_strategy(AggregationStrategy::Krum(2)); // Tolerate 2 Byzantine nodes
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0], 1),
            create_test_gradient("node2", vec![2.0], 1),
            // Need at least 2*f+3 = 7 nodes for f=2
        ];
        
        let result = aggregator.aggregate(gradients, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_byzantine_fault_tolerance() {
        let mut aggregator = GradientAggregator::new(0).await.unwrap();
        aggregator.set_strategy(AggregationStrategy::Krum(2)); // Tolerate 2 Byzantine nodes
        
        let gradients = vec![
            // Honest nodes with similar gradients
            create_test_gradient("honest1", vec![1.0, 1.0], 1),
            create_test_gradient("honest2", vec![1.1, 1.1], 1),
            create_test_gradient("honest3", vec![1.2, 1.2], 1),
            create_test_gradient("honest4", vec![0.9, 0.9], 1),
            create_test_gradient("honest5", vec![1.3, 1.3], 1),
            // Byzantine nodes with malicious gradients
            create_test_gradient("byzantine1", vec![1000.0, 1000.0], 1),
            create_test_gradient("byzantine2", vec![-1000.0, -1000.0], 1),
        ];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Should select an honest gradient
        assert!(result.values[0] < 10.0 && result.values[0] > -10.0);
        assert!(result.values[1] < 10.0 && result.values[1] > -10.0);
    }

    #[tokio::test]
    async fn test_gradient_verification() {
        let aggregator = GradientAggregator::new(0).await.unwrap();
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0, 2.0], 1),
            create_test_gradient("node2", vec![f32::NAN, f32::INFINITY], 1), // Invalid
            create_test_gradient("node3", vec![3.0, 4.0], 1),
            create_test_gradient("node4", vec![1e8_f32; 2], 1), // Excessive norm
        ];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Should filter out invalid gradients and process valid ones
        assert!(result.values.iter().all(|v| v.is_finite()));
    }

    #[tokio::test]
    async fn test_empty_gradients() {
        let aggregator = GradientAggregator::new(0).await.unwrap();
        let gradients = vec![];
        
        let result = aggregator.aggregate(gradients, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_gradient() {
        let aggregator = GradientAggregator::new(0).await.unwrap();
        let gradients = vec![create_test_gradient("node1", vec![1.0, 2.0], 1)];
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        assert_eq!(result.values, vec![1.0, 2.0]);
    }

    #[tokio::test]
    async fn test_parallel_aggregation() {
        let aggregator = GradientAggregator::new(0).await.unwrap();
        
        let batch1 = vec![
            create_test_gradient("node1", vec![1.0, 2.0], 1),
            create_test_gradient("node2", vec![3.0, 4.0], 1),
        ];
        
        let batch2 = vec![
            create_test_gradient("node3", vec![5.0, 6.0], 1),
            create_test_gradient("node4", vec![7.0, 8.0], 1),
        ];
        
        let batches = vec![batch1, batch2];
        let result = aggregator.parallel_aggregate(batches, 1).await.unwrap();
        
        // Should aggregate all gradients across batches
        assert_eq!(result.values, vec![4.0, 5.0]); // (1+3+5+7)/4, (2+4+6+8)/4
    }

    #[tokio::test]
    async fn test_compression_level_effect() {
        let aggregator_no_compression = GradientAggregator::new(0).await.unwrap();
        let aggregator_high_compression = GradientAggregator::new(9).await.unwrap();
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0; 1000], 1),
            create_test_gradient("node2", vec![2.0; 1000], 1),
        ];
        
        let (_, bytes_no_comp) = aggregator_no_compression.aggregate(gradients.clone(), 1).await.unwrap();
        let (_, bytes_high_comp) = aggregator_high_compression.aggregate(gradients, 1).await.unwrap();
        
        // Higher compression should use fewer bytes
        assert!(bytes_high_comp <= bytes_no_comp);
    }

    #[tokio::test]
    async fn test_large_scale_aggregation() {
        let aggregator = GradientAggregator::new(0).await.unwrap();
        
        // Create 100 nodes with random gradients
        let mut gradients = Vec::new();
        for i in 0..100 {
            let values: Vec<f32> = (0..1000).map(|j| (i * j) as f32 * 0.001).collect();
            gradients.push(create_test_gradient(&format!("node{}", i), values, 1));
        }
        
        let (result, _) = aggregator.aggregate(gradients, 1).await.unwrap();
        
        // Should handle large-scale aggregation
        assert_eq!(result.values.len(), 1000);
        assert!(result.values.iter().all(|v| v.is_finite()));
    }

    #[tokio::test]
    async fn test_aggregation_strategies_consistency() {
        let strategies = vec![
            AggregationStrategy::Average,
            AggregationStrategy::TrimmedMean(0.1),
            AggregationStrategy::Median,
        ];
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0, 2.0], 1),
            create_test_gradient("node2", vec![3.0, 4.0], 1),
            create_test_gradient("node3", vec![5.0, 6.0], 1),
        ];
        
        for strategy in strategies {
            let mut aggregator = GradientAggregator::new(0).await.unwrap();
            aggregator.set_strategy(strategy);
            
            let (result, _) = aggregator.aggregate(gradients.clone(), 1).await.unwrap();
            
            // All strategies should produce valid results
            assert!(result.values.iter().all(|v| v.is_finite()));
            assert_eq!(result.values.len(), 2);
        }
    }

    #[tokio::test]
    async fn test_mixed_gradient_sizes() {
        let aggregator = GradientAggregator::new(0).await.unwrap();
        
        let gradients = vec![
            create_test_gradient("node1", vec![1.0, 2.0, 3.0], 1),
            create_test_gradient("node2", vec![4.0, 5.0], 1), // Different size
        ];
        
        let result = aggregator.aggregate(gradients, 1).await;
        
        // Should handle mixed sizes gracefully (error or truncate)
        assert!(result.is_err() || result.unwrap().0.values.len() <= 3);
    }
}