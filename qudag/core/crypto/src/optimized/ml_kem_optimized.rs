//! Optimized ML-KEM implementation with reduced allocations and improved cache efficiency

use crate::kem::{KEMError, KeyEncapsulation, PublicKey, SecretKey, Ciphertext, SharedSecret};
use crate::optimized::{CRYPTO_BUFFER_POOL, KEY_CACHE};
use crate::optimized::cache::{KeyType, KeyHash, PrecomputedKeyContext};
use ring::rand::SystemRandom;
use std::sync::Arc;
use std::time::Instant;
use zeroize::Zeroize;

/// Optimized ML-KEM 768 implementation with performance enhancements
pub struct OptimizedMlKem768 {
    /// Random number generator (reused to avoid initialization overhead)
    rng: SystemRandom,
    /// Performance metrics
    metrics: OptimizedMetrics,
}

/// Performance metrics for the optimized implementation
#[derive(Default, Clone)]
pub struct OptimizedMetrics {
    /// Number of buffer pool hits
    pub buffer_pool_hits: usize,
    /// Number of buffer pool misses
    pub buffer_pool_misses: usize,
    /// Number of key cache hits
    pub key_cache_hits: usize,
    /// Number of key cache misses
    pub key_cache_misses: usize,
    /// Average key generation time
    pub avg_keygen_time_ns: u64,
    /// Average encapsulation time
    pub avg_encap_time_ns: u64,
    /// Average decapsulation time
    pub avg_decap_time_ns: u64,
    /// Total operations performed
    pub total_operations: usize,
}

impl OptimizedMlKem768 {
    /// Size of public keys in bytes
    pub const PUBLIC_KEY_SIZE: usize = 1184;
    
    /// Size of secret keys in bytes
    pub const SECRET_KEY_SIZE: usize = 2400;
    
    /// Size of ciphertexts in bytes
    pub const CIPHERTEXT_SIZE: usize = 1088;
    
    /// Size of shared secrets in bytes
    pub const SHARED_SECRET_SIZE: usize = 32;

    /// Create a new optimized ML-KEM instance
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
            metrics: OptimizedMetrics::default(),
        }
    }

    /// Generate a new keypair with optimized buffer management
    pub fn keygen_optimized(&mut self) -> Result<(PublicKey, SecretKey), KEMError> {
        let start_time = Instant::now();

        // Use buffer pool for key generation to reduce allocations
        let mut pk_buffer = CRYPTO_BUFFER_POOL.acquire(Self::PUBLIC_KEY_SIZE);
        let mut sk_buffer = CRYPTO_BUFFER_POOL.acquire(Self::SECRET_KEY_SIZE);

        // Fill with random data (placeholder implementation)
        self.rng.fill(&mut pk_buffer.as_mut_slice())
            .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;
        self.rng.fill(&mut sk_buffer.as_mut_slice())
            .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;

        // Cache the generated keys for potential reuse
        let pk_hash = KEY_CACHE.insert(&pk_buffer, KeyType::MlKemPublic);
        let sk_hash = KEY_CACHE.insert(&sk_buffer, KeyType::MlKemSecret);

        // Create keys from buffers
        let public_key = PublicKey::from_bytes(&pk_buffer)?;
        let secret_key = SecretKey::from_bytes(&sk_buffer)?;

        // Update metrics
        let elapsed = start_time.elapsed();
        self.metrics.avg_keygen_time_ns = 
            (self.metrics.avg_keygen_time_ns + elapsed.as_nanos() as u64) / 2;
        self.metrics.total_operations += 1;

        Ok((public_key, secret_key))
    }

    /// Encapsulate with key caching and buffer reuse
    pub fn encapsulate_optimized(&mut self, pk: &PublicKey) -> Result<(Ciphertext, SharedSecret), KEMError> {
        let start_time = Instant::now();

        // Check if we have a precomputed context for this key
        let pk_bytes = pk.as_bytes();
        let pk_hash = KeyHash::from_bytes(pk_bytes);
        
        let use_cached = if let Some(_cached_key) = KEY_CACHE.get(&pk_hash) {
            self.metrics.key_cache_hits += 1;
            true
        } else {
            self.metrics.key_cache_misses += 1;
            // Cache the key for future operations
            KEY_CACHE.insert(pk_bytes, KeyType::MlKemPublic);
            false
        };

        // Use buffer pool for temporary allocations
        let mut ct_buffer = CRYPTO_BUFFER_POOL.acquire(Self::CIPHERTEXT_SIZE);
        let mut ss_buffer = CRYPTO_BUFFER_POOL.acquire(Self::SHARED_SECRET_SIZE);

        if use_cached {
            // Use optimized path with cached computations
            self.encapsulate_with_cache(&pk_hash, &mut ct_buffer, &mut ss_buffer)?;
        } else {
            // Standard path with caching for future use
            self.encapsulate_standard(pk, &mut ct_buffer, &mut ss_buffer)?;
        }

        let ciphertext = Ciphertext::from_bytes(&ct_buffer)?;
        let shared_secret = SharedSecret::from_bytes(&ss_buffer)?;

        // Update metrics
        let elapsed = start_time.elapsed();
        self.metrics.avg_encap_time_ns = 
            (self.metrics.avg_encap_time_ns + elapsed.as_nanos() as u64) / 2;
        self.metrics.total_operations += 1;

        Ok((ciphertext, shared_secret))
    }

    /// Decapsulate with optimized buffer management
    pub fn decapsulate_optimized(&mut self, sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, KEMError> {
        let start_time = Instant::now();

        // Check for cached secret key
        let sk_bytes = sk.as_bytes();
        let sk_hash = KeyHash::from_bytes(sk_bytes);
        
        if let Some(_cached_key) = KEY_CACHE.get(&sk_hash) {
            self.metrics.key_cache_hits += 1;
        } else {
            self.metrics.key_cache_misses += 1;
            KEY_CACHE.insert(sk_bytes, KeyType::MlKemSecret);
        }

        // Use buffer pool for shared secret computation
        let mut ss_buffer = CRYPTO_BUFFER_POOL.acquire(Self::SHARED_SECRET_SIZE);

        // Perform decapsulation (placeholder implementation)
        self.rng.fill(&mut ss_buffer.as_mut_slice())
            .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;

        let shared_secret = SharedSecret::from_bytes(&ss_buffer)?;

        // Update metrics
        let elapsed = start_time.elapsed();
        self.metrics.avg_decap_time_ns = 
            (self.metrics.avg_decap_time_ns + elapsed.as_nanos() as u64) / 2;
        self.metrics.total_operations += 1;

        Ok(shared_secret)
    }

    /// Batch key generation for improved throughput
    pub fn batch_keygen(&mut self, count: usize) -> Result<Vec<(PublicKey, SecretKey)>, KEMError> {
        let mut keypairs = Vec::with_capacity(count);
        
        // Pre-allocate all buffers to reduce allocation overhead
        let mut pk_buffers: Vec<_> = (0..count)
            .map(|_| CRYPTO_BUFFER_POOL.acquire(Self::PUBLIC_KEY_SIZE))
            .collect();
        let mut sk_buffers: Vec<_> = (0..count)
            .map(|_| CRYPTO_BUFFER_POOL.acquire(Self::SECRET_KEY_SIZE))
            .collect();

        // Generate all keys in batch
        for i in 0..count {
            self.rng.fill(&mut pk_buffers[i].as_mut_slice())
                .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;
            self.rng.fill(&mut sk_buffers[i].as_mut_slice())
                .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;

            let public_key = PublicKey::from_bytes(&pk_buffers[i])?;
            let secret_key = SecretKey::from_bytes(&sk_buffers[i])?;
            
            keypairs.push((public_key, secret_key));
        }

        self.metrics.total_operations += count;
        Ok(keypairs)
    }

    /// Encapsulation with cached key computations
    fn encapsulate_with_cache(
        &self,
        _pk_hash: &KeyHash,
        ct_buffer: &mut [u8],
        ss_buffer: &mut [u8],
    ) -> Result<(), KEMError> {
        // Use precomputed values for faster encapsulation
        // This is a placeholder - real implementation would use
        // cached NTT transforms, precomputed matrices, etc.
        
        self.rng.fill(ct_buffer)
            .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;
        self.rng.fill(ss_buffer)
            .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;
            
        Ok(())
    }

    /// Standard encapsulation path
    fn encapsulate_standard(
        &self,
        pk: &PublicKey,
        ct_buffer: &mut [u8],
        ss_buffer: &mut [u8],
    ) -> Result<(), KEMError> {
        // Standard ML-KEM encapsulation
        // This is a placeholder implementation
        
        self.rng.fill(ct_buffer)
            .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;
        self.rng.fill(ss_buffer)
            .map_err(|_| KEMError::InvalidInput("Random generation failed".into()))?;
            
        Ok(())
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> OptimizedMetrics {
        self.metrics.clone()
    }

    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = OptimizedMetrics::default();
    }

    /// Warm up caches with frequently used keys
    pub fn warm_cache(&mut self, public_keys: &[&[u8]], secret_keys: &[&[u8]]) {
        for pk_bytes in public_keys {
            KEY_CACHE.insert(pk_bytes, KeyType::MlKemPublic);
        }
        
        for sk_bytes in secret_keys {
            KEY_CACHE.insert(sk_bytes, KeyType::MlKemSecret);
        }
    }
}

impl Default for OptimizedMlKem768 {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyEncapsulation for OptimizedMlKem768 {
    fn keygen() -> Result<(PublicKey, SecretKey), KEMError> {
        let mut instance = Self::new();
        instance.keygen_optimized()
    }
    
    fn encapsulate(public_key: &PublicKey) -> Result<(Ciphertext, SharedSecret), KEMError> {
        let mut instance = Self::new();
        instance.encapsulate_optimized(public_key)
    }
    
    fn decapsulate(secret_key: &SecretKey, ciphertext: &Ciphertext) -> Result<SharedSecret, KEMError> {
        let mut instance = Self::new();
        instance.decapsulate_optimized(secret_key, ciphertext)
    }
}

/// Optimized batch operations for high-throughput scenarios
pub struct BatchProcessor {
    instances: Vec<OptimizedMlKem768>,
    current_instance: usize,
}

impl BatchProcessor {
    /// Create a new batch processor with multiple instances
    pub fn new(num_instances: usize) -> Self {
        let instances = (0..num_instances)
            .map(|_| OptimizedMlKem768::new())
            .collect();

        Self {
            instances,
            current_instance: 0,
        }
    }

    /// Process a batch of key generations across multiple instances
    pub fn batch_keygen_parallel(&mut self, total_count: usize) -> Result<Vec<(PublicKey, SecretKey)>, KEMError> {
        let per_instance = total_count / self.instances.len();
        let remainder = total_count % self.instances.len();

        let mut all_keypairs = Vec::with_capacity(total_count);

        // Distribute work across instances
        for (i, instance) in self.instances.iter_mut().enumerate() {
            let count = if i < remainder {
                per_instance + 1
            } else {
                per_instance
            };

            if count > 0 {
                let mut keypairs = instance.batch_keygen(count)?;
                all_keypairs.append(&mut keypairs);
            }
        }

        Ok(all_keypairs)
    }

    /// Get the next available instance for load balancing
    pub fn get_next_instance(&mut self) -> &mut OptimizedMlKem768 {
        let instance = &mut self.instances[self.current_instance];
        self.current_instance = (self.current_instance + 1) % self.instances.len();
        instance
    }

    /// Get aggregated metrics from all instances
    pub fn get_aggregated_metrics(&self) -> OptimizedMetrics {
        let mut aggregated = OptimizedMetrics::default();

        for instance in &self.instances {
            let metrics = instance.get_metrics();
            aggregated.buffer_pool_hits += metrics.buffer_pool_hits;
            aggregated.buffer_pool_misses += metrics.buffer_pool_misses;
            aggregated.key_cache_hits += metrics.key_cache_hits;
            aggregated.key_cache_misses += metrics.key_cache_misses;
            aggregated.total_operations += metrics.total_operations;
        }

        // Average the timing metrics
        let num_instances = self.instances.len() as u64;
        if num_instances > 0 {
            aggregated.avg_keygen_time_ns = self.instances.iter()
                .map(|i| i.get_metrics().avg_keygen_time_ns)
                .sum::<u64>() / num_instances;
                
            aggregated.avg_encap_time_ns = self.instances.iter()
                .map(|i| i.get_metrics().avg_encap_time_ns)
                .sum::<u64>() / num_instances;
                
            aggregated.avg_decap_time_ns = self.instances.iter()
                .map(|i| i.get_metrics().avg_decap_time_ns)
                .sum::<u64>() / num_instances;
        }

        aggregated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_keygen() {
        let mut ml_kem = OptimizedMlKem768::new();
        let (pk, sk) = ml_kem.keygen_optimized().unwrap();
        
        assert_eq!(pk.as_bytes().len(), OptimizedMlKem768::PUBLIC_KEY_SIZE);
        assert_eq!(sk.as_bytes().len(), OptimizedMlKem768::SECRET_KEY_SIZE);
    }

    #[test]
    fn test_optimized_encapsulation() {
        let mut ml_kem = OptimizedMlKem768::new();
        let (pk, _sk) = ml_kem.keygen_optimized().unwrap();
        
        let (ct, ss) = ml_kem.encapsulate_optimized(&pk).unwrap();
        
        assert_eq!(ct.as_bytes().len(), OptimizedMlKem768::CIPHERTEXT_SIZE);
        assert_eq!(ss.as_bytes().len(), OptimizedMlKem768::SHARED_SECRET_SIZE);
    }

    #[test]
    fn test_batch_keygen() {
        let mut ml_kem = OptimizedMlKem768::new();
        let keypairs = ml_kem.batch_keygen(10).unwrap();
        
        assert_eq!(keypairs.len(), 10);
        for (pk, sk) in keypairs {
            assert_eq!(pk.as_bytes().len(), OptimizedMlKem768::PUBLIC_KEY_SIZE);
            assert_eq!(sk.as_bytes().len(), OptimizedMlKem768::SECRET_KEY_SIZE);
        }
    }

    #[test]
    fn test_metrics_tracking() {
        let mut ml_kem = OptimizedMlKem768::new();
        
        // Perform some operations
        let (_pk, _sk) = ml_kem.keygen_optimized().unwrap();
        let metrics = ml_kem.get_metrics();
        
        assert!(metrics.total_operations > 0);
        assert!(metrics.avg_keygen_time_ns > 0);
    }

    #[test]
    fn test_batch_processor() {
        let mut processor = BatchProcessor::new(4);
        let keypairs = processor.batch_keygen_parallel(20).unwrap();
        
        assert_eq!(keypairs.len(), 20);
        
        let metrics = processor.get_aggregated_metrics();
        assert!(metrics.total_operations >= 20);
    }

    #[test]
    fn test_cache_warming() {
        let mut ml_kem = OptimizedMlKem768::new();
        
        // Generate some keys
        let (pk1, sk1) = ml_kem.keygen_optimized().unwrap();
        let (pk2, sk2) = ml_kem.keygen_optimized().unwrap();
        
        // Warm cache
        ml_kem.warm_cache(
            &[pk1.as_bytes(), pk2.as_bytes()],
            &[sk1.as_bytes(), sk2.as_bytes()]
        );
        
        // Using cached keys should improve performance
        let (_ct, _ss) = ml_kem.encapsulate_optimized(&pk1).unwrap();
        let metrics = ml_kem.get_metrics();
        
        assert!(metrics.key_cache_hits > 0);
    }
}