use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Node configuration (compatibility with protocol module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Data directory
    pub data_dir: PathBuf,
    /// Network port
    pub network_port: u16,
    /// Maximum peers
    pub max_peers: usize,
    /// Initial peers
    pub initial_peers: Vec<String>,
}

/// Extended node configuration for CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedNodeConfig {
    /// Data directory
    pub data_dir: PathBuf,
    /// Network port
    pub port: u16,
    /// Initial peers
    pub peers: Vec<String>,
    /// Log level
    pub log_level: String,
    /// Node identity
    pub identity: IdentityConfig,
    /// Network configuration
    pub network: NetworkConfig,
}

/// Node identity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// Node ID
    pub node_id: Option<String>,
    /// Private key file
    pub key_file: Option<PathBuf>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: String,
    /// External address
    pub external_addr: Option<String>,
    /// Maximum peers
    pub max_peers: usize,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            network_port: 8000,
            max_peers: 50,
            initial_peers: Vec::new(),
        }
    }
}

impl Default for ExtendedNodeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            port: 8000,
            peers: Vec::new(),
            log_level: "info".to_string(),
            identity: IdentityConfig {
                node_id: None,
                key_file: None,
            },
            network: NetworkConfig {
                listen_addr: "0.0.0.0".to_string(),
                external_addr: None,
                max_peers: 50,
                bootstrap_nodes: Vec::new(),
            },
        }
    }
}

impl NodeConfig {
    /// Load configuration from file
    pub fn load(path: PathBuf) -> Result<Self> {
        let config = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }

    /// Save configuration to file
    pub fn save(&self, path: PathBuf) -> Result<()> {
        let config = toml::to_string_pretty(self)?;
        std::fs::write(path, config)?;
        Ok(())
    }
}

/// Node configuration manager for handling config updates
pub struct NodeConfigManager {
    config_path: PathBuf,
    config: Arc<RwLock<NodeConfig>>,
}

impl NodeConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: PathBuf) -> Result<Self> {
        // Create parent directory if needed
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load or create default config
        let config = if config_path.exists() {
            NodeConfig::load(config_path.clone())?
        } else {
            let default = NodeConfig::default();
            default.save(config_path.clone())?;
            default
        };

        Ok(Self {
            config_path,
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Load current configuration
    pub async fn load_config(&self) -> Result<NodeConfig> {
        Ok(self.config.read().await.clone())
    }

    /// Update configuration with a closure
    pub async fn update_config<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut NodeConfig) -> Result<()>,
    {
        let mut config = self.config.write().await;
        updater(&mut config)?;
        config.save(self.config_path.clone())?;
        info!("Configuration updated and saved");
        Ok(())
    }

    /// Reload configuration from disk
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = NodeConfig::load(self.config_path.clone())?;
        *self.config.write().await = new_config;
        info!("Configuration reloaded from disk");
        Ok(())
    }

    /// Get configuration path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
