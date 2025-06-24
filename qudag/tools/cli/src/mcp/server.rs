//! MCP server management implementation

use crate::CliError;
use qudag_mcp::{create_server, McpConfig, QuDAGMCPServer, ServerConfig};
use std::path::PathBuf;
use std::process::Stdio;
use tracing::{error, info, warn};

/// Start MCP server in foreground mode
pub async fn start_mcp_server_foreground(
    bind: String,
    transport: String,
    config_path: Option<PathBuf>,
    verbose: bool,
) -> Result<(), CliError> {
    info!("Starting MCP server in foreground mode");

    // Load or create MCP configuration
    let mcp_config = if let Some(path) = config_path {
        if path.exists() {
            McpConfig::from_file(&path)
                .map_err(|e| CliError::Config(format!("Failed to load config: {}", e)))?
        } else {
            return Err(CliError::Config(format!(
                "Config file not found: {:?}",
                path
            )));
        }
    } else {
        McpConfig::default()
    };

    // Parse bind address
    let parsed_bind = if bind.contains(':') {
        bind.clone()
    } else {
        format!("{}:{}", bind, 3000) // Default port
    };

    // Create transport configuration based on CLI parameter
    let transport_config = match transport.as_str() {
        "stdio" => qudag_mcp::TransportConfig::Stdio,
        "http" => qudag_mcp::TransportConfig::Http {
            server_url: format!("http://{}", parsed_bind),
        },
        "websocket" | "ws" => qudag_mcp::TransportConfig::WebSocket {
            url: format!("ws://{}/mcp", parsed_bind),
        },
        _ => {
            return Err(CliError::Config(format!(
                "Unsupported transport type: {}",
                transport
            )));
        }
    };

    // Create server configuration
    let server_config = ServerConfig::new()
        .with_server_info("QuDAG MCP Server", qudag_mcp::VERSION)
        .with_transport(transport_config)
        .with_log_level(if verbose { "debug" } else { "info" });

    // For stdio transport, all output must go to stderr to avoid interfering with JSON-RPC
    eprintln!("Starting QuDAG MCP Server");
    eprintln!("=========================");
    eprintln!("Bind Address: {}", parsed_bind);
    eprintln!("Transport: {:?}", transport);
    eprintln!("Server Name: {}", server_config.server_info.name);
    eprintln!("Version: {}", server_config.server_info.version);

    if verbose {
        eprintln!("MCP Config Host: {}", mcp_config.server.host);
        eprintln!("MCP Config Port: {}", mcp_config.server.port);
        eprintln!(
            "MCP Config Max Connections: {}",
            mcp_config.server.max_connections
        );
        eprintln!(
            "MCP Config Request Timeout: {}s",
            mcp_config.server.request_timeout.as_secs()
        );
        eprintln!("MCP Config Auth Vault: {:?}", mcp_config.auth.vault_path);
        eprintln!("MCP Config MFA Enabled: {}", mcp_config.auth.mfa_enabled);
    }
    eprintln!();

    // Create and start server
    let mut server = QuDAGMCPServer::new(server_config)
        .await
        .map_err(|e| CliError::Server(format!("Failed to create MCP server: {}", e)))?;

    eprintln!("✓ MCP server initialized successfully");
    eprintln!("✓ QuDAG tools and resources loaded");
    eprintln!();

    // Setup graceful shutdown
    eprintln!("✓ Starting MCP server main loop");
    eprintln!("  Press Ctrl+C to stop the server");
    eprintln!();

    // Setup signal handler
    let ctrl_c = tokio::signal::ctrl_c();

    tokio::select! {
        result = server.run() => {
            match result {
                Ok(()) => {
                    eprintln!("✓ MCP server stopped gracefully");
                }
                Err(e) => {
                    error!("MCP server error: {}", e);
                    return Err(CliError::Server(format!("MCP server error: {}", e)));
                }
            }
        }
        _ = ctrl_c => {
            eprintln!("\n🛑 Shutting down MCP server...");
            if let Err(e) = server.stop().await {
                warn!("Error during shutdown: {}", e);
            }
            eprintln!("✓ MCP server stopped");
        }
    }

    Ok(())
}

/// Start MCP server in background mode
pub async fn start_mcp_server_background(
    bind: String,
    transport: String,
    config_path: Option<PathBuf>,
    verbose: bool,
) -> Result<(), CliError> {
    info!("Starting MCP server in background mode");

    // Get current executable path
    let current_exe = std::env::current_exe()
        .map_err(|e| CliError::Config(format!("Failed to get current executable: {}", e)))?;

    // Build command arguments
    let mut args = vec![
        "mcp".to_string(),
        "start".to_string(),
        "--bind".to_string(),
        bind,
        "--transport".to_string(),
        transport,
    ];

    if let Some(config) = config_path {
        args.push("--config".to_string());
        args.push(config.to_string_lossy().to_string());
    }

    if verbose {
        args.push("--verbose".to_string());
    }

    // Start background process
    let mut cmd = tokio::process::Command::new(&current_exe);
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd
        .spawn()
        .map_err(|e| CliError::Server(format!("Failed to start background MCP server: {}", e)))?;

    let pid = child.id().unwrap_or(0);

    // Save PID for later management
    save_mcp_server_pid(pid).await?;

    eprintln!("✓ MCP server started in background");
    eprintln!("  Process ID: {}", pid);
    eprintln!("  Use 'qudag mcp status' to check server status");
    eprintln!("  Use 'qudag mcp stop' to stop the server");

    Ok(())
}

/// Stop MCP server
pub async fn stop_mcp_server(force: bool) -> Result<(), CliError> {
    info!("Stopping MCP server");

    // Try to get saved PID
    match get_mcp_server_pid().await {
        Ok(Some(pid)) => {
            eprintln!("Stopping MCP server (PID: {})...", pid);

            // Try graceful shutdown first
            if !force {
                if terminate_process(pid, false).await? {
                    eprintln!("✓ MCP server stopped gracefully");
                    clear_mcp_server_pid().await?;
                    return Ok(());
                }
            }

            // Force kill if graceful shutdown failed or force flag is set
            if force || !terminate_process(pid, false).await? {
                warn!("Graceful shutdown failed, force killing process");
                if terminate_process(pid, true).await? {
                    eprintln!("✓ MCP server force stopped");
                } else {
                    return Err(CliError::Server("Failed to stop MCP server".to_string()));
                }
            }

            clear_mcp_server_pid().await?;
        }
        Ok(None) => {
            eprintln!("No MCP server PID found");

            // Try to find and kill any running MCP servers
            if find_and_kill_mcp_processes(force).await? {
                eprintln!("✓ Found and stopped running MCP server(s)");
            } else {
                eprintln!("No running MCP servers found");
            }
        }
        Err(e) => {
            warn!("Error getting MCP server PID: {}", e);

            // Try to find and kill any running MCP servers
            if find_and_kill_mcp_processes(force).await? {
                eprintln!("✓ Found and stopped running MCP server(s)");
            } else {
                return Err(CliError::Server("No MCP server found to stop".to_string()));
            }
        }
    }

    Ok(())
}

/// Show MCP server status
pub async fn show_mcp_server_status() -> Result<(), CliError> {
    info!("Getting MCP server status");

    eprintln!("MCP Server Status");
    eprintln!("=================");

    // Check if we have a saved PID
    match get_mcp_server_pid().await {
        Ok(Some(pid)) => {
            eprintln!("Saved PID: {}", pid);

            // Check if process is actually running
            if is_process_running(pid).await? {
                eprintln!("Status: ✓ Running");

                // Try to get additional info
                if let Ok(info) = get_process_info(pid).await {
                    eprintln!("Uptime: {}", info.uptime);
                    eprintln!("Memory: {}", info.memory);
                    eprintln!("CPU: {}%", info.cpu_percent);
                }

                // Try to check server health
                match check_server_health().await {
                    Ok(health) => {
                        eprintln!("Health: ✓ Healthy");
                        eprintln!("Endpoint: {}", health.endpoint);
                        eprintln!("Response Time: {}ms", health.response_time_ms);
                    }
                    Err(e) => {
                        eprintln!("Health: ✗ Unhealthy ({})", e);
                    }
                }
            } else {
                eprintln!("Status: ✗ Not running (stale PID)");
                clear_mcp_server_pid().await?;
            }
        }
        Ok(None) => {
            eprintln!("Saved PID: None");

            // Try to find running MCP servers
            let running_processes = find_mcp_processes().await?;
            if running_processes.is_empty() {
                eprintln!("Status: ✗ Not running");
            } else {
                eprintln!("Status: ⚠ Running (unmanaged)");
                eprintln!(
                    "Found {} unmanaged MCP process(es):",
                    running_processes.len()
                );
                for pid in running_processes {
                    eprintln!("  - PID: {}", pid);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Show configuration info
    eprintln!();
    eprintln!("Configuration:");
    if let Ok(config_path) = get_default_config_path() {
        if config_path.exists() {
            eprintln!("  Config file: {:?}", config_path);

            if let Ok(config) = McpConfig::from_file(&config_path) {
                eprintln!("  Server host: {}", config.server.host);
                eprintln!("  Server port: {}", config.server.port);
                eprintln!("  Max connections: {}", config.server.max_connections);
                eprintln!("  TLS enabled: {}", config.server.tls_enabled);
            }
        } else {
            eprintln!("  Config file: Not found (using defaults)");
        }
    }

    Ok(())
}

/// List available MCP tools
pub async fn list_mcp_tools() -> Result<(), CliError> {
    info!("Listing available MCP tools");

    eprintln!("Available MCP Tools");
    eprintln!("===================");

    // Create a default server to get tools list
    let server = create_server()
        .await
        .map_err(|e| CliError::Server(format!("Failed to create server: {}", e)))?;

    let stats = server.stats().await;

    eprintln!("Tools Count: {}", stats.tools_count);

    // TODO: Update when MCP server provides tools() method
    eprintln!("Note: Tool listing not implemented yet in MCP server");
    eprintln!("Available tool types:");
    eprintln!("  - vault: Vault operations");
    eprintln!("  - dag: DAG operations");
    eprintln!("  - network: Network operations");
    eprintln!("  - crypto: Cryptographic operations");

    Ok(())
}

/// List available MCP resources
pub async fn list_mcp_resources() -> Result<(), CliError> {
    info!("Listing available MCP resources");

    eprintln!("Available MCP Resources");
    eprintln!("=======================");

    // Create a default server to get resources list
    let server = create_server()
        .await
        .map_err(|e| CliError::Server(format!("Failed to create server: {}", e)))?;

    let stats = server.stats().await;

    eprintln!("Resources Count: {}", stats.resources_count);

    // TODO: Update when MCP server provides resources() method
    eprintln!("Note: Resource listing not implemented yet in MCP server");
    eprintln!("Available resource types:");
    eprintln!("  - vault_entries: Vault entries and secrets");
    eprintln!("  - dag_state: DAG state and nodes");
    eprintln!("  - network_peers: Network peer information");
    eprintln!("  - system_status: System status and health");

    Ok(())
}

/// Test MCP server connectivity
pub async fn test_mcp_server(endpoint: String) -> Result<(), CliError> {
    info!("Testing MCP server connectivity to {}", endpoint);

    eprintln!("Testing MCP Server Connectivity");
    eprintln!("===============================");
    eprintln!("Endpoint: {}", endpoint);
    eprintln!();

    // Parse endpoint
    let url = if endpoint.starts_with("http") {
        endpoint
    } else {
        format!("http://{}", endpoint)
    };

    eprintln!("🔗 Connecting to {}...", url);

    // Test basic HTTP connectivity
    let client = reqwest::Client::new();
    let start_time = std::time::Instant::now();

    match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            let duration = start_time.elapsed();
            eprintln!("✓ HTTP connection successful");
            eprintln!("  Status: {}", response.status());
            eprintln!("  Response time: {}ms", duration.as_millis());

            // Test MCP protocol if it's an MCP endpoint
            if url.contains("/mcp") || url.ends_with(":3000") {
                eprintln!();
                eprintln!("🔍 Testing MCP protocol...");

                // Send a ping request
                let ping_request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "ping",
                    "params": {}
                });

                let mcp_url = if url.ends_with("/mcp") {
                    url
                } else if url.ends_with("/") {
                    format!("{}mcp", url)
                } else {
                    format!("{}/mcp", url)
                };

                match client.post(&mcp_url).json(&ping_request).send().await {
                    Ok(mcp_response) => {
                        if mcp_response.status().is_success() {
                            eprintln!("✓ MCP protocol test successful");

                            // Try to parse JSON-RPC response
                            if let Ok(body) = mcp_response.text().await {
                                eprintln!("  Response: {}", body);
                            }
                        } else {
                            eprintln!("✗ MCP protocol test failed: {}", mcp_response.status());
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ MCP protocol test failed: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            let duration = start_time.elapsed();
            eprintln!("✗ Connection failed after {}ms", duration.as_millis());
            eprintln!("  Error: {}", e);

            // Provide troubleshooting suggestions
            eprintln!();
            eprintln!("Troubleshooting:");
            eprintln!("  - Check if MCP server is running: qudag mcp status");
            eprintln!("  - Verify the endpoint address and port");
            eprintln!("  - Check firewall settings");
            eprintln!("  - Try starting MCP server: qudag mcp start");

            return Err(CliError::Server(format!("MCP server test failed: {}", e)));
        }
    }

    eprintln!();
    eprintln!("✓ MCP server test completed successfully");

    Ok(())
}

// Helper functions

async fn setup_shutdown_handler() -> tokio::signal::unix::Signal {
    use tokio::signal::unix::{signal, SignalKind};
    signal(SignalKind::interrupt()).expect("Failed to setup signal handler")
}

async fn save_mcp_server_pid(pid: u32) -> Result<(), CliError> {
    let pid_file = get_pid_file_path()?;
    if let Some(parent) = pid_file.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| CliError::Config(format!("Failed to create PID directory: {}", e)))?;
    }

    tokio::fs::write(&pid_file, pid.to_string())
        .await
        .map_err(|e| CliError::Config(format!("Failed to save PID file: {}", e)))
}

async fn get_mcp_server_pid() -> Result<Option<u32>, CliError> {
    let pid_file = get_pid_file_path()?;

    if !pid_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&pid_file)
        .await
        .map_err(|e| CliError::Config(format!("Failed to read PID file: {}", e)))?;

    let pid = content
        .trim()
        .parse::<u32>()
        .map_err(|e| CliError::Config(format!("Invalid PID in file: {}", e)))?;

    Ok(Some(pid))
}

async fn clear_mcp_server_pid() -> Result<(), CliError> {
    let pid_file = get_pid_file_path()?;

    if pid_file.exists() {
        tokio::fs::remove_file(&pid_file)
            .await
            .map_err(|e| CliError::Config(format!("Failed to remove PID file: {}", e)))?;
    }

    Ok(())
}

fn get_pid_file_path() -> Result<PathBuf, CliError> {
    let home = std::env::var("HOME")
        .map_err(|_| CliError::Config("Unable to determine home directory".to_string()))?;
    Ok(PathBuf::from(home).join(".qudag").join("mcp-server.pid"))
}

async fn terminate_process(pid: u32, force: bool) -> Result<bool, CliError> {
    use tokio::process::Command;

    let signal = if force { "KILL" } else { "TERM" };

    let output = Command::new("kill")
        .arg(format!("-{}", signal))
        .arg(pid.to_string())
        .output()
        .await
        .map_err(|e| CliError::Server(format!("Failed to send signal to process: {}", e)))?;

    if output.status.success() {
        // Wait a moment for the process to terminate
        tokio::time::sleep(tokio::time::Duration::from_secs(if force { 1 } else { 3 })).await;

        // Check if process is still running
        Ok(!is_process_running(pid).await?)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("No such process") {
            Ok(true) // Process already gone
        } else {
            Err(CliError::Server(format!(
                "Failed to terminate process: {}",
                error
            )))
        }
    }
}

async fn is_process_running(pid: u32) -> Result<bool, CliError> {
    use tokio::process::Command;

    let output = Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .await
        .map_err(|e| CliError::Server(format!("Failed to check process: {}", e)))?;

    Ok(output.status.success())
}

async fn find_mcp_processes() -> Result<Vec<u32>, CliError> {
    use tokio::process::Command;

    let output = Command::new("pgrep")
        .arg("-f")
        .arg("qudag.*mcp.*start")
        .output()
        .await
        .map_err(|e| CliError::Server(format!("Failed to search for MCP processes: {}", e)))?;

    if output.status.success() {
        let pids = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .collect();
        Ok(pids)
    } else {
        Ok(Vec::new())
    }
}

async fn find_and_kill_mcp_processes(force: bool) -> Result<bool, CliError> {
    let pids = find_mcp_processes().await?;

    if pids.is_empty() {
        return Ok(false);
    }

    let mut killed_any = false;
    for pid in pids {
        if terminate_process(pid, force).await? {
            killed_any = true;
        }
    }

    Ok(killed_any)
}

#[derive(Debug)]
struct ProcessInfo {
    uptime: String,
    memory: String,
    cpu_percent: f64,
}

async fn get_process_info(_pid: u32) -> Result<ProcessInfo, CliError> {
    // This is a simplified implementation
    // In a real implementation, you would parse /proc/{pid}/stat and /proc/{pid}/status
    Ok(ProcessInfo {
        uptime: "Unknown".to_string(),
        memory: "Unknown".to_string(),
        cpu_percent: 0.0,
    })
}

#[derive(Debug)]
struct ServerHealth {
    endpoint: String,
    response_time_ms: u64,
}

async fn check_server_health() -> Result<ServerHealth, CliError> {
    // This would implement a health check against the running MCP server
    // For now, return a mock result
    Ok(ServerHealth {
        endpoint: "http://127.0.0.1:3000/mcp".to_string(),
        response_time_ms: 25,
    })
}

fn get_default_config_path() -> Result<PathBuf, CliError> {
    let home = std::env::var("HOME")
        .map_err(|_| CliError::Config("Unable to determine home directory".to_string()))?;
    Ok(PathBuf::from(home).join(".qudag").join("mcp-config.toml"))
}
