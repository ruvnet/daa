//! CLI configuration management

use anyhow::{Context, Result};
use colored::Colorize;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::output::OutputFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default account to use for operations
    pub default_account: Option<String>,
    
    /// Exchange node endpoint
    pub node_endpoint: String,
    
    /// Vault directory path
    pub vault_path: PathBuf,
    
    /// Network configuration
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
    
    /// Connection timeout in seconds
    pub timeout: u64,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_account: None,
            node_endpoint: "http://localhost:8585".to_string(),
            vault_path: Self::default_vault_path(),
            network: NetworkConfig {
                bootstrap_peers: vec![],
                timeout: 30,
            },
        }
    }
}

impl CliConfig {
    /// Get default configuration directory
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "qudag", "exchange")
            .context("Could not determine config directory")?;
        Ok(proj_dirs.config_dir().to_path_buf())
    }
    
    /// Get default configuration file path
    pub fn default_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }
    
    /// Get default vault directory
    pub fn default_vault_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "qudag", "exchange") {
            proj_dirs.data_dir().join("vault")
        } else {
            PathBuf::from(".qudag-vault")
        }
    }
    
    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        toml::from_str(&content)
            .context("Failed to parse config file")
    }
    
    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        fs::write(path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
}

/// Show current configuration
pub fn show(config: &CliConfig, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("{}", "Current Configuration:".green().bold());
            println!();
            println!("Default Account: {}", 
                config.default_account.as_deref().unwrap_or("(none)").yellow());
            println!("Node Endpoint:   {}", config.node_endpoint.cyan());
            println!("Vault Path:      {}", config.vault_path.display());
            println!();
            println!("{}", "Network:".green());
            println!("  Bootstrap Peers: {}", 
                if config.network.bootstrap_peers.is_empty() {
                    "(none)".dimmed().to_string()
                } else {
                    format!("{:?}", config.network.bootstrap_peers)
                });
            println!("  Timeout: {} seconds", config.network.timeout);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(config)?);
        }
    }
    Ok(())
}

/// Initialize configuration
pub fn init(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        println!("{}", "Configuration already exists. Use --force to overwrite.".yellow());
        return Ok(());
    }
    
    let config = CliConfig::default();
    config.save(path)?;
    
    println!("{}", "Configuration initialized successfully!".green().bold());
    println!("Config file: {}", path.display());
    println!();
    println!("Next steps:");
    println!("  1. Run 'qudag-exchange create-account' to create your first account");
    println!("  2. Use 'qudag-exchange config set' to customize settings");
    
    Ok(())
}

/// Set configuration value
pub fn set(path: &Path, key: &str, value: &str) -> Result<()> {
    let mut config = if path.exists() {
        CliConfig::load(path)?
    } else {
        CliConfig::default()
    };
    
    match key {
        "default_account" => config.default_account = Some(value.to_string()),
        "node_endpoint" => config.node_endpoint = value.to_string(),
        "vault_path" => config.vault_path = PathBuf::from(value),
        "network.timeout" => config.network.timeout = value.parse()
            .context("Invalid timeout value")?,
        _ => anyhow::bail!("Unknown configuration key: {}", key),
    }
    
    config.save(path)?;
    println!("{} {} = {}", "Set".green(), key.yellow(), value.cyan());
    
    Ok(())
}