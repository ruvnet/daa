#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Swarm coordination and multi-agent system for QuDAG.
//!
//! This module provides distributed agent coordination, task orchestration,
//! and hierarchical swarm management capabilities for the QuDAG protocol.
//!
//! ## Features
//!
//! - Hierarchical swarm coordination
//! - Async agent operations
//! - Task distribution and load balancing
//! - Work stealing and fault tolerance
//! - Real-time monitoring and statistics
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use qudag_swarm::{HierarchicalSwarm, SwarmConfig, Task, TaskPriority};
//! use std::sync::Arc;
//! use tokio::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create swarm with default configuration
//!     let config = SwarmConfig::default();
//!     let swarm = HierarchicalSwarm::new(config);
//!     
//!     // Submit a task
//!     let task = Task {
//!         id: "task_1".to_string(),
//!         payload: b"compute_hash".to_vec(),
//!         priority: TaskPriority::Normal,
//!         timeout: Duration::from_secs(30),
//!     };
//!     
//!     swarm.submit_task(task).await?;
//!     
//!     Ok(())
//! }
//! ```

/// Optimized implementations for high-performance coordination
pub mod optimized;

pub use optimized::async_coordination::{
    AgentError, AgentId, AgentMessage, AgentState, AgentStatus, AsyncAgent,
    DistributionStrategy, HierarchicalSwarm, SwarmConfig, SwarmStatistics,
    Task, TaskPriority, TaskResult,
};

// Re-export commonly used types at the module root
pub use optimized::async_coordination::HierarchicalSwarm as Swarm;