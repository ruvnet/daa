//! Stub modules for QuDAG MCP types

#[derive(Debug, Clone)]
pub struct MCPClient;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPMessage {
    pub content: String,
}

#[derive(Debug, thiserror::Error)]
#[error("MCP error")]
pub struct MCPError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tool {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub output: String,
}

pub mod qudag_mcp {
    pub use super::{MCPClient, MCPMessage, MCPError, Tool, ToolCall, ToolResult};
}
