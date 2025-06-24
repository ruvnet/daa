use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Network simulation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Message latency statistics
    pub latency: LatencyMetrics,
    /// Message throughput statistics
    pub throughput: ThroughputMetrics,
    /// Consensus metrics
    pub consensus: ConsensusMetrics,
}

/// Message latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Average message latency
    pub avg_latency: Duration,
    /// 95th percentile latency
    pub p95_latency: Duration,
    /// 99th percentile latency
    pub p99_latency: Duration,
    /// Maximum observed latency
    pub max_latency: Duration,
}

/// Message throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Messages per second
    pub msgs_per_sec: f64,
    /// Bytes per second
    pub bytes_per_sec: f64,
    /// Message drop rate
    pub drop_rate: f64,
}

/// Consensus metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    /// Average time to finality
    pub avg_finality_time: Duration,
    /// Number of finalized transactions
    pub finalized_tx_count: usize,
    /// Number of pending transactions
    pub pending_tx_count: usize,
}

impl NetworkMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            latency: LatencyMetrics {
                avg_latency: Duration::from_secs(0),
                p95_latency: Duration::from_secs(0),
                p99_latency: Duration::from_secs(0),
                max_latency: Duration::from_secs(0),
            },
            throughput: ThroughputMetrics {
                msgs_per_sec: 0.0,
                bytes_per_sec: 0.0,
                drop_rate: 0.0,
            },
            consensus: ConsensusMetrics {
                avg_finality_time: Duration::from_secs(0),
                finalized_tx_count: 0,
                pending_tx_count: 0,
            },
        }
    }
}
