//! Stub modules for QuDAG types until crates are published

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub height: u64,
    pub hash: Hash,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub id: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }
}

#[derive(Debug, Clone)]
pub struct Network;

#[derive(Debug, Clone)]
pub struct NetworkConfig;

#[derive(Debug, Clone)]
pub struct NetworkEvent;

#[derive(Debug, Clone)]
pub struct ProtocolMessage;

#[derive(Debug, thiserror::Error)]
#[error("Protocol error")]
pub struct ProtocolError;

#[derive(Debug)]
pub struct ConsensusEngine;

#[derive(Debug)]
pub struct ConsensusMessage;

#[derive(Debug)]
pub struct ConsensusState;

#[derive(Debug, Clone)]
pub struct PeerId(String);

#[derive(Debug)]
pub struct ProtocolHandler;

pub mod qudag_core {
    pub use super::{Block, Transaction, Hash};
}

pub mod qudag_network {
    pub use super::{Network, NetworkConfig, NetworkEvent, PeerId};
}

pub mod qudag_protocol {
    pub use super::{ProtocolMessage, ProtocolError, ProtocolHandler};
}

pub mod qudag_consensus {
    pub use super::{ConsensusEngine as ConsensusEngine, ConsensusMessage, ConsensusState};
}
