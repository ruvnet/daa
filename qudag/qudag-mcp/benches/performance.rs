//! Performance benchmarks for QuDAG MCP implementation

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qudag_mcp::*;
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark protocol message serialization/deserialization
fn bench_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");

    // Setup test data
    let request =
        MCPRequest::initialize(ClientInfo::new("benchmark-client", "1.0.0"), HashMap::new());
    let message = MCPMessage::Request(request);
    let json = message.to_json().unwrap();

    group.bench_function("serialize_request", |b| {
        b.iter(|| {
            let msg = message.clone();
            black_box(msg.to_json().unwrap())
        });
    });

    group.bench_function("deserialize_request", |b| {
        b.iter(|| black_box(MCPMessage::from_json(&json).unwrap()));
    });

    // Benchmark large message handling
    let large_data = "x".repeat(10_000); // 10KB payload
    let large_request =
        MCPRequest::call_tool("test_tool", serde_json::json!({"large_data": large_data}));
    let large_message = MCPMessage::Request(large_request);

    group.bench_function("serialize_large_message", |b| {
        b.iter(|| {
            let msg = large_message.clone();
            black_box(msg.to_json().unwrap())
        });
    });

    group.finish();
}

/// Benchmark tool registry operations
fn bench_tool_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tool_operations");

    let registry = tools::ToolRegistry::new();

    group.bench_function("list_tools", |b| {
        b.iter(|| black_box(registry.list_tools()));
    });

    group.bench_function("tool_lookup", |b| {
        let tool_name = ToolName::new("dag_get_tips");
        b.iter(|| black_box(registry.has_tool(&tool_name)));
    });

    group.bench_function("execute_dag_tool", |b| {
        let tool_name = ToolName::new("dag_get_tips");
        b.to_async(&rt)
            .iter(|| async { black_box(registry.call_tool(&tool_name, None).await.unwrap()) });
    });

    group.bench_function("execute_crypto_tool", |b| {
        let tool_name = ToolName::new("crypto_generate_keypair");
        let args = serde_json::json!({
            "algorithm": "ml-kem",
            "security_level": 3
        });
        b.to_async(&rt).iter(|| async {
            black_box(
                registry
                    .call_tool(&tool_name, Some(args.clone()))
                    .await
                    .unwrap(),
            )
        });
    });

    // Benchmark concurrent tool execution
    group.bench_function("concurrent_tool_execution", |b| {
        let tool_name = ToolName::new("dag_get_tips");
        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];
            for _ in 0..10 {
                let registry = &registry;
                let name = tool_name.clone();
                let handle =
                    tokio::spawn(async move { registry.call_tool(&name, None).await.unwrap() });
                handles.push(handle);
            }

            for handle in handles {
                black_box(handle.await.unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark resource registry operations
fn bench_resource_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("resource_operations");

    let registry = resources::ResourceRegistry::new();

    group.bench_function("list_resources", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(registry.list_resources().await.unwrap()) });
    });

    group.bench_function("read_dag_resource", |b| {
        let uri = ResourceURI::dag("vertices/all");
        b.to_async(&rt)
            .iter(|| async { black_box(registry.read_resource(&uri).await.unwrap()) });
    });

    group.bench_function("read_crypto_resource", |b| {
        let uri = ResourceURI::crypto("algorithms/supported");
        b.to_async(&rt)
            .iter(|| async { black_box(registry.read_resource(&uri).await.unwrap()) });
    });

    group.bench_function("read_vault_resource", |b| {
        let uri = ResourceURI::vault("stats/usage");
        b.to_async(&rt)
            .iter(|| async { black_box(registry.read_resource(&uri).await.unwrap()) });
    });

    group.bench_function("read_network_resource", |b| {
        let uri = ResourceURI::network("peers/connected");
        b.to_async(&rt)
            .iter(|| async { black_box(registry.read_resource(&uri).await.unwrap()) });
    });

    // Benchmark concurrent resource access
    group.bench_function("concurrent_resource_access", |b| {
        let uris = vec![
            ResourceURI::dag("vertices/all"),
            ResourceURI::crypto("algorithms/supported"),
            ResourceURI::vault("stats/usage"),
            ResourceURI::network("peers/connected"),
        ];

        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];
            for uri in &uris {
                let registry = &registry;
                let uri = uri.clone();
                let handle =
                    tokio::spawn(async move { registry.read_resource(&uri).await.unwrap() });
                handles.push(handle);
            }

            for handle in handles {
                black_box(handle.await.unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark server operations
fn bench_server_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("server_operations");

    let config = ServerConfig::new()
        .with_server_info("Benchmark Server", "1.0.0")
        .with_log_level("error"); // Reduce logging for benchmarks

    group.bench_function("server_creation", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(QuDAGMCPServer::new(config.clone()).await.unwrap()) });
    });

    group.bench_function("server_stats", |b| {
        let server = rt.block_on(QuDAGMCPServer::new(config.clone())).unwrap();
        b.to_async(&rt)
            .iter(|| async { black_box(server.stats().await) });
    });

    // Benchmark request handling
    let server = rt.block_on(QuDAGMCPServer::new(config.clone())).unwrap();

    group.bench_function("handle_initialize", |b| {
        let request =
            MCPRequest::initialize(ClientInfo::new("benchmark-client", "1.0.0"), HashMap::new());
        b.to_async(&rt)
            .iter(|| async { black_box(server.handle_initialize(&request).await.unwrap()) });
    });

    group.bench_function("handle_tools_list", |b| {
        let request = MCPRequest::list_tools();
        b.to_async(&rt)
            .iter(|| async { black_box(server.handle_tools_list(&request).await.unwrap()) });
    });

    group.bench_function("handle_resources_list", |b| {
        let request = MCPRequest::list_resources();
        b.to_async(&rt)
            .iter(|| async { black_box(server.handle_resources_list(&request).await.unwrap()) });
    });

    group.finish();
}

/// Benchmark client operations
fn bench_client_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("client_operations");

    let config = ClientConfig::new()
        .with_client_info("Benchmark Client", "1.0.0")
        .with_timeout(Duration::from_secs(1))
        .with_log_level("error"); // Reduce logging for benchmarks

    group.bench_function("client_creation", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(QuDAGMCPClient::new(config.clone()).await.unwrap()) });
    });

    group.bench_function("client_stats", |b| {
        let client = rt.block_on(QuDAGMCPClient::new(config.clone())).unwrap();
        b.to_async(&rt)
            .iter(|| async { black_box(client.stats().await) });
    });

    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Benchmark error creation
    group.bench_function("create_tool_not_found", |b| {
        b.iter(|| black_box(MCPError::tool_not_found("nonexistent_tool")));
    });

    group.bench_function("create_resource_not_found", |b| {
        b.iter(|| black_box(MCPError::resource_not_found("nonexistent://resource")));
    });

    // Benchmark error serialization
    let error = MCPError::ToolNotFound {
        name: "test_tool".to_string(),
    };
    group.bench_function("serialize_error", |b| {
        b.iter(|| black_box(error.to_json_rpc_error()));
    });

    // Benchmark error response creation
    group.bench_function("create_error_response", |b| {
        let id = RequestId::generate();
        let error = MCPError::MethodNotFound {
            method: "test/method".to_string(),
        };
        b.iter(|| black_box(MCPResponse::error(id.clone(), error.clone())));
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_operations");

    let tool_registry = tools::ToolRegistry::new();
    let resource_registry = resources::ResourceRegistry::new();

    // Benchmark varying levels of concurrency
    for concurrency in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_tool_calls", concurrency),
            concurrency,
            |b, &concurrency| {
                let tool_name = ToolName::new("dag_get_tips");
                b.to_async(&rt).iter(|| async {
                    let mut handles = vec![];
                    for _ in 0..concurrency {
                        let registry = &tool_registry;
                        let name = tool_name.clone();
                        let handle =
                            tokio::spawn(
                                async move { registry.call_tool(&name, None).await.unwrap() },
                            );
                        handles.push(handle);
                    }

                    for handle in handles {
                        black_box(handle.await.unwrap());
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("concurrent_resource_reads", concurrency),
            concurrency,
            |b, &concurrency| {
                let uri = ResourceURI::dag("vertices/all");
                b.to_async(&rt).iter(|| async {
                    let mut handles = vec![];
                    for _ in 0..concurrency {
                        let registry = &resource_registry;
                        let uri = uri.clone();
                        let handle =
                            tokio::spawn(
                                async move { registry.read_resource(&uri).await.unwrap() },
                            );
                        handles.push(handle);
                    }

                    for handle in handles {
                        black_box(handle.await.unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_operations");

    // Benchmark creating many instances
    group.bench_function("create_many_servers", |b| {
        let config = ServerConfig::new().with_log_level("error");
        b.to_async(&rt).iter(|| async {
            let mut servers = vec![];
            for _ in 0..100 {
                servers.push(QuDAGMCPServer::new(config.clone()).await.unwrap());
            }
            black_box(servers)
        });
    });

    group.bench_function("create_many_clients", |b| {
        let config = ClientConfig::new().with_log_level("error");
        b.to_async(&rt).iter(|| async {
            let mut clients = vec![];
            for _ in 0..100 {
                clients.push(QuDAGMCPClient::new(config.clone()).await.unwrap());
            }
            black_box(clients)
        });
    });

    // Benchmark large data handling
    let large_payload_sizes = [1_000, 10_000, 100_000]; // 1KB, 10KB, 100KB

    for size in large_payload_sizes.iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("handle_large_payload", size),
            size,
            |b, &size| {
                let large_data = "x".repeat(size);
                let args = serde_json::json!({
                    "id": "large_vertex",
                    "payload": large_data,
                    "parents": []
                });
                let tool_registry = tools::ToolRegistry::new();
                let tool_name = ToolName::new("dag_add_vertex");

                b.to_async(&rt).iter(|| async {
                    black_box(
                        tool_registry
                            .call_tool(&tool_name, Some(args.clone()))
                            .await
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

/// Benchmark protocol compliance operations
fn bench_protocol_compliance(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_compliance");

    // Benchmark request ID generation
    group.bench_function("generate_request_ids", |b| {
        b.iter(|| black_box(RequestId::generate()));
    });

    // Benchmark protocol version checking
    group.bench_function("version_check", |b| {
        let supported_version = crate::MCP_PROTOCOL_VERSION;
        b.iter(|| black_box(supported_version == "2025-03-26"));
    });

    // Benchmark capability matching
    group.bench_function("capability_matching", |b| {
        let server_caps = ServerCapabilities::default();
        let client_caps = HashMap::<String, serde_json::Value>::new();

        b.iter(|| {
            // Simulate capability negotiation
            let _logging_supported =
                server_caps.logging.is_some() && client_caps.get("logging").is_none();
            black_box(_logging_supported)
        });
    });

    group.finish();
}

/// Benchmark real-world usage scenarios
fn bench_usage_scenarios(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("usage_scenarios");

    // Scenario: Full client-server interaction
    group.bench_function("full_interaction_simulation", |b| {
        let server_config = ServerConfig::new().with_log_level("error");
        let client_config = ClientConfig::new().with_log_level("error");

        b.to_async(&rt).iter(|| async {
            // Create server and client
            let server = QuDAGMCPServer::new(server_config.clone()).await.unwrap();
            let client = QuDAGMCPClient::new(client_config.clone()).await.unwrap();

            // Simulate initialization
            let init_request = MCPRequest::initialize(
                client.config.client_info.clone(),
                client.config.capabilities.clone(),
            );
            let _init_response = server.handle_initialize(&init_request).await.unwrap();

            // Simulate tool discovery
            let tools_request = MCPRequest::list_tools();
            let _tools_response = server.handle_tools_list(&tools_request).await.unwrap();

            // Simulate resource access
            let resources_request = MCPRequest::list_resources();
            let _resources_response = server
                .handle_resources_list(&resources_request)
                .await
                .unwrap();

            black_box((server, client))
        });
    });

    // Scenario: Heavy tool usage
    group.bench_function("heavy_tool_usage", |b| {
        let tool_registry = tools::ToolRegistry::new();

        b.to_async(&rt).iter(|| async {
            let tools = [
                "dag_get_tips",
                "dag_get_order",
                "crypto_generate_keypair",
                "dag_add_vertex",
                "crypto_sign",
            ];

            for tool_name in &tools {
                let name = ToolName::new(tool_name);
                let args = match *tool_name {
                    "dag_add_vertex" => Some(serde_json::json!({
                        "id": "test_vertex",
                        "payload": "test payload",
                        "parents": []
                    })),
                    "crypto_generate_keypair" => Some(serde_json::json!({
                        "algorithm": "ml-kem",
                        "security_level": 3
                    })),
                    "crypto_sign" => Some(serde_json::json!({
                        "data": "dGVzdCBkYXRh", // "test data" in base64
                        "private_key": "mock_key"
                    })),
                    _ => None,
                };

                let _result = tool_registry.call_tool(&name, args).await.unwrap();
            }

            black_box(())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_message_serialization,
    bench_tool_operations,
    bench_resource_operations,
    bench_server_operations,
    bench_client_operations,
    bench_error_handling,
    bench_concurrent_operations,
    bench_memory_operations,
    bench_protocol_compliance,
    bench_usage_scenarios
);

criterion_main!(benches);
