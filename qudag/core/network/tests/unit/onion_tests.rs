//! Unit tests for onion routing module

use qudag_network::onion::{
    MLKEMOnionRouter, MetadataConfig, MetadataProtector, MixConfig, MixMessage, 
    MixMessageType, MixNode, MixNodeStats, OnionError, OnionLayer, OnionRouter, 
    ProtectedMetadata, TrafficAnalysisConfig, TrafficAnalysisResistance,
};
use qudag_network::types::{NetworkMessage, PeerId};
use qudag_crypto::ml_kem::{MLKEMKeyPair, MLKEMParams};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_onion_layer_creation() {
    let peer_id = PeerId::random();
    let key = vec![0u8; 32];
    let payload = b"test message".to_vec();
    
    let layer = OnionLayer::new(peer_id, key.clone(), payload.clone());
    
    assert_eq!(layer.peer_id(), peer_id);
    assert_eq!(layer.encryption_key(), &key);
    assert_eq!(layer.payload(), &payload);
}

#[tokio::test]
async fn test_onion_router_basic() {
    let mut router = OnionRouter::new();
    
    // Create a simple route
    let peers = vec![PeerId::random(), PeerId::random(), PeerId::random()];
    let message = NetworkMessage::Data(b"secret data".to_vec());
    
    // Wrap message
    let wrapped = router.wrap_message(message.clone(), peers.clone()).await;
    assert!(wrapped.is_ok());
    
    let onion_message = wrapped.unwrap();
    assert!(!onion_message.layers.is_empty());
    assert_eq!(onion_message.layers.len(), peers.len());
}

#[tokio::test]
async fn test_onion_router_unwrap() {
    let mut router = OnionRouter::new();
    let my_peer_id = PeerId::random();
    
    // Create message with us as first hop
    let peers = vec![my_peer_id, PeerId::random(), PeerId::random()];
    let message = NetworkMessage::Data(b"test".to_vec());
    
    let wrapped = router.wrap_message(message, peers).await.unwrap();
    
    // Unwrap first layer
    let unwrapped = router.unwrap_layer(wrapped, my_peer_id).await;
    assert!(unwrapped.is_ok());
    
    let (next_hop, remaining) = unwrapped.unwrap();
    assert!(next_hop.is_some());
    assert!(remaining.is_some());
}

#[tokio::test]
async fn test_ml_kem_onion_router() {
    let router = MLKEMOnionRouter::new(MLKEMParams::ML_KEM_768);
    
    // Generate keys for nodes
    let mut keys = HashMap::new();
    let peers: Vec<PeerId> = (0..3).map(|_| PeerId::random()).collect();
    
    for peer in &peers {
        let keypair = MLKEMKeyPair::generate(MLKEMParams::ML_KEM_768);
        keys.insert(*peer, keypair);
    }
    
    // Create quantum-resistant onion message
    let message = b"quantum secret".to_vec();
    let public_keys: HashMap<_, _> = keys.iter()
        .map(|(id, kp)| (*id, kp.public_key.clone()))
        .collect();
    
    let wrapped = router.wrap_quantum_message(message.clone(), peers.clone(), &public_keys).await;
    assert!(wrapped.is_ok());
    
    // Verify structure
    let onion = wrapped.unwrap();
    assert_eq!(onion.layers.len(), peers.len());
}

#[tokio::test]
async fn test_mix_node_basic() {
    let node_id = PeerId::random();
    let config = MixConfig {
        pool_size: 10,
        delay_min: Duration::from_millis(10),
        delay_max: Duration::from_millis(50),
        batch_size: 5,
    };
    
    let mut mix_node = MixNode::new(node_id, config);
    
    // Add messages
    for i in 0..10 {
        let msg = MixMessage {
            id: i,
            msg_type: MixMessageType::Forward,
            payload: vec![i as u8; 100],
            next_hop: Some(PeerId::random()),
            timestamp: Instant::now(),
        };
        mix_node.add_message(msg);
    }
    
    // Process batch
    tokio::time::sleep(Duration::from_millis(60)).await;
    let batch = mix_node.process_batch().await;
    
    assert!(!batch.is_empty());
    assert!(batch.len() <= 5); // Batch size limit
}

#[tokio::test]
async fn test_mix_node_stats() {
    let node_id = PeerId::random();
    let config = MixConfig::default();
    let mut mix_node = MixNode::new(node_id, config);
    
    // Add some messages
    for i in 0..5 {
        let msg = MixMessage {
            id: i,
            msg_type: MixMessageType::Forward,
            payload: vec![0u8; 50],
            next_hop: Some(PeerId::random()),
            timestamp: Instant::now(),
        };
        mix_node.add_message(msg);
    }
    
    let stats = mix_node.get_stats();
    assert_eq!(stats.messages_received, 5);
    assert_eq!(stats.messages_sent, 0); // Not processed yet
    assert_eq!(stats.current_pool_size, 5);
}

#[tokio::test]
async fn test_metadata_protector() {
    let config = MetadataConfig {
        hide_size: true,
        padding_min: 100,
        padding_max: 500,
        fake_traffic_rate: 0.1,
    };
    
    let protector = MetadataProtector::new(config);
    
    // Protect metadata
    let original = b"sensitive metadata".to_vec();
    let protected = protector.protect(original.clone()).await;
    
    assert!(protected.is_ok());
    let result = protected.unwrap();
    
    // Should be padded
    assert!(result.padded_data.len() >= 100);
    assert!(result.padded_data.len() <= 500);
    assert_eq!(result.original_size, original.len());
}

#[tokio::test]
async fn test_traffic_analysis_resistance() {
    let config = TrafficAnalysisConfig {
        enable_padding: true,
        enable_timing_obfuscation: true,
        enable_dummy_traffic: true,
        dummy_traffic_rate: 0.2,
        timing_jitter_ms: 50,
    };
    
    let mut tar = TrafficAnalysisResistance::new(config);
    
    // Apply resistance measures
    let message = b"secret".to_vec();
    let obfuscated = tar.apply_resistance(message.clone()).await;
    
    assert!(obfuscated.is_ok());
    let result = obfuscated.unwrap();
    
    // Should have padding
    assert!(result.len() > message.len());
}

#[tokio::test]
async fn test_onion_routing_full_cycle() {
    let mut routers = HashMap::new();
    let peers: Vec<PeerId> = (0..4).map(|_| PeerId::random()).collect();
    
    // Create router for each peer
    for peer in &peers {
        routers.insert(*peer, OnionRouter::new());
    }
    
    // Original message
    let original_message = NetworkMessage::Data(b"end-to-end test".to_vec());
    
    // Sender wraps message
    let wrapped = routers.get_mut(&peers[0]).unwrap()
        .wrap_message(original_message.clone(), peers[1..].to_vec())
        .await
        .unwrap();
    
    // Simulate forwarding through network
    let mut current_message = wrapped;
    let mut current_peer_idx = 1;
    
    while current_peer_idx < peers.len() {
        let current_peer = peers[current_peer_idx];
        let router = routers.get_mut(&current_peer).unwrap();
        
        let result = router.unwrap_layer(current_message.clone(), current_peer).await;
        
        if let Ok((next_hop, remaining)) = result {
            if let Some(remaining_msg) = remaining {
                current_message = remaining_msg;
                current_peer_idx += 1;
            } else {
                // Final destination reached
                break;
            }
        } else {
            panic!("Failed to unwrap at peer {:?}", current_peer);
        }
    }
}

#[tokio::test]
async fn test_onion_error_handling() {
    let mut router = OnionRouter::new();
    
    // Test with empty route
    let message = NetworkMessage::Data(b"test".to_vec());
    let result = router.wrap_message(message.clone(), vec![]).await;
    assert!(matches!(result, Err(OnionError::InvalidRoute)));
    
    // Test unwrapping with wrong peer ID
    let peers = vec![PeerId::random(), PeerId::random()];
    let wrapped = router.wrap_message(message, peers).await.unwrap();
    
    let wrong_peer = PeerId::random();
    let result = router.unwrap_layer(wrapped, wrong_peer).await;
    assert!(matches!(result, Err(OnionError::InvalidLayer)));
}

#[tokio::test]
async fn test_mix_cascade() {
    // Create cascade of mix nodes
    let mut mix_nodes = vec![];
    let config = MixConfig {
        pool_size: 5,
        delay_min: Duration::from_millis(1),
        delay_max: Duration::from_millis(5),
        batch_size: 3,
    };
    
    for i in 0..3 {
        let node_id = PeerId::from_bytes([i as u8; 32]);
        mix_nodes.push(MixNode::new(node_id, config.clone()));
    }
    
    // Send messages through cascade
    for i in 0..10 {
        let msg = MixMessage {
            id: i,
            msg_type: MixMessageType::Forward,
            payload: vec![i as u8; 50],
            next_hop: Some(mix_nodes[1].node_id()),
            timestamp: Instant::now(),
        };
        mix_nodes[0].add_message(msg);
    }
    
    // Process through cascade
    tokio::time::sleep(Duration::from_millis(10)).await;
    let batch1 = mix_nodes[0].process_batch().await;
    
    for msg in batch1 {
        if let Some(next_hop) = msg.next_hop {
            if next_hop == mix_nodes[1].node_id() {
                mix_nodes[1].add_message(msg);
            }
        }
    }
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    let batch2 = mix_nodes[1].process_batch().await;
    assert!(!batch2.is_empty());
}

#[tokio::test]
async fn test_traffic_pattern_obfuscation() {
    let config = TrafficAnalysisConfig {
        enable_padding: true,
        enable_timing_obfuscation: true,
        enable_dummy_traffic: true,
        dummy_traffic_rate: 0.5,
        timing_jitter_ms: 100,
    };
    
    let mut tar = TrafficAnalysisResistance::new(config);
    
    // Generate traffic pattern
    let mut sizes = vec![];
    let mut timings = vec![];
    let start = Instant::now();
    
    for i in 0..20 {
        let message = vec![0u8; 10 + i * 5];
        let obfuscated = tar.apply_resistance(message).await.unwrap();
        sizes.push(obfuscated.len());
        timings.push(start.elapsed());
        
        // Should generate dummy traffic sometimes
        if tar.should_send_dummy() {
            let dummy = tar.generate_dummy_traffic().await;
            assert!(!dummy.is_empty());
        }
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Verify obfuscation
    let unique_sizes: std::collections::HashSet<_> = sizes.iter().collect();
    assert!(unique_sizes.len() < sizes.len()); // Some sizes should repeat due to padding
}