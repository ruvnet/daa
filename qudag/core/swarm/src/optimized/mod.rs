//! Optimized swarm implementations for high-performance coordination
//!
//! This module provides optimized implementations with focus on:
//! - Async coordination patterns
//! - Lock-free task distribution
//! - Hierarchical agent management
//! - Work stealing algorithms

pub mod async_coordination;

pub use async_coordination::{
    AgentError, AgentId, AgentMessage, AgentState, AgentStatus, AsyncAgent,
    DistributionStrategy, HierarchicalSwarm, SwarmConfig, SwarmStatistics,
    Task, TaskPriority, TaskResult,
};