//! # DAA AI Agent
//! 
//! This crate provides AI integration for DAA (Decentralized Autonomous Application)
//! using QuDAG MCP (Model Context Protocol) for communication with Claude and other AI systems.

pub mod agent;
pub mod mcp;
pub mod tools;
pub mod error;
pub mod streaming;

pub use agent::{AIAgent, AIAgentConfig, AIResponse};
pub use mcp::{McpAIClient, McpToolDefinition, McpMessage};
pub use tools::{DAAToolSet, ToolResult};
pub use error::{AIError, Result};
pub use streaming::{StreamingJsonParser, JsonMessage};

// Re-export commonly used types
pub use agent::{
    QueryRequest,
    QueryResponse,
    AgentCapabilities,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_agent_creation() {
        let config = AIAgentConfig::default();
        let agent = AIAgent::new(config).await;
        assert!(agent.is_ok());
    }
}