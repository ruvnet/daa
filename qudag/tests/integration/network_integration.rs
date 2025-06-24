//! Integration tests for network layer functionality

use qudag_network::{
    NetworkManager, P2PMessage, RoutingTable, Connection,
    types::{PeerId, MessageId, NetworkConfig},
    routing::OnionRouter,
    discovery::PeerDiscovery,
    metrics::NetworkMetrics,
};
use qudag_crypto::{
    fingerprint::Fingerprint,
    ml_kem::MlKem768,
};
use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn, error};

#[tokio::test]
async fn test_peer_discovery_and_connection() {
    // Test automatic peer discovery and connection establishment
    let config1 = NetworkConfig {
        listen_port: 10001,
        max_peers: 50,
        connection_timeout: Duration::from_secs(5),
        discovery_interval: Duration::from_secs(1),
        bootstrap_nodes: vec![],
    };
    
    let config2 = NetworkConfig {
        listen_port: 10002,
        max_peers: 50,
        connection_timeout: Duration::from_secs(5),
        discovery_interval: Duration::from_secs(1),
        bootstrap_nodes: vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10001)
        ],
    };
    
    let config3 = NetworkConfig {
        listen_port: 10003,
        max_peers: 50,
        connection_timeout: Duration::from_secs(5),
        discovery_interval: Duration::from_secs(1),
        bootstrap_nodes: vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10001)
        ],
    };
    
    let mut network1 = NetworkManager::new(config1).await.unwrap();
    let mut network2 = NetworkManager::new(config2).await.unwrap();
    let mut network3 = NetworkManager::new(config3).await.unwrap();
    
    // Start all networks
    network1.start().await.unwrap();
    network2.start().await.unwrap();
    network3.start().await.unwrap();
    
    // Allow time for discovery
    sleep(Duration::from_secs(3)).await;
    
    // Verify connections established
    let peers1 = network1.get_connected_peers().await;
    let peers2 = network2.get_connected_peers().await;
    let peers3 = network3.get_connected_peers().await;
    
    assert!(peers1.len() >= 2, "Network1 should have at least 2 peers");
    assert!(peers2.len() >= 2, "Network2 should have at least 2 peers");
    assert!(peers3.len() >= 2, "Network3 should have at least 2 peers");
    
    info!(
        "Peer counts - Network1: {}, Network2: {}, Network3: {}",
        peers1.len(), peers2.len(), peers3.len()
    );
    
    // Test message broadcasting
    let test_message = P2PMessage::new(
        MessageId::generate(),
        network1.local_peer_id(),
        b"Test broadcast message".to_vec(),
    );
    
    network1.broadcast_message(test_message.clone()).await.unwrap();
    
    // Allow propagation
    sleep(Duration::from_millis(500)).await;
    
    // Verify message received by other networks
    let messages2 = network2.get_received_messages().await;
    let messages3 = network3.get_received_messages().await;
    
    assert!(
        messages2.iter().any(|msg| msg.id() == test_message.id()),
        "Network2 didn't receive broadcast message"
    );
    assert!(
        messages3.iter().any(|msg| msg.id() == test_message.id()),
        "Network3 didn't receive broadcast message"
    );
    
    // Stop networks
    network1.stop().await.unwrap();
    network2.stop().await.unwrap();
    network3.stop().await.unwrap();
}

#[tokio::test]
async fn test_onion_routing() {
    // Test anonymous onion routing through multiple hops
    let mut networks = Vec::new();
    
    // Create 5 nodes for onion routing
    for i in 0..5 {
        let config = NetworkConfig {
            listen_port: 10100 + i as u16,
            max_peers: 10,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10100)]
            },
        };
        
        let network = NetworkManager::new(config).await.unwrap();
        networks.push(network);
    }
    
    // Start all networks
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    // Allow network formation
    sleep(Duration::from_secs(2)).await;
    
    // Create onion router for source node
    let mut onion_router = OnionRouter::new(networks[0].local_peer_id());
    
    // Get routing path from node 0 to node 4 through nodes 1, 2, 3
    let path = vec![
        networks[1].local_peer_id(),
        networks[2].local_peer_id(),
        networks[3].local_peer_id(),
        networks[4].local_peer_id(),
    ];
    
    // Generate encryption keys for each hop
    let mut hop_keys = Vec::new();
    for network in &networks[1..] {
        let (pk, _sk) = MlKem768::generate_keypair();
        hop_keys.push(pk);
        // In real implementation, public keys would be exchanged
    }
    
    // Create onion-encrypted message
    let original_message = b"Anonymous message through onion routing";
    let onion_message = onion_router.create_onion_message(
        original_message.to_vec(),
        &path,
        &hop_keys,
    ).await.unwrap();
    
    // Send onion message
    networks[0].send_onion_message(onion_message).await.unwrap();
    
    // Allow routing time
    sleep(Duration::from_millis(1000)).await;
    
    // Verify destination received original message
    let received_messages = networks[4].get_received_messages().await;
    assert!(
        received_messages.iter().any(|msg| {
            msg.payload() == original_message
        }),
        "Destination didn't receive onion-routed message"
    );
    
    // Verify intermediate nodes don't have plaintext
    for i in 1..4 {
        let intermediate_messages = networks[i].get_plaintext_messages().await;
        assert!(
            !intermediate_messages.iter().any(|msg| msg == original_message),
            "Intermediate node {} has access to plaintext", i
        );
    }
    
    // Stop all networks
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_network_resilience() {
    // Test network resilience to node failures
    let mut networks = Vec::new();
    
    // Create 6 nodes
    for i in 0..6 {
        let config = NetworkConfig {
            listen_port: 10200 + i as u16,
            max_peers: 10,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10200)]
            },
        };
        
        let network = NetworkManager::new(config).await.unwrap();
        networks.push(network);
    }
    
    // Start all networks
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    // Allow network formation
    sleep(Duration::from_secs(2)).await;
    
    // Verify initial connectivity
    for (i, network) in networks.iter().enumerate() {
        let peers = network.get_connected_peers().await;
        info!("Network {} has {} peers initially", i, peers.len());
        assert!(peers.len() >= 3, "Network {} should have at least 3 peers", i);
    }
    
    // Send test messages
    let test_messages: Vec<_> = (0..10).map(|i| {
        P2PMessage::new(
            MessageId::generate(),
            networks[0].local_peer_id(),
            format!("Test message {}", i).into_bytes(),
        )
    }).collect();
    
    for msg in &test_messages {
        networks[0].broadcast_message(msg.clone()).await.unwrap();
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Verify all nodes received messages
    for (i, network) in networks.iter().enumerate() {
        let received = network.get_received_messages().await;
        assert_eq!(
            received.len(), 10,
            "Network {} should have received 10 messages before failure", i
        );
    }
    
    // Simulate node failure - stop nodes 1 and 2
    info!("Simulating failure of nodes 1 and 2");
    networks[1].stop().await.unwrap();
    networks[2].stop().await.unwrap();
    
    // Allow time for network to adapt
    sleep(Duration::from_secs(3)).await;
    
    // Send more messages after failure
    let recovery_messages: Vec<_> = (10..15).map(|i| {
        P2PMessage::new(
            MessageId::generate(),
            networks[0].local_peer_id(),
            format!("Recovery message {}", i).into_bytes(),
        )
    }).collect();
    
    for msg in &recovery_messages {
        networks[0].broadcast_message(msg.clone()).await.unwrap();
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Verify remaining nodes still receive messages
    for i in [0, 3, 4, 5] {
        let received = networks[i].get_received_messages().await;
        assert!(
            received.len() >= 15,
            "Network {} should have received recovery messages", i
        );
    }
    
    // Test network recovery - restart failed nodes
    info!("Restarting failed nodes");
    networks[1].start().await.unwrap();
    networks[2].start().await.unwrap();
    
    // Allow reconnection
    sleep(Duration::from_secs(3)).await;
    
    // Send final test messages
    let final_message = P2PMessage::new(
        MessageId::generate(),
        networks[0].local_peer_id(),
        b"Final recovery test message".to_vec(),
    );
    
    networks[0].broadcast_message(final_message.clone()).await.unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    // Verify all nodes (including recovered ones) receive final message
    for (i, network) in networks.iter().enumerate() {
        let received = network.get_received_messages().await;
        assert!(
            received.iter().any(|msg| msg.id() == final_message.id()),
            "Network {} should have received final recovery message", i
        );
    }
    
    // Stop all networks
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_routing_table_management() {
    // Test routing table updates and maintenance
    let config = NetworkConfig {
        listen_port: 10300,
        max_peers: 10,
        connection_timeout: Duration::from_secs(5),
        discovery_interval: Duration::from_secs(1),
        bootstrap_nodes: vec![],
    };
    
    let mut network = NetworkManager::new(config).await.unwrap();
    network.start().await.unwrap();
    
    let routing_table = network.routing_table();
    
    // Add peers manually for testing
    let peer_ids: Vec<PeerId> = (0..20).map(|i| {
        PeerId::from_bytes(&[i as u8; 32])
    }).collect();
    
    for (i, peer_id) in peer_ids.iter().enumerate() {
        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            11000 + i as u16
        );
        routing_table.add_peer(*peer_id, addr).await.unwrap();
    }
    
    // Verify routing table size
    let table_size = routing_table.size().await;
    assert_eq!(table_size, 20, "Routing table should contain 20 peers");
    
    // Test peer lookup
    for peer_id in &peer_ids[..10] {
        let addr = routing_table.lookup_peer(*peer_id).await.unwrap();
        assert!(addr.is_some(), "Should find address for peer");
    }
    
    // Test routing table cleanup (remove offline peers)
    for peer_id in &peer_ids[10..15] {
        routing_table.mark_peer_offline(*peer_id).await.unwrap();
    }
    
    routing_table.cleanup_offline_peers().await.unwrap();
    
    let new_size = routing_table.size().await;
    assert_eq!(new_size, 15, "Routing table should have 15 peers after cleanup");
    
    // Test peer discovery integration
    let discovered_peers = network.discover_peers().await.unwrap();
    info!("Discovered {} peers", discovered_peers.len());
    
    network.stop().await.unwrap();
}

#[tokio::test]
async fn test_message_ordering_and_delivery() {
    // Test message ordering and guaranteed delivery
    let config1 = NetworkConfig {
        listen_port: 10400,
        max_peers: 10,
        connection_timeout: Duration::from_secs(5),
        discovery_interval: Duration::from_secs(1),
        bootstrap_nodes: vec![],
    };
    
    let config2 = NetworkConfig {
        listen_port: 10401,
        max_peers: 10,
        connection_timeout: Duration::from_secs(5),
        discovery_interval: Duration::from_secs(1),
        bootstrap_nodes: vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10400)
        ],
    };
    
    let mut network1 = NetworkManager::new(config1).await.unwrap();
    let mut network2 = NetworkManager::new(config2).await.unwrap();
    
    network1.start().await.unwrap();
    network2.start().await.unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    // Send ordered sequence of messages
    let message_count = 100;
    let mut sent_messages = Vec::new();
    
    for i in 0..message_count {
        let message = P2PMessage::new(
            MessageId::generate(),
            network1.local_peer_id(),
            format!("Ordered message {}", i).into_bytes(),
        );
        sent_messages.push(message.clone());
        network1.send_reliable_message(network2.local_peer_id(), message).await.unwrap();
        
        // Small delay to ensure ordering
        tokio::task::yield_now().await;
    }
    
    // Allow delivery
    sleep(Duration::from_millis(1000)).await;
    
    // Verify all messages received in order
    let received_messages = network2.get_ordered_messages().await;
    assert_eq!(
        received_messages.len(), message_count,
        "Should receive all {} messages", message_count
    );
    
    for (i, msg) in received_messages.iter().enumerate() {
        let expected_content = format!("Ordered message {}", i);
        assert_eq!(
            msg.payload(),
            expected_content.as_bytes(),
            "Message {} out of order", i
        );
    }
    
    network1.stop().await.unwrap();
    network2.stop().await.unwrap();
}

#[tokio::test]
async fn test_network_metrics_and_monitoring() {
    // Test network metrics collection and monitoring
    let config = NetworkConfig {
        listen_port: 10500,
        max_peers: 10,
        connection_timeout: Duration::from_secs(5),
        discovery_interval: Duration::from_secs(1),
        bootstrap_nodes: vec![],
    };
    
    let mut network = NetworkManager::new(config).await.unwrap();
    network.start().await.unwrap();
    
    let metrics = network.metrics();
    
    // Initial metrics should be zero
    assert_eq!(metrics.messages_sent(), 0);
    assert_eq!(metrics.messages_received(), 0);
    assert_eq!(metrics.bytes_sent(), 0);
    assert_eq!(metrics.bytes_received(), 0);
    
    // Send some messages to self for testing
    let test_messages: Vec<_> = (0..10).map(|i| {
        P2PMessage::new(
            MessageId::generate(),
            network.local_peer_id(),
            format!("Metrics test message {}", i).into_bytes(),
        )
    }).collect();
    
    for msg in &test_messages {
        network.broadcast_message(msg.clone()).await.unwrap();
    }
    
    sleep(Duration::from_millis(200)).await;
    
    // Check updated metrics
    let updated_metrics = network.metrics();
    assert_eq!(updated_metrics.messages_sent(), 10);
    assert!(updated_metrics.bytes_sent() > 0);
    
    // Test latency metrics
    let latencies = updated_metrics.message_latencies();
    assert!(!latencies.is_empty(), "Should have latency measurements");
    
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    info!("Average message latency: {:?}", avg_latency);
    
    // Test throughput calculation
    let throughput = updated_metrics.calculate_throughput().await;
    info!("Network throughput: {} msgs/sec", throughput);
    
    network.stop().await.unwrap();
}

#[tokio::test]
async fn test_connection_pooling() {
    // Test connection pooling and reuse
    let mut networks = Vec::new();
    
    // Create 3 nodes
    for i in 0..3 {
        let config = NetworkConfig {
            listen_port: 10600 + i as u16,
            max_peers: 10,
            connection_timeout: Duration::from_secs(5),
            discovery_interval: Duration::from_secs(1),
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10600)]
            },
        };
        
        let network = NetworkManager::new(config).await.unwrap();
        networks.push(network);
    }
    
    // Start networks
    for network in networks.iter_mut() {
        network.start().await.unwrap();
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Send many messages to test connection reuse
    for round in 0..5 {
        for i in 0..50 {
            let message = P2PMessage::new(
                MessageId::generate(),
                networks[0].local_peer_id(),
                format!("Round {} Message {}", round, i).into_bytes(),
            );
            
            networks[0].send_message(
                networks[1].local_peer_id(),
                message
            ).await.unwrap();
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Verify connection pool efficiency
    let connection_stats = networks[0].connection_pool_stats().await;
    assert!(
        connection_stats.reuse_count > 0,
        "Should have connection reuse"
    );
    assert!(
        connection_stats.active_connections <= 2,
        "Should maintain efficient connection count"
    );
    
    info!(
        "Connection pool stats - Active: {}, Reused: {}, Created: {}",
        connection_stats.active_connections,
        connection_stats.reuse_count,
        connection_stats.total_created
    );
    
    // Stop networks
    for network in networks.iter_mut() {
        network.stop().await.unwrap();
    }
}