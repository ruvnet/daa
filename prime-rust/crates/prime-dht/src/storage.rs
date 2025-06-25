//! DHT storage implementation with TTL and persistence

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Storage entry with TTL
#[derive(Debug, Clone)]
pub struct StorageEntry {
    pub value: Vec<u8>,
    pub timestamp: Instant,
    pub ttl: Duration,
}

impl StorageEntry {
    pub fn new(value: Vec<u8>, ttl: Duration) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }
}

/// DHT storage backend
pub struct Storage {
    entries: HashMap<Vec<u8>, StorageEntry>,
    max_size: usize,
}

impl Storage {
    pub fn new() -> Self {
        Self::with_max_size(10_000)
    }

    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.put_with_ttl(key, value, Duration::from_secs(86400))
    }

    pub fn put_with_ttl(&mut self, key: Vec<u8>, value: Vec<u8>, ttl: Duration) {
        // Remove expired entries if we're at capacity
        if self.entries.len() >= self.max_size {
            self.cleanup_expired();
        }

        // If still at capacity, remove oldest entry
        if self.entries.len() >= self.max_size {
            if let Some(oldest_key) = self.find_oldest_key() {
                self.entries.remove(&oldest_key);
            }
        }

        self.entries.insert(key, StorageEntry::new(value, ttl));
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.entries.get(key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.value.clone())
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.entries.remove(key).map(|entry| entry.value)
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        self.entries.get(key)
            .map(|entry| !entry.is_expired())
            .unwrap_or(false)
    }

    pub fn cleanup_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn find_oldest_key(&self) -> Option<Vec<u8>> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(key, _)| key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use test_case::test_case;

    #[test]
    fn test_basic_storage_operations() {
        let mut storage = Storage::new();
        
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();
        
        storage.put(key.clone(), value.clone());
        assert_eq!(storage.get(&key), Some(value.clone()));
        assert!(storage.contains(&key));
        
        assert_eq!(storage.remove(&key), Some(value));
        assert_eq!(storage.get(&key), None);
        assert!(!storage.contains(&key));
    }

    #[test]
    fn test_ttl_expiration() {
        let mut storage = Storage::new();
        
        let key = b"ttl_key".to_vec();
        let value = b"ttl_value".to_vec();
        
        // Put with very short TTL
        storage.put_with_ttl(key.clone(), value.clone(), Duration::from_millis(1));
        
        // Should be available immediately
        assert_eq!(storage.get(&key), Some(value));
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(2));
        
        // Should be expired now
        assert_eq!(storage.get(&key), None);
        assert!(!storage.contains(&key));
    }

    #[test]
    fn test_max_size_enforcement() {
        let mut storage = Storage::with_max_size(3);
        
        storage.put(b"key1".to_vec(), b"value1".to_vec());
        storage.put(b"key2".to_vec(), b"value2".to_vec());
        storage.put(b"key3".to_vec(), b"value3".to_vec());
        
        assert_eq!(storage.len(), 3);
        
        // Adding fourth item should evict oldest
        storage.put(b"key4".to_vec(), b"value4".to_vec());
        assert_eq!(storage.len(), 3);
        
        // key1 should be evicted
        assert!(!storage.contains(b"key1"));
        assert!(storage.contains(b"key4"));
    }

    #[test_case(0, 10 ; "empty storage")]
    #[test_case(5, 10 ; "half full")]
    #[test_case(10, 10 ; "full storage")]
    fn test_storage_capacity(initial_items: usize, max_size: usize) {
        let mut storage = Storage::with_max_size(max_size);
        
        for i in 0..initial_items {
            let key = format!("key_{}", i).into_bytes();
            let value = format!("value_{}", i).into_bytes();
            storage.put(key, value);
        }
        
        assert_eq!(storage.len(), initial_items.min(max_size));
    }

    proptest! {
        #[test]
        fn test_storage_consistency(
            operations in prop::collection::vec(
                (
                    prop::collection::vec(0u8..255, 1..10), // key
                    prop::collection::vec(0u8..255, 1..100), // value
                    prop::bool::ANY, // is_put (true) or get (false)
                ),
                0..100
            )
        ) {
            let mut storage = Storage::with_max_size(50);
            let mut expected = HashMap::new();
            
            for (key, value, is_put) in operations {
                if is_put {
                    storage.put(key.clone(), value.clone());
                    expected.insert(key, value);
                    
                    // Maintain max size in expected map
                    if expected.len() > 50 {
                        // Remove arbitrary item (in real impl it would be oldest)
                        let to_remove = expected.keys().next().cloned().unwrap();
                        expected.remove(&to_remove);
                    }
                } else {
                    let stored = storage.get(&key);
                    let expected_value = expected.get(&key).cloned();
                    
                    // If we expect a value, it should match
                    if let Some(exp_val) = expected_value {
                        assert_eq!(stored, Some(exp_val));
                    }
                }
            }
            
            // Storage size should not exceed max
            assert!(storage.len() <= 50);
        }
        
        #[test]
        fn test_ttl_properties(
            ttl_ms in 1u64..1000u64,
            wait_ms in 0u64..2000u64,
        ) {
            let mut storage = Storage::new();
            let key = b"ttl_test".to_vec();
            let value = b"ttl_value".to_vec();
            
            storage.put_with_ttl(
                key.clone(), 
                value.clone(), 
                Duration::from_millis(ttl_ms)
            );
            
            if wait_ms < ttl_ms {
                // Should not be expired yet
                assert!(storage.contains(&key));
            } else {
                // Might be expired (allowing some timing variance)
                std::thread::sleep(Duration::from_millis(wait_ms.saturating_sub(ttl_ms) + 1));
                assert!(!storage.contains(&key));
            }
        }
    }
}