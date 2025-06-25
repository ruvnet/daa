//! Stub modules for QuDAG protocol types

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Node {
    pub node_id: Vec<u8>,
}

impl Node {
    pub async fn new(_config: NodeConfig) -> Result<Self> {
        Ok(Self {
            node_id: vec![1, 2, 3, 4],
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        Ok(())
    }
    
    pub async fn handle_message(&mut self, _message: Message) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeConfig {
    pub port: u16,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self { port: 8080 }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub content: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Network error: {0}")]
    Network(String),
}

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("Invalid message: {0}")]
    Invalid(String),
}

pub mod qudag_protocol {
    pub use super::{Node, NodeConfig, Message, ProtocolError, MessageError};
}
