//! Protocol configuration implementation.

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    InvalidValue(String),

    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Configuration parsing error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Node configuration
    pub node: NodeConfig,

    /// Network configuration
    pub network: NetworkConfig,

    /// Consensus configuration
    pub consensus: ConsensusConfig,
}

/// Node-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node ID
    pub node_id: String,

    /// Data directory
    pub data_dir: PathBuf,

    /// Log level
    pub log_level: String,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen port
    pub port: u16,

    /// Maximum number of peers
    pub max_peers: usize,

    /// Connection timeout
    pub connect_timeout: Duration,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Finality threshold
    pub finality_threshold: f64,

    /// Round timeout
    pub round_timeout: Duration,

    /// Maximum rounds
    pub max_rounds: usize,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: "node-0".to_string(),
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            max_peers: 50,
            connect_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            finality_threshold: 0.67,
            round_timeout: Duration::from_secs(10),
            max_rounds: 100,
        }
    }
}

impl Config {
    /// Load configuration from file with optional environment variable overrides
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|_| ConfigError::FileNotFound(path.as_ref().display().to_string()))?;

        let mut config: Config =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // Apply environment variable overrides
        config.apply_env_overrides()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from TOML file
    pub fn load_from_toml<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|_| ConfigError::FileNotFound(path.as_ref().display().to_string()))?;

        let mut config: Config =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // Apply environment variable overrides
        config.apply_env_overrides()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Node configuration overrides
        if let Ok(node_id) = env::var("QUDAG_NODE_ID") {
            self.node.node_id = node_id;
        }

        if let Ok(data_dir) = env::var("QUDAG_DATA_DIR") {
            self.node.data_dir = PathBuf::from(data_dir);
        }

        if let Ok(log_level) = env::var("QUDAG_LOG_LEVEL") {
            self.node.log_level = log_level;
        }

        // Network configuration overrides
        if let Ok(port) = env::var("QUDAG_PORT") {
            self.network.port = port
                .parse()
                .map_err(|e| ConfigError::EnvError(format!("Invalid port: {}", e)))?;
        }

        if let Ok(max_peers) = env::var("QUDAG_MAX_PEERS") {
            self.network.max_peers = max_peers
                .parse()
                .map_err(|e| ConfigError::EnvError(format!("Invalid max_peers: {}", e)))?;
        }

        if let Ok(timeout) = env::var("QUDAG_CONNECT_TIMEOUT") {
            let timeout_secs: u64 = timeout
                .parse()
                .map_err(|e| ConfigError::EnvError(format!("Invalid connect_timeout: {}", e)))?;
            self.network.connect_timeout = Duration::from_secs(timeout_secs);
        }

        // Consensus configuration overrides
        if let Ok(threshold) = env::var("QUDAG_FINALITY_THRESHOLD") {
            self.consensus.finality_threshold = threshold
                .parse()
                .map_err(|e| ConfigError::EnvError(format!("Invalid finality_threshold: {}", e)))?;
        }

        if let Ok(timeout) = env::var("QUDAG_ROUND_TIMEOUT") {
            let timeout_secs: u64 = timeout
                .parse()
                .map_err(|e| ConfigError::EnvError(format!("Invalid round_timeout: {}", e)))?;
            self.consensus.round_timeout = Duration::from_secs(timeout_secs);
        }

        if let Ok(max_rounds) = env::var("QUDAG_MAX_ROUNDS") {
            self.consensus.max_rounds = max_rounds
                .parse()
                .map_err(|e| ConfigError::EnvError(format!("Invalid max_rounds: {}", e)))?;
        }

        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate node configuration
        if self.node.node_id.is_empty() {
            return Err(ConfigError::InvalidValue(
                "node_id cannot be empty".to_string(),
            ));
        }

        if self.node.log_level.is_empty() {
            return Err(ConfigError::InvalidValue(
                "log_level cannot be empty".to_string(),
            ));
        }

        // Validate allowed log levels
        match self.node.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(ConfigError::InvalidValue(format!(
                    "Invalid log_level: {}",
                    self.node.log_level
                )))
            }
        }

        // Validate network configuration
        if self.network.port == 0 {
            return Err(ConfigError::InvalidValue("port cannot be 0".to_string()));
        }

        if self.network.max_peers == 0 {
            return Err(ConfigError::InvalidValue(
                "max_peers must be > 0".to_string(),
            ));
        }

        if self.network.max_peers > 10000 {
            return Err(ConfigError::InvalidValue(
                "max_peers must be <= 10000".to_string(),
            ));
        }

        if self.network.connect_timeout.is_zero() {
            return Err(ConfigError::InvalidValue(
                "connect_timeout must be > 0".to_string(),
            ));
        }

        if self.network.connect_timeout > Duration::from_secs(300) {
            return Err(ConfigError::InvalidValue(
                "connect_timeout must be <= 300s".to_string(),
            ));
        }

        // Validate consensus configuration
        if self.consensus.finality_threshold <= 0.0 || self.consensus.finality_threshold > 1.0 {
            return Err(ConfigError::InvalidValue(
                "finality_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.consensus.round_timeout.is_zero() {
            return Err(ConfigError::InvalidValue(
                "round_timeout must be > 0".to_string(),
            ));
        }

        if self.consensus.max_rounds == 0 {
            return Err(ConfigError::InvalidValue(
                "max_rounds must be > 0".to_string(),
            ));
        }

        if self.consensus.max_rounds > 1000 {
            return Err(ConfigError::InvalidValue(
                "max_rounds must be <= 1000".to_string(),
            ));
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Save configuration to TOML file
    pub fn save_to_toml<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::SerializationError(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )))
        })?;
        fs::write(path, content)?;
        Ok(())
    }
}
