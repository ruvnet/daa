//! Integration test for complete onion routing system

use qudag_network::onion::{
    CircuitManager, DirectoryClient, MLKEMOnionRouter, MetadataProtector, MixMessage,
    MixMessageType, MixNode, TrafficAnalysisResistance,
};
use qudag_network::router::Router;
use qudag_network::types::{MessagePriority, NetworkMessage, RoutingStrategy};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};
use tracing::{debug, info};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_complete_onion_routing_system() {
    // Initialize logging for test visibility
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    info!("Starting complete onion routing system test");

    // Phase 1: Setup infrastructure
    let onion_router = Arc::new(Mutex::new(MLKEMOnionRouter::new().await.unwrap()));
    let circuit_manager = Arc::new(Mutex::new(CircuitManager::new()));
    let directory_client = Arc::new(DirectoryClient::new());
    let router = Arc::new(Router::new().await.unwrap());

    info!("Infrastructure initialized");

    // Phase 2: Build multiple circuits
    let mut circuit_ids = Vec::new();
    for i in 0..3 {
        let mut cm = circuit_manager.lock().await;
        match cm.build_circuit(3 + i, &directory_client).await {
            Ok(circuit_id) => {
                cm.activate_circuit(circuit_id).unwrap();
                circuit_ids.push(circuit_id);
                info!(
                    "Built and activated circuit {} with {} hops",
                    circuit_id,
                    3 + i
                );
            }
            Err(e) => {
                panic!("Failed to build circuit: {:?}", e);
            }
        }
    }

    // Phase 3: Test message routing through circuits
    let test_messages = vec![
        ("High priority secret", MessagePriority::High),
        ("Normal message", MessagePriority::Normal),
        ("Low priority bulk data", MessagePriority::Low),
    ];

    for (content, priority) in test_messages {
        let message = NetworkMessage {
            id: format!("test-{}", uuid::Uuid::new_v4()),
            source: vec![0u8; 32],
            destination: vec![255u8; 32],
            payload: content.as_bytes().to_vec(),
            priority,
            ttl: Duration::from_secs(60),
        };

        match router
            .route(&message, RoutingStrategy::Anonymous { hops: 4 })
            .await
        {
            Ok(route) => {
                info!("Routed '{}' through {} hops", content, route.len());

                // Update circuit metrics
                let mut cm = circuit_manager.lock().await;
                if let Some(circuit) = cm.get_active_circuit() {
                    cm.update_circuit_metrics(circuit.id, message.payload.len() as u64, true);
                }
            }
            Err(e) => {
                panic!("Failed to route message: {:?}", e);
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    // Phase 4: Test mix network functionality
    info!("Testing mix network batching and traffic shaping");

    let mut mix_node = MixNode::new(vec![1, 2, 3, 4]);
    let mix_messages = 150; // Will trigger batch processing

    for i in 0..mix_messages {
        let msg = MixMessage {
            content: format!("Mix message {}", i).into_bytes(),
            priority: (i % 3) as u8,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: if i % 10 == 0 {
                MixMessageType::Dummy
            } else {
                MixMessageType::Real
            },
            normalized_size: 0,
        };

        mix_node.add_message(msg).await.unwrap();
    }

    // Force batch flush
    let batch = mix_node.flush_batch().await.unwrap();
    info!("Flushed batch with {} messages", batch.len());

    let stats = mix_node.get_stats();
    info!(
        "Mix node stats - Buffer: {}, Dummy ratio: {:.2}",
        stats.buffer_size, stats.dummy_ratio
    );

    // Phase 5: Test traffic analysis resistance
    info!("Testing traffic analysis resistance");

    let tar = TrafficAnalysisResistance::new();
    let mut messages = Vec::new();

    for i in 0..20 {
        messages.push(MixMessage {
            content: vec![i as u8; 50 + i * 50], // Variable sizes
            priority: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: MixMessageType::Real,
            normalized_size: 0,
        });
    }

    tar.apply_resistance(&mut messages).await.unwrap();

    // Verify all messages have standard sizes
    let unique_sizes: std::collections::HashSet<_> =
        messages.iter().map(|m| m.normalized_size).collect();
    info!("Normalized to {} unique message sizes", unique_sizes.len());
    assert!(unique_sizes.len() <= 5); // Should be limited standard sizes

    // Phase 6: Test metadata protection
    info!("Testing metadata protection");

    let protector = MetadataProtector::new();
    let sensitive_data = b"User location: 37.7749N, 122.4194W";
    let protected = protector.protect_metadata(sensitive_data).unwrap();

    info!(
        "Protected metadata increased size from {} to {} bytes",
        sensitive_data.len(),
        protected.normalized_size
    );
    assert!(protected.random_headers.len() >= 2);
    assert!(!protected.anonymous_ids.is_empty());

    // Phase 7: Test circuit rotation
    info!("Testing circuit rotation");

    let rotation_task = tokio::spawn({
        let cm = circuit_manager.clone();
        let dc = directory_client.clone();
        async move {
            let mut rotation_count = 0;
            let mut check_interval = interval(Duration::from_secs(1));

            for _ in 0..5 {
                check_interval.tick().await;

                let mut circuit_mgr = cm.lock().await;
                if circuit_mgr.needs_rotation() {
                    rotation_count += 1;

                    // Build replacement circuit
                    if let Ok(new_id) = circuit_mgr.build_circuit(4, &dc).await {
                        let _ = circuit_mgr.activate_circuit(new_id);
                        debug!("Rotated to new circuit {}", new_id);
                    }

                    circuit_mgr.cleanup_inactive_circuits();
                }
            }

            rotation_count
        }
    });

    // Phase 8: Concurrent message processing
    info!("Testing concurrent message processing");

    let mut handles = Vec::new();
    for thread_id in 0..4 {
        let router_clone = router.clone();
        let cm_clone = circuit_manager.clone();

        let handle = tokio::spawn(async move {
            for i in 0..25 {
                let message = NetworkMessage {
                    id: format!("concurrent-{}-{}", thread_id, i),
                    source: vec![thread_id as u8; 32],
                    destination: vec![255u8; 32],
                    payload: format!("Thread {} message {}", thread_id, i).into_bytes(),
                    priority: MessagePriority::Normal,
                    ttl: Duration::from_secs(30),
                };

                if let Ok(route) = router_clone
                    .route(&message, RoutingStrategy::Anonymous { hops: 3 })
                    .await
                {
                    debug!(
                        "Thread {} routed message {} through {} hops",
                        thread_id,
                        i,
                        route.len()
                    );

                    // Update metrics
                    let mut cm = cm_clone.lock().await;
                    if let Some(circuit) = cm.get_active_circuit() {
                        cm.update_circuit_metrics(circuit.id, message.payload.len() as u64, true);
                    }
                }

                sleep(Duration::from_millis(10)).await;
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Phase 9: Final statistics
    let final_stats = circuit_manager.lock().await.get_stats();
    info!("Final circuit statistics:");
    info!("  Total circuits: {}", final_stats.total_circuits);
    info!("  Active circuits: {}", final_stats.active_circuits);
    info!("  Total bandwidth: {} bytes", final_stats.total_bandwidth);
    info!("  Average quality: {:.2}", final_stats.average_quality);

    // Verify system health
    assert!(final_stats.active_circuits > 0);
    assert!(final_stats.total_bandwidth > 0);
    assert!(final_stats.average_quality > 0.5);

    // Cleanup
    rotation_task.abort();
    info!("Onion routing system test completed successfully");
}

#[tokio::test]
async fn test_circuit_failure_recovery() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let mut circuit_manager = CircuitManager::new();
    let directory_client = DirectoryClient::new();

    // Build initial circuit
    let circuit_id = circuit_manager
        .build_circuit(3, &directory_client)
        .await
        .unwrap();
    circuit_manager.activate_circuit(circuit_id).unwrap();

    // Simulate failures
    for _ in 0..10 {
        circuit_manager.update_circuit_metrics(circuit_id, 0, false);
    }

    // Check quality degradation
    let circuit = circuit_manager.get_active_circuit().unwrap();
    assert!(circuit.quality_score < 0.5);

    // Build replacement circuit
    let new_circuit_id = circuit_manager
        .build_circuit(3, &directory_client)
        .await
        .unwrap();
    circuit_manager.activate_circuit(new_circuit_id).unwrap();

    // Teardown failed circuit
    circuit_manager.teardown_circuit(circuit_id).await.unwrap();

    let stats = circuit_manager.get_stats();
    assert_eq!(stats.active_circuits, 1);
}

#[tokio::test]
async fn test_load_balancing() {
    let directory_client = DirectoryClient::new();

    // Get load balancing weights
    let weights = directory_client.get_load_balancing_weights().await;

    // Simulate weighted selection
    let mut selections = std::collections::HashMap::new();
    let trials = 10000;

    for _ in 0..trials {
        let mut target = rand::random::<f64>();
        for (node_id, weight) in &weights {
            target -= weight;
            if target <= 0.0 {
                *selections.entry(node_id.clone()).or_insert(0) += 1;
                break;
            }
        }
    }

    // Verify distribution roughly matches weights
    for (node_id, count) in selections {
        let expected_ratio = weights.get(&node_id).unwrap();
        let actual_ratio = count as f64 / trials as f64;
        let deviation = (actual_ratio - expected_ratio).abs();

        // Allow 5% deviation
        assert!(
            deviation < 0.05,
            "Node selection deviation too high: expected {:.2}, got {:.2}",
            expected_ratio,
            actual_ratio
        );
    }
}
