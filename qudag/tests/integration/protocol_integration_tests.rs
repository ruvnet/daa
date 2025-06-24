use qudag_protocol::{Coordinator, ProtocolConfig};
use qudag_crypto::KeyPair;
use qudag_network::NetworkManager;
use qudag_dag::QrDag;

use tokio::time::{sleep, Duration};
use tracing::info;

// Helper function to create test nodes
async fn create_test_nodes(count: usize) -> Vec<Coordinator> {
    let mut nodes = Vec::with_capacity(count);
    
    for i in 0..count {
        let mut config = ProtocolConfig::default();
        config.network_port = 8000 + i as u16;
        
        let node = Coordinator::new(config).await.unwrap();
        nodes.push(node);
    }
    
    nodes
}

#[tokio::test]
async fn test_multi_node_message_propagation() {
    let mut nodes = create_test_nodes(3).await;
    
    // Start all nodes
    for node in nodes.iter_mut() {
        node.start().await.unwrap();
    }
    
    // Give nodes time to discover each other
    sleep(Duration::from_secs(1)).await;
    
    // Broadcast message from first node
    let message = b"test message".to_vec();
    nodes[0].broadcast_message(message.clone()).await.unwrap();
    
    // Wait for propagation
    sleep(Duration::from_secs(1)).await;
    
    // Verify message reached all nodes
    for node in &nodes {
        let dag = node.dag_manager().unwrap();
        assert!(dag.contains_message(&message));
    }
    
    // Stop all nodes
    for node in nodes.iter_mut() {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_node_recovery() {
    let mut nodes = create_test_nodes(2).await;
    
    // Start nodes
    for node in nodes.iter_mut() {
        node.start().await.unwrap();
    }
    
    // Send some messages
    let messages = vec![
        b"message1".to_vec(),
        b"message2".to_vec(),
        b"message3".to_vec(),
    ];
    
    for msg in &messages {
        nodes[0].broadcast_message(msg.clone()).await.unwrap();
    }
    
    // Wait for propagation
    sleep(Duration::from_secs(1)).await;
    
    // Stop second node
    nodes[1].stop().await.unwrap();
    
    // Send more messages
    let new_messages = vec![
        b"message4".to_vec(),
        b"message5".to_vec(),
    ];
    
    for msg in &new_messages {
        nodes[0].broadcast_message(msg.clone()).await.unwrap();
    }
    
    // Restart second node
    nodes[1].start().await.unwrap();
    
    // Wait for sync
    sleep(Duration::from_secs(2)).await;
    
    // Verify all messages are present
    let all_messages = [messages, new_messages].concat();
    let dag = nodes[1].dag_manager().unwrap();
    
    for msg in all_messages {
        assert!(dag.contains_message(&msg));
    }
    
    // Stop nodes
    for node in nodes.iter_mut() {
        node.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_performance() {
    let mut nodes = create_test_nodes(5).await;
    
    // Start nodes
    for node in nodes.iter_mut() {
        node.start().await.unwrap();
    }
    
    // Wait for network setup
    sleep(Duration::from_secs(1)).await;
    
    // Send batch of messages and measure time
    let message_count = 1000;
    let start = std::time::Instant::now();
    
    for i in 0..message_count {
        let message = format!("test message {}", i).into_bytes();
        nodes[0].broadcast_message(message).await.unwrap();
    }
    
    let elapsed = start.elapsed();
    info!(
        "Sent {} messages in {:?} ({} msgs/sec)", 
        message_count,
        elapsed,
        message_count as f64 / elapsed.as_secs_f64()
    );
    
    // Wait for propagation
    sleep(Duration::from_secs(2)).await;
    
    // Verify all nodes have all messages
    for node in &nodes {
        let dag = node.dag_manager().unwrap();
        for i in 0..message_count {
            let message = format!("test message {}", i).into_bytes();
            assert!(dag.contains_message(&message));
        }
    }
    
    // Stop nodes
    for node in nodes.iter_mut() {
        node.stop().await.unwrap();
    }
}