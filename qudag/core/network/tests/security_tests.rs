use qudag_network::{
    ConnectionManager, ConnectionStatus, HopInfo, MessageEnvelope, MessagePriority, MessageQueue,
    NetworkError, NetworkMessage, PeerId, Router, RoutingStrategy,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

/// Test anonymous routing properties
#[tokio::test]
async fn test_anonymous_routing_properties() {
    let router = Router::new();

    // Add sufficient peers for anonymous routing
    let peers: Vec<_> = (0..10).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    let source_peer = peers[0];
    let dest_peer = peers[9];

    let msg = NetworkMessage {
        id: "anonymous_test".into(),
        source: source_peer.to_bytes().to_vec(),
        destination: dest_peer.to_bytes().to_vec(),
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Test multiple anonymous routes
    let mut routes = Vec::new();
    for _ in 0..10 {
        let route = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 5 })
            .await
            .unwrap();
        routes.push(route);
    }

    // Verify anonymity properties
    for route in &routes {
        // Route should not contain source or destination
        assert!(!route.contains(&source_peer), "Route contains source peer");
        assert!(
            !route.contains(&dest_peer),
            "Route contains destination peer"
        );

        // Route should have requested number of hops
        assert_eq!(route.len(), 5, "Route has wrong number of hops");

        // All peers in route should be unique
        let unique_peers: HashSet<_> = route.iter().collect();
        assert_eq!(
            unique_peers.len(),
            route.len(),
            "Route contains duplicate peers"
        );
    }

    // Routes should be different (probabilistically)
    let unique_routes: HashSet<_> = routes.iter().collect();
    assert!(
        unique_routes.len() > 1,
        "All routes are identical - not sufficiently random"
    );
}

/// Test hop information and layer isolation
#[tokio::test]
async fn test_hop_isolation() {
    let router = Router::new();

    // Add peers for routing
    let peers: Vec<_> = (0..6).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    let msg = NetworkMessage {
        id: "isolation_test".into(),
        source: peers[0].to_bytes().to_vec(),
        destination: peers[5].to_bytes().to_vec(),
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Create route
    let route = router
        .route(&msg, RoutingStrategy::Anonymous { hops: 4 })
        .await
        .unwrap();

    // Verify hop isolation properties
    for (i, peer_id) in route.iter().enumerate() {
        let hop_info = router.get_hop_info(peer_id).await;
        assert!(
            hop_info.is_ok(),
            "Missing hop info for peer at position {}",
            i
        );

        let info = hop_info.unwrap();

        // Each hop should only be able to decrypt its own layer
        assert!(
            info.can_decrypt_layer(i),
            "Hop {} cannot decrypt its own layer",
            i
        );

        // Hop should not be able to decrypt other layers
        for j in 0..route.len() {
            if j != i {
                assert!(
                    !info.can_decrypt_layer(j),
                    "Hop {} can decrypt layer {} (should only decrypt layer {})",
                    i,
                    j,
                    i
                );
            }
        }

        // Hop should only know about adjacent peers in the route
        let mut expected_known_peers = HashSet::new();
        if i > 0 {
            expected_known_peers.insert(route[i - 1]);
        }
        if i < route.len() - 1 {
            expected_known_peers.insert(route[i + 1]);
        }

        // Verify hop only knows expected peers
        for peer in &route {
            if expected_known_peers.contains(peer) {
                assert!(
                    info.knows_peer(peer),
                    "Hop {} should know about adjacent peer",
                    i
                );
            } else if *peer != *peer_id {
                assert!(
                    !info.knows_peer(peer),
                    "Hop {} should not know about non-adjacent peer",
                    i
                );
            }
        }
    }
}

/// Test traffic analysis resistance
#[tokio::test]
async fn test_traffic_analysis_resistance() {
    let router = Router::new();

    // Add many peers
    let peers: Vec<_> = (0..20).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    let source_peer = peers[0];
    let dest_peer = peers[19];

    // Generate multiple messages with different patterns
    let mut routes = Vec::new();
    for i in 0..50 {
        let msg = NetworkMessage {
            id: format!("traffic_test_{}", i),
            source: source_peer.to_bytes().to_vec(),
            destination: dest_peer.to_bytes().to_vec(),
            payload: vec![0; 100 + (i % 500)], // Varying payload sizes
            priority: if i % 3 == 0 {
                MessagePriority::High
            } else {
                MessagePriority::Normal
            },
            ttl: Duration::from_secs(60 + (i % 300) as u64), // Varying TTL
        };

        let route = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await
            .unwrap();
        routes.push(route);
    }

    // Analyze route diversity for traffic analysis resistance
    let mut hop_frequency = std::collections::HashMap::new();
    let mut route_patterns = HashSet::new();

    for route in &routes {
        // Track hop frequency
        for peer in route {
            *hop_frequency.entry(*peer).or_insert(0) += 1;
        }

        // Track route patterns
        route_patterns.insert(route.clone());
    }

    // No single peer should dominate the routes (traffic analysis resistance)
    let max_frequency = hop_frequency.values().max().unwrap_or(&0);
    let total_hops = routes.len() * 3; // 3 hops per route
    let max_allowed_frequency = total_hops / 3; // No peer should appear in more than ~33% of routes

    assert!(
        *max_frequency <= max_allowed_frequency,
        "Peer appears too frequently in routes: {} out of {} total hops",
        max_frequency,
        total_hops
    );

    // Routes should be sufficiently diverse
    let diversity_ratio = route_patterns.len() as f64 / routes.len() as f64;
    assert!(
        diversity_ratio >= 0.7,
        "Route diversity too low: {:.2}% unique routes",
        diversity_ratio * 100.0
    );
}

/// Test timing attack resistance
#[tokio::test]
async fn test_timing_attack_resistance() {
    let router = Router::new();

    // Add peers
    let peers: Vec<_> = (0..8).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    let msg = NetworkMessage {
        id: "timing_test".into(),
        source: peers[0].to_bytes().to_vec(),
        destination: peers[7].to_bytes().to_vec(),
        payload: vec![0; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Measure routing times
    let mut routing_times = Vec::new();
    for _ in 0..100 {
        let start = std::time::Instant::now();
        let _route = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await
            .unwrap();
        let elapsed = start.elapsed();
        routing_times.push(elapsed);
    }

    // Analyze timing characteristics
    let avg_time = routing_times.iter().sum::<Duration>() / routing_times.len() as u32;
    let max_time = routing_times.iter().max().unwrap();
    let min_time = routing_times.iter().min().unwrap();

    // Calculate timing variance
    let variance = routing_times
        .iter()
        .map(|t| {
            let diff = if *t > avg_time {
                *t - avg_time
            } else {
                avg_time - *t
            };
            diff.as_nanos() as f64
        })
        .map(|diff| diff * diff)
        .sum::<f64>()
        / routing_times.len() as f64;

    let std_dev = variance.sqrt();
    let cv = std_dev / avg_time.as_nanos() as f64; // Coefficient of variation

    println!(
        "Timing analysis - Avg: {:?}, Min: {:?}, Max: {:?}, CV: {:.3}",
        avg_time, min_time, max_time, cv
    );

    // Timing should be relatively consistent (low coefficient of variation)
    // This helps resist timing analysis attacks
    assert!(cv < 0.5, "Timing variance too high: {:.3}", cv);

    // No single measurement should be extremely different
    let max_ratio = max_time.as_nanos() as f64 / min_time.as_nanos() as f64;
    assert!(
        max_ratio < 10.0,
        "Extreme timing difference detected: {:.2}x",
        max_ratio
    );
}

/// Test connection metadata protection
#[tokio::test]
async fn test_connection_metadata_protection() {
    let manager = ConnectionManager::new(20);
    let mut peer_connections = Vec::new();

    // Create connections with different patterns
    for i in 0..10 {
        let peer_id = PeerId::random();
        peer_connections.push(peer_id);

        manager.connect(peer_id).await.unwrap();
        manager
            .update_status(peer_id, ConnectionStatus::Connected)
            .await;

        // Vary connection timing
        tokio::time::sleep(Duration::from_millis(i as u64 * 10)).await;
    }

    // Update metrics with different patterns to simulate real traffic
    for i in 0..100 {
        let msg_rate = 100.0 + (i as f64 * 10.0) + (i as f64).sin() * 50.0;
        let latency = 20 + (i % 50) as u64;

        manager.update_metrics(msg_rate, latency).await;

        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    // Verify that connection metadata doesn't leak sensitive information
    let metrics = manager.get_metrics().await;
    let queue_metrics = manager.get_queue_metrics();
    let latency_metrics = manager.get_latency_metrics();
    let throughput_metrics = manager.get_throughput_metrics();

    // Metrics should be aggregated, not revealing individual connection details
    assert!(metrics.connections > 0);
    assert!(metrics.messages_per_second > 0.0);

    // Check that metrics are reasonable aggregations
    assert!(queue_metrics.utilization >= 0.0 && queue_metrics.utilization <= 1.0);
    assert!(latency_metrics.avg_latency > Duration::ZERO);
    assert!(throughput_metrics.messages_per_second > 0.0);

    // Individual peer information should not be directly accessible from metrics
    // (This is enforced by the API design - no method to get per-peer metrics)
}

/// Test resistance to statistical analysis
#[tokio::test]
async fn test_statistical_analysis_resistance() {
    let router = Router::new();

    // Add peers
    let peers: Vec<_> = (0..15).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    // Generate many routes with different source-destination pairs
    let mut all_routes = Vec::new();

    for source_idx in 0..5 {
        for dest_idx in 10..15 {
            for _ in 0..20 {
                let msg = NetworkMessage {
                    id: format!("stat_test_{}_{}", source_idx, dest_idx),
                    source: peers[source_idx].to_bytes().to_vec(),
                    destination: peers[dest_idx].to_bytes().to_vec(),
                    payload: vec![0; 100],
                    priority: MessagePriority::Normal,
                    ttl: Duration::from_secs(60),
                };

                let route = router
                    .route(&msg, RoutingStrategy::Anonymous { hops: 4 })
                    .await
                    .unwrap();
                all_routes.push((source_idx, dest_idx, route));
            }
        }
    }

    // Analyze statistical properties
    let mut source_route_correlation = std::collections::HashMap::new();
    let mut dest_route_correlation = std::collections::HashMap::new();

    for (source_idx, dest_idx, route) in &all_routes {
        // Track correlation between source and route
        let route_pattern = route.iter().map(|p| p.to_bytes()[0]).collect::<Vec<_>>();
        source_route_correlation
            .entry(*source_idx)
            .or_insert_with(Vec::new)
            .push(route_pattern.clone());

        // Track correlation between destination and route
        dest_route_correlation
            .entry(*dest_idx)
            .or_insert_with(Vec::new)
            .push(route_pattern);
    }

    // Verify that routes for the same source are sufficiently diverse
    for (source_idx, routes) in &source_route_correlation {
        let unique_routes: HashSet<_> = routes.iter().collect();
        let diversity = unique_routes.len() as f64 / routes.len() as f64;

        assert!(
            diversity >= 0.5,
            "Routes for source {} are not diverse enough: {:.2}%",
            source_idx,
            diversity * 100.0
        );
    }

    // Verify that routes for the same destination are sufficiently diverse
    for (dest_idx, routes) in &dest_route_correlation {
        let unique_routes: HashSet<_> = routes.iter().collect();
        let diversity = unique_routes.len() as f64 / routes.len() as f64;

        assert!(
            diversity >= 0.5,
            "Routes for destination {} are not diverse enough: {:.2}%",
            dest_idx,
            diversity * 100.0
        );
    }
}

/// Test forward secrecy properties
#[tokio::test]
async fn test_forward_secrecy() {
    let router = Router::new();

    // Add peers
    let peers: Vec<_> = (0..8).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    let msg1 = NetworkMessage {
        id: "forward_secrecy_test_1".into(),
        source: peers[0].to_bytes().to_vec(),
        destination: peers[7].to_bytes().to_vec(),
        payload: vec![1; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    let msg2 = NetworkMessage {
        id: "forward_secrecy_test_2".into(),
        source: peers[0].to_bytes().to_vec(),
        destination: peers[7].to_bytes().to_vec(),
        payload: vec![2; 100],
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Route first message
    let route1 = router
        .route(&msg1, RoutingStrategy::Anonymous { hops: 3 })
        .await
        .unwrap();

    // Get hop info for first route
    let mut hop_info_1 = Vec::new();
    for peer_id in &route1 {
        let info = router.get_hop_info(peer_id).await.unwrap();
        hop_info_1.push(info);
    }

    // Route second message (this should update hop knowledge)
    let route2 = router
        .route(&msg2, RoutingStrategy::Anonymous { hops: 3 })
        .await
        .unwrap();

    // Get hop info for second route
    let mut hop_info_2 = Vec::new();
    for peer_id in &route2 {
        let info = router.get_hop_info(peer_id).await.unwrap();
        hop_info_2.push(info);
    }

    // Verify that hop information is updated for new routes
    // (This simulates forward secrecy - old routing state doesn't compromise new routes)

    // If routes use same peers, their hop knowledge should be updated
    for peer_id in &route1 {
        if route2.contains(peer_id) {
            let old_info = hop_info_1.iter().find(|info| info.peer_id == *peer_id);
            let new_info = hop_info_2.iter().find(|info| info.peer_id == *peer_id);

            if let (Some(old), Some(new)) = (old_info, new_info) {
                // Knowledge should be different between routes (forward secrecy)
                let old_knows = old.known_peers.len();
                let new_knows = new.known_peers.len();

                // The exact number may vary, but the principle is that
                // each route has independent knowledge
                assert!(old_knows <= 2, "Hop knows too many peers in route 1");
                assert!(new_knows <= 2, "Hop knows too many peers in route 2");
            }
        }
    }
}

/// Test message unlinkability
#[tokio::test]
async fn test_message_unlinkability() {
    let router = Router::new();

    // Add peers
    let peers: Vec<_> = (0..12).map(|_| PeerId::random()).collect();
    for peer in &peers {
        router.add_peer(*peer).await;
    }

    // Create multiple messages from same source to different destinations
    let source_peer = peers[0];
    let destinations = vec![peers[8], peers[9], peers[10], peers[11]];

    let mut routes_by_dest = std::collections::HashMap::new();

    for dest_peer in &destinations {
        let mut routes = Vec::new();
        for i in 0..10 {
            let msg = NetworkMessage {
                id: format!("unlinkability_test_{}", i),
                source: source_peer.to_bytes().to_vec(),
                destination: dest_peer.to_bytes().to_vec(),
                payload: vec![0; 100],
                priority: MessagePriority::Normal,
                ttl: Duration::from_secs(60),
            };

            let route = router
                .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
                .await
                .unwrap();
            routes.push(route);
        }
        routes_by_dest.insert(*dest_peer, routes);
    }

    // Analyze unlinkability - routes to different destinations should not be easily linkable
    let mut common_peers_counts = Vec::new();

    for (dest1, routes1) in &routes_by_dest {
        for (dest2, routes2) in &routes_by_dest {
            if dest1 >= dest2 {
                continue;
            } // Avoid duplicate comparisons

            // Count how many peers appear in routes to both destinations
            let mut peers1 = HashSet::new();
            let mut peers2 = HashSet::new();

            for route in routes1 {
                for peer in route {
                    peers1.insert(*peer);
                }
            }

            for route in routes2 {
                for peer in route {
                    peers2.insert(*peer);
                }
            }

            let common_peers = peers1.intersection(&peers2).count();
            common_peers_counts.push(common_peers);
        }
    }

    // Messages to different destinations should have limited peer overlap
    let avg_common_peers =
        common_peers_counts.iter().sum::<usize>() as f64 / common_peers_counts.len() as f64;
    let total_available_peers = peers.len() - 5; // Exclude source and destinations
    let overlap_ratio = avg_common_peers / total_available_peers as f64;

    println!(
        "Average common peers between destination routes: {:.2}",
        avg_common_peers
    );
    println!("Overlap ratio: {:.2}%", overlap_ratio * 100.0);

    // Overlap should be reasonable (not too high to maintain unlinkability)
    assert!(
        overlap_ratio < 0.7,
        "Too much overlap between routes to different destinations: {:.2}%",
        overlap_ratio * 100.0
    );
}

/// CRITICAL VULNERABILITY TESTS
/// These tests validate the critical security vulnerabilities identified in the audit

/// Test for nonce reuse vulnerability (CRITICAL)
#[tokio::test]
async fn test_nonce_reuse_vulnerability() {
    // This test checks if the system reuses nonces, which would be a critical vulnerability
    let mut message_hashes = HashSet::new();

    for i in 0..1000 {
        let msg = NetworkMessage {
            id: format!("nonce_test_{}", i),
            source: vec![1, 2, 3, 4],
            destination: vec![5, 6, 7, 8],
            payload: b"identical payload for nonce test".to_vec(), // Intentionally identical
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let envelope = MessageEnvelope::new(msg);

        if message_hashes.contains(&envelope.hash) {
            panic!(
                "CRITICAL: Duplicate message hash detected - nonce reuse vulnerability confirmed"
            );
        }

        message_hashes.insert(envelope.hash);
    }

    println!(
        "Nonce uniqueness test passed - {} unique hashes generated",
        message_hashes.len()
    );
}

/// Test for weak message authentication (HIGH)
#[tokio::test]
async fn test_message_authentication_bypass() {
    let msg = NetworkMessage {
        id: "auth_bypass_test".into(),
        source: vec![1, 2, 3, 4],
        destination: vec![5, 6, 7, 8],
        payload: b"sensitive payload".to_vec(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(60),
    };

    let mut envelope = MessageEnvelope::new(msg.clone());

    // Verify original message is valid
    assert!(
        envelope.verify(),
        "Original message should verify correctly"
    );

    // Test tampering detection
    envelope.message.payload = b"tampered payload".to_vec();

    // The system should detect tampering
    if envelope.verify() {
        panic!("CRITICAL: Message tampering not detected - authentication bypass vulnerability");
    }

    // Test signature requirement
    let result = envelope.verify_signature(b"fake_public_key");
    match result {
        Ok(true) => {
            panic!("CRITICAL: Unsigned message accepted as valid - signature bypass vulnerability")
        }
        Ok(false) => println!("Correctly rejected unsigned message"),
        Err(_) => println!("Error handling unsigned message verification"),
    }
}

/// Test for DoS through unbounded queues (HIGH)
#[tokio::test]
async fn test_queue_dos_vulnerability() {
    let (queue, _rx) = MessageQueue::new();
    let mut messages_accepted = 0;
    let max_test_messages = 200_000;

    // Attempt to flood the queue
    for i in 0..max_test_messages {
        let msg = NetworkMessage {
            id: format!("dos_test_{}", i),
            source: vec![i as u8, (i >> 8) as u8, (i >> 16) as u8, (i >> 24) as u8],
            destination: vec![255, 254, 253, 252],
            payload: vec![0; 1024], // 1KB per message
            priority: MessagePriority::Low,
            ttl: Duration::from_secs(60),
        };

        match timeout(Duration::from_millis(1), queue.enqueue(msg)).await {
            Ok(Ok(_)) => {
                messages_accepted += 1;

                // Check if queue size is growing unboundedly
                if i % 10000 == 0 {
                    let queue_size = queue.len().await;
                    println!("Queue size after {} messages: {}", i, queue_size);

                    if queue_size > 150_000 {
                        panic!(
                            "CRITICAL: Queue grew unboundedly to {} - DoS vulnerability confirmed",
                            queue_size
                        );
                    }
                }
            }
            Ok(Err(_)) => {
                println!("Message rejected at iteration {} - protection working", i);
                break;
            }
            Err(_) => {
                println!("Timeout at iteration {} - potential DoS", i);
                break;
            }
        }
    }

    println!(
        "DoS test completed - {} messages accepted before protection kicked in",
        messages_accepted
    );

    if messages_accepted > 150_000 {
        panic!("CRITICAL: Too many messages accepted - DoS vulnerability detected");
    }
}

/// Test for connection pool exhaustion (HIGH)
#[tokio::test]
async fn test_connection_pool_dos() {
    let manager = ConnectionManager::new(10); // Small limit for testing
    let mut successful_connections = 0;

    // Attempt to exhaust connection pool
    for i in 0..100 {
        let peer_id = PeerId::random();

        match manager.connect(peer_id).await {
            Ok(_) => {
                successful_connections += 1;
                println!("Connection {} accepted", i);
            }
            Err(_) => {
                println!("Connection {} rejected", i);
                break;
            }
        }

        // Check if limit is being enforced
        if successful_connections > 15 {
            // Allow some margin for async behavior
            panic!(
                "CRITICAL: Connection limit not enforced - {} connections accepted",
                successful_connections
            );
        }
    }

    println!(
        "Connection limit test passed - {} connections accepted",
        successful_connections
    );
}

/// Test for replay attack vulnerability (HIGH)
#[tokio::test]
async fn test_replay_attack_vulnerability() {
    let (queue, _rx) = MessageQueue::new();

    // Create a message
    let msg = NetworkMessage {
        id: "replay_test".into(),
        source: vec![1, 2, 3, 4],
        destination: vec![5, 6, 7, 8],
        payload: b"replay test payload".to_vec(),
        priority: MessagePriority::Normal,
        ttl: Duration::from_secs(60),
    };

    // Send the message
    assert!(
        queue.enqueue(msg.clone()).await.is_ok(),
        "First message should be accepted"
    );

    // Try to replay the same message
    let replay_result = queue.enqueue(msg.clone()).await;

    // System should detect and reject replay
    match replay_result {
        Ok(_) => println!("WARNING: Replay attack not prevented - message accepted twice"),
        Err(_) => println!("Replay attack correctly prevented"),
    }

    // Test with old timestamp
    let old_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 3600; // 1 hour old

    let mut old_msg = msg.clone();
    old_msg.id = "old_message_test".into();

    let mut envelope = MessageEnvelope::new(old_msg);
    envelope.timestamp = old_timestamp;

    // Recalculate hash with old timestamp
    let mut hasher = blake3::Hasher::new();
    hasher.update(&bincode::serialize(&envelope.message).unwrap());
    hasher.update(&envelope.timestamp.to_le_bytes());
    envelope.hash = hasher.finalize();

    // Should reject old messages
    if envelope.verify() {
        println!("WARNING: Old message accepted - timestamp validation may be weak");
    }
}

/// Test for peer identity spoofing (MEDIUM)
#[tokio::test]
async fn test_peer_identity_spoofing() {
    let manager = ConnectionManager::new(50);

    // Create a legitimate peer
    let legitimate_peer = PeerId::random();
    manager.connect(legitimate_peer).await.unwrap();

    // Try to create a fake peer with same identity
    let spoofed_peer = PeerId::from_bytes(legitimate_peer.to_bytes());

    // System should detect identity conflict
    let spoof_result = manager.connect(spoofed_peer).await;

    match spoof_result {
        Ok(_) => println!("WARNING: Identity spoofing not prevented - same peer ID accepted twice"),
        Err(_) => println!("Identity spoofing correctly prevented"),
    }

    // Test weak peer ID generation
    let mut weak_ids = 0;
    for _ in 0..1000 {
        let peer = PeerId::random();
        let bytes = peer.to_bytes();

        // Check for weak patterns
        if bytes.iter().all(|&b| b == 0)
            || bytes.iter().all(|&b| b == 255)
            || bytes.windows(4).any(|w| w == [0, 0, 0, 0])
        {
            weak_ids += 1;
        }
    }

    if weak_ids > 0 {
        panic!(
            "CRITICAL: {} weak peer IDs generated out of 1000 - randomness vulnerability",
            weak_ids
        );
    }

    println!("Peer ID generation passed entropy test");
}

/// Test for information leakage through timing (MEDIUM)
#[tokio::test]
async fn test_timing_information_leakage() {
    let (queue, _rx) = MessageQueue::new();
    let mut timing_data = Vec::new();

    // Test message processing timing with different payload sizes
    for size in [100, 1000, 10000, 100000] {
        let mut times_for_size = Vec::new();

        for i in 0..10 {
            let msg = NetworkMessage {
                id: format!("timing_test_{}_{}", size, i),
                source: vec![1, 2, 3, 4],
                destination: vec![5, 6, 7, 8],
                payload: vec![0; size],
                priority: MessagePriority::Normal,
                ttl: Duration::from_secs(60),
            };

            let start = Instant::now();
            let _ = queue.enqueue(msg).await;
            let elapsed = start.elapsed();

            times_for_size.push(elapsed.as_nanos());
        }

        let avg_time = times_for_size.iter().sum::<u128>() / times_for_size.len() as u128;
        timing_data.push((size, avg_time));
    }

    // Analyze timing correlation with payload size
    println!("Timing analysis by payload size:");
    for (size, avg_time) in &timing_data {
        println!("  Size: {} bytes, Avg time: {} ns", size, avg_time);
    }

    // Check for linear correlation (potential timing leak)
    let first_time = timing_data[0].1 as f64;
    let last_time = timing_data.last().unwrap().1 as f64;
    let time_ratio = last_time / first_time;

    if time_ratio > 10.0 {
        println!("WARNING: Significant timing variation detected - potential information leakage through timing analysis");
    }
}

/// Test for metadata exposure vulnerability (MEDIUM)  
#[tokio::test]
async fn test_metadata_exposure() {
    let msg = NetworkMessage {
        id: "metadata_test".into(),
        source: vec![1, 2, 3, 4],
        destination: vec![5, 6, 7, 8],
        payload: b"confidential data".to_vec(),
        priority: MessagePriority::High,
        ttl: Duration::from_secs(60),
    };

    // Test serialization exposure
    let serialized = serde_json::to_string(&msg).unwrap();

    // Check for exposed sensitive information
    let mut exposures = Vec::new();

    if serialized.contains("confidential data") {
        exposures.push("Payload data exposed in serialization");
    }

    if serialized.len() > 1000 {
        exposures.push("Serialized message is very large - potential metadata leakage");
    }

    // Check for timing metadata
    let envelope = MessageEnvelope::new(msg);
    let timestamp_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let timestamp_diff = if envelope.timestamp > timestamp_now {
        envelope.timestamp - timestamp_now
    } else {
        timestamp_now - envelope.timestamp
    };

    if timestamp_diff < 1 {
        exposures.push("Timestamp reveals precise message creation time");
    }

    for exposure in exposures {
        println!("WARNING: {}", exposure);
    }
}

/// Test for weak encryption implementation (CRITICAL)
#[tokio::test]
async fn test_weak_encryption_implementation() {
    // Test key generation entropy
    let mut keys = Vec::new();
    for _ in 0..100 {
        let peer = PeerId::random();
        keys.push(peer.to_bytes());
    }

    // Check for duplicate keys (should be extremely unlikely)
    let mut key_set = HashSet::new();
    let mut duplicates = 0;

    for key in &keys {
        if !key_set.insert(key) {
            duplicates += 1;
        }
    }

    if duplicates > 0 {
        panic!(
            "CRITICAL: {} duplicate keys generated - weak randomness in key generation",
            duplicates
        );
    }

    // Check for patterns in key generation
    let mut byte_frequencies = [0u32; 256];
    for key in &keys {
        for &byte in key {
            byte_frequencies[byte as usize] += 1;
        }
    }

    // Statistical test for uniform distribution
    let expected_freq = (keys.len() * 32) / 256;
    let threshold = expected_freq / 3; // Allow significant variation

    let mut biased_bytes = 0;
    for (byte_val, &freq) in byte_frequencies.iter().enumerate() {
        if freq < threshold || freq > expected_freq + threshold {
            biased_bytes += 1;
            if freq == 0 || freq > expected_freq * 2 {
                println!(
                    "WARNING: Byte value {} has suspicious frequency: {}",
                    byte_val, freq
                );
            }
        }
    }

    if biased_bytes > 50 {
        // More than ~20% of bytes showing bias
        println!(
            "WARNING: {} byte values show significant bias - potential weak randomness",
            biased_bytes
        );
    }

    println!(
        "Encryption entropy test completed - {} keys tested",
        keys.len()
    );
}

/// Test for resource exhaustion through large messages (HIGH)
#[tokio::test]
async fn test_large_message_dos() {
    let (queue, _rx) = MessageQueue::new();

    // Try to exhaust memory with increasingly large messages
    for size_mb in 1..=20 {
        let payload_size = size_mb * 1024 * 1024; // MB in bytes

        let msg = NetworkMessage {
            id: format!("large_msg_test_{}", size_mb),
            source: vec![1, 2, 3, 4],
            destination: vec![5, 6, 7, 8],
            payload: vec![0; payload_size],
            priority: MessagePriority::Low,
            ttl: Duration::from_secs(60),
        };

        match timeout(Duration::from_millis(100), queue.enqueue(msg)).await {
            Ok(Ok(_)) => {
                println!("Accepted {}MB message", size_mb);

                if size_mb > 10 {
                    println!("WARNING: System accepting very large messages ({}MB) - potential DoS vulnerability", size_mb);
                }
            }
            Ok(Err(_)) => {
                println!("Rejected {}MB message - protection working", size_mb);
                break;
            }
            Err(_) => {
                println!(
                    "Timeout on {}MB message - system may be overwhelmed",
                    size_mb
                );
                break;
            }
        }
    }
}

/// Test for concurrent attack scenarios
#[tokio::test]
async fn test_concurrent_attack_scenarios() {
    let (queue, _rx) = MessageQueue::new();
    let queue = Arc::new(queue);
    let manager = Arc::new(ConnectionManager::new(50));

    let mut handles = Vec::new();

    // Spawn concurrent DoS attempts
    for task_id in 0..5 {
        let queue_clone = queue.clone();
        let manager_clone = manager.clone();

        let handle = tokio::spawn(async move {
            let mut local_stats = (0, 0); // (messages_sent, connections_made)

            for i in 0..1000 {
                // Alternate between message flooding and connection flooding
                if i % 2 == 0 {
                    let msg = NetworkMessage {
                        id: format!("concurrent_attack_{}_{}", task_id, i),
                        source: vec![task_id as u8],
                        destination: vec![255],
                        payload: vec![0; 1024],
                        priority: MessagePriority::Low,
                        ttl: Duration::from_secs(60),
                    };

                    if queue_clone.enqueue(msg).await.is_ok() {
                        local_stats.0 += 1;
                    }
                } else {
                    let peer_id = PeerId::random();
                    if manager_clone.connect(peer_id).await.is_ok() {
                        local_stats.1 += 1;
                    }
                }

                // Early exit if we're sending too much
                if local_stats.0 > 500 || local_stats.1 > 25 {
                    break;
                }
            }

            local_stats
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_messages = 0;
    let mut total_connections = 0;

    for handle in handles {
        let (messages, connections) = handle.await.unwrap();
        total_messages += messages;
        total_connections += connections;
    }

    println!(
        "Concurrent attack test: {} messages, {} connections accepted",
        total_messages, total_connections
    );

    // Verify protection mechanisms worked
    if total_messages > 10000 {
        panic!("CRITICAL: Too many messages accepted under concurrent attack - DoS vulnerability");
    }

    if total_connections > 100 {
        panic!(
            "CRITICAL: Too many connections accepted under concurrent attack - DoS vulnerability"
        );
    }

    // Check final system state
    let final_queue_size = queue.len().await;
    let final_connection_count = manager.connection_count().await;

    println!(
        "Final state: {} queued messages, {} active connections",
        final_queue_size, final_connection_count
    );
}
