use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_node_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = NodeManagerConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = NodeManager::new(config).unwrap();
    assert!(!manager.is_running().await);
}

#[tokio::test]
async fn test_node_status_when_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let config = NodeManagerConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = NodeManager::new(config).unwrap();
    let status = manager.get_status().await.unwrap();

    assert!(!status.is_running);
    assert!(status.pid.is_none());
    assert!(status.uptime_seconds.is_none());
}

#[tokio::test]
async fn test_systemd_service_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config = NodeManagerConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = NodeManager::new(config).unwrap();
    let service_content = manager.generate_systemd_service(None).await.unwrap();

    assert!(service_content.contains("[Unit]"));
    assert!(service_content.contains("[Service]"));
    assert!(service_content.contains("[Install]"));
    assert!(service_content.contains("ExecStart"));
    assert!(service_content.contains("ExecStop"));
}

#[tokio::test]
async fn test_log_rotation() {
    let temp_dir = TempDir::new().unwrap();
    let config = NodeManagerConfig {
        base_dir: temp_dir.path().to_path_buf(),
        log_rotation_size_mb: 1, // Small size for testing
        max_log_files: 3,
        ..Default::default()
    };

    let manager = NodeManager::new(config).unwrap();

    // Create a large log file
    let log_file = temp_dir.path().join("qudag.log");
    let large_content = "x".repeat(2 * 1024 * 1024); // 2MB
    std::fs::write(&log_file, large_content).unwrap();

    // Rotate logs
    manager.rotate_logs().await.unwrap();

    // Check that log was rotated
    assert!(temp_dir.path().join("qudag.log.1").exists());
    assert!(!log_file.exists() || std::fs::metadata(&log_file).unwrap().len() == 0);
}

#[tokio::test]
async fn test_stop_node_when_not_running() {
    let temp_dir = TempDir::new().unwrap();
    let config = NodeManagerConfig {
        base_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = NodeManager::new(config).unwrap();

    // Should return error when trying to stop non-running node
    let result = manager.stop_node(false).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No node is currently running"));
}
