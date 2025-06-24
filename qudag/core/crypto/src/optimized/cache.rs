//! Key caching for improved crypto performance

use std::sync::Arc;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use ring::digest;

/// Key cache for frequently used cryptographic keys
pub struct KeyCache {
    /// LRU cache for keys
    cache: RwLock<lru::LruCache<KeyHash, CachedKey>>,
    /// Cache statistics
    stats: CacheStats,
}

/// Hash of a key for cache lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyHash([u8; 32]);

impl KeyHash {
    /// Create a key hash from bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        let digest = digest::digest(&digest::SHA256, data);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(digest.as_ref());
        KeyHash(hash)
    }
}

/// Cached key with metadata
#[derive(Clone)]
pub struct CachedKey {
    /// The actual key data
    pub data: Vec<u8>,
    /// When the key was cached
    pub cached_at: Instant,
    /// How many times the key has been accessed
    pub access_count: usize,
    /// Key type for categorization
    pub key_type: KeyType,
}

/// Type of cryptographic key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// ML-KEM public key
    MlKemPublic,
    /// ML-KEM secret key
    MlKemSecret,
    /// Transport encryption key
    Transport,
    /// Derived symmetric key
    Symmetric,
}

#[derive(Default)]
struct CacheStats {
    hits: std::sync::atomic::AtomicUsize,
    misses: std::sync::atomic::AtomicUsize,
    evictions: std::sync::atomic::AtomicUsize,
}

impl KeyCache {
    /// Create a new key cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(lru::LruCache::new(capacity.try_into().unwrap())),
            stats: CacheStats::default(),
        }
    }

    /// Insert a key into the cache
    pub fn insert(&self, key_data: &[u8], key_type: KeyType) -> KeyHash {
        let key_hash = KeyHash::from_bytes(key_data);
        let cached_key = CachedKey {
            data: key_data.to_vec(),
            cached_at: Instant::now(),
            access_count: 0,
            key_type,
        };

        let mut cache = self.cache.write();
        if cache.put(key_hash, cached_key).is_some() {
            self.stats.evictions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        key_hash
    }

    /// Get a key from the cache
    pub fn get(&self, key_hash: &KeyHash) -> Option<CachedKey> {
        let mut cache = self.cache.write();
        if let Some(mut cached_key) = cache.get_mut(key_hash) {
            cached_key.access_count += 1;
            self.stats.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(cached_key.clone())
        } else {
            self.stats.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    /// Check if a key exists in the cache
    pub fn contains(&self, key_hash: &KeyHash) -> bool {
        let cache = self.cache.read();
        cache.contains(key_hash)
    }

    /// Remove expired keys from the cache
    pub fn cleanup_expired(&self, max_age: Duration) {
        let mut cache = self.cache.write();
        let now = Instant::now();
        
        // Collect expired keys
        let expired_keys: Vec<KeyHash> = cache
            .iter()
            .filter_map(|(hash, key)| {
                if now.duration_since(key.cached_at) > max_age {
                    Some(*hash)
                } else {
                    None
                }
            })
            .collect();

        // Remove expired keys
        for key_hash in expired_keys {
            cache.pop(&key_hash);
            self.stats.evictions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStatistics {
        let hits = self.stats.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.stats.misses.load(std::sync::atomic::Ordering::Relaxed);
        let evictions = self.stats.evictions.load(std::sync::atomic::Ordering::Relaxed);
        
        let total_requests = hits + misses;
        let hit_rate = if total_requests > 0 {
            hits as f64 / total_requests as f64
        } else {
            0.0
        };

        let cache_size = self.cache.read().len();

        CacheStatistics {
            hits,
            misses,
            evictions,
            hit_rate,
            current_size: cache_size,
        }
    }

    /// Clear all cached keys
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        self.cache.read().cap().into()
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }
}

/// Statistics for cache performance monitoring
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub hit_rate: f64,
    pub current_size: usize,
}

impl std::fmt::Display for CacheStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {} hits, {} misses, {:.2}% hit rate, {} evictions, {} items",
            self.hits, self.misses, self.hit_rate * 100.0, self.evictions, self.current_size
        )
    }
}

/// Precomputed key context for faster operations
pub struct PrecomputedKeyContext {
    /// The key hash for cache lookups
    pub key_hash: KeyHash,
    /// Precomputed values for faster operations
    pub precomputed_values: Vec<u8>,
    /// Key derivation context
    pub derivation_context: Option<Vec<u8>>,
}

impl PrecomputedKeyContext {
    /// Create a new precomputed context for ML-KEM operations
    pub fn for_ml_kem(public_key: &[u8]) -> Self {
        let key_hash = KeyHash::from_bytes(public_key);
        
        // Precompute values that can speed up ML-KEM operations
        let mut precomputed_values = Vec::with_capacity(1024);
        
        // Example: precompute polynomial multiplication tables
        // This is a simplified example - real implementation would
        // precompute NTT roots, Montgomery constants, etc.
        for i in 0..256 {
            let val = ((i * 31) % 3329) as u8; // Simplified computation
            precomputed_values.push(val);
        }

        Self {
            key_hash,
            precomputed_values,
            derivation_context: None,
        }
    }

    /// Create a context for transport keys
    pub fn for_transport(key_data: &[u8], derivation_info: &[u8]) -> Self {
        let key_hash = KeyHash::from_bytes(key_data);
        
        Self {
            key_hash,
            precomputed_values: Vec::new(),
            derivation_context: Some(derivation_info.to_vec()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_cache_basic() {
        let cache = KeyCache::new(10);
        let key_data = b"test key data";
        
        // Insert key
        let key_hash = cache.insert(key_data, KeyType::MlKemPublic);
        
        // Retrieve key
        let cached_key = cache.get(&key_hash).unwrap();
        assert_eq!(cached_key.data, key_data);
        assert_eq!(cached_key.key_type, KeyType::MlKemPublic);
        assert_eq!(cached_key.access_count, 1);
        
        // Retrieve again - access count should increase
        let cached_key2 = cache.get(&key_hash).unwrap();
        assert_eq!(cached_key2.access_count, 2);
    }

    #[test]
    fn test_key_hash() {
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different data";
        
        let hash1 = KeyHash::from_bytes(data1);
        let hash2 = KeyHash::from_bytes(data2);
        let hash3 = KeyHash::from_bytes(data3);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_stats() {
        let cache = KeyCache::new(10);
        let key_data = b"test key";
        
        // Should be a miss
        let key_hash = KeyHash::from_bytes(key_data);
        assert!(cache.get(&key_hash).is_none());
        
        // Insert and hit
        let key_hash = cache.insert(key_data, KeyType::Symmetric);
        let _cached = cache.get(&key_hash).unwrap();
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = KeyCache::new(10);
        let key_data = b"test key";
        
        let key_hash = cache.insert(key_data, KeyType::Transport);
        assert!(cache.contains(&key_hash));
        
        // Cleanup with very short max age should remove the key
        cache.cleanup_expired(Duration::from_nanos(1));
        std::thread::sleep(Duration::from_millis(1));
        cache.cleanup_expired(Duration::from_nanos(1));
        
        assert!(!cache.contains(&key_hash));
    }

    #[test]
    fn test_precomputed_context() {
        let public_key = vec![1u8; 1184]; // ML-KEM-768 public key size
        let context = PrecomputedKeyContext::for_ml_kem(&public_key);
        
        assert_eq!(context.precomputed_values.len(), 256);
        assert!(context.derivation_context.is_none());
        
        let transport_key = vec![2u8; 32];
        let derivation_info = b"transport key derivation";
        let transport_context = PrecomputedKeyContext::for_transport(&transport_key, derivation_info);
        
        assert!(transport_context.derivation_context.is_some());
    }
}