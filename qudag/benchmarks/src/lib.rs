#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Performance benchmarks for the QuDAG protocol.
//!
//! This crate provides comprehensive benchmarking tools and utilities for measuring
//! the performance characteristics of the QuDAG protocol, including throughput,
//! latency, scalability, and resource usage metrics.

pub mod baseline;
pub mod guards;
pub mod metrics;
pub mod monitoring;
pub mod regression;
pub mod scenarios;
pub mod system;
pub mod utils;

pub use baseline::*;
pub use guards::*;
pub use monitoring::*;
pub use regression::*;
pub use utils::{BenchmarkMetrics, ResourceMonitor};

/// Re-export commonly used types for benchmarking
pub use criterion;

/// Re-export the external metrics crate for convenience
pub use metrics as metrics_crate;
