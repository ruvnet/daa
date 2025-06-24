//! Configuration structures for the DAA orchestrator

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Orchestrator instance name
    pub name: String,
    
    /// Autonomy loop configuration
    pub autonomy: AutonomyConfig,
    
    /// QuDAG integration configuration
    pub qudag: QuDAGConfig,
    
    /// MCP server configuration
    pub mcp: McpConfig,
    
    /// API server configuration
    pub api: ApiConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            name: "daa-orchestrator".to_string(),
            autonomy: AutonomyConfig::default(),
            qudag: QuDAGConfig::default(),
            mcp: McpConfig::default(),
            api: ApiConfig::default(),
            logging: LoggingConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// Autonomy loop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyConfig {
    /// Whether the autonomy loop is enabled
    pub enabled: bool,
    
    /// Loop iteration interval in milliseconds
    pub loop_interval_ms: u64,
    
    /// Maximum number of tasks to process per iteration
    pub max_tasks_per_iteration: usize,
    
    /// Task timeout in milliseconds
    pub task_timeout_ms: u64,
    
    /// Whether to enable learning from decisions
    pub enable_learning: bool,
    
    /// Rules engine configuration
    pub rules_config: RulesConfig,
    
    /// AI agents configuration
    pub ai_config: AiConfig,
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            loop_interval_ms: 1000, // 1 second
            max_tasks_per_iteration: 10,
            task_timeout_ms: 30000, // 30 seconds
            enable_learning: true,
            rules_config: RulesConfig::default(),
            ai_config: AiConfig::default(),
        }
    }
}

/// Rules engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// Whether rules engine is enabled
    pub enabled: bool,
    
    /// Fail-fast mode for rule evaluation
    pub fail_fast: bool,
    
    /// Maximum daily spending limit
    pub max_daily_spending: f64,
    
    /// Minimum balance requirements
    pub min_balance_threshold: f64,
    
    /// Risk assessment thresholds
    pub max_risk_score: f64,
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fail_fast: false,
            max_daily_spending: 10000.0,
            min_balance_threshold: 100.0,
            max_risk_score: 0.8,
        }
    }
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Whether AI agents are enabled
    pub enabled: bool,
    
    /// Maximum number of concurrent agents
    pub max_agents: usize,
    
    /// Agent task queue size
    pub agent_queue_size: usize,
    
    /// Learning data retention period in days
    pub learning_retention_days: i64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_agents: 5,
            agent_queue_size: 100,
            learning_retention_days: 30,
        }
    }
}

/// QuDAG integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuDAGConfig {
    /// Whether QuDAG integration is enabled
    pub enabled: bool,
    
    /// QuDAG node endpoint
    pub node_endpoint: String,
    
    /// QuDAG network ID
    pub network_id: String,
    
    /// Node ID for this orchestrator
    pub node_id: String,
    
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    
    /// Reconnection attempts
    pub max_reconnection_attempts: usize,
    
    /// Consensus participation
    pub participate_in_consensus: bool,
    
    /// Exchange integration
    pub exchange_config: ExchangeConfig,
}

impl Default for QuDAGConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            node_endpoint: "localhost:7000".to_string(),
            network_id: "qudag-testnet".to_string(),
            node_id: uuid::Uuid::new_v4().to_string(),
            bootstrap_peers: vec![
                "localhost:7001".to_string(),
                "localhost:7002".to_string(),
            ],
            connection_timeout_ms: 10000,
            max_reconnection_attempts: 5,
            participate_in_consensus: true,
            exchange_config: ExchangeConfig::default(),
        }
    }
}

/// Exchange configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    /// Whether exchange integration is enabled
    pub enabled: bool,
    
    /// Exchange endpoint
    pub endpoint: String,
    
    /// Trading pairs to monitor
    pub trading_pairs: Vec<String>,
    
    /// Order book depth
    pub order_book_depth: usize,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            endpoint: "localhost:8080".to_string(),
            trading_pairs: vec!["rUv/USD".to_string()],
            order_book_depth: 20,
        }
    }
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Whether MCP server is enabled
    pub enabled: bool,
    
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    
    /// Whether to enable authentication
    pub enable_auth: bool,
    
    /// API key for authentication (if enabled)
    pub api_key: Option<String>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "0.0.0.0".to_string(),
            port: 3001,
            max_connections: 100,
            request_timeout_ms: 30000,
            enable_auth: false,
            api_key: None,
        }
    }
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Whether API server is enabled
    pub enabled: bool,
    
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    
    /// Whether to enable CORS
    pub enable_cors: bool,
    
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "0.0.0.0".to_string(),
            port: 3000,
            max_connections: 100,
            request_timeout_ms: 30000,
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Whether to log to stdout
    pub stdout: bool,
    
    /// Log file path (optional)
    pub file_path: Option<String>,
    
    /// Whether to enable structured logging (JSON)
    pub structured: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            stdout: true,
            file_path: None,
            structured: false,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check interval in seconds
    pub interval_seconds: u64,
    
    /// Component timeout in milliseconds
    pub component_timeout_ms: u64,
    
    /// Whether to restart failed components
    pub auto_restart: bool,
    
    /// Maximum restart attempts per component
    pub max_restart_attempts: usize,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 30,
            component_timeout_ms: 5000,
            auto_restart: true,
            max_restart_attempts: 3,
        }
    }
}

impl OrchestratorConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: OrchestratorConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate port numbers
        if self.mcp.enabled && self.mcp.port == 0 {
            return Err("MCP server port cannot be 0".to_string());
        }

        if self.api.enabled && self.api.port == 0 {
            return Err("API server port cannot be 0".to_string());
        }

        // Validate timeouts
        if self.autonomy.task_timeout_ms == 0 {
            return Err("Task timeout cannot be 0".to_string());
        }

        if self.qudag.connection_timeout_ms == 0 {
            return Err("QuDAG connection timeout cannot be 0".to_string());
        }

        // Validate node ID
        if self.qudag.enabled && self.qudag.node_id.is_empty() {
            return Err("QuDAG node ID cannot be empty".to_string());
        }

        // Validate autonomy config
        if self.autonomy.max_tasks_per_iteration == 0 {
            return Err("Max tasks per iteration cannot be 0".to_string());
        }

        Ok(())
    }

    /// Get the autonomy loop interval as Duration
    pub fn autonomy_loop_interval(&self) -> Duration {
        Duration::from_millis(self.autonomy.loop_interval_ms)
    }

    /// Get the task timeout as Duration
    pub fn task_timeout(&self) -> Duration {
        Duration::from_millis(self.autonomy.task_timeout_ms)
    }

    /// Get the QuDAG connection timeout as Duration
    pub fn qudag_connection_timeout(&self) -> Duration {
        Duration::from_millis(self.qudag.connection_timeout_ms)
    }

    /// Get the health check interval as Duration
    pub fn health_check_interval(&self) -> Duration {
        Duration::from_secs(self.health_check.interval_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.name, "daa-orchestrator");
        assert!(config.autonomy.enabled);
        assert!(config.qudag.enabled);
        assert!(config.mcp.enabled);
        assert!(config.api.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = OrchestratorConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid port
        config.mcp.port = 0;
        assert!(config.validate().is_err());

        // Fix port and test empty node ID
        config.mcp.port = 3001;
        config.qudag.node_id = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_duration_conversions() {
        let config = OrchestratorConfig::default();
        
        assert_eq!(config.autonomy_loop_interval(), Duration::from_millis(1000));
        assert_eq!(config.task_timeout(), Duration::from_millis(30000));
        assert_eq!(config.qudag_connection_timeout(), Duration::from_millis(10000));
        assert_eq!(config.health_check_interval(), Duration::from_secs(30));
    }

    #[test]
    fn test_config_serialization() {
        let config = OrchestratorConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(!toml_str.is_empty());

        let parsed_config: OrchestratorConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.name, parsed_config.name);
    }
}