#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Command-line interface for the QuDAG protocol with performance optimizations.
//!
//! This module provides a comprehensive CLI for managing QuDAG nodes,
//! including node operations, peer management, network diagnostics,
//! and DAG visualization capabilities. Features include:
//!
//! - Fast startup with lazy initialization
//! - Async operation optimization with timeouts and retries
//! - Resource management and memory tracking
//! - Performance monitoring and reporting
//! - Error propagation optimization

pub mod async_optimizations;
pub mod commands;
pub mod config;
pub mod mcp;
pub mod node_manager;
pub mod output;
pub mod peer_manager;
pub mod performance;
pub mod rpc;
pub mod startup;

#[cfg(test)]
pub mod mocks;

pub use commands::{
    check_node_connectivity, execute_status_command, show_status, start_node, stop_node,
    CommandRouter, DagStatistics, MemoryUsage, NetworkStatistics, NodeState, NodeStatusResponse,
    OutputFormat, PeerStatusInfo, StatusArgs,
};

pub use config::{NodeConfig, NodeConfigManager};

/// CLI-specific error types
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Node error: {0}")]
    Node(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Visualization error: {0}")]
    Visualization(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Command error: {0}")]
    Command(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Status error: {0}")]
    Status(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Server error: {0}")]
    Server(String),
}
