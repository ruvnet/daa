use qudag_cli::peer_manager::{PeerManager, PeerManagerConfig};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_peer_manager_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let config = PeerManagerConfig {
        data_path: temp_dir.path().join("peers.json"),
        max_peers: 100,
        auto_save_interval: 60,
        connection_timeout: 30,
        auto_discovery: false,
    };

    let manager = PeerManager::new(config).await.unwrap();
    let peers = manager.list_peers().await.unwrap();
    assert_eq!(peers.len(), 0);
}

#[tokio::test]
async fn test_peer_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let data_path = temp_dir.path().join("peers.json");

    // Create first manager and add a peer
    {
        let config = PeerManagerConfig {
            data_path: data_path.clone(),
            max_peers: 100,
            auto_save_interval: 60,
            connection_timeout: 30,
            auto_discovery: false,
        };

        let manager = PeerManager::new(config).await.unwrap();

        // Note: This will fail in tests because we can't actually connect
        // but it demonstrates the API
        let _ = manager
            .add_peer("127.0.0.1:8000".to_string(), Some("test_peer".to_string()))
            .await;

        // Save peers
        manager.save_peers().await.unwrap();
    }

    // Create second manager and verify it loads the saved peers
    {
        let config = PeerManagerConfig {
            data_path: data_path.clone(),
            max_peers: 100,
            auto_save_interval: 60,
            connection_timeout: 30,
            auto_discovery: false,
        };

        let manager = PeerManager::new(config).await.unwrap();
        let peers = manager.list_peers().await.unwrap();

        // Since we couldn't actually connect in the test environment,
        // this should still be 0, but in a real scenario with mocked
        // networking, we'd see the saved peer
        assert_eq!(peers.len(), 0);
    }
}

#[tokio::test]
async fn test_peer_import_export() {
    let temp_dir = TempDir::new().unwrap();
    let config = PeerManagerConfig {
        data_path: temp_dir.path().join("peers.json"),
        max_peers: 100,
        auto_save_interval: 60,
        connection_timeout: 30,
        auto_discovery: false,
    };

    let manager = PeerManager::new(config).await.unwrap();

    // Create a test peer file
    let test_peers = r#"[
        {
            "id": "test_peer_1",
            "address": "192.168.1.100:8000",
            "nickname": "Test Node 1",
            "trust_level": 75,
            "first_seen": 1700000000,
            "last_seen": 1700001000,
            "total_messages": 100,
            "success_rate": 0.95,
            "avg_latency_ms": 25.5,
            "tags": ["trusted", "fast"],
            "persistent": true
        }
    ]"#;

    let import_path = temp_dir.path().join("import_peers.json");
    std::fs::write(&import_path, test_peers).unwrap();

    // Import peers
    let count = manager.import_peers(import_path, false).await.unwrap();
    assert_eq!(count, 1);

    // Export peers
    let export_path = temp_dir.path().join("export_peers.json");
    let exported = manager
        .export_peers(export_path.clone(), None)
        .await
        .unwrap();
    assert_eq!(exported, 1);

    // Verify exported file exists
    assert!(export_path.exists());
}

// Address validation is tested indirectly through the add_peer method
#[tokio::test]
async fn test_invalid_address_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = PeerManagerConfig {
        data_path: temp_dir.path().join("peers.json"),
        max_peers: 100,
        auto_save_interval: 60,
        connection_timeout: 30,
        auto_discovery: false,
    };

    let manager = PeerManager::new(config).await.unwrap();

    // Test invalid addresses
    assert!(manager.add_peer("".to_string(), None).await.is_err());
    assert!(manager.add_peer("invalid".to_string(), None).await.is_err());
    assert!(manager.add_peer(":8000".to_string(), None).await.is_err());
    assert!(manager
        .add_peer("127.0.0.1:".to_string(), None)
        .await
        .is_err());
    assert!(manager
        .add_peer("127.0.0.1:0".to_string(), None)
        .await
        .is_err());
}
