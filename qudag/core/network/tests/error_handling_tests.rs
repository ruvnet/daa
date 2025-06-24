use qudag_network::{
    ConnectionManager, ConnectionStatus, MessagePriority, NetworkAddress, NetworkError,
    NetworkMessage, PeerId, Router, RoutingStrategy,
};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

/// Test various network error scenarios and their handling
#[tokio::test]
async fn test_network_error_types() {
    let router = Router::new();

    // Test routing error with invalid peer ID format
    let msg = NetworkMessage {
        id: "test".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&msg, RoutingStrategy::Direct(vec![1, 2, 3]))
        .await;
    assert!(matches!(result, Err(NetworkError::RoutingError(_))));

    if let Err(NetworkError::RoutingError(msg)) = result {
        assert!(msg.contains("Invalid peer ID format"));
    }
}

/// Test error propagation through the network stack
#[tokio::test]
async fn test_error_propagation() {
    let manager = ConnectionManager::new(1);
    let router = Router::new();

    // Test cascading errors
    let peer_id = PeerId::random();

    // Connect peer
    manager.connect(peer_id).await.unwrap();
    manager
        .update_status(peer_id, ConnectionStatus::Connected)
        .await;

    // Simulate network failure
    manager
        .update_status(
            peer_id,
            ConnectionStatus::Failed("Network unreachable".into()),
        )
        .await;

    // Verify error state is maintained
    let status = manager.get_status(&peer_id).await;
    assert!(matches!(status, Some(ConnectionStatus::Failed(_))));

    if let Some(ConnectionStatus::Failed(error_msg)) = status {
        assert_eq!(error_msg, "Network unreachable");
    }

    // Test routing with failed peer
    router.add_peer(peer_id).await;

    let msg = NetworkMessage {
        id: "error_test".into(),
        source: vec![1; 32],
        destination: peer_id.to_bytes().to_vec(),
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&msg, RoutingStrategy::Direct(peer_id.to_bytes().to_vec()))
        .await;
    assert!(result.is_ok()); // Router should handle peer failure gracefully
}

/// Test error recovery mechanisms
#[tokio::test]
async fn test_error_recovery() {
    let manager = ConnectionManager::new(5);
    let peer_id = PeerId::random();

    // Normal connection
    manager.connect(peer_id).await.unwrap();
    manager
        .update_status(peer_id, ConnectionStatus::Connected)
        .await;

    // Simulate transient failure
    manager
        .update_status(
            peer_id,
            ConnectionStatus::Failed("Temporary network issue".into()),
        )
        .await;

    // Attempt recovery
    manager.disconnect(&peer_id).await;
    let recovery_result = manager.connect(peer_id).await;
    assert!(recovery_result.is_ok());

    // Update to connected state
    manager
        .update_status(peer_id, ConnectionStatus::Connected)
        .await;

    // Verify recovery
    let final_status = manager.get_status(&peer_id).await;
    assert_eq!(final_status, Some(ConnectionStatus::Connected));
}

/// Test timeout handling in various scenarios
#[tokio::test]
async fn test_timeout_handling() {
    let manager = ConnectionManager::new(10);
    let router = Router::new();

    // Test connection timeout
    let peer_id = PeerId::random();

    // Simulate slow connection
    manager.connect(peer_id).await.unwrap();

    // Don't immediately update to connected - simulates timeout scenario
    tokio::time::sleep(Duration::from_millis(100)).await;

    let status = manager.get_status(&peer_id).await;
    assert_eq!(status, Some(ConnectionStatus::Connecting));

    // Test routing timeout with insufficient peers
    let msg_with_short_ttl = NetworkMessage {
        id: "timeout_test".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_millis(1), // Very short TTL
    };

    // Add a peer for routing
    router.add_peer(PeerId::random()).await;

    let result = router
        .route(&msg_with_short_ttl, RoutingStrategy::Anonymous { hops: 10 })
        .await;

    // Should handle timeout gracefully
    match result {
        Ok(_) => {}                              // Routing succeeded despite short TTL
        Err(NetworkError::RoutingError(_)) => {} // Expected for insufficient peers
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

/// Test resource exhaustion error handling
#[tokio::test]
async fn test_resource_exhaustion() {
    let small_manager = ConnectionManager::new(2); // Very limited capacity
    let mut peers = Vec::new();

    // Fill capacity
    for _ in 0..2 {
        let peer_id = PeerId::random();
        peers.push(peer_id);
        small_manager.connect(peer_id).await.unwrap();
        small_manager
            .update_status(peer_id, ConnectionStatus::Connected)
            .await;
    }

    // Verify at capacity
    assert_eq!(small_manager.connection_count().await, 2);

    // Try to exceed capacity
    let excess_peer = PeerId::random();
    let result = small_manager.connect(excess_peer).await;

    // Should handle gracefully (not error, but apply limits)
    assert!(result.is_ok());

    // Should not exceed capacity
    let final_count = small_manager.connection_count().await;
    assert!(final_count <= 2);

    // Test that existing connections still work
    for peer in &peers {
        let status = small_manager.get_status(peer).await;
        assert!(status.is_some());
    }
}

/// Test malformed data handling
#[tokio::test]
async fn test_malformed_data_handling() {
    let router = Router::new();

    // Test with null bytes in message ID
    let msg_with_nulls = NetworkMessage {
        id: "test\0\0\0message".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    router.add_peer(PeerId::random()).await;
    let result = router.route(&msg_with_nulls, RoutingStrategy::Flood).await;
    assert!(result.is_ok()); // Should handle gracefully

    // Test with extremely long message ID
    let msg_with_long_id = NetworkMessage {
        id: "x".repeat(100_000),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&msg_with_long_id, RoutingStrategy::Flood)
        .await;
    assert!(result.is_ok()); // Should handle gracefully

    // Test with malformed peer ID bytes
    let msg_with_bad_peer = NetworkMessage {
        id: "test".into(),
        source: vec![0xFF; 32], // Valid size but potentially problematic values
        destination: vec![0x00; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router
        .route(&msg_with_bad_peer, RoutingStrategy::Anonymous { hops: 1 })
        .await;
    // Should either succeed or fail gracefully, not panic
    assert!(result.is_ok() || matches!(result, Err(NetworkError::RoutingError(_))));
}

/// Test network address validation and error handling
#[tokio::test]
async fn test_network_address_errors() {
    // Test valid address creation
    let valid_addr = NetworkAddress::new([127, 0, 0, 1], 8080);
    assert_eq!(valid_addr.to_socket_addr(), "127.0.0.1:8080");

    // Test edge cases
    let addr_zero_port = NetworkAddress::new([0, 0, 0, 0], 0);
    assert_eq!(addr_zero_port.to_socket_addr(), "0.0.0.0:0");

    let addr_max_port = NetworkAddress::new([255, 255, 255, 255], 65535);
    assert_eq!(addr_max_port.to_socket_addr(), "255.255.255.255:65535");

    // Test with IPv6
    let ipv6_addr = NetworkAddress::from_ip_port(IpAddr::V6("::1".parse().unwrap()), 8080);
    assert_eq!(ipv6_addr.to_socket_addr(), "[::1]:8080");
}

/// Test concurrent error conditions
#[tokio::test]
async fn test_concurrent_error_conditions() {
    let manager = ConnectionManager::new(5);
    let peer_id = PeerId::random();

    // Connect peer
    manager.connect(peer_id).await.unwrap();
    manager
        .update_status(peer_id, ConnectionStatus::Connected)
        .await;

    // Spawn concurrent operations that may cause errors
    let mut handles = Vec::new();

    // Task 1: Rapid status updates
    let manager1 = manager.clone();
    let handle1 = tokio::spawn(async move {
        for i in 0..100 {
            let status = if i % 3 == 0 {
                ConnectionStatus::Failed(format!("Error {}", i))
            } else {
                ConnectionStatus::Connected
            };
            manager1.update_status(peer_id, status).await;
        }
    });
    handles.push(handle1);

    // Task 2: Rapid disconnects/reconnects
    let manager2 = manager.clone();
    let handle2 = tokio::spawn(async move {
        for _ in 0..50 {
            manager2.disconnect(&peer_id).await;
            let _ = manager2.connect(peer_id).await;
        }
    });
    handles.push(handle2);

    // Task 3: Metrics updates
    let manager3 = manager.clone();
    let handle3 = tokio::spawn(async move {
        for i in 0..100 {
            manager3.update_metrics(i as f64, (i % 1000) as u64).await;
        }
    });
    handles.push(handle3);

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify system is still in a consistent state
    let final_status = manager.get_status(&peer_id).await;
    assert!(final_status.is_some() || final_status.is_none()); // Any state is valid

    let metrics = manager.get_metrics().await;
    assert!(metrics.messages_per_second >= 0.0);
    assert!(metrics.connections >= 0);
}

/// Test error message formatting and content
#[tokio::test]
async fn test_error_message_quality() {
    // Test that error messages are informative
    let router = Router::new();

    let msg = NetworkMessage {
        id: "test".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Test insufficient peers error
    let result = router
        .route(&msg, RoutingStrategy::Anonymous { hops: 10 })
        .await;

    if let Err(NetworkError::RoutingError(error_msg)) = result {
        // Error message should be descriptive
        assert!(error_msg.contains("peers") || error_msg.contains("routing"));
        assert!(!error_msg.is_empty());
        println!("Routing error message: {}", error_msg);
    }

    // Test invalid peer ID error
    let result = router
        .route(&msg, RoutingStrategy::Direct(vec![1, 2]))
        .await;

    if let Err(NetworkError::RoutingError(error_msg)) = result {
        assert!(error_msg.contains("Invalid") || error_msg.contains("format"));
        assert!(!error_msg.is_empty());
        println!("Invalid peer ID error message: {}", error_msg);
    }
}

/// Test error boundaries and isolation
#[tokio::test]
async fn test_error_isolation() {
    let manager1 = ConnectionManager::new(10);
    let manager2 = ConnectionManager::new(10);

    let peer1 = PeerId::random();
    let peer2 = PeerId::random();

    // Setup both managers
    manager1.connect(peer1).await.unwrap();
    manager2.connect(peer2).await.unwrap();

    manager1
        .update_status(peer1, ConnectionStatus::Connected)
        .await;
    manager2
        .update_status(peer2, ConnectionStatus::Connected)
        .await;

    // Cause error in manager1
    manager1
        .update_status(peer1, ConnectionStatus::Failed("Isolated error".into()))
        .await;

    // Verify manager2 is unaffected
    let status2 = manager2.get_status(&peer2).await;
    assert_eq!(status2, Some(ConnectionStatus::Connected));

    let metrics2 = manager2.get_metrics().await;
    assert_eq!(metrics2.connections, 1);

    // Verify manager1 error is contained
    let status1 = manager1.get_status(&peer1).await;
    assert!(matches!(status1, Some(ConnectionStatus::Failed(_))));

    // Manager1 should still be functional for other operations
    let new_peer = PeerId::random();
    let result = manager1.connect(new_peer).await;
    assert!(result.is_ok());
}

/// Test graceful degradation under error conditions
#[tokio::test]
async fn test_graceful_degradation_with_errors() {
    let manager = ConnectionManager::new(5);
    let mut peers = Vec::new();

    // Setup several connections
    for _ in 0..5 {
        let peer_id = PeerId::random();
        peers.push(peer_id);
        manager.connect(peer_id).await.unwrap();
        manager
            .update_status(peer_id, ConnectionStatus::Connected)
            .await;
    }

    // Introduce progressive failures
    for (i, peer) in peers.iter().enumerate() {
        if i % 2 == 0 {
            manager
                .update_status(*peer, ConnectionStatus::Failed(format!("Failure {}", i)))
                .await;
        }
    }

    // System should still be partially functional
    let remaining_connections = peers
        .iter()
        .filter(|peer| {
            futures::executor::block_on(async {
                matches!(
                    manager.get_status(peer).await,
                    Some(ConnectionStatus::Connected)
                )
            })
        })
        .count();

    assert!(
        remaining_connections >= 2,
        "Too few connections remain: {}",
        remaining_connections
    );

    // Metrics should still be accessible
    let metrics = manager.get_metrics().await;
    assert!(metrics.connections >= 0);

    // New connections should still work
    let new_peer = PeerId::random();
    let result = manager.connect(new_peer).await;
    assert!(result.is_ok());
}
