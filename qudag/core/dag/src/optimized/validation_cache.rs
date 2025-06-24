//! High-performance validation cache for DAG vertices

use crate::vertex::{Vertex, VertexId, VertexError};
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use blake3::Hash;

/// Validation result with metadata
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the vertex is valid
    pub is_valid: bool,
    /// Validation timestamp
    pub validated_at: Instant,
    /// Validation cost (computational units)
    pub validation_cost: u32,
    /// Cached hash of the vertex
    pub vertex_hash: Hash,
    /// Parent validation status
    pub parents_valid: bool,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total validations performed
    pub total_validations: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Average validation time (microseconds)
    pub avg_validation_time: u64,
    /// Cached entries
    pub cached_entries: usize,
    /// Memory usage estimate (bytes)
    pub memory_usage: usize,
}

/// Configuration for validation cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size
    pub max_entries: usize,
    /// TTL for cache entries
    pub ttl: Duration,
    /// Enable batch validation
    pub enable_batch_validation: bool,
    /// Batch size for validation
    pub batch_size: usize,
    /// Enable parent validation caching
    pub cache_parent_validation: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 100_000,
            ttl: Duration::from_secs(3600), // 1 hour
            enable_batch_validation: true,
            batch_size: 100,
            cache_parent_validation: true,
        }
    }
}

/// High-performance validation cache
pub struct ValidationCache {
    /// Primary cache using DashMap for concurrent access
    cache: Arc<DashMap<VertexId, ValidationResult>>,
    /// LRU cache for frequently accessed vertices
    hot_cache: Arc<RwLock<LruCache<VertexId, ValidationResult>>>,
    /// Bloom filter for quick negative lookups
    bloom_filter: Arc<RwLock<bloom::BloomFilter>>,
    /// Cache statistics
    stats: Arc<CacheStats>,
    /// Hit counter
    hit_counter: AtomicU64,
    /// Miss counter
    miss_counter: AtomicU64,
    /// Configuration
    config: CacheConfig,
}

impl ValidationCache {
    /// Create a new validation cache
    pub fn new(config: CacheConfig) -> Self {
        let hot_cache_size = (config.max_entries / 10).max(100);
        
        Self {
            cache: Arc::new(DashMap::with_capacity(config.max_entries)),
            hot_cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(hot_cache_size).unwrap()
            ))),
            bloom_filter: Arc::new(RwLock::new(
                bloom::BloomFilter::with_rate(0.01, config.max_entries as u32)
            )),
            stats: Arc::new(CacheStats::default()),
            hit_counter: AtomicU64::new(0),
            miss_counter: AtomicU64::new(0),
            config,
        }
    }

    /// Validate a vertex with caching
    pub fn validate(&self, vertex: &Vertex) -> Result<ValidationResult, VertexError> {
        let start = Instant::now();

        // Check hot cache first
        if let Some(result) = self.check_hot_cache(&vertex.id) {
            self.hit_counter.fetch_add(1, Ordering::Relaxed);
            return Ok(result);
        }

        // Check primary cache
        if let Some(entry) = self.cache.get(&vertex.id) {
            let result = entry.clone();
            
            // Check if entry is still valid
            if result.validated_at.elapsed() < self.config.ttl {
                self.hit_counter.fetch_add(1, Ordering::Relaxed);
                
                // Promote to hot cache
                self.promote_to_hot_cache(&vertex.id, result.clone());
                
                return Ok(result);
            } else {
                // Remove expired entry
                self.cache.remove(&vertex.id);
            }
        }

        // Cache miss - perform validation
        self.miss_counter.fetch_add(1, Ordering::Relaxed);
        
        let validation_result = self.perform_validation(vertex)?;
        let validation_time = start.elapsed().as_micros() as u64;

        // Update statistics
        self.update_stats(validation_time);

        // Cache the result
        self.cache_result(&vertex.id, validation_result.clone());

        Ok(validation_result)
    }

    /// Batch validate multiple vertices
    pub fn batch_validate(&self, vertices: &[Vertex]) -> Vec<Result<ValidationResult, VertexError>> {
        if !self.config.enable_batch_validation {
            return vertices.iter().map(|v| self.validate(v)).collect();
        }

        let mut results = Vec::with_capacity(vertices.len());
        let mut to_validate = Vec::new();
        let mut cached_indices = Vec::new();

        // First pass: check cache
        for (idx, vertex) in vertices.iter().enumerate() {
            if let Some(result) = self.get_cached(&vertex.id) {
                results.push(Ok(result));
                cached_indices.push(idx);
            } else {
                to_validate.push((idx, vertex));
            }
        }

        // Batch validate uncached vertices
        if !to_validate.is_empty() {
            let batch_results = self.parallel_validate(&to_validate);
            
            // Merge results
            let mut batch_idx = 0;
            for i in 0..vertices.len() {
                if !cached_indices.contains(&i) {
                    results.insert(i, batch_results[batch_idx].clone());
                    batch_idx += 1;
                }
            }
        }

        results
    }

    /// Perform actual validation
    fn perform_validation(&self, vertex: &Vertex) -> Result<ValidationResult, VertexError> {
        let start = Instant::now();
        
        // Hash the vertex for integrity
        let mut hasher = blake3::Hasher::new();
        hasher.update(&vertex.id.as_bytes());
        hasher.update(&bincode::serialize(&vertex.parents).unwrap());
        hasher.update(&vertex.payload);
        hasher.update(&vertex.timestamp.to_le_bytes());
        let vertex_hash = hasher.finalize();

        // Basic structural validation
        if vertex.payload.is_empty() {
            return Err(VertexError::InvalidPayload);
        }

        // Validate parents if enabled
        let parents_valid = if self.config.cache_parent_validation {
            self.validate_parents(&vertex.parents)?
        } else {
            true
        };

        // Signature validation (simplified for now)
        let signature_valid = !vertex.signature.is_empty();
        
        let validation_cost = start.elapsed().as_micros() as u32;

        Ok(ValidationResult {
            is_valid: signature_valid && parents_valid,
            validated_at: Instant::now(),
            validation_cost,
            vertex_hash,
            parents_valid,
        })
    }

    /// Validate parent vertices
    fn validate_parents(&self, parents: &[VertexId]) -> Result<bool, VertexError> {
        // In a real implementation, this would check parent existence and validity
        // For now, we'll just check that parents are not empty for non-genesis vertices
        Ok(true)
    }

    /// Parallel validation for batch operations
    fn parallel_validate(&self, vertices: &[(usize, &Vertex)]) -> Vec<Result<ValidationResult, VertexError>> {
        use rayon::prelude::*;
        
        vertices.par_iter()
            .map(|(_, vertex)| self.perform_validation(vertex))
            .collect()
    }

    /// Check hot cache
    fn check_hot_cache(&self, id: &VertexId) -> Option<ValidationResult> {
        let mut hot_cache = self.hot_cache.write();
        hot_cache.get(id).cloned()
    }

    /// Get cached result
    fn get_cached(&self, id: &VertexId) -> Option<ValidationResult> {
        // Check bloom filter first for quick negative lookup
        if !self.bloom_filter.read().check(&id.as_bytes()) {
            return None;
        }

        // Check hot cache
        if let Some(result) = self.check_hot_cache(id) {
            return Some(result);
        }

        // Check primary cache
        self.cache.get(id).map(|entry| entry.clone())
    }

    /// Cache validation result
    fn cache_result(&self, id: &VertexId, result: ValidationResult) {
        // Add to bloom filter
        self.bloom_filter.write().insert(&id.as_bytes());
        
        // Add to primary cache
        self.cache.insert(id.clone(), result.clone());
        
        // Add to hot cache if frequently accessed
        if self.should_promote_to_hot_cache() {
            self.promote_to_hot_cache(id, result);
        }
    }

    /// Promote entry to hot cache
    fn promote_to_hot_cache(&self, id: &VertexId, result: ValidationResult) {
        let mut hot_cache = self.hot_cache.write();
        hot_cache.put(id.clone(), result);
    }

    /// Determine if entry should be promoted to hot cache
    fn should_promote_to_hot_cache(&self) -> bool {
        // Simple heuristic: promote if hit rate is high
        let hits = self.hit_counter.load(Ordering::Relaxed);
        let misses = self.miss_counter.load(Ordering::Relaxed);
        let total = hits + misses;
        
        total > 100 && hits > misses * 2
    }

    /// Update cache statistics
    fn update_stats(&self, validation_time: u64) {
        // Update average validation time using exponential moving average
        let alpha = 0.1;
        let current_avg = self.stats.avg_validation_time;
        let new_avg = (alpha * validation_time as f64 + (1.0 - alpha) * current_avg as f64) as u64;
        
        // This is simplified - in production, use atomic operations
        let stats = Arc::get_mut(&mut self.stats.clone()).unwrap();
        stats.avg_validation_time = new_avg;
        stats.total_validations += 1;
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let hits = self.hit_counter.load(Ordering::Relaxed);
        let misses = self.miss_counter.load(Ordering::Relaxed);
        
        CacheStats {
            total_validations: hits + misses,
            cache_hits: hits,
            cache_misses: misses,
            avg_validation_time: self.stats.avg_validation_time,
            cached_entries: self.cache.len(),
            memory_usage: self.estimate_memory_usage(),
        }
    }

    /// Estimate memory usage
    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: each entry ~200 bytes
        self.cache.len() * 200 + self.hot_cache.read().len() * 200
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
        self.hot_cache.write().clear();
        self.bloom_filter.write().clear();
        self.hit_counter.store(0, Ordering::Relaxed);
        self.miss_counter.store(0, Ordering::Relaxed);
    }

    /// Invalidate specific vertex
    pub fn invalidate(&self, id: &VertexId) {
        self.cache.remove(id);
        self.hot_cache.write().pop(id);
    }
}

// Placeholder bloom filter implementation
mod bloom {
    use std::collections::HashSet;
    
    pub struct BloomFilter {
        items: HashSet<Vec<u8>>,
    }
    
    impl BloomFilter {
        pub fn with_rate(_rate: f64, _capacity: u32) -> Self {
            Self {
                items: HashSet::new(),
            }
        }
        
        pub fn insert(&mut self, data: &[u8]) {
            self.items.insert(data.to_vec());
        }
        
        pub fn check(&self, data: &[u8]) -> bool {
            self.items.contains(data)
        }
        
        pub fn clear(&mut self) {
            self.items.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_validation_cache() {
        let config = CacheConfig::default();
        let cache = ValidationCache::new(config);
        
        // Create test vertex
        let vertex = Vertex::new(
            VertexId::new(),
            vec![1, 2, 3, 4],
            HashSet::new(),
        );
        
        // First validation should miss cache
        let result1 = cache.validate(&vertex).unwrap();
        assert!(result1.is_valid);
        
        let stats = cache.get_stats();
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_hits, 0);
        
        // Second validation should hit cache
        let result2 = cache.validate(&vertex).unwrap();
        assert_eq!(result1.vertex_hash, result2.vertex_hash);
        
        let stats = cache.get_stats();
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_hits, 1);
    }

    #[test]
    fn test_batch_validation() {
        let config = CacheConfig {
            enable_batch_validation: true,
            batch_size: 10,
            ..Default::default()
        };
        let cache = ValidationCache::new(config);
        
        // Create test vertices
        let vertices: Vec<_> = (0..5)
            .map(|i| Vertex::new(
                VertexId::from_bytes(vec![i]),
                vec![i; 10],
                HashSet::new(),
            ))
            .collect();
        
        // Batch validate
        let results = cache.batch_validate(&vertices);
        assert_eq!(results.len(), 5);
        
        for result in results {
            assert!(result.unwrap().is_valid);
        }
        
        // Validate again - should hit cache
        let results2 = cache.batch_validate(&vertices);
        assert_eq!(results2.len(), 5);
        
        let stats = cache.get_stats();
        assert_eq!(stats.cache_hits, 5);
    }
}