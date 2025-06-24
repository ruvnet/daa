//! Real-time monitoring and observability for benchmarks
//!
//! This module provides tools for monitoring benchmark execution in real-time,
//! collecting metrics, and providing observability into system performance.

use crate::metrics::SystemMetrics;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Real-time monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Sampling interval for metrics collection
    pub sample_interval: Duration,
    /// Maximum number of samples to retain
    pub max_samples: usize,
    /// Enable detailed resource monitoring
    pub detailed_monitoring: bool,
}

/// Real-time benchmark monitor
pub struct BenchmarkMonitor {
    config: MonitoringConfig,
    samples: Arc<RwLock<Vec<TimestampedMetrics>>>,
    start_time: Instant,
}

/// Timestamped metrics sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedMetrics {
    /// Timestamp relative to benchmark start
    pub timestamp: Duration,
    /// System metrics at this timestamp
    pub metrics: SystemMetrics,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            sample_interval: Duration::from_millis(100),
            max_samples: 10000,
            detailed_monitoring: true,
        }
    }
}

impl BenchmarkMonitor {
    /// Create a new benchmark monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }

    /// Create a new benchmark monitor with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        Self {
            config,
            samples: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// Start monitoring (non-blocking)
    pub async fn start(&self) {
        // Placeholder for monitoring loop
        // In a real implementation, this would spawn a background task
        // that periodically collects metrics
    }

    /// Stop monitoring and return collected samples
    pub async fn stop(&self) -> Vec<TimestampedMetrics> {
        let samples = self.samples.read().await;
        samples.clone()
    }

    /// Record a metrics sample
    pub async fn record_sample(&self, metrics: SystemMetrics) {
        let timestamp = self.start_time.elapsed();
        let sample = TimestampedMetrics { timestamp, metrics };

        let mut samples = self.samples.write().await;
        samples.push(sample);

        // Keep only the most recent samples
        if samples.len() > self.config.max_samples {
            samples.remove(0);
        }
    }

    /// Get current sample count
    pub async fn sample_count(&self) -> usize {
        self.samples.read().await.len()
    }

    /// Get monitoring configuration
    pub fn config(&self) -> &MonitoringConfig {
        &self.config
    }
}

/// Monitoring utilities
pub mod utils {
    use super::*;

    /// Calculate metrics statistics from samples
    pub fn calculate_stats(samples: &[TimestampedMetrics]) -> MetricsStats {
        if samples.is_empty() {
            return MetricsStats::default();
        }

        let cpu_values: Vec<f64> = samples.iter().map(|s| s.metrics.node.cpu_usage).collect();
        let memory_values: Vec<f64> = samples
            .iter()
            .map(|s| s.metrics.node.memory_usage as f64)
            .collect();
        let throughput_values: Vec<f64> = samples
            .iter()
            .map(|s| s.metrics.network.messages_per_second)
            .collect();

        MetricsStats {
            cpu_avg: cpu_values.iter().sum::<f64>() / cpu_values.len() as f64,
            cpu_max: cpu_values.iter().cloned().fold(0.0, f64::max),
            memory_avg: memory_values.iter().sum::<f64>() / memory_values.len() as f64,
            memory_max: memory_values.iter().cloned().fold(0.0, f64::max),
            throughput_avg: throughput_values.iter().sum::<f64>() / throughput_values.len() as f64,
            throughput_max: throughput_values.iter().cloned().fold(0.0, f64::max),
            sample_count: samples.len(),
        }
    }

    /// Export samples to JSON
    pub fn export_to_json(samples: &[TimestampedMetrics]) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(samples)
    }
}

/// Statistical summary of metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStats {
    /// Average CPU usage
    pub cpu_avg: f64,
    /// Maximum CPU usage
    pub cpu_max: f64,
    /// Average memory usage
    pub memory_avg: f64,
    /// Maximum memory usage
    pub memory_max: f64,
    /// Average throughput
    pub throughput_avg: f64,
    /// Maximum throughput
    pub throughput_max: f64,
    /// Number of samples
    pub sample_count: usize,
}

impl Default for MetricsStats {
    fn default() -> Self {
        Self {
            cpu_avg: 0.0,
            cpu_max: 0.0,
            memory_avg: 0.0,
            memory_max: 0.0,
            throughput_avg: 0.0,
            throughput_max: 0.0,
            sample_count: 0,
        }
    }
}
