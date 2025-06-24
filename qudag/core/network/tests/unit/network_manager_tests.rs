//! Unit tests for NetworkManager

use qudag_network::{
    NetworkConfig, NetworkManager, NetworkEvent, PeerMetadata, ReputationManager, NetworkStats,
    types::{NetworkError, PeerId as OurPeerId},
};
use libp2p::PeerId as LibP2PPeerId;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn test_network_manager_initialization() {
    let mut manager = NetworkManager::new();
    assert!(manager.initialize().await.is_ok());
    assert!(manager.local_peer_id().is_some());
}

#[tokio::test]
async fn test_network_manager_with_config() {
    let config = NetworkConfig {
        max_connections: 20,
        connection_timeout: Duration::from_secs(10),
        discovery_interval: Duration::from_secs(30),
        bootstrap_peers: vec!["peer1".to_string(), "peer2".to_string()],
        enable_dht: false,
        quantum_resistant: true,
    };
    
    let manager = NetworkManager::with_config(config.clone());
    assert_eq!(manager.config.max_connections, 20);
    assert_eq!(manager.config.bootstrap_peers.len(), 2);
}

#[tokio::test]
async fn test_peer_connection_lifecycle() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    // Mock peer connection
    let peer_address = "127.0.0.1:8080";
    let result = manager.connect_peer(peer_address).await;
    
    // Should return a peer ID (mocked in current implementation)
    assert!(result.is_ok());
    let peer_id = result.unwrap();
    
    // Check if peer is in connected list
    let connected_peers = manager.get_connected_peers().await;
    assert!(connected_peers.contains(&peer_id));
    
    // Test disconnection
    assert!(manager.disconnect_peer(&peer_id).await.is_ok());
    
    // Verify peer is removed
    let connected_peers = manager.get_connected_peers().await;
    assert!(!connected_peers.contains(&peer_id));
}

#[tokio::test]
async fn test_message_sending() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    // Connect a peer first
    let peer_id = manager.connect_peer("127.0.0.1:8080").await.unwrap();
    
    // Send message
    let message = vec![1, 2, 3, 4, 5];
    assert!(manager.send_message(&peer_id, message.clone()).await.is_ok());
    
    // Test sending to non-connected peer
    let unknown_peer = LibP2PPeerId::random();
    let result = manager.send_message(&unknown_peer, message).await;
    assert!(matches!(result, Err(NetworkError::ConnectionError(_))));
}

#[tokio::test]
async fn test_peer_metadata_tracking() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    let peer_id = manager.connect_peer("127.0.0.1:8080").await.unwrap();
    
    // Check metadata exists
    let metadata = manager.get_peer_metadata(&peer_id).await;
    assert!(metadata.is_some());
    
    let meta = metadata.unwrap();
    assert_eq!(meta.protocol_version, 1);
    assert_eq!(meta.reputation, 0.0);
    assert_eq!(meta.latency_ms, 0);
}

#[tokio::test]
async fn test_network_stats() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    // Connect some peers
    let peer1 = manager.connect_peer("127.0.0.1:8080").await.unwrap();
    let peer2 = manager.connect_peer("127.0.0.1:8081").await.unwrap();
    
    // Add one to trusted
    manager.add_trusted_peer(peer1).await;
    
    let stats = manager.get_network_stats().await;
    assert_eq!(stats.connected_peers, 2);
    assert_eq!(stats.trusted_peers, 1);
    assert_eq!(stats.blacklisted_peers, 0);
}

#[tokio::test]
async fn test_blacklist_functionality() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    let peer_id = manager.connect_peer("127.0.0.1:8080").await.unwrap();
    
    // Blacklist the peer
    manager.blacklist_peer(peer_id).await;
    
    // Check stats
    let stats = manager.get_network_stats().await;
    assert_eq!(stats.blacklisted_peers, 1);
    assert_eq!(stats.connected_peers, 0); // Should be disconnected
    
    // Try to reconnect - should fail
    let result = manager.connect_peer("127.0.0.1:8080").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_maintenance_inactive_peer_cleanup() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    // Connect peers
    let _peer1 = manager.connect_peer("127.0.0.1:8080").await.unwrap();
    let _peer2 = manager.connect_peer("127.0.0.1:8081").await.unwrap();
    
    assert_eq!(manager.get_connected_peers().await.len(), 2);
    
    // Simulate time passing (in real scenario, would need to mock time)
    // For now, just run maintenance
    manager.maintenance().await;
    
    // Peers should still be connected (within timeout)
    assert_eq!(manager.get_connected_peers().await.len(), 2);
}

#[tokio::test]
async fn test_discovery_start() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    // Start discovery should succeed
    assert!(manager.start_discovery().await.is_ok());
}

#[tokio::test]
async fn test_shutdown() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    // Connect some peers
    let _peer1 = manager.connect_peer("127.0.0.1:8080").await.unwrap();
    let _peer2 = manager.connect_peer("127.0.0.1:8081").await.unwrap();
    
    // Shutdown
    assert!(manager.shutdown().await.is_ok());
    
    // All peers should be disconnected
    assert_eq!(manager.get_connected_peers().await.len(), 0);
}

#[tokio::test]
async fn test_concurrent_peer_connections() {
    let mut manager = NetworkManager::new();
    manager.initialize().await.unwrap();
    
    // Spawn multiple concurrent connections
    let mut handles = vec![];
    let manager_arc = std::sync::Arc::new(manager);
    
    for i in 0..10 {
        let mgr = manager_arc.clone();
        let handle = tokio::spawn(async move {
            mgr.connect_peer(&format!("127.0.0.1:{}", 8080 + i)).await
        });
        handles.push(handle);
    }
    
    // Wait for all connections
    let results: Vec<_> = futures::future::join_all(handles).await;
    let successful_connections = results.iter().filter(|r| r.is_ok()).count();
    
    assert!(successful_connections > 0);
}

#[tokio::test]
async fn test_network_event_handling() {
    // Create channel for testing
    let (tx, mut rx) = mpsc::channel(100);
    
    // Send test events
    let peer_id = LibP2PPeerId::random();
    tx.send(NetworkEvent::PeerConnected(peer_id)).await.unwrap();
    tx.send(NetworkEvent::MessageReceived { from: peer_id, data: vec![1, 2, 3] }).await.unwrap();
    tx.send(NetworkEvent::PeerDisconnected(peer_id)).await.unwrap();
    
    // Verify events are received
    assert!(matches!(rx.recv().await, Some(NetworkEvent::PeerConnected(_))));
    assert!(matches!(rx.recv().await, Some(NetworkEvent::MessageReceived { .. })));
    assert!(matches!(rx.recv().await, Some(NetworkEvent::PeerDisconnected(_))));
}

#[cfg(test)]
mod reputation_manager_tests {
    use super::*;
    
    #[test]
    fn test_reputation_scoring() {
        let mut manager = ReputationManager::default();
        let peer_id = LibP2PPeerId::random();
        
        // Initial reputation should be 0
        assert_eq!(manager.get_reputation(&peer_id), 0.0);
        
        // Update reputation
        manager.update_reputation(peer_id, 10.0);
        assert_eq!(manager.get_reputation(&peer_id), 10.0);
        
        // Test clamping
        manager.update_reputation(peer_id, 200.0);
        assert_eq!(manager.get_reputation(&peer_id), 100.0); // Should be clamped to 100
    }
    
    #[test]
    fn test_auto_blacklisting() {
        let mut manager = ReputationManager::default();
        let peer_id = LibP2PPeerId::random();
        
        // Lower reputation below threshold
        manager.update_reputation(peer_id, -60.0);
        assert!(manager.is_blacklisted(&peer_id));
    }
    
    #[test]
    fn test_trusted_peers() {
        let mut manager = ReputationManager::default();
        let peer_id = LibP2PPeerId::random();
        
        manager.add_trusted(peer_id);
        assert!(manager.is_trusted(&peer_id));
        assert_eq!(manager.get_reputation(&peer_id), 75.0);
    }
    
    #[test]
    fn test_blacklist_expiry_cleanup() {
        let mut manager = ReputationManager::default();
        let peer_id = LibP2PPeerId::random();
        
        // Add to blacklist
        manager.update_reputation(peer_id, -100.0);
        assert!(manager.is_blacklisted(&peer_id));
        
        // Cleanup (won't remove recent entries)
        manager.cleanup_expired();
        assert!(manager.is_blacklisted(&peer_id)); // Should still be blacklisted
    }
}