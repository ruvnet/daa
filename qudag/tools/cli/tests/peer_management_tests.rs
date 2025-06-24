//! Comprehensive tests for peer management commands
//! Following TDD RED phase - these tests should fail initially

use assert_cmd::Command;
use predicates::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Mock peer information for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockPeerInfo {
    pub peer_id: String,
    pub address: String,
    pub connected_at: u64,
    pub last_seen: u64,
    pub status: PeerStatus,
    pub latency_ms: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum PeerStatus {
    Connected,
    Connecting,
    Disconnected,
    Banned,
}

/// Mock peer manager for testing
struct MockPeerManager {
    peers: Arc<RwLock<HashMap<String, MockPeerInfo>>>,
    max_peers: usize,
    auto_disconnect: bool,
}

impl MockPeerManager {
    fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            max_peers: 50,
            auto_disconnect: false,
        }
    }

    async fn add_peer(&self, address: &str) -> Result<String, String> {
        let mut peers = self.peers.write().await;

        // Check if already connected
        if peers
            .values()
            .any(|p| p.address == address && p.status == PeerStatus::Connected)
        {
            return Err("Peer already connected".to_string());
        }

        // Check max peers limit
        if peers.len() >= self.max_peers {
            return Err("Maximum peer limit reached".to_string());
        }

        // Validate address format
        if !Self::validate_address(address) {
            return Err("Invalid peer address format".to_string());
        }

        let peer_id = format!("peer_{}", uuid::Uuid::new_v4().simple());
        let peer_info = MockPeerInfo {
            peer_id: peer_id.clone(),
            address: address.to_string(),
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: PeerStatus::Connected,
            latency_ms: 50,
            bytes_sent: 0,
            bytes_received: 0,
        };

        peers.insert(peer_id.clone(), peer_info);
        Ok(peer_id)
    }

    async fn remove_peer(&self, peer_id: &str) -> Result<(), String> {
        let mut peers = self.peers.write().await;

        match peers.get_mut(peer_id) {
            Some(peer) => {
                // Graceful disconnection
                peer.status = PeerStatus::Disconnected;
                // In real implementation, would send disconnect message
                peers.remove(peer_id);
                Ok(())
            }
            None => Err("Peer not found".to_string()),
        }
    }

    async fn list_peers(&self) -> Vec<MockPeerInfo> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    async fn get_peer(&self, peer_id: &str) -> Option<MockPeerInfo> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }

    fn validate_address(address: &str) -> bool {
        // Check various address formats
        // IPv4:port
        if let Ok(_) = address.parse::<std::net::SocketAddrV4>() {
            return true;
        }

        // IPv6:port
        if let Ok(_) = address.parse::<std::net::SocketAddrV6>() {
            return true;
        }

        // Domain:port
        if address.contains(':') {
            let parts: Vec<&str> = address.split(':').collect();
            if parts.len() == 2 {
                if let Ok(port) = parts[1].parse::<u16>() {
                    if port > 0 && !parts[0].is_empty() {
                        return true;
                    }
                }
            }
        }

        // .onion address (Tor)
        if address.ends_with(".onion") && address.contains(':') {
            return true;
        }

        // .dark address (QuDAG dark addressing)
        if address.ends_with(".dark") {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 'qudag peer list' command with no peers
    #[test]
    fn test_peer_list_empty() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No peers currently connected"));
    }

    /// Test 'qudag peer list' command with multiple peers
    #[tokio::test]
    async fn test_peer_list_with_peers() {
        // This test assumes we have a mock RPC server running
        // In RED phase, this should fail
        let mut cmd = Command::cargo_bin("qudag").unwrap();

        // First add some peers (this will fail in RED phase)
        cmd.args(&["peer", "add", "192.168.1.100:8000"])
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "example.com:8001"])
            .assert()
            .success();

        // Now list peers
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Connected Peers:"))
            .stdout(predicate::str::contains("192.168.1.100:8000"))
            .stdout(predicate::str::contains("example.com:8001"))
            .stdout(predicate::str::contains("Status: Connected"))
            .stdout(predicate::str::contains("Latency:"))
            .stdout(predicate::str::contains("Data transferred:"));
    }

    /// Test 'qudag peer list' with JSON output format
    #[test]
    fn test_peer_list_json_format() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "list", "--format", "json"])
            .assert()
            .success()
            .stdout(predicate::str::is_match(r#"\{"peers":\s*\[\]\}"#).unwrap());
    }

    /// Test 'qudag peer add' with valid IPv4 address
    #[test]
    fn test_peer_add_ipv4() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100:8000"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Adding peer: 192.168.1.100:8000"))
            .stdout(predicate::str::contains("Successfully connected to peer"));
    }

    /// Test 'qudag peer add' with valid IPv6 address
    #[test]
    fn test_peer_add_ipv6() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "[2001:db8::1]:8000"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Adding peer: [2001:db8::1]:8000"))
            .stdout(predicate::str::contains("Successfully connected to peer"));
    }

    /// Test 'qudag peer add' with domain name
    #[test]
    fn test_peer_add_domain() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "node1.qudag.network:8000"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Adding peer: node1.qudag.network:8000",
            ))
            .stdout(predicate::str::contains("Successfully connected to peer"));
    }

    /// Test 'qudag peer add' with .onion address (Tor)
    #[test]
    fn test_peer_add_onion() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "3g2upl4pq6kufc4m.onion:8000"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Adding peer: 3g2upl4pq6kufc4m.onion:8000",
            ))
            .stdout(predicate::str::contains(
                "Successfully connected to peer via Tor",
            ));
    }

    /// Test 'qudag peer add' with .dark address (QuDAG dark addressing)
    #[test]
    fn test_peer_add_dark_address() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "mynode.dark"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Adding peer: mynode.dark"))
            .stdout(predicate::str::contains("Resolving dark address"))
            .stdout(predicate::str::contains("Successfully connected to peer"));
    }

    /// Test 'qudag peer add' with invalid address format
    #[test]
    fn test_peer_add_invalid_address() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "invalid-address"])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Error: Invalid peer address format",
            ));
    }

    /// Test 'qudag peer add' with missing port
    #[test]
    fn test_peer_add_missing_port() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: Port number required"));
    }

    /// Test 'qudag peer add' with invalid port
    #[test]
    fn test_peer_add_invalid_port() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100:99999"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: Invalid port number"));
    }

    /// Test adding duplicate peer
    #[test]
    fn test_peer_add_duplicate() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100:8000"])
            .assert()
            .success();

        // Try to add the same peer again
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100:8000"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: Peer already connected"));
    }

    /// Test maximum peer limit
    #[tokio::test]
    async fn test_peer_add_max_limit() {
        // Add peers up to the limit (assuming limit is 50)
        for i in 1..=50 {
            let mut cmd = Command::cargo_bin("qudag").unwrap();
            cmd.args(&["peer", "add", &format!("192.168.1.{}:8000", i)])
                .assert()
                .success();
        }

        // Try to add one more
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.51:8000"])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Error: Maximum peer limit reached",
            ));
    }

    /// Test 'qudag peer remove' with valid peer
    #[test]
    fn test_peer_remove_valid() {
        // First add a peer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        let output = cmd
            .args(&["peer", "add", "192.168.1.100:8000"])
            .output()
            .unwrap();

        // Extract peer ID from output (assuming format: "Connected with peer ID: <id>")
        let stdout = String::from_utf8_lossy(&output.stdout);
        let peer_id = stdout
            .lines()
            .find(|line| line.contains("peer ID:"))
            .and_then(|line| line.split("peer ID:").nth(1))
            .map(|s| s.trim())
            .unwrap_or("peer_12345");

        // Now remove the peer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "remove", peer_id])
            .assert()
            .success()
            .stdout(predicate::str::contains("Removing peer:"))
            .stdout(predicate::str::contains("Gracefully disconnecting"))
            .stdout(predicate::str::contains("Peer removed successfully"));
    }

    /// Test 'qudag peer remove' with address instead of ID
    #[test]
    fn test_peer_remove_by_address() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "remove", "192.168.1.100:8000"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Removing peer: 192.168.1.100:8000",
            ))
            .stdout(predicate::str::contains("Peer removed successfully"));
    }

    /// Test 'qudag peer remove' with non-existent peer
    #[test]
    fn test_peer_remove_nonexistent() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "remove", "nonexistent_peer_id"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: Peer not found"));
    }

    /// Test removing peer with active connections
    #[test]
    fn test_peer_remove_with_active_connections() {
        // Add a peer and simulate active data transfer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100:8000"])
            .assert()
            .success();

        // Remove with --force flag
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "remove", "192.168.1.100:8000", "--force"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Force disconnecting peer"))
            .stdout(predicate::str::contains("Active connections terminated"));
    }

    /// Test concurrent peer operations
    #[tokio::test]
    async fn test_concurrent_peer_operations() {
        use tokio::task::JoinSet;

        let mut tasks = JoinSet::new();

        // Spawn multiple add operations concurrently
        for i in 0..10 {
            tasks.spawn(async move {
                let mut cmd = Command::cargo_bin("qudag").unwrap();
                cmd.args(&["peer", "add", &format!("192.168.1.{}:8000", i)])
                    .assert()
                    .success()
            });
        }

        // Spawn list operations concurrently
        for _ in 0..5 {
            tasks.spawn(async {
                let mut cmd = Command::cargo_bin("qudag").unwrap();
                cmd.args(&["peer", "list"]).assert().success()
            });
        }

        // Wait for all operations to complete
        while let Some(result) = tasks.join_next().await {
            assert!(result.is_ok());
        }

        // Verify final state
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Connected Peers:"));
    }

    /// Test peer reconnection after removal
    #[test]
    fn test_peer_reconnection() {
        let address = "192.168.1.100:8000";

        // Add peer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", address]).assert().success();

        // Remove peer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "remove", address]).assert().success();

        // Add peer again
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", address])
            .assert()
            .success()
            .stdout(predicate::str::contains("Successfully connected to peer"));
    }

    /// Test peer ban functionality
    #[test]
    fn test_peer_ban() {
        let address = "192.168.1.100:8000";

        // Ban a peer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "ban", address])
            .assert()
            .success()
            .stdout(predicate::str::contains("Peer banned:"));

        // Try to add banned peer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", address])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: Peer is banned"));
    }

    /// Test peer statistics
    #[test]
    fn test_peer_stats() {
        // Add a peer first
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100:8000"])
            .assert()
            .success();

        // Get peer statistics
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "stats", "192.168.1.100:8000"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Peer Statistics"))
            .stdout(predicate::str::contains("Connected since:"))
            .stdout(predicate::str::contains("Latency:"))
            .stdout(predicate::str::contains("Bytes sent:"))
            .stdout(predicate::str::contains("Bytes received:"))
            .stdout(predicate::str::contains("Message count:"));
    }

    /// Test filtering peers by status
    #[test]
    fn test_peer_list_filter_by_status() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "list", "--status", "connected"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Connected Peers:"));

        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "list", "--status", "disconnected"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Disconnected Peers:"));
    }

    /// Test peer timeout handling
    #[tokio::test]
    async fn test_peer_timeout() {
        // Add a peer
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "192.168.1.100:8000", "--timeout", "5"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Connection timeout set to 5 seconds",
            ));

        // Simulate timeout scenario
        tokio::time::sleep(Duration::from_secs(6)).await;

        // Check peer status
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Status: Disconnected (timeout)"));
    }

    /// Test batch peer operations
    #[test]
    fn test_peer_batch_add() {
        // Add multiple peers from file
        let peer_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            peer_file.path(),
            "192.168.1.100:8000\n\
             192.168.1.101:8000\n\
             example.com:8001\n\
             mynode.dark\n",
        )
        .unwrap();

        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&["peer", "add", "--file", peer_file.path().to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains("Adding 4 peers from file"))
            .stdout(predicate::str::contains("Successfully added: 4"))
            .stdout(predicate::str::contains("Failed: 0"));
    }

    /// Test peer export functionality
    #[test]
    fn test_peer_export() {
        let export_file = tempfile::NamedTempFile::new().unwrap();

        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.args(&[
            "peer",
            "export",
            "--output",
            export_file.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported peer list to"));

        // Verify file content
        let content = std::fs::read_to_string(export_file.path()).unwrap();
        assert!(content.contains("peers") || content.contains("[]"));
    }

    /// Test mock peer manager directly
    #[tokio::test]
    async fn test_mock_peer_manager() {
        let manager = MockPeerManager::new();

        // Test adding valid peer
        let peer_id = manager.add_peer("192.168.1.100:8000").await.unwrap();
        assert!(!peer_id.is_empty());

        // Test listing peers
        let peers = manager.list_peers().await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].address, "192.168.1.100:8000");

        // Test duplicate peer
        let result = manager.add_peer("192.168.1.100:8000").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Peer already connected");

        // Test removing peer
        let result = manager.remove_peer(&peer_id).await;
        assert!(result.is_ok());

        // Verify peer was removed
        let peers = manager.list_peers().await;
        assert_eq!(peers.len(), 0);
    }

    /// Test address validation
    #[test]
    fn test_address_validation() {
        assert!(MockPeerManager::validate_address("192.168.1.100:8000"));
        assert!(MockPeerManager::validate_address("[2001:db8::1]:8000"));
        assert!(MockPeerManager::validate_address("example.com:8000"));
        assert!(MockPeerManager::validate_address(
            "3g2upl4pq6kufc4m.onion:8000"
        ));
        assert!(MockPeerManager::validate_address("mynode.dark"));

        assert!(!MockPeerManager::validate_address("invalid"));
        assert!(!MockPeerManager::validate_address("192.168.1.100"));
        assert!(!MockPeerManager::validate_address(":8000"));
        assert!(!MockPeerManager::validate_address("example.com:"));
        assert!(!MockPeerManager::validate_address("example.com:99999"));
    }
}

/// Performance benchmarks for peer operations
#[cfg(all(test, not(debug_assertions)))]
mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    fn bench_peer_add(c: &mut Criterion) {
        c.bench_function("peer_add", |b| {
            b.iter(|| {
                let mut cmd = Command::cargo_bin("qudag").unwrap();
                cmd.args(&["peer", "add", black_box("192.168.1.100:8000")])
                    .output()
                    .unwrap();
            });
        });
    }

    fn bench_peer_list(c: &mut Criterion) {
        c.bench_function("peer_list", |b| {
            b.iter(|| {
                let mut cmd = Command::cargo_bin("qudag").unwrap();
                cmd.args(&["peer", "list"]).output().unwrap();
            });
        });
    }
}
