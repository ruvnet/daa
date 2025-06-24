use serde::{Deserialize, Serialize};
use std::time::Duration;

/// System-wide performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Node metrics
    pub node: NodeMetrics,
    /// Network metrics
    pub network: NetworkMetrics,
    /// DAG metrics
    pub dag: DagMetrics,
}

/// Node performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Open file descriptors
    pub file_descriptors: u64,
    /// Thread count
    pub thread_count: u64,
}

/// Network performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Messages per second
    pub messages_per_second: f64,
    /// Bytes per second
    pub bytes_per_second: f64,
    /// Average message latency
    pub avg_latency: Duration,
    /// Connection count
    pub connections: u64,
}

/// DAG performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagMetrics {
    /// Vertices per second
    pub vertices_per_second: f64,
    /// Average finalization time
    pub avg_finalization_time: Duration,
    /// Finalized vertices
    pub finalized_vertices: u64,
    /// Pending vertices
    pub pending_vertices: u64,
}

impl SystemMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            node: NodeMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                file_descriptors: 0,
                thread_count: 0,
            },
            network: NetworkMetrics {
                messages_per_second: 0.0,
                bytes_per_second: 0.0,
                avg_latency: Duration::from_secs(0),
                connections: 0,
            },
            dag: DagMetrics {
                vertices_per_second: 0.0,
                avg_finalization_time: Duration::from_secs(0),
                finalized_vertices: 0,
                pending_vertices: 0,
            },
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
