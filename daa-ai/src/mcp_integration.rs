//! MCP (Model Context Protocol) integration for QuDAG

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::error::{AiError, Result};

/// MCP request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: String,
    pub params: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl McpRequest {
    pub fn new(id: String, method: String) -> Self {
        Self {
            id,
            method,
            params: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_param(mut self, key: String, value: serde_json::Value) -> Self {
        self.params.insert(key, value);
        self
    }
}

/// MCP response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl McpResponse {
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(id: String, error: String) -> Self {
        Self {
            id,
            result: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

/// MCP client for making requests to remote services
pub struct McpClient {
    client_id: String,
    endpoint: Option<String>,
    client: reqwest::Client,
}

impl McpClient {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            endpoint: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    pub async fn send_request(&self, request: &McpRequest) -> Result<McpResponse> {
        if let Some(ref endpoint) = self.endpoint {
            let response = self.client
                .post(endpoint)
                .json(request)
                .send()
                .await?;

            if response.status().is_success() {
                let mcp_response: McpResponse = response.json().await?;
                Ok(mcp_response)
            } else {
                Err(AiError::McpError(format!(
                    "Request failed with status: {}", 
                    response.status()
                )))
            }
        } else {
            // Mock response for testing
            Ok(McpResponse::success(
                request.id.clone(),
                serde_json::json!({"mock": "response"}),
            ))
        }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

/// MCP request handler trait
#[async_trait]
pub trait McpRequestHandler: Send + Sync {
    async fn handle_request(&self, request: &McpRequest) -> Result<McpResponse>;
    fn supported_methods(&self) -> Vec<String>;
}

/// Mock MCP request handler
pub struct MockMcpHandler {
    handler_id: String,
}

impl MockMcpHandler {
    pub fn new(handler_id: String) -> Self {
        Self { handler_id }
    }
}

#[async_trait]
impl McpRequestHandler for MockMcpHandler {
    async fn handle_request(&self, request: &McpRequest) -> Result<McpResponse> {
        match request.method.as_str() {
            "ping" => Ok(McpResponse::success(
                request.id.clone(),
                serde_json::json!({"pong": "success"}),
            )),
            "echo" => Ok(McpResponse::success(
                request.id.clone(),
                serde_json::json!({"echo": request.params}),
            )),
            _ => Ok(McpResponse::error(
                request.id.clone(),
                format!("Unsupported method: {}", request.method),
            )),
        }
    }

    fn supported_methods(&self) -> Vec<String> {
        vec!["ping".to_string(), "echo".to_string()]
    }
}

/// MCP server for handling incoming requests
pub struct McpServer {
    server_id: String,
    handlers: HashMap<String, Box<dyn McpRequestHandler>>,
    request_sender: Option<mpsc::UnboundedSender<McpRequest>>,
    response_receiver: Option<mpsc::UnboundedReceiver<McpResponse>>,
}

impl McpServer {
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            handlers: HashMap::new(),
            request_sender: None,
            response_receiver: None,
        }
    }

    pub fn add_handler(&mut self, method: String, handler: Box<dyn McpRequestHandler>) {
        self.handlers.insert(method, handler);
    }

    pub async fn handle_request(&self, request: &McpRequest) -> Result<McpResponse> {
        if let Some(handler) = self.handlers.get(&request.method) {
            handler.handle_request(request).await
        } else {
            Ok(McpResponse::error(
                request.id.clone(),
                format!("No handler for method: {}", request.method),
            ))
        }
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    pub fn supported_methods(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    pub async fn start(&mut self) -> Result<()> {
        // Mock server start
        let (tx, rx) = mpsc::unbounded_channel();
        self.request_sender = Some(tx);
        self.response_receiver = Some(rx);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        // Mock server stop
        self.request_sender = None;
        self.response_receiver = None;
        Ok(())
    }
}

/// MCP integration manager
pub struct McpIntegration {
    integration_id: String,
    client: Option<McpClient>,
    server: Option<McpServer>,
}

impl McpIntegration {
    pub fn new(integration_id: String) -> Self {
        Self {
            integration_id,
            client: None,
            server: None,
        }
    }

    pub fn with_client(mut self, client: McpClient) -> Self {
        self.client = Some(client);
        self
    }

    pub fn with_server(mut self, server: McpServer) -> Self {
        self.server = Some(server);
        self
    }

    pub async fn initialize(&mut self) -> Result<()> {
        if let Some(ref mut server) = self.server {
            server.start().await?;
        }
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(ref mut server) = self.server {
            server.stop().await?;
        }
        Ok(())
    }

    pub async fn send_request(&self, request: &McpRequest) -> Result<McpResponse> {
        if let Some(ref client) = self.client {
            client.send_request(request).await
        } else {
            Err(AiError::McpError("No MCP client configured".to_string()))
        }
    }

    pub async fn handle_request(&self, request: &McpRequest) -> Result<McpResponse> {
        if let Some(ref server) = self.server {
            server.handle_request(request).await
        } else {
            Err(AiError::McpError("No MCP server configured".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_request() {
        let request = McpRequest::new(
            "req_1".to_string(),
            "ping".to_string(),
        ).with_param("test".to_string(), serde_json::json!("value"));

        assert_eq!(request.id, "req_1");
        assert_eq!(request.method, "ping");
        assert_eq!(request.params.len(), 1);
    }

    #[test]
    fn test_mcp_response() {
        let success_response = McpResponse::success(
            "req_1".to_string(),
            serde_json::json!({"result": "ok"}),
        );

        assert_eq!(success_response.id, "req_1");
        assert!(success_response.result.is_some());
        assert!(success_response.error.is_none());

        let error_response = McpResponse::error(
            "req_2".to_string(),
            "Something went wrong".to_string(),
        );

        assert_eq!(error_response.id, "req_2");
        assert!(error_response.result.is_none());
        assert!(error_response.error.is_some());
    }

    #[tokio::test]
    async fn test_mcp_client() {
        let client = McpClient::new("test_client".to_string());
        
        let request = McpRequest::new(
            "req_1".to_string(),
            "ping".to_string(),
        );

        // This will use mock response since no endpoint is set
        let response = client.send_request(&request).await.unwrap();
        assert_eq!(response.id, "req_1");
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_mock_handler() {
        let handler = MockMcpHandler::new("test_handler".to_string());
        
        let ping_request = McpRequest::new(
            "req_1".to_string(),
            "ping".to_string(),
        );

        let response = handler.handle_request(&ping_request).await.unwrap();
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let unknown_request = McpRequest::new(
            "req_2".to_string(),
            "unknown".to_string(),
        );

        let response = handler.handle_request(&unknown_request).await.unwrap();
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_mcp_server() {
        let mut server = McpServer::new("test_server".to_string());
        let handler = Box::new(MockMcpHandler::new("handler".to_string()));
        
        server.add_handler("ping".to_string(), handler);
        server.start().await.unwrap();

        let request = McpRequest::new(
            "req_1".to_string(),
            "ping".to_string(),
        );

        let response = server.handle_request(&request).await.unwrap();
        assert!(response.result.is_some());

        server.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mcp_integration() {
        let client = McpClient::new("test_client".to_string());
        let mut server = McpServer::new("test_server".to_string());
        let handler = Box::new(MockMcpHandler::new("handler".to_string()));
        
        server.add_handler("ping".to_string(), handler);

        let mut integration = McpIntegration::new("test_integration".to_string())
            .with_client(client)
            .with_server(server);

        integration.initialize().await.unwrap();

        let request = McpRequest::new(
            "req_1".to_string(),
            "ping".to_string(),
        );

        // Test client request
        let client_response = integration.send_request(&request).await.unwrap();
        assert!(client_response.result.is_some());

        // Test server request handling
        let server_response = integration.handle_request(&request).await.unwrap();
        assert!(server_response.result.is_some());

        integration.shutdown().await.unwrap();
    }
}