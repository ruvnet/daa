//! Comprehensive tests for ML-KEM based onion routing

use qudag_crypto::kem::KeyEncapsulation;
use qudag_crypto::ml_kem::MlKem768;
use qudag_network::onion::*;
use qudag_network::router::Router;
use qudag_network::types::{MessagePriority, NetworkMessage, PeerId, RoutingStrategy};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_ml_kem_onion_router_creation() {
    let router = MLKEMOnionRouter::new().await.unwrap();
    assert!(router.get_public_key().as_bytes().len() > 0);
}

#[tokio::test]
async fn test_onion_layer_encryption_decryption() {
    let router = MLKEMOnionRouter::new().await.unwrap();

    // Create a simple route with 3 hops
    let mut route = Vec::new();
    for _ in 0..3 {
        let (pub_key, _) = MlKem768::keygen().unwrap();
        route.push(pub_key.as_bytes().to_vec());
    }

    let message = b"Secret message".to_vec();

    // Encrypt layers
    let layers = router
        .encrypt_layers(message.clone(), route.clone())
        .await
        .unwrap();
    assert_eq!(layers.len(), 3);

    // Verify each layer has proper structure
    for layer in &layers {
        assert!(!layer.kem_ciphertext.is_empty());
        assert!(!layer.payload.is_empty());
        assert!(layer.validate().is_ok());
    }
}

#[tokio::test]
async fn test_circuit_management() {
    let mut circuit_mgr = CircuitManager::new();
    let directory = DirectoryClient::new();

    // Build a circuit
    let circuit_id = circuit_mgr.build_circuit(3, &directory).await.unwrap();

    // Activate circuit
    circuit_mgr.activate_circuit(circuit_id).unwrap();

    // Get active circuit
    let circuit = circuit_mgr.get_active_circuit();
    assert!(circuit.is_some());
    assert_eq!(circuit.unwrap().state, CircuitState::Active);

    // Update metrics
    circuit_mgr.update_circuit_metrics(circuit_id, 1024, true);

    // Check stats
    let stats = circuit_mgr.get_stats();
    assert_eq!(stats.active_circuits, 1);
    assert_eq!(stats.total_bandwidth, 1024);
}

#[tokio::test]
async fn test_circuit_rotation() {
    let mut circuit_mgr = CircuitManager::new();
    let directory = DirectoryClient::new();

    // Build and activate multiple circuits
    let circuit1 = circuit_mgr.build_circuit(3, &directory).await.unwrap();
    circuit_mgr.activate_circuit(circuit1).unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let circuit2 = circuit_mgr.build_circuit(3, &directory).await.unwrap();
    circuit_mgr.activate_circuit(circuit2).unwrap();

    let stats = circuit_mgr.get_stats();
    assert_eq!(stats.total_circuits, 2);
    assert_eq!(stats.active_circuits, 2);
}

#[tokio::test]
async fn test_directory_client() {
    let directory = DirectoryClient::new();

    // Select random nodes
    let nodes = directory.select_random_nodes(3).await.unwrap();
    assert_eq!(nodes.len(), 3);

    // Verify all nodes are different
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            assert_ne!(nodes[i], nodes[j]);
        }
    }

    // Get load balancing weights
    let weights = directory.get_load_balancing_weights().await;
    assert!(!weights.is_empty());

    // Verify weights sum to approximately 1.0
    let total_weight: f64 = weights.values().sum();
    assert!((total_weight - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_mix_node_batching() {
    let mut mix_node = MixNode::new(vec![1, 2, 3]);

    // Add messages below batch threshold
    for i in 0..50 {
        let msg = MixMessage {
            content: format!("Message {}", i).into_bytes(),
            priority: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: MixMessageType::Real,
            normalized_size: 0,
        };

        mix_node.add_message(msg).await.unwrap();
    }

    // Force flush
    let batch = mix_node.flush_batch().await.unwrap();

    // Should have added dummy messages
    assert!(batch.len() >= 50);

    // Check for dummy messages
    let dummy_count = batch
        .iter()
        .filter(|m| matches!(m.message_type, MixMessageType::Dummy))
        .count();
    assert!(dummy_count > 0);
}

#[tokio::test]
async fn test_traffic_analysis_resistance() {
    let tar = TrafficAnalysisResistance::new();

    // Create test messages
    let mut messages = vec![];
    for i in 0..10 {
        messages.push(MixMessage {
            content: vec![i as u8; 100 + i * 10],
            priority: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: MixMessageType::Real,
            normalized_size: 0,
        });
    }

    // Apply resistance measures
    tar.apply_resistance(&mut messages).await.unwrap();

    // Verify messages have been normalized
    let sizes: Vec<_> = messages.iter().map(|m| m.normalized_size).collect();

    // All messages should have standard sizes
    for size in &sizes {
        assert!([512, 1024, 2048, 4096, 8192].contains(size));
    }
}

#[tokio::test]
async fn test_metadata_protection() {
    let protector = MetadataProtector::new();

    let original = b"sensitive metadata";
    let protected = protector.protect_metadata(original).unwrap();

    // Verify obfuscation
    assert!(protected.obfuscated_timestamp > 0);
    assert!(!protected.random_headers.is_empty());
    assert!(protected.normalized_size >= original.len());
    assert!(!protected.anonymous_ids.is_empty());

    // Test IP anonymization
    let original_ip = "192.168.1.100";
    let anon_ip = protector.anonymize_ip(original_ip).unwrap();
    assert_ne!(original_ip, anon_ip);
    assert!(anon_ip.starts_with("10.")); // Private range
}

#[tokio::test]
async fn test_router_with_onion_routing() {
    let router = Router::new().await.unwrap();

    // Add some test peers
    for i in 0..10 {
        router.add_peer(PeerId::random()).await;
    }

    // Create test message
    let message = NetworkMessage {
        id: "test-onion".to_string(),
        source: vec![0u8; 32],
        destination: vec![255u8; 32],
        payload: b"Anonymous message".to_vec(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(60),
    };

    // Route anonymously
    let route = router
        .route(&message, RoutingStrategy::Anonymous { hops: 3 })
        .await;

    // Should succeed with circuit building
    assert!(route.is_ok());
    let route = route.unwrap();
    assert!(route.len() >= 3);
}

#[tokio::test]
async fn test_circuit_teardown() {
    let mut circuit_mgr = CircuitManager::new();
    let directory = DirectoryClient::new();

    // Build and activate circuit
    let circuit_id = circuit_mgr.build_circuit(3, &directory).await.unwrap();
    circuit_mgr.activate_circuit(circuit_id).unwrap();

    // Teardown circuit
    circuit_mgr.teardown_circuit(circuit_id).await.unwrap();

    // Verify circuit is closed
    let stats = circuit_mgr.get_stats();
    assert_eq!(stats.active_circuits, 0);
}

#[tokio::test]
async fn test_layer_size_normalization() {
    let mut layer = OnionLayer::new(vec![1, 2, 3], vec![4, 5, 6, 7, 8], vec![9, 10]);

    let original_size = layer.total_size();
    layer.normalize_size(4096);

    assert_eq!(layer.total_size(), 4096);
    assert!(layer.padding.len() > 0);
}

#[tokio::test]
async fn test_circuit_quality_scoring() {
    let mut circuit_mgr = CircuitManager::new();
    let directory = DirectoryClient::new();

    let circuit_id = circuit_mgr.build_circuit(3, &directory).await.unwrap();
    circuit_mgr.activate_circuit(circuit_id).unwrap();

    // Simulate successful operations
    for _ in 0..10 {
        circuit_mgr.update_circuit_metrics(circuit_id, 1024, true);
    }

    let circuit = circuit_mgr.get_active_circuit().unwrap();
    assert!(circuit.quality_score > 0.9);

    // Simulate failures
    for _ in 0..5 {
        circuit_mgr.update_circuit_metrics(circuit_id, 0, false);
    }

    let circuit = circuit_mgr.get_active_circuit().unwrap();
    assert!(circuit.quality_score < 0.9);
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut circuit_mgr = CircuitManager::new();
    let directory = DirectoryClient::new();

    // First circuit should succeed
    let result1 = circuit_mgr.build_circuit(3, &directory).await;
    assert!(result1.is_ok());

    // Immediate second circuit should fail due to rate limiting
    let result2 = circuit_mgr.build_circuit(3, &directory).await;
    assert!(result2.is_err());

    // Wait for rate limit to expire
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should succeed now
    let result3 = circuit_mgr.build_circuit(3, &directory).await;
    assert!(result3.is_ok());
}

#[tokio::test]
async fn test_bandwidth_measurement() {
    let directory = DirectoryClient::new();

    // Force populate nodes
    directory.select_random_nodes(1).await.unwrap();

    // Test bandwidth measurement
    let nodes = directory.nodes.lock().await;
    if let Some((node_id, _)) = nodes.iter().next() {
        drop(nodes); // Release lock
        let bandwidth = directory.measure_bandwidth(node_id).await.unwrap();
        assert!(bandwidth > 0);
    }
}
