//! Stub modules for QuDAG protocol types

#[derive(Debug, Clone)]
pub struct Node;

#[derive(Debug, Clone)]
pub struct NodeConfig;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub content: String,
}

pub mod qudag_protocol {
    pub use super::{Node, NodeConfig, Message};
}
