use crate::onion::{CircuitManager, DirectoryClient, MLKEMOnionRouter};
use crate::types::{NetworkError, NetworkMessage, PeerId, RoutingStrategy};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Information about a hop in a route
#[derive(Debug, Clone)]
pub struct HopInfo {
    #[allow(dead_code)]
    peer_id: PeerId,
    known_peers: HashSet<PeerId>,
    layer_keys: HashMap<usize, Vec<u8>>,
}

impl HopInfo {
    /// Check if this hop can decrypt a specific layer
    pub fn can_decrypt_layer(&self, layer: usize) -> bool {
        self.layer_keys.contains_key(&layer)
    }

    /// Check if this hop knows about a specific peer
    pub fn knows_peer(&self, peer: &PeerId) -> bool {
        self.known_peers.contains(peer)
    }
}

/// Anonymous router for network messages with ML-KEM onion routing
#[derive(Clone)]
pub struct Router {
    /// Known peers in the network
    peers: Arc<RwLock<HashSet<PeerId>>>,
    /// Hop information for each peer
    hop_info: Arc<RwLock<HashMap<PeerId, HopInfo>>>,
    /// ML-KEM onion router instance
    onion_router: Arc<Mutex<MLKEMOnionRouter>>,
    /// Circuit manager
    circuit_manager: Arc<Mutex<CircuitManager>>,
    /// Directory client
    directory_client: Arc<DirectoryClient>,
}

impl Router {
    /// Create a new router with ML-KEM onion routing
    pub async fn new() -> Result<Self, NetworkError> {
        let onion_router = MLKEMOnionRouter::new().await.map_err(|e| {
            NetworkError::RoutingError(format!("Failed to create onion router: {:?}", e))
        })?;

        let circuit_manager = Arc::new(Mutex::new(CircuitManager::new()));
        let directory_client = Arc::new(DirectoryClient::new());

        Ok(Self {
            peers: Arc::new(RwLock::new(HashSet::new())),
            hop_info: Arc::new(RwLock::new(HashMap::new())),
            onion_router: Arc::new(Mutex::new(onion_router)),
            circuit_manager,
            directory_client,
        })
    }

    /// Add a peer to the network
    pub async fn add_peer(&self, peer_id: PeerId) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id);

        // Create hop info for this peer
        let mut hop_info = self.hop_info.write().await;
        let mut known_peers = HashSet::new();

        // Each peer knows about a random subset of other peers (simulating network topology)
        let all_peers: Vec<_> = peers.iter().filter(|&&p| p != peer_id).cloned().collect();
        let mut rng = thread_rng();
        let subset_size = (all_peers.len() / 2).clamp(1, 3); // Know about 1-3 peers
        let known_subset: Vec<_> = all_peers
            .choose_multiple(&mut rng, subset_size)
            .cloned()
            .collect();

        for peer in known_subset {
            known_peers.insert(peer);
        }

        // Generate layer keys for this peer (simulating onion routing capabilities)
        let mut layer_keys = HashMap::new();
        for i in 0..5 {
            // Support up to 5 layers
            layer_keys.insert(i, vec![i as u8; 32]); // Simple key generation
        }

        hop_info.insert(
            peer_id,
            HopInfo {
                peer_id,
                known_peers,
                layer_keys,
            },
        );
    }

    /// Route a message using the specified strategy
    pub async fn route(
        &self,
        message: &NetworkMessage,
        strategy: RoutingStrategy,
    ) -> Result<Vec<PeerId>, NetworkError> {
        match strategy {
            RoutingStrategy::Anonymous { hops } => self.route_anonymous(message, hops).await,
            RoutingStrategy::Direct(peer_bytes) => {
                // Convert bytes to PeerId if possible
                if peer_bytes.len() == 32 {
                    let mut peer_id_bytes = [0u8; 32];
                    peer_id_bytes.copy_from_slice(&peer_bytes);
                    Ok(vec![PeerId::from_bytes(peer_id_bytes)])
                } else {
                    Err(NetworkError::RoutingError("Invalid peer ID format".into()))
                }
            }
            RoutingStrategy::Flood => {
                let peers = self.peers.read().await;
                Ok(peers.iter().cloned().collect())
            }
            RoutingStrategy::RandomSubset(count) => {
                let peers = self.peers.read().await;
                let mut rng = thread_rng();
                let selected: Vec<_> = peers
                    .iter()
                    .choose_multiple(&mut rng, count)
                    .into_iter()
                    .cloned()
                    .collect();
                Ok(selected)
            }
        }
    }

    /// Route a message anonymously using ML-KEM onion routing
    async fn route_anonymous(
        &self,
        message: &NetworkMessage,
        hops: usize,
    ) -> Result<Vec<PeerId>, NetworkError> {
        // Ensure we have at least 3 hops for anonymity
        let actual_hops = hops.max(3);
        let peers = self.peers.read().await;

        // Filter out source and destination from available peers
        let source_peer = if message.source.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&message.source);
            Some(PeerId::from_bytes(bytes))
        } else {
            None
        };

        let dest_peer = if message.destination.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&message.destination);
            Some(PeerId::from_bytes(bytes))
        } else {
            None
        };

        let available_peers: Vec<_> = peers
            .iter()
            .filter(|&&p| Some(p) != source_peer && Some(p) != dest_peer)
            .cloned()
            .collect();

        if available_peers.len() < hops {
            return Err(NetworkError::RoutingError(
                "Not enough peers for anonymous routing".into(),
            ));
        }

        // Build a circuit using the circuit manager
        let mut circuit_mgr = self.circuit_manager.lock().await;
        let circuit_id = circuit_mgr
            .build_circuit(actual_hops, &self.directory_client)
            .await
            .map_err(|e| NetworkError::RoutingError(format!("Circuit build failed: {:?}", e)))?;

        // Activate the circuit
        circuit_mgr.activate_circuit(circuit_id).map_err(|e| {
            NetworkError::RoutingError(format!("Circuit activation failed: {:?}", e))
        })?;

        // Get circuit hops
        let circuit = circuit_mgr
            .get_active_circuit()
            .ok_or_else(|| NetworkError::RoutingError("No active circuit available".into()))?;

        // Create onion layers for the message
        let onion_router = self.onion_router.lock().await;
        let _layers = onion_router
            .encrypt_layers(message.payload.clone(), circuit.hops.clone())
            .await
            .map_err(|e| NetworkError::RoutingError(format!("Onion encryption failed: {:?}", e)))?;

        // Convert node IDs to PeerIds for routing
        let route: Vec<PeerId> = circuit
            .hops
            .iter()
            .filter_map(|node_id| {
                if node_id.len() == 32 {
                    let mut peer_id_bytes = [0u8; 32];
                    peer_id_bytes.copy_from_slice(&node_id[..32]);
                    Some(PeerId::from_bytes(peer_id_bytes))
                } else {
                    None
                }
            })
            .collect();

        // Update circuit metrics
        circuit_mgr.update_circuit_metrics(circuit_id, message.payload.len() as u64, true);

        Ok(route)
    }

    /// Update hop knowledge to simulate onion routing properties
    #[allow(dead_code)]
    async fn update_hop_knowledge(&self, route: &[PeerId]) {
        let mut hop_info = self.hop_info.write().await;

        for (i, &peer_id) in route.iter().enumerate() {
            if let Some(info) = hop_info.get_mut(&peer_id) {
                // Clear previous knowledge
                info.known_peers.clear();

                // Each hop only knows about its immediate neighbors
                if i > 0 {
                    info.known_peers.insert(route[i - 1]);
                }
                if i < route.len() - 1 {
                    info.known_peers.insert(route[i + 1]);
                }

                // Update layer keys - each hop can only decrypt its own layer
                info.layer_keys.clear();
                info.layer_keys.insert(i, vec![i as u8; 32]);
            }
        }
    }

    /// Get hop information for a peer
    pub async fn get_hop_info(&self, peer_id: &PeerId) -> Result<HopInfo, NetworkError> {
        let hop_info = self.hop_info.read().await;
        hop_info
            .get(peer_id)
            .cloned()
            .ok_or_else(|| NetworkError::RoutingError("Hop information not found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessagePriority;
    use std::time::Duration;

    #[tokio::test]
    async fn test_router_creation() {
        let router = Router::new().await.unwrap();
        let peers = router.peers.read().await;
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn test_add_peer() {
        let router = Router::new().await.unwrap();
        let peer_id = PeerId::random();

        router.add_peer(peer_id).await;

        let peers = router.peers.read().await;
        assert!(peers.contains(&peer_id));
    }

    #[tokio::test]
    async fn test_anonymous_routing() {
        let router = Router::new().await.unwrap();

        // Add test peers
        let peers: Vec<_> = (0..5).map(|_| PeerId::random()).collect();
        for peer in &peers {
            router.add_peer(*peer).await;
        }

        // Create test message
        let msg = NetworkMessage {
            id: "test".into(),
            source: peers[0].to_bytes().to_vec(),
            destination: peers[4].to_bytes().to_vec(),
            payload: vec![1, 2, 3],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };

        // Test anonymous routing
        let route = router
            .route(&msg, RoutingStrategy::Anonymous { hops: 3 })
            .await
            .unwrap();

        assert_eq!(route.len(), 3);
        assert!(!route.contains(&peers[0])); // Should not include source
        assert!(!route.contains(&peers[4])); // Should not include destination
    }
}
