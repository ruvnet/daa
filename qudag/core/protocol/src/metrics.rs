use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics for the QuDAG protocol
pub struct ProtocolMetrics {
    // Cryptographic metrics
    pub crypto_operations: AtomicU64,
    pub key_cache_hits: AtomicU64,
    pub key_cache_misses: AtomicU64,

    // Network metrics
    pub messages_processed: AtomicU64,
    pub active_connections: AtomicU64,
    pub connection_errors: AtomicU64,
    pub route_cache_hits: AtomicU64,

    // Consensus metrics
    pub consensus_rounds: AtomicU64,
    pub dag_updates: AtomicU64,
    pub node_count: AtomicU64,

    // Resource metrics
    pub memory_usage: AtomicU64,
    pub thread_count: AtomicU64,
    pub queue_depth: AtomicU64,

    // Last update timestamp
    last_update: Arc<RwLock<Instant>>,
    update_interval: Duration,
}

impl Default for ProtocolMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            // Crypto metrics
            crypto_operations: AtomicU64::new(0),
            key_cache_hits: AtomicU64::new(0),
            key_cache_misses: AtomicU64::new(0),

            // Network metrics
            messages_processed: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            connection_errors: AtomicU64::new(0),
            route_cache_hits: AtomicU64::new(0),

            // Consensus metrics
            consensus_rounds: AtomicU64::new(0),
            dag_updates: AtomicU64::new(0),
            node_count: AtomicU64::new(0),

            // Resource metrics
            memory_usage: AtomicU64::new(0),
            thread_count: AtomicU64::new(0),
            queue_depth: AtomicU64::new(0),

            // Update tracking
            last_update: Arc::new(RwLock::new(Instant::now())),
            update_interval: Duration::from_secs(1),
        }
    }

    /// Record cryptographic operation
    pub fn record_crypto_op(&self, _latency: Duration) {
        self.crypto_operations.fetch_add(1, Ordering::Relaxed);
        self.maybe_flush_metrics();
    }

    /// Record message processing
    pub fn record_message(&self, _latency: Duration) {
        self.messages_processed.fetch_add(1, Ordering::Relaxed);
        self.maybe_flush_metrics();
    }

    /// Record consensus round
    pub fn record_consensus(&self, _latency: Duration) {
        self.consensus_rounds.fetch_add(1, Ordering::Relaxed);
        self.maybe_flush_metrics();
    }

    /// Update resource metrics
    pub fn update_resources(&self, memory: u64, threads: u64, queue: u64) {
        self.memory_usage.store(memory, Ordering::Relaxed);
        self.thread_count.store(threads, Ordering::Relaxed);
        self.queue_depth.store(queue, Ordering::Relaxed);
        self.maybe_flush_metrics();
    }

    /// Get performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            messages_per_second: self.messages_processed.load(Ordering::Relaxed) as f64
                / self.last_update.read().elapsed().as_secs_f64(),
            avg_message_latency: 0.0, // TODO: Implement proper latency tracking
            avg_consensus_latency: 0.0, // TODO: Implement proper latency tracking
            active_connections: self.active_connections.load(Ordering::Relaxed),
            memory_usage: self.memory_usage.load(Ordering::Relaxed),
        }
    }

    // Flush metrics if update interval elapsed
    fn maybe_flush_metrics(&self) {
        let mut last_update = self.last_update.write();
        if last_update.elapsed() >= self.update_interval {
            *last_update = Instant::now();
        }
    }
}

/// Performance summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub messages_per_second: f64,
    pub avg_message_latency: f64,
    pub avg_consensus_latency: f64,
    pub active_connections: u64,
    pub memory_usage: u64,
}
