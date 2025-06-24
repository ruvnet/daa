//! # DAA MCP - Model Context Protocol Interface for DAA Management
//! 
//! This crate provides a comprehensive Model Context Protocol (MCP) interface
//! for managing Decentralized Autonomous Agents (DAAs). It enables external
//! systems to interact with DAA agents through standardized JSON-RPC 2.0
//! protocols, providing tools for agent lifecycle management, coordination,
//! and monitoring.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use tokio::sync::RwLock;

pub mod server;
pub mod client;
pub mod tools;
pub mod resources;
pub mod prompts;
pub mod transport;
pub mod discovery;
pub mod swarm;
pub mod integration;

// Re-export key types
pub use server::DaaMcpServer;
pub use client::DaaMcpClient;
pub use tools::*;
pub use resources::*;
pub use prompts::*;

/// DAA MCP Error types
#[derive(Error, Debug)]
pub enum DaaMcpError {
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Invalid tool: {0}")]
    InvalidTool(String),
    
    #[error("Resource not available: {0}")]
    ResourceNotAvailable(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("DAA orchestrator error: {0}")]
    Orchestrator(#[from] daa_orchestrator::Error),
    
    #[error("AI integration error: {0}")]
    AI(#[from] daa_ai::Error),
}

pub type Result<T> = std::result::Result<T, DaaMcpError>;

/// MCP Protocol version
pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

/// JSON-RPC 2.0 message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP Error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP Initialization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resource capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Prompt capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Server capabilities response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// DAA Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaaAgentInfo {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub endpoint: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub rules: HashMap<String, serde_json::Value>,
    pub economic_config: Option<EconomicConfig>,
    pub ai_config: Option<AiConfig>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Economic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicConfig {
    pub initial_balance: u64,
    pub token_symbol: String,
    pub max_daily_spend: Option<u64>,
    pub risk_threshold: Option<f64>,
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub system_prompt: Option<String>,
}

/// Task definition for agent coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaaTask {
    pub id: String,
    pub task_type: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: TaskPriority,
    pub timeout: Option<u64>, // seconds
    pub dependencies: Vec<String>,
    pub assigned_agents: Vec<String>,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: String,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metrics: HashMap<String, f64>,
}

/// Task execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Swarm coordination message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMessage {
    pub id: String,
    pub from_agent: String,
    pub to_agents: Vec<String>, // empty means broadcast
    pub message_type: SwarmMessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: Option<u64>, // time to live in seconds
}

/// Swarm message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmMessageType {
    TaskAssignment,
    TaskUpdate,
    StateSync,
    ResourceRequest,
    ResourceResponse,
    Heartbeat,
    Discovery,
    Coordination,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: Option<String>,
    pub annotations: Option<ResourceAnnotations>,
}

/// Resource annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnnotations {
    pub audience: Option<Vec<String>>,
    pub priority: Option<f64>,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Content>>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content structure for tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Configuration for the DAA MCP server
#[derive(Debug, Clone)]
pub struct DaaMcpConfig {
    pub server_name: String,
    pub server_version: String,
    pub bind_address: String,
    pub port: u16,
    pub enable_websocket: bool,
    pub enable_discovery: bool,
    pub max_agents: usize,
    pub heartbeat_interval: std::time::Duration,
    pub task_timeout: std::time::Duration,
}

impl Default for DaaMcpConfig {
    fn default() -> Self {
        Self {
            server_name: "daa-mcp-server".to_string(),
            server_version: "0.2.0".to_string(),
            bind_address: "127.0.0.1".to_string(),
            port: 3001,
            enable_websocket: true,
            enable_discovery: true,
            max_agents: 100,
            heartbeat_interval: std::time::Duration::from_secs(30),
            task_timeout: std::time::Duration::from_secs(300),
        }
    }
}

/// Shared state for the MCP server
#[derive(Debug)]
pub struct McpServerState {
    pub agents: Arc<RwLock<HashMap<String, DaaAgentInfo>>>,
    pub tasks: Arc<RwLock<HashMap<String, DaaTask>>>,
    pub task_results: Arc<RwLock<HashMap<String, TaskResult>>>,
    pub swarm_messages: Arc<RwLock<Vec<SwarmMessage>>>,
    pub config: DaaMcpConfig,
}

impl McpServerState {
    pub fn new(config: DaaMcpConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_results: Arc::new(RwLock::new(HashMap::new())),
            swarm_messages: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_message_serialization() {
        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(1.into())),
            method: Some("tools/list".to_string()),
            params: None,
            result: None,
            error: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: McpMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.jsonrpc, "2.0");
        assert_eq!(parsed.method, Some("tools/list".to_string()));
    }

    #[test]
    fn test_agent_info_creation() {
        let agent = DaaAgentInfo {
            id: "agent-001".to_string(),
            name: "Treasury Agent".to_string(),
            agent_type: "treasury".to_string(),
            status: AgentStatus::Running,
            capabilities: vec!["trading".to_string(), "risk_management".to_string()],
            endpoint: Some("http://localhost:3002".to_string()),
            created_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(agent.agent_type, "treasury");
        assert_eq!(agent.capabilities.len(), 2);
    }
}