//! Core types for the QuDAG MCP implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP request type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: serde_json::Value,
    /// Method name
    pub method: String,
    /// Parameters
    pub params: Option<serde_json::Value>,
}

/// MCP response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: serde_json::Value,
    /// Result (if successful)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<serde_json::Value>,
}

/// MCP resource type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// MIME type
    pub mime_type: Option<String>,
}

/// MCP tool type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    pub input_schema: serde_json::Value,
}

/// MCP event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEvent {
    /// Event type
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: u64,
}

/// Resource type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    /// DAG resource
    Dag,
    /// Vault resource
    Vault,
    /// Network resource
    Network,
    /// Crypto resource
    Crypto,
}

/// Tool type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolType {
    /// DAG tool
    Dag,
    /// Vault tool
    Vault,
    /// Network tool
    Network,
    /// Crypto tool
    Crypto,
}

/// Event type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// System event
    System,
    /// Security event
    Security,
    /// Network event
    Network,
    /// DAG event
    Dag,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

impl ServerInfo {
    /// Create new server info
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: "QuDAG MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Supported tools
    pub tools: Option<serde_json::Value>,
    /// Supported resources
    pub resources: Option<serde_json::Value>,
    /// Supported prompts
    pub prompts: Option<serde_json::Value>,
    /// Experimental features
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            tools: Some(serde_json::json!({
                "listChanged": true
            })),
            resources: Some(serde_json::json!({
                "subscribe": true,
                "listChanged": true
            })),
            prompts: Some(serde_json::json!({
                "listChanged": true
            })),
            experimental: Some(HashMap::new()),
        }
    }
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
}

impl ClientInfo {
    /// Create new client info
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Result content
    pub content: Vec<ToolResultContent>,
    /// Whether this is an error
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

/// Tool result content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolResultContent {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },
    /// Image content
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Resource content
    #[serde(rename = "resource")]
    Resource { resource: McpResource },
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// MIME type
    pub mime_type: Option<String>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// Resource URI
    pub uri: String,
    /// MIME type
    pub mime_type: Option<String>,
    /// Text content
    pub text: Option<String>,
    /// Binary data (base64 encoded)
    pub blob: Option<String>,
}

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Prompt name
    pub name: String,
    /// Prompt description
    pub description: Option<String>,
    /// Prompt arguments
    pub arguments: Option<Vec<PromptArgument>>,
}

impl Prompt {
    /// Create a new prompt
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: None,
        }
    }

    /// Set the prompt description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the prompt arguments
    pub fn with_arguments(mut self, arguments: Vec<PromptArgument>) -> Self {
        self.arguments = Some(arguments);
        self
    }
}

/// Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: Option<String>,
    /// Whether argument is required
    pub required: Option<bool>,
}

impl PromptArgument {
    /// Create a new prompt argument
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: None,
        }
    }

    /// Mark this argument as required
    pub fn required(mut self) -> Self {
        self.required = Some(true);
        self
    }

    /// Mark this argument as optional
    pub fn optional(mut self) -> Self {
        self.required = Some(false);
        self
    }

    /// Set the argument description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: MessageContent,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    /// User message
    #[serde(rename = "user")]
    User,
    /// Assistant message
    #[serde(rename = "assistant")]
    Assistant,
    /// System message
    #[serde(rename = "system")]
    System,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },
    /// Image content
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// Resource URI helper
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceURI(pub String);

impl ResourceURI {
    /// Create new resource URI
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    /// Create DAG resource URI
    pub fn dag(path: &str) -> Self {
        Self(format!("dag://{}", path))
    }

    /// Create vault resource URI
    pub fn vault(path: &str) -> Self {
        Self(format!("vault://{}", path))
    }

    /// Create network resource URI
    pub fn network(path: &str) -> Self {
        Self(format!("network://{}", path))
    }

    /// Create crypto resource URI
    pub fn crypto(path: &str) -> Self {
        Self(format!("crypto://{}", path))
    }

    /// Get the URI string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ResourceURI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ResourceURI {
    fn from(uri: String) -> Self {
        Self(uri)
    }
}

impl From<&str> for ResourceURI {
    fn from(uri: &str) -> Self {
        Self(uri.to_string())
    }
}

/// Tool name helper
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolName(pub String);

impl ToolName {
    /// Create new tool name
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the name string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ToolName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for ToolName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

/// Prompt name helper
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PromptName(pub String);

impl PromptName {
    /// Create new prompt name
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the name string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PromptName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PromptName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for PromptName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}
