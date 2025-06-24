use anyhow::Result;
use qudag_cli::{
    rpc::{is_node_running, RpcClient},
    CommandRouter,
};
use qudag_protocol::rpc_server::RpcServer;
use std::time::Duration;
use tokio::time::sleep;

/// Integration tests for RPC functionality
/// These tests verify the end-to-end communication between CLI and RPC server

#[tokio::test]
async fn test_peer_management_integration() -> Result<()> {
    // Start a test RPC server
    let (mut server, mut command_rx) = RpcServer::new_tcp(8900);

    // Start the server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Handle commands in background
    let command_handle = tokio::spawn(async move {
        while let Some((command, response_tx)) = command_rx.recv().await {
            use qudag_protocol::rpc_server::RpcCommand;
            let response = match command {
                RpcCommand::GetStatus => serde_json::json!({
                    "node_id": "test_node",
                    "state": "Running",
                    "uptime": 0,
                    "peers": [],
                    "network_stats": {
                        "total_connections": 0,
                        "active_connections": 0,
                        "messages_sent": 0,
                        "messages_received": 0,
                        "bytes_sent": 0,
                        "bytes_received": 0,
                        "average_latency": 0.0,
                        "uptime": 0
                    },
                    "dag_stats": {
                        "vertex_count": 0,
                        "edge_count": 0,
                        "tip_count": 0,
                        "finalized_height": 0,
                        "pending_transactions": 0
                    },
                    "memory_usage": {
                        "total_allocated": 0,
                        "current_usage": 0,
                        "peak_usage": 0
                    }
                }),
                _ => serde_json::json!({"status": "ok"}),
            };
            let _ = response_tx.send(response);
        }
    });

    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    // Create a command router instance
    let router = CommandRouter::new();

    // Test peer list command (should work even with no peers)
    let result = router.handle_peer_list(Some(8900)).await;
    assert!(result.is_ok());

    // Test adding a peer
    let result = router
        .handle_peer_add("127.0.0.1:8001".to_string(), Some(8900), None)
        .await;
    assert!(result.is_ok());

    // Test network stats
    let result = router.handle_network_stats(Some(8900), false).await;
    assert!(result.is_ok());

    // Clean up
    drop(server_handle);
    drop(command_handle);

    Ok(())
}

#[tokio::test]
async fn test_rpc_client_connectivity() -> Result<()> {
    // Test with non-existent server
    let client =
        RpcClient::new_tcp("127.0.0.1".to_string(), 9999).with_timeout(Duration::from_millis(100));

    let result = client.list_peers().await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_peer_command_validation() -> Result<()> {
    // Create a command router instance
    let router = CommandRouter::new();

    // Test invalid address format
    let result = router
        .handle_peer_add("invalid_address".to_string(), Some(8901), None)
        .await;
    assert!(result.is_err());

    // Test valid address format (even though connection will fail)
    let result = router
        .handle_peer_add("127.0.0.1:8001".to_string(), Some(8901), None)
        .await;
    // This should fail due to connection error, not validation error
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to add peer"));

    Ok(())
}

#[tokio::test]
async fn test_node_running_check() {
    // Test with non-existent port
    assert!(!is_node_running(9998).await);

    // Test with invalid port
    assert!(!is_node_running(0).await);
}

#[tokio::test]
async fn test_rpc_timeout_handling() -> Result<()> {
    let client =
        RpcClient::new_tcp("127.0.0.1".to_string(), 9997).with_timeout(Duration::from_millis(10)); // Very short timeout

    let start = std::time::Instant::now();
    let result = client.get_status().await;
    let elapsed = start.elapsed();

    // Should fail quickly due to timeout
    assert!(result.is_err());
    assert!(elapsed < Duration::from_millis(500)); // Should timeout much faster than 500ms

    Ok(())
}

#[tokio::test]
async fn test_concurrent_rpc_requests() -> Result<()> {
    // This test would require a running server, but demonstrates the pattern
    let client =
        RpcClient::new_tcp("127.0.0.1".to_string(), 9996).with_timeout(Duration::from_millis(100));

    // Launch multiple concurrent requests
    let client_clone =
        RpcClient::new_tcp("127.0.0.1".to_string(), 9996).with_timeout(Duration::from_millis(100));

    let (result1, result2) = tokio::join!(client.list_peers(), client_clone.get_network_stats());

    // All should fail (no server), but should handle concurrency properly
    assert!(result1.is_err());
    assert!(result2.is_err());

    Ok(())
}

#[cfg(test)]
mod address_validation_tests {
    use super::*;

    fn is_valid_peer_address(address: &str) -> bool {
        // Copy the validation logic from commands.rs for testing
        if let Some((host, port_str)) = address.rsplit_once(':') {
            if host.is_empty() || port_str.is_empty() {
                return false;
            }

            if let Ok(port) = port_str.parse::<u16>() {
                if port == 0 {
                    return false;
                }
            } else {
                return false;
            }

            if host.parse::<std::net::IpAddr>().is_ok() {
                return true;
            }

            if host.len() <= 253 && !host.is_empty() {
                return host
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '.' || c == '-');
            }
        }

        false
    }

    #[test]
    fn test_valid_addresses() {
        assert!(is_valid_peer_address("127.0.0.1:8000"));
        assert!(is_valid_peer_address("192.168.1.1:9000"));
        assert!(is_valid_peer_address("localhost:8000"));
        assert!(is_valid_peer_address("example.com:443"));
        assert!(is_valid_peer_address("node-1.example.com:8000"));
    }

    #[test]
    fn test_invalid_addresses() {
        assert!(!is_valid_peer_address("127.0.0.1")); // No port
        assert!(!is_valid_peer_address(":8000")); // No host
        assert!(!is_valid_peer_address("127.0.0.1:")); // Empty port
        assert!(!is_valid_peer_address("127.0.0.1:0")); // Invalid port
        assert!(!is_valid_peer_address("127.0.0.1:99999")); // Port too high
        assert!(!is_valid_peer_address("invalid_host:8000")); // Invalid hostname
        assert!(!is_valid_peer_address("")); // Empty string
    }
}
