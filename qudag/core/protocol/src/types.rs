use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Protocol errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Consensus error: {0}")]
    ConsensusError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

/// Protocol events
#[derive(Debug, Clone)]
pub enum ProtocolEvent {
    /// New message received
    MessageReceived {
        /// Message ID
        id: String,
        /// Message payload
        payload: Vec<u8>,
        /// Source peer
        source: Vec<u8>,
    },

    /// Message finalized by consensus
    MessageFinalized {
        /// Message ID
        id: String,
        /// Finalization time
        time: Duration,
    },

    /// Protocol state changed
    StateChanged {
        /// Previous state
        old_state: ProtocolState,
        /// New state
        new_state: ProtocolState,
    },

    /// Protocol error occurred
    Error {
        /// Error description
        error: String,
        /// Error context
        context: String,
    },
}

/// Protocol state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolState {
    /// Initial state
    Initial,
    /// Handshake in progress
    Handshaking,
    /// Protocol running
    Running,
    /// Protocol stopping
    Stopping,
    /// Protocol stopped
    Stopped,
    /// Error state
    Error,
}

/// Protocol metrics
#[derive(Debug, Clone, Default)]
pub struct ProtocolMetrics {
    /// Messages processed per second
    pub messages_per_second: f64,
    /// Average finalization time
    pub avg_finalization_time: Duration,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Active consensus rounds
    pub active_rounds: usize,
}
