//! Basic QuDAG MCP Server example
//!
//! This example demonstrates how to create and run a basic MCP server
//! that exposes QuDAG's DAG, crypto, network, and vault capabilities.

use qudag_mcp::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting QuDAG MCP Server Example");

    // Create server configuration
    let config = ServerConfig::new()
        .with_server_info("QuDAG MCP Demo Server", "1.0.0")
        .with_transport(transport::TransportFactory::stdio())
        .with_log_level("info");

    println!("📋 Server Configuration:");
    println!("  - Name: {}", config.server_info.name);
    println!("  - Version: {}", config.server_info.version);
    println!("  - Transport: stdio");
    println!("  - Log Level: {}", config.log_level);

    // Create the MCP server
    let mut server = QuDAGMCPServer::new(config).await?;

    println!("\n✅ Server created successfully!");

    // Display server capabilities
    let stats = server.stats().await;
    println!("\n📊 Server Statistics:");
    println!("  - Available Tools: {}", stats.tools_count);
    println!("  - Available Resources: {}", stats.resources_count);
    println!("  - Active Subscriptions: {}", stats.active_subscriptions);
    println!("  - Client Connected: {}", stats.client_connected);

    // List available tools
    println!("\n🔧 Available Tools:");
    let tool_registry = tools::ToolRegistry::new();
    let tools = tool_registry.list_tools();
    for tool in tools {
        println!("  - {}: {}", tool.name.as_str(), tool.description);
    }

    // List available resources
    println!("\n📦 Available Resources:");
    let resource_registry = resources::ResourceRegistry::new();
    let resources = resource_registry.list_resources().await?;
    for resource in resources {
        let description = resource.description.as_deref().unwrap_or("No description");
        println!("  - {}: {}", resource.uri, description);
    }

    // Demonstrate tool execution
    println!("\n🎯 Demonstrating Tool Execution:");

    // Execute DAG tips tool
    let tips_result = tool_registry
        .call_tool(&ToolName::new("dag_get_tips"), None)
        .await?;

    if let ToolResultContent::Text { text } = &tips_result.content[0] {
        let tips_data: serde_json::Value = serde_json::from_str(text)?;
        println!(
            "  DAG Tips: {} tips found",
            tips_data["count"].as_u64().unwrap_or(0)
        );
    }

    // Execute crypto key generation
    let keygen_args = serde_json::json!({
        "algorithm": "ml-kem",
        "security_level": 3
    });

    let keygen_result = tool_registry
        .call_tool(&ToolName::new("crypto_generate_keypair"), Some(keygen_args))
        .await?;

    if let ToolResultContent::Text { text } = &keygen_result.content[0] {
        let key_data: serde_json::Value = serde_json::from_str(text)?;
        println!(
            "  Crypto Keygen: {} key generated",
            key_data["algorithm"].as_str().unwrap_or("unknown")
        );
    }

    // Demonstrate resource access
    println!("\n📊 Demonstrating Resource Access:");

    // Read DAG statistics
    let dag_stats_uri = ResourceURI::dag("stats/summary");
    let dag_stats = resource_registry.read_resource(&dag_stats_uri).await?;

    if let Some(text) = &dag_stats[0].text {
        let stats_data: serde_json::Value = serde_json::from_str(text)?;
        println!(
            "  DAG Stats: {} total vertices",
            stats_data["total_vertices"].as_u64().unwrap_or(0)
        );
    }

    // Read crypto algorithms
    let crypto_uri = ResourceURI::crypto("algorithms/supported");
    let crypto_data = resource_registry.read_resource(&crypto_uri).await?;

    if let Some(text) = &crypto_data[0].text {
        let crypto_info: serde_json::Value = serde_json::from_str(text)?;
        if let Some(algorithms) = crypto_info["algorithms"].as_array() {
            println!(
                "  Crypto Algorithms: {} quantum-resistant algorithms supported",
                algorithms.len()
            );
        }
    }

    println!("\n🎉 Demo completed successfully!");
    println!("\n📝 To connect a client to this server:");
    println!("  1. The server is configured for stdio transport");
    println!("  2. Send JSON-RPC 2.0 messages over stdin/stdout");
    println!("  3. Start with an 'initialize' request");
    println!("  4. Use 'tools/list' and 'resources/list' to explore capabilities");

    // Interactive mode option
    println!("\n❓ Would you like to run the server in interactive mode? (y/n)");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        println!("\n🔄 Starting interactive server mode...");
        println!("📬 Send JSON-RPC messages to interact with the server");
        println!("🛑 Press Ctrl+C to stop the server");

        // Run the server
        server.run().await?;
    } else {
        println!("👋 Exiting demo. Server not started in interactive mode.");
    }

    Ok(())
}

/// Helper function to print JSON in a readable format
#[allow(dead_code)]
fn print_json(label: &str, value: &serde_json::Value) {
    println!("{}:", label);
    if let Ok(pretty) = serde_json::to_string_pretty(value) {
        for line in pretty.lines() {
            println!("  {}", line);
        }
    } else {
        println!("  {}", value);
    }
}
