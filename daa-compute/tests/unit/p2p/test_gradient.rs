//! Unit tests for gradient sharing and aggregation

use daa_compute::p2p::gradient::{AllReduceAlgorithm, GradientManager, GradientMessage};
use libp2p::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

#[cfg(test)]
mod tests {
    use super::*;

    /// A gradient message as a peer would send it over the network.
    fn create_test_message(
        manager: &GradientManager,
        peer_id: PeerId,
        values: &[f32],
        round: u64,
    ) -> GradientMessage {
        GradientMessage {
            peer_id,
            round,
            compressed_gradient: manager.compress_gradient(values).unwrap(),
            timestamp: SystemTime::now(),
        }
    }

    #[tokio::test]
    async fn test_gradient_manager_initialization() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);

        assert_eq!(manager.current_round(), 0);
        assert!(manager.gradients_for_round(0).await.is_empty());
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

        let result = manager
            .all_reduce(AllReduceAlgorithm::Ring, &gradients)
            .await
            .unwrap();

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

        let result = manager
            .all_reduce(AllReduceAlgorithm::Ring, &gradients)
            .await
            .unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }

    #[tokio::test]
    async fn test_gradient_aggregation_empty() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);

        let gradients = HashMap::new();
        let result = manager
            .all_reduce(AllReduceAlgorithm::Ring, &gradients)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gradient_aggregation_different_sizes() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 2);

        let mut gradients = HashMap::new();
        gradients.insert(PeerId::random(), vec![1.0, 2.0, 3.0]);
        gradients.insert(peer_id, vec![4.0, 5.0]); // Different size

        let result = manager
            .all_reduce(AllReduceAlgorithm::Ring, &gradients)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gradient_storage_and_retrieval() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);

        let sender = PeerId::random();
        let message = create_test_message(&manager, sender, &[1.0, 2.0, 3.0], 0);
        manager.handle_gradient_message(message).await.unwrap();

        // Check that gradient was stored (int8-quantised in transit)
        let stored = manager.gradients_for_round(0).await;
        assert_eq!(stored.len(), 1);
        for (got, want) in stored[&sender].iter().zip([1.0, 2.0, 3.0]) {
            assert!((got - want).abs() < 0.01);
        }
    }

    #[tokio::test]
    async fn test_gradient_nan_handling() {
        let peer_id = PeerId::random();
        let manager = GradientManager::new(peer_id, 3);

        let mut gradients = HashMap::new();
        gradients.insert(PeerId::random(), vec![1.0, f32::NAN, 3.0]);
        gradients.insert(peer_id, vec![4.0, 5.0, 6.0]);

        // Should handle NaN gracefully
        let result = manager
            .all_reduce(AllReduceAlgorithm::Ring, &gradients)
            .await;
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
        let result = manager
            .all_reduce(AllReduceAlgorithm::Ring, &gradients)
            .await;
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

        let result = manager
            .all_reduce(AllReduceAlgorithm::Ring, &gradients)
            .await
            .unwrap();
        assert_eq!(result.len(), 1000);

        // Check first value: (0+1000+2000+...+9000)/10 = 4500
        assert!((result[0] - 4500.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_concurrent_gradient_updates() {
        let peer_id = PeerId::random();
        let manager = Arc::new(GradientManager::new(peer_id, 5));

        // Spawn multiple concurrent tasks updating gradients
        let mut handles = vec![];

        for i in 0..5 {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                let message = create_test_message(&mgr, PeerId::random(), &[i as f32; 100], 0);
                mgr.handle_gradient_message(message).await
            });
            handles.push(handle);
        }

        // Wait for all updates
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let stored = manager.gradients_for_round(0).await;
        assert_eq!(stored.len(), 5);
    }

    #[tokio::test]
    async fn test_gradient_message_serialization() {
        let manager = GradientManager::new(PeerId::random(), 3);
        let sender = PeerId::random();
        let message = create_test_message(&manager, sender, &[1.0, 2.0, 3.0], 42);

        // Round-trip through the wire format used on the gossip topic
        let bytes = bincode::serialize(&message).unwrap();
        let decoded: GradientMessage = bincode::deserialize(&bytes).unwrap();

        assert_eq!(decoded.peer_id, sender);
        assert_eq!(decoded.round, 42);
        let values = manager
            .decompress_gradient(&decoded.compressed_gradient)
            .unwrap();
        for (got, want) in values.iter().zip([1.0, 2.0, 3.0]) {
            assert!((got - want).abs() < 0.01);
        }
    }
}
