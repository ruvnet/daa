//! Unit tests for connection module

use qudag_network::connection::{
    CircuitBreaker, CircuitBreakerState, ConnectionInfo, ConnectionManager, 
    HealthStatistics, SecureConfig, SecureConnection, TransportKeys, UnhealthyConnectionInfo,
};
use qudag_network::types::{ConnectionStatus, NetworkError, PeerId};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn test_connection_manager_basic_operations() {
    let manager = ConnectionManager::new(10);
    
    // Test adding connections
    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    
    assert!(manager.connect(peer1).await.is_ok());
    assert!(manager.connect(peer2).await.is_ok());
    assert_eq!(manager.connection_count(), 2);
    
    // Test connection limit
    for i in 0..10 {
        let peer = PeerId::random();
        let result = manager.connect(peer).await;
        if i < 8 {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(NetworkError::ConnectionLimitReached)));
        }
    }
}

#[tokio::test]
async fn test_connection_status_updates() {
    let manager = ConnectionManager::new(10);
    let peer = PeerId::random();
    
    manager.connect(peer).await.unwrap();
    
    // Update status
    manager.update_status(peer, ConnectionStatus::Connected);
    assert_eq!(manager.get_status(&peer), Some(ConnectionStatus::Connected));
    
    manager.update_status(peer, ConnectionStatus::Disconnected);
    assert_eq!(manager.get_status(&peer), Some(ConnectionStatus::Disconnected));
    
    // Check unknown peer
    let unknown = PeerId::random();
    assert_eq!(manager.get_status(&unknown), None);
}

#[tokio::test]
async fn test_connection_manager_disconnection() {
    let manager = ConnectionManager::new(10);
    let peer = PeerId::random();
    
    manager.connect(peer).await.unwrap();
    assert_eq!(manager.connection_count(), 1);
    
    manager.disconnect(&peer);
    assert_eq!(manager.connection_count(), 0);
    assert_eq!(manager.get_status(&peer), None);
}

#[tokio::test]
async fn test_connection_metrics() {
    let manager = ConnectionManager::new(10);
    
    // Add some connections
    for _ in 0..5 {
        manager.connect(PeerId::random()).await.unwrap();
    }
    
    // Update metrics
    manager.update_metrics(1000.0, 50);
    
    let metrics = manager.get_metrics();
    assert_eq!(metrics.active_connections, 5);
    assert!(metrics.messages_per_second > 0.0);
    assert!(metrics.bytes_per_second > 0.0);
}

#[tokio::test]
async fn test_health_check() {
    let manager = ConnectionManager::new(10);
    let peer1 = PeerId::random();
    let peer2 = PeerId::random();
    
    manager.connect(peer1).await.unwrap();
    manager.connect(peer2).await.unwrap();
    
    // Set one as healthy, one as unhealthy
    manager.update_status(peer1, ConnectionStatus::Connected);
    manager.update_status(peer2, ConnectionStatus::Disconnected);
    
    let health = manager.health_check();
    assert_eq!(health.total_connections, 2);
    assert_eq!(health.healthy_connections, 1);
    assert_eq!(health.unhealthy_connections.len(), 1);
}

#[tokio::test]
async fn test_circuit_breaker_basic() {
    let breaker = CircuitBreaker::new(3, Duration::from_secs(1), Duration::from_secs(5));
    
    // Initial state should be closed
    assert!(matches!(breaker.state(), CircuitBreakerState::Closed));
    assert!(breaker.can_proceed());
    
    // Record failures
    for _ in 0..3 {
        breaker.record_failure();
    }
    
    // Should be open now
    assert!(matches!(breaker.state(), CircuitBreakerState::Open));
    assert!(!breaker.can_proceed());
}

#[tokio::test]
async fn test_circuit_breaker_recovery() {
    let breaker = CircuitBreaker::new(2, Duration::from_millis(100), Duration::from_millis(200));
    
    // Trigger open state
    breaker.record_failure();
    breaker.record_failure();
    assert!(matches!(breaker.state(), CircuitBreakerState::Open));
    
    // Wait for recovery window
    sleep(Duration::from_millis(250)).await;
    
    // Should be half-open
    assert!(matches!(breaker.state(), CircuitBreakerState::HalfOpen));
    assert!(breaker.can_proceed());
    
    // Record success to close
    breaker.record_success();
    assert!(matches!(breaker.state(), CircuitBreakerState::Closed));
}

#[tokio::test]
async fn test_circuit_breaker_half_open_failure() {
    let breaker = CircuitBreaker::new(1, Duration::from_millis(50), Duration::from_millis(100));
    
    // Open the breaker
    breaker.record_failure();
    assert!(matches!(breaker.state(), CircuitBreakerState::Open));
    
    // Wait for half-open
    sleep(Duration::from_millis(150)).await;
    assert!(matches!(breaker.state(), CircuitBreakerState::HalfOpen));
    
    // Fail again
    breaker.record_failure();
    assert!(matches!(breaker.state(), CircuitBreakerState::Open));
}

#[tokio::test]
async fn test_secure_connection_creation() {
    let config = SecureConfig {
        enable_encryption: true,
        enable_authentication: true,
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };
    
    let peer = PeerId::random();
    let conn = SecureConnection::new(peer, config.clone());
    
    assert_eq!(conn.peer_id(), peer);
    assert_eq!(conn.config().enable_encryption, true);
    assert!(conn.is_secure());
}

#[tokio::test]
async fn test_transport_keys_generation() {
    let keys1 = TransportKeys::generate();
    let keys2 = TransportKeys::generate();
    
    // Keys should be unique
    assert_ne!(keys1.public_key(), keys2.public_key());
    assert_ne!(keys1.private_key(), keys2.private_key());
    
    // Key lengths should be correct
    assert_eq!(keys1.public_key().len(), 32);
    assert_eq!(keys1.private_key().len(), 32);
}

#[tokio::test]
async fn test_connection_info() {
    let peer = PeerId::random();
    let info = ConnectionInfo::new(peer, "127.0.0.1:8080".to_string());
    
    assert_eq!(info.peer_id(), peer);
    assert_eq!(info.address(), "127.0.0.1:8080");
    assert!(info.connected_at().elapsed() < Duration::from_secs(1));
    assert_eq!(info.bytes_sent(), 0);
    assert_eq!(info.bytes_received(), 0);
    
    // Update stats
    info.update_sent(1000);
    info.update_received(2000);
    assert_eq!(info.bytes_sent(), 1000);
    assert_eq!(info.bytes_received(), 2000);
}

#[tokio::test]
async fn test_concurrent_connection_management() {
    let manager = Arc::new(ConnectionManager::new(100));
    let mut handles = vec![];
    
    // Spawn multiple tasks to add connections
    for i in 0..50 {
        let mgr = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let peer = PeerId::from_bytes([i as u8; 32]);
            mgr.connect(peer).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    let successful = results.iter().filter(|r| r.is_ok()).count();
    
    assert_eq!(successful, 50);
    assert_eq!(manager.connection_count(), 50);
}

#[tokio::test]
async fn test_connection_manager_stress() {
    let manager = Arc::new(ConnectionManager::new(1000));
    let mut handles = vec![];
    
    // Mix of connects, disconnects, and status updates
    for i in 0..100 {
        let mgr = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let peer = PeerId::from_bytes([i as u8; 32]);
            
            // Connect
            mgr.connect(peer).await.unwrap();
            
            // Random operations
            for _ in 0..10 {
                match i % 3 {
                    0 => mgr.update_status(peer, ConnectionStatus::Connected),
                    1 => mgr.update_metrics(100.0, 10),
                    _ => { let _ = mgr.get_status(&peer); }
                }
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
            
            // Maybe disconnect
            if i % 2 == 0 {
                mgr.disconnect(&peer);
            }
        });
        handles.push(handle);
    }
    
    // Wait for completion
    timeout(Duration::from_secs(10), futures::future::join_all(handles))
        .await
        .expect("Stress test timed out");
    
    // Verify state consistency
    let health = manager.health_check();
    assert!(health.total_connections <= 100);
    assert!(health.healthy_connections <= health.total_connections);
}

#[tokio::test]
async fn test_connection_manager_edge_cases() {
    let manager = ConnectionManager::new(5);
    
    // Test double connect
    let peer = PeerId::random();
    assert!(manager.connect(peer).await.is_ok());
    assert!(manager.connect(peer).await.is_ok()); // Should be idempotent
    assert_eq!(manager.connection_count(), 1);
    
    // Test double disconnect
    manager.disconnect(&peer);
    manager.disconnect(&peer); // Should be safe
    assert_eq!(manager.connection_count(), 0);
    
    // Test operations on non-existent peer
    let unknown = PeerId::random();
    assert_eq!(manager.get_status(&unknown), None);
    manager.update_status(unknown, ConnectionStatus::Connected); // Should be safe
    manager.disconnect(&unknown); // Should be safe
}