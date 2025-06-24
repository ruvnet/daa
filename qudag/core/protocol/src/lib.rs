#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Main protocol implementation and coordination for QuDAG.

pub mod allocator;
pub mod compatibility;
pub mod config;
pub mod coordinator;
pub mod handshake;
pub mod instrumentation;
pub mod message;
pub mod metrics;
pub mod node;
pub mod node_runner;
pub mod node_runner_adapter;
pub mod optimization_config;
pub mod persistence;
pub mod rpc_server;
pub mod state;
pub mod synchronization;
pub mod types;
pub mod validation;
pub mod versioning;

pub use allocator::{get_memory_usage, get_total_allocated, get_total_deallocated};
pub use compatibility::{CompatibilityAdapter, CompatibilityError, MessageTransformer};
pub use config::Config as ProtocolConfig;
pub use handshake::{
    HandshakeConfig, HandshakeCoordinator, HandshakeError, HandshakeKeys, HandshakeSession,
};
pub use instrumentation::{MemoryMetrics, MemoryTracker};
pub use message::{Message, MessageError, MessageFactory, MessageType, ProtocolVersion};
pub use node::{Node, NodeConfig, NodeStateProvider};
pub use node_runner_adapter::NodeRunnerAdapter;
pub use optimization_config::{
    AsyncCoordinationConfig, ConfigError as OptimizationConfigError, MessageChunkingConfig,
    OptimizationConfig, ValidationCacheConfig,
};
#[cfg(feature = "rocksdb")]
pub use persistence::RocksDbBackend;
pub use persistence::{
    FileStateStore, MemoryStateStore, PeerInfo as PersistencePeerInfo, PersistenceError,
    PersistentNodeRunner, Result as PersistenceResult, StartupState, StateStore, StorageStats,
};
pub use rpc_server::{
    NetworkStats, NodeRunnerTrait, PeerInfo as RpcPeerInfo, RpcCommand, RpcServer, RpcTransport,
};
pub use state::{ProtocolState, ProtocolStateMachine, StateError, StateMachineConfig};
pub use types::{ProtocolError, ProtocolEvent};
pub use versioning::{
    VersionError, VersionInfo, VersionManager, VersionPreferences, VersionRegistry,
};

// Re-export coordinator for test compatibility
pub use coordinator::Coordinator;

// Re-export node runner types
pub use node_runner::{NodeRunner, NodeRunnerConfig, NodeRunnerError};
