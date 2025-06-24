//! QuDAG MCP Server implementation

use crate::error::{Error, Result};
use crate::protocol::*;
use crate::resources::ResourceRegistry;
use crate::tools::ToolRegistry;
use crate::transport::{Transport, TransportConfig};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub server_info: ServerInfo,
    pub capabilities: ServerCapabilities,
    pub transport: TransportConfig,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_info: ServerInfo::default(),
            capabilities: ServerCapabilities::default(),
            transport: TransportConfig::Stdio,
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_server_info(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.server_info = ServerInfo::new(name, version);
        self
    }

    pub fn with_transport(mut self, transport: TransportConfig) -> Self {
        self.transport = transport;
        self
    }

    pub fn with_log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = level.into();
        self
    }
}

/// Server state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerState {
    Uninitialized,
    Initializing,
    Initialized,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

/// QuDAG MCP Server
pub struct QuDAGMCPServer {
    config: ServerConfig,
    state: Arc<RwLock<ServerState>>,
    transport: Option<Box<dyn Transport>>,
    tool_registry: Arc<ToolRegistry>,
    resource_registry: Arc<ResourceRegistry>,
    active_subscriptions: Arc<RwLock<HashMap<String, Vec<ResourceURI>>>>,
    client_info: Arc<RwLock<Option<ClientInfo>>>,
}

impl QuDAGMCPServer {
    /// Create a new QuDAG MCP server
    pub async fn new(config: ServerConfig) -> Result<Self> {
        // Initialize logging to stderr (only if not already initialized)
        // For stdio transport, all logs MUST go to stderr to avoid interfering with JSON-RPC
        let _ = tracing_subscriber::fmt()
            .with_env_filter(&config.log_level)
            .with_writer(std::io::stderr)
            .try_init();

        info!("Creating QuDAG MCP Server with config: {:?}", config);

        let tool_registry = Arc::new(ToolRegistry::new());
        let resource_registry = Arc::new(ResourceRegistry::new());

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ServerState::Uninitialized)),
            transport: None,
            tool_registry,
            resource_registry,
            active_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            client_info: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the server
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting QuDAG MCP Server");

        *self.state.write().await = ServerState::Initializing;

        // Create transport
        let transport = self.config.transport.create_transport().await?;
        self.transport = Some(transport);

        *self.state.write().await = ServerState::Running;
        info!("QuDAG MCP Server is running");

        // Main message loop
        while self.is_running().await {
            if let Some(transport) = &mut self.transport {
                match transport.receive().await {
                    Ok(Some(message)) => {
                        if let Err(e) = self.handle_message(message).await {
                            error!("Error handling message: {}", e);
                        }
                    }
                    Ok(None) => {
                        info!("No message received, continuing...");
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        error!("Transport error: {}", e);
                        if matches!(e, Error::Transport { transport_type, .. } if transport_type == "connection")
                        {
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }

        *self.state.write().await = ServerState::Stopped;
        info!("QuDAG MCP Server stopped");
        Ok(())
    }

    /// Stop the server
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping QuDAG MCP Server");
        *self.state.write().await = ServerState::Stopping;

        if let Some(transport) = &mut self.transport {
            transport.close().await?;
        }
        self.transport = None;

        *self.state.write().await = ServerState::Stopped;
        Ok(())
    }

    /// Get current server state
    pub async fn state(&self) -> ServerState {
        self.state.read().await.clone()
    }

    /// Check if server is running
    pub async fn is_running(&self) -> bool {
        matches!(
            *self.state.read().await,
            ServerState::Running | ServerState::Initialized
        )
    }

    /// Handle incoming MCP message
    async fn handle_message(&mut self, message: MCPMessage) -> Result<()> {
        info!("Received message: {:?}", message);

        match message {
            MCPMessage::Request(request) => {
                let response = self.handle_request(request).await;
                if let Some(transport) = &mut self.transport {
                    transport.send(MCPMessage::Response(response)).await?;
                }
            }
            MCPMessage::Notification(notification) => {
                self.handle_notification(notification).await?;
            }
            MCPMessage::Response(_) => {
                warn!("Unexpected response message received by server");
            }
        }

        Ok(())
    }

    /// Handle MCP request
    async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        debug!("Handling request: {}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request).await,
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tools_call(&request).await,
            "resources/list" => self.handle_resources_list(&request).await,
            "resources/read" => self.handle_resources_read(&request).await,
            "resources/subscribe" => self.handle_resources_subscribe(&request).await,
            "prompts/list" => self.handle_prompts_list(&request).await,
            "prompts/get" => self.handle_prompts_get(&request).await,
            _ => Err(Error::method_not_found(&request.method)),
        };

        match result {
            Ok(response) => response,
            Err(error) => {
                error!("Request failed: {}", error);
                MCPResponse::error(request.id, error)
            }
        }
    }

    /// Handle MCP notification
    async fn handle_notification(&self, notification: MCPNotification) -> Result<()> {
        debug!("Handling notification: {}", notification.method);

        match notification.method.as_str() {
            "notifications/initialized" => {
                info!("Client initialization complete");
                *self.state.write().await = ServerState::Initialized;
            }
            "notifications/cancelled" => {
                // Handle request cancellation if needed
                debug!("Request cancelled");
            }
            _ => {
                warn!("Unknown notification method: {}", notification.method);
            }
        }

        Ok(())
    }

    /// Handle initialize request
    async fn handle_initialize(&self, request: &MCPRequest) -> Result<MCPResponse> {
        let params: InitializeParams =
            serde_json::from_value(request.params.clone().unwrap_or_default())
                .map_err(|e| Error::invalid_params(e.to_string()))?;

        info!(
            "Initialize request from client: {}",
            params.client_info.name
        );

        // Validate protocol version
        if params.protocol_version != crate::MCP_PROTOCOL_VERSION {
            return Err(Error::unsupported_protocol_version(params.protocol_version));
        }

        // Store client info
        *self.client_info.write().await = Some(params.client_info);

        // Send initialized response
        Ok(MCPResponse::initialize_success(
            request.id.clone(),
            self.config.server_info.clone(),
            self.config.capabilities.clone(),
        ))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, request: &MCPRequest) -> Result<MCPResponse> {
        let tools = self.tool_registry.list_tools().await?;
        info!("Listing {} tools", tools.len());

        Ok(MCPResponse::tools_list_success(request.id.clone(), tools))
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: &MCPRequest) -> Result<MCPResponse> {
        let params: CallToolParams =
            serde_json::from_value(request.params.clone().unwrap_or_default())
                .map_err(|e| Error::invalid_params(e.to_string()))?;

        info!("Calling tool: {}", params.name);

        let tool_name = ToolName::new(params.name);
        let result = self
            .tool_registry
            .call_tool(&tool_name, params.arguments)
            .await?;

        Ok(MCPResponse::tool_call_success(request.id.clone(), result))
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, request: &MCPRequest) -> Result<MCPResponse> {
        let resources = self.resource_registry.list_resources().await?;
        info!("Listing {} resources", resources.len());

        Ok(MCPResponse::resources_list_success(
            request.id.clone(),
            resources,
        ))
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, request: &MCPRequest) -> Result<MCPResponse> {
        let params: ReadResourceParams =
            serde_json::from_value(request.params.clone().unwrap_or_default())
                .map_err(|e| Error::invalid_params(e.to_string()))?;

        info!("Reading resource: {}", params.uri);

        let uri = ResourceURI::new(params.uri);
        let contents = self.resource_registry.read_resource(&uri).await?;

        Ok(MCPResponse::resource_read_success(
            request.id.clone(),
            contents,
        ))
    }

    /// Handle resources/subscribe request
    async fn handle_resources_subscribe(&self, request: &MCPRequest) -> Result<MCPResponse> {
        let params: SubscribeResourceParams =
            serde_json::from_value(request.params.clone().unwrap_or_default())
                .map_err(|e| Error::invalid_params(e.to_string()))?;

        info!("Subscribing to resource: {}", params.uri);

        let uri = ResourceURI::new(params.uri);

        // Store subscription
        let client_id = self
            .client_info
            .read()
            .await
            .as_ref()
            .map(|info| info.name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let mut subscriptions = self.active_subscriptions.write().await;
        subscriptions
            .entry(client_id)
            .or_default()
            .push(uri.clone());

        // Attempt to subscribe with the resource provider
        self.resource_registry.subscribe_to_resource(&uri).await?;

        Ok(MCPResponse::success(
            request.id.clone(),
            serde_json::json!({ "subscribed": true }),
        ))
    }

    /// Handle prompts/list request
    async fn handle_prompts_list(&self, request: &MCPRequest) -> Result<MCPResponse> {
        // Enhanced prompts for common QuDAG workflows
        let prompts = vec![
            Prompt::new("security_audit")
                .with_description("Comprehensive security audit template for QuDAG components")
                .with_arguments(vec![
                    PromptArgument::new("target")
                        .required()
                        .with_description("System component to audit (vault, dag, network, mcp)"),
                    PromptArgument::new("depth")
                        .optional()
                        .with_description("Audit depth level (basic, standard, comprehensive)"),
                    PromptArgument::new("quantum_check")
                        .optional()
                        .with_description("Include quantum security assessment"),
                ]),
            Prompt::new("performance_analysis")
                .with_description("Performance analysis and optimization suggestions")
                .with_arguments(vec![
                    PromptArgument::new("component")
                        .required()
                        .with_description("Component to analyze"),
                    PromptArgument::new("metrics")
                        .optional()
                        .with_description("Specific metrics to focus on"),
                    PromptArgument::new("duration")
                        .optional()
                        .with_description("Analysis time window (e.g., 1h, 24h, 7d)"),
                ]),
            Prompt::new("setup_validator_node")
                .with_description("Step-by-step guide to set up a QuDAG validator node")
                .with_arguments(vec![
                    PromptArgument::new("environment")
                        .required()
                        .with_description("Deployment environment (mainnet, testnet, local)"),
                    PromptArgument::new("stake_amount")
                        .optional()
                        .with_description("Initial stake amount"),
                    PromptArgument::new("region")
                        .optional()
                        .with_description("Geographic region for deployment"),
                ]),
            Prompt::new("vault_migration")
                .with_description("Guide for migrating passwords to QuDAG vault")
                .with_arguments(vec![
                    PromptArgument::new("source_format")
                        .required()
                        .with_description("Source password manager format"),
                    PromptArgument::new("categories")
                        .optional()
                        .with_description("Categories to migrate"),
                    PromptArgument::new("encryption_upgrade")
                        .optional()
                        .with_description("Upgrade to quantum-resistant encryption"),
                ]),
            Prompt::new("dag_transaction_builder")
                .with_description("Interactive DAG transaction builder")
                .with_arguments(vec![
                    PromptArgument::new("transaction_type")
                        .required()
                        .with_description("Type of transaction (transfer, stake, contract)"),
                    PromptArgument::new("priority")
                        .optional()
                        .with_description("Transaction priority level"),
                    PromptArgument::new("quantum_sign")
                        .optional()
                        .with_description("Use quantum-resistant signatures"),
                ]),
            Prompt::new("network_diagnostics")
                .with_description("Diagnose and troubleshoot network connectivity issues")
                .with_arguments(vec![
                    PromptArgument::new("issue_type")
                        .optional()
                        .with_description("Type of issue (connectivity, sync, performance)"),
                    PromptArgument::new("peer_id")
                        .optional()
                        .with_description("Specific peer to diagnose"),
                    PromptArgument::new("verbose")
                        .optional()
                        .with_description("Enable verbose diagnostics"),
                ]),
            Prompt::new("quantum_readiness_check")
                .with_description("Assess quantum readiness of your QuDAG deployment")
                .with_arguments(vec![
                    PromptArgument::new("components")
                        .optional()
                        .with_description("Components to check (all, crypto, network, storage)"),
                    PromptArgument::new("upgrade_path")
                        .optional()
                        .with_description("Show upgrade recommendations"),
                ]),
            Prompt::new("backup_and_recovery")
                .with_description("Create and manage backups for QuDAG data")
                .with_arguments(vec![
                    PromptArgument::new("operation")
                        .required()
                        .with_description("Operation type (backup, restore, verify)"),
                    PromptArgument::new("components")
                        .optional()
                        .with_description("Components to backup (vault, dag, config, all)"),
                    PromptArgument::new("encryption")
                        .optional()
                        .with_description("Encrypt backup with quantum-resistant algorithm"),
                ]),
            Prompt::new("smart_contract_deploy")
                .with_description("Deploy smart contracts to QuDAG network")
                .with_arguments(vec![
                    PromptArgument::new("contract_type")
                        .required()
                        .with_description("Contract type (defi, nft, dao, custom)"),
                    PromptArgument::new("network")
                        .optional()
                        .with_description("Target network (mainnet, testnet)"),
                    PromptArgument::new("audit_level")
                        .optional()
                        .with_description("Security audit requirements"),
                ]),
            Prompt::new("monitoring_setup")
                .with_description("Set up comprehensive monitoring for QuDAG services")
                .with_arguments(vec![
                    PromptArgument::new("services")
                        .optional()
                        .with_description("Services to monitor (all, mcp, dag, vault)"),
                    PromptArgument::new("alerting")
                        .optional()
                        .with_description("Configure alerting rules"),
                    PromptArgument::new("metrics_retention")
                        .optional()
                        .with_description("Metrics retention period"),
                ]),
        ];

        Ok(MCPResponse::prompts_list_success(
            request.id.clone(),
            prompts,
        ))
    }

    /// Handle prompts/get request
    async fn handle_prompts_get(&self, request: &MCPRequest) -> Result<MCPResponse> {
        let params: GetPromptParams =
            serde_json::from_value(request.params.clone().unwrap_or_default())
                .map_err(|e| Error::invalid_params(e.to_string()))?;

        info!("Getting prompt: {}", params.name);

        // Enhanced prompt templates
        let messages = match params.name.as_str() {
            "security_audit" => {
                let default_target = "system".to_string();
                let default_depth = "standard".to_string();
                let target = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("target"))
                    .unwrap_or(&default_target);
                let depth = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("depth"))
                    .unwrap_or(&default_depth);
                let quantum_check = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("quantum_check"))
                    .map(|v| v == "true")
                    .unwrap_or(false);

                vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: MessageContent::Text {
                            text: "You are a security expert specializing in distributed systems and quantum-resistant cryptography. You have deep knowledge of QuDAG's architecture, including its DAG consensus, vault security, and network protocols.".to_string()
                        },
                    },
                    PromptMessage {
                        role: MessageRole::User,
                        content: MessageContent::Text {
                            text: format!("Please conduct a {} security audit of the {} component. {}Focus on:\n1. Potential vulnerabilities and attack vectors\n2. Authentication and authorization mechanisms\n3. Data encryption and integrity\n4. Network security and peer validation\n5. Best practices and compliance\n6. Specific recommendations for improvement\n\nProvide your findings in a structured format with severity levels.", 
                                depth,
                                target,
                                if quantum_check { "Include quantum security assessment, focusing on post-quantum cryptography readiness. " } else { "" }
                            )
                        },
                    }
                ]
            }
            "performance_analysis" => {
                let default_component = "system".to_string();
                let component = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("component"))
                    .unwrap_or(&default_component);

                vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: MessageContent::Text {
                            text: "You are a performance optimization expert.".to_string()
                        },
                    },
                    PromptMessage {
                        role: MessageRole::User,
                        content: MessageContent::Text {
                            text: format!("Analyze the performance of the {} component and provide optimization recommendations.", component)
                        },
                    }
                ]
            }
            "setup_validator_node" => {
                let default_env = "testnet".to_string();
                let environment = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("environment"))
                    .unwrap_or(&default_env);
                let stake_amount = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("stake_amount"))
                    .map(|s| s.clone())
                    .unwrap_or_else(|| "100000".to_string());

                vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: MessageContent::Text {
                            text: "You are a QuDAG infrastructure expert who helps users set up and configure validator nodes. You understand the technical requirements, staking mechanisms, and best practices for running secure, high-performance validators.".to_string()
                        },
                    },
                    PromptMessage {
                        role: MessageRole::User,
                        content: MessageContent::Text {
                            text: format!("I want to set up a QuDAG validator node on {}. I plan to stake {} tokens. Please provide:\n1. System requirements and recommendations\n2. Step-by-step installation guide\n3. Configuration for optimal performance\n4. Security hardening steps\n5. Monitoring and maintenance procedures\n6. Staking process and rewards information\n7. Backup and disaster recovery setup", 
                                environment, stake_amount)
                        },
                    }
                ]
            }
            "vault_migration" => {
                let default_source = "generic".to_string();
                let source_format = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("source_format"))
                    .unwrap_or(&default_source);
                let encryption_upgrade = params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("encryption_upgrade"))
                    .map(|v| v == "true")
                    .unwrap_or(true);

                vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: MessageContent::Text {
                            text: "You are a password migration specialist with expertise in QuDAG's quantum-resistant vault. You understand various password manager formats and can guide users through secure migration processes.".to_string()
                        },
                    },
                    PromptMessage {
                        role: MessageRole::User,
                        content: MessageContent::Text {
                            text: format!("I need to migrate my passwords from {} to QuDAG vault. {}Please provide:\n1. Export process from the source system\n2. Data preparation and formatting requirements\n3. Import process into QuDAG vault\n4. Security considerations during migration\n5. Verification steps after migration\n6. Best practices for organizing entries\n7. Cleanup of source data", 
                                source_format,
                                if encryption_upgrade { "I want to upgrade to quantum-resistant encryption. " } else { "" }
                            )
                        },
                    }
                ]
            }
            "quantum_readiness_check" => {
                vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: MessageContent::Text {
                            text: "You are a quantum computing security expert specializing in post-quantum cryptography. You understand QuDAG's quantum-resistant features and can assess system readiness for the quantum era.".to_string()
                        },
                    },
                    PromptMessage {
                        role: MessageRole::User,
                        content: MessageContent::Text {
                            text: "Please perform a comprehensive quantum readiness assessment of my QuDAG deployment. Check:\n1. Current cryptographic algorithms in use\n2. Quantum vulnerability assessment\n3. ML-KEM and ML-DSA implementation status\n4. Key sizes and security levels\n5. Migration path to quantum-resistant algorithms\n6. Performance impact analysis\n7. Timeline recommendations\n8. Best practices for quantum security".to_string()
                        },
                    }
                ]
            }
            _ => {
                return Err(Error::prompt_not_found(params.name));
            }
        };

        Ok(MCPResponse::prompt_get_success(
            request.id.clone(),
            Some("Generated prompt for QuDAG system analysis".to_string()),
            messages,
        ))
    }

    /// Send notification to client
    pub async fn send_notification(&mut self, notification: MCPNotification) -> Result<()> {
        if let Some(transport) = &mut self.transport {
            transport
                .send(MCPMessage::Notification(notification))
                .await?;
        }
        Ok(())
    }

    /// Notify clients of tool list changes
    pub async fn notify_tools_changed(&mut self) -> Result<()> {
        self.send_notification(MCPNotification::tools_list_changed())
            .await
    }

    /// Notify clients of resource list changes
    pub async fn notify_resources_changed(&mut self) -> Result<()> {
        self.send_notification(MCPNotification::resources_list_changed())
            .await
    }

    /// Notify clients of resource updates
    pub async fn notify_resource_updated(&mut self, uri: impl Into<String>) -> Result<()> {
        self.send_notification(MCPNotification::resource_updated(uri))
            .await
    }

    /// Get server statistics
    pub async fn stats(&self) -> ServerStats {
        let tools_count = self
            .tool_registry
            .list_tools()
            .await
            .map(|t| t.len())
            .unwrap_or(0);
        let resources_count = self
            .resource_registry
            .list_resources()
            .await
            .map(|r| r.len())
            .unwrap_or(0);
        let subscriptions_count = self
            .active_subscriptions
            .read()
            .await
            .values()
            .map(|subs| subs.len())
            .sum();

        ServerStats {
            state: self.state().await,
            tools_count,
            resources_count,
            active_subscriptions: subscriptions_count,
            client_connected: self.client_info.read().await.is_some(),
        }
    }
}

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub state: ServerState,
    pub tools_count: usize,
    pub resources_count: usize,
    pub active_subscriptions: usize,
    pub client_connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportFactory;

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::new();
        let server = QuDAGMCPServer::new(config).await.unwrap();

        assert!(matches!(server.state().await, ServerState::Uninitialized));
    }

    #[tokio::test]
    async fn test_server_config() {
        let config = ServerConfig::new()
            .with_server_info("Test Server", "1.0.0")
            .with_transport(TransportFactory::stdio())
            .with_log_level("debug");

        assert_eq!(config.server_info.name, "Test Server");
        assert_eq!(config.server_info.version, "1.0.0");
        assert_eq!(config.log_level, "debug");
    }

    #[tokio::test]
    async fn test_server_stats() {
        let config = ServerConfig::new();
        let server = QuDAGMCPServer::new(config).await.unwrap();

        let stats = server.stats().await;
        assert!(stats.tools_count > 0); // Should have some tools registered
        assert!(stats.resources_count > 0); // Should have some resources registered
        assert!(!stats.client_connected); // No client connected initially
    }

    #[tokio::test]
    async fn test_initialize_request_handling() {
        let config = ServerConfig::new();
        let server = QuDAGMCPServer::new(config).await.unwrap();

        let request =
            MCPRequest::initialize(ClientInfo::new("test-client", "1.0.0"), HashMap::new());

        let response = server.handle_initialize(&request).await.unwrap();
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_tools_list_request_handling() {
        let config = ServerConfig::new();
        let server = QuDAGMCPServer::new(config).await.unwrap();

        let request = MCPRequest::list_tools();
        let response = server.handle_tools_list(&request).await.unwrap();

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["tools"].is_array());
    }

    #[tokio::test]
    async fn test_resources_list_request_handling() {
        let config = ServerConfig::new();
        let server = QuDAGMCPServer::new(config).await.unwrap();

        let request = MCPRequest::list_resources();
        let response = server.handle_resources_list(&request).await.unwrap();

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["resources"].is_array());
    }
}
