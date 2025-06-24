//! MCP Server implementation for DAA management
//! 
//! This module provides the core MCP server that exposes DAA management
//! capabilities through standardized JSON-RPC 2.0 protocols.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    DaaMcpConfig, DaaMcpError, InitializeRequest, InitializeResponse, McpMessage, McpServerState,
    Result, ServerCapabilities, ServerInfo, ToolCapabilities, ResourceCapabilities,
    PromptCapabilities, MCP_PROTOCOL_VERSION,
};

/// Main DAA MCP Server
pub struct DaaMcpServer {
    state: Arc<McpServerState>,
    app: Router,
}

impl DaaMcpServer {
    /// Create a new DAA MCP server
    pub fn new(config: DaaMcpConfig) -> Self {
        let state = Arc::new(McpServerState::new(config));
        let app = create_app(state.clone());

        Self { state, app }
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.state.config.bind_address, self.state.config.port);
        
        info!("Starting DAA MCP Server on {}", bind_addr);
        
        // Start background tasks
        self.start_background_tasks().await;
        
        // Start HTTP server
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        axum::serve(listener, self.app.clone()).await?;
        
        Ok(())
    }

    /// Start background maintenance tasks
    async fn start_background_tasks(&self) {
        let state = self.state.clone();
        
        // Heartbeat monitor
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(state.config.heartbeat_interval);
            loop {
                interval.tick().await;
                if let Err(e) = heartbeat_monitor(state.clone()).await {
                    error!("Heartbeat monitor error: {}", e);
                }
            }
        });

        // Task timeout monitor
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = task_timeout_monitor(state.clone()).await {
                    error!("Task timeout monitor error: {}", e);
                }
            }
        });

        // Swarm message cleanup
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = cleanup_expired_messages(state.clone()).await {
                    error!("Message cleanup error: {}", e);
                }
            }
        });
    }

    /// Get server statistics
    pub async fn get_stats(&self) -> HashMap<String, Value> {
        let mut stats = HashMap::new();
        
        let agents = self.state.agents.read().await;
        let tasks = self.state.tasks.read().await;
        let results = self.state.task_results.read().await;
        let messages = self.state.swarm_messages.read().await;

        stats.insert("agent_count".to_string(), Value::Number(agents.len().into()));
        stats.insert("task_count".to_string(), Value::Number(tasks.len().into()));
        stats.insert("completed_tasks".to_string(), Value::Number(results.len().into()));
        stats.insert("message_count".to_string(), Value::Number(messages.len().into()));
        stats.insert("uptime".to_string(), Value::String("running".to_string()));

        stats
    }
}

/// Create the Axum application router
fn create_app(state: Arc<McpServerState>) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/mcp/ws", get(handle_websocket_upgrade))
        .route("/health", get(health_check))
        .route("/stats", get(get_server_stats))
        .with_state(state)
}

/// Handle HTTP MCP requests
async fn handle_mcp_request(
    State(state): State<Arc<McpServerState>>,
    Json(message): Json<McpMessage>,
) -> Response {
    debug!("Received MCP request: {:?}", message);

    match process_mcp_message(state, message).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            error!("MCP request error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(create_error_response(e))).into_response()
        }
    }
}

/// Handle WebSocket upgrade for real-time MCP communication
async fn handle_websocket_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<McpServerState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connections
async fn handle_websocket(
    socket: axum::extract::ws::WebSocket,
    state: Arc<McpServerState>,
) {
    use axum::extract::ws::Message;
    use futures::{sink::SinkExt, stream::StreamExt};

    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket connection established");

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("WebSocket received: {}", text);
                
                match serde_json::from_str::<McpMessage>(&text) {
                    Ok(mcp_msg) => {
                        match process_mcp_message(state.clone(), mcp_msg).await {
                            Ok(response) => {
                                let response_text = serde_json::to_string(&response).unwrap();
                                if sender.send(Message::Text(response_text)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let error_response = create_error_response(e);
                                let error_text = serde_json::to_string(&error_response).unwrap();
                                if sender.send(Message::Text(error_text)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse WebSocket message: {}", e);
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Process incoming MCP messages
async fn process_mcp_message(
    state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let method = message.method.as_deref().unwrap_or("");
    
    match method {
        "initialize" => handle_initialize(state, message).await,
        "tools/list" => handle_tools_list(state, message).await,
        "tools/call" => handle_tools_call(state, message).await,
        "resources/list" => handle_resources_list(state, message).await,
        "resources/read" => handle_resources_read(state, message).await,
        "prompts/list" => handle_prompts_list(state, message).await,
        "prompts/get" => handle_prompts_get(state, message).await,
        _ => {
            warn!("Unknown method: {}", method);
            Err(DaaMcpError::Protocol(format!("Unknown method: {}", method)))
        }
    }
}

/// Handle initialization request
async fn handle_initialize(
    _state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let _init_request: InitializeRequest = serde_json::from_value(
        message.params.unwrap_or_default()
    )?;

    let response = InitializeResponse {
        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolCapabilities {
                list_changed: Some(true),
            }),
            resources: Some(ResourceCapabilities {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            prompts: Some(PromptCapabilities {
                list_changed: Some(true),
            }),
        },
        server_info: ServerInfo {
            name: "daa-mcp-server".to_string(),
            version: "0.2.0".to_string(),
        },
    };

    Ok(McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(serde_json::to_value(response)?),
        error: None,
    })
}

/// Handle tools list request
async fn handle_tools_list(
    _state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let tools = crate::tools::get_available_tools();
    
    Ok(McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(serde_json::json!({
            "tools": tools
        })),
        error: None,
    })
}

/// Handle tool call request
async fn handle_tools_call(
    state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let params = message.params.as_ref().ok_or_else(|| {
        DaaMcpError::Protocol("Missing parameters for tool call".to_string())
    })?;

    let tool_name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing tool name".to_string()))?;

    let arguments = params.get("arguments")
        .cloned()
        .unwrap_or_default();

    let result = crate::tools::execute_tool(state, tool_name, arguments).await?;
    
    Ok(McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(serde_json::to_value(result)?),
        error: None,
    })
}

/// Handle resources list request
async fn handle_resources_list(
    _state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let resources = crate::resources::get_available_resources().await;
    
    Ok(McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(serde_json::json!({
            "resources": resources
        })),
        error: None,
    })
}

/// Handle resource read request
async fn handle_resources_read(
    state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let params = message.params.as_ref().ok_or_else(|| {
        DaaMcpError::Protocol("Missing parameters for resource read".to_string())
    })?;

    let uri = params.get("uri")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing resource URI".to_string()))?;

    let content = crate::resources::read_resource(state, uri).await?;
    
    Ok(McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(serde_json::json!({
            "contents": content
        })),
        error: None,
    })
}

/// Handle prompts list request
async fn handle_prompts_list(
    _state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let prompts = crate::prompts::get_available_prompts();
    
    Ok(McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(serde_json::json!({
            "prompts": prompts
        })),
        error: None,
    })
}

/// Handle prompt get request
async fn handle_prompts_get(
    state: Arc<McpServerState>,
    message: McpMessage,
) -> Result<McpMessage> {
    let params = message.params.as_ref().ok_or_else(|| {
        DaaMcpError::Protocol("Missing parameters for prompt get".to_string())
    })?;

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing prompt name".to_string()))?;

    let arguments = params.get("arguments")
        .cloned()
        .unwrap_or_default();

    let prompt = crate::prompts::get_prompt(state, name, arguments).await?;
    
    Ok(McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(serde_json::to_value(prompt)?),
        error: None,
    })
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "daa-mcp-server"
    }))
}

/// Get server statistics
async fn get_server_stats(State(state): State<Arc<McpServerState>>) -> Json<Value> {
    let mut stats = HashMap::new();
    
    let agents = state.agents.read().await;
    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;
    let messages = state.swarm_messages.read().await;

    stats.insert("agents", agents.len());
    stats.insert("tasks", tasks.len());
    stats.insert("completed_tasks", results.len());
    stats.insert("messages", messages.len());

    Json(serde_json::json!({
        "stats": stats,
        "timestamp": chrono::Utc::now()
    }))
}

/// Create error response
fn create_error_response(error: DaaMcpError) -> McpMessage {
    McpMessage {
        jsonrpc: "2.0".to_string(),
        id: None,
        method: None,
        params: None,
        result: None,
        error: Some(crate::McpError {
            code: -32603,
            message: error.to_string(),
            data: None,
        }),
    }
}

/// Background task: Monitor agent heartbeats
async fn heartbeat_monitor(state: Arc<McpServerState>) -> Result<()> {
    let mut agents = state.agents.write().await;
    let timeout = chrono::Duration::seconds(90); // 90 second timeout
    let now = chrono::Utc::now();

    let mut to_remove = Vec::new();
    
    for (id, agent) in agents.iter_mut() {
        if now.signed_duration_since(agent.last_seen) > timeout {
            warn!("Agent {} timed out", id);
            agent.status = crate::AgentStatus::Error;
            to_remove.push(id.clone());
        }
    }

    for id in to_remove {
        agents.remove(&id);
        info!("Removed timed out agent: {}", id);
    }

    Ok(())
}

/// Background task: Monitor task timeouts
async fn task_timeout_monitor(state: Arc<McpServerState>) -> Result<()> {
    let tasks = state.tasks.read().await;
    let mut results = state.task_results.write().await;
    let now = chrono::Utc::now();

    for (task_id, task) in tasks.iter() {
        if let Some(timeout) = task.timeout {
            // Check if task has a result
            if !results.contains_key(task_id) {
                // Check if task has timed out (assuming created_at field exists)
                let timeout_duration = chrono::Duration::seconds(timeout as i64);
                // For this example, we'll assume tasks are created "now - timeout"
                // In a real implementation, you'd store creation time
                
                // Create a timeout result
                let timeout_result = crate::TaskResult {
                    task_id: task_id.clone(),
                    agent_id: "system".to_string(),
                    status: crate::TaskStatus::Failed,
                    result: None,
                    error: Some("Task timed out".to_string()),
                    started_at: now,
                    completed_at: Some(now),
                    metrics: HashMap::new(),
                };
                
                results.insert(task_id.clone(), timeout_result);
                warn!("Task {} timed out", task_id);
            }
        }
    }

    Ok(())
}

/// Background task: Clean up expired swarm messages
async fn cleanup_expired_messages(state: Arc<McpServerState>) -> Result<()> {
    let mut messages = state.swarm_messages.write().await;
    let now = chrono::Utc::now();
    
    let original_count = messages.len();
    
    messages.retain(|msg| {
        if let Some(ttl) = msg.ttl {
            let expires_at = msg.timestamp + chrono::Duration::seconds(ttl as i64);
            expires_at > now
        } else {
            true // Keep messages without TTL
        }
    });

    let removed_count = original_count - messages.len();
    if removed_count > 0 {
        debug!("Cleaned up {} expired swarm messages", removed_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let config = DaaMcpConfig::default();
        let server = DaaMcpServer::new(config);
        
        let stats = server.get_stats().await;
        assert_eq!(stats.get("agent_count").unwrap(), &Value::Number(0.into()));
    }

    #[tokio::test]
    async fn test_initialize_message() {
        let config = DaaMcpConfig::default();
        let state = Arc::new(McpServerState::new(config));
        
        let init_msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(1.into())),
            method: Some("initialize".to_string()),
            params: Some(serde_json::json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
            result: None,
            error: None,
        };

        let response = handle_initialize(state, init_msg).await.unwrap();
        assert!(response.result.is_some());
    }
}