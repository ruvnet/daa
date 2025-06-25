//! Stub modules for QuDAG types until crates are published

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub height: u64,
    pub hash: Hash,
    pub header: crate::block::BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(hash: Hash, header: crate::block::BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            height: 0,
            hash,
            header,
            transactions,
        }
    }
    
    pub fn new_empty() -> Self {
        Self {
            height: 0,
            hash: Hash::default(),
            header: crate::block::BlockHeader {
                parent_hash: Hash::default(),
                merkle_root: Hash::default(),
                timestamp: 0,
                transaction_count: 0,
                extra_data: Vec::new(),
            },
            transactions: Vec::new(),
        }
    }
    
    pub fn hash(&self) -> Hash {
        self.hash
    }
    
    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }
    
    pub fn header(&self) -> &crate::block::BlockHeader {
        &self.header
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub id: String,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            data: Vec::new(),
            signature: Vec::new(),
        }
    }
    
    pub fn new_with_data(hash: Hash, data: Vec<u8>, signature: Vec<u8>) -> Self {
        Self {
            id: hash.to_string(),
            data,
            signature,
        }
    }
    
    pub fn hash(&self) -> Hash {
        // Simple hash implementation for stub
        Hash::default()
    }
    
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
    
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut data = [0u8; 32];
        data.copy_from_slice(&bytes[..32]);
        Self(data)
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self([0; 32])
    }
}

#[derive(Debug, Clone)]
pub struct Network;

impl Network {
    pub async fn new(_config: NetworkConfig) -> Result<Self, String> {
        Ok(Self)
    }
    
    pub async fn start(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    pub async fn next_event(&mut self) -> Result<NetworkEvent, String> {
        // Stub implementation
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
    
    pub async fn broadcast_transaction(&mut self, _hash: Hash) -> Result<(), String> {
        Ok(())
    }
    
    pub async fn broadcast_block(&mut self, _block: Block) -> Result<(), String> {
        Ok(())
    }
    
    pub async fn broadcast(&mut self, _data: Vec<u8>) -> Result<(), String> {
        Ok(())
    }
    
    pub async fn send_to_peer(&mut self, _peer: PeerId, _data: Vec<u8>) -> Result<(), String> {
        Ok(())
    }
    
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<NetworkEvent> {
        let (_sender, receiver) = tokio::sync::broadcast::channel(1);
        receiver
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfig;

impl Default for NetworkConfig {
    fn default() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    TransactionReceived(Transaction),
    BlockReceived(Block),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    MessageReceived { peer_id: PeerId, data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct ProtocolMessage {
    data: Vec<u8>,
}

impl ProtocolMessage {
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Protocol error")]
pub struct ProtocolError;

#[derive(Debug)]
pub struct ConsensusEngine;

impl ConsensusEngine {
    pub async fn new() -> Result<Self, String> {
        Ok(Self)
    }
    
    pub async fn start(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConsensusMessage;

#[derive(Debug)]
pub struct ConsensusState;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerId(String);

impl PeerId {
    pub fn random() -> Self {
        Self(format!("peer_{}", uuid::Uuid::new_v4()))
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

// Remove ProtocolHandler struct as it conflicts with the trait in network.rs

pub mod qudag_core {
    pub use super::{Block, Transaction, Hash};
}

pub mod qudag_network {
    pub use super::{Network, NetworkConfig, NetworkEvent, PeerId};
}

pub mod qudag_protocol {
    pub use super::{ProtocolMessage, ProtocolError};
}

pub mod qudag_consensus {
    pub use super::ConsensusEngine;
}
