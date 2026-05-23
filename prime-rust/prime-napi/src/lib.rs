//! # Prime ML NAPI - Node.js Bindings for Prime Distributed ML Framework
//!
//! This crate provides high-performance Node.js bindings for the Prime ML framework,
//! enabling federated learning with zero-copy tensor operations.
//!
//! ## Features
//!
//! - 🚀 Zero-copy tensor operations using `napi::Buffer`
//! - 🔄 Parallel gradient aggregation
//! - 🌐 Distributed training coordination
//! - 📊 Real-time training metrics
//! - 🔒 Type-safe Rust implementation
//! - ⚡ High-performance async operations
//!
//! ## Example
//!
//! ```javascript
//! const { TrainingNode, Coordinator } = require('@prime/ml-napi');
//!
//! // Create a training node
//! const node = new TrainingNode('node-1');
//! await node.initTraining({ batchSize: 32, learningRate: 0.001 });
//!
//! // Train for one epoch
//! const metrics = await node.trainEpoch();
//! console.log('Loss:', metrics.loss, 'Accuracy:', metrics.accuracy);
//! ```

#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod buffer;
mod coordinator;
mod trainer;
mod types;

use napi::Result;

/// Initialize the Prime ML NAPI module
#[napi]
pub fn init() -> Result<String> {
    Ok("Prime ML NAPI v0.2.1 initialized".to_string())
}

/// Get the version of the Prime ML NAPI bindings
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Re-export main types
pub use buffer::{create_tensor_buffer, tensor_from_buffer, TensorBuffer};
pub use coordinator::{Coordinator, CoordinatorConfig, CoordinatorStatusJs};
pub use trainer::{GradientUpdateJs, TrainingConfigJs, TrainingMetricsJs, TrainingNode};
pub use types::{AggregationStrategyJs, NodeInfoJs, OptimizerTypeJs};
