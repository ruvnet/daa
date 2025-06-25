//! Kademlia routing table implementation

use libp2p::PeerId;
use std::collections::BTreeMap;

/// K-bucket for storing peers at a specific distance
#[derive(Debug, Clone)]
pub struct KBucket {
    peers: Vec<PeerId>,
    max_size: usize,
}

impl KBucket {
    pub fn new(max_size: usize) -> Self {
        Self {
            peers: Vec::new(),
            max_size,
        }
    }

    pub fn add_peer(&mut self, peer: PeerId) -> bool {
        if self.peers.contains(&peer) {
            // Move to end (most recently seen)
            self.peers.retain(|p| p != &peer);
            self.peers.push(peer);
            return true;
        }

        if self.peers.len() < self.max_size {
            self.peers.push(peer);
            true
        } else {
            // Bucket full, could implement replacement policy
            false
        }
    }

    pub fn remove_peer(&mut self, peer: &PeerId) {
        self.peers.retain(|p| p != peer);
    }

    pub fn get_peers(&self) -> &[PeerId] {
        &self.peers
    }

    pub fn is_full(&self) -> bool {
        self.peers.len() >= self.max_size
    }
}

/// Kademlia routing table
pub struct RoutingTable {
    local_peer: PeerId,
    buckets: BTreeMap<u32, KBucket>,
    k_bucket_size: usize,
}

impl RoutingTable {
    pub fn new(local_peer: PeerId, k_bucket_size: usize) -> Self {
        Self {
            local_peer,
            buckets: BTreeMap::new(),
            k_bucket_size,
        }
    }

    pub fn add_peer(&mut self, peer: PeerId) {
        if peer == self.local_peer {
            return;
        }

        let distance = self.distance(&peer);
        let bucket = self.buckets
            .entry(distance)
            .or_insert_with(|| KBucket::new(self.k_bucket_size));
        
        bucket.add_peer(peer);
    }

    pub fn remove_peer(&mut self, peer: &PeerId) {
        let distance = self.distance(peer);
        if let Some(bucket) = self.buckets.get_mut(&distance) {
            bucket.remove_peer(peer);
        }
    }

    pub fn find_closest(&self, target: &PeerId, count: usize) -> Vec<PeerId> {
        let mut closest = Vec::new();
        
        // Get target distance
        let target_distance = self.distance(target);
        
        // Search buckets starting from target distance
        for i in 0..160 {
            for direction in &[0i32, 1, -1] {
                let bucket_idx = (target_distance as i32 + i as i32 * direction) as u32;
                
                if let Some(bucket) = self.buckets.get(&bucket_idx) {
                    for peer in bucket.get_peers() {
                        if !closest.contains(peer) {
                            closest.push(peer.clone());
                            if closest.len() >= count {
                                return closest;
                            }
                        }
                    }
                }
            }
        }
        
        closest
    }

    fn distance(&self, peer: &PeerId) -> u32 {
        // Calculate XOR distance (simplified)
        let local_bytes = self.local_peer.to_bytes();
        let peer_bytes = peer.to_bytes();
        
        for i in 0..local_bytes.len().min(peer_bytes.len()) {
            let xor = local_bytes[i] ^ peer_bytes[i];
            if xor != 0 {
                return (i as u32) * 8 + (8 - xor.leading_zeros());
            }
        }
        
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use test_case::test_case;

    #[test]
    fn test_k_bucket_operations() {
        let mut bucket = KBucket::new(3);
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();
        let peer4 = PeerId::random();
        
        assert!(bucket.add_peer(peer1));
        assert!(bucket.add_peer(peer2));
        assert!(bucket.add_peer(peer3));
        assert!(bucket.is_full());
        assert!(!bucket.add_peer(peer4)); // Should fail, bucket full
        
        bucket.remove_peer(&peer2);
        assert!(!bucket.is_full());
        assert!(bucket.add_peer(peer4)); // Should succeed now
    }

    #[test]
    fn test_routing_table_basic() {
        let local_peer = PeerId::random();
        let mut table = RoutingTable::new(local_peer, 20);
        
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        
        table.add_peer(peer1);
        table.add_peer(peer2);
        
        let closest = table.find_closest(&peer1, 10);
        assert!(closest.contains(&peer1));
    }

    #[test_case(1 ; "find 1 peer")]
    #[test_case(5 ; "find 5 peers")]
    #[test_case(20 ; "find 20 peers")]
    fn test_find_closest_with_count(count: usize) {
        let local_peer = PeerId::random();
        let mut table = RoutingTable::new(local_peer, 20);
        
        // Add many peers
        let peers: Vec<PeerId> = (0..50).map(|_| PeerId::random()).collect();
        for peer in &peers {
            table.add_peer(peer.clone());
        }
        
        let target = PeerId::random();
        let closest = table.find_closest(&target, count);
        
        assert!(closest.len() <= count);
        assert!(closest.len() <= peers.len());
    }

    proptest! {
        #[test]
        fn test_routing_table_consistency(
            peer_count in 10..100usize,
            k_bucket_size in 5..30usize,
        ) {
            let local_peer = PeerId::random();
            let mut table = RoutingTable::new(local_peer, k_bucket_size);
            
            let peers: Vec<PeerId> = (0..peer_count)
                .map(|_| PeerId::random())
                .collect();
            
            // Add all peers
            for peer in &peers {
                table.add_peer(peer.clone());
            }
            
            // Verify we can find peers
            let found = table.find_closest(&peers[0], peer_count);
            assert!(!found.is_empty());
            assert!(found.len() <= peer_count);
            
            // Verify no duplicates
            let mut unique = found.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(found.len(), unique.len());
        }
        
        #[test]
        fn test_k_bucket_replacement_policy(
            operations in prop::collection::vec(
                prop_oneof![
                    Just(true),  // Add
                    Just(false), // Remove
                ],
                0..50
            )
        ) {
            let mut bucket = KBucket::new(10);
            let mut peers = Vec::new();
            
            for (i, is_add) in operations.iter().enumerate() {
                if *is_add {
                    let peer = PeerId::random();
                    peers.push(peer);
                    bucket.add_peer(peer);
                } else if !peers.is_empty() {
                    let idx = i % peers.len();
                    bucket.remove_peer(&peers[idx]);
                }
                
                // Verify invariants
                assert!(bucket.peers.len() <= bucket.max_size);
            }
        }
    }
}