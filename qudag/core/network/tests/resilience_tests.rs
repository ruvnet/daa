use qudag_network::{
    ConnectionManager, ConnectionStatus, LatencyMetrics, MessagePriority, NetworkError,
    NetworkMessage, PeerId, QueueMetrics, Router, RoutingStrategy, ThroughputMetrics,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Barrier;
use tokio::time::{sleep, timeout};

/// Test network resilience under high load
#[tokio::test]
async fn test_high_load_resilience() {
    let manager = ConnectionManager::new(100);
    let num_peers = 50;
    let messages_per_peer = 100;

    // Create and connect many peers
    let mut peers = Vec::new();
    for _ in 0..num_peers {
        let peer_id = PeerId::random();
        peers.push(peer_id);
        manager.connect(peer_id).await.unwrap();
        manager
            .update_status(peer_id, ConnectionStatus::Connected)
            .await;
    }

    let start_time = Instant::now();
    let barrier = Arc::new(Barrier::new(num_peers));
    let mut handles = Vec::new();

    // Spawn high-load message processing tasks
    for (i, peer_id) in peers.iter().enumerate() {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();
        let peer_id = *peer_id;

        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            // Send burst of messages
            for j in 0..messages_per_peer {
                let result = timeout(Duration::from_millis(100), async {
                    manager_clone
                        .update_metrics(1000.0 + j as f64, 10 + (j % 100) as u64)
                        .await;
                })
                .await;

                if result.is_err() {
                    eprintln!("Timeout updating metrics for peer {} message {}", i, j);
                    break;
                }

                // Small delay to prevent overwhelming
                if j % 10 == 0 {
                    sleep(Duration::from_micros(100)).await;
                }
            }

            (peer_id, i)
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut completed = 0;
    for handle in handles {
        match timeout(Duration::from_secs(30), handle).await {
            Ok(Ok(_)) => completed += 1,
            Ok(Err(e)) => eprintln!("Task panicked: {:?}", e),
            Err(_) => eprintln!("Task timed out"),
        }
    }

    let elapsed = start_time.elapsed();
    println!(
        "High load test: {} peers, {} completed in {:?}",
        num_peers, completed, elapsed
    );

    // Verify system remained stable
    assert!(
        completed >= num_peers / 2,
        "Too many tasks failed: {}/{}",
        completed,
        num_peers
    );

    // Check metrics are reasonable
    let metrics = manager.get_metrics().await;
    assert!(metrics.connections <= 100);
    assert!(metrics.messages_per_second >= 0.0);
}

/// Test connection failure recovery
#[tokio::test]
async fn test_connection_failure_recovery() {
    let manager = ConnectionManager::new(20);
    let mut peers = Vec::new();

    // Setup initial connections
    for _ in 0..10 {
        let peer_id = PeerId::random();
        peers.push(peer_id);
        manager.connect(peer_id).await.unwrap();
        manager
            .update_status(peer_id, ConnectionStatus::Connected)
            .await;
    }

    let initial_count = manager.connection_count().await;
    assert_eq!(initial_count, 10);

    // Simulate random connection failures
    let mut failed_peers = Vec::new();
    for (i, peer_id) in peers.iter().enumerate() {
        if i % 3 == 0 {
            // Fail every third connection
            manager
                .update_status(
                    *peer_id,
                    ConnectionStatus::Failed("Simulated failure".into()),
                )
                .await;
            manager.disconnect(peer_id).await;
            failed_peers.push(*peer_id);
        }
    }

    let after_failures = manager.connection_count().await;
    assert!(after_failures < initial_count);

    // Simulate recovery by reconnecting failed peers
    for peer_id in &failed_peers {
        let result = manager.connect(*peer_id).await;
        assert!(result.is_ok());
        manager
            .update_status(*peer_id, ConnectionStatus::Connected)
            .await;
    }

    let after_recovery = manager.connection_count().await;
    assert!(after_recovery >= initial_count);

    // Verify all peers are in a valid state
    for peer_id in &peers {
        let status = manager.get_status(peer_id).await;
        assert!(status.is_some());

        match status.unwrap() {
            ConnectionStatus::Connected | ConnectionStatus::Connecting => {
                // These are valid states
            }
            other => panic!("Unexpected status after recovery: {:?}", other),
        }
    }
}

/// Test routing resilience with peer churn
#[tokio::test]
async fn test_routing_peer_churn() {
    let router = Router::new();
    let stable_peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
    let churning_peers: Vec<_> = (0..10).map(|_| PeerId::random()).collect();

    // Add stable peers
    for peer in &stable_peers {
        router.add_peer(*peer).await;
    }

    let mut successful_routes = 0;
    let total_attempts = 100;

    for i in 0..total_attempts {
        // Add/remove churning peers randomly
        if i % 2 == 0 && i / 2 < churning_peers.len() {
            router.add_peer(churning_peers[i / 2]).await;
        }

        // Create test message
        let msg = NetworkMessage {
            id: format!("churn_test_{}", i),
            source: stable_peers[0].to_bytes().to_vec(),
            destination: stable_peers[4].to_bytes().to_vec(),
            payload: vec![0; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Attempt routing
        let result = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await;

        match result {
            Ok(route) => {
                successful_routes += 1;
                // Verify route doesn't include source/destination
                assert!(!route.contains(&stable_peers[0]));
                assert!(!route.contains(&stable_peers[4]));
            }
            Err(NetworkError::RoutingError(_)) => {
                // Expected when insufficient peers
            }
            Err(e) => panic!("Unexpected routing error: {:?}", e),
        }

        // Small delay between operations
        sleep(Duration::from_millis(1)).await;
    }

    let success_rate = successful_routes as f64 / total_attempts as f64;
    println!(
        "Routing success rate with peer churn: {:.2}%",
        success_rate * 100.0
    );

    // Should maintain reasonable success rate despite churn
    assert!(
        success_rate >= 0.5,
        "Success rate too low: {:.2}%",
        success_rate * 100.0
    );
}

/// Test message queue backpressure handling
#[tokio::test]
async fn test_backpressure_handling() {
    let manager = ConnectionManager::with_pool_timeout(10, Duration::from_secs(60));
    let peer_id = PeerId::random();

    manager.connect(peer_id).await.unwrap();
    manager
        .update_status(peer_id, ConnectionStatus::Connected)
        .await;

    let start_time = Instant::now();
    let mut update_times = Vec::new();

    // Rapidly update metrics to test backpressure
    for i in 0..1000 {
        let update_start = Instant::now();

        let result = timeout(
            Duration::from_millis(10),
            manager.update_metrics(i as f64, i as u64 % 100),
        )
        .await;

        let update_time = update_start.elapsed();
        update_times.push(update_time);

        if result.is_err() {
            // Timeout indicates backpressure is working
            break;
        }

        // Check if latency is increasing (indicating backpressure)
        if i > 100 && i % 100 == 0 {
            let recent_avg = update_times[i - 50..].iter().sum::<Duration>() / 50;
            let early_avg = update_times[10..60].iter().sum::<Duration>() / 50;

            if recent_avg > early_avg * 2 {
                println!(
                    "Backpressure detected at iteration {}: {:?} vs {:?}",
                    i, recent_avg, early_avg
                );
                break;
            }
        }
    }

    let total_time = start_time.elapsed();
    let avg_update_time = update_times.iter().sum::<Duration>() / update_times.len() as u32;

    println!(
        "Backpressure test: {} updates in {:?}, avg: {:?}",
        update_times.len(),
        total_time,
        avg_update_time
    );

    // Verify metrics are still accessible after backpressure
    let metrics = manager.get_metrics().await;
    assert!(metrics.messages_per_second >= 0.0);
}

/// Test graceful degradation under resource constraints
#[tokio::test]
async fn test_graceful_degradation() {
    let small_manager = ConnectionManager::new(3); // Very limited
    let mut peers = Vec::new();
    let mut connection_results = Vec::new();

    // Try to exceed capacity significantly
    for i in 0..15 {
        let peer_id = PeerId::random();
        peers.push(peer_id);

        let result = small_manager.connect(peer_id).await;
        connection_results.push(result.is_ok());

        if result.is_ok() {
            small_manager
                .update_status(peer_id, ConnectionStatus::Connected)
                .await;
        }

        // Check that we never exceed the limit
        let count = small_manager.connection_count().await;
        assert!(count <= 3, "Manager exceeded capacity: {} > 3", count);
    }

    // Verify that at least some connections succeeded
    let successful_connections = connection_results.iter().filter(|&&x| x).count();
    assert!(
        successful_connections >= 3,
        "Too few successful connections: {}",
        successful_connections
    );

    // Verify graceful handling of excess requests
    let final_count = small_manager.connection_count().await;
    assert!(final_count <= 3);

    // Test that existing connections still work
    let metrics = small_manager.get_metrics().await;
    assert!(metrics.connections <= 3);
}

/// Test network split-brain scenarios
#[tokio::test]
async fn test_split_brain_handling() {
    let manager1 = ConnectionManager::new(20);
    let manager2 = ConnectionManager::new(20);

    let shared_peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
    let partition1_peers: Vec<_> = (0..3).map(|_| PeerId::random()).collect();
    let partition2_peers: Vec<_> = (0..3).map(|_| PeerId::random()).collect();

    // Setup initial network state
    for peer in &shared_peers {
        manager1.connect(*peer).await.unwrap();
        manager2.connect(*peer).await.unwrap();
        manager1
            .update_status(*peer, ConnectionStatus::Connected)
            .await;
        manager2
            .update_status(*peer, ConnectionStatus::Connected)
            .await;
    }

    // Simulate network partition
    for peer in &partition1_peers {
        manager1.connect(*peer).await.unwrap();
        manager1
            .update_status(*peer, ConnectionStatus::Connected)
            .await;
        // Don't add to manager2 - simulates partition
    }

    for peer in &partition2_peers {
        manager2.connect(*peer).await.unwrap();
        manager2
            .update_status(*peer, ConnectionStatus::Connected)
            .await;
        // Don't add to manager1 - simulates partition
    }

    // Verify each partition maintains its state
    let count1 = manager1.connection_count().await;
    let count2 = manager2.connection_count().await;

    assert_eq!(count1, shared_peers.len() + partition1_peers.len());
    assert_eq!(count2, shared_peers.len() + partition2_peers.len());

    // Simulate partition healing - share information
    for peer in &partition2_peers {
        manager1.connect(*peer).await.unwrap();
        manager1
            .update_status(*peer, ConnectionStatus::Connected)
            .await;
    }

    for peer in &partition1_peers {
        manager2.connect(*peer).await.unwrap();
        manager2
            .update_status(*peer, ConnectionStatus::Connected)
            .await;
    }

    // Verify convergence
    let final_count1 = manager1.connection_count().await;
    let final_count2 = manager2.connection_count().await;
    let expected_total = shared_peers.len() + partition1_peers.len() + partition2_peers.len();

    assert_eq!(final_count1, expected_total);
    assert_eq!(final_count2, expected_total);
}

/// Test metrics consistency under concurrent access
#[tokio::test]
async fn test_metrics_consistency() {
    let manager = ConnectionManager::new(50);
    let num_updaters = 10;
    let updates_per_task = 100;

    let barrier = Arc::new(Barrier::new(num_updaters));
    let mut handles = Vec::new();

    // Spawn concurrent metrics updaters
    for i in 0..num_updaters {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            for j in 0..updates_per_task {
                let messages_per_sec = (i * updates_per_task + j) as f64;
                let latency_ms = (j % 200) as u64;

                manager_clone
                    .update_metrics(messages_per_sec, latency_ms)
                    .await;

                // Occasionally read metrics to test concurrent access
                if j % 10 == 0 {
                    let _metrics = manager_clone.get_metrics().await;
                    let _queue_metrics = manager_clone.get_queue_metrics();
                    let _latency_metrics = manager_clone.get_latency_metrics();
                    let _throughput_metrics = manager_clone.get_throughput_metrics();
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all updaters to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final metrics are consistent
    let final_metrics = manager.get_metrics().await;
    let queue_metrics = manager.get_queue_metrics();
    let latency_metrics = manager.get_latency_metrics();
    let throughput_metrics = manager.get_throughput_metrics();

    // All metrics should be valid (non-negative, finite)
    assert!(final_metrics.messages_per_second >= 0.0);
    assert!(final_metrics.connections >= 0);
    assert!(final_metrics.avg_latency >= Duration::ZERO);

    assert!(queue_metrics.utilization >= 0.0 && queue_metrics.utilization <= 1.0);
    assert!(latency_metrics.avg_latency >= Duration::ZERO);
    assert!(latency_metrics.peak_latency >= latency_metrics.avg_latency);

    assert!(throughput_metrics.messages_per_second >= 0.0);
    assert!(throughput_metrics.total_messages > 0); // Should have processed some messages

    println!(
        "Final metrics: {:.2} msg/s, {} connections, {:?} latency",
        final_metrics.messages_per_second, final_metrics.connections, final_metrics.avg_latency
    );
}

/// Test router state consistency during concurrent operations
#[tokio::test]
async fn test_router_concurrent_consistency() {
    let router = Router::new();
    let num_tasks = 20;
    let operations_per_task = 50;

    let barrier = Arc::new(Barrier::new(num_tasks));
    let mut handles = Vec::new();

    // Spawn concurrent router operations
    for i in 0..num_tasks {
        let router_clone = router.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut local_peers = Vec::new();

            for j in 0..operations_per_task {
                match j % 4 {
                    0 => {
                        // Add peer
                        let peer_id = PeerId::random();
                        local_peers.push(peer_id);
                        router_clone.add_peer(peer_id).await;
                    }
                    1 => {
                        // Route message
                        if !local_peers.is_empty() {
                            let msg = NetworkMessage {
                                id: format!("concurrent_{}_{}", i, j),
                                source: vec![1; 32],
                                destination: vec![2; 32],
                                payload: vec![0; 100],
                                priority: MessagePriority::Normal,
                                ttl: Duration::from_secs(60),
                            };

                            let _ = router_clone
                                .route(&msg, RoutingStrategy::Anonymous { hops: 2 })
                                .await;
                        }
                    }
                    2 => {
                        // Get hop info
                        if !local_peers.is_empty() {
                            let peer = local_peers[j % local_peers.len()];
                            let _ = router_clone.get_hop_info(&peer).await;
                        }
                    }
                    _ => {
                        // Route with different strategy
                        let msg = NetworkMessage {
                            id: format!("flood_{}_{}", i, j),
                            source: vec![3; 32],
                            destination: vec![4; 32],
                            payload: vec![0; 50],
                            priority: MessagePriority::Low,
                            ttl: Duration::from_secs(30),
                        };

                        let _ = router_clone.route(&msg, RoutingStrategy::Flood).await;
                    }
                }
            }

            local_peers.len()
        });
        handles.push(handle);
    }

    // Wait for all tasks and collect results
    let mut total_peers_added = 0;
    for handle in handles {
        let peers_added = handle.await.unwrap();
        total_peers_added += peers_added;
    }

    println!(
        "Concurrent router test: {} peers added across {} tasks",
        total_peers_added, num_tasks
    );

    // Verify router is still functional
    let test_msg = NetworkMessage {
        id: "final_test".into(),
        source: vec![1; 32],
        destination: vec![2; 32],
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let result = router.route(&test_msg, RoutingStrategy::Flood).await;
    assert!(result.is_ok());
}
