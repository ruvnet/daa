//! Protocol state synchronization implementation.

use crate::state::ProtocolState;
use thiserror::Error;

/// Errors that can occur during synchronization.
#[derive(Debug, Error)]
pub enum SyncError {
    /// Synchronization failed
    #[error("Synchronization failed")]
    SyncFailed,

    /// Invalid state received
    #[error("Invalid state received")]
    InvalidState,

    /// Version mismatch
    #[error("Version mismatch")]
    VersionMismatch,
}

/// Synchronization mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMode {
    /// Full state sync
    Full,

    /// Incremental state sync
    Incremental,

    /// Partial state sync
    Partial,
}

/// Synchronization configuration.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Sync mode
    pub mode: SyncMode,

    /// Sync interval in seconds
    pub interval: u64,

    /// Maximum sync attempts
    pub max_attempts: u32,
}

/// State synchronization trait defining the interface for sync operations.
pub trait StateSynchronization {
    /// Initialize synchronization with configuration.
    fn init(config: SyncConfig) -> Result<(), SyncError>;

    /// Start state synchronization.
    fn start_sync(&mut self) -> Result<(), SyncError>;

    /// Stop state synchronization.
    fn stop_sync(&mut self) -> Result<(), SyncError>;

    /// Request state from peers.
    fn request_state(&mut self) -> Result<ProtocolState, SyncError>;

    /// Send state to peers.
    fn send_state(&mut self, state: &ProtocolState) -> Result<(), SyncError>;

    /// Resolve state conflicts.
    fn resolve_conflicts(&mut self, states: Vec<ProtocolState>)
        -> Result<ProtocolState, SyncError>;
}
