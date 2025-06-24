//! Configuration management tool for MCP

use async_trait::async_trait;
use serde_json::{json, Value};

use super::{get_optional_bool_arg, get_optional_string_arg, get_required_string_arg, McpTool};
use crate::error::{Error, Result};

/// Config tool for configuration management
pub struct ConfigTool {
    name: String,
    description: String,
}

impl ConfigTool {
    /// Create a new config tool
    pub fn new() -> Self {
        Self {
            name: "config".to_string(),
            description: "Configuration management for QuDAG including reading, writing, validating, and migrating configuration files.".to_string(),
        }
    }

    /// Get configuration value
    async fn get(&self, args: &Value) -> Result<Value> {
        let key = get_required_string_arg(args, "key")?;
        let config_type =
            get_optional_string_arg(args, "configType").unwrap_or_else(|| "qudag".to_string());

        // Mock implementation - would read from actual config files
        let value = match (config_type.as_str(), key.as_str()) {
            ("qudag", "network.port") => json!(9000),
            ("qudag", "network.host") => json!("0.0.0.0"),
            ("qudag", "dag.consensus_threshold") => json!(0.67),
            ("mcp", "server.port") => json!(3000),
            ("mcp", "server.host") => json!("localhost"),
            ("vault", "encryption.algorithm") => json!("ml-kem-768"),
            _ => json!(null),
        };

        Ok(json!({
            "success": true,
            "configType": config_type,
            "key": key,
            "value": value,
            "exists": !value.is_null(),
            "source": "default"
        }))
    }

    /// Set configuration value
    async fn set(&self, args: &Value) -> Result<Value> {
        let key = get_required_string_arg(args, "key")?;
        let value = args
            .get("value")
            .ok_or_else(|| Error::invalid_params("Value required"))?;
        let config_type =
            get_optional_string_arg(args, "configType").unwrap_or_else(|| "qudag".to_string());
        let persist = get_optional_bool_arg(args, "persist").unwrap_or(true);

        // Validate key format
        if !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
        {
            return Err(Error::invalid_params("Invalid key format"));
        }

        // Mock implementation
        Ok(json!({
            "success": true,
            "configType": config_type,
            "key": key,
            "oldValue": null,
            "newValue": value,
            "persisted": persist,
            "configFile": format!("~/.qudag/{}.toml", config_type)
        }))
    }

    /// List all configuration
    async fn list(&self, args: &Value) -> Result<Value> {
        let config_type =
            get_optional_string_arg(args, "configType").unwrap_or_else(|| "qudag".to_string());
        let prefix = get_optional_string_arg(args, "prefix");

        // Mock configuration structure
        let configs = match config_type.as_str() {
            "qudag" => json!({
                "network": {
                    "port": 9000,
                    "host": "0.0.0.0",
                    "max_peers": 50,
                    "discovery_interval": 30
                },
                "dag": {
                    "consensus_threshold": 0.67,
                    "max_parents": 8,
                    "pruning_interval": 3600
                },
                "storage": {
                    "path": "~/.qudag/data",
                    "max_size": "10GB",
                    "compression": true
                }
            }),
            "mcp" => json!({
                "server": {
                    "port": 3000,
                    "host": "localhost",
                    "max_connections": 100
                },
                "auth": {
                    "enabled": true,
                    "jwt_expiration": 3600
                },
                "logging": {
                    "level": "info",
                    "format": "json"
                }
            }),
            "vault" => json!({
                "encryption": {
                    "algorithm": "ml-kem-768",
                    "key_derivation": "argon2id"
                },
                "storage": {
                    "backend": "sqlite",
                    "path": "~/.qudag/vault.db"
                }
            }),
            _ => json!({}),
        };

        // Filter by prefix if provided
        let filtered = if let Some(prefix) = prefix {
            filter_config_by_prefix(&configs, &prefix)
        } else {
            configs
        };

        Ok(json!({
            "success": true,
            "configType": config_type,
            "configuration": filtered,
            "source": "merged",
            "files": [
                format!("/etc/qudag/{}.toml", config_type),
                format!("~/.qudag/{}.toml", config_type)
            ]
        }))
    }

    /// Validate configuration
    async fn validate(&self, args: &Value) -> Result<Value> {
        let config_type =
            get_optional_string_arg(args, "configType").unwrap_or_else(|| "qudag".to_string());
        let config = args.get("config");
        let strict = get_optional_bool_arg(args, "strict").unwrap_or(false);

        // Mock validation
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if let Some(config) = config {
            // Example validation rules
            if let Some(port) = config.pointer("/network/port").and_then(|v| v.as_u64()) {
                if port > 65535 {
                    errors.push(json!({
                        "path": "network.port",
                        "message": "Port must be <= 65535",
                        "value": port
                    }));
                }
                if port < 1024 && strict {
                    warnings.push(json!({
                        "path": "network.port",
                        "message": "Port < 1024 requires root privileges",
                        "value": port
                    }));
                }
            }
        }

        Ok(json!({
            "success": true,
            "valid": errors.is_empty(),
            "configType": config_type,
            "errors": errors,
            "warnings": warnings,
            "checked": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Export configuration
    async fn export(&self, args: &Value) -> Result<Value> {
        let config_type =
            get_optional_string_arg(args, "configType").unwrap_or_else(|| "qudag".to_string());
        let format = get_optional_string_arg(args, "format").unwrap_or_else(|| "toml".to_string());
        let include_defaults = get_optional_bool_arg(args, "includeDefaults").unwrap_or(false);

        // Get current configuration
        let config = self.list(args).await?;

        // Convert to requested format
        let exported = match format.as_str() {
            "toml" => {
                // Mock TOML export
                format!("# QuDAG {} Configuration\n# Generated: {}\n\n[network]\nport = 9000\nhost = \"0.0.0.0\"\n\n[dag]\nconsensus_threshold = 0.67", 
                    config_type,
                    chrono::Utc::now().to_rfc3339()
                )
            }
            "json" => serde_json::to_string_pretty(&config["configuration"]).unwrap_or_default(),
            "yaml" => {
                // Mock YAML export
                format!("# QuDAG {} Configuration\nnetwork:\n  port: 9000\n  host: 0.0.0.0\ndag:\n  consensus_threshold: 0.67", config_type)
            }
            _ => {
                return Err(Error::invalid_params(format!(
                    "Unsupported format: {}",
                    format
                )))
            }
        };

        Ok(json!({
            "success": true,
            "configType": config_type,
            "format": format,
            "content": exported,
            "includeDefaults": include_defaults,
            "size": exported.len()
        }))
    }

    /// Reset configuration to defaults
    async fn reset(&self, args: &Value) -> Result<Value> {
        let config_type =
            get_optional_string_arg(args, "configType").unwrap_or_else(|| "qudag".to_string());
        let confirm = get_optional_bool_arg(args, "confirm").unwrap_or(false);

        if !confirm {
            return Ok(json!({
                "success": false,
                "error": "Confirmation required. Set 'confirm' to true to reset configuration."
            }));
        }

        // Mock implementation
        Ok(json!({
            "success": true,
            "configType": config_type,
            "message": format!("{} configuration reset to defaults", config_type),
            "backupCreated": true,
            "backupPath": format!("~/.qudag/{}.toml.backup.{}", config_type, chrono::Utc::now().timestamp())
        }))
    }
}

/// Filter configuration by prefix
fn filter_config_by_prefix(config: &Value, prefix: &str) -> Value {
    // Simple implementation - in production would be more sophisticated
    if let Some(obj) = config.as_object() {
        let filtered: serde_json::Map<String, Value> = obj
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        json!(filtered)
    } else {
        config.clone()
    }
}

#[async_trait]
impl McpTool for ConfigTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "The configuration operation to perform",
                    "enum": ["get", "set", "list", "validate", "export", "reset"]
                },
                "configType": {
                    "type": "string",
                    "description": "Type of configuration to manage",
                    "enum": ["qudag", "mcp", "vault", "network", "dag"]
                },
                "key": {
                    "type": "string",
                    "description": "Configuration key (dot notation, e.g., 'network.port')"
                },
                "value": {
                    "description": "Configuration value to set (any JSON type)"
                },
                "persist": {
                    "type": "boolean",
                    "description": "Persist configuration to file"
                },
                "prefix": {
                    "type": "string",
                    "description": "Filter configuration by key prefix"
                },
                "config": {
                    "type": "object",
                    "description": "Configuration object to validate"
                },
                "strict": {
                    "type": "boolean",
                    "description": "Enable strict validation mode"
                },
                "format": {
                    "type": "string",
                    "description": "Export format",
                    "enum": ["toml", "json", "yaml"]
                },
                "includeDefaults": {
                    "type": "boolean",
                    "description": "Include default values in export"
                },
                "confirm": {
                    "type": "boolean",
                    "description": "Confirm destructive operations"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, arguments: Option<Value>) -> Result<Value> {
        let args = arguments
            .as_ref()
            .ok_or_else(|| Error::invalid_params("Arguments required"))?;
        let operation = get_required_string_arg(args, "operation")?;

        match operation.as_str() {
            "get" => self.get(args).await,
            "set" => self.set(args).await,
            "list" => self.list(args).await,
            "validate" => self.validate(args).await,
            "export" => self.export(args).await,
            "reset" => self.reset(args).await,
            _ => Err(Error::invalid_params(format!(
                "Unknown operation: {}",
                operation
            ))),
        }
    }
}
