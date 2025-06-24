//! CLI configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{Cli, ConfigAction};

/// CLI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Orchestrator configuration file path
    pub orchestrator_config: Option<PathBuf>,
    
    /// Default output format
    pub default_output_format: OutputFormat,
    
    /// Connection settings
    pub connection: ConnectionConfig,
    
    /// Display preferences
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Human,
    Json,
    Yaml,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Default API endpoint
    pub api_endpoint: String,
    
    /// Default MCP endpoint
    pub mcp_endpoint: String,
    
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    
    /// Retry attempts
    pub retry_attempts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Enable colored output by default
    pub colored: bool,
    
    /// Default page size for lists
    pub page_size: usize,
    
    /// Show timestamps in output
    pub show_timestamps: bool,
    
    /// Compact output mode
    pub compact: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            orchestrator_config: None,
            default_output_format: OutputFormat::Human,
            connection: ConnectionConfig {
                api_endpoint: "http://localhost:3000".to_string(),
                mcp_endpoint: "http://localhost:3001".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
            display: DisplayConfig {
                colored: true,
                page_size: 20,
                show_timestamps: true,
                compact: false,
            },
        }
    }
}

impl CliConfig {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let config: CliConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;
        
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate timeout
        if self.connection.timeout_seconds == 0 {
            anyhow::bail!("Connection timeout cannot be 0");
        }

        // Validate retry attempts
        if self.connection.retry_attempts == 0 {
            anyhow::bail!("Retry attempts cannot be 0");
        }

        // Validate page size
        if self.display.page_size == 0 {
            anyhow::bail!("Page size cannot be 0");
        }

        Ok(())
    }

    /// Get a configuration value by key (dot notation)
    pub fn get_value(&self, key: &str) -> Result<String> {
        match key {
            "orchestrator_config" => Ok(self.orchestrator_config
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "null".to_string())),
            "default_output_format" => Ok(format!("{:?}", self.default_output_format)),
            "connection.api_endpoint" => Ok(self.connection.api_endpoint.clone()),
            "connection.mcp_endpoint" => Ok(self.connection.mcp_endpoint.clone()),
            "connection.timeout_seconds" => Ok(self.connection.timeout_seconds.to_string()),
            "connection.retry_attempts" => Ok(self.connection.retry_attempts.to_string()),
            "display.colored" => Ok(self.display.colored.to_string()),
            "display.page_size" => Ok(self.display.page_size.to_string()),
            "display.show_timestamps" => Ok(self.display.show_timestamps.to_string()),
            "display.compact" => Ok(self.display.compact.to_string()),
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }
    }

    /// Set a configuration value by key (dot notation)
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "orchestrator_config" => {
                self.orchestrator_config = if value == "null" {
                    None
                } else {
                    Some(PathBuf::from(value))
                };
            }
            "default_output_format" => {
                self.default_output_format = match value.to_lowercase().as_str() {
                    "human" => OutputFormat::Human,
                    "json" => OutputFormat::Json,
                    "yaml" => OutputFormat::Yaml,
                    "table" => OutputFormat::Table,
                    _ => anyhow::bail!("Invalid output format: {}", value),
                };
            }
            "connection.api_endpoint" => {
                self.connection.api_endpoint = value.to_string();
            }
            "connection.mcp_endpoint" => {
                self.connection.mcp_endpoint = value.to_string();
            }
            "connection.timeout_seconds" => {
                self.connection.timeout_seconds = value.parse()
                    .with_context(|| format!("Invalid timeout value: {}", value))?;
            }
            "connection.retry_attempts" => {
                self.connection.retry_attempts = value.parse()
                    .with_context(|| format!("Invalid retry attempts value: {}", value))?;
            }
            "display.colored" => {
                self.display.colored = value.parse()
                    .with_context(|| format!("Invalid boolean value: {}", value))?;
            }
            "display.page_size" => {
                self.display.page_size = value.parse()
                    .with_context(|| format!("Invalid page size value: {}", value))?;
            }
            "display.show_timestamps" => {
                self.display.show_timestamps = value.parse()
                    .with_context(|| format!("Invalid boolean value: {}", value))?;
            }
            "display.compact" => {
                self.display.compact = value.parse()
                    .with_context(|| format!("Invalid boolean value: {}", value))?;
            }
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }
        Ok(())
    }
}

/// Handle config command
pub async fn handle_config(action: ConfigAction, config: &CliConfig, cli: &Cli) -> Result<()> {
    match action {
        ConfigAction::Show => {
            if cli.json {
                println!("{}", serde_json::to_string_pretty(config)?);
            } else {
                println!("DAA CLI Configuration:");
                println!("  Orchestrator Config: {:?}", config.orchestrator_config);
                println!("  Output Format: {:?}", config.default_output_format);
                println!("  API Endpoint: {}", config.connection.api_endpoint);
                println!("  MCP Endpoint: {}", config.connection.mcp_endpoint);
                println!("  Timeout: {}s", config.connection.timeout_seconds);
                println!("  Retry Attempts: {}", config.connection.retry_attempts);
                println!("  Colored Output: {}", config.display.colored);
                println!("  Page Size: {}", config.display.page_size);
                println!("  Show Timestamps: {}", config.display.show_timestamps);
                println!("  Compact Mode: {}", config.display.compact);
            }
        }
        ConfigAction::Get { key } => {
            let value = config.get_value(&key)?;
            if cli.json {
                println!("{}", serde_json::json!({ "key": key, "value": value }));
            } else {
                println!("{}: {}", key, value);
            }
        }
        ConfigAction::Set { key, value } => {
            let mut new_config = config.clone();
            new_config.set_value(&key, &value)?;
            new_config.validate()?;
            
            // Save to config file
            let config_path = crate::utils::get_default_config_path()?;
            new_config.to_file(&config_path)?;
            
            if cli.json {
                println!("{}", serde_json::json!({ "key": key, "value": value, "status": "updated" }));
            } else {
                println!("Updated {}: {}", key, value);
                println!("Configuration saved to: {}", config_path.display());
            }
        }
        ConfigAction::Validate => {
            match config.validate() {
                Ok(_) => {
                    if cli.json {
                        println!("{}", serde_json::json!({ "status": "valid" }));
                    } else {
                        println!("✓ Configuration is valid");
                    }
                }
                Err(e) => {
                    if cli.json {
                        println!("{}", serde_json::json!({ "status": "invalid", "error": e.to_string() }));
                    } else {
                        println!("✗ Configuration is invalid: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }
        ConfigAction::Reset { yes } => {
            if !yes {
                print!("This will reset your configuration to defaults. Are you sure? (y/N): ");
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Configuration reset cancelled");
                    return Ok(());
                }
            }
            
            let default_config = CliConfig::default();
            let config_path = crate::utils::get_default_config_path()?;
            default_config.to_file(&config_path)?;
            
            if cli.json {
                println!("{}", serde_json::json!({ "status": "reset" }));
            } else {
                println!("Configuration reset to defaults");
                println!("Configuration saved to: {}", config_path.display());
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = CliConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(!toml_str.is_empty());

        let parsed_config: CliConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed_config.validate().is_ok());
    }

    #[test]
    fn test_config_file_operations() {
        let config = CliConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        // Test saving
        config.to_file(temp_file.path()).unwrap();
        
        // Test loading
        let loaded_config = CliConfig::from_file(temp_file.path()).unwrap();
        assert!(loaded_config.validate().is_ok());
    }

    #[test]
    fn test_get_set_values() {
        let mut config = CliConfig::default();
        
        // Test getting values
        assert_eq!(config.get_value("connection.timeout_seconds").unwrap(), "30");
        assert_eq!(config.get_value("display.colored").unwrap(), "true");
        
        // Test setting values
        config.set_value("connection.timeout_seconds", "60").unwrap();
        assert_eq!(config.connection.timeout_seconds, 60);
        
        config.set_value("display.colored", "false").unwrap();
        assert!(!config.display.colored);
    }

    #[test]
    fn test_invalid_values() {
        let mut config = CliConfig::default();
        
        // Test invalid timeout
        assert!(config.set_value("connection.timeout_seconds", "invalid").is_err());
        
        // Test invalid boolean
        assert!(config.set_value("display.colored", "maybe").is_err());
        
        // Test unknown key
        assert!(config.get_value("unknown.key").is_err());
        assert!(config.set_value("unknown.key", "value").is_err());
    }
}