//! Stub modules for QuDAG MCP types

#[derive(Debug, Clone)]
pub struct MCPClient {
    server_url: String,
}

impl MCPClient {
    pub async fn new(server_url: &str) -> Result<Self, MCPError> {
        Ok(Self {
            server_url: server_url.to_string(),
        })
    }
    
    pub async fn connect(&self) -> Result<(), MCPError> {
        // Stub implementation
        Ok(())
    }
    
    pub async fn call_tool(&self, tool_call: ToolCall) -> Result<ToolResult, MCPError> {
        // Stub implementation
        Ok(ToolResult {
            output: format!("Called tool {} with id {}", tool_call.name, tool_call.id),
        })
    }
}

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
    pub id: String,
    pub name: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub output: String,
}

pub mod qudag_mcp {
    pub use super::{MCPClient, MCPMessage, MCPError, Tool, ToolCall, ToolResult};
}
