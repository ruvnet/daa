//! Peer discovery mechanisms for the DHT

use libp2p::PeerId;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use daa_prime_core::Result;

/// Peer discovery strategy
#[derive(Debug, Clone)]
pub enum DiscoveryStrategy {
    /// Use mDNS for local network discovery
    Mdns,
    /// Use bootstrap nodes
    Bootstrap(Vec<PeerId>),
    /// Use both mDNS and bootstrap
    Hybrid {
        bootstrap_nodes: Vec<PeerId>,
        enable_mdns: bool,
    },
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub peer_id: PeerId,
    pub addresses: Vec<libp2p::Multiaddr>,
    pub discovered_at: Instant,
    pub discovery_source: DiscoverySource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiscoverySource {
    Mdns,
    Bootstrap,
    Kad,
    Manual,
}

/// Peer discovery manager
pub struct Discovery {
    strategy: DiscoveryStrategy,
    discovered_peers: HashSet<PeerId>,
    peer_info: Vec<DiscoveredPeer>,
    discovery_interval: Duration,
    last_discovery: Instant,
}

impl Discovery {
    pub fn new(strategy: DiscoveryStrategy) -> Self {
        Self {
            strategy,
            discovered_peers: HashSet::new(),
            peer_info: Vec::new(),
            discovery_interval: Duration::from_secs(30),
            last_discovery: Instant::now(),
        }
    }

    pub fn add_discovered_peer(&mut self, peer: DiscoveredPeer) -> bool {
        if self.discovered_peers.insert(peer.peer_id) {
            self.peer_info.push(peer);
            true
        } else {
            false
        }
    }

    pub fn get_discovered_peers(&self) -> &[DiscoveredPeer] {
        &self.peer_info
    }

    pub fn get_bootstrap_nodes(&self) -> Vec<PeerId> {
        match &self.strategy {
            DiscoveryStrategy::Bootstrap(nodes) => nodes.clone(),
            DiscoveryStrategy::Hybrid { bootstrap_nodes, .. } => bootstrap_nodes.clone(),
            _ => Vec::new(),
        }
    }

    pub fn should_discover(&self) -> bool {
        self.last_discovery.elapsed() >= self.discovery_interval
    }

    pub fn mark_discovery_complete(&mut self) {
        self.last_discovery = Instant::now();
    }

    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.discovered_peers.remove(peer_id);
        self.peer_info.retain(|p| &p.peer_id != peer_id);
    }

    pub fn peer_count(&self) -> usize {
        self.discovered_peers.len()
    }

    pub fn clear_old_discoveries(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.peer_info.retain(|peer| {
            let age = now.duration_since(peer.discovered_at);
            if age > max_age {
                self.discovered_peers.remove(&peer.peer_id);
                false
            } else {
                true
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use test_case::test_case;

    #[test]
    fn test_discovery_creation() {
        let strategy = DiscoveryStrategy::Mdns;
        let discovery = Discovery::new(strategy);
        
        assert_eq!(discovery.peer_count(), 0);
        assert!(discovery.get_discovered_peers().is_empty());
    }

    #[test]
    fn test_peer_discovery() {
        let mut discovery = Discovery::new(DiscoveryStrategy::Mdns);
        
        let peer1 = DiscoveredPeer {
            peer_id: PeerId::random(),
            addresses: vec!["/ip4/127.0.0.1/tcp/8000".parse().unwrap()],
            discovered_at: Instant::now(),
            discovery_source: DiscoverySource::Mdns,
        };
        
        assert!(discovery.add_discovered_peer(peer1.clone()));
        assert_eq!(discovery.peer_count(), 1);
        
        // Adding same peer again should return false
        assert!(!discovery.add_discovered_peer(peer1));
        assert_eq!(discovery.peer_count(), 1);
    }

    #[test]
    fn test_bootstrap_strategy() {
        let bootstrap_nodes = vec![
            PeerId::random(),
            PeerId::random(),
            PeerId::random(),
        ];
        
        let strategy = DiscoveryStrategy::Bootstrap(bootstrap_nodes.clone());
        let discovery = Discovery::new(strategy);
        
        assert_eq!(discovery.get_bootstrap_nodes(), bootstrap_nodes);
    }

    #[test_case(DiscoverySource::Mdns ; "mDNS discovery")]
    #[test_case(DiscoverySource::Bootstrap ; "Bootstrap discovery")]
    #[test_case(DiscoverySource::Kad ; "Kademlia discovery")]
    #[test_case(DiscoverySource::Manual ; "Manual discovery")]
    fn test_discovery_sources(source: DiscoverySource) {
        let mut discovery = Discovery::new(DiscoveryStrategy::Mdns);
        
        let peer = DiscoveredPeer {
            peer_id: PeerId::random(),
            addresses: vec!["/ip4/127.0.0.1/tcp/8000".parse().unwrap()],
            discovered_at: Instant::now(),
            discovery_source: source.clone(),
        };
        
        discovery.add_discovered_peer(peer);
        
        let discovered = &discovery.get_discovered_peers()[0];
        assert_eq!(discovered.discovery_source, source);
    }

    #[test]
    fn test_old_discovery_cleanup() {
        let mut discovery = Discovery::new(DiscoveryStrategy::Mdns);
        
        // Add peer with old timestamp
        let old_peer = DiscoveredPeer {
            peer_id: PeerId::random(),
            addresses: vec![],
            discovered_at: Instant::now() - Duration::from_secs(3600),
            discovery_source: DiscoverySource::Mdns,
        };
        
        // Add recent peer
        let new_peer = DiscoveredPeer {
            peer_id: PeerId::random(),
            addresses: vec![],
            discovered_at: Instant::now(),
            discovery_source: DiscoverySource::Mdns,
        };
        
        discovery.add_discovered_peer(old_peer);
        discovery.add_discovered_peer(new_peer);
        
        assert_eq!(discovery.peer_count(), 2);
        
        // Clean up peers older than 30 minutes
        discovery.clear_old_discoveries(Duration::from_secs(1800));
        
        assert_eq!(discovery.peer_count(), 1);
    }

    proptest! {
        #[test]
        fn test_discovery_consistency(
            peer_count in 1..100usize,
            remove_count in 0..50usize,
        ) {
            let mut discovery = Discovery::new(DiscoveryStrategy::Mdns);
            let mut peer_ids = Vec::new();
            
            // Add peers
            for _ in 0..peer_count {
                let peer = DiscoveredPeer {
                    peer_id: PeerId::random(),
                    addresses: vec!["/ip4/127.0.0.1/tcp/8000".parse().unwrap()],
                    discovered_at: Instant::now(),
                    discovery_source: DiscoverySource::Mdns,
                };
                peer_ids.push(peer.peer_id);
                discovery.add_discovered_peer(peer);
            }
            
            assert_eq!(discovery.peer_count(), peer_count);
            
            // Remove some peers
            let to_remove = remove_count.min(peer_count);
            for i in 0..to_remove {
                discovery.remove_peer(&peer_ids[i]);
            }
            
            assert_eq!(discovery.peer_count(), peer_count - to_remove);
            
            // Verify consistency
            assert_eq!(
                discovery.discovered_peers.len(),
                discovery.peer_info.len()
            );
        }
        
        #[test]
        fn test_discovery_interval(
            interval_secs in 1u64..300u64,
            elapsed_secs in 0u64..600u64,
        ) {
            let mut discovery = Discovery::new(DiscoveryStrategy::Mdns);
            discovery.discovery_interval = Duration::from_secs(interval_secs);
            
            // Simulate time passing
            discovery.last_discovery = Instant::now() - Duration::from_secs(elapsed_secs);
            
            let should_discover = discovery.should_discover();
            
            if elapsed_secs >= interval_secs {
                assert!(should_discover);
            } else {
                assert!(!should_discover);
            }
        }
    }
}