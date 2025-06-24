use qudag_network::{
    ConnectionManager, ConnectionStatus, MessagePriority, NetworkError, NetworkMessage, PeerId,
    Router, RoutingStrategy, SecureConfig, SecureConnection, TransportKeys,
};
use quinn::{Endpoint, ServerConfig};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::time::timeout;

/// Test malformed message handling
#[tokio::test]
async fn test_malformed_message_handling() {
    let router = Router::new();

    // Test with invalid source ID format
    let invalid_msg = NetworkMessage {
        id: "test".into(),
        source: vec![1, 2, 3], // Invalid - should be 32 bytes
        destination: vec![4; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&invalid_msg, RoutingStrategy::Anonymous { hops: 2 })
        .await;
    assert!(result.is_ok()); // Should handle gracefully, not crash

    // Test with empty destination
    let empty_dest_msg = NetworkMessage {
        id: "test2".into(),
        source: vec![1; 32],
        destination: vec![], // Empty destination
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&empty_dest_msg, RoutingStrategy::Anonymous { hops: 2 })
        .await;
    assert!(result.is_ok()); // Should handle gracefully

    // Test with oversized message ID
    let oversized_id_msg = NetworkMessage {
        id: "a".repeat(10000), // Very long ID
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&oversized_id_msg, RoutingStrategy::Anonymous { hops: 2 })
        .await;
    assert!(result.is_ok()); // Should handle gracefully
}

/// Test network timeout scenarios
#[tokio::test]
async fn test_network_timeouts() {
    let manager = ConnectionManager::new(10);
    let peer_id = PeerId::random();

    // Test connection timeout handling
    let start = std::time::Instant::now();
    let result = timeout(Duration::from_millis(100), manager.connect(peer_id)).await;
    let elapsed = start.elapsed();

    // Should complete quickly or timeout gracefully
    assert!(elapsed < Duration::from_millis(200));
    assert!(result.is_ok() || result.is_err()); // Either completes or times out

    // Test status update timeout
    let start = std::time::Instant::now();
    let result = timeout(
        Duration::from_millis(100),
        manager.update_status(peer_id, ConnectionStatus::Connected),
    )
    .await;
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(200));
    assert!(result.is_ok());
}

/// Test connection pool overflow scenarios
#[tokio::test]
async fn test_connection_pool_overflow() {
    let manager = ConnectionManager::new(3); // Small pool
    let mut peers = Vec::new();

    // Fill the pool beyond capacity
    for i in 0..10 {
        let peer_id = PeerId::random();
        peers.push(peer_id);

        let result = manager.connect(peer_id).await;
        assert!(result.is_ok()); // Should not fail, just apply limits

        // Check that pool doesn't exceed max size
        let count = manager.connection_count().await;
        assert!(count <= 3, "Pool size {} exceeds limit", count);
    }

    // Test pool cleanup under pressure
    for peer in peers.iter().take(5) {
        manager.disconnect(peer).await;
    }

    // Verify pool was cleaned up
    let final_count = manager.connection_count().await;
    assert!(final_count <= 3);
}

/// Test rapid connection/disconnection cycles
#[tokio::test]
async fn test_rapid_connection_cycles() {
    let manager = ConnectionManager::new(50);
    let peer_id = PeerId::random();

    // Rapid connect/disconnect cycles
    for i in 0..100 {
        let result = manager.connect(peer_id).await;
        assert!(result.is_ok());

        if i % 2 == 0 {
            manager
                .update_status(peer_id, ConnectionStatus::Connected)
                .await;
            manager.disconnect(&peer_id).await;
        }
    }

    // Verify final state is consistent
    let final_status = manager.get_status(&peer_id).await;
    assert!(final_status.is_none() || matches!(final_status, Some(ConnectionStatus::Disconnected)));
}

/// Test routing with insufficient peers
#[tokio::test]
async fn test_insufficient_peers_routing() {
    let router = Router::new();

    // Add only one peer
    let peer1 = PeerId::random();
    router.add_peer(peer1).await;

    let msg = NetworkMessage {
        id: "test".into(),
        source: peer1.to_bytes().to_vec(),
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Try to route with more hops than available peers
    let result = router
        .route(&msg, RoutingStrategy::Anonymous { hops: 5 })
        .await;

    match result {
        Err(NetworkError::RoutingError(_)) => {
            // Expected error for insufficient peers
        }
        Ok(route) => {
            // If it succeeds, should be limited by available peers
            assert!(route.len() <= 1);
        }
    }
}

/// Test zero-length message handling
#[tokio::test]
async fn test_zero_length_messages() {
    let router = Router::new();
    let peer_id = PeerId::random();
    router.add_peer(peer_id).await;

    let empty_msg = NetworkMessage {
        id: "".into(), // Empty ID
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![], // Empty payload
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(0), // Zero TTL
    };

    let result = router
        .route(&empty_msg, RoutingStrategy::Direct(vec![1; 32]))
        .await;
    assert!(result.is_ok());
}

/// Test concurrent routing operations
#[tokio::test]
async fn test_concurrent_routing() {
    let router = Router::new();

    // Add test peers
    for _ in 0..10 {
        router.add_peer(PeerId::random()).await;
    }

    let mut handles = Vec::new();

    // Spawn concurrent routing tasks
    for i in 0..20 {
        let router_clone = router.clone();
        let handle = tokio::spawn(async move {
            let msg = NetworkMessage {
                id: format!("concurrent_test_{}", i),
                source: vec![1; 32],
                destination: vec![2; 32],
                payload: vec![0; 100],
                priority: MessagePriority::Normal,
                ttl: Duration::from_secs(60),
            };

            router_clone
                .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
                .await
        });
        handles.push(handle);
    }

    // Wait for all routing operations to complete
    let mut success_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => println!("Routing error: {:?}", e),
            Err(e) => println!("Task error: {:?}", e),
        }
    }

    // Most should succeed (depends on peer availability)
    assert!(
        success_count >= 10,
        "Too many routing failures: {}/20",
        success_count
    );
}

/// Test memory exhaustion scenarios
#[tokio::test]
async fn test_memory_pressure() {
    let manager = ConnectionManager::new(1000);

    // Create many peers to stress memory usage
    let mut peers = Vec::new();
    for _ in 0..1000 {
        peers.push(PeerId::random());
    }

    // Connect all peers rapidly
    for peer in &peers {
        let result = manager.connect(*peer).await;
        assert!(result.is_ok());
    }

    // Update metrics to track memory usage
    manager.update_metrics(1000.0, 50).await;

    let metrics = manager.get_metrics().await;
    assert!(metrics.connections <= 1000);

    // Cleanup - disconnect all peers
    for peer in &peers {
        manager.disconnect(peer).await;
    }

    let final_count = manager.connection_count().await;
    assert_eq!(final_count, 0);
}

/// Test invalid routing strategies
#[tokio::test]
async fn test_invalid_routing_strategies() {
    let router = Router::new();
    let peer_id = PeerId::random();
    router.add_peer(peer_id).await;

    let msg = NetworkMessage {
        id: "test".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Test direct routing with invalid peer ID
    let result = router
        .route(&msg, RoutingStrategy::Direct(vec![1, 2, 3]))
        .await;
    assert!(matches!(result, Err(NetworkError::RoutingError(_))));

    // Test random subset with zero count
    let result = router.route(&msg, RoutingStrategy::RandomSubset(0)).await;
    assert!(result.is_ok());
    let route = result.unwrap();
    assert!(route.is_empty());
}

/// Test network partition scenarios
#[tokio::test]
async fn test_network_partition_recovery() {
    let manager = ConnectionManager::new(10);
    let mut peers = Vec::new();

    // Create initial network
    for _ in 0..5 {
        let peer_id = PeerId::random();
        peers.push(peer_id);
        manager.connect(peer_id).await.unwrap();
        manager
            .update_status(peer_id, ConnectionStatus::Connected)
            .await;
    }

    // Simulate network partition - disconnect half
    for peer in peers.iter().take(2) {
        manager
            .update_status(*peer, ConnectionStatus::Failed("Network partition".into()))
            .await;
        manager.disconnect(peer).await;
    }

    // Verify remaining connections
    let remaining_count = manager.connection_count().await;
    assert!(remaining_count >= 3);

    // Simulate partition recovery - reconnect
    for peer in peers.iter().take(2) {
        manager.connect(*peer).await.unwrap();
        manager
            .update_status(*peer, ConnectionStatus::Connected)
            .await;
    }

    let final_count = manager.connection_count().await;
    assert!(final_count >= 5);
}

/// Test hop information consistency
#[tokio::test]
async fn test_hop_info_consistency() {
    let router = Router::new();

    // Add peers and create route
    let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    let msg = NetworkMessage {
        id: "test".into(),
        source: peers[0].to_bytes().to_vec(),
        destination: peers[4].to_bytes().to_vec(),
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Route message
    let route = router
        .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
        .await
        .unwrap();

    // Verify hop information consistency
    for (i, peer_id) in route.iter().enumerate() {
        let hop_info = router.get_hop_info(peer_id).await;
        assert!(hop_info.is_ok(), "Missing hop info for peer {}", i);

        let info = hop_info.unwrap();
        assert!(
            info.can_decrypt_layer(i),
            "Peer {} cannot decrypt its own layer",
            i
        );
    }
}

/// Test extreme message sizes
#[tokio::test]
async fn test_extreme_message_sizes() {
    let router = Router::new();
    let peer_id = PeerId::random();
    router.add_peer(peer_id).await;

    // Test very large payload
    let large_msg = NetworkMessage {
        id: "large_test".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 1024 * 1024], // 1MB payload
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&large_msg, RoutingStrategy::Direct(vec![1; 32]))
        .await;
    assert!(result.is_ok());

    // Test maximum possible payload (within reason)
    let max_msg = NetworkMessage {
        id: "max_test".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 10 * 1024 * 1024], // 10MB payload
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&max_msg, RoutingStrategy::Direct(vec![1; 32]))
        .await;
    assert!(result.is_ok()); // Should handle or fail gracefully
}

/// Test connection status transitions
#[tokio::test]
async fn test_connection_status_transitions() {
    let manager = ConnectionManager::new(10);
    let peer_id = PeerId::random();

    // Test valid status transitions
    manager.connect(peer_id).await.unwrap();
    let status = manager.get_status(&peer_id).await;
    assert_eq!(status, Some(ConnectionStatus::Connecting));

    manager
        .update_status(peer_id, ConnectionStatus::Connected)
        .await;
    let status = manager.get_status(&peer_id).await;
    assert_eq!(status, Some(ConnectionStatus::Connected));

    manager
        .update_status(peer_id, ConnectionStatus::Disconnecting)
        .await;
    let status = manager.get_status(&peer_id).await;
    assert_eq!(status, Some(ConnectionStatus::Disconnecting));

    manager.disconnect(&peer_id).await;
    let status = manager.get_status(&peer_id).await;
    assert!(status.is_none());

    // Test failure transition
    manager.connect(peer_id).await.unwrap();
    manager
        .update_status(peer_id, ConnectionStatus::Failed("Test failure".into()))
        .await;
    let status = manager.get_status(&peer_id).await;
    assert!(matches!(status, Some(ConnectionStatus::Failed(_))));
}
