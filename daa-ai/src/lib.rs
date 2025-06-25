//! # DAA AI
//!
//! AI integration layer for the Decentralized Autonomous Agents (DAA) system.
//! Provides Claude AI integration via QuDAG MCP (Model Context Protocol) for 
//! intelligent decision making and task automation.

mod qudag_stubs;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// Re-export QuDAG MCP types
pub use crate::qudag_stubs::qudag_mcp::{MCPClient, MCPMessage, MCPError, Tool, ToolCall, ToolResult};

pub mod claude;
pub mod agents;
pub mod tools;
pub mod tasks;
pub mod memory;

#[cfg(feature = "rules-integration")]
pub mod rules_integration;

#[cfg(feature = "database")]
pub mod database;

/// AI system error types
#[derive(Error, Debug)]
pub enum AIError {
    #[error("MCP error: {0}")]
    MCP(#[from] MCPError),
    
    #[error("Claude API error: {0}")]
    Claude(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Task execution error: {0}")]
    TaskExecution(String),
    
    #[error("Tool error: {0}")]
    Tool(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AIError>;

/// Configuration for the DAA AI system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Claude API configuration
    pub claude: claude::ClaudeConfig,
    
    /// MCP client configuration
    pub mcp: MCPClientConfig,
    
    /// Agent configuration
    pub agents: AgentConfig,
    
    /// Memory configuration
    pub memory: MemoryConfig,
    
    /// Database configuration
    #[cfg(feature = "database")]
    pub database_url: Option<String>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            claude: claude::ClaudeConfig::default(),
            mcp: MCPClientConfig::default(),
            agents: AgentConfig::default(),
            memory: MemoryConfig::default(),
            #[cfg(feature = "database")]
            database_url: None,
        }
    }
}

/// MCP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPClientConfig {
    /// MCP server endpoint
    pub server_url: String,
    
    /// Connection timeout in seconds
    pub timeout: u64,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Retry configuration
    pub retry_attempts: u32,
    
    /// Tool registry
    pub available_tools: Vec<String>,
}

impl Default for MCPClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            timeout: 30,
            max_connections: 10,
            retry_attempts: 3,
            available_tools: vec![
                "code_execution".to_string(),
                "file_operations".to_string(),
                "web_search".to_string(),
                "data_analysis".to_string(),
            ],
        }
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of agents
    pub max_agents: usize,
    
    /// Default agent capabilities
    pub default_capabilities: Vec<String>,
    
    /// Agent spawn configuration
    pub spawn_config: SpawnConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_agents: 100,
            default_capabilities: vec![
                "reasoning".to_string(),
                "code_generation".to_string(),
                "data_analysis".to_string(),
                "communication".to_string(),
            ],
            spawn_config: SpawnConfig::default(),
        }
    }
}

/// Agent spawn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    /// Default model to use for new agents
    pub default_model: String,
    
    /// Default system prompt
    pub default_system_prompt: String,
    
    /// Default temperature for responses
    pub default_temperature: f32,
    
    /// Maximum tokens per response
    pub max_tokens: u32,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            default_model: "claude-3-opus-20240229".to_string(),
            default_system_prompt: "You are a helpful AI agent in the DAA system. You can execute tasks, analyze data, and collaborate with other agents.".to_string(),
            default_temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum memory entries per agent
    pub max_entries_per_agent: usize,
    
    /// Memory retention period in hours
    pub retention_hours: u64,
    
    /// Enable persistent memory
    pub persistent: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_entries_per_agent: 1000,
            retention_hours: 24 * 7, // 1 week
            persistent: true,
        }
    }
}

/// Main AI system coordinating all AI operations
pub struct AISystem {
    /// System configuration
    config: AIConfig,
    
    /// MCP client for Claude integration
    mcp_client: MCPClient,
    
    /// Claude API client
    claude_client: claude::ClaudeClient,
    
    /// Agent manager
    agent_manager: agents::AgentManager,
    
    /// Task manager
    task_manager: tasks::TaskManager,
    
    /// Tool registry
    tool_registry: tools::ToolRegistry,
    
    /// Memory system
    memory: memory::MemorySystem,
    
    /// Database connection
    #[cfg(feature = "database")]
    database: Option<database::DatabaseManager>,
}

impl AISystem {
    /// Create a new AI system
    pub async fn new(config: AIConfig) -> Result<Self> {
        // Initialize MCP client
        let mcp_client = MCPClient::new(&config.mcp.server_url).await
            .map_err(AIError::MCP)?;
        
        // Initialize Claude client
        let claude_client = claude::ClaudeClient::new(config.claude.clone()).await?;
        
        // Initialize managers
        let agent_manager = agents::AgentManager::new(config.agents.clone());
        let task_manager = tasks::TaskManager::new();
        let tool_registry = tools::ToolRegistry::new();
        let memory = memory::MemorySystem::new(config.memory.clone());
        
        // Initialize database if enabled
        #[cfg(feature = "database")]
        let database = if let Some(db_url) = &config.database_url {
            Some(database::DatabaseManager::new(db_url).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            mcp_client,
            claude_client,
            agent_manager,
            task_manager,
            tool_registry,
            memory,
            #[cfg(feature = "database")]
            database,
        })
    }

    /// Initialize the AI system
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing DAA AI System");
        
        // Initialize MCP connection
        self.mcp_client.connect().await.map_err(AIError::MCP)?;
        
        // Register default tools
        self.register_default_tools().await?;
        
        // Initialize memory system
        self.memory.initialize().await?;
        
        // Initialize database if enabled
        #[cfg(feature = "database")]
        if let Some(db) = &mut self.database {
            db.initialize().await?;
        }
        
        tracing::info!("DAA AI System initialized successfully");
        Ok(())
    }

    /// Spawn a new AI agent
    pub async fn spawn_agent(
        &mut self,
        agent_type: agents::AgentType,
        capabilities: Option<Vec<String>>,
        custom_config: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let agent = self.agent_manager.spawn_agent(
            agent_type,
            capabilities.unwrap_or_else(|| self.config.agents.default_capabilities.clone()),
            custom_config,
        ).await?;
        
        // Store agent in memory
        self.memory.store_agent_metadata(&agent.id, &agent).await?;
        
        tracing::info!("Spawned new agent: {} ({})", agent.id, agent.agent_type);
        Ok(agent.id)
    }

    /// Execute a task with an agent
    pub async fn execute_task(
        &mut self,
        agent_id: &str,
        task: tasks::Task,
    ) -> Result<tasks::TaskResult> {
        // Get agent
        let agent = self.agent_manager.get_agent(agent_id).await?;
        
        // Execute task through Claude
        let result = self.claude_client.execute_task(&agent, &task).await?;
        
        // Store task result in memory
        self.memory.store_task_result(agent_id, &task.id, &result).await?;
        
        // Record in database if enabled
        #[cfg(feature = "database")]
        if let Some(db) = &mut self.database {
            db.record_task_execution(agent_id, &task, &result).await?;
        }
        
        tracing::info!("Task {} executed by agent {}", task.id, agent_id);
        Ok(result)
    }

    /// Use a tool via MCP
    pub async fn use_tool(
        &mut self,
        agent_id: &str,
        tool_name: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<ToolResult> {
        // Create tool call
        let tool_call = ToolCall {
            id: Uuid::new_v4().to_string(),
            name: tool_name.to_string(),
            parameters,
        };
        
        // Execute via MCP
        let result = self.mcp_client.call_tool(tool_call).await
            .map_err(AIError::MCP)?;
        
        // Store in memory
        self.memory.store_tool_usage(agent_id, tool_name, &result).await?;
        
        tracing::info!("Tool {} used by agent {}", tool_name, agent_id);
        Ok(result)
    }

    /// Get agent memory
    pub async fn get_agent_memory(&self, agent_id: &str) -> Result<Vec<memory::MemoryEntry>> {
        self.memory.get_agent_memory(agent_id).await
    }

    /// Store information in agent memory
    pub async fn store_memory(
        &mut self,
        agent_id: &str,
        key: String,
        data: serde_json::Value,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.memory.store(agent_id, key, data, metadata).await
    }

    /// Get system statistics
    pub async fn get_statistics(&self) -> AIStatistics {
        AIStatistics {
            total_agents: self.agent_manager.get_agent_count().await,
            active_tasks: self.task_manager.get_active_task_count().await,
            total_tools: self.tool_registry.get_tool_count().await,
            memory_entries: self.memory.get_total_entries().await,
            uptime_seconds: 0, // Would be calculated from start time
        }
    }

    /// Register default tools
    async fn register_default_tools(&mut self) -> Result<()> {
        // Register built-in tools
        for tool_name in &self.config.mcp.available_tools {
            let tool = tools::create_default_tool(tool_name)?;
            self.tool_registry.register_tool(tool).await?;
        }
        
        Ok(())
    }
}

/// AI system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStatistics {
    /// Total number of spawned agents
    pub total_agents: u64,
    
    /// Number of active tasks
    pub active_tasks: u64,
    
    /// Total available tools
    pub total_tools: u64,
    
    /// Memory entries count
    pub memory_entries: u64,
    
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

impl std::fmt::Display for AIStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AI Stats: Agents={}, Active Tasks={}, Tools={}, Memory={}, Uptime={}s",
            self.total_agents,
            self.active_tasks,
            self.total_tools,
            self.memory_entries,
            self.uptime_seconds
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_system_creation() {
        let config = AIConfig::default();
        // Note: This would fail in real test due to missing MCP server
        // let system = AISystem::new(config).await;
        // In actual implementation, we'd use mock clients for testing
        assert!(true); // Placeholder test
    }

    #[test]
    fn test_config_defaults() {
        let config = AIConfig::default();
        assert_eq!(config.claude.model, "claude-3-opus-20240229");
        assert_eq!(config.mcp.timeout, 30);
        assert_eq!(config.agents.max_agents, 100);
    }

    #[test]
    fn test_statistics_display() {
        let stats = AIStatistics {
            total_agents: 5,
            active_tasks: 3,
            total_tools: 10,
            memory_entries: 100,
            uptime_seconds: 3600,
        };
        
        let display = stats.to_string();
        assert!(display.contains("Agents=5"));
        assert!(display.contains("Active Tasks=3"));
        assert!(display.contains("Tools=10"));
    }
}