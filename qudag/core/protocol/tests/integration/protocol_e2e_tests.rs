use qudag_protocol::{Coordinator, ProtocolConfig, ProtocolState};
use qudag_crypto::KeyPair;
use qudag_dag::QrDag;
use qudag_network::NetworkManager;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_full_protocol_lifecycle() {
    // Initialize multiple protocol instances to simulate network
    let config1 = ProtocolConfig {
        network_port: 8001,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
    };
    
    let config2 = ProtocolConfig {
        network_port: 8002,
        bootstrap_nodes: vec!["127.0.0.1:8001".to_string()],
        max_peers: 10,
        validation_timeout: 1000,
    };
    
    let mut coordinator1 = Coordinator::new(config1).await.unwrap();
    let mut coordinator2 = Coordinator::new(config2).await.unwrap();
    
    // Start both coordinators
    coordinator1.start().await.unwrap();
    coordinator2.start().await.unwrap();
    
    // Allow time for network connection
    sleep(Duration::from_millis(100)).await;
    
    // Send message through the network
    let test_message = b"test message".to_vec();
    coordinator1.broadcast_message(test_message.clone()).await.unwrap();
    
    // Allow time for message propagation
    sleep(Duration::from_millis(100)).await;
    
    // Verify message was received and processed by both nodes
    assert!(coordinator1.dag_manager().unwrap().contains_message(&test_message));
    assert!(coordinator2.dag_manager().unwrap().contains_message(&test_message));
    
    // Test shutdown
    coordinator1.stop().await.unwrap();
    coordinator2.stop().await.unwrap();
    
    assert_eq!(coordinator1.state().await, ProtocolState::Stopped);
    assert_eq!(coordinator2.state().await, ProtocolState::Stopped);
}

#[tokio::test]
async fn test_consensus_convergence() {
    // Initialize a network of three nodes
    let configs = vec![
        ProtocolConfig {
            network_port: 8003,
            bootstrap_nodes: vec![],
            max_peers: 10,
            validation_timeout: 1000,
        },
        ProtocolConfig {
            network_port: 8004,
            bootstrap_nodes: vec!["127.0.0.1:8003".to_string()],
            max_peers: 10,
            validation_timeout: 1000,
        },
        ProtocolConfig {
            network_port: 8005,
            bootstrap_nodes: vec!["127.0.0.1:8003".to_string()],
            max_peers: 10,
            validation_timeout: 1000,
        },
    ];
    
    let mut coordinators = Vec::new();
    for config in configs {
        let coordinator = Coordinator::new(config).await.unwrap();
        coordinators.push(coordinator);
    }
    
    // Start all coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.start().await.unwrap();
    }
    
    // Allow time for network formation
    sleep(Duration::from_millis(200)).await;
    
    // Generate test messages
    let messages = vec![
        b"message1".to_vec(),
        b"message2".to_vec(),
        b"message3".to_vec(),
    ];
    
    // Broadcast messages from different nodes
    for (i, message) in messages.iter().enumerate() {
        coordinators[i].broadcast_message(message.clone()).await.unwrap();
    }
    
    // Allow time for consensus convergence
    sleep(Duration::from_millis(500)).await;
    
    // Verify all nodes have the same messages
    for message in &messages {
        for coordinator in &coordinators {
            assert!(coordinator.dag_manager().unwrap().contains_message(message));
        }
    }
    
    // Stop all coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_byzantine_resistance() {
    // TODO: Implement Byzantine fault tolerance testing
    // This will involve creating malicious nodes and ensuring the network
    // maintains consistency despite their presence
}

#[tokio::test]
async fn test_performance_under_load() {
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    coordinator.start().await.unwrap();
    
    // Generate large number of messages
    let num_messages = 1000;
    let message_size = 1000;
    let messages: Vec<Vec<u8>> = (0..num_messages)
        .map(|i| vec![i as u8; message_size])
        .collect();
    
    // Measure time to process all messages
    let start = std::time::Instant::now();
    
    for message in messages {
        coordinator.broadcast_message(message).await.unwrap();
    }
    
    let duration = start.elapsed();
    
    // Assert performance meets requirements (sub-second for 1000 messages)
    assert!(duration < Duration::from_secs(1));
    
    coordinator.stop().await.unwrap();
}

#[tokio::test]
async fn test_cryptographic_verification() {
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    coordinator.start().await.unwrap();
    
    // Get original keypair
    let keypair = coordinator.crypto_manager().unwrap();
    
    // Create test message
    let message = b"test message".to_vec();
    
    // Sign and broadcast message
    coordinator.broadcast_message(message.clone()).await.unwrap();
    
    // Verify signature in DAG
    let dag = coordinator.dag_manager().unwrap();
    assert!(dag.verify_message(&message, keypair.public_key()));
    
    coordinator.stop().await.unwrap();
}