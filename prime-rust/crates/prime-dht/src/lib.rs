//! Kademlia-based DHT implementation for Prime distributed ML

use daa_prime_core::{Error, Result};
use libp2p::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod routing;
pub mod storage;
pub mod discovery;

/// DHT configuration
#[derive(Debug, Clone)]
pub struct DhtConfig {
    pub k_bucket_size: usize,
    pub alpha: usize,
    pub replication_factor: usize,
    pub refresh_interval: std::time::Duration,
    pub ttl: std::time::Duration,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            k_bucket_size: 20,
            alpha: 3,
            replication_factor: 3,
            refresh_interval: std::time::Duration::from_secs(3600),
            ttl: std::time::Duration::from_secs(86400),
        }
    }
}

/// Main DHT implementation
pub struct Dht {
    peer_id: PeerId,
    config: DhtConfig,
    storage: Arc<RwLock<storage::Storage>>,
    routing_table: Arc<RwLock<routing::RoutingTable>>,
}

impl Dht {
    pub fn new(peer_id: PeerId, config: DhtConfig) -> Self {
        Self {
            peer_id,
            config: config.clone(),
            storage: Arc::new(RwLock::new(storage::Storage::new())),
            routing_table: Arc::new(RwLock::new(routing::RoutingTable::new(peer_id, config.k_bucket_size))),
        }
    }

    pub async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        // Store locally
        self.storage.write().await.put(key.clone(), value.clone());
        
        // Find k closest nodes
        let target = self.key_to_peer_id(&key);
        let closest = self.routing_table.read().await.find_closest(&target, self.config.replication_factor);
        
        // Replicate to closest nodes
        // TODO: Implement actual replication
        
        Ok(())
    }

    pub async fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        // Check local storage first
        if let Some(value) = self.storage.read().await.get(&key) {
            return Ok(Some(value));
        }
        
        // Query network
        let target = self.key_to_peer_id(&key);
        let closest = self.routing_table.read().await.find_closest(&target, self.config.alpha);
        
        // TODO: Implement actual network query
        
        Ok(None)
    }

    fn key_to_peer_id(&self, key: &[u8]) -> PeerId {
        // Convert key to PeerId for routing
        // This is a simplified version
        use std::convert::TryFrom;
        let hash = libp2p::multihash::Multihash::wrap(0x12, key).unwrap();
        PeerId::from_multihash(hash).unwrap_or(self.peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use quickcheck::{Arbitrary, Gen};

    #[tokio::test]
    async fn test_dht_creation() {
        let peer_id = PeerId::random();
        let config = DhtConfig::default();
        let dht = Dht::new(peer_id, config);
        
        assert_eq!(dht.peer_id, peer_id);
        assert_eq!(dht.config.k_bucket_size, 20);
    }

    #[tokio::test]
    async fn test_basic_put_get() {
        let peer_id = PeerId::random();
        let dht = Dht::new(peer_id, DhtConfig::default());
        
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();
        
        dht.put(key.clone(), value.clone()).await.unwrap();
        let retrieved = dht.get(key).await.unwrap();
        
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_multiple_entries() {
        let peer_id = PeerId::random();
        let dht = Dht::new(peer_id, DhtConfig::default());
        
        let entries = vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value2".to_vec()),
            (b"key3".to_vec(), b"value3".to_vec()),
        ];
        
        for (key, value) in &entries {
            dht.put(key.clone(), value.clone()).await.unwrap();
        }
        
        for (key, expected_value) in &entries {
            let retrieved = dht.get(key.clone()).await.unwrap();
            assert_eq!(retrieved, Some(expected_value.clone()));
        }
    }

    proptest! {
        #[test]
        fn test_dht_config_validation(
            k_bucket in 5usize..100usize,
            alpha in 1usize..10usize,
            replication in 1usize..20usize,
        ) {
            let config = DhtConfig {
                k_bucket_size: k_bucket,
                alpha,
                replication_factor: replication,
                refresh_interval: std::time::Duration::from_secs(3600),
                ttl: std::time::Duration::from_secs(86400),
            };
            
            assert!(config.k_bucket_size >= 5);
            assert!(config.alpha >= 1);
            assert!(config.replication_factor >= 1);
            assert!(config.alpha <= config.k_bucket_size);
        }
    }

    // QuickCheck test for key-value operations
    impl Arbitrary for DhtConfig {
        fn arbitrary(g: &mut Gen) -> Self {
            DhtConfig {
                k_bucket_size: *g.choose(&[10, 20, 30]).unwrap(),
                alpha: *g.choose(&[3, 5, 7]).unwrap(),
                replication_factor: *g.choose(&[1, 3, 5]).unwrap(),
                refresh_interval: std::time::Duration::from_secs(*g.choose(&[1800, 3600, 7200]).unwrap()),
                ttl: std::time::Duration::from_secs(*g.choose(&[3600, 86400, 604800]).unwrap()),
            }
        }
    }

    #[quickcheck_macros::quickcheck]
    async fn test_put_get_properties(key: Vec<u8>, value: Vec<u8>) -> bool {
        if key.is_empty() || value.is_empty() {
            return true; // Skip empty inputs
        }
        
        let peer_id = PeerId::random();
        let dht = Dht::new(peer_id, DhtConfig::default());
        
        // Put should succeed
        dht.put(key.clone(), value.clone()).await.is_ok() &&
        // Get should return the same value
        dht.get(key.clone()).await.unwrap() == Some(value)
    }
}