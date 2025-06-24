//! Configuration management for QuDAG Exchange CLI

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default wallet address
    pub default_wallet: Option<String>,
    
    /// Node endpoint
    pub node_endpoint: String,
    
    /// Network ID
    pub network_id: String,
    
    /// Transaction settings
    pub transaction: TransactionConfig,
    
    /// Display settings
    pub display: DisplayConfig,
}

/// Transaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    /// Default transaction fee
    pub default_fee: u64,
    
    /// Auto-confirm transactions
    pub auto_confirm: bool,
    
    /// Transaction timeout in seconds
    pub timeout: u64,
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Use colors in output
    pub colors: bool,
    
    /// Show timestamps
    pub timestamps: bool,
    
    /// Decimal places for rUv display
    pub decimal_places: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_wallet: None,
            node_endpoint: "http://localhost:8080".to_string(),
            network_id: "mainnet".to_string(),
            transaction: TransactionConfig {
                default_fee: 1,
                auto_confirm: true,
                timeout: 30,
            },
            display: DisplayConfig {
                colors: true,
                timestamps: true,
                decimal_places: 8,
            },
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get default config path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        let qudag_dir = config_dir.join("qudag-exchange");
        fs::create_dir_all(&qudag_dir)?;
        
        Ok(qudag_dir.join("config.toml"))
    }
}

/// Initialize configuration file
pub fn initialize_config() -> Result<()> {
    let config_path = Config::default_path()?;
    
    if config_path.exists() {
        println!("Configuration file already exists at: {:?}", config_path);
        return Ok(());
    }

    let config = Config::default();
    config.save(config_path.to_str().unwrap())?;
    
    println!("Configuration file created at: {:?}", config_path);
    Ok(())
}