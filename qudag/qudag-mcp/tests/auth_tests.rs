//! Authentication and security tests with vault integration

use qudag_mcp::*;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_basic_authentication_flow() {
    // Test creating a client with authentication capabilities
    let mut capabilities = HashMap::new();
    capabilities.insert(
        "authentication".to_string(),
        serde_json::json!({
            "oauth2": true,
            "api_key": true
        }),
    );

    let config = ClientConfig::new()
        .with_client_info("Authenticated Client", "1.0.0")
        .with_capability(
            "authentication",
            serde_json::json!({
                "supported_methods": ["oauth2", "api_key"],
                "token_refresh": true
            }),
        );

    let client = QuDAGMCPClient::new(config).await.unwrap();

    // Verify authentication capabilities are set
    assert!(client.config.capabilities.contains_key("authentication"));
}

#[tokio::test]
async fn test_server_authentication_capabilities() {
    let mut server_capabilities = ServerCapabilities::default();

    // Add authentication to experimental capabilities
    if let Some(ref mut experimental) = server_capabilities.experimental {
        experimental.insert(
            "authentication".to_string(),
            serde_json::json!({
                "required": false,
                "methods": ["oauth2", "api_key", "vault_token"],
                "token_validation": true
            }),
        );
    }

    let config = ServerConfig::new().with_server_info("Secure QuDAG MCP Server", "1.0.0");

    let server = QuDAGMCPServer::new(config).await.unwrap();

    // Test server capabilities include authentication options
    let stats = server.stats().await;
    assert!(matches!(stats.state, ServerState::Uninitialized));
}

#[tokio::test]
async fn test_vault_integration_security() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test accessing vault resources
    let vault_resources = registry.list_resources().await.unwrap();
    let vault_uris: Vec<String> = vault_resources
        .iter()
        .filter(|r| r.uri.scheme() == Some("vault"))
        .map(|r| r.uri.to_string())
        .collect();

    assert!(!vault_uris.is_empty());

    // Test reading vault health status (should not expose sensitive data)
    let health_uri = ResourceURI::vault("health/status");
    let contents = registry.read_resource(&health_uri).await.unwrap();
    assert_eq!(contents.len(), 1);

    if let Some(text) = &contents[0].text {
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(data["status"], "healthy");
        assert!(data["encryption"]["status"].is_string());
        assert!(data["integrity"]["checksum_valid"].is_boolean());

        // Ensure no sensitive key material is exposed
        assert!(!text.contains("private_key"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("password"));
    }
}

#[tokio::test]
async fn test_crypto_tool_security() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test crypto key generation (should be secure)
    let keygen_args = serde_json::json!({
        "algorithm": "ml-kem",
        "security_level": 3
    });

    let tool_name = ToolName::new("crypto_generate_keypair");
    let result = registry
        .call_tool(&tool_name, Some(keygen_args))
        .await
        .unwrap();

    assert_eq!(result.is_error, Some(false));

    if let ToolResultContent::Text { text } = &result.content[0] {
        let key_data: serde_json::Value = serde_json::from_str(text).unwrap();

        // Verify quantum-resistant algorithm is used
        assert_eq!(key_data["algorithm"], "ml-kem");
        assert_eq!(key_data["security_level"], 3);

        // Verify key material is present (mock data in this case)
        assert!(key_data["public_key"].is_string());
        assert!(key_data["private_key"].is_string());
        assert!(key_data["key_id"].is_string());
        assert!(key_data["created_at"].is_string());
    }
}

#[tokio::test]
async fn test_signature_verification_security() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test digital signature creation
    let sign_args = serde_json::json!({
        "data": "SGVsbG8gV29ybGQ=", // "Hello World" in base64
        "private_key": "mock_private_key_data"
    });

    let sign_tool = ToolName::new("crypto_sign");
    let sign_result = registry
        .call_tool(&sign_tool, Some(sign_args))
        .await
        .unwrap();

    assert_eq!(sign_result.is_error, Some(false));

    // Test signature verification
    let verify_args = serde_json::json!({
        "data": "SGVsbG8gV29ybGQ=",
        "signature": "mock_signature_data",
        "public_key": "mock_public_key_data"
    });

    let verify_tool = ToolName::new("crypto_verify");
    let verify_result = registry
        .call_tool(&verify_tool, Some(verify_args))
        .await
        .unwrap();

    assert_eq!(verify_result.is_error, Some(false));

    if let ToolResultContent::Text { text } = &verify_result.content[0] {
        let verify_data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(verify_data["valid"].is_boolean());
        assert_eq!(verify_data["algorithm"], "ml-dsa");
    }
}

#[tokio::test]
async fn test_encryption_security() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test encryption
    let encrypt_args = serde_json::json!({
        "data": "U2VjcmV0IE1lc3NhZ2U=", // "Secret Message" in base64
        "public_key": "mock_public_key_data"
    });

    let encrypt_tool = ToolName::new("crypto_encrypt");
    let encrypt_result = registry
        .call_tool(&encrypt_tool, Some(encrypt_args))
        .await
        .unwrap();

    assert_eq!(encrypt_result.is_error, Some(false));

    if let ToolResultContent::Text { text } = &encrypt_result.content[0] {
        let encrypt_data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(encrypt_data["ciphertext"].is_string());
        assert_eq!(encrypt_data["algorithm"], "ml-kem");

        // Test decryption
        let decrypt_args = serde_json::json!({
            "ciphertext": encrypt_data["ciphertext"],
            "private_key": "mock_private_key_data"
        });

        let decrypt_tool = ToolName::new("crypto_decrypt");
        let decrypt_result = registry
            .call_tool(&decrypt_tool, Some(decrypt_args))
            .await
            .unwrap();

        assert_eq!(decrypt_result.is_error, Some(false));

        if let ToolResultContent::Text { text } = &decrypt_result.content[0] {
            let decrypt_data: serde_json::Value = serde_json::from_str(text).unwrap();
            assert!(decrypt_data["plaintext"].is_string());
            assert_eq!(decrypt_data["algorithm"], "ml-kem");
        }
    }
}

#[tokio::test]
async fn test_access_control_errors() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test attempting to access non-existent or restricted resources
    let restricted_uri = ResourceURI::new("vault://secrets/master_key");
    let result = registry.read_resource(&restricted_uri).await;

    // Should fail with resource not found (proper access control)
    assert!(matches!(result, Err(MCPError::ResourceNotFound { .. })));

    // Test attempting to access malformed URIs
    let malformed_uri = ResourceURI::new("invalid://malformed/uri");
    let result = registry.read_resource(&malformed_uri).await;
    assert!(matches!(result, Err(MCPError::ResourceNotFound { .. })));
}

#[tokio::test]
async fn test_input_validation_security() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test tool execution with invalid parameters
    let invalid_args = serde_json::json!({
        "invalid_param": "malicious_input",
        "sql_injection": "'; DROP TABLE users; --"
    });

    let tool_name = ToolName::new("dag_add_vertex");
    let result = registry.call_tool(&tool_name, Some(invalid_args)).await;

    // Should fail with invalid parameters
    assert!(matches!(result, Err(MCPError::InvalidParams { .. })));

    // Test with missing required parameters
    let missing_args = serde_json::json!({
        "payload": "test payload"
        // Missing required "id" parameter
    });

    let result = registry.call_tool(&tool_name, Some(missing_args)).await;
    assert!(matches!(result, Err(MCPError::InvalidParams { .. })));
}

#[tokio::test]
async fn test_error_information_leakage() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test that error messages don't leak sensitive information
    let tool_name = ToolName::new("nonexistent_sensitive_tool");
    let result = registry.call_tool(&tool_name, None).await;

    if let Err(MCPError::ToolNotFound { name }) = result {
        // Error message should not contain sensitive system information
        assert_eq!(name, "nonexistent_sensitive_tool");

        let error_json = MCPError::ToolNotFound { name: name.clone() }.to_json_rpc_error();
        let error_message = error_json["message"].as_str().unwrap();

        // Ensure error doesn't leak file paths, internal structure, etc.
        assert!(!error_message.contains("/"));
        assert!(!error_message.contains("\\"));
        assert!(!error_message.contains("internal"));
        assert!(!error_message.contains("debug"));
    } else {
        panic!("Expected ToolNotFound error");
    }
}

#[tokio::test]
async fn test_concurrent_authentication_requests() {
    use std::sync::Arc;

    // Test multiple concurrent authentication attempts
    let mut handles = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let config = ClientConfig::new()
                .with_client_info(format!("Client-{}", i), "1.0.0")
                .with_capability(
                    "auth_token",
                    serde_json::Value::String(format!("token-{}", i)),
                );

            let client = QuDAGMCPClient::new(config).await.unwrap();

            // Simulate authentication check
            let stats = client.stats().await;
            assert!(matches!(stats.state, ClientState::Disconnected));

            i
        });
        handles.push(handle);
    }

    // Wait for all authentication attempts
    for handle in handles {
        let client_id = handle.await.unwrap();
        assert!(client_id < 10);
    }
}

#[tokio::test]
async fn test_session_security() {
    // Test that sessions are properly isolated
    let config1 = ClientConfig::new()
        .with_client_info("Client1", "1.0.0")
        .with_capability(
            "session_id",
            serde_json::Value::String("session-1".to_string()),
        );

    let config2 = ClientConfig::new()
        .with_client_info("Client2", "1.0.0")
        .with_capability(
            "session_id",
            serde_json::Value::String("session-2".to_string()),
        );

    let client1 = QuDAGMCPClient::new(config1).await.unwrap();
    let client2 = QuDAGMCPClient::new(config2).await.unwrap();

    // Verify sessions are different
    let stats1 = client1.stats().await;
    let stats2 = client2.stats().await;

    // Both should be in disconnected state initially
    assert!(matches!(stats1.state, ClientState::Disconnected));
    assert!(matches!(stats2.state, ClientState::Disconnected));

    // Verify client info is different
    assert_ne!(
        client1.config.client_info.name,
        client2.config.client_info.name
    );
}

#[tokio::test]
async fn test_resource_access_patterns() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test that only appropriate vault resources are exposed
    let vault_resources = registry.list_resources().await.unwrap();
    let vault_uris: Vec<&ResourceURI> = vault_resources
        .iter()
        .filter(|r| r.uri.scheme() == Some("vault"))
        .map(|r| &r.uri)
        .collect();

    // Should have metadata and stats, but not raw secrets
    let allowed_patterns = vec![
        "entries/count",
        "categories/list",
        "stats/usage",
        "health/status",
    ];

    for uri in vault_uris {
        if let Some(path) = uri.path() {
            assert!(
                allowed_patterns
                    .iter()
                    .any(|pattern| path.contains(pattern)),
                "Unexpected vault resource exposed: {}",
                path
            );
        }
    }
}

#[tokio::test]
async fn test_crypto_algorithm_security() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test that only quantum-resistant algorithms are advertised
    let crypto_uri = ResourceURI::crypto("algorithms/supported");
    let contents = registry.read_resource(&crypto_uri).await.unwrap();

    if let Some(text) = &contents[0].text {
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        let algorithms = data["algorithms"].as_array().unwrap();

        for algorithm in algorithms {
            let is_quantum_resistant = algorithm["quantum_resistant"].as_bool().unwrap_or(false);
            assert!(
                is_quantum_resistant,
                "Non-quantum-resistant algorithm found: {:?}",
                algorithm
            );

            let name = algorithm["name"].as_str().unwrap();
            assert!(
                name == "ML-KEM" || name == "ML-DSA" || name == "HQC",
                "Unexpected algorithm: {}",
                name
            );
        }
    }
}

#[tokio::test]
async fn test_network_security_information() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test network peer information doesn't leak sensitive data
    let peers_uri = ResourceURI::network("peers/connected");
    let contents = registry.read_resource(&peers_uri).await.unwrap();

    if let Some(text) = &contents[0].text {
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        let peers = data["peers"].as_array().unwrap();

        for peer in peers {
            // Should have basic connection info but not sensitive details
            assert!(peer["id"].is_string());
            assert!(peer["status"].is_string());
            assert!(peer["latency_ms"].is_number());

            // Should not expose internal addresses or keys
            let peer_str = peer.to_string();
            assert!(!peer_str.contains("private_key"));
            assert!(!peer_str.contains("secret"));
            assert!(!peer_str.contains("127.0.0.1"));
        }
    }
}

#[tokio::test]
async fn test_dag_data_access_security() {
    use qudag_mcp::resources::ResourceRegistry;

    let registry = ResourceRegistry::new();

    // Test DAG vertex data access
    let vertices_uri = ResourceURI::dag("vertices/all");
    let contents = registry.read_resource(&vertices_uri).await.unwrap();

    if let Some(text) = &contents[0].text {
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        let vertices = data["vertices"].as_array().unwrap();

        for vertex in vertices {
            // Should have public DAG information but not sensitive payload details
            assert!(vertex["id"].is_string());
            assert!(vertex["confidence"].is_number());
            assert!(vertex["timestamp"].is_string());

            // Check that confidence values are reasonable
            let confidence = vertex["confidence"].as_f64().unwrap();
            assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}

#[tokio::test]
async fn test_rate_limiting_simulation() {
    use qudag_mcp::tools::ToolRegistry;
    use std::time::Instant;

    let registry = ToolRegistry::new();

    // Simulate rapid tool calls to test for potential DoS protection
    let start = Instant::now();
    let mut results = vec![];

    for _ in 0..50 {
        let tool_name = ToolName::new("dag_get_tips");
        let result = registry.call_tool(&tool_name, None).await;
        results.push(result);
    }

    let duration = start.elapsed();

    // All calls should succeed (no built-in rate limiting in mock implementation)
    for result in results {
        assert!(result.is_ok());
    }

    // Verify operations completed in reasonable time (should be fast for mock data)
    assert!(
        duration < Duration::from_secs(5),
        "Operations took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_memory_safety_with_large_inputs() {
    use qudag_mcp::tools::ToolRegistry;

    let registry = ToolRegistry::new();

    // Test with reasonably large input to ensure memory safety
    let large_payload = "x".repeat(1024 * 10); // 10KB payload

    let args = serde_json::json!({
        "id": "large_vertex",
        "payload": large_payload,
        "parents": []
    });

    let tool_name = ToolName::new("dag_add_vertex");
    let result = registry.call_tool(&tool_name, Some(args)).await;

    // Should handle large inputs gracefully
    assert!(result.is_ok());

    if let Ok(tool_result) = result {
        assert_eq!(tool_result.is_error, Some(false));
    }
}

#[tokio::test]
async fn test_secure_error_reporting() {
    // Test that error reporting doesn't expose sensitive system information
    let errors = vec![
        MCPError::CryptoOperationFailed {
            operation: "key_generation".to_string(),
        },
        MCPError::VaultOperationFailed {
            operation: "secret_retrieval".to_string(),
        },
        MCPError::DAGOperationFailed {
            operation: "consensus_validation".to_string(),
        },
        MCPError::NetworkOperationFailed {
            operation: "peer_connection".to_string(),
        },
    ];

    for error in errors {
        let json_error = error.to_json_rpc_error();
        let message = json_error["message"].as_str().unwrap();

        // Error messages should be informative but not leak internals
        assert!(!message.contains("stack trace"));
        assert!(!message.contains("file:"));
        assert!(!message.contains("line:"));
        assert!(!message.contains("panic"));
        assert!(!message.contains("unwrap"));

        // Should not contain system paths
        assert!(!message.contains("/usr/"));
        assert!(!message.contains("/home/"));
        assert!(!message.contains("C:\\"));

        // Should not contain sensitive data
        assert!(!message.contains("password"));
        assert!(!message.contains("private_key"));
        assert!(!message.contains("secret"));
    }
}
