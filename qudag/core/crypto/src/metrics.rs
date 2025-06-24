use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use std::sync::Arc;

/// Cryptographic operation metrics
#[derive(Debug, Default)]
pub struct CryptoMetrics {
    /// Key operations counter
    pub key_operations: AtomicU64,
    /// Key cache hits
    pub key_cache_hits: AtomicU64,
    /// Key cache misses
    pub key_cache_misses: AtomicU64,
    /// Encryption operations
    pub encryption_ops: AtomicU64,
    /// Decryption operations
    pub decryption_ops: AtomicU64,
    /// Average operation latency
    pub avg_latency: RwLock<Duration>,
    /// Peak latency
    pub peak_latency: RwLock<Duration>,
    /// Key operation timings
    latency_samples: RwLock<Vec<Duration>>,
}

impl CryptoMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            key_operations: AtomicU64::new(0),
            key_cache_hits: AtomicU64::new(0),
            key_cache_misses: AtomicU64::new(0),
            encryption_ops: AtomicU64::new(0),
            decryption_ops: AtomicU64::new(0),
            avg_latency: RwLock::new(Duration::default()),
            peak_latency: RwLock::new(Duration::default()),
            latency_samples: RwLock::new(Vec::with_capacity(100)),
        }
    }
    
    /// Record key operation
    pub fn record_key_op(&self, latency: Duration) {
        self.key_operations.fetch_add(1, Ordering::Relaxed);
        self.record_latency(latency);
    }
    
    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.key_cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.key_cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record encryption operation
    pub fn record_encryption(&self, latency: Duration) {
        self.encryption_ops.fetch_add(1, Ordering::Relaxed);
        self.record_latency(latency);
    }
    
    /// Record decryption operation
    pub fn record_decryption(&self, latency: Duration) {
        self.decryption_ops.fetch_add(1, Ordering::Relaxed);
        self.record_latency(latency);
    }
    
    /// Record operation latency
    fn record_latency(&self, latency: Duration) {
        let mut avg = self.avg_latency.write();
        let mut peak = self.peak_latency.write();
        let mut samples = self.latency_samples.write();
        
        // Update average
        *avg = if samples.is_empty() {
            latency
        } else {
            Duration::from_nanos(
                ((avg.as_nanos() as f64 * samples.len() as f64) +
                 latency.as_nanos() as f64) as u64 / (samples.len() + 1) as f64 as u64
            )
        };
        
        // Update peak
        *peak = (*peak).max(latency);
        
        // Add to samples
        if samples.len() >= 100 {
            samples.remove(0);
        }
        samples.push(latency);
    }
    
    /// Get latency percentile
    pub fn get_latency_percentile(&self, percentile: f64) -> Duration {
        let samples = self.latency_samples.read();
        if samples.is_empty() {
            return Duration::default();
        }
        
        let mut sorted = samples.clone();
        sorted.sort();
        
        let index = ((sorted.len() as f64 * percentile / 100.0).round() as usize)
            .min(sorted.len() - 1);
            
        sorted[index]
    }
    
    /// Get metrics summary
    pub fn get_summary(&self) -> CryptoMetricsSummary {
        CryptoMetricsSummary {
            total_operations: self.key_operations.load(Ordering::Relaxed),
            cache_hit_ratio: self.key_cache_hits.load(Ordering::Relaxed) as f64 /
                (self.key_cache_hits.load(Ordering::Relaxed) + 
                 self.key_cache_misses.load(Ordering::Relaxed)) as f64,
            avg_latency_us: self.avg_latency.read().as_micros() as f64,
            peak_latency_us: self.peak_latency.read().as_micros() as f64,
            p99_latency_us: self.get_latency_percentile(99.0).as_micros() as f64,
        }
    }
}

/// Crypto metrics summary
#[derive(Debug, Clone)]
pub struct CryptoMetricsSummary {
    pub total_operations: u64,
    pub cache_hit_ratio: f64,
    pub avg_latency_us: f64,
    pub peak_latency_us: f64,
    pub p99_latency_us: f64,
}