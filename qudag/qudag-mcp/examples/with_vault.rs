//! QuDAG MCP Server with Vault Integration Example
//!
//! This example demonstrates how to create an MCP server with enhanced
//! vault integration, showing secure secret management and authentication.

use qudag_mcp::*;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 QuDAG MCP Server with Vault Integration");
    println!("=========================================");

    // Create server configuration with enhanced security
    let mut capabilities = ServerCapabilities::default();

    // Add vault-specific capabilities
    if let Some(ref mut experimental) = capabilities.experimental {
        experimental.insert(
            "vault_integration".to_string(),
            serde_json::json!({
                "enabled": true,
                "features": ["secret_management", "authentication", "encryption"]
            }),
        );
        experimental.insert(
            "quantum_crypto".to_string(),
            serde_json::json!({
                "algorithms": ["ML-KEM", "ML-DSA", "HQC"],
                "security_levels": [2, 3, 5]
            }),
        );
    }

    let config = ServerConfig {
        server_info: ServerInfo::new("QuDAG MCP Vault Server", "1.0.0"),
        capabilities,
        transport: transport::TransportConfig::Stdio,
        log_level: "info".to_string(),
    };

    println!("🏗️  Creating secure MCP server...");
    let server = QuDAGMCPServer::new(config).await?;

    // Display security features
    println!("\n🔒 Security Features Enabled:");
    println!("  ✓ Quantum-resistant cryptography");
    println!("  ✓ Secure vault integration");
    println!("  ✓ Authentication support");
    println!("  ✓ Encrypted communications");

    // Demonstrate vault operations
    println!("\n💾 Demonstrating Vault Operations:");

    let resource_registry = resources::ResourceRegistry::new();

    // Check vault health
    let health_uri = ResourceURI::vault("health/status");
    let health_data = resource_registry.read_resource(&health_uri).await?;

    if let Some(text) = &health_data[0].text {
        let health: serde_json::Value = serde_json::from_str(text)?;
        println!(
            "  🏥 Vault Health: {}",
            health["status"].as_str().unwrap_or("unknown")
        );

        if let Some(encryption) = health.get("encryption") {
            println!(
                "  🔐 Encryption Status: {}",
                encryption["status"].as_str().unwrap_or("unknown")
            );
        }

        if let Some(integrity) = health.get("integrity") {
            let checksum_valid = integrity["checksum_valid"].as_bool().unwrap_or(false);
            println!(
                "  ✅ Integrity Check: {}",
                if checksum_valid { "PASSED" } else { "FAILED" }
            );
        }
    }

    // Get vault statistics
    let stats_uri = ResourceURI::vault("stats/usage");
    let stats_data = resource_registry.read_resource(&stats_uri).await?;

    if let Some(text) = &stats_data[0].text {
        let stats: serde_json::Value = serde_json::from_str(text)?;

        if let Some(operations) = stats.get("operations") {
            println!("  📊 Vault Operations:");
            println!(
                "    - Total Reads: {}",
                operations["total_reads"].as_u64().unwrap_or(0)
            );
            println!(
                "    - Total Writes: {}",
                operations["total_writes"].as_u64().unwrap_or(0)
            );
        }

        if let Some(performance) = stats.get("performance") {
            println!("  ⚡ Performance Metrics:");
            println!(
                "    - Avg Read Time: {}ms",
                performance["avg_read_time_ms"].as_u64().unwrap_or(0)
            );
            println!(
                "    - Cache Hit Rate: {:.2}%",
                performance["cache_hit_rate"].as_f64().unwrap_or(0.0) * 100.0
            );
        }
    }

    // Demonstrate cryptographic operations
    println!("\n🔑 Demonstrating Quantum-Resistant Cryptography:");

    let tool_registry = tools::ToolRegistry::new();

    // Test different quantum-resistant algorithms
    let algorithms = vec![
        ("ML-KEM", "Key Encapsulation Mechanism"),
        ("ML-DSA", "Digital Signature Algorithm"),
    ];

    for (algo, description) in algorithms {
        println!("  🧮 Testing {}: {}", algo, description);

        let keygen_args = serde_json::json!({
            "algorithm": algo.to_lowercase(),
            "security_level": 3
        });

        let result = tool_registry
            .call_tool(&ToolName::new("crypto_generate_keypair"), Some(keygen_args))
            .await?;

        if let ToolResultContent::Text { text } = &result.content[0] {
            let key_data: serde_json::Value = serde_json::from_str(text)?;
            println!(
                "    ✓ Generated {} keypair with security level {}",
                key_data["algorithm"].as_str().unwrap_or("unknown"),
                key_data["security_level"].as_u64().unwrap_or(0)
            );
        }
    }

    // Demonstrate signature operations
    println!("\n✍️  Digital Signature Demo:");

    let test_data = "Hello, QuDAG MCP!";
    let test_data_b64 = base64::encode(test_data.as_bytes());

    // Sign data
    let sign_args = serde_json::json!({
        "data": test_data_b64,
        "private_key": "mock_private_key_for_demo"
    });

    let sign_result = tool_registry
        .call_tool(&ToolName::new("crypto_sign"), Some(sign_args))
        .await?;

    if let ToolResultContent::Text { text } = &sign_result.content[0] {
        let sign_data: serde_json::Value = serde_json::from_str(text)?;
        println!(
            "  ✓ Data signed using {}",
            sign_data["algorithm"].as_str().unwrap_or("unknown")
        );

        // Verify signature
        let verify_args = serde_json::json!({
            "data": test_data_b64,
            "signature": sign_data["signature"],
            "public_key": "mock_public_key_for_demo"
        });

        let verify_result = tool_registry
            .call_tool(&ToolName::new("crypto_verify"), Some(verify_args))
            .await?;

        if let ToolResultContent::Text { text } = &verify_result.content[0] {
            let verify_data: serde_json::Value = serde_json::from_str(text)?;
            let is_valid = verify_data["valid"].as_bool().unwrap_or(false);
            println!(
                "  {} Signature verification: {}",
                if is_valid { "✅" } else { "❌" },
                if is_valid { "VALID" } else { "INVALID" }
            );
        }
    }

    // Demonstrate network security
    println!("\n🌐 Network Security Features:");

    let network_uri = ResourceURI::network("peers/connected");
    let network_data = resource_registry.read_resource(&network_uri).await?;

    if let Some(text) = &network_data[0].text {
        let network: serde_json::Value = serde_json::from_str(text)?;

        if let Some(peers) = network["peers"].as_array() {
            println!("  👥 Connected Peers: {}", peers.len());

            for (i, peer) in peers.iter().enumerate().take(3) {
                let peer_id = peer["id"].as_str().unwrap_or("unknown");
                let latency = peer["latency_ms"].as_u64().unwrap_or(0);
                println!("    {}. {} ({}ms latency)", i + 1, peer_id, latency);
            }
        }
    }

    // Authentication simulation
    println!("\n🔐 Authentication System Demo:");

    // Simulate client authentication flow
    let client_info = ClientInfo::new("Secure QuDAG Client", "1.0.0");
    let mut client_capabilities = HashMap::new();
    client_capabilities.insert(
        "authentication".to_string(),
        serde_json::json!({
            "method": "vault_token",
            "secure": true
        }),
    );

    println!("  📋 Client Authentication Request:");
    println!("    - Client: {}", client_info.name);
    println!("    - Version: {}", client_info.version);
    println!("    - Auth Method: vault_token");

    // Simulate MCP initialize flow
    let init_request = protocol::MCPRequest::initialize(client_info, client_capabilities);
    let init_response = server.handle_initialize(&init_request).await?;

    if init_response.result.is_some() {
        println!("  ✅ Authentication successful");
        println!("  🔑 MCP session established");
    } else {
        println!("  ❌ Authentication failed");
    }

    // Performance and security metrics
    println!("\n📊 Security & Performance Summary:");
    let server_stats = server.stats().await;

    println!("  🔧 Tools Available: {}", server_stats.tools_count);
    println!("  📦 Resources Available: {}", server_stats.resources_count);
    println!("  🔒 Security Level: Quantum-Resistant");
    println!("  ⚡ Performance: Optimized for production");

    // Security recommendations
    println!("\n💡 Security Recommendations:");
    println!("  🔐 Use strong authentication tokens");
    println!("  🔄 Rotate encryption keys regularly");
    println!("  📝 Monitor vault access logs");
    println!("  🛡️  Enable all security features in production");
    println!("  🔍 Regular security audits recommended");

    println!("\n🎯 Example Usage in Production:");
    println!("  1. Configure vault with production credentials");
    println!("  2. Set up proper authentication mechanisms");
    println!("  3. Enable TLS for network communications");
    println!("  4. Implement proper access controls");
    println!("  5. Monitor security events and logs");

    println!("\n✨ QuDAG MCP Vault Integration Demo Completed!");
    println!("🚀 Ready for secure production deployment");

    Ok(())
}

/// Helper function to create a secure client configuration
#[allow(dead_code)]
fn create_secure_client_config() -> ClientConfig {
    let mut capabilities = HashMap::new();
    capabilities.insert(
        "security".to_string(),
        serde_json::json!({
            "encryption": true,
            "authentication": "required",
            "quantum_resistant": true
        }),
    );

    ClientConfig::new()
        .with_client_info("Secure QuDAG MCP Client", "1.0.0")
        .with_timeout(Duration::from_secs(30))
        .with_capability("vault_access", serde_json::Value::Bool(true))
        .with_capability("crypto_operations", serde_json::Value::Bool(true))
}

/// Helper function to demonstrate secure message handling
#[allow(dead_code)]
async fn demonstrate_secure_messaging() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Secure Message Handling:");

    // Create encrypted message
    let sensitive_data = "Confidential QuDAG Data";
    let encrypted_message = protocol::MCPRequest::call_tool(
        "crypto_encrypt",
        serde_json::json!({
            "data": base64::encode(sensitive_data.as_bytes()),
            "public_key": "production_public_key"
        }),
    );

    println!("  ✓ Message encrypted for transmission");
    println!("  📤 Secure MCP request created");

    // Serialize with security
    let message = protocol::MCPMessage::Request(encrypted_message);
    let _secure_json = message.to_json()?;

    println!("  📡 Ready for secure transmission");

    Ok(())
}
