// QuDAG Node with Integrated MCP Server
// Supports streamable HTTP MCP endpoints with SSE and WebSocket

use std::env;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use serde_json::json;
use std::collections::HashMap;

// MCP-specific imports (simulated for testnet)
use std::convert::Infallible;

#[derive(Clone, Debug)]
struct PeerInfo {
    id: String,
    address: String,
    last_seen: Instant,
    connection_time: Instant,
    message_count: u64,
}

struct NetworkState {
    node_id: String,
    connected_peers: HashMap<String, PeerInfo>,
    bootstrap_peers: Vec<String>,
    message_count: u64,
    bytes_sent: u64,
    bytes_received: u64,
}

struct NodeState {
    node_name: String,
    network_id: String,
    peer_count: usize,
    block_height: u64,
    last_block_time: Instant,
    connected_peers: Vec<String>,
    is_synced: bool,
    uptime: Instant,
    messages_processed: u64,
    network: Arc<Mutex<NetworkState>>,
}

// MCP server state
struct McpState {
    tools: HashMap<String, serde_json::Value>,
    resources: HashMap<String, serde_json::Value>,
    capabilities: serde_json::Value,
    active_sessions: HashMap<String, Instant>,
}

fn main() {
    println!("QuDAG Node with MCP Server Starting...");
    println!("Version: 1.0.0-mcp");
    println!("Build: rust-mcp-http-sse");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let default_config = String::from("/data/qudag/config.toml");
    let config_path = args.iter()
        .position(|arg| arg == "--config")
        .and_then(|i| args.get(i + 1))
        .unwrap_or(&default_config);
    
    println!("Loading configuration from: {}", config_path);
    
    // Environment variables
    let node_name = env::var("QUDAG_NODE_NAME").unwrap_or_else(|_| "mcp-node".to_string());
    let network_id = env::var("QUDAG_NETWORK_ID").unwrap_or_else(|_| "qudag-testnet".to_string());
    let p2p_port = env::var("QUDAG_P2P_PORT").unwrap_or_else(|_| "4001".to_string());
    let http_port = env::var("QUDAG_RPC_PORT").unwrap_or_else(|_| "8080".to_string());
    let metrics_port = env::var("QUDAG_METRICS_PORT").unwrap_or_else(|_| "9090".to_string());
    let mcp_port = env::var("QUDAG_MCP_PORT").unwrap_or_else(|_| "3333".to_string());
    let bootstrap_peers_str = env::var("QUDAG_BOOTSTRAP_PEERS").unwrap_or_default();
    
    // Parse bootstrap peers
    let bootstrap_peers: Vec<String> = if !bootstrap_peers_str.is_empty() {
        bootstrap_peers_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![
            "/dns4/qudag-testnet-node1.fly.dev/tcp/4001".to_string(),
            "/dns4/qudag-testnet-node2.fly.dev/tcp/4001".to_string(),
            "/dns4/qudag-testnet-node3.fly.dev/tcp/4001".to_string(),
            "/dns4/qudag-testnet-node4.fly.dev/tcp/4001".to_string(),
        ]
    };
    
    let is_bootstrap = env::var("QUDAG_IS_BOOTSTRAP").unwrap_or_else(|_| "false".to_string()) == "true";
    
    println!("MCP Node Configuration:");
    println!("  Name: {}", node_name);
    println!("  Network ID: {}", network_id);
    println!("  P2P Port: {}", p2p_port);
    println!("  HTTP Port: {}", http_port);
    println!("  MCP Port: {}", mcp_port);
    println!("  Metrics Port: {}", metrics_port);
    println!("  Bootstrap Node: {}", is_bootstrap);
    
    // Generate a unique node ID
    let node_id = format!("{}-{:08x}", node_name, fastrand::u32(..));
    
    // Initialize network state
    let network_state = NetworkState {
        node_id: node_id.clone(),
        connected_peers: HashMap::new(),
        bootstrap_peers: bootstrap_peers.clone(),
        message_count: 0,
        bytes_sent: 0,
        bytes_received: 0,
    };
    
    // Initialize node state
    let state = Arc::new(Mutex::new(NodeState {
        node_name: node_name.clone(),
        network_id: network_id.clone(),
        peer_count: 0,
        block_height: if is_bootstrap { 1 } else { 0 },
        last_block_time: Instant::now(),
        connected_peers: Vec::new(),
        is_synced: is_bootstrap,
        uptime: Instant::now(),
        messages_processed: 0,
        network: Arc::new(Mutex::new(network_state)),
    }));
    
    // Initialize MCP state
    let mcp_state = Arc::new(Mutex::new(McpState {
        tools: init_mcp_tools(),
        resources: init_mcp_resources(),
        capabilities: init_mcp_capabilities(),
        active_sessions: HashMap::new(),
    }));
    
    println!("✓ Node state initialized");
    println!("  Node ID: {}", node_id);
    
    // Start P2P listener thread
    let p2p_state = state.clone();
    let p2p_port_clone = p2p_port.clone();
    thread::spawn(move || {
        start_p2p_listener(&p2p_port_clone, p2p_state);
    });
    
    // Start HTTP server thread
    let http_state = state.clone();
    let http_port_clone = http_port.clone();
    thread::spawn(move || {
        start_http_server(&http_port_clone, http_state);
    });
    
    // Start MCP server thread
    let mcp_server_state = state.clone();
    let mcp_server_mcp_state = mcp_state.clone();
    let mcp_port_clone = mcp_port.clone();
    thread::spawn(move || {
        start_mcp_server(&mcp_port_clone, mcp_server_state, mcp_server_mcp_state);
    });
    
    // Start metrics server thread
    let metrics_state = state.clone();
    let metrics_port_clone = metrics_port.clone();
    thread::spawn(move || {
        start_metrics_server(&metrics_port_clone, metrics_state);
    });
    
    // Main loop
    println!("✓ QuDAG MCP node started successfully");
    println!("  P2P networking: Active on port {}", p2p_port);
    println!("  HTTP API: Active on port {}", http_port);
    println!("  MCP Server: Active on port {}", mcp_port);
    println!("  Metrics: Active on port {}", metrics_port);
    println!();
    println!("MCP endpoints available at:");
    println!("  Direct: http://0.0.0.0:{}/mcp", mcp_port);
    println!("  Via HTTP: http://0.0.0.0:{}/mcp", http_port);
    println!("  Via HTTPS: https://qudag-testnet-node1.fly.dev/mcp");
    println!();
    println!("Available endpoints:");
    println!("  /mcp - Discovery");
    println!("  /mcp/info - Server info");
    println!("  /mcp/tools - List tools");
    println!("  /mcp/resources - List resources");
    println!("  /mcp/events - SSE stream");
    println!("  /mcp/rpc - JSON-RPC");
    println!();
    println!("Press Ctrl+C to stop the node");
    
    let mut loop_counter = 0;
    
    loop {
        thread::sleep(Duration::from_secs(5));
        loop_counter += 1;
        
        let mut state_lock = state.lock().unwrap();
        
        // Simulate block production
        if state_lock.last_block_time.elapsed() > Duration::from_secs(if is_bootstrap { 3 } else { 5 }) {
            state_lock.block_height += 1;
            state_lock.last_block_time = Instant::now();
            state_lock.messages_processed += fastrand::u64(1..=3);
            
            // Update network stats
            {
                let mut network_lock = state_lock.network.lock().unwrap();
                network_lock.message_count += 1;
                network_lock.bytes_sent += fastrand::u64(50..=500);
            }
            
            println!("[Block] MCP node produced block: height={}, peers={}", 
                state_lock.block_height, state_lock.peer_count);
        }
        
        // Status report
        if loop_counter % 12 == 0 {
            let network_lock = state_lock.network.lock().unwrap();
            println!("[Status] MCP Node Report:");
            println!("  Height: {}, Peers: {}, Synced: {}", 
                state_lock.block_height, state_lock.peer_count, state_lock.is_synced);
            println!("  Network Messages: {}, Bytes Sent: {}", 
                network_lock.message_count, network_lock.bytes_sent);
        }
    }
}

// Initialize MCP tools
fn init_mcp_tools() -> HashMap<String, serde_json::Value> {
    let mut tools = HashMap::new();
    
    tools.insert("qudag_crypto".to_string(), json!({
        "name": "qudag_crypto",
        "description": "Quantum-resistant cryptography operations",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["generate_keys", "sign", "verify", "encrypt", "decrypt"]
                },
                "algorithm": {
                    "type": "string",
                    "enum": ["ml-dsa", "ml-kem", "hqc", "blake3"]
                }
            }
        }
    }));
    
    tools.insert("qudag_vault".to_string(), json!({
        "name": "qudag_vault",
        "description": "Secure vault operations",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["create", "unlock", "store", "retrieve", "list"]
                }
            }
        }
    }));
    
    tools.insert("qudag_dag".to_string(), json!({
        "name": "qudag_dag",
        "description": "DAG consensus operations",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["get_status", "add_vertex", "get_tips", "validate"]
                }
            }
        }
    }));
    
    tools.insert("qudag_network".to_string(), json!({
        "name": "qudag_network",
        "description": "P2P network operations",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["list_peers", "connect", "disconnect", "broadcast"]
                }
            }
        }
    }));
    
    tools.insert("qudag_exchange".to_string(), json!({
        "name": "qudag_exchange",
        "description": "rUv token exchange operations",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["get_balance", "transfer", "get_fees", "list_accounts"]
                }
            }
        }
    }));
    
    tools
}

// Initialize MCP resources
fn init_mcp_resources() -> HashMap<String, serde_json::Value> {
    let mut resources = HashMap::new();
    
    resources.insert("dag_status".to_string(), json!({
        "name": "dag_status",
        "description": "Current DAG consensus status",
        "uri": "qudag://dag/status"
    }));
    
    resources.insert("network_peers".to_string(), json!({
        "name": "network_peers",
        "description": "Connected P2P network peers",
        "uri": "qudag://network/peers"
    }));
    
    resources.insert("crypto_keys".to_string(), json!({
        "name": "crypto_keys",
        "description": "Available cryptographic keys",
        "uri": "qudag://crypto/keys"
    }));
    
    resources.insert("vault_status".to_string(), json!({
        "name": "vault_status",
        "description": "Vault status and contents",
        "uri": "qudag://vault/status"
    }));
    
    resources.insert("exchange_info".to_string(), json!({
        "name": "exchange_info",
        "description": "Exchange status and accounts",
        "uri": "qudag://exchange/info"
    }));
    
    resources
}

// Initialize MCP capabilities
fn init_mcp_capabilities() -> serde_json::Value {
    json!({
        "experimental": {
            "streamingTools": true,
            "partialResults": true
        },
        "sampling": {},
        "tools": true,
        "resources": {
            "subscribe": true,
            "listChanged": true
        },
        "prompts": {
            "listChanged": true
        },
        "logging": {}
    })
}

// Start MCP server with HTTP/SSE support
fn start_mcp_server(port: &str, state: Arc<Mutex<NodeState>>, mcp_state: Arc<Mutex<McpState>>) {
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(&addr) {
        Ok(listener) => {
            println!("[MCP] Server listening on http://{}", addr);
            println!("[MCP] Available endpoints:");
            println!("  - GET  /mcp - MCP discovery");
            println!("  - GET  /mcp/info - Server information");
            println!("  - GET  /mcp/tools - List available tools");
            println!("  - POST /mcp/tools/call - Execute tool");
            println!("  - GET  /mcp/resources - List resources");
            println!("  - GET  /mcp/resources/:name - Get resource");
            println!("  - GET  /mcp/events - Server-sent events stream");
            println!("  - POST /mcp/rpc - JSON-RPC endpoint");
            
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let state_clone = state.clone();
                        let mcp_state_clone = mcp_state.clone();
                        thread::spawn(move || {
                            handle_mcp_request(&mut stream, &state_clone, &mcp_state_clone);
                        });
                    }
                    Err(e) => {
                        println!("[MCP] Connection error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("[MCP] Server failed to bind to {}: {}", addr, e);
        }
    }
}

// Handle MCP HTTP requests
fn handle_mcp_request(stream: &mut TcpStream, state: &Arc<Mutex<NodeState>>, mcp_state: &Arc<Mutex<McpState>>) {
    let mut buffer = [0; 4096];
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            
            // Parse HTTP request
            let lines: Vec<&str> = request.lines().collect();
            if lines.is_empty() {
                return;
            }
            
            let request_line = lines[0];
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() < 2 {
                return;
            }
            
            let method = parts[0];
            let path = parts[1];
            
            // Route MCP requests
            match (method, path) {
                ("GET", "/mcp") | ("GET", "/mcp/") => {
                    handle_mcp_discovery(stream, mcp_state);
                }
                ("GET", "/mcp/info") => {
                    handle_mcp_info(stream, state);
                }
                ("GET", "/mcp/tools") => {
                    handle_mcp_tools_list(stream, mcp_state);
                }
                ("POST", "/mcp/tools/call") => {
                    handle_mcp_tool_call(stream, &request, state, mcp_state);
                }
                ("GET", "/mcp/resources") => {
                    handle_mcp_resources_list(stream, mcp_state);
                }
                ("GET", path) if path.starts_with("/mcp/resources/") => {
                    let resource_name = path.trim_start_matches("/mcp/resources/");
                    handle_mcp_resource_get(stream, resource_name, state, mcp_state);
                }
                ("GET", "/mcp/events") => {
                    handle_mcp_sse(stream, state, mcp_state);
                }
                ("POST", "/mcp/rpc") => {
                    handle_mcp_rpc(stream, &request, state, mcp_state);
                }
                ("GET", "/.well-known/mcp") => {
                    handle_mcp_wellknown(stream);
                }
                _ => {
                    let response = "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{\"error\":\"MCP endpoint not found\"}\r\n";
                    let _ = stream.write(response.as_bytes());
                }
            }
        }
        Err(_) => {}
    }
}

// MCP discovery endpoint
fn handle_mcp_discovery(stream: &mut TcpStream, mcp_state: &Arc<Mutex<McpState>>) {
    let mcp_lock = mcp_state.lock().unwrap();
    let discovery = json!({
        "mcp": {
            "version": "0.1.0",
            "serverInfo": {
                "name": "QuDAG MCP Server",
                "version": "1.0.0",
                "protocolVersion": "2024-11-05"
            },
            "capabilities": mcp_lock.capabilities,
            "instructions": "QuDAG MCP server for quantum-resistant DAG operations"
        }
    });
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        discovery.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

// MCP server info
fn handle_mcp_info(stream: &mut TcpStream, state: &Arc<Mutex<NodeState>>) {
    let state_lock = state.lock().unwrap();
    let network_lock = state_lock.network.lock().unwrap();
    
    let info = json!({
        "name": "QuDAG MCP Server",
        "version": "1.0.0",
        "protocolVersion": "2024-11-05",
        "vendor": "QuDAG",
        "supportedVersions": ["2024-11-05"],
        "nodeInfo": {
            "id": network_lock.node_id,
            "name": state_lock.node_name,
            "network": state_lock.network_id,
            "height": state_lock.block_height,
            "peers": state_lock.peer_count,
            "uptime": state_lock.uptime.elapsed().as_secs()
        }
    });
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        info.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

// List MCP tools
fn handle_mcp_tools_list(stream: &mut TcpStream, mcp_state: &Arc<Mutex<McpState>>) {
    let mcp_lock = mcp_state.lock().unwrap();
    let tools: Vec<&serde_json::Value> = mcp_lock.tools.values().collect();
    
    let response_data = json!({
        "tools": tools
    });
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        response_data.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

// Execute MCP tool
fn handle_mcp_tool_call(stream: &mut TcpStream, request: &str, state: &Arc<Mutex<NodeState>>, mcp_state: &Arc<Mutex<McpState>>) {
    // Extract body from request
    let body_start = request.find("\r\n\r\n").unwrap_or(request.len()) + 4;
    let body = &request[body_start..];
    
    // Parse JSON body
    if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(body) {
        let tool_name = json_body["name"].as_str().unwrap_or("");
        let arguments = &json_body["arguments"];
        
        // Simulate tool execution
        let result = match tool_name {
            "qudag_crypto" => {
                json!({
                    "success": true,
                    "result": {
                        "operation": arguments["operation"],
                        "algorithm": arguments["algorithm"],
                        "key": "ml-dsa-pubkey-example-12345",
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    }
                })
            }
            "qudag_dag" => {
                let state_lock = state.lock().unwrap();
                json!({
                    "success": true,
                    "result": {
                        "height": state_lock.block_height,
                        "synced": state_lock.is_synced,
                        "tips": 2,
                        "vertices": state_lock.messages_processed
                    }
                })
            }
            "qudag_network" => {
                let state_lock = state.lock().unwrap();
                json!({
                    "success": true,
                    "result": {
                        "peers": state_lock.peer_count,
                        "connected": state_lock.connected_peers.clone()
                    }
                })
            }
            _ => {
                json!({
                    "success": false,
                    "error": format!("Unknown tool: {}", tool_name)
                })
            }
        };
        
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
            result.to_string()
        );
        let _ = stream.write(response.as_bytes());
    } else {
        let error = json!({"error": "Invalid JSON body"});
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
            error.to_string()
        );
        let _ = stream.write(response.as_bytes());
    }
}

// List MCP resources
fn handle_mcp_resources_list(stream: &mut TcpStream, mcp_state: &Arc<Mutex<McpState>>) {
    let mcp_lock = mcp_state.lock().unwrap();
    let resources: Vec<&serde_json::Value> = mcp_lock.resources.values().collect();
    
    let response_data = json!({
        "resources": resources
    });
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        response_data.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

// Get specific MCP resource
fn handle_mcp_resource_get(stream: &mut TcpStream, resource_name: &str, state: &Arc<Mutex<NodeState>>, _mcp_state: &Arc<Mutex<McpState>>) {
    let state_lock = state.lock().unwrap();
    let network_lock = state_lock.network.lock().unwrap();
    
    let resource_data = match resource_name {
        "dag_status" => {
            json!({
                "contents": {
                    "height": state_lock.block_height,
                    "synced": state_lock.is_synced,
                    "messages_processed": state_lock.messages_processed,
                    "last_block_time": state_lock.last_block_time.elapsed().as_secs()
                }
            })
        }
        "network_peers" => {
            json!({
                "contents": {
                    "peer_count": state_lock.peer_count,
                    "connected_peers": state_lock.connected_peers.clone(),
                    "bootstrap_peers": network_lock.bootstrap_peers.clone()
                }
            })
        }
        "crypto_keys" => {
            json!({
                "contents": {
                    "available_algorithms": ["ml-dsa", "ml-kem", "hqc"],
                    "keys": [
                        {"id": "key-1", "algorithm": "ml-dsa", "type": "signing"},
                        {"id": "key-2", "algorithm": "ml-kem", "type": "encryption"}
                    ]
                }
            })
        }
        _ => {
            json!({
                "error": format!("Resource not found: {}", resource_name)
            })
        }
    };
    
    let status = if resource_data.get("error").is_some() { "404 Not Found" } else { "200 OK" };
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        status,
        resource_data.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

// Server-Sent Events for MCP
fn handle_mcp_sse(stream: &mut TcpStream, state: &Arc<Mutex<NodeState>>, _mcp_state: &Arc<Mutex<McpState>>) {
    // Send SSE headers
    let headers = "HTTP/1.1 200 OK\r\n\
                   Content-Type: text/event-stream\r\n\
                   Cache-Control: no-cache\r\n\
                   Connection: keep-alive\r\n\
                   Access-Control-Allow-Origin: *\r\n\r\n";
    
    if stream.write(headers.as_bytes()).is_err() {
        return;
    }
    
    // Send initial event
    let initial_event = format!("event: connected\ndata: {{\"message\":\"Connected to QuDAG MCP SSE stream\"}}\n\n");
    if stream.write(initial_event.as_bytes()).is_err() {
        return;
    }
    
    // Stream events
    let mut event_counter = 0;
    loop {
        thread::sleep(Duration::from_secs(5));
        event_counter += 1;
        
        let state_lock = state.lock().unwrap();
        
        // Send status update event
        let event_data = json!({
            "type": "status_update",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "data": {
                "height": state_lock.block_height,
                "peers": state_lock.peer_count,
                "synced": state_lock.is_synced,
                "messages": state_lock.messages_processed
            }
        });
        
        let event = format!("event: status\ndata: {}\nid: {}\n\n", 
            event_data.to_string(), event_counter);
        
        if stream.write(event.as_bytes()).is_err() {
            break;
        }
        
        // Occasionally send resource update events
        if event_counter % 3 == 0 {
            let resource_event = json!({
                "type": "resource_update",
                "resource": "dag_status",
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            });
            
            let event = format!("event: resource_update\ndata: {}\nid: {}\n\n", 
                resource_event.to_string(), event_counter + 1000);
            
            if stream.write(event.as_bytes()).is_err() {
                break;
            }
        }
    }
}

// JSON-RPC handler for MCP
fn handle_mcp_rpc(stream: &mut TcpStream, request: &str, state: &Arc<Mutex<NodeState>>, mcp_state: &Arc<Mutex<McpState>>) {
    // Extract body from request
    let body_start = request.find("\r\n\r\n").unwrap_or(request.len()) + 4;
    let body = &request[body_start..];
    
    // Parse JSON-RPC request
    if let Ok(json_body) = serde_json::from_str::<serde_json::Value>(body) {
        let method = json_body["method"].as_str().unwrap_or("");
        let params = &json_body["params"];
        let id = &json_body["id"];
        
        let result = match method {
            "mcp/list_tools" => {
                let mcp_lock = mcp_state.lock().unwrap();
                let tools: Vec<&serde_json::Value> = mcp_lock.tools.values().collect();
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "tools": tools
                    },
                    "id": id
                })
            }
            "mcp/server_capabilities" => {
                let mcp_lock = mcp_state.lock().unwrap();
                json!({
                    "jsonrpc": "2.0",
                    "result": mcp_lock.capabilities.clone(),
                    "id": id
                })
            }
            "tools/call" => {
                // Delegate to tool handler
                let tool_name = params["name"].as_str().unwrap_or("");
                let state_lock = state.lock().unwrap();
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "success": true,
                        "tool": tool_name,
                        "output": format!("Executed {} at height {}", tool_name, state_lock.block_height)
                    },
                    "id": id
                })
            }
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    },
                    "id": id
                })
            }
        };
        
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
            result.to_string()
        );
        let _ = stream.write(response.as_bytes());
    } else {
        let error = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32700,
                "message": "Parse error"
            },
            "id": null
        });
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
            error.to_string()
        );
        let _ = stream.write(response.as_bytes());
    }
}

// Well-known MCP endpoint
fn handle_mcp_wellknown(stream: &mut TcpStream) {
    let wellknown = json!({
        "mcp": {
            "endpoint": "/mcp",
            "version": "2024-11-05"
        }
    });
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        wellknown.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

// P2P listener (same as before)
fn start_p2p_listener(port: &str, state: Arc<Mutex<NodeState>>) {
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(&addr) {
        Ok(listener) => {
            println!("[P2P] Listening on {}", addr);
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let peer_addr = stream.peer_addr().map(|a| a.to_string())
                            .unwrap_or_else(|_| "unknown".to_string());
                        
                        println!("[P2P] Connection from {}", peer_addr);
                        
                        // Send handshake
                        let handshake = format!("QUDAG-MCP/1.0\nNode-ID: {}\nNetwork: {}\n\n", 
                            state.lock().unwrap().network.lock().unwrap().node_id,
                            state.lock().unwrap().network_id);
                        
                        if let Err(_) = stream.write(handshake.as_bytes()) {
                            continue;
                        }
                        
                        // Add peer
                        let peer_id = format!("peer-{:08x}", fastrand::u32(..));
                        {
                            let state_lock = state.lock().unwrap();
                            let mut network_lock = state_lock.network.lock().unwrap();
                            network_lock.connected_peers.insert(peer_id.clone(), PeerInfo {
                                id: peer_id.clone(),
                                address: peer_addr,
                                last_seen: Instant::now(),
                                connection_time: Instant::now(),
                                message_count: 0,
                            });
                            network_lock.bytes_received += handshake.len() as u64;
                        }
                        
                        println!("[P2P] Peer {} connected", peer_id);
                    }
                    Err(e) => {
                        println!("[P2P] Connection error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("[P2P] Failed to bind to {}: {}", addr, e);
        }
    }
}

// HTTP server (same as enhanced node)
fn start_http_server(port: &str, state: Arc<Mutex<NodeState>>) {
    let addr = format!("0.0.0.0:{}", port);
    let addr: SocketAddr = addr.parse().expect("Invalid address");
    
    match TcpListener::bind(&addr) {
        Ok(listener) => {
            println!("[HTTP] Server listening on http://{}", addr);
            println!("[HTTP] Available endpoints:");
            println!("  - GET /health");
            println!("  - GET /api/v1/status");
            println!("  - GET /metrics");
            
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        handle_http_request(&mut stream, &state);
                    }
                    Err(_) => {}
                }
            }
        }
        Err(e) => {
            println!("[HTTP] Server failed to bind to {}: {}", addr, e);
        }
    }
}

fn handle_http_request(stream: &mut TcpStream, state: &Arc<Mutex<NodeState>>) {
    let mut buffer = [0; 4096];
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            
            // Check if this is an MCP request
            if request.contains("/mcp") || request.contains("/.well-known/mcp") {
                // Get MCP state from node state
                let mcp_state = {
                    let state_lock = state.lock().unwrap();
                    // Create MCP state on demand
                    Arc::new(Mutex::new(McpState {
                        tools: init_mcp_tools(),
                        resources: init_mcp_resources(),
                        capabilities: init_mcp_capabilities(),
                        active_sessions: HashMap::new(),
                    }))
                };
                
                // Delegate to MCP handler
                handle_mcp_request(stream, state, &mcp_state);
            } else if request.contains("GET /health") {
                handle_health_endpoint(stream, state);
            } else if request.contains("GET /api/v1/status") {
                handle_status_endpoint(stream, state);
            } else if request.contains("GET /metrics") {
                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nMetrics available on port 9090\r\n";
                let _ = stream.write(response.as_bytes());
            } else {
                let response = "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{\"error\":\"Not Found\"}\r\n";
                let _ = stream.write(response.as_bytes());
            }
        }
        Err(_) => {}
    }
}

fn handle_health_endpoint(stream: &mut TcpStream, state: &Arc<Mutex<NodeState>>) {
    let state_lock = state.lock().unwrap();
    let network_lock = state_lock.network.lock().unwrap();
    
    let health_data = json!({
        "status": if state_lock.is_synced { "healthy" } else { "syncing" },
        "timestamp": state_lock.uptime.elapsed().as_secs(),
        "version": "1.0.0-mcp",
        "details": {
            "node_id": network_lock.node_id,
            "node_name": state_lock.node_name,
            "network_id": state_lock.network_id,
            "synced": state_lock.is_synced,
            "peers": state_lock.peer_count,
            "height": state_lock.block_height,
            "network_messages": network_lock.message_count,
            "bytes_sent": network_lock.bytes_sent,
            "bytes_received": network_lock.bytes_received,
            "mcp_enabled": true,
            "mcp_port": 3333,
            "mcp_http_enabled": true
        }
    });
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        health_data.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

fn handle_status_endpoint(stream: &mut TcpStream, state: &Arc<Mutex<NodeState>>) {
    let state_lock = state.lock().unwrap();
    let network_lock = state_lock.network.lock().unwrap();
    
    let status_data = json!({
        "node": {
            "id": network_lock.node_id,
            "name": state_lock.node_name,
            "version": "1.0.0-mcp",
            "network_id": state_lock.network_id,
            "uptime_seconds": state_lock.uptime.elapsed().as_secs()
        },
        "p2p": {
            "listening": true,
            "peer_count": state_lock.peer_count,
            "connected_peers": state_lock.connected_peers,
            "network_messages": network_lock.message_count,
            "bytes_sent": network_lock.bytes_sent,
            "bytes_received": network_lock.bytes_received
        },
        "dag": {
            "height": state_lock.block_height,
            "messages_processed": state_lock.messages_processed,
            "synced": state_lock.is_synced
        },
        "mcp": {
            "enabled": true,
            "port": 3333,
            "http_port": 8080,
            "accessible_via": [
                "http://NODE_IP:3333/mcp",
                "https://qudag-testnet-node1.fly.dev/mcp"
            ],
            "endpoints": [
                "/mcp",
                "/mcp/info",
                "/mcp/tools",
                "/mcp/resources",
                "/mcp/events",
                "/mcp/rpc",
                "/.well-known/mcp"
            ]
        }
    });
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
        status_data.to_string()
    );
    let _ = stream.write(response.as_bytes());
}

fn start_metrics_server(port: &str, state: Arc<Mutex<NodeState>>) {
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(&addr) {
        Ok(listener) => {
            println!("[Metrics] Server listening on http://{}/metrics", addr);
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        handle_metrics_request(&mut stream, &state);
                    }
                    Err(_) => {}
                }
            }
        }
        Err(e) => {
            println!("[Metrics] Server failed to bind to {}: {}", addr, e);
        }
    }
}

fn handle_metrics_request(stream: &mut TcpStream, state: &Arc<Mutex<NodeState>>) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            
            if request.contains("GET /metrics") {
                let state_lock = state.lock().unwrap();
                let network_lock = state_lock.network.lock().unwrap();
                
                let metrics = format!(
                    "# HELP qudag_peer_count Number of connected peers\n\
                     # TYPE qudag_peer_count gauge\n\
                     qudag_peer_count {}\n\
                     # HELP qudag_block_height Current block height\n\
                     # TYPE qudag_block_height counter\n\
                     qudag_block_height {}\n\
                     # HELP qudag_is_synced Whether node is synced\n\
                     # TYPE qudag_is_synced gauge\n\
                     qudag_is_synced {}\n\
                     # HELP qudag_messages_processed Total messages processed\n\
                     # TYPE qudag_messages_processed counter\n\
                     qudag_messages_processed {}\n\
                     # HELP qudag_network_messages Total network messages\n\
                     # TYPE qudag_network_messages counter\n\
                     qudag_network_messages {}\n\
                     # HELP qudag_bytes_sent Total bytes sent\n\
                     # TYPE qudag_bytes_sent counter\n\
                     qudag_bytes_sent {}\n\
                     # HELP qudag_bytes_received Total bytes received\n\
                     # TYPE qudag_bytes_received counter\n\
                     qudag_bytes_received {}\n\
                     # HELP qudag_uptime_seconds Node uptime in seconds\n\
                     # TYPE qudag_uptime_seconds counter\n\
                     qudag_uptime_seconds {}\n\
                     # HELP qudag_mcp_enabled MCP server enabled\n\
                     # TYPE qudag_mcp_enabled gauge\n\
                     qudag_mcp_enabled 1\n",
                    state_lock.peer_count,
                    state_lock.block_height,
                    if state_lock.is_synced { 1 } else { 0 },
                    state_lock.messages_processed,
                    network_lock.message_count,
                    network_lock.bytes_sent,
                    network_lock.bytes_received,
                    state_lock.uptime.elapsed().as_secs()
                );
                
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n",
                    metrics
                );
                let _ = stream.write(response.as_bytes());
            } else {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                let _ = stream.write(response.as_bytes());
            }
        }
        Err(_) => {}
    }
}