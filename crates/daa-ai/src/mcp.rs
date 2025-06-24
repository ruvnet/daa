//! MCP (Model Context Protocol) client implementation for DAA AI

use crate::error::{AIError, Result};
use crate::streaming::{JsonMessage, JsonMessageStream, StreamingJsonParser};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// MCP client for communicating with AI services
#[derive(Debug, Clone)]
pub struct McpAIClient {
    inner: Arc<RwLock<McpClientInner>>,
}

#[derive(Debug)]
struct McpClientInner {
    config: McpClientConfig,
    connection: Option<McpConnection>,
    tool_definitions: HashMap<String, McpToolDefinition>,
    request_counter: u64,
}

/// Configuration for MCP client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    pub server_url: String,
    pub transport: McpTransport,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub enable_streaming: bool,
    pub tools: Vec<McpToolDefinition>,
}

/// MCP transport type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpTransport {
    Http,
    WebSocket,
    Stdio,
}

/// MCP connection state
#[derive(Debug)]
struct McpConnection {
    transport: McpTransport,
    http_client: Option<reqwest::Client>,
    websocket: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    stdio_tx: Option<mpsc::UnboundedSender<String>>,
    stdio_rx: Option<mpsc::UnboundedReceiver<String>>,
}

/// MCP message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

/// MCP error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub handler: String, // Handler identifier
}

/// MCP tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// MCP tool execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl McpAIClient {
    /// Create a new MCP client
    pub async fn new(config: McpClientConfig) -> Result<Self> {
        let tool_definitions: HashMap<String, McpToolDefinition> = config
            .tools
            .iter()
            .map(|tool| (tool.name.clone(), tool.clone()))
            .collect();
        
        let client = Self {
            inner: Arc::new(RwLock::new(McpClientInner {
                config,
                connection: None,
                tool_definitions,
                request_counter: 0,
            })),
        };
        
        Ok(client)
    }
    
    /// Connect to the MCP server
    pub async fn connect(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        let connection = match inner.config.transport {
            McpTransport::Http => {
                info!("Connecting to MCP server via HTTP: {}", inner.config.server_url);
                let http_client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(inner.config.timeout_seconds))
                    .build()?;
                
                McpConnection {
                    transport: McpTransport::Http,
                    http_client: Some(http_client),
                    websocket: None,
                    stdio_tx: None,
                    stdio_rx: None,
                }
            }
            McpTransport::WebSocket => {
                info!("Connecting to MCP server via WebSocket: {}", inner.config.server_url);
                
                let url = url::Url::parse(&inner.config.server_url)
                    .map_err(|e| AIError::InvalidConfiguration(format!("Invalid WebSocket URL: {}", e)))?;
                
                let (ws_stream, _) = tokio_tungstenite::connect_async(url).await
                    .map_err(|e| AIError::McpConnectionError(format!("WebSocket connection failed: {}", e)))?;
                
                McpConnection {
                    transport: McpTransport::WebSocket,
                    http_client: None,
                    websocket: Some(ws_stream),
                    stdio_tx: None,
                    stdio_rx: None,
                }
            }
            McpTransport::Stdio => {
                info!("Connecting to MCP server via stdio");
                
                let (tx, rx) = mpsc::unbounded_channel();
                
                McpConnection {
                    transport: McpTransport::Stdio,
                    http_client: None,
                    websocket: None,
                    stdio_tx: Some(tx),
                    stdio_rx: Some(rx),
                }
            }
        };
        
        inner.connection = Some(connection);
        
        // Initialize connection with server
        self.initialize_connection().await?;
        
        Ok(())
    }
    
    /// Initialize connection with server
    async fn initialize_connection(&self) -> Result<()> {
        let init_message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: Some("initialize".to_string()),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    },
                    "resources": {
                        "subscribe": true,
                        "listChanged": true
                    }
                },
                "clientInfo": {
                    "name": "daa-ai",
                    "version": "0.1.0"
                }
            })),
            result: None,
            error: None,
        };
        
        let response = self.send_request(init_message).await?;
        
        if response.error.is_some() {
            return Err(AIError::McpProtocolError(
                format!("Initialization failed: {:?}", response.error)
            ));
        }
        
        info!("MCP connection initialized successfully");
        Ok(())
    }
    
    /// Send a request to the MCP server
    pub async fn send_request(&self, message: McpMessage) -> Result<McpMessage> {
        let inner = self.inner.read().await;
        
        let connection = inner.connection.as_ref()
            .ok_or(AIError::McpConnectionError("Not connected".to_string()))?;
        
        match &connection.transport {
            McpTransport::Http => {
                let client = connection.http_client.as_ref()
                    .ok_or(AIError::McpConnectionError("HTTP client not initialized".to_string()))?;
                
                let response = client
                    .post(&inner.config.server_url)
                    .json(&message)
                    .send()
                    .await?;
                
                if !response.status().is_success() {
                    return Err(AIError::McpProtocolError(
                        format!("HTTP request failed: {}", response.status())
                    ));
                }
                
                let response_data: McpMessage = response.json().await?;
                Ok(response_data)
            }
            McpTransport::WebSocket => {
                // WebSocket implementation would go here
                // For now, return a mock response
                Err(AIError::McpProtocolError("WebSocket not fully implemented".to_string()))
            }
            McpTransport::Stdio => {
                // Stdio implementation would go here
                // For now, return a mock response
                Err(AIError::McpProtocolError("Stdio not fully implemented".to_string()))
            }
        }
    }
    
    /// Execute a tool via MCP
    pub async fn execute_tool(&self, request: McpToolRequest) -> Result<McpToolResponse> {
        let mut inner = self.inner.write().await;
        
        // Check if tool is defined
        if !inner.tool_definitions.contains_key(&request.name) {
            return Err(AIError::ToolNotFound(request.name));
        }
        
        inner.request_counter += 1;
        let request_id = inner.request_counter;
        
        drop(inner); // Release lock for async operation
        
        let mcp_message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(request_id)),
            method: Some("tools/call".to_string()),
            params: Some(serde_json::json!({
                "name": request.name,
                "arguments": request.arguments
            })),
            result: None,
            error: None,
        };
        
        let response = self.send_request(mcp_message).await?;
        
        if let Some(error) = response.error {
            return Ok(McpToolResponse {
                success: false,
                result: None,
                error: Some(error.message),
                metadata: HashMap::new(),
            });
        }
        
        Ok(McpToolResponse {
            success: true,
            result: response.result,
            error: None,
            metadata: HashMap::new(),
        })
    }
    
    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<McpToolDefinition>> {
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: Some("tools/list".to_string()),
            params: None,
            result: None,
            error: None,
        };
        
        let response = self.send_request(message).await?;
        
        if let Some(error) = response.error {
            return Err(AIError::McpProtocolError(error.message));
        }
        
        // Parse tools from response
        let tools = response.result
            .and_then(|r| r.get("tools"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_else(Vec::new);
        
        Ok(tools)
    }
    
    /// Register a new tool
    pub async fn register_tool(&self, tool: McpToolDefinition) -> Result<()> {
        let mut inner = self.inner.write().await;
        inner.tool_definitions.insert(tool.name.clone(), tool.clone());
        
        info!("Registered tool: {}", tool.name);
        Ok(())
    }
    
    /// Get streaming response from MCP server
    pub async fn get_streaming_response(&self, message: McpMessage) -> Result<impl futures::Stream<Item = Result<JsonMessage>>> {
        let inner = self.inner.read().await;
        
        let connection = inner.connection.as_ref()
            .ok_or(AIError::McpConnectionError("Not connected".to_string()))?;
        
        match &connection.transport {
            McpTransport::Http => {
                let client = connection.http_client.as_ref()
                    .ok_or(AIError::McpConnectionError("HTTP client not initialized".to_string()))?;
                
                let response = client
                    .post(&inner.config.server_url)
                    .json(&message)
                    .send()
                    .await?;
                
                if !response.status().is_success() {
                    return Err(AIError::McpProtocolError(
                        format!("HTTP request failed: {}", response.status())
                    ));
                }
                
                let byte_stream = response.bytes_stream().map(|result| {
                    result.map_err(|e| AIError::NetworkError(e))
                });
                
                Ok(JsonMessageStream::new(byte_stream))
            }
            _ => {
                Err(AIError::McpProtocolError("Streaming not supported for this transport".to_string()))
            }
        }
    }
    
    /// Disconnect from MCP server
    pub async fn disconnect(&self) -> Result<()> {
        let mut inner = self.inner.write().await;
        
        if inner.connection.is_some() {
            info!("Disconnecting from MCP server");
            inner.connection = None;
        }
        
        Ok(())
    }
    
    /// Check if client is connected
    pub async fn is_connected(&self) -> bool {
        let inner = self.inner.read().await;
        inner.connection.is_some()
    }
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000/mcp".to_string(),
            transport: McpTransport::Http,
            timeout_seconds: 30,
            max_retries: 3,
            enable_streaming: true,
            tools: Vec::new(),
        }
    }
}

impl McpMessage {
    /// Create a new request message
    pub fn request(id: serde_json::Value, method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: Some(method),
            params,
            result: None,
            error: None,
        }
    }
    
    /// Create a new response message
    pub fn response(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }
    
    /// Create a new error response
    pub fn error_response(id: serde_json::Value, error: McpError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }
}

/// Trait for handling MCP tool execution
#[async_trait]
pub trait McpToolHandler: Send + Sync {
    /// Execute the tool with given arguments
    async fn execute(&self, arguments: serde_json::Value) -> Result<serde_json::Value>;
    
    /// Get tool definition
    fn definition(&self) -> McpToolDefinition;
}