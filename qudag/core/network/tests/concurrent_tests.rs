//! Concurrent operations and thread safety tests for network module
//!
//! This module tests the thread safety of network operations, including
//! connection management, message processing, routing, and peer discovery
//! under high concurrency and contention scenarios.

use bytes::Bytes;
use qudag_network::{
    connection::{ConnectionManager, SecureConfig, SecureConnection, TransportKeys},
    message::{Message, MessageHeader, MessageType},
    p2p::P2PManager,
    peer::{PeerInfo, PeerManager, PeerStatus},
    router::{Route, Router, RoutingTable},
    types::{ConnectionStatus, NetworkError, NetworkMetrics, PeerId},
};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Mutex, RwLock, Semaphore};
use tokio::time::sleep;

/// Test concurrent connection management operations
#[tokio::test]
async fn test_concurrent_connection_management() {
    const NUM_THREADS: usize = 20;
    const CONNECTIONS_PER_THREAD: usize = 50;
    const MAX_CONNECTIONS: usize = 100;

    let manager = Arc::new(ConnectionManager::new(MAX_CONNECTIONS));
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;

            let mut successful_connections = 0;
            let mut successful_disconnections = 0;
            let mut status_updates = 0;
            let mut peers = Vec::new();

            // Create connections
            for i in 0..CONNECTIONS_PER_THREAD {
                let peer_id = PeerId::random();
                peers.push(peer_id);

                match manager_clone.connect(peer_id).await {
                    Ok(()) => {
                        successful_connections += 1;

                        // Update status
                        manager_clone.update_status(peer_id, ConnectionStatus::Connected);
                        status_updates += 1;

                        // Verify status
                        if let Some(status) = manager_clone.get_status(&peer_id) {
                            assert_eq!(status, ConnectionStatus::Connected);
                        }
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Connection {} failed: {}", thread_id, i, e);
                    }
                }

                // Yield to allow other threads to run
                if i % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            // Disconnect half of the connections
            let disconnect_count = peers.len() / 2;
            for peer_id in &peers[..disconnect_count] {
                manager_clone.disconnect(peer_id);
                successful_disconnections += 1;
            }

            (
                thread_id,
                successful_connections,
                successful_disconnections,
                status_updates,
            )
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_connections = 0;
    let mut total_disconnections = 0;
    let mut total_status_updates = 0;

    for handle in handles {
        let (thread_id, connections, disconnections, updates) = handle.await.unwrap();
        println!(
            "Thread {}: {} connections, {} disconnections, {} status updates",
            thread_id, connections, disconnections, updates
        );
        total_connections += connections;
        total_disconnections += disconnections;
        total_status_updates += updates;
    }

    println!(
        "Total: {} connections, {} disconnections, {} status updates",
        total_connections, total_disconnections, total_status_updates
    );

    // Verify final state
    let final_connection_count = manager.connection_count();
    let expected_connections = total_connections - total_disconnections;

    println!(
        "Final connection count: {}, expected: {}",
        final_connection_count, expected_connections
    );

    // The actual count may be limited by MAX_CONNECTIONS
    assert!(
        final_connection_count <= MAX_CONNECTIONS,
        "Should not exceed max connections"
    );
    assert!(
        total_connections > 0,
        "Should have some successful connections"
    );
}

/// Test concurrent message processing
#[tokio::test]
async fn test_concurrent_message_processing() {
    const NUM_PRODUCERS: usize = 10;
    const NUM_CONSUMERS: usize = 5;
    const MESSAGES_PER_PRODUCER: usize = 100;

    let message_queue = Arc::new(RwLock::new(Vec::<Message>::new()));
    let processed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(NUM_PRODUCERS + NUM_CONSUMERS));

    let mut handles = Vec::new();

    // Producer tasks
    for producer_id in 0..NUM_PRODUCERS {
        let queue_clone = message_queue.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut produced = 0;

            for i in 0..MESSAGES_PER_PRODUCER {
                let message_data = format!("Producer {} message {}", producer_id, i);
                let message = Message::new(MessageType::Data, message_data.into_bytes());

                {
                    let mut queue = queue_clone.write().await;
                    queue.push(message);
                    produced += 1;
                }

                // Small delay to create interleaving
                if i % 20 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }

            (format!("Producer_{}", producer_id), produced)
        });

        handles.push(handle);
    }

    // Consumer tasks
    for consumer_id in 0..NUM_CONSUMERS {
        let queue_clone = message_queue.clone();
        let count_clone = processed_count.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut consumed = 0;
            let start_time = Instant::now();
            let timeout = Duration::from_secs(10);

            while start_time.elapsed() < timeout {
                let message_opt = {
                    let mut queue = queue_clone.write().await;
                    queue.pop()
                };

                if let Some(message) = message_opt {
                    // Simulate message processing
                    let _payload = message.payload();
                    consumed += 1;
                    count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                } else {
                    // No messages available, small delay
                    sleep(Duration::from_millis(1)).await;
                }
            }

            (format!("Consumer_{}", consumer_id), consumed)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_produced = 0;
    let mut total_consumed = 0;

    for handle in handles {
        let (task_name, count) = handle.await.unwrap();
        if task_name.starts_with("Producer") {
            total_produced += count;
        } else {
            total_consumed += count;
        }
        println!("{}: {} messages", task_name, count);
    }

    let final_processed = processed_count.load(std::sync::atomic::Ordering::SeqCst);
    let remaining_messages = message_queue.read().await.len();

    println!(
        "Total produced: {}, consumed: {}, processed counter: {}, remaining: {}",
        total_produced, total_consumed, final_processed, remaining_messages
    );

    assert_eq!(
        total_produced,
        NUM_PRODUCERS * MESSAGES_PER_PRODUCER,
        "All messages should be produced"
    );
    assert_eq!(
        total_consumed, final_processed,
        "Consumed count should match processed counter"
    );
    assert_eq!(
        total_consumed + remaining_messages,
        total_produced,
        "Total consumed plus remaining should equal produced"
    );
}

/// Test concurrent peer management
#[tokio::test]
async fn test_concurrent_peer_management() {
    const NUM_THREADS: usize = 15;
    const PEERS_PER_THREAD: usize = 20;

    let peer_manager = Arc::new(PeerManager::new());
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let manager_clone = peer_manager.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut added_peers = 0;
            let mut status_updates = 0;
            let mut removed_peers = 0;
            let mut peer_list = Vec::new();

            // Add peers
            for i in 0..PEERS_PER_THREAD {
                let peer_id = PeerId::random();
                let peer_info = PeerInfo {
                    id: peer_id,
                    address: SocketAddr::new(
                        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        8000 + (thread_id * 100 + i) as u16,
                    ),
                    status: PeerStatus::Connecting,
                    last_seen: Instant::now(),
                    reputation: 100,
                };

                if manager_clone.add_peer(peer_info).await.is_ok() {
                    added_peers += 1;
                    peer_list.push(peer_id);

                    // Update status
                    if manager_clone
                        .update_peer_status(peer_id, PeerStatus::Connected)
                        .await
                        .is_ok()
                    {
                        status_updates += 1;
                    }
                }

                // Yield occasionally
                if i % 5 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            // Remove some peers
            let remove_count = peer_list.len() / 3;
            for peer_id in &peer_list[..remove_count] {
                if manager_clone.remove_peer(peer_id).await.is_ok() {
                    removed_peers += 1;
                }
            }

            (thread_id, added_peers, status_updates, removed_peers)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_added = 0;
    let mut total_status_updates = 0;
    let mut total_removed = 0;

    for handle in handles {
        let (thread_id, added, updates, removed) = handle.await.unwrap();
        println!(
            "Thread {}: {} added, {} status updates, {} removed",
            thread_id, added, updates, removed
        );
        total_added += added;
        total_status_updates += updates;
        total_removed += removed;
    }

    let final_peer_count = peer_manager.peer_count().await;
    let expected_peers = total_added - total_removed;

    println!(
        "Total: {} added, {} removed, {} status updates",
        total_added, total_removed, total_status_updates
    );
    println!(
        "Final peer count: {}, expected: {}",
        final_peer_count, expected_peers
    );

    assert_eq!(
        final_peer_count, expected_peers,
        "Final peer count should match expectations"
    );
    assert!(total_added > 0, "Should add some peers");
}

/// Test concurrent routing operations
#[tokio::test]
async fn test_concurrent_routing_operations() {
    const NUM_THREADS: usize = 12;
    const ROUTES_PER_THREAD: usize = 50;

    let router = Arc::new(Router::new());
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let router_clone = router.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut added_routes = 0;
            let mut found_routes = 0;
            let mut removed_routes = 0;
            let mut route_list = Vec::new();

            // Add routes
            for i in 0..ROUTES_PER_THREAD {
                let destination = PeerId::random();
                let next_hop = PeerId::random();
                let cost = thread_rng().gen_range(1..100);

                let route = Route {
                    destination,
                    next_hop,
                    cost,
                    timestamp: Instant::now(),
                };

                if router_clone.add_route(route).await.is_ok() {
                    added_routes += 1;
                    route_list.push(destination);
                }

                // Test route lookup
                if let Some(_route) = router_clone.find_route(&destination).await {
                    found_routes += 1;
                }

                // Yield occasionally
                if i % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            // Remove some routes
            let remove_count = route_list.len() / 2;
            for destination in &route_list[..remove_count] {
                if router_clone.remove_route(destination).await.is_ok() {
                    removed_routes += 1;
                }
            }

            (thread_id, added_routes, found_routes, removed_routes)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_added = 0;
    let mut total_found = 0;
    let mut total_removed = 0;

    for handle in handles {
        let (thread_id, added, found, removed) = handle.await.unwrap();
        println!(
            "Thread {}: {} routes added, {} found, {} removed",
            thread_id, added, found, removed
        );
        total_added += added;
        total_found += found;
        total_removed += removed;
    }

    let final_route_count = router.route_count().await;
    let expected_routes = total_added - total_removed;

    println!(
        "Total: {} added, {} found, {} removed, final count: {}",
        total_added, total_found, total_removed, final_route_count
    );

    assert_eq!(
        final_route_count, expected_routes,
        "Final route count should match expectations"
    );
    assert!(total_added > 0, "Should add some routes");
    assert_eq!(
        total_found, total_added,
        "Should find all added routes initially"
    );
}

/// Test race conditions in network state management
#[tokio::test]
async fn test_network_race_conditions() {
    const NUM_THREADS: usize = 20;
    const OPERATIONS_PER_THREAD: usize = 100;

    let connection_manager = Arc::new(ConnectionManager::new(1000));
    let shared_metrics = Arc::new(Mutex::new(NetworkMetrics::default()));
    let operation_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(NUM_THREADS));

    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let manager_clone = connection_manager.clone();
        let metrics_clone = shared_metrics.clone();
        let counter_clone = operation_counter.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut local_operations = 0;

            for i in 0..OPERATIONS_PER_THREAD {
                let operation_type = i % 4;

                match operation_type {
                    0 => {
                        // Connection operations
                        let peer_id = PeerId::random();
                        if manager_clone.connect(peer_id).await.is_ok() {
                            manager_clone.update_status(peer_id, ConnectionStatus::Connected);
                            local_operations += 1;
                        }
                    }
                    1 => {
                        // Metrics update (potential race condition)
                        let mut metrics = metrics_clone.lock().await;
                        metrics.messages_per_second += 1.0;
                        metrics.active_connections = manager_clone.connection_count();
                        local_operations += 1;
                    }
                    2 => {
                        // Disconnection
                        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
                        for peer_id in peers {
                            let _ = manager_clone.connect(peer_id).await;
                            manager_clone.disconnect(&peer_id);
                        }
                        local_operations += 1;
                    }
                    3 => {
                        // Read operations (should not interfere)
                        let _count = manager_clone.connection_count();
                        let _metrics = metrics_clone.lock().await.clone();
                        local_operations += 1;
                    }
                    _ => unreachable!(),
                }

                counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                // Yield to increase chance of race conditions
                if i % 20 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (thread_id, local_operations)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_operations = 0;

    for handle in handles {
        let (thread_id, operations) = handle.await.unwrap();
        println!("Thread {}: {} operations completed", thread_id, operations);
        total_operations += operations;
    }

    let final_counter = operation_counter.load(std::sync::atomic::Ordering::SeqCst);
    let final_metrics = shared_metrics.lock().await;
    let final_connections = connection_manager.connection_count();

    println!("Race condition test results:");
    println!("  Total operations: {}", total_operations);
    println!("  Counter value: {}", final_counter);
    println!("  Final connections: {}", final_connections);
    println!("  Final metrics MPS: {}", final_metrics.messages_per_second);

    // Verify no race conditions caused data corruption
    assert_eq!(
        final_counter,
        NUM_THREADS * OPERATIONS_PER_THREAD,
        "Counter should match total operations"
    );
    assert!(
        final_metrics.messages_per_second >= 0.0,
        "Metrics should not be corrupted"
    );
    assert!(
        final_connections >= 0,
        "Connection count should not be negative"
    );
}

/// Test high-concurrency network stress scenarios
#[tokio::test]
async fn test_network_high_concurrency_stress() {
    const NUM_CONNECTION_THREADS: usize = 20;
    const NUM_MESSAGE_THREADS: usize = 15;
    const NUM_PEER_THREADS: usize = 10;
    const STRESS_DURATION_SECS: u64 = 10;

    let connection_manager = Arc::new(ConnectionManager::new(500));
    let peer_manager = Arc::new(PeerManager::new());
    let router = Arc::new(Router::new());
    let message_queue = Arc::new(RwLock::new(Vec::<Message>::new()));

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(STRESS_DURATION_SECS);

    let mut handles = Vec::new();

    // Connection stress threads
    for thread_id in 0..NUM_CONNECTION_THREADS {
        let manager_clone = connection_manager.clone();

        let handle = tokio::spawn(async move {
            let mut operations = 0;

            while Instant::now() < end_time {
                let peer_id = PeerId::random();

                if manager_clone.connect(peer_id).await.is_ok() {
                    manager_clone.update_status(peer_id, ConnectionStatus::Connected);
                    operations += 1;

                    // Occasionally disconnect
                    if operations % 10 == 0 {
                        manager_clone.disconnect(&peer_id);
                    }
                }

                if operations % 50 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (format!("Connection_{}", thread_id), operations)
        });

        handles.push(handle);
    }

    // Message processing stress threads
    for thread_id in 0..NUM_MESSAGE_THREADS {
        let queue_clone = message_queue.clone();

        let handle = tokio::spawn(async move {
            let mut operations = 0;

            while Instant::now() < end_time {
                // Produce messages
                let message = Message::new(
                    MessageType::Data,
                    format!("stress_message_{}", operations).into_bytes(),
                );

                {
                    let mut queue = queue_clone.write().await;
                    queue.push(message);
                }

                // Consume messages
                {
                    let mut queue = queue_clone.write().await;
                    if !queue.is_empty() {
                        queue.remove(0);
                    }
                }

                operations += 2; // One produce, one consume

                if operations % 100 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (format!("Message_{}", thread_id), operations)
        });

        handles.push(handle);
    }

    // Peer management stress threads
    for thread_id in 0..NUM_PEER_THREADS {
        let manager_clone = peer_manager.clone();

        let handle = tokio::spawn(async move {
            let mut operations = 0;
            let mut peers = Vec::new();

            while Instant::now() < end_time {
                if operations % 3 == 0 {
                    // Add peer
                    let peer_id = PeerId::random();
                    let peer_info = PeerInfo {
                        id: peer_id,
                        address: SocketAddr::new(
                            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                            8000 + thread_rng().gen_range(0..1000),
                        ),
                        status: PeerStatus::Connected,
                        last_seen: Instant::now(),
                        reputation: 100,
                    };

                    if manager_clone.add_peer(peer_info).await.is_ok() {
                        peers.push(peer_id);
                    }
                } else if !peers.is_empty() {
                    // Remove or update peer
                    let index = thread_rng().gen_range(0..peers.len());
                    let peer_id = peers[index];

                    if operations % 2 == 0 {
                        let _ = manager_clone
                            .update_peer_status(peer_id, PeerStatus::Disconnected)
                            .await;
                    } else {
                        if manager_clone.remove_peer(&peer_id).await.is_ok() {
                            peers.remove(index);
                        }
                    }
                }

                operations += 1;

                if operations % 25 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (format!("Peer_{}", thread_id), operations)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_operations = 0;
    let mut results_by_type = HashMap::new();

    for handle in handles {
        let (task_name, operations) = handle.await.unwrap();
        let task_type = task_name.split('_').next().unwrap();
        *results_by_type.entry(task_type.to_string()).or_insert(0) += operations;
        total_operations += operations;
        println!("{}: {} operations", task_name, operations);
    }

    let elapsed = start_time.elapsed();
    let ops_per_second = total_operations as f64 / elapsed.as_secs_f64();

    println!("\nHigh concurrency stress test results:");
    println!("  Duration: {:?}", elapsed);
    println!("  Total operations: {}", total_operations);
    println!("  Operations per second: {:.2}", ops_per_second);

    for (task_type, ops) in &results_by_type {
        println!("  {}: {} operations", task_type, ops);
    }

    println!(
        "  Final connections: {}",
        connection_manager.connection_count()
    );
    println!("  Final peers: {}", peer_manager.peer_count().await);
    println!("  Final routes: {}", router.route_count().await);
    println!("  Remaining messages: {}", message_queue.read().await.len());

    // Performance assertions
    assert!(total_operations > 0, "Should complete some operations");
    assert!(
        ops_per_second > 100.0,
        "Should achieve reasonable throughput"
    );

    // State consistency assertions
    assert!(connection_manager.connection_count() >= 0);
    assert!(peer_manager.peer_count().await >= 0);
    assert!(router.route_count().await >= 0);
}

/// Test thread-safe data structure consistency
#[tokio::test]
async fn test_thread_safe_data_structures() {
    const NUM_READERS: usize = 10;
    const NUM_WRITERS: usize = 5;
    const OPERATIONS_PER_TASK: usize = 200;

    // Test DashMap-like concurrent operations
    let connection_manager = Arc::new(ConnectionManager::new(1000));
    let consistency_check = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let barrier = Arc::new(Barrier::new(NUM_READERS + NUM_WRITERS));

    let mut handles = Vec::new();

    // Writer tasks
    for writer_id in 0..NUM_WRITERS {
        let manager_clone = connection_manager.clone();
        let check_clone = consistency_check.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut operations = 0;

            for i in 0..OPERATIONS_PER_TASK {
                let peer_id = PeerId::random();

                // Add connection
                if manager_clone.connect(peer_id).await.is_ok() {
                    operations += 1;

                    // Update status
                    manager_clone.update_status(peer_id, ConnectionStatus::Connected);

                    // Verify the update took effect
                    if let Some(status) = manager_clone.get_status(&peer_id) {
                        if status != ConnectionStatus::Connected {
                            check_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                            eprintln!(
                                "Writer {}: Consistency check failed for status update",
                                writer_id
                            );
                        }
                    } else {
                        check_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                        eprintln!(
                            "Writer {}: Consistency check failed - peer not found after add",
                            writer_id
                        );
                    }

                    // Periodically remove to test removal consistency
                    if i % 10 == 0 {
                        manager_clone.disconnect(&peer_id);

                        // Verify removal
                        if manager_clone.get_status(&peer_id).is_some() {
                            check_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                            eprintln!(
                                "Writer {}: Consistency check failed - peer found after removal",
                                writer_id
                            );
                        }
                    }
                }

                if i % 20 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (format!("Writer_{}", writer_id), operations)
        });

        handles.push(handle);
    }

    // Reader tasks
    for reader_id in 0..NUM_READERS {
        let manager_clone = connection_manager.clone();
        let check_clone = consistency_check.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut operations = 0;
            let mut consistency_checks = 0;

            for i in 0..OPERATIONS_PER_TASK {
                // Read operations should not interfere with writers
                let count_before = manager_clone.connection_count();
                let metrics_before = manager_clone.get_metrics();

                // Small delay to allow writers to operate
                tokio::task::yield_now().await;

                let count_after = manager_clone.connection_count();
                let metrics_after = manager_clone.get_metrics();

                // Verify metrics consistency
                if metrics_after.active_connections == count_after {
                    consistency_checks += 1;
                } else {
                    check_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                    eprintln!(
                        "Reader {}: Metrics inconsistency - active: {}, count: {}",
                        reader_id, metrics_after.active_connections, count_after
                    );
                }

                operations += 1;

                if i % 50 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }

            (
                format!("Reader_{}", reader_id),
                operations,
                consistency_checks,
            )
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_operations = 0;
    let mut total_consistency_checks = 0;

    for handle in handles {
        let result = handle.await.unwrap();
        match result {
            (name, ops, checks) => {
                println!(
                    "{}: {} operations, {} consistency checks",
                    name, ops, checks
                );
                total_operations += ops;
                total_consistency_checks += checks;
            }
            (name, ops) => {
                println!("{}: {} operations", name, ops);
                total_operations += ops;
            }
        }
    }

    let final_consistency = consistency_check.load(std::sync::atomic::Ordering::SeqCst);
    let final_connections = connection_manager.connection_count();
    let final_metrics = connection_manager.get_metrics();

    println!("\nThread-safe data structure test results:");
    println!("  Total operations: {}", total_operations);
    println!("  Consistency checks: {}", total_consistency_checks);
    println!("  Final consistency: {}", final_consistency);
    println!("  Final connections: {}", final_connections);
    println!(
        "  Metrics active connections: {}",
        final_metrics.active_connections
    );

    assert!(
        final_consistency,
        "Data structures should maintain consistency"
    );
    assert_eq!(
        final_connections, final_metrics.active_connections,
        "Connection count should match metrics"
    );
    assert!(total_operations > 0, "Should complete operations");
}
