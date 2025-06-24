//! MCP Client for connecting to DAA management servers
//! 
//! This module provides client functionality for connecting to DAA MCP servers
//! and executing remote operations.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    ClientCapabilities, ClientInfo, DaaMcpError, InitializeRequest, InitializeResponse,
    McpMessage, Result, ToolCall, ToolResult, MCP_PROTOCOL_VERSION,
};

/// MCP Client for DAA management
pub struct DaaMcpClient {
    endpoint: String,
    client: reqwest::Client,
    session_id: Option<String>,
    server_capabilities: Option<crate::ServerCapabilities>,
    pending_requests: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<McpMessage>>>>,
    request_counter: Arc<RwLock<u64>>,
}

impl DaaMcpClient {
    /// Create a new DAA MCP client
    pub fn new(endpoint: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            endpoint,
            client,
            session_id: None,
            server_capabilities: None,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            request_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Initialize connection with the MCP server
    pub async fn initialize(&mut self) -> Result<InitializeResponse> {
        let request = InitializeRequest {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities {
                tools: Some(crate::ToolCapabilities {
                    list_changed: Some(true),
                }),
                resources: Some(crate::ResourceCapabilities {
                    subscribe: Some(true),
                    list_changed: Some(true),
                }),
                prompts: Some(crate::PromptCapabilities {
                    list_changed: Some(true),
                }),
            },
            client_info: ClientInfo {
                name: "daa-mcp-client".to_string(),
                version: "0.2.0".to_string(),
            },
        };

        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(self.next_request_id().await.into())),
            method: Some("initialize".to_string()),
            params: Some(serde_json::to_value(request)?),
            result: None,
            error: None,
        };

        let response = self.send_request(message).await?;
        
        if let Some(result) = response.result {
            let init_response: InitializeResponse = serde_json::from_value(result)?;
            self.server_capabilities = Some(init_response.capabilities.clone());
            self.session_id = Some(Uuid::new_v4().to_string());
            
            info!("Successfully initialized MCP connection to {}", self.endpoint);
            Ok(init_response)
        } else if let Some(error) = response.error {
            Err(DaaMcpError::Protocol(format!("Initialization failed: {}", error.message)))
        } else {
            Err(DaaMcpError::Protocol("Invalid initialization response".to_string()))
        }
    }

    /// List available tools on the server
    pub async fn list_tools(&self) -> Result<Vec<crate::ToolInfo>> {
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(self.next_request_id().await.into())),
            method: Some("tools/list".to_string()),
            params: None,
            result: None,
            error: None,
        };

        let response = self.send_request(message).await?;
        
        if let Some(result) = response.result {
            let tools_result = result.get("tools")
                .ok_or_else(|| DaaMcpError::Protocol("Missing tools in response".to_string()))?;
            let tools: Vec<crate::ToolInfo> = serde_json::from_value(tools_result.clone())?;
            Ok(tools)
        } else if let Some(error) = response.error {
            Err(DaaMcpError::Protocol(format!("Failed to list tools: {}", error.message)))
        } else {
            Err(DaaMcpError::Protocol("Invalid tools list response".to_string()))
        }
    }

    /// Call a tool on the server
    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<ToolResult> {
        let tool_call = ToolCall {
            name: tool_name.to_string(),
            arguments,
        };

        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(self.next_request_id().await.into())),
            method: Some("tools/call".to_string()),
            params: Some(serde_json::json!({
                "name": tool_call.name,
                "arguments": tool_call.arguments
            })),
            result: None,
            error: None,
        };

        let response = self.send_request(message).await?;
        
        if let Some(result) = response.result {
            let tool_result: ToolResult = serde_json::from_value(result)?;
            Ok(tool_result)
        } else if let Some(error) = response.error {
            Err(DaaMcpError::Protocol(format!("Tool call failed: {}", error.message)))
        } else {
            Err(DaaMcpError::Protocol("Invalid tool call response".to_string()))
        }
    }

    /// List available resources on the server
    pub async fn list_resources(&self) -> Result<Vec<crate::ResourceInfo>> {
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(self.next_request_id().await.into())),
            method: Some("resources/list".to_string()),
            params: None,
            result: None,
            error: None,
        };

        let response = self.send_request(message).await?;
        
        if let Some(result) = response.result {
            let resources_result = result.get("resources")
                .ok_or_else(|| DaaMcpError::Protocol("Missing resources in response".to_string()))?;
            let resources: Vec<crate::ResourceInfo> = serde_json::from_value(resources_result.clone())?;
            Ok(resources)
        } else if let Some(error) = response.error {
            Err(DaaMcpError::Protocol(format!("Failed to list resources: {}", error.message)))
        } else {
            Err(DaaMcpError::Protocol("Invalid resources list response".to_string()))
        }
    }

    /// Read a resource from the server
    pub async fn read_resource(&self, uri: &str) -> Result<Vec<crate::Content>> {
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(self.next_request_id().await.into())),
            method: Some("resources/read".to_string()),
            params: Some(serde_json::json!({
                "uri": uri
            })),
            result: None,
            error: None,
        };

        let response = self.send_request(message).await?;
        
        if let Some(result) = response.result {
            let contents_result = result.get("contents")
                .ok_or_else(|| DaaMcpError::Protocol("Missing contents in response".to_string()))?;
            let contents: Vec<crate::Content> = serde_json::from_value(contents_result.clone())?;
            Ok(contents)
        } else if let Some(error) = response.error {
            Err(DaaMcpError::Protocol(format!("Failed to read resource: {}", error.message)))
        } else {
            Err(DaaMcpError::Protocol("Invalid resource read response".to_string()))
        }
    }

    /// List available prompts on the server
    pub async fn list_prompts(&self) -> Result<Vec<crate::prompts::PromptInfo>> {
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(self.next_request_id().await.into())),
            method: Some("prompts/list".to_string()),
            params: None,
            result: None,
            error: None,
        };

        let response = self.send_request(message).await?;
        
        if let Some(result) = response.result {
            let prompts_result = result.get("prompts")
                .ok_or_else(|| DaaMcpError::Protocol("Missing prompts in response".to_string()))?;
            let prompts: Vec<crate::prompts::PromptInfo> = serde_json::from_value(prompts_result.clone())?;
            Ok(prompts)
        } else if let Some(error) = response.error {
            Err(DaaMcpError::Protocol(format!("Failed to list prompts: {}", error.message)))
        } else {
            Err(DaaMcpError::Protocol("Invalid prompts list response".to_string()))
        }
    }

    /// Get a prompt from the server
    pub async fn get_prompt(&self, name: &str, arguments: Value) -> Result<crate::prompts::PromptResponse> {
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(self.next_request_id().await.into())),
            method: Some("prompts/get".to_string()),
            params: Some(serde_json::json!({
                "name": name,
                "arguments": arguments
            })),
            result: None,
            error: None,
        };

        let response = self.send_request(message).await?;
        
        if let Some(result) = response.result {
            let prompt: crate::prompts::PromptResponse = serde_json::from_value(result)?;
            Ok(prompt)
        } else if let Some(error) = response.error {
            Err(DaaMcpError::Protocol(format!("Failed to get prompt: {}", error.message)))
        } else {
            Err(DaaMcpError::Protocol("Invalid prompt response".to_string()))
        }
    }

    /// Send a raw MCP message to the server
    async fn send_request(&self, message: McpMessage) -> Result<McpMessage> {
        debug!("Sending MCP request: {:?}", message);

        let url = if self.endpoint.ends_with('/') {
            format!("{}mcp", self.endpoint)
        } else {
            format!("{}/mcp", self.endpoint)
        };

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&message)
            .send()
            .await
            .map_err(|e| DaaMcpError::Network(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if !response.status().is_success() {
            return Err(DaaMcpError::Network(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP error: {}", response.status()),
            )));
        }

        let response_message: McpMessage = response.json().await
            .map_err(|e| DaaMcpError::Network(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        debug!("Received MCP response: {:?}", response_message);
        Ok(response_message)
    }

    /// Get the next request ID
    async fn next_request_id(&self) -> u64 {
        let mut counter = self.request_counter.write().await;
        *counter += 1;
        *counter
    }

    /// Get server capabilities
    pub fn server_capabilities(&self) -> Option<&crate::ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    /// Check if connected and initialized
    pub fn is_connected(&self) -> bool {
        self.session_id.is_some()
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

/// High-level DAA operations client
pub struct DaaOperationsClient {
    client: DaaMcpClient,
}

impl DaaOperationsClient {
    /// Create a new DAA operations client
    pub async fn connect(endpoint: String) -> Result<Self> {
        let mut client = DaaMcpClient::new(endpoint);
        client.initialize().await?;
        
        Ok(Self { client })
    }

    /// Spawn a new DAA agent
    pub async fn spawn_agent(&self, config: crate::AgentConfig) -> Result<String> {
        let result = self.client.call_tool("spawn_agent", serde_json::json!({
            "config": config
        })).await?;

        // Extract agent ID from the result
        if let Some(content) = result.content {
            if let Some(first_content) = content.first() {
                // Parse the agent ID from the response text
                // This is a simple implementation - in practice, you'd want more robust parsing
                if first_content.text.contains("ID:") {
                    let parts: Vec<&str> = first_content.text.split("ID:").collect();
                    if parts.len() > 1 {
                        return Ok(parts[1].trim().to_string());
                    }
                }
            }
        }

        Err(DaaMcpError::Protocol("Failed to extract agent ID from response".to_string()))
    }

    /// Stop a DAA agent
    pub async fn stop_agent(&self, agent_id: &str) -> Result<()> {
        let result = self.client.call_tool("stop_agent", serde_json::json!({
            "agent_id": agent_id
        })).await?;

        if result.is_error == Some(true) {
            Err(DaaMcpError::Protocol("Failed to stop agent".to_string()))
        } else {
            Ok(())
        }
    }

    /// List all agents
    pub async fn list_agents(&self) -> Result<Vec<crate::DaaAgentInfo>> {
        let result = self.client.call_tool("list_agents", serde_json::json!({})).await?;

        if let Some(content) = result.content {
            if let Some(first_content) = content.first() {
                if first_content.content_type == "application/json" {
                    let data: serde_json::Value = serde_json::from_str(&first_content.text)?;
                    if let Some(agents_array) = data.get("agents") {
                        let agents: Vec<crate::DaaAgentInfo> = serde_json::from_value(agents_array.clone())?;
                        return Ok(agents);
                    }
                }
            }
        }

        Err(DaaMcpError::Protocol("Failed to parse agents list".to_string()))
    }

    /// Create a new task
    pub async fn create_task(&self, task: crate::DaaTask) -> Result<String> {
        let result = self.client.call_tool("create_task", serde_json::json!({
            "task_type": task.task_type,
            "description": task.description,
            "parameters": task.parameters,
            "priority": task.priority,
            "timeout": task.timeout,
            "dependencies": task.dependencies,
            "assigned_agents": task.assigned_agents
        })).await?;

        if let Some(content) = result.content {
            if let Some(first_content) = content.first() {
                // Extract task ID from response
                if first_content.text.contains("ID:") {
                    let parts: Vec<&str> = first_content.text.split("ID:").collect();
                    if parts.len() > 1 {
                        return Ok(parts[1].trim().to_string());
                    }
                }
            }
        }

        Err(DaaMcpError::Protocol("Failed to extract task ID from response".to_string()))
    }

    /// Coordinate agent swarm
    pub async fn coordinate_swarm(
        &self,
        objective: &str,
        agent_types: Vec<String>,
        coordination_strategy: Option<String>,
    ) -> Result<String> {
        let result = self.client.call_tool("coordinate_swarm", serde_json::json!({
            "objective": objective,
            "agent_types": agent_types,
            "coordination_strategy": coordination_strategy.unwrap_or_else(|| "hierarchical".to_string())
        })).await?;

        if let Some(content) = result.content {
            if let Some(first_content) = content.first() {
                if first_content.content_type == "application/json" {
                    let data: serde_json::Value = serde_json::from_str(&first_content.text)?;
                    if let Some(swarm_id) = data.get("swarm_id").and_then(|v| v.as_str()) {
                        return Ok(swarm_id.to_string());
                    }
                }
            }
        }

        Err(DaaMcpError::Protocol("Failed to extract swarm ID from response".to_string()))
    }

    /// Get system metrics
    pub async fn get_system_metrics(&self) -> Result<serde_json::Value> {
        let result = self.client.call_tool("get_system_metrics", serde_json::json!({})).await?;

        if let Some(content) = result.content {
            if let Some(first_content) = content.first() {
                if first_content.content_type == "application/json" {
                    let metrics: serde_json::Value = serde_json::from_str(&first_content.text)?;
                    return Ok(metrics);
                }
            }
        }

        Err(DaaMcpError::Protocol("Failed to parse system metrics".to_string()))
    }

    /// Perform health check
    pub async fn health_check(&self, deep_check: bool) -> Result<serde_json::Value> {
        let result = self.client.call_tool("healthcheck", serde_json::json!({
            "deep_check": deep_check
        })).await?;

        if let Some(content) = result.content {
            if let Some(first_content) = content.first() {
                if first_content.content_type == "application/json" {
                    let health: serde_json::Value = serde_json::from_str(&first_content.text)?;
                    return Ok(health);
                }
            }
        }

        Err(DaaMcpError::Protocol("Failed to parse health check result".to_string()))
    }

    /// Get the underlying MCP client
    pub fn mcp_client(&self) -> &DaaMcpClient {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_client_creation() {
        let client = DaaMcpClient::new("http://localhost:3001".to_string());
        assert!(!client.is_connected());
        assert!(client.session_id().is_none());
    }

    #[tokio::test]
    async fn test_request_id_generation() {
        let client = DaaMcpClient::new("http://localhost:3001".to_string());
        
        let id1 = client.next_request_id().await;
        let id2 = client.next_request_id().await;
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }
}