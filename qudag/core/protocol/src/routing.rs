//! Message routing module for QuDAG protocol with anonymity preservation.

use crate::types::ProtocolError;
use qudag_crypto::ml_kem::{MlKem768, KemError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Routing errors
#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("Route not found")]
    RouteNotFound,
    
    #[error("Invalid route")]
    InvalidRoute,
    
    #[error("Routing loop detected")]
    RoutingLoop,
    
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed")]
    DecryptionFailed,
    
    #[error("Hop limit exceeded")]
    HopLimitExceeded,
    
    #[error("Peer not found")]
    PeerNotFound,
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: [u8; 32],
    /// Network address
    pub address: SocketAddr,
    /// Public key for encryption
    pub public_key: Vec<u8>,
    /// Latency to peer
    pub latency: Duration,
    /// Bandwidth to peer
    pub bandwidth: u64,
    /// Reliability score (0.0 - 1.0)
    pub reliability: f64,
    /// Last seen timestamp
    pub last_seen: Instant,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(peer_id: [u8; 32], address: SocketAddr, public_key: Vec<u8>) -> Self {
        Self {
            peer_id,
            address,
            public_key,
            latency: Duration::from_millis(100),
            bandwidth: 1_000_000, // 1 Mbps default
            reliability: 1.0,
            last_seen: Instant::now(),
        }
    }

    /// Update peer metrics
    pub fn update_metrics(&mut self, latency: Duration, bandwidth: u64, success: bool) {
        self.latency = latency;
        self.bandwidth = bandwidth;
        self.last_seen = Instant::now();
        
        // Update reliability using exponential moving average
        let alpha = 0.1;
        let new_reliability = if success { 1.0 } else { 0.0 };
        self.reliability = alpha * new_reliability + (1.0 - alpha) * self.reliability;
    }

    /// Check if peer is alive
    pub fn is_alive(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() < timeout
    }

    /// Calculate routing score
    pub fn routing_score(&self) -> f64 {
        // Higher score is better
        // Factors: reliability, inverse latency, bandwidth
        let latency_score = 1.0 / (self.latency.as_secs_f64() + 0.001);
        let bandwidth_score = (self.bandwidth as f64).log10() / 10.0;
        
        self.reliability * (0.5 * latency_score + 0.3 * bandwidth_score + 0.2)
    }
}

/// Onion routing layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionLayer {
    /// Encrypted next hop
    pub next_hop: Vec<u8>,
    /// Encrypted payload
    pub payload: Vec<u8>,
    /// Layer authentication tag
    pub auth_tag: Vec<u8>,
}

/// Onion routing header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionHeader {
    /// Routing layers
    pub layers: Vec<OnionLayer>,
    /// Total hop count
    pub hop_count: u8,
    /// Message ID for tracking
    pub message_id: [u8; 16],
}

/// Routing path
#[derive(Debug, Clone)]
pub struct RoutingPath {
    /// Path hops (peer IDs)
    pub hops: Vec<[u8; 32]>,
    /// Path quality score
    pub quality_score: f64,
    /// Path latency estimate
    pub estimated_latency: Duration,
    /// Path bandwidth estimate
    pub estimated_bandwidth: u64,
    /// Creation timestamp
    pub created_at: Instant,
}

impl RoutingPath {
    /// Create new routing path
    pub fn new(hops: Vec<[u8; 32]>) -> Self {
        Self {
            hops,
            quality_score: 0.0,
            estimated_latency: Duration::from_millis(0),
            estimated_bandwidth: 0,
            created_at: Instant::now(),
        }
    }

    /// Calculate path quality
    pub fn calculate_quality(&mut self, peer_table: &PeerTable) {
        let mut total_latency = Duration::from_millis(0);
        let mut min_bandwidth = u64::MAX;
        let mut reliability_product = 1.0;

        for peer_id in &self.hops {
            if let Some(peer) = peer_table.get_peer(peer_id) {
                total_latency += peer.latency;
                min_bandwidth = min_bandwidth.min(peer.bandwidth);
                reliability_product *= peer.reliability;
            }
        }

        self.estimated_latency = total_latency;
        self.estimated_bandwidth = min_bandwidth;
        
        // Quality score considers reliability, latency, and bandwidth
        let latency_score = 1.0 / (total_latency.as_secs_f64() + 0.001);
        let bandwidth_score = (min_bandwidth as f64).log10() / 10.0;
        
        self.quality_score = reliability_product * (0.5 * latency_score + 0.5 * bandwidth_score);
    }

    /// Check if path is still valid
    pub fn is_valid(&self, peer_table: &PeerTable, max_age: Duration) -> bool {
        if self.created_at.elapsed() > max_age {
            return false;
        }

        // Check if all hops are still alive
        for peer_id in &self.hops {
            if let Some(peer) = peer_table.get_peer(peer_id) {
                if !peer.is_alive(Duration::from_secs(60)) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Peer routing table
#[derive(Debug)]
pub struct PeerTable {
    /// Map of peer ID to peer info
    peers: HashMap<[u8; 32], PeerInfo>,
    /// Connection graph for routing
    connections: HashMap<[u8; 32], Vec<[u8; 32]>>,
}

impl PeerTable {
    /// Create new peer table
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    /// Add peer to table
    pub fn add_peer(&mut self, peer: PeerInfo) {
        let peer_id = peer.peer_id;
        self.peers.insert(peer_id, peer);
        self.connections.entry(peer_id).or_insert_with(Vec::new);
    }

    /// Remove peer from table
    pub fn remove_peer(&mut self, peer_id: &[u8; 32]) {
        self.peers.remove(peer_id);
        self.connections.remove(peer_id);
        
        // Remove from other peers' connection lists
        for connections in self.connections.values_mut() {
            connections.retain(|id| id != peer_id);
        }
    }

    /// Get peer info
    pub fn get_peer(&self, peer_id: &[u8; 32]) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    /// Get all peers
    pub fn get_all_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }

    /// Add connection between peers
    pub fn add_connection(&mut self, peer1: [u8; 32], peer2: [u8; 32]) {
        self.connections.entry(peer1).or_default().push(peer2);
        self.connections.entry(peer2).or_default().push(peer1);
    }

    /// Get connected peers
    pub fn get_connections(&self, peer_id: &[u8; 32]) -> Vec<[u8; 32]> {
        self.connections.get(peer_id).cloned().unwrap_or_default()
    }

    /// Find shortest path using Dijkstra's algorithm
    pub fn find_shortest_path(&self, source: &[u8; 32], destination: &[u8; 32]) -> Option<RoutingPath> {
        if source == destination {
            return Some(RoutingPath::new(vec![*source]));
        }

        let mut distances: HashMap<[u8; 32], f64> = HashMap::new();
        let mut previous: HashMap<[u8; 32], [u8; 32]> = HashMap::new();
        let mut unvisited: Vec<[u8; 32]> = self.peers.keys().copied().collect();

        // Initialize distances
        for peer_id in &unvisited {
            distances.insert(*peer_id, if peer_id == source { 0.0 } else { f64::INFINITY });
        }

        while !unvisited.is_empty() {
            // Find unvisited node with minimum distance
            let current_idx = unvisited
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    distances[a].partial_cmp(&distances[b]).unwrap()
                })
                .map(|(i, _)| i)?;
            
            let current = unvisited.remove(current_idx);

            if current == *destination {
                break;
            }

            if distances[&current] == f64::INFINITY {
                break;
            }

            // Update distances to neighbors
            for neighbor in self.get_connections(&current) {
                if let Some(neighbor_peer) = self.get_peer(&neighbor) {
                    let weight = 1.0 / neighbor_peer.routing_score(); // Lower score = higher weight
                    let alt_distance = distances[&current] + weight;
                    
                    if alt_distance < distances[&neighbor] {
                        distances.insert(neighbor, alt_distance);
                        previous.insert(neighbor, current);
                    }
                }
            }
        }

        // Reconstruct path
        if !previous.contains_key(destination) {
            return None;
        }

        let mut path = Vec::new();
        let mut current = *destination;
        
        while current != *source {
            path.push(current);
            current = previous[&current];
        }
        path.push(*source);
        path.reverse();

        let mut routing_path = RoutingPath::new(path);
        routing_path.calculate_quality(self);
        
        Some(routing_path)
    }

    /// Find multiple paths for redundancy
    pub fn find_multiple_paths(
        &self,
        source: &[u8; 32],
        destination: &[u8; 32],
        count: usize,
    ) -> Vec<RoutingPath> {
        let mut paths = Vec::new();
        let mut excluded_nodes = Vec::new();

        for _ in 0..count {
            // Temporarily remove excluded nodes
            let mut temp_table = self.clone();
            for node in &excluded_nodes {
                temp_table.remove_peer(node);
            }

            if let Some(path) = temp_table.find_shortest_path(source, destination) {
                // Exclude intermediate nodes for next iteration
                for (i, hop) in path.hops.iter().enumerate() {
                    if i > 0 && i < path.hops.len() - 1 {
                        excluded_nodes.push(*hop);
                    }
                }
                paths.push(path);
            } else {
                break;
            }
        }

        paths
    }

    /// Clean up dead peers
    pub fn cleanup_dead_peers(&mut self, timeout: Duration) {
        let dead_peers: Vec<[u8; 32]> = self.peers
            .iter()
            .filter(|(_, peer)| !peer.is_alive(timeout))
            .map(|(id, _)| *id)
            .collect();

        for peer_id in dead_peers {
            warn!("Removing dead peer: {:?}", hex::encode(peer_id));
            self.remove_peer(&peer_id);
        }
    }
}

impl Clone for PeerTable {
    fn clone(&self) -> Self {
        Self {
            peers: self.peers.clone(),
            connections: self.connections.clone(),
        }
    }
}

impl Default for PeerTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Message router with onion routing support
pub struct MessageRouter {
    /// Local peer ID
    local_peer_id: [u8; 32],
    /// Peer routing table
    peer_table: PeerTable,
    /// Cached routing paths
    path_cache: HashMap<[u8; 32], Vec<RoutingPath>>,
    /// Onion routing keys
    onion_keys: HashMap<[u8; 32], Vec<u8>>,
    /// Message tracking
    message_tracking: HashMap<[u8; 16], RouteTrackingInfo>,
    /// ML-KEM instance for encryption
    ml_kem: MlKem768,
}

/// Route tracking information
#[derive(Debug)]
struct RouteTrackingInfo {
    /// Source peer
    source: [u8; 32],
    /// Destination peer
    destination: [u8; 32],
    /// Creation timestamp
    created_at: Instant,
    /// Hop count
    hop_count: u8,
}

impl MessageRouter {
    /// Create new message router
    pub fn new(local_peer_id: [u8; 32]) -> Result<Self, RoutingError> {
        Ok(Self {
            local_peer_id,
            peer_table: PeerTable::new(),
            path_cache: HashMap::new(),
            onion_keys: HashMap::new(),
            message_tracking: HashMap::new(),
            ml_kem: MlKem768::new().map_err(|_| RoutingError::EncryptionFailed)?,
        })
    }

    /// Add peer to routing table
    pub fn add_peer(&mut self, peer: PeerInfo) {
        let peer_id = peer.peer_id;
        self.peer_table.add_peer(peer);
        // Invalidate cached paths involving this peer
        self.path_cache.clear();
    }

    /// Remove peer from routing table
    pub fn remove_peer(&mut self, peer_id: &[u8; 32]) {
        self.peer_table.remove_peer(peer_id);
        self.path_cache.remove(peer_id);
        self.onion_keys.remove(peer_id);
    }

    /// Find route to destination
    pub fn find_route(&mut self, destination: &[u8; 32]) -> Result<RoutingPath, RoutingError> {
        // Check cache first
        if let Some(cached_paths) = self.path_cache.get(destination) {
            if let Some(valid_path) = cached_paths.iter().find(|path| path.is_valid(&self.peer_table, Duration::from_secs(300))) {
                return Ok(valid_path.clone());
            }
        }

        // Find new path
        let path = self.peer_table
            .find_shortest_path(&self.local_peer_id, destination)
            .ok_or(RoutingError::RouteNotFound)?;

        // Cache the path
        self.path_cache.entry(*destination)
            .or_insert_with(Vec::new)
            .push(path.clone());

        Ok(path)
    }

    /// Create onion routing header
    pub fn create_onion_header(
        &self,
        path: &RoutingPath,
        payload: &[u8],
        message_id: [u8; 16],
    ) -> Result<OnionHeader, RoutingError> {
        let mut layers = Vec::new();
        let mut current_payload = payload.to_vec();

        // Build layers from destination to source
        for (i, peer_id) in path.hops.iter().rev().enumerate() {
            let peer = self.peer_table.get_peer(peer_id)
                .ok_or(RoutingError::PeerNotFound)?;

            // Encrypt payload with peer's public key
            let (ciphertext, _) = self.ml_kem.encapsulate(&peer.public_key)
                .map_err(|_| RoutingError::EncryptionFailed)?;

            let next_hop = if i == path.hops.len() - 1 {
                vec![] // Final destination
            } else {
                path.hops[path.hops.len() - i - 2].to_vec()
            };

            let layer = OnionLayer {
                next_hop: next_hop.clone(),
                payload: current_payload.clone(),
                auth_tag: vec![], // TODO: Add HMAC authentication
            };

            layers.push(layer);
            current_payload = ciphertext;
        }

        layers.reverse();

        Ok(OnionHeader {
            layers,
            hop_count: path.hops.len() as u8,
            message_id,
        })
    }

    /// Process onion layer (decrypt and forward)
    pub fn process_onion_layer(
        &self,
        header: &mut OnionHeader,
        secret_key: &[u8],
    ) -> Result<(Option<[u8; 32]>, Vec<u8>), RoutingError> {
        if header.layers.is_empty() {
            return Err(RoutingError::InvalidRoute);
        }

        let layer = header.layers.remove(0);
        
        // Decrypt payload
        let decrypted_payload = self.ml_kem.decapsulate(secret_key, &layer.payload)
            .map_err(|_| RoutingError::DecryptionFailed)?;

        // Extract next hop
        let next_hop = if layer.next_hop.is_empty() {
            None
        } else if layer.next_hop.len() == 32 {
            let mut hop = [0u8; 32];
            hop.copy_from_slice(&layer.next_hop);
            Some(hop)
        } else {
            return Err(RoutingError::InvalidRoute);
        };

        Ok((next_hop, decrypted_payload))
    }

    /// Route message with onion routing
    pub async fn route_message(
        &mut self,
        destination: &[u8; 32],
        payload: &[u8],
    ) -> Result<(), RoutingError> {
        let path = self.find_route(destination)?;
        
        if path.hops.len() > 10 {
            return Err(RoutingError::HopLimitExceeded);
        }

        // Generate unique message ID
        let message_id: [u8; 16] = rand::random();
        
        // Track message
        self.message_tracking.insert(message_id, RouteTrackingInfo {
            source: self.local_peer_id,
            destination: *destination,
            created_at: Instant::now(),
            hop_count: path.hops.len() as u8,
        });

        // Create onion header
        let onion_header = self.create_onion_header(&path, payload, message_id)?;

        // Send to first hop
        if let Some(first_hop) = path.hops.get(1) { // Skip self
            info!("Routing message {} to {:?} via path of {} hops", 
                  hex::encode(message_id), hex::encode(destination), path.hops.len());
            
            // TODO: Actually send the message through network layer
            debug!("Sending onion header to first hop: {:?}", hex::encode(first_hop));
        }

        Ok(())
    }

    /// Update peer metrics
    pub fn update_peer_metrics(
        &mut self,
        peer_id: &[u8; 32],
        latency: Duration,
        bandwidth: u64,
        success: bool,
    ) {
        if let Some(peer) = self.peer_table.peers.get_mut(peer_id) {
            peer.update_metrics(latency, bandwidth, success);
            
            // Invalidate cached paths if peer performance changed significantly
            if !success || peer.reliability < 0.8 {
                self.path_cache.clear();
            }
        }
    }

    /// Clean up expired data
    pub fn cleanup(&mut self) {
        // Clean up dead peers
        self.peer_table.cleanup_dead_peers(Duration::from_secs(300));
        
        // Clean up expired message tracking
        let now = Instant::now();
        self.message_tracking.retain(|_, info| {
            now.duration_since(info.created_at) < Duration::from_secs(3600)
        });

        // Clean up stale path cache
        self.path_cache.retain(|_, paths| {
            paths.retain(|path| path.is_valid(&self.peer_table, Duration::from_secs(300)));
            !paths.is_empty()
        });
    }

    /// Get routing statistics
    pub fn get_stats(&self) -> RoutingStats {
        RoutingStats {
            peer_count: self.peer_table.peers.len(),
            cached_paths: self.path_cache.len(),
            tracked_messages: self.message_tracking.len(),
            average_path_length: self.path_cache.values()
                .flatten()
                .map(|path| path.hops.len() as f64)
                .sum::<f64>() / self.path_cache.len() as f64,
        }
    }
}

/// Routing statistics
#[derive(Debug, Clone)]
pub struct RoutingStats {
    /// Number of known peers
    pub peer_count: usize,
    /// Number of cached paths
    pub cached_paths: usize,
    /// Number of tracked messages
    pub tracked_messages: usize,
    /// Average path length
    pub average_path_length: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn create_test_peer(id: u8, addr: &str) -> PeerInfo {
        let mut peer_id = [0u8; 32];
        peer_id[0] = id;
        
        PeerInfo::new(
            peer_id,
            addr.parse::<SocketAddr>().unwrap(),
            vec![id; 32],
        )
    }

    #[test]
    fn test_peer_info_creation() {
        let peer = create_test_peer(1, "127.0.0.1:8000");
        assert_eq!(peer.peer_id[0], 1);
        assert_eq!(peer.reliability, 1.0);
        assert!(peer.is_alive(Duration::from_secs(1)));
    }

    #[test]
    fn test_peer_table_operations() {
        let mut table = PeerTable::new();
        let peer1 = create_test_peer(1, "127.0.0.1:8001");
        let peer2 = create_test_peer(2, "127.0.0.1:8002");
        
        table.add_peer(peer1.clone());
        table.add_peer(peer2.clone());
        
        assert_eq!(table.get_all_peers().len(), 2);
        assert!(table.get_peer(&peer1.peer_id).is_some());
        
        table.add_connection(peer1.peer_id, peer2.peer_id);
        let connections = table.get_connections(&peer1.peer_id);
        assert!(connections.contains(&peer2.peer_id));
    }

    #[test]
    fn test_routing_path_creation() {
        let hops = vec![[1; 32], [2; 32], [3; 32]];
        let path = RoutingPath::new(hops.clone());
        
        assert_eq!(path.hops, hops);
        assert_eq!(path.quality_score, 0.0);
    }

    #[tokio::test]
    async fn test_message_router_creation() {
        let local_id = [1; 32];
        let router = MessageRouter::new(local_id);
        assert!(router.is_ok());
    }

    #[test]
    fn test_shortest_path_finding() {
        let mut table = PeerTable::new();
        
        // Create a simple network: 1 -> 2 -> 3
        let peer1 = create_test_peer(1, "127.0.0.1:8001");
        let peer2 = create_test_peer(2, "127.0.0.1:8002");
        let peer3 = create_test_peer(3, "127.0.0.1:8003");
        
        table.add_peer(peer1.clone());
        table.add_peer(peer2.clone());
        table.add_peer(peer3.clone());
        
        table.add_connection(peer1.peer_id, peer2.peer_id);
        table.add_connection(peer2.peer_id, peer3.peer_id);
        
        let path = table.find_shortest_path(&peer1.peer_id, &peer3.peer_id);
        assert!(path.is_some());
        
        let path = path.unwrap();
        assert_eq!(path.hops.len(), 3);
        assert_eq!(path.hops[0], peer1.peer_id);
        assert_eq!(path.hops[1], peer2.peer_id);
        assert_eq!(path.hops[2], peer3.peer_id);
    }

    #[tokio::test]
    async fn test_route_finding() {
        let local_id = [1; 32];
        let mut router = MessageRouter::new(local_id).unwrap();
        
        // Add peers
        let peer2 = create_test_peer(2, "127.0.0.1:8002");
        let peer3 = create_test_peer(3, "127.0.0.1:8003");
        
        router.add_peer(peer2.clone());
        router.add_peer(peer3.clone());
        
        // Add connections
        router.peer_table.add_connection(local_id, peer2.peer_id);
        router.peer_table.add_connection(peer2.peer_id, peer3.peer_id);
        
        let route = router.find_route(&peer3.peer_id);
        assert!(route.is_ok());
        
        let route = route.unwrap();
        assert_eq!(route.hops.len(), 3);
    }
}