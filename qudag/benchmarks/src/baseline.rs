//! Performance baseline tracking and management.
//!
//! This module provides utilities for capturing, storing, and comparing performance baselines
//! to detect regressions and improvements in the QuDAG protocol implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Performance baseline data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Benchmark name
    pub name: String,
    /// Target metric (e.g., "latency", "throughput", "memory")
    pub metric: String,
    /// Baseline value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Standard deviation from multiple runs
    pub std_dev: f64,
    /// Timestamp when baseline was captured
    pub timestamp: SystemTime,
    /// Git commit hash if available
    pub commit_hash: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Collection of performance baselines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineCollection {
    /// All baselines organized by benchmark name
    pub baselines: HashMap<String, PerformanceBaseline>,
    /// Last updated timestamp
    pub last_updated: SystemTime,
    /// Version of the baseline format
    pub version: String,
}

impl BaselineCollection {
    /// Create a new empty baseline collection
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            last_updated: SystemTime::now(),
            version: "1.0".to_string(),
        }
    }

    /// Add or update a baseline
    pub fn set_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baselines.insert(baseline.name.clone(), baseline);
        self.last_updated = SystemTime::now();
    }

    /// Get a baseline by name
    pub fn get_baseline(&self, name: &str) -> Option<&PerformanceBaseline> {
        self.baselines.get(name)
    }

    /// Load baselines from file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let collection: BaselineCollection = serde_json::from_str(&content)?;
        Ok(collection)
    }

    /// Save baselines to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get all baseline names
    pub fn get_names(&self) -> Vec<String> {
        self.baselines.keys().cloned().collect()
    }
}

/// Builder for creating performance baselines
pub struct BaselineBuilder {
    name: String,
    metric: String,
    values: Vec<f64>,
    unit: String,
    metadata: HashMap<String, String>,
}

impl BaselineBuilder {
    /// Create a new baseline builder
    pub fn new(name: &str, metric: &str) -> Self {
        Self {
            name: name.to_string(),
            metric: metric.to_string(),
            values: Vec::new(),
            unit: "units".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add a measurement value
    pub fn add_value(mut self, value: f64) -> Self {
        self.values.push(value);
        self
    }

    /// Add multiple measurement values
    pub fn add_values(mut self, values: Vec<f64>) -> Self {
        self.values.extend(values);
        self
    }

    /// Set the unit of measurement
    pub fn unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the baseline
    pub fn build(self) -> Result<PerformanceBaseline, &'static str> {
        if self.values.is_empty() {
            return Err("No measurement values provided");
        }

        let mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
        let variance =
            self.values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / self.values.len() as f64;
        let std_dev = variance.sqrt();

        Ok(PerformanceBaseline {
            name: self.name,
            metric: self.metric,
            value: mean,
            unit: self.unit,
            std_dev,
            timestamp: SystemTime::now(),
            commit_hash: get_git_commit_hash(),
            metadata: self.metadata,
        })
    }
}

/// Baseline capture utilities
pub struct BaselineCapture;

impl BaselineCapture {
    /// Capture baseline from duration measurements
    pub fn from_durations(name: &str, durations: Vec<Duration>) -> PerformanceBaseline {
        let values: Vec<f64> = durations.iter().map(|d| d.as_nanos() as f64).collect();

        BaselineBuilder::new(name, "latency")
            .add_values(values)
            .unit("nanoseconds")
            .build()
            .expect("Failed to build baseline from durations")
    }

    /// Capture baseline from throughput measurements (operations per second)
    pub fn from_throughput(name: &str, ops_per_sec: Vec<f64>) -> PerformanceBaseline {
        BaselineBuilder::new(name, "throughput")
            .add_values(ops_per_sec)
            .unit("ops/sec")
            .build()
            .expect("Failed to build baseline from throughput")
    }

    /// Capture baseline from memory usage measurements
    pub fn from_memory_usage(name: &str, memory_bytes: Vec<u64>) -> PerformanceBaseline {
        let values: Vec<f64> = memory_bytes.iter().map(|&m| m as f64).collect();

        BaselineBuilder::new(name, "memory")
            .add_values(values)
            .unit("bytes")
            .build()
            .expect("Failed to build baseline from memory usage")
    }
}

/// Default baseline targets for QuDAG protocol
pub struct DefaultTargets;

impl DefaultTargets {
    /// Sub-second consensus finality (99th percentile)
    pub const CONSENSUS_FINALITY_NS: f64 = 1_000_000_000.0; // 1 second in nanoseconds

    /// 10,000+ messages/second throughput per node
    pub const THROUGHPUT_TARGET: f64 = 10_000.0;

    /// <100MB memory usage for base node
    pub const MEMORY_TARGET_BYTES: f64 = 100.0 * 1024.0 * 1024.0; // 100MB

    /// Cryptographic operation latency targets
    pub const ML_KEM_KEYGEN_NS: f64 = 1_000_000.0; // 1ms
    pub const ML_KEM_ENCAPS_NS: f64 = 500_000.0; // 0.5ms
    pub const ML_KEM_DECAPS_NS: f64 = 500_000.0; // 0.5ms

    /// Create default baseline collection with targets
    pub fn create_default_baselines() -> BaselineCollection {
        let mut collection = BaselineCollection::new();

        // Add consensus targets
        collection.set_baseline(PerformanceBaseline {
            name: "consensus_finality".to_string(),
            metric: "latency".to_string(),
            value: Self::CONSENSUS_FINALITY_NS,
            unit: "nanoseconds".to_string(),
            std_dev: Self::CONSENSUS_FINALITY_NS * 0.1, // 10% tolerance
            timestamp: SystemTime::now(),
            commit_hash: get_git_commit_hash(),
            metadata: [("type".to_string(), "target".to_string())].into(),
        });

        // Add throughput targets
        collection.set_baseline(PerformanceBaseline {
            name: "node_throughput".to_string(),
            metric: "throughput".to_string(),
            value: Self::THROUGHPUT_TARGET,
            unit: "ops/sec".to_string(),
            std_dev: Self::THROUGHPUT_TARGET * 0.1, // 10% tolerance
            timestamp: SystemTime::now(),
            commit_hash: get_git_commit_hash(),
            metadata: [("type".to_string(), "target".to_string())].into(),
        });

        // Add memory targets
        collection.set_baseline(PerformanceBaseline {
            name: "node_memory".to_string(),
            metric: "memory".to_string(),
            value: Self::MEMORY_TARGET_BYTES,
            unit: "bytes".to_string(),
            std_dev: Self::MEMORY_TARGET_BYTES * 0.1, // 10% tolerance
            timestamp: SystemTime::now(),
            commit_hash: get_git_commit_hash(),
            metadata: [("type".to_string(), "target".to_string())].into(),
        });

        collection
    }
}

/// Utility function to get git commit hash
fn get_git_commit_hash() -> Option<String> {
    use std::process::Command;

    Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_builder() {
        let baseline = BaselineBuilder::new("test_benchmark", "latency")
            .add_value(100.0)
            .add_value(110.0)
            .add_value(90.0)
            .unit("ms")
            .metadata("version", "1.0")
            .build()
            .unwrap();

        assert_eq!(baseline.name, "test_benchmark");
        assert_eq!(baseline.metric, "latency");
        assert_eq!(baseline.unit, "ms");
        assert_eq!(baseline.value, 100.0); // mean
        assert!(baseline.std_dev > 0.0);
        assert_eq!(baseline.metadata.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_baseline_collection() {
        let mut collection = BaselineCollection::new();
        let baseline = BaselineBuilder::new("test", "latency")
            .add_value(100.0)
            .build()
            .unwrap();

        collection.set_baseline(baseline);
        assert!(collection.get_baseline("test").is_some());
        assert!(collection.get_baseline("nonexistent").is_none());
    }

    #[test]
    fn test_baseline_capture() {
        let durations = vec![
            Duration::from_millis(100),
            Duration::from_millis(110),
            Duration::from_millis(90),
        ];

        let baseline = BaselineCapture::from_durations("test_latency", durations);
        assert_eq!(baseline.name, "test_latency");
        assert_eq!(baseline.metric, "latency");
        assert_eq!(baseline.unit, "nanoseconds");
        assert!(baseline.value > 0.0);
    }
}
