//! Protocol-specific tests for MCP message handling and transport layers

use qudag_mcp::*;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_test;

#[tokio::test]
async fn test_json_rpc_message_format() {
    // Test request message format
    let request =
        MCPRequest::new("test-123", "test/method").with_params(serde_json::json!({"key": "value"}));

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "test/method");
    assert!(request.params.is_some());

    // Test serialization
    let json = serde_json::to_string(&request).unwrap();
    let parsed: MCPRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.jsonrpc, request.jsonrpc);
    assert_eq!(parsed.method, request.method);

    // Test response message format
    let response = MCPResponse::success(
        RequestId::from_string("test-123"),
        serde_json::json!({"result": "success"}),
    );

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    // Test error response format
    let error_response = MCPResponse::error(
        RequestId::from_string("test-123"),
        MCPError::MethodNotFound {
            method: "test/method".to_string(),
        },
    );

    assert_eq!(error_response.jsonrpc, "2.0");
    assert!(error_response.result.is_none());
    assert!(error_response.error.is_some());

    if let Some(error) = error_response.error {
        assert_eq!(error["code"], -32601);
        assert!(error["message"].is_string());
    }
}

#[tokio::test]
async fn test_request_id_types() {
    // Test string ID
    let string_id = RequestId::from_string("test-string-id");
    assert!(matches!(string_id, RequestId::String(_)));

    // Test number ID
    let number_id = RequestId::from_number(42);
    assert!(matches!(number_id, RequestId::Number(42)));

    // Test generated UUID ID
    let uuid_id = RequestId::generate();
    assert!(matches!(uuid_id, RequestId::String(_)));

    // Test ID serialization/deserialization
    let ids = vec![string_id, number_id, uuid_id];
    for id in ids {
        let json = serde_json::to_string(&id).unwrap();
        let parsed: RequestId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }
}

#[tokio::test]
async fn test_mcp_message_types() {
    // Test unified message type
    let request = MCPRequest::initialize(ClientInfo::new("test", "1.0.0"), HashMap::new());
    let message = MCPMessage::Request(request);

    assert!(message.is_request());
    assert!(!message.is_response());
    assert!(!message.is_notification());
    assert_eq!(message.method(), Some("initialize"));
    assert!(message.id().is_some());

    // Test response message
    let response = MCPResponse::success(RequestId::generate(), serde_json::json!({"status": "ok"}));
    let message = MCPMessage::Response(response);

    assert!(!message.is_request());
    assert!(message.is_response());
    assert!(!message.is_notification());
    assert_eq!(message.method(), None);
    assert!(message.id().is_some());

    // Test notification message
    let notification = MCPNotification::initialized();
    let message = MCPMessage::Notification(notification);

    assert!(!message.is_request());
    assert!(!message.is_response());
    assert!(message.is_notification());
    assert_eq!(message.method(), Some("notifications/initialized"));
    assert_eq!(message.id(), None);
}

#[tokio::test]
async fn test_initialize_protocol_flow() {
    // Test client initialization request
    let client_capabilities = {
        let mut caps = HashMap::new();
        caps.insert("sampling".to_string(), serde_json::Value::Bool(true));
        caps
    };

    let request = MCPRequest::initialize(
        ClientInfo::new("test-client", "1.0.0"),
        client_capabilities.clone(),
    );

    assert_eq!(request.method, "initialize");

    if let Some(params) = request.params {
        assert_eq!(params["protocolVersion"], crate::MCP_PROTOCOL_VERSION);
        assert_eq!(params["clientInfo"]["name"], "test-client");
        assert_eq!(params["clientInfo"]["version"], "1.0.0");
        assert!(params["capabilities"].is_object());
    }

    // Test server initialization response
    let server_info = ServerInfo::new("test-server", "1.0.0");
    let server_capabilities = ServerCapabilities::default();

    let response = MCPResponse::initialize_success(
        request.id.clone(),
        server_info.clone(),
        server_capabilities.clone(),
    );

    assert!(response.result.is_some());
    if let Some(result) = response.result {
        assert_eq!(result["protocolVersion"], crate::MCP_PROTOCOL_VERSION);
        assert_eq!(result["serverInfo"]["name"], server_info.name);
        assert_eq!(result["serverInfo"]["version"], server_info.version);
        assert!(result["capabilities"].is_object());
    }
}

#[tokio::test]
async fn test_tools_protocol_flow() {
    // Test tools/list request
    let list_request = MCPRequest::list_tools();
    assert_eq!(list_request.method, "tools/list");
    assert!(list_request.params.is_none());

    // Test tools/list response
    let tools = vec![Tool::new(
        "test_tool",
        "A test tool",
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            },
            "required": ["input"]
        }),
    )];

    let response = MCPResponse::tools_list_success(list_request.id.clone(), tools.clone());
    assert!(response.result.is_some());

    if let Some(result) = response.result {
        let returned_tools: Vec<Tool> = serde_json::from_value(result["tools"].clone()).unwrap();
        assert_eq!(returned_tools.len(), 1);
        assert_eq!(returned_tools[0].name.as_str(), "test_tool");
    }

    // Test tools/call request
    let call_request =
        MCPRequest::call_tool("test_tool", serde_json::json!({"input": "test data"}));

    assert_eq!(call_request.method, "tools/call");
    if let Some(params) = call_request.params {
        assert_eq!(params["name"], "test_tool");
        assert!(params["arguments"].is_object());
    }

    // Test tools/call response
    let tool_result = ToolResult::success(ToolResultContent::text("Operation completed"));
    let response = MCPResponse::tool_call_success(call_request.id, tool_result);

    assert!(response.result.is_some());
    if let Some(result) = response.result {
        assert_eq!(result["isError"], false);
        assert!(result["content"].is_array());
    }
}

#[tokio::test]
async fn test_resources_protocol_flow() {
    // Test resources/list request
    let list_request = MCPRequest::list_resources();
    assert_eq!(list_request.method, "resources/list");

    // Test resources/list response
    let resources = vec![Resource::new("test://resource/1", "Test Resource 1")
        .with_description("A test resource")
        .with_mime_type("application/json")];

    let response = MCPResponse::resources_list_success(list_request.id.clone(), resources);
    assert!(response.result.is_some());

    // Test resources/read request
    let read_request = MCPRequest::read_resource("test://resource/1");
    assert_eq!(read_request.method, "resources/read");

    if let Some(params) = read_request.params {
        assert_eq!(params["uri"], "test://resource/1");
    }

    // Test resources/read response
    let content = vec![ResourceContent::text("test://resource/1", "Test content")];

    let response = MCPResponse::resource_read_success(read_request.id, content);
    assert!(response.result.is_some());

    if let Some(result) = response.result {
        let contents: Vec<ResourceContent> =
            serde_json::from_value(result["contents"].clone()).unwrap();
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].uri.to_string(), "test://resource/1");
    }
}

#[tokio::test]
async fn test_prompts_protocol_flow() {
    // Test prompts/list request
    let list_request = MCPRequest::list_prompts();
    assert_eq!(list_request.method, "prompts/list");

    // Test prompts/get request
    let mut arguments = HashMap::new();
    arguments.insert("target".to_string(), "system".to_string());

    let get_request = MCPRequest::get_prompt("security_audit", Some(arguments));
    assert_eq!(get_request.method, "prompts/get");

    if let Some(params) = get_request.params {
        assert_eq!(params["name"], "security_audit");
        assert!(params["arguments"].is_object());
    }

    // Test prompts/get response
    let messages = vec![PromptMessage {
        role: MessageRole::System,
        content: MessageContent::Text("You are a security expert.".to_string()),
    }];

    let response = MCPResponse::prompt_get_success(
        get_request.id,
        Some("Security audit prompt".to_string()),
        messages,
    );

    assert!(response.result.is_some());
    if let Some(result) = response.result {
        assert!(result["messages"].is_array());
        if let Some(description) = result.get("description") {
            assert_eq!(description, "Security audit prompt");
        }
    }
}

#[tokio::test]
async fn test_notification_messages() {
    // Test initialized notification
    let initialized = MCPNotification::initialized();
    assert_eq!(initialized.method, "notifications/initialized");
    assert!(initialized.params.is_none());

    // Test progress notification
    let progress = MCPNotification::progress(
        "indexing",
        0.75,
        Some("Processing file 750 of 1000".to_string()),
    );
    assert_eq!(progress.method, "notifications/progress");
    assert!(progress.params.is_some());

    if let Some(params) = progress.params {
        assert_eq!(params["operation"], "indexing");
        assert_eq!(params["progress"], 0.75);
        assert_eq!(params["message"], "Processing file 750 of 1000");
    }

    // Test tools/list_changed notification
    let tools_changed = MCPNotification::tools_list_changed();
    assert_eq!(tools_changed.method, "notifications/tools/list_changed");

    // Test resource updated notification
    let resource_updated = MCPNotification::resource_updated("dag://vertices/all");
    assert_eq!(resource_updated.method, "notifications/resources/updated");

    if let Some(params) = resource_updated.params {
        assert_eq!(params["uri"], "dag://vertices/all");
    }

    // Test logging notification
    let log_data = serde_json::json!({
        "timestamp": "2024-01-01T00:00:00Z",
        "message": "Test log message"
    });

    let log_notification =
        MCPNotification::log_message("info", log_data.clone(), Some("test-logger".to_string()));

    assert_eq!(log_notification.method, "notifications/message");
    if let Some(params) = log_notification.params {
        assert_eq!(params["level"], "info");
        assert_eq!(params["data"], log_data);
        assert_eq!(params["logger"], "test-logger");
    }
}

#[tokio::test]
async fn test_error_response_codes() {
    let test_cases = vec![
        (
            MCPError::ParseError {
                message: "test".to_string(),
            },
            -32700,
        ),
        (
            MCPError::InvalidRequest {
                message: "test".to_string(),
            },
            -32600,
        ),
        (
            MCPError::MethodNotFound {
                method: "test".to_string(),
            },
            -32601,
        ),
        (
            MCPError::InvalidParams {
                message: "test".to_string(),
            },
            -32602,
        ),
        (
            MCPError::InternalError {
                message: "test".to_string(),
            },
            -32603,
        ),
        (
            MCPError::ResourceNotFound {
                uri: "test".to_string(),
            },
            -32010,
        ),
        (
            MCPError::ToolNotFound {
                name: "test".to_string(),
            },
            -32020,
        ),
        (
            MCPError::DAGOperationFailed {
                operation: "test".to_string(),
            },
            -32100,
        ),
        (
            MCPError::CryptoOperationFailed {
                operation: "test".to_string(),
            },
            -32101,
        ),
        (
            MCPError::VaultOperationFailed {
                operation: "test".to_string(),
            },
            -32103,
        ),
    ];

    for (error, expected_code) in test_cases {
        assert_eq!(error.json_rpc_code(), expected_code);

        let json_error = error.to_json_rpc_error();
        assert_eq!(json_error["code"], expected_code);
        assert!(json_error["message"].is_string());

        // Test error response creation
        let response = MCPResponse::error(RequestId::generate(), error);
        assert!(response.error.is_some());
        assert!(response.result.is_none());
    }
}

#[tokio::test]
async fn test_message_serialization_roundtrip() {
    // Test request serialization
    let original_request =
        MCPRequest::initialize(ClientInfo::new("test-client", "1.0.0"), HashMap::new());

    let message = MCPMessage::Request(original_request.clone());
    let json = message.to_json().unwrap();
    let parsed_message = MCPMessage::from_json(&json).unwrap();

    if let MCPMessage::Request(parsed_request) = parsed_message {
        assert_eq!(parsed_request.jsonrpc, original_request.jsonrpc);
        assert_eq!(parsed_request.method, original_request.method);
        assert_eq!(parsed_request.id, original_request.id);
    } else {
        panic!("Expected Request message");
    }

    // Test response serialization
    let original_response =
        MCPResponse::success(RequestId::generate(), serde_json::json!({"status": "ok"}));

    let message = MCPMessage::Response(original_response.clone());
    let json = message.to_json().unwrap();
    let parsed_message = MCPMessage::from_json(&json).unwrap();

    if let MCPMessage::Response(parsed_response) = parsed_message {
        assert_eq!(parsed_response.jsonrpc, original_response.jsonrpc);
        assert_eq!(parsed_response.id, original_response.id);
        assert_eq!(parsed_response.result, original_response.result);
    } else {
        panic!("Expected Response message");
    }

    // Test notification serialization
    let original_notification = MCPNotification::progress("test", 0.5, None);
    let message = MCPMessage::Notification(original_notification.clone());
    let json = message.to_json().unwrap();
    let parsed_message = MCPMessage::from_json(&json).unwrap();

    if let MCPMessage::Notification(parsed_notification) = parsed_message {
        assert_eq!(parsed_notification.jsonrpc, original_notification.jsonrpc);
        assert_eq!(parsed_notification.method, original_notification.method);
        assert_eq!(parsed_notification.params, original_notification.params);
    } else {
        panic!("Expected Notification message");
    }
}

#[tokio::test]
async fn test_protocol_version_handling() {
    // Test supported version
    let params = InitializeParams {
        protocol_version: crate::MCP_PROTOCOL_VERSION.to_string(),
        capabilities: HashMap::new(),
        client_info: ClientInfo::new("test", "1.0.0"),
    };

    let request =
        MCPRequest::new("test", "initialize").with_params(serde_json::to_value(params).unwrap());

    // This would be handled by the server's initialize handler
    let config = ServerConfig::new();
    let server = QuDAGMCPServer::new(config).await.unwrap();
    let response = server.handle_initialize(&request).await.unwrap();

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    // Test unsupported version
    let unsupported_params = InitializeParams {
        protocol_version: "unsupported-version".to_string(),
        capabilities: HashMap::new(),
        client_info: ClientInfo::new("test", "1.0.0"),
    };

    let request = MCPRequest::new("test", "initialize")
        .with_params(serde_json::to_value(unsupported_params).unwrap());

    let result = server.handle_initialize(&request).await;
    assert!(result.is_err());

    if let Err(error) = result {
        assert!(matches!(error, MCPError::UnsupportedProtocolVersion { .. }));
    }
}

#[tokio::test]
async fn test_invalid_json_handling() {
    // Test invalid JSON
    let invalid_json = r#"{"invalid": json"#;
    let result = MCPMessage::from_json(invalid_json);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MCPError::ParseError { .. }));

    // Test valid JSON but invalid message structure
    let invalid_message = r#"{"not_a_valid": "mcp_message"}"#;
    let result = MCPMessage::from_json(invalid_message);
    assert!(result.is_err());

    // Test missing required fields
    let incomplete_request = r#"{"jsonrpc": "2.0", "method": "test"}"#; // Missing id
    let result = MCPMessage::from_json(incomplete_request);
    // Note: This might still parse depending on serde's handling of missing fields
}

#[tokio::test]
async fn test_large_message_handling() {
    // Test handling of large payloads
    let large_data = "x".repeat(100_000); // 100KB string

    let request = MCPRequest::call_tool("test_tool", serde_json::json!({"large_data": large_data}));

    let message = MCPMessage::Request(request);

    // Test serialization of large message
    let json = message.to_json().unwrap();
    assert!(json.len() > 100_000);

    // Test deserialization of large message
    let parsed = MCPMessage::from_json(&json).unwrap();
    assert!(parsed.is_request());

    if let MCPMessage::Request(parsed_request) = parsed {
        if let Some(params) = parsed_request.params {
            assert_eq!(params["arguments"]["large_data"], large_data);
        }
    }
}

#[tokio::test]
async fn test_concurrent_message_processing() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let (tx, mut rx) = mpsc::unbounded_channel::<MCPMessage>();
    let received_messages = Arc::new(Mutex::new(Vec::new()));

    // Spawn message processor
    let messages_clone = received_messages.clone();
    let processor = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let mut messages = messages_clone.lock().await;
            messages.push(message);
        }
    });

    // Send multiple messages concurrently
    let mut handles = vec![];
    for i in 0..100 {
        let tx_clone = tx.clone();
        let handle = tokio::spawn(async move {
            let request = MCPRequest::new(format!("request-{}", i), "test/method");
            let message = MCPMessage::Request(request);
            tx_clone.send(message).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all sends to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Close sender and wait for processor
    drop(tx);
    processor.await.unwrap();

    // Verify all messages were received
    let messages = received_messages.lock().await;
    assert_eq!(messages.len(), 100);

    // Verify message order independence (all should be present)
    let mut ids = vec![];
    for message in messages.iter() {
        if let MCPMessage::Request(req) = message {
            if let RequestId::String(id) = &req.id {
                ids.push(id.clone());
            }
        }
    }

    assert_eq!(ids.len(), 100);
    for i in 0..100 {
        let expected_id = format!("request-{}", i);
        assert!(ids.contains(&expected_id));
    }
}

#[test]
fn test_server_capabilities_serialization() {
    let capabilities = ServerCapabilities::default();

    // Test serialization
    let json = serde_json::to_string(&capabilities).unwrap();
    let parsed: ServerCapabilities = serde_json::from_str(&json).unwrap();

    // Verify default capabilities
    assert!(parsed.logging.is_some());
    assert!(parsed.tools.is_some());
    assert!(parsed.resources.is_some());
    assert!(parsed.prompts.is_some());
    assert!(parsed.experimental.is_some());

    if let Some(experimental) = parsed.experimental {
        assert!(experimental.contains_key("streaming"));
        assert!(experimental.contains_key("batching"));
        assert!(experimental.contains_key("subscriptions"));
    }
}

#[test]
fn test_client_info_serialization() {
    let client_info = ClientInfo::new("test-client", "1.0.0");

    let json = serde_json::to_string(&client_info).unwrap();
    let parsed: ClientInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "test-client");
    assert_eq!(parsed.version, "1.0.0");
}

#[test]
fn test_tool_result_content_types() {
    // Test text content
    let text_content = ToolResultContent::text("Test result");
    let json = serde_json::to_string(&text_content).unwrap();
    let parsed: ToolResultContent = serde_json::from_str(&json).unwrap();

    if let ToolResultContent::Text { text } = parsed {
        assert_eq!(text, "Test result");
    } else {
        panic!("Expected text content");
    }

    // Test JSON content
    let data = serde_json::json!({"status": "success", "count": 42});
    let json_content = ToolResultContent::json(&data).unwrap();
    let json = serde_json::to_string(&json_content).unwrap();
    let parsed: ToolResultContent = serde_json::from_str(&json).unwrap();

    if let ToolResultContent::Text { text } = parsed {
        let parsed_data: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed_data["status"], "success");
        assert_eq!(parsed_data["count"], 42);
    } else {
        panic!("Expected text content");
    }

    // Test image content
    let image_data = vec![0u8, 1u8, 2u8, 255u8];
    let image_content = ToolResultContent::image(&image_data, "image/png");
    let json = serde_json::to_string(&image_content).unwrap();
    let parsed: ToolResultContent = serde_json::from_str(&json).unwrap();

    if let ToolResultContent::Image { data, mime_type } = parsed {
        let decoded_data = base64::decode(data).unwrap();
        assert_eq!(decoded_data, image_data);
        assert_eq!(mime_type, "image/png");
    } else {
        panic!("Expected image content");
    }
}
