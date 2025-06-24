# QuDAG CLI Architecture Implementation Plan

## Executive Summary

This document outlines a comprehensive implementation plan for the QuDAG CLI command architecture, focusing on building a robust, extensible, and user-friendly command-line interface that seamlessly integrates with the core protocol modules while maintaining high standards for error handling, performance, and testing.

## 1. Overall CLI Architecture Design

### 1.1 Architectural Principles

- **Separation of Concerns**: Clear boundaries between CLI logic, business logic, and protocol interaction
- **Async-First Design**: All operations are async by default to handle network operations efficiently
- **Plugin Architecture**: Extensible command system that allows easy addition of new commands
- **Resource Management**: Proper handling of system resources with explicit lifecycle management
- **Observability**: Comprehensive logging, metrics, and tracing support

### 1.2 Core Components

```
┌─────────────────────────────────────────────────────────────────┐
│                          CLI Entry Point                         │
│                         (main.rs / clap)                        │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│                      Command Router                              │
│                   (command_router.rs)                           │
└────────────────────┬────────────────────────────────────────────┘
                     │
     ┌───────────────┼───────────────┬────────────────┐
     │               │               │                │
┌────▼────┐    ┌────▼────┐    ┌────▼────┐     ┌────▼────┐
│  Node   │    │  Peer   │    │ Network │     │  DAG    │
│Commands │    │Commands │    │Commands │     │Commands │
└────┬────┘    └────┬────┘    └────┬────┘     └────┬────┘
     │               │               │                │
┌────▼───────────────▼───────────────▼────────────────▼────┐
│                    Command Executor                       │
│                  (executor/mod.rs)                       │
└────────────────────┬─────────────────────────────────────┘
                     │
┌────────────────────▼─────────────────────────────────────┐
│                  Protocol Interface                       │
│                    (client/mod.rs)                       │
└────────────────────┬─────────────────────────────────────┘
                     │
     ┌───────────────┼───────────────┬────────────────┐
     │               │               │                │
┌────▼────┐    ┌────▼────┐    ┌────▼────┐     ┌────▼────┐
│Protocol │    │ Network │    │   DAG   │     │ Crypto  │
│  Core   │    │  Layer  │    │  Module │     │ Module  │
└─────────┘    └─────────┘    └─────────┘     └─────────┘
```

### 1.3 Module Structure

```
tools/cli/
├── src/
│   ├── main.rs                    # Entry point and CLI setup
│   ├── lib.rs                     # Public API and error types
│   │
│   ├── commands/                  # Command definitions and parsing
│   │   ├── mod.rs                # Command trait and registry
│   │   ├── node.rs               # Node management commands
│   │   ├── peer.rs               # Peer management commands
│   │   ├── network.rs            # Network operation commands
│   │   ├── dag.rs                # DAG visualization commands
│   │   └── address.rs            # Dark addressing commands
│   │
│   ├── executor/                  # Command execution logic
│   │   ├── mod.rs                # Executor trait and implementation
│   │   ├── node_executor.rs      # Node command execution
│   │   ├── peer_executor.rs      # Peer command execution
│   │   ├── network_executor.rs   # Network command execution
│   │   └── dag_executor.rs       # DAG command execution
│   │
│   ├── client/                    # Protocol client interface
│   │   ├── mod.rs                # Client trait definition
│   │   ├── rpc_client.rs         # RPC-based client implementation
│   │   ├── local_client.rs       # Direct protocol access client
│   │   └── mock_client.rs        # Mock client for testing
│   │
│   ├── output/                    # Output formatting and display
│   │   ├── mod.rs                # Output trait and formatters
│   │   ├── table.rs              # Table formatting
│   │   ├── json.rs               # JSON output
│   │   ├── progress.rs           # Progress indicators
│   │   └── error.rs              # Error display formatting
│   │
│   ├── config/                    # Configuration management
│   │   ├── mod.rs                # Configuration types
│   │   ├── cli_config.rs         # CLI-specific configuration
│   │   ├── node_config.rs        # Node configuration
│   │   └── loader.rs             # Configuration loading
│   │
│   └── utils/                     # Utility functions
│       ├── mod.rs                # Common utilities
│       ├── validation.rs         # Input validation
│       ├── telemetry.rs          # Logging and metrics
│       └── async_utils.rs        # Async helpers
│
└── tests/
    ├── unit/                      # Unit tests
    ├── integration/               # Integration tests
    └── e2e/                       # End-to-end tests
```

## 2. Command Structure and Routing

### 2.1 Command Trait Design

```rust
#[async_trait]
pub trait Command: Send + Sync {
    /// Execute the command with given context
    async fn execute(&self, ctx: &CommandContext) -> Result<CommandOutput>;
    
    /// Validate command arguments before execution
    fn validate(&self) -> Result<()>;
    
    /// Get command metadata for help and documentation
    fn metadata(&self) -> CommandMetadata;
}

pub struct CommandContext {
    pub client: Arc<dyn ProtocolClient>,
    pub config: Arc<CliConfig>,
    pub output: Arc<dyn OutputFormatter>,
    pub telemetry: Arc<TelemetryHandle>,
}

pub struct CommandOutput {
    pub data: serde_json::Value,
    pub format_hint: OutputFormat,
    pub exit_code: i32,
}
```

### 2.2 Command Registry

```rust
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
    aliases: HashMap<String, String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        
        // Register core commands
        registry.register("node", Box::new(NodeCommand::new()));
        registry.register("peer", Box::new(PeerCommand::new()));
        registry.register("network", Box::new(NetworkCommand::new()));
        registry.register("dag", Box::new(DagCommand::new()));
        
        // Register aliases
        registry.alias("status", "node status");
        registry.alias("peers", "peer list");
        
        registry
    }
}
```

### 2.3 Routing Implementation

```rust
pub struct CommandRouter {
    registry: CommandRegistry,
    middleware: Vec<Box<dyn Middleware>>,
}

impl CommandRouter {
    pub async fn route(&self, args: &CliArgs) -> Result<CommandOutput> {
        // Apply middleware chain
        let mut ctx = self.build_context(args).await?;
        for middleware in &self.middleware {
            ctx = middleware.process(ctx).await?;
        }
        
        // Find and execute command
        let command = self.registry.get(&args.command)?;
        command.validate()?;
        
        // Execute with retry and timeout
        let output = timeout(
            Duration::from_secs(args.timeout.unwrap_or(30)),
            command.execute(&ctx)
        ).await??;
        
        Ok(output)
    }
}
```

## 3. Integration with Core Protocol Modules

### 3.1 Protocol Client Interface

```rust
#[async_trait]
pub trait ProtocolClient: Send + Sync {
    // Node operations
    async fn get_node_status(&self) -> Result<NodeStatus>;
    async fn start_node(&self, config: NodeConfig) -> Result<()>;
    async fn stop_node(&self) -> Result<()>;
    
    // Peer operations
    async fn list_peers(&self) -> Result<Vec<PeerInfo>>;
    async fn add_peer(&self, address: NetworkAddress) -> Result<()>;
    async fn remove_peer(&self, peer_id: PeerId) -> Result<()>;
    
    // Network operations
    async fn get_network_stats(&self) -> Result<NetworkStats>;
    async fn test_connectivity(&self, target: Option<PeerId>) -> Result<ConnectivityReport>;
    
    // DAG operations
    async fn get_dag_state(&self) -> Result<DagState>;
    async fn get_dag_visualization(&self) -> Result<DagVisualization>;
}
```

### 3.2 RPC Client Implementation

```rust
pub struct RpcClient {
    endpoint: String,
    client: reqwest::Client,
    auth_token: Option<String>,
}

impl RpcClient {
    pub async fn connect(endpoint: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
            
        Ok(Self {
            endpoint: endpoint.to_string(),
            client,
            auth_token: None,
        })
    }
    
    async fn call<T: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R> {
        let request = RpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params: serde_json::to_value(params)?,
            id: generate_request_id(),
        };
        
        let response = self.client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await?;
            
        let rpc_response: RpcResponse<R> = response.json().await?;
        
        match rpc_response.result {
            Some(result) => Ok(result),
            None => Err(rpc_response.error.unwrap().into()),
        }
    }
}
```

### 3.3 Local Client for Embedded Node

```rust
pub struct LocalClient {
    node: Arc<RwLock<Node>>,
    runtime: Arc<Runtime>,
}

impl LocalClient {
    pub fn new(config: NodeConfig) -> Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        let node = runtime.block_on(async {
            Node::new(config).await
        })?;
        
        Ok(Self {
            node: Arc::new(RwLock::new(node)),
            runtime,
        })
    }
}

#[async_trait]
impl ProtocolClient for LocalClient {
    async fn get_node_status(&self) -> Result<NodeStatus> {
        let node = self.node.read().await;
        Ok(node.status())
    }
    
    // ... other implementations
}
```

## 4. Error Handling and User Feedback

### 4.1 Error Type Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Command error: {0}")]
    Command(#[from] CommandError),
    
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Timeout error: operation timed out after {0} seconds")]
    Timeout(u64),
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    
    #[error("Command not found: {0}")]
    NotFound(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}
```

### 4.2 Error Display and Recovery

```rust
pub struct ErrorDisplay {
    verbosity: Verbosity,
    color: bool,
}

impl ErrorDisplay {
    pub fn format(&self, error: &CliError) -> String {
        match self.verbosity {
            Verbosity::Quiet => self.format_simple(error),
            Verbosity::Normal => self.format_normal(error),
            Verbosity::Verbose => self.format_detailed(error),
        }
    }
    
    fn format_detailed(&self, error: &CliError) -> String {
        let mut output = String::new();
        
        // Main error message
        writeln!(&mut output, "{} {}", 
            self.error_prefix(), 
            error
        ).unwrap();
        
        // Error chain
        let mut current = error.source();
        let mut depth = 1;
        while let Some(err) = current {
            writeln!(&mut output, "  {} Caused by: {}", 
                self.indent(depth), 
                err
            ).unwrap();
            current = err.source();
            depth += 1;
        }
        
        // Suggestions for recovery
        if let Some(suggestion) = self.get_suggestion(error) {
            writeln!(&mut output, "\n{} {}", 
                self.suggestion_prefix(), 
                suggestion
            ).unwrap();
        }
        
        output
    }
}
```

### 4.3 User Feedback Mechanisms

```rust
pub struct ProgressReporter {
    multi_progress: MultiProgress,
    spinners: HashMap<String, ProgressBar>,
}

impl ProgressReporter {
    pub fn start_operation(&self, name: &str, message: &str) -> ProgressHandle {
        let spinner = self.multi_progress.add(
            ProgressBar::new_spinner()
                .with_style(self.spinner_style())
                .with_message(message.to_string())
        );
        
        spinner.enable_steady_tick(Duration::from_millis(100));
        
        ProgressHandle {
            name: name.to_string(),
            bar: spinner,
        }
    }
}

pub struct InteractivePrompt {
    theme: ColorfulTheme,
}

impl InteractivePrompt {
    pub async fn confirm(&self, message: &str) -> Result<bool> {
        let result = Confirm::with_theme(&self.theme)
            .with_prompt(message)
            .default(false)
            .interact()?;
            
        Ok(result)
    }
    
    pub async fn select_peer(&self, peers: Vec<PeerInfo>) -> Result<PeerInfo> {
        let items: Vec<String> = peers.iter()
            .map(|p| format!("{} ({})", p.id, p.address))
            .collect();
            
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Select a peer")
            .items(&items)
            .interact()?;
            
        Ok(peers[selection].clone())
    }
}
```

## 5. Configuration Management

### 5.1 Configuration Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Global CLI settings
    pub global: GlobalConfig,
    
    /// Node-specific settings
    pub node: NodeConfig,
    
    /// Network settings
    pub network: NetworkConfig,
    
    /// Output preferences
    pub output: OutputConfig,
    
    /// Profiles for different environments
    pub profiles: HashMap<String, ProfileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Default RPC endpoint
    pub rpc_endpoint: String,
    
    /// Command timeout in seconds
    pub timeout: u64,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Telemetry settings
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub name: String,
    pub rpc_endpoint: String,
    pub auth_token: Option<String>,
    pub node_config: NodeConfig,
}
```

### 5.2 Configuration Loading

```rust
pub struct ConfigLoader {
    paths: ConfigPaths,
    env_prefix: String,
}

impl ConfigLoader {
    pub async fn load(&self) -> Result<CliConfig> {
        // 1. Load default configuration
        let mut config = self.load_defaults();
        
        // 2. Merge system-wide configuration
        if let Some(system_config) = self.load_system_config().await? {
            config.merge(system_config);
        }
        
        // 3. Merge user configuration
        if let Some(user_config) = self.load_user_config().await? {
            config.merge(user_config);
        }
        
        // 4. Apply environment variables
        self.apply_env_vars(&mut config)?;
        
        // 5. Validate final configuration
        config.validate()?;
        
        Ok(config)
    }
    
    fn load_defaults(&self) -> CliConfig {
        CliConfig {
            global: GlobalConfig {
                rpc_endpoint: "http://localhost:8080/rpc".to_string(),
                timeout: 30,
                retry: RetryConfig {
                    max_attempts: 3,
                    backoff: ExponentialBackoff::default(),
                },
                telemetry: TelemetryConfig {
                    enabled: true,
                    level: Level::INFO,
                },
            },
            // ... other defaults
        }
    }
}
```

### 5.3 Configuration Validation

```rust
impl CliConfig {
    pub fn validate(&self) -> Result<()> {
        // Validate RPC endpoint
        let url = Url::parse(&self.global.rpc_endpoint)
            .map_err(|_| ConfigError::InvalidEndpoint)?;
            
        if !["http", "https"].contains(&url.scheme()) {
            return Err(ConfigError::InvalidScheme);
        }
        
        // Validate timeout
        if self.global.timeout == 0 {
            return Err(ConfigError::InvalidTimeout);
        }
        
        // Validate node configuration
        self.node.validate()?;
        
        // Validate profiles
        for (name, profile) in &self.profiles {
            profile.validate()
                .map_err(|e| ConfigError::InvalidProfile(name.clone(), e))?;
        }
        
        Ok(())
    }
}
```

## 6. Testing Strategy

### 6.1 Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_node_status_command() {
        // Setup mock client
        let mut mock_client = MockProtocolClient::new();
        mock_client
            .expect_get_node_status()
            .times(1)
            .returning(|| Ok(NodeStatus {
                state: NodeState::Running,
                uptime: Duration::from_secs(3600),
                peers: 5,
                messages: 1000,
            }));
            
        // Create command context
        let ctx = CommandContext {
            client: Arc::new(mock_client),
            config: Arc::new(CliConfig::default()),
            output: Arc::new(JsonOutputFormatter::new()),
            telemetry: Arc::new(TelemetryHandle::noop()),
        };
        
        // Execute command
        let cmd = NodeStatusCommand::new();
        let output = cmd.execute(&ctx).await.unwrap();
        
        // Verify output
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.data["state"], "Running");
        assert_eq!(output.data["peers"], 5);
    }
    
    #[test]
    fn test_command_validation() {
        let cmd = AddPeerCommand {
            address: "invalid-address".to_string(),
        };
        
        assert!(cmd.validate().is_err());
    }
}
```

### 6.2 Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use testcontainers::*;
    
    #[tokio::test]
    async fn test_cli_with_real_node() {
        // Start test node
        let node = TestNode::start().await;
        
        // Configure CLI to connect to test node
        let config = CliConfig {
            global: GlobalConfig {
                rpc_endpoint: node.rpc_endpoint(),
                ..Default::default()
            },
            ..Default::default()
        };
        
        // Create CLI instance
        let cli = Cli::new(config);
        
        // Test node status
        let output = cli.execute(&["node", "status"]).await.unwrap();
        assert_eq!(output.exit_code, 0);
        
        // Test peer operations
        let output = cli.execute(&["peer", "add", "127.0.0.1:9000"]).await.unwrap();
        assert_eq!(output.exit_code, 0);
        
        let output = cli.execute(&["peer", "list"]).await.unwrap();
        assert!(output.data["peers"].as_array().unwrap().len() > 0);
    }
}
```

### 6.3 End-to-End Testing

```rust
#[cfg(test)]
mod e2e_tests {
    use super::*;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    
    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.arg("--help");
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("QuDAG Protocol CLI"));
    }
    
    #[test]
    fn test_invalid_command() {
        let mut cmd = Command::cargo_bin("qudag").unwrap();
        cmd.arg("invalid-command");
        
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Command not found"));
    }
    
    #[test]
    fn test_node_status_output_formats() {
        for format in &["json", "table", "yaml"] {
            let mut cmd = Command::cargo_bin("qudag").unwrap();
            cmd.args(&["node", "status", "--format", format]);
            
            cmd.assert().success();
        }
    }
}
```

### 6.4 Property-Based Testing

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_address_validation(address in any::<String>()) {
            let result = validate_network_address(&address);
            
            // Valid addresses should parse correctly
            if let Ok(parsed) = result {
                assert!(parsed.to_string().len() > 0);
            }
        }
        
        #[test]
        fn test_config_serialization(config in arb_cli_config()) {
            // Serialize to TOML
            let toml = toml::to_string(&config).unwrap();
            
            // Deserialize back
            let deserialized: CliConfig = toml::from_str(&toml).unwrap();
            
            // Should be equivalent
            assert_eq!(config, deserialized);
        }
    }
    
    fn arb_cli_config() -> impl Strategy<Value = CliConfig> {
        // Generate arbitrary valid configurations
        (
            any::<String>(),
            1u64..3600,
            any::<bool>(),
        ).prop_map(|(endpoint, timeout, telemetry)| {
            CliConfig {
                global: GlobalConfig {
                    rpc_endpoint: format!("http://{}:8080", endpoint),
                    timeout,
                    telemetry: TelemetryConfig {
                        enabled: telemetry,
                        level: Level::INFO,
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        })
    }
}
```

## 7. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- [ ] Set up basic CLI structure with clap
- [ ] Implement command trait and registry
- [ ] Create basic error types and handling
- [ ] Set up configuration management
- [ ] Implement output formatters (table, json)

### Phase 2: Core Commands (Week 3-4)
- [ ] Implement node management commands (start, stop, status)
- [ ] Implement peer management commands (list, add, remove)
- [ ] Implement network commands (stats, test)
- [ ] Add progress indicators and user feedback

### Phase 3: Protocol Integration (Week 5-6)
- [ ] Design and implement ProtocolClient trait
- [ ] Create RPC client implementation
- [ ] Add local client for embedded node
- [ ] Implement retry and timeout logic

### Phase 4: Advanced Features (Week 7-8)
- [ ] Add DAG visualization commands
- [ ] Implement dark addressing commands
- [ ] Add configuration profiles
- [ ] Implement command aliases and shortcuts

### Phase 5: Testing and Polish (Week 9-10)
- [ ] Complete unit test coverage (>90%)
- [ ] Add integration tests with test nodes
- [ ] Create end-to-end test suite
- [ ] Add property-based tests
- [ ] Performance optimization and profiling

### Phase 6: Documentation and Release (Week 11-12)
- [ ] Write comprehensive user documentation
- [ ] Create developer guide for extending CLI
- [ ] Add interactive tutorials
- [ ] Package and release CLI binaries

## 8. Security Considerations

### 8.1 Authentication and Authorization
- Implement secure token storage using OS keychain
- Support multiple authentication methods (token, certificate)
- Validate all RPC responses for authenticity

### 8.2 Input Validation
- Sanitize all user inputs before processing
- Validate network addresses and peer identifiers
- Prevent command injection attacks

### 8.3 Secure Communication
- Use TLS for all RPC communications
- Implement certificate pinning for known nodes
- Support secure configuration file encryption

## 9. Performance Optimization

### 9.1 Startup Performance
- Lazy load configuration and dependencies
- Use compile-time dependency injection
- Minimize initial allocations

### 9.2 Command Execution
- Implement connection pooling for RPC clients
- Use async/await for all I/O operations
- Cache frequently accessed data

### 9.3 Resource Management
- Implement proper cleanup on shutdown
- Monitor memory usage and prevent leaks
- Use bounded channels for async communication

## 10. Extensibility

### 10.1 Plugin System
- Define plugin API for custom commands
- Support dynamic loading of command modules
- Provide hooks for middleware integration

### 10.2 Custom Output Formats
- Allow registration of custom formatters
- Support template-based output
- Enable output piping and redirection

### 10.3 Scripting Support
- Add batch command execution
- Support command pipelines
- Enable script recording and playback

## Conclusion

This comprehensive plan provides a solid foundation for building a robust, user-friendly, and extensible CLI for the QuDAG protocol. The architecture emphasizes modularity, testability, and performance while maintaining a clean separation between CLI logic and protocol implementation. By following this plan, we can create a CLI that serves both as a powerful tool for node operators and as a reference implementation for integrating with the QuDAG protocol.