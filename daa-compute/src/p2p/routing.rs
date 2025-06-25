//! Message routing protocols for P2P network
//!
//! Implements various routing strategies including DHT-based routing,
//! onion routing for privacy, and optimized paths for gradient sharing.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use libp2p::{PeerId, Multiaddr};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use parking_lot::RwLock;
use std::sync::Arc;

/// Routing table entry
#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub latency: Option<Duration>,
    pub bandwidth: Option<f64>, // MB/s
    pub last_seen: Instant,
    pub reliability_score: f32, // 0.0 to 1.0
}

/// Routing strategy
#[derive(Debug, Clone, Copy)]
pub enum RoutingStrategy {
    /// Direct routing (single hop)
    Direct,
    /// Multi-hop routing through DHT
    DhtRouting,
    /// Onion routing for privacy
    OnionRouting { layers: usize },
    /// Optimized routing based on metrics
    OptimizedRouting,
}

/// Message to be routed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedMessage {
    pub id: String,
    #[serde(with = "peer_id_serde")]
    pub source: PeerId,
    #[serde(with = "peer_id_serde")]
    pub destination: PeerId,
    pub payload: Vec<u8>,
    pub ttl: u8,
    pub routing_strategy: RoutingStrategyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategyType {
    Direct,
    DhtRouting,
    OnionRouting,
    OptimizedRouting,
}

/// Onion layer for privacy routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionLayer {
    #[serde(with = "peer_id_serde")]
    pub next_hop: PeerId,
    pub encrypted_payload: Vec<u8>,
}

mod peer_id_serde {
    use super::*;
    use serde::{Deserializer, Serializer};
    use std::str::FromStr;
    
    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&peer_id.to_string())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PeerId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Router manages message routing in the P2P network
pub struct Router {
    local_peer_id: PeerId,
    routing_table: Arc<RwLock<HashMap<PeerId, RouteEntry>>>,
    pending_messages: Arc<RwLock<HashMap<String, RoutedMessage>>>,
    route_cache: Arc<RwLock<HashMap<(PeerId, PeerId), Vec<PeerId>>>>,
}

impl Router {
    pub fn new(local_peer_id: PeerId) -> Self {
        Self {
            local_peer_id,
            routing_table: Arc::new(RwLock::new(HashMap::new())),
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            route_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add or update a peer in the routing table
    pub fn update_peer(&self, entry: RouteEntry) {
        self.routing_table.write().insert(entry.peer_id, entry);
    }
    
    /// Remove a peer from the routing table
    pub fn remove_peer(&self, peer_id: &PeerId) {
        self.routing_table.write().remove(peer_id);
        
        // Clear cached routes involving this peer
        let mut cache = self.route_cache.write();
        cache.retain(|(src, dst), route| {
            !route.contains(peer_id) && src != peer_id && dst != peer_id
        });
    }
    
    /// Find the best route to a destination
    pub fn find_route(&self, destination: &PeerId, strategy: RoutingStrategy) -> Result<Vec<PeerId>> {
        match strategy {
            RoutingStrategy::Direct => self.find_direct_route(destination),
            RoutingStrategy::DhtRouting => self.find_dht_route(destination),
            RoutingStrategy::OnionRouting { layers } => self.find_onion_route(destination, layers),
            RoutingStrategy::OptimizedRouting => self.find_optimized_route(destination),
        }
    }
    
    /// Find direct route (single hop)
    fn find_direct_route(&self, destination: &PeerId) -> Result<Vec<PeerId>> {
        let table = self.routing_table.read();
        if table.contains_key(destination) {
            Ok(vec![destination.clone()])
        } else {
            Err(anyhow!("No direct route to destination"))
        }
    }
    
    /// Find route through DHT
    fn find_dht_route(&self, destination: &PeerId) -> Result<Vec<PeerId>> {
        // Check cache first
        let cache_key = (self.local_peer_id, *destination);
        if let Some(cached_route) = self.route_cache.read().get(&cache_key) {
            return Ok(cached_route.clone());
        }
        
        // Implement Kademlia-style routing
        let table = self.routing_table.read();
        let mut candidates: Vec<_> = table.values()
            .map(|entry| {
                let distance = xor_distance(&entry.peer_id, destination);
                (entry.peer_id, distance)
            })
            .collect();
        
        // Sort by XOR distance
        candidates.sort_by_key(|(_, dist)| *dist);
        
        // Take the k closest peers (k=3 for redundancy)
        let route: Vec<PeerId> = candidates.into_iter()
            .take(3)
            .map(|(peer_id, _)| peer_id)
            .collect();
        
        if route.is_empty() {
            return Err(anyhow!("No route found"));
        }
        
        // Cache the route
        self.route_cache.write().insert(cache_key, route.clone());
        
        Ok(route)
    }
    
    /// Find onion route for privacy
    fn find_onion_route(&self, destination: &PeerId, layers: usize) -> Result<Vec<PeerId>> {
        let table = self.routing_table.read();
        
        // Select random peers for onion layers
        let mut available_peers: Vec<_> = table.keys()
            .filter(|&peer| peer != destination && peer != &self.local_peer_id)
            .cloned()
            .collect();
        
        if available_peers.len() < layers {
            return Err(anyhow!("Not enough peers for {} onion layers", layers));
        }
        
        // Shuffle and select peers
        use rand::seq::SliceRandom;
        available_peers.shuffle(&mut rand::thread_rng());
        
        let mut route: Vec<PeerId> = available_peers.into_iter()
            .take(layers)
            .collect();
        
        // Add destination as final hop
        route.push(*destination);
        
        Ok(route)
    }
    
    /// Find optimized route based on latency and bandwidth
    fn find_optimized_route(&self, destination: &PeerId) -> Result<Vec<PeerId>> {
        let table = self.routing_table.read();
        
        // Use Dijkstra's algorithm with custom cost function
        let mut distances: HashMap<PeerId, f64> = HashMap::new();
        let mut previous: HashMap<PeerId, PeerId> = HashMap::new();
        let mut queue = VecDeque::new();
        
        // Initialize
        distances.insert(self.local_peer_id, 0.0);
        queue.push_back(self.local_peer_id);
        
        while let Some(current) = queue.pop_front() {
            let current_distance = distances[&current];
            
            // Check all neighbors
            for (peer_id, entry) in table.iter() {
                if peer_id == &current {
                    continue;
                }
                
                // Calculate cost based on latency, bandwidth, and reliability
                let cost = calculate_route_cost(entry);
                let new_distance = current_distance + cost;
                
                if !distances.contains_key(peer_id) || new_distance < distances[peer_id] {
                    distances.insert(*peer_id, new_distance);
                    previous.insert(*peer_id, current);
                    queue.push_back(*peer_id);
                }
            }
        }
        
        // Reconstruct path
        let mut path = Vec::new();
        let mut current = destination;
        
        while current != &self.local_peer_id {
            path.push(*current);
            current = previous.get(current)
                .ok_or_else(|| anyhow!("No path to destination"))?;
        }
        
        path.reverse();
        Ok(path)
    }
    
    /// Create an onion-encrypted message
    pub fn create_onion_message(
        &self,
        route: &[PeerId],
        payload: Vec<u8>,
    ) -> Result<Vec<OnionLayer>> {
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
        use ring::rand::{SecureRandom, SystemRandom};
        
        let mut layers = Vec::new();
        let mut current_payload = payload;
        
        // Encrypt in reverse order (destination to source)
        for peer_id in route.iter().rev() {
            // Generate ephemeral key
            let rng = SystemRandom::new();
            let mut key_bytes = vec![0u8; CHACHA20_POLY1305.key_len()];
            rng.fill(&mut key_bytes)?;
            
            let key = UnboundKey::new(&CHACHA20_POLY1305, &key_bytes)?;
            let key = LessSafeKey::new(key);
            
            // Generate nonce
            let mut nonce_bytes = vec![0u8; CHACHA20_POLY1305.nonce_len()];
            rng.fill(&mut nonce_bytes)?;
            let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)?;
            
            // Encrypt
            let peer_bytes = peer_id.to_bytes();
            let aad = Aad::from(&peer_bytes);
            key.seal_in_place_append_tag(nonce, aad, &mut current_payload)?;
            
            layers.push(OnionLayer {
                next_hop: *peer_id,
                encrypted_payload: current_payload.clone(),
            });
        }
        
        Ok(layers)
    }
    
    /// Process received routed message
    pub fn process_message(&self, message: RoutedMessage) -> Result<Option<Vec<u8>>> {
        // Check TTL
        if message.ttl == 0 {
            return Err(anyhow!("Message TTL expired"));
        }
        
        // Check if we are the destination
        if message.destination == self.local_peer_id {
            return Ok(Some(message.payload));
        }
        
        // Forward the message
        let mut forwarded = message.clone();
        forwarded.ttl -= 1;
        
        // Store for forwarding
        self.pending_messages.write().insert(message.id.clone(), forwarded);
        
        Ok(None)
    }
}

/// Calculate XOR distance between two peer IDs (for Kademlia routing)
fn xor_distance(a: &PeerId, b: &PeerId) -> u128 {
    let a_bytes = a.to_bytes();
    let b_bytes = b.to_bytes();
    
    let mut distance = 0u128;
    for i in 0..16.min(a_bytes.len()).min(b_bytes.len()) {
        distance = (distance << 8) | (a_bytes[i] ^ b_bytes[i]) as u128;
    }
    
    distance
}

/// Calculate route cost based on peer metrics
fn calculate_route_cost(entry: &RouteEntry) -> f64 {
    let latency_cost = entry.latency
        .map(|l| l.as_millis() as f64 / 1000.0)
        .unwrap_or(1.0);
    
    let bandwidth_cost = entry.bandwidth
        .map(|b| 100.0 / b) // Higher bandwidth = lower cost
        .unwrap_or(1.0);
    
    let reliability_cost = 1.0 / (entry.reliability_score as f64 + 0.1);
    
    // Weighted sum
    0.5 * latency_cost + 0.3 * bandwidth_cost + 0.2 * reliability_cost
}

/// Gradient-aware routing for efficient all-reduce
pub struct GradientRouter {
    base_router: Router,
    topology: NetworkTopology,
}

#[derive(Debug, Clone)]
pub enum NetworkTopology {
    /// All nodes connected to all (full mesh)
    FullMesh,
    /// Ring topology for bandwidth efficiency
    Ring,
    /// Tree topology for latency efficiency
    Tree { fanout: usize },
    /// Hierarchical topology for geo-distributed
    Hierarchical { levels: usize },
}

impl GradientRouter {
    pub fn new(local_peer_id: PeerId, topology: NetworkTopology) -> Self {
        Self {
            base_router: Router::new(local_peer_id),
            topology,
        }
    }
    
    /// Get neighbors for gradient exchange based on topology
    pub fn get_gradient_neighbors(&self) -> Result<Vec<PeerId>> {
        let table = self.base_router.routing_table.read();
        let all_peers: Vec<_> = table.keys().cloned().collect();
        
        match &self.topology {
            NetworkTopology::FullMesh => Ok(all_peers),
            
            NetworkTopology::Ring => {
                // Find neighbors in ring
                let mut sorted_peers = all_peers;
                sorted_peers.sort();
                
                let our_idx = sorted_peers.iter()
                    .position(|p| p == &self.base_router.local_peer_id)
                    .unwrap_or(0);
                
                let n = sorted_peers.len();
                let prev = sorted_peers[(our_idx + n - 1) % n];
                let next = sorted_peers[(our_idx + 1) % n];
                
                Ok(vec![prev, next])
            }
            
            NetworkTopology::Tree { fanout } => {
                // Return children in tree
                Ok(all_peers.into_iter().take(*fanout).collect())
            }
            
            NetworkTopology::Hierarchical { .. } => {
                // Return peers in same hierarchical level
                // Simplified: group by first byte of peer ID
                let our_group = self.base_router.local_peer_id.to_bytes()[0];
                Ok(all_peers.into_iter()
                    .filter(|p| p.to_bytes()[0] == our_group)
                    .collect())
            }
        }
    }
}