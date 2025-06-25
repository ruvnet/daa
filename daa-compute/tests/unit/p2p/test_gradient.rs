//! Unit tests for gradient sharing and aggregation

use daa_compute::p2p::gradient::{GradientManager, AllReduceAlgorithm, GradientMessage};
use daa_compute::training::{Gradient, ModelParameters};
use libp2p::PeerId;
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
    async fn test_gradient_manager_initialization() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);
        
        assert_eq!(manager.get_num_peers(), 3);
        assert!(manager.get_aggregated_gradient().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_ring_allreduce_basic() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);
        
        let mut gradients = HashMap::new();
        gradients.insert(PeerId::random(), vec![1.0, 2.0, 3.0]);
        gradients.insert(PeerId::random(), vec![4.0, 5.0, 6.0]);
        gradients.insert(peer_id, vec![7.0, 8.0, 9.0]);
        
        let result = manager.ring_allreduce(&gradients).await.unwrap();
        
        // Check averaging: (1+4+7)/3=4, (2+5+8)/3=5, (3+6+9)/3=6
        assert_eq!(result.len(), 3);
        assert!((result[0] - 4.0).abs() < 0.001);
        assert!((result[1] - 5.0).abs() < 0.001);
        assert!((result[2] - 6.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_ring_allreduce_single_node() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 1);
        
        let mut gradients = HashMap::new();
        gradients.insert(peer_id, vec![1.0, 2.0, 3.0]);
        
        let result = manager.ring_allreduce(&gradients).await.unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }

    #[tokio::test]
    async fn test_gradient_aggregation_empty() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);
        
        let gradients = HashMap::new();
        let result = manager.ring_allreduce(&gradients).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gradient_aggregation_different_sizes() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 2);
        
        let mut gradients = HashMap::new();
        gradients.insert(PeerId::random(), vec![1.0, 2.0, 3.0]);
        gradients.insert(peer_id, vec![4.0, 5.0]); // Different size
        
        let result = manager.ring_allreduce(&gradients).await;
        assert!(result.is_err() || result.unwrap().len() == 2);
    }

    #[tokio::test]
    async fn test_gradient_storage_and_retrieval() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);
        
        let gradient = create_test_gradient("node1", vec![1.0, 2.0, 3.0], 1);
        manager.store_gradient(PeerId::random(), gradient.clone()).await.unwrap();
        
        // Check that gradient was stored
        let stored = manager.get_stored_gradients().await;
        assert_eq!(stored.len(), 1);
    }

    #[tokio::test]
    async fn test_gradient_nan_handling() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);
        
        let mut gradients = HashMap::new();
        gradients.insert(PeerId::random(), vec![1.0, f32::NAN, 3.0]);
        gradients.insert(peer_id, vec![4.0, 5.0, 6.0]);
        
        // Should handle NaN gracefully
        let result = manager.ring_allreduce(&gradients).await;
        if let Ok(values) = result {
            assert!(values.iter().all(|v| v.is_finite()));
        }
    }

    #[tokio::test]
    async fn test_gradient_infinity_handling() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);
        
        let mut gradients = HashMap::new();
        gradients.insert(PeerId::random(), vec![1.0, f32::INFINITY, 3.0]);
        gradients.insert(peer_id, vec![4.0, 5.0, 6.0]);
        
        // Should handle infinity gracefully
        let result = manager.ring_allreduce(&gradients).await;
        if let Ok(values) = result {
            assert!(values.iter().all(|v| v.is_finite()));
        }
    }

    #[tokio::test]
    async fn test_large_gradient_aggregation() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 10);
        
        let mut gradients = HashMap::new();
        
        // Create large gradients from 10 nodes
        for i in 0..10 {
            let values: Vec<f32> = (0..1000).map(|j| (i * 1000 + j) as f32).collect();
            let peer = if i == 0 { peer_id } else { PeerId::random() };
            gradients.insert(peer, values);
        }
        
        let result = manager.ring_allreduce(&gradients).await.unwrap();
        assert_eq!(result.len(), 1000);
        
        // Check first value: (0+1000+2000+...+9000)/10 = 4500
        assert!((result[0] - 4500.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_concurrent_gradient_updates() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 5);
        
        // Spawn multiple concurrent tasks updating gradients
        let mut handles = vec![];
        
        for i in 0..5 {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                let gradient = create_test_gradient(&format!("node{}", i), vec![i as f32; 100], 1);
                mgr.store_gradient(PeerId::random(), gradient).await
            });
            handles.push(handle);
        }
        
        // Wait for all updates
        for handle in handles {
            handle.await.unwrap().unwrap();
        }
        
        let stored = manager.get_stored_gradients().await;
        assert_eq!(stored.len(), 5);
    }

    #[tokio::test]
    async fn test_gradient_message_serialization() {
        let gradient = create_test_gradient("test", vec![1.0, 2.0, 3.0], 42);
        let message = GradientMessage::Update {
            gradient: gradient.clone(),
            round: 42,
        };
        
        // Test that gradient message can be created and matched
        match message {
            GradientMessage::Update { gradient: g, round: r } => {
                assert_eq!(g.node_id, "test");
                assert_eq!(g.values, vec![1.0, 2.0, 3.0]);
                assert_eq!(r, 42);
            }
            _ => panic!("Wrong message type"),
        }
    }
}