//! Integration tests for QuDAG MCP implementation

use qudag_mcp::*;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

/// Mock transport for testing that simulates client-server communication
struct MockTransport {
    incoming: mpsc::UnboundedReceiver<MCPMessage>,
    outgoing: mpsc::UnboundedSender<MCPMessage>,
    peer_incoming: mpsc::UnboundedSender<MCPMessage>,
    peer_outgoing: mpsc::UnboundedReceiver<MCPMessage>,
    connected: bool,
}

impl MockTransport {
    fn create_pair() -> (MockTransport, MockTransport) {
        let (tx1, rx1) = mpsc::unbounded_channel();
        let (tx2, rx2) = mpsc::unbounded_channel();

        let transport1 = MockTransport {
            incoming: rx2,
            outgoing: tx1.clone(),
            peer_incoming: tx1,
            peer_outgoing: rx1,
            connected: true,
        };

        let transport2 = MockTransport {
            incoming: rx1,
            outgoing: tx2.clone(),
            peer_incoming: tx2,
            peer_outgoing: rx2,
            connected: true,
        };

        (transport1, transport2)
    }
}

#[async_trait::async_trait]
impl Transport for MockTransport {
    async fn send(&mut self, message: MCPMessage) -> Result<()> {
        if !self.connected {
            return Err(MCPError::ConnectionLost);
        }

        self.outgoing
            .send(message)
            .map_err(|_| MCPError::ConnectionLost)?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<MCPMessage>> {
        if !self.connected {
            return Err(MCPError::ConnectionLost);
        }

        match timeout(Duration::from_millis(100), self.incoming.recv()).await {
            Ok(Some(message)) => Ok(Some(message)),
            Ok(None) => {
                self.connected = false;
                Ok(None)
            }
            Err(_) => Ok(None), // Timeout
        }
    }

    async fn close(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// Test helper to create a test server with mock transport
async fn create_test_server() -> Result<(QuDAGMCPServer, MockTransport)> {
    let (server_transport, client_transport) = MockTransport::create_pair();

    let config = ServerConfig::new()
        .with_server_info("Test QuDAG MCP Server", "1.0.0-test")
        .with_log_level("debug");

    let mut server = QuDAGMCPServer::new(config).await?;

    // Manually set the transport for testing
    // Note: This requires exposing the transport field or adding a test method
    // For now, we'll test the server components individually

    Ok((server, server_transport))
}

/// Test helper to create a test client with mock transport
async fn create_test_client() -> Result<(QuDAGMCPClient, MockTransport)> {
    let (client_transport, server_transport) = MockTransport::create_pair();

    let config = ClientConfig::new()
        .with_client_info("Test QuDAG MCP Client", "1.0.0-test")
        .with_timeout(Duration::from_secs(5))
        .with_log_level("debug");

    let client = QuDAGMCPClient::new(config).await?;

    Ok((client, client_transport))
}

#[tokio::test]
async fn test_server_initialization() {
    let config = ServerConfig::new()
        .with_server_info("Test Server", "1.0.0")
        .with_log_level("debug");

    let server = QuDAGMCPServer::new(config).await.unwrap();

    // Test initial state
    assert!(matches!(server.state().await, ServerState::Uninitialized));

    // Test server statistics
    let stats = server.stats().await;
    assert!(stats.tools_count > 0);
    assert!(stats.resources_count > 0);
    assert_eq!(stats.active_subscriptions, 0);
    assert!(!stats.client_connected);
}

#[tokio::test]
async fn test_client_initialization() {
    let config = ClientConfig::new()
        .with_client_info("Test Client", "1.0.0")
        .with_timeout(Duration::from_secs(5))
        .with_log_level("debug");

    let client = QuDAGMCPClient::new(config).await.unwrap();

    // Test initial state
    assert!(matches!(client.state().await, ClientState::Disconnected));
    assert!(!client.is_connected().await);

    // Test client statistics
    let stats = client.stats().await;
    assert_eq!(stats.pending_requests, 0);
    assert!(!stats.server_connected);
}

#[tokio::test]
async fn test_mcp_protocol_messages() {
    // Test MCPRequest creation
    let request = MCPRequest::initialize(ClientInfo::new("test-client", "1.0.0"), HashMap::new());
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "initialize");
    assert!(request.params.is_some());

    // Test message serialization
    let message = MCPMessage::Request(request);
    let json = message.to_json().unwrap();
    let parsed = MCPMessage::from_json(&json).unwrap();

    assert!(parsed.is_request());
    assert_eq!(parsed.method(), Some("initialize"));

    // Test MCPResponse creation
    let response = MCPResponse::success(RequestId::generate(), serde_json::json!({"status": "ok"}));
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    // Test MCPNotification creation
    let notification = MCPNotification::initialized();
    assert_eq!(notification.jsonrpc, "2.0");
    assert_eq!(notification.method, "notifications/initialized");
}

#[tokio::test]
async fn test_tool_registry_integration() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test listing tools
    let tools = registry.list_tools();
    assert!(!tools.is_empty());

    // Verify we have expected QuDAG tools
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.as_str().to_string()).collect();
    assert!(tool_names.contains(&"dag_add_vertex".to_string()));
    assert!(tool_names.contains(&"dag_get_tips".to_string()));
    assert!(tool_names.contains(&"crypto_generate_keypair".to_string()));
    assert!(tool_names.contains(&"crypto_sign".to_string()));

    // Test tool execution
    let tool_name = ToolName::new("dag_get_tips");
    let result = registry.call_tool(&tool_name, None).await.unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());
}

#[tokio::test]
async fn test_resource_registry_integration() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test listing resources
    let resources = registry.list_resources().await.unwrap();
    assert!(!resources.is_empty());

    // Verify we have expected QuDAG resources
    let resource_uris: Vec<String> = resources.iter().map(|r| r.uri.to_string()).collect();
    assert!(resource_uris.iter().any(|uri| uri.starts_with("dag://")));
    assert!(resource_uris.iter().any(|uri| uri.starts_with("crypto://")));
    assert!(resource_uris.iter().any(|uri| uri.starts_with("vault://")));
    assert!(resource_uris
        .iter()
        .any(|uri| uri.starts_with("network://")));

    // Test resource reading
    let dag_uri = ResourceURI::dag("vertices/all");
    let contents = registry.read_resource(&dag_uri).await.unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0].uri, dag_uri);
    assert!(contents[0].mime_type.as_ref().unwrap().contains("json"));
}

#[tokio::test]
async fn test_error_handling() {
    use qudag_mcp::resources::ResourceRegistry;
    use qudag_mcp::tools::ToolRegistry;

    let tool_registry = ToolRegistry::new();
    let resource_registry = ResourceRegistry::new();

    // Test tool not found error
    let nonexistent_tool = ToolName::new("nonexistent_tool");
    let result = tool_registry.call_tool(&nonexistent_tool, None).await;
    assert!(matches!(result, Err(MCPError::ToolNotFound { .. })));

    // Test resource not found error
    let nonexistent_resource = ResourceURI::new("invalid://nonexistent/resource");
    let result = resource_registry.read_resource(&nonexistent_resource).await;
    assert!(matches!(result, Err(MCPError::ResourceNotFound { .. })));

    // Test JSON-RPC error codes
    let error = MCPError::ToolNotFound {
        name: "test".to_string(),
    };
    assert_eq!(error.json_rpc_code(), -32020);

    let error = MCPError::ResourceNotFound {
        uri: "test://uri".to_string(),
    };
    assert_eq!(error.json_rpc_code(), -32010);
}

#[tokio::test]
async fn test_tool_execution_with_parameters() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test DAG add vertex tool with parameters
    let args = serde_json::json!({
        "id": "test_vertex_123",
        "payload": "test vertex payload",
        "parents": ["parent1", "parent2"]
    });

    let tool_name = ToolName::new("dag_add_vertex");
    let result = registry.call_tool(&tool_name, Some(args)).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify the result contains expected data
    if let ToolResultContent::Text { text } = &result.content[0] {
        let result_data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(result_data["vertex_id"], "test_vertex_123");
        assert_eq!(result_data["success"], true);
    } else {
        panic!("Expected text content in tool result");
    }
}

#[tokio::test]
async fn test_crypto_tool_execution() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test crypto key generation
    let args = serde_json::json!({
        "algorithm": "ml-kem",
        "security_level": 3
    });

    let tool_name = ToolName::new("crypto_generate_keypair");
    let result = registry.call_tool(&tool_name, Some(args)).await.unwrap();

    assert_eq!(result.is_error, Some(false));

    // Verify the result contains crypto data
    if let ToolResultContent::Text { text } = &result.content[0] {
        let result_data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(result_data["algorithm"], "ml-kem");
        assert_eq!(result_data["security_level"], 3);
        assert!(result_data["public_key"].is_string());
        assert!(result_data["private_key"].is_string());
    }
}

#[tokio::test]
async fn test_resource_content_formats() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test DAG resources
    let dag_uri = ResourceURI::dag("stats/summary");
    let contents = registry.read_resource(&dag_uri).await.unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0].mime_type.as_ref().unwrap(), "application/json");

    // Parse the JSON content
    if let Some(text) = &contents[0].text {
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(data["total_vertices"].is_number());
        assert!(data["total_edges"].is_number());
        assert!(data["last_updated"].is_string());
    }

    // Test crypto resources
    let crypto_uri = ResourceURI::crypto("algorithms/supported");
    let contents = registry.read_resource(&crypto_uri).await.unwrap();
    assert_eq!(contents.len(), 1);

    if let Some(text) = &contents[0].text {
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(data["algorithms"].is_array());
        let algorithms = data["algorithms"].as_array().unwrap();
        assert!(!algorithms.is_empty());

        // Verify quantum-resistant algorithms are listed
        let algo_names: Vec<&str> = algorithms
            .iter()
            .filter_map(|a| a["name"].as_str())
            .collect();
        assert!(algo_names.contains(&"ML-KEM"));
        assert!(algo_names.contains(&"ML-DSA"));
    }
}

#[tokio::test]
async fn test_server_request_handling() {
    let config = ServerConfig::new();
    let server = QuDAGMCPServer::new(config).await.unwrap();

    // Test initialize request
    let init_request =
        MCPRequest::initialize(ClientInfo::new("test-client", "1.0.0"), HashMap::new());

    let response = server.handle_initialize(&init_request).await.unwrap();
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    if let Some(result) = response.result {
        assert_eq!(result["protocolVersion"], crate::MCP_PROTOCOL_VERSION);
        assert!(result["serverInfo"].is_object());
        assert!(result["capabilities"].is_object());
    }

    // Test tools/list request
    let tools_request = MCPRequest::list_tools();
    let response = server.handle_tools_list(&tools_request).await.unwrap();
    assert!(response.result.is_some());

    if let Some(result) = response.result {
        let tools = result["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
    }

    // Test resources/list request
    let resources_request = MCPRequest::list_resources();
    let response = server
        .handle_resources_list(&resources_request)
        .await
        .unwrap();
    assert!(response.result.is_some());

    if let Some(result) = response.result {
        let resources = result["resources"].as_array().unwrap();
        assert!(!resources.is_empty());
    }
}

#[tokio::test]
async fn test_invalid_request_handling() {
    let config = ServerConfig::new();
    let server = QuDAGMCPServer::new(config).await.unwrap();

    // Test invalid method
    let invalid_request = MCPRequest::new("test-id", "invalid/method");
    let response = server.handle_request(invalid_request).await;
    assert!(response.error.is_some());

    if let Some(error) = response.error {
        assert_eq!(error["code"], -32601); // Method not found
    }

    // Test invalid parameters for tools/call
    let invalid_params_request = MCPRequest::new("test-id", "tools/call")
        .with_params(serde_json::json!({"invalid": "params"}));
    let response = server.handle_request(invalid_params_request).await;
    assert!(response.error.is_some());
}

#[tokio::test]
async fn test_concurrent_operations() {
    use qudag_mcp::tools::ToolRegistry;
    use std::sync::Arc;

    let registry = Arc::new(ToolRegistry::new());
    let mut handles = vec![];

    // Spawn multiple concurrent tool executions
    for i in 0..10 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            let tool_name = ToolName::new("dag_get_tips");
            let result = registry_clone.call_tool(&tool_name, None).await;
            (i, result)
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let (i, result) = handle.await.unwrap();
        assert!(result.is_ok(), "Operation {} failed: {:?}", i, result);
    }
}

#[tokio::test]
async fn test_resource_uri_parsing() {
    // Test URI scheme parsing
    let dag_uri = ResourceURI::dag("vertices/all");
    assert_eq!(dag_uri.scheme(), Some("dag"));
    assert_eq!(dag_uri.path(), Some("vertices/all"));

    let crypto_uri = ResourceURI::crypto("keys/public");
    assert_eq!(crypto_uri.scheme(), Some("crypto"));
    assert_eq!(crypto_uri.path(), Some("keys/public"));

    let vault_uri = ResourceURI::vault("secrets/config");
    assert_eq!(vault_uri.scheme(), Some("vault"));
    assert_eq!(vault_uri.path(), Some("secrets/config"));

    let network_uri = ResourceURI::network("peers/connected");
    assert_eq!(network_uri.scheme(), Some("network"));
    assert_eq!(network_uri.path(), Some("peers/connected"));

    // Test invalid URI
    let invalid_uri = ResourceURI::new("invalid-uri-without-scheme");
    assert_eq!(invalid_uri.scheme(), None);
    assert_eq!(invalid_uri.path(), None);
}

#[tokio::test]
async fn test_tool_schema_validation() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();
    let tools = registry.list_tools();

    // Verify all tools have proper schemas
    for tool in tools {
        assert!(!tool.name.as_str().is_empty());
        assert!(!tool.description.is_empty());
        assert!(tool.input_schema.is_object());

        let schema = tool.input_schema.as_object().unwrap();
        assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "object");

        if let Some(properties) = schema.get("properties") {
            assert!(properties.is_object());
        }

        if let Some(required) = schema.get("required") {
            assert!(required.is_array());
        }
    }
}

#[tokio::test]
async fn test_memory_safety_and_cleanup() {
    // Test that we can create and drop many instances without issues
    for _ in 0..100 {
        let config = ServerConfig::new();
        let _server = QuDAGMCPServer::new(config).await.unwrap();

        let config = ClientConfig::new();
        let _client = QuDAGMCPClient::new(config).await.unwrap();
    }

    // Test concurrent creation and cleanup
    let mut handles = vec![];
    for _ in 0..50 {
        let handle = tokio::spawn(async {
            let config = ServerConfig::new();
            let server = QuDAGMCPServer::new(config).await.unwrap();
            let _stats = server.stats().await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
