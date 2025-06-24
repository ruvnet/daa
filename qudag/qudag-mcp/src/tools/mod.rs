//! MCP tools implementation for QuDAG operations

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub mod config;
pub mod crypto;
pub mod dag;
pub mod exchange;
pub mod network;
pub mod system;
pub mod vault;

pub use config::ConfigTool;
pub use crypto::CryptoTool;
pub use dag::DagTool;
pub use exchange::ExchangeTool;
pub use network::NetworkTool;
pub use system::SystemTool;
pub use vault::VaultTool;

use crate::error::{Error, Result};
use crate::types::{Tool, ToolName, ToolResult, ToolResultContent};

/// Tool registry for managing available tools
#[derive(Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn McpTool>>,
}

impl ToolRegistry {
    /// Create new tool registry
    pub fn new() -> Self {
        tracing::info!("Creating new tool registry");
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register built-in tools with their proper names
        let vault_tool = Arc::new(VaultTool::new());
        let dag_tool = Arc::new(DagTool::new());
        let network_tool = Arc::new(NetworkTool::new());
        let crypto_tool = Arc::new(CryptoTool::new());
        let system_tool = Arc::new(SystemTool::new());
        let config_tool = Arc::new(ConfigTool::new());
        let exchange_tool = Arc::new(ExchangeTool::new());

        tracing::info!("Registering vault tool: {}", vault_tool.name());
        registry.register(vault_tool.name(), vault_tool.clone());

        tracing::info!("Registering dag tool: {}", dag_tool.name());
        registry.register(dag_tool.name(), dag_tool.clone());

        tracing::info!("Registering network tool: {}", network_tool.name());
        registry.register(network_tool.name(), network_tool.clone());

        tracing::info!("Registering crypto tool: {}", crypto_tool.name());
        registry.register(crypto_tool.name(), crypto_tool.clone());

        tracing::info!("Registering system tool: {}", system_tool.name());
        registry.register(system_tool.name(), system_tool.clone());

        tracing::info!("Registering config tool: {}", config_tool.name());
        registry.register(config_tool.name(), config_tool.clone());

        tracing::info!("Registering exchange tool: {}", exchange_tool.name());
        registry.register(exchange_tool.name(), exchange_tool.clone());

        tracing::info!("Tool registry created with {} tools", registry.tools.len());
        registry
    }

    /// Register a tool
    pub fn register(&mut self, name: &str, tool: Arc<dyn McpTool>) {
        self.tools.insert(name.to_string(), tool);
    }

    /// Get all available tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let mut tools = Vec::new();
        for (name, tool) in &self.tools {
            tracing::info!("Listing tool: {}", name);
            let tool_def = tool.definition();
            tracing::debug!("Tool definition: {:?}", tool_def);
            tools.push(tool_def);
        }
        tracing::info!("Total tools available: {}", tools.len());
        Ok(tools)
    }

    /// Call a tool
    pub async fn call_tool(
        &self,
        name: &ToolName,
        arguments: Option<serde_json::Value>,
    ) -> Result<ToolResult> {
        if let Some(tool) = self.tools.get(name.as_str()) {
            let result = tool.execute(arguments).await?;
            Ok(ToolResult {
                content: vec![ToolResultContent::Text {
                    text: result.to_string(),
                }],
                is_error: Some(false),
            })
        } else {
            Err(Error::tool_not_found(name.as_str()))
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for MCP tools
#[async_trait]
pub trait McpTool {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Get the tool description
    fn description(&self) -> &str;

    /// Get the tool input schema
    fn input_schema(&self) -> Value;

    /// Get the tool definition
    fn definition(&self) -> Tool {
        Tool {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.input_schema(),
        }
    }

    /// Execute the tool with given arguments
    async fn execute(&self, arguments: Option<Value>) -> Result<Value>;

    /// Validate tool arguments
    fn validate_arguments(&self, arguments: Option<&Value>) -> Result<()> {
        // Default implementation - can be overridden
        if arguments.is_none() {
            return Ok(());
        }
        Ok(())
    }

    /// Get tool metadata
    fn metadata(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// Helper function to validate required argument
pub fn get_required_arg<'a>(args: &'a Value, key: &str) -> Result<&'a Value> {
    args.get(key)
        .ok_or_else(|| Error::invalid_request(format!("Missing required argument: {}", key)))
}

/// Helper function to get optional string argument
pub fn get_optional_string_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Helper function to get required string argument
pub fn get_required_string_arg(args: &Value, key: &str) -> Result<String> {
    get_required_arg(args, key)?
        .as_str()
        .ok_or_else(|| Error::invalid_request(format!("Argument '{}' must be a string", key)))
        .map(|s| s.to_string())
}

/// Helper function to get optional boolean argument
pub fn get_optional_bool_arg(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| v.as_bool())
}

/// Helper function to get optional number argument
pub fn get_optional_u64_arg(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(|v| v.as_u64())
}
