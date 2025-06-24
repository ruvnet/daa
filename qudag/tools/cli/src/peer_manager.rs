use anyhow::{anyhow, Result};
use qudag_network::{NetworkConfig, NetworkManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

/// Peer information stored persistently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID (string representation)
    pub id: String,
    /// Peer network address
    pub address: String,
    /// Optional nickname for the peer
    pub nickname: Option<String>,
    /// Trust level (0-100)
    pub trust_level: u8,
    /// First seen timestamp
    pub first_seen: u64,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Total messages exchanged
    pub total_messages: u64,
    /// Connection success rate (0.0-1.0)
    pub success_rate: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: Option<f64>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Whether this peer is permanently saved
    pub persistent: bool,
}

impl PeerInfo {
    /// Create a new PeerInfo instance
    pub fn new(id: String, address: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            address,
            nickname: None,
            trust_level: 50, // Start with neutral trust
            first_seen: now,
            last_seen: now,
            total_messages: 0,
            success_rate: 1.0,
            avg_latency_ms: None,
            tags: Vec::new(),
            persistent: false,
        }
    }
}

/// Peer manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerManagerConfig {
    /// Path to store peer data
    pub data_path: PathBuf,
    /// Maximum number of peers to remember
    pub max_peers: usize,
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable auto-discovery
    pub auto_discovery: bool,
}

impl Default for PeerManagerConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let data_path = home_dir.join(".qudag").join("peers.json");

        Self {
            data_path,
            max_peers: 1000,
            auto_save_interval: 300, // 5 minutes
            connection_timeout: 30,
            auto_discovery: true,
        }
    }
}

/// Peer manager for handling peer operations
pub struct PeerManager {
    /// Configuration
    config: PeerManagerConfig,
    /// Known peers
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// Network manager instance
    network_manager: Arc<Mutex<NetworkManager>>,
    /// Last save timestamp
    last_save: Arc<Mutex<SystemTime>>,
}

impl PeerManager {
    /// Create a new peer manager
    pub async fn new(config: PeerManagerConfig) -> Result<Self> {
        // Create data directory if it doesn't exist
        if let Some(parent) = config.data_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing peers from disk
        let peers = Self::load_peers(&config.data_path)?;

        // Initialize network manager
        let network_config = NetworkConfig {
            max_connections: config.max_peers,
            connection_timeout: Duration::from_secs(config.connection_timeout),
            enable_dht: config.auto_discovery,
            ..Default::default()
        };

        let mut network_manager = NetworkManager::with_config(network_config);
        network_manager.initialize().await?;

        Ok(Self {
            config,
            peers: Arc::new(RwLock::new(peers)),
            network_manager: Arc::new(Mutex::new(network_manager)),
            last_save: Arc::new(Mutex::new(SystemTime::now())),
        })
    }

    /// Load peers from disk
    fn load_peers(path: &Path) -> Result<HashMap<String, PeerInfo>> {
        if !path.exists() {
            debug!("No existing peers file found at {:?}", path);
            return Ok(HashMap::new());
        }

        let data = fs::read_to_string(path)?;
        let peers: Vec<PeerInfo> = serde_json::from_str(&data)?;

        let mut peer_map = HashMap::new();
        for peer in peers {
            peer_map.insert(peer.id.clone(), peer);
        }

        info!("Loaded {} peers from disk", peer_map.len());
        Ok(peer_map)
    }

    /// Save peers to disk
    pub async fn save_peers(&self) -> Result<()> {
        let peers = self.peers.read().await;
        let peer_list: Vec<&PeerInfo> = peers.values().collect();

        let data = serde_json::to_string_pretty(&peer_list)?;
        fs::write(&self.config.data_path, data)?;

        *self.last_save.lock().await = SystemTime::now();
        debug!("Saved {} peers to disk", peer_list.len());
        Ok(())
    }

    /// Auto-save if needed
    pub async fn auto_save_if_needed(&self) -> Result<()> {
        let last_save = *self.last_save.lock().await;
        let elapsed = SystemTime::now().duration_since(last_save)?.as_secs();

        if elapsed >= self.config.auto_save_interval {
            self.save_peers().await?;
        }

        Ok(())
    }

    /// Add a new peer
    pub async fn add_peer(&self, address: String, nickname: Option<String>) -> Result<String> {
        // Validate address format
        if !Self::is_valid_address(&address) {
            return Err(anyhow!("Invalid peer address format: {}", address));
        }

        // Connect to peer using network manager
        let network_manager = self.network_manager.lock().await;
        let peer_id = network_manager
            .connect_peer(&address)
            .await
            .map_err(|e| anyhow!("Failed to connect to peer: {}", e))?;

        // Convert libp2p PeerId to string
        let peer_id_str = peer_id.to_string();

        // Create or update peer info
        let mut peer_info = PeerInfo::new(peer_id_str.clone(), address.clone());
        if let Some(nick) = nickname {
            peer_info.nickname = Some(nick);
        }
        peer_info.persistent = true;

        // Store peer info
        {
            let mut peers = self.peers.write().await;
            peers.insert(peer_id_str.clone(), peer_info);
        }

        // Auto-save if needed
        let _ = self.auto_save_if_needed().await;

        info!("Successfully added peer: {} ({})", peer_id_str, address);
        Ok(peer_id_str)
    }

    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: String) -> Result<()> {
        // Parse peer ID to libp2p format
        let libp2p_peer_id = libp2p::PeerId::from_bytes(peer_id.as_bytes())
            .map_err(|_| anyhow!("Invalid peer ID format"))?;

        // Disconnect from peer
        let network_manager = self.network_manager.lock().await;
        network_manager
            .disconnect_peer(&libp2p_peer_id)
            .await
            .map_err(|e| anyhow!("Failed to disconnect peer: {}", e))?;

        // Remove from storage
        {
            let mut peers = self.peers.write().await;
            peers.remove(&peer_id);
        }

        // Auto-save if needed
        let _ = self.auto_save_if_needed().await;

        info!("Successfully removed peer: {}", peer_id);
        Ok(())
    }

    /// List all peers
    pub async fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        // Get connected peers from network manager
        let network_manager = self.network_manager.lock().await;
        let connected_peer_ids = network_manager.get_connected_peers().await;

        // Update peer info with current connection status
        let mut peers = self.peers.write().await;

        for peer_id in connected_peer_ids {
            let peer_id_str = peer_id.to_string();

            // Get metadata from network manager
            if let Some(metadata) = network_manager.get_peer_metadata(&peer_id).await {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Update existing peer or create new one
                match peers.get_mut(&peer_id_str) {
                    Some(peer_info) => {
                        peer_info.last_seen = now;
                        peer_info.avg_latency_ms = Some(metadata.latency_ms as f64);
                    }
                    None => {
                        let mut peer_info =
                            PeerInfo::new(peer_id_str.clone(), metadata.address.clone());
                        peer_info.last_seen = now;
                        peer_info.avg_latency_ms = Some(metadata.latency_ms as f64);
                        peers.insert(peer_id_str, peer_info);
                    }
                }
            }
        }

        // Return all peers
        Ok(peers.values().cloned().collect())
    }

    /// Get detailed information about a specific peer
    pub async fn get_peer_info(&self, peer_id: String) -> Result<PeerInfo> {
        let peers = self.peers.read().await;
        peers
            .get(&peer_id)
            .cloned()
            .ok_or_else(|| anyhow!("Peer not found: {}", peer_id))
    }

    /// Update peer metadata
    pub async fn update_peer_metadata(
        &self,
        peer_id: String,
        nickname: Option<String>,
        trust_level: Option<u8>,
        tags: Option<Vec<String>>,
    ) -> Result<()> {
        let mut peers = self.peers.write().await;

        let peer = peers
            .get_mut(&peer_id)
            .ok_or_else(|| anyhow!("Peer not found: {}", peer_id))?;

        if let Some(nick) = nickname {
            peer.nickname = Some(nick);
        }

        if let Some(trust) = trust_level {
            peer.trust_level = trust.min(100);
        }

        if let Some(t) = tags {
            peer.tags = t;
        }

        drop(peers); // Release lock before saving

        // Auto-save if needed
        let _ = self.auto_save_if_needed().await;

        Ok(())
    }

    /// Ban a peer
    pub async fn ban_peer(&self, peer_id: String) -> Result<()> {
        // Parse peer ID to libp2p format
        let libp2p_peer_id = libp2p::PeerId::from_bytes(peer_id.as_bytes())
            .map_err(|_| anyhow!("Invalid peer ID format"))?;

        // Blacklist in network manager
        let network_manager = self.network_manager.lock().await;
        network_manager.blacklist_peer(libp2p_peer_id).await;

        // Update peer info
        {
            let mut peers = self.peers.write().await;
            if let Some(peer) = peers.get_mut(&peer_id) {
                peer.trust_level = 0;
                peer.tags.push("banned".to_string());
            }
        }

        // Auto-save if needed
        let _ = self.auto_save_if_needed().await;

        warn!("Banned peer: {}", peer_id);
        Ok(())
    }

    /// Unban a peer by address
    pub async fn unban_peer(&self, address: String) -> Result<()> {
        // Find peer by address
        let peer_id = {
            let peers = self.peers.read().await;
            peers
                .values()
                .find(|p| p.address == address)
                .map(|p| p.id.clone())
        };

        if let Some(pid) = peer_id {
            let mut peers = self.peers.write().await;
            if let Some(peer) = peers.get_mut(&pid) {
                peer.trust_level = 50; // Reset to neutral
                peer.tags.retain(|t| t != "banned");
            }
        }

        // Auto-save if needed
        let _ = self.auto_save_if_needed().await;

        info!("Unbanned peer with address: {}", address);
        Ok(())
    }

    /// Import peers from JSON file
    pub async fn import_peers(&self, path: PathBuf, merge: bool) -> Result<usize> {
        let data = fs::read_to_string(&path)?;
        let imported_peers: Vec<PeerInfo> = serde_json::from_str(&data)?;

        let mut count = 0;
        {
            let mut peers = self.peers.write().await;

            if !merge {
                peers.clear();
            }

            for peer in imported_peers {
                if !peers.contains_key(&peer.id) {
                    count += 1;
                }
                peers.insert(peer.id.clone(), peer);
            }
        }

        // Save to disk
        self.save_peers().await?;

        info!("Imported {} new peers from {:?}", count, path);
        Ok(count)
    }

    /// Export peers to JSON file
    pub async fn export_peers(
        &self,
        path: PathBuf,
        filter_tags: Option<Vec<String>>,
    ) -> Result<usize> {
        let peers = self.peers.read().await;

        let export_list: Vec<&PeerInfo> = if let Some(tags) = filter_tags {
            peers
                .values()
                .filter(|p| tags.iter().any(|t| p.tags.contains(t)))
                .collect()
        } else {
            peers.values().collect()
        };

        let data = serde_json::to_string_pretty(&export_list)?;
        fs::write(&path, data)?;

        info!("Exported {} peers to {:?}", export_list.len(), path);
        Ok(export_list.len())
    }

    /// Test connectivity to all known peers
    pub async fn test_all_peers(
        &self,
        progress_callback: impl Fn(usize, usize),
    ) -> Result<Vec<(String, bool, Option<f64>)>> {
        let peer_ids: Vec<String> = {
            let peers = self.peers.read().await;
            peers.keys().cloned().collect()
        };

        let total = peer_ids.len();
        let mut results = Vec::new();

        for (idx, peer_id) in peer_ids.iter().enumerate() {
            progress_callback(idx + 1, total);

            // Get peer info
            let peer_info = {
                let peers = self.peers.read().await;
                peers.get(peer_id).cloned()
            };

            if let Some(info) = peer_info {
                // Try to connect if not already connected
                let start = std::time::Instant::now();
                let connected = match self.add_peer(info.address.clone(), None).await {
                    Ok(_) => {
                        let latency = start.elapsed().as_millis() as f64;
                        results.push((peer_id.clone(), true, Some(latency)));
                        true
                    }
                    Err(_) => {
                        results.push((peer_id.clone(), false, None));
                        false
                    }
                };

                // Update peer info
                if connected {
                    let mut peers = self.peers.write().await;
                    if let Some(peer) = peers.get_mut(peer_id) {
                        peer.success_rate = (peer.success_rate * 0.9) + 0.1;
                        peer.last_seen = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                    }
                } else {
                    let mut peers = self.peers.write().await;
                    if let Some(peer) = peers.get_mut(peer_id) {
                        peer.success_rate *= 0.9;
                    }
                }
            }
        }

        // Auto-save if needed
        let _ = self.auto_save_if_needed().await;

        Ok(results)
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats> {
        let network_manager = self.network_manager.lock().await;
        let stats = network_manager.get_network_stats().await;

        Ok(NetworkStats {
            total_known_peers: self.peers.read().await.len(),
            connected_peers: stats.connected_peers,
            average_reputation: stats.average_reputation,
            blacklisted_peers: stats.blacklisted_peers,
            trusted_peers: stats.trusted_peers,
        })
    }

    /// Validate peer address format
    pub fn is_valid_address(address: &str) -> bool {
        // Check format: IP:PORT or hostname:PORT
        if let Some((host, port_str)) = address.rsplit_once(':') {
            if host.is_empty() || port_str.is_empty() {
                return false;
            }

            // Validate port
            if let Ok(port) = port_str.parse::<u16>() {
                if port == 0 {
                    return false;
                }
            } else {
                return false;
            }

            // Basic validation for host
            if host.parse::<std::net::IpAddr>().is_ok() {
                return true; // Valid IP
            }

            // Basic hostname validation
            if host.len() <= 253 && !host.is_empty() {
                return host
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '.' || c == '-');
            }
        }

        false
    }

    /// Shutdown the peer manager
    pub async fn shutdown(&self) -> Result<()> {
        // Save peers before shutdown
        self.save_peers().await?;

        // Shutdown network manager
        let mut network_manager = self.network_manager.lock().await;
        network_manager
            .shutdown()
            .await
            .map_err(|e| anyhow!("Failed to shutdown network manager: {}", e))?;

        info!("PeerManager shutdown complete");
        Ok(())
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_known_peers: usize,
    pub connected_peers: usize,
    pub average_reputation: f64,
    pub blacklisted_peers: usize,
    pub trusted_peers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_peer_info_creation() {
        let peer = PeerInfo::new("peer123".to_string(), "127.0.0.1:8000".to_string());
        assert_eq!(peer.id, "peer123");
        assert_eq!(peer.address, "127.0.0.1:8000");
        assert_eq!(peer.trust_level, 50);
        assert!(peer.nickname.is_none());
    }

    #[tokio::test]
    async fn test_peer_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = PeerManagerConfig {
            data_path: temp_dir.path().join("peers.json"),
            ..Default::default()
        };

        let manager = PeerManager::new(config).await.unwrap();
        let peers = manager.list_peers().await.unwrap();
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn test_address_validation() {
        assert!(PeerManager::is_valid_address("127.0.0.1:8000"));
        assert!(PeerManager::is_valid_address("192.168.1.1:9999"));
        assert!(PeerManager::is_valid_address("example.com:8080"));
        assert!(PeerManager::is_valid_address("sub.domain.com:443"));

        assert!(!PeerManager::is_valid_address("invalid"));
        assert!(!PeerManager::is_valid_address(":8000"));
        assert!(!PeerManager::is_valid_address("127.0.0.1:"));
        assert!(!PeerManager::is_valid_address("127.0.0.1:0"));
        assert!(!PeerManager::is_valid_address("127.0.0.1:70000"));
    }
}
