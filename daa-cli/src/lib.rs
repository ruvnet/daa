//! # DAA CLI
//!
//! Command-line interface for the Decentralized Autonomous Agents (DAA) system.
//! Provides comprehensive tooling for managing DAA components with QuDAG integration.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use colored::*;
use tabled::{Table, Tabled};

pub mod commands;
pub mod config;
pub mod utils;

#[cfg(feature = "chain")]
pub mod chain;

#[cfg(feature = "economy")]
pub mod economy;

#[cfg(feature = "rules")]
pub mod rules;

#[cfg(feature = "ai")]
pub mod ai;

#[cfg(feature = "orchestrator")]
pub mod orchestrator;

/// CLI error types
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Command execution error: {0}")]
    Execution(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[cfg(feature = "chain")]
    #[error("Chain error: {0}")]
    Chain(#[from] daa_chain::ChainError),
    
    #[cfg(feature = "economy")]
    #[error("Economy error: {0}")]
    Economy(#[from] daa_economy::EconomyError),
    
    #[cfg(feature = "rules")]
    #[error("Rules error: {0}")]
    Rules(#[from] daa_rules::RulesError),
    
    #[cfg(feature = "ai")]
    #[error("AI error: {0}")]
    AI(#[from] daa_ai::AIError),
    
    #[cfg(feature = "orchestrator")]
    #[error("Orchestrator error: {0}")]
    Orchestrator(#[from] daa_orchestrator::OrchestratorError),
}

pub type Result<T> = std::result::Result<T, CliError>;

/// DAA CLI - Command Line Interface for Decentralized Autonomous Agents
#[derive(Parser, Debug)]
#[command(
    name = "daa",
    version,
    about = "Command-line interface for Decentralized Autonomous Agents",
    long_about = "
DAA CLI provides comprehensive tooling for managing and interacting with
the Decentralized Autonomous Agents system, including blockchain operations,
economic management, rules governance, AI coordination, and orchestration.

Features:
- Chain management with QuDAG integration
- rUv token operations and economy management  
- Rules engine configuration and execution
- AI agent spawning and task coordination
- Workflow orchestration and service management

Use --help with any subcommand for detailed information.
"
)]
pub struct DaaCli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
    
    /// Output format (json, table, yaml)
    #[arg(short, long, global = true, default_value = "table")]
    pub output: OutputFormat,
    
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
    
    #[command(subcommand)]
    pub command: Commands,
}

/// Output format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Table,
    Yaml,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            "yaml" => Ok(OutputFormat::Yaml),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize DAA system configuration
    Init {
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
        
        /// Configuration template to use
        #[arg(short, long, default_value = "default")]
        template: String,
    },
    
    /// Display system status and health
    Status {
        /// Show detailed component status
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    
    /// Chain operations
    #[cfg(feature = "chain")]
    Chain {
        #[command(subcommand)]
        action: chain::ChainAction,
    },
    
    /// Economy operations
    #[cfg(feature = "economy")]
    Economy {
        #[command(subcommand)]
        action: economy::EconomyAction,
    },
    
    /// Rules management
    #[cfg(feature = "rules")]
    Rules {
        #[command(subcommand)]
        action: rules::RulesAction,
    },
    
    /// AI operations
    #[cfg(feature = "ai")]
    AI {
        #[command(subcommand)]
        action: ai::AIAction,
    },
    
    /// Orchestration operations
    #[cfg(feature = "orchestrator")]
    Orchestrator {
        #[command(subcommand)]
        action: orchestrator::OrchestratorAction,
    },
}

/// Configuration management actions
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Get configuration value
    Get {
        /// Configuration key (dot notation supported)
        key: String,
    },
    
    /// Set configuration value
    Set {
        /// Configuration key (dot notation supported)
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Validate configuration
    Validate,
    
    /// Reset configuration to defaults
    Reset {
        /// Confirm reset without prompt
        #[arg(short, long)]
        yes: bool,
    },
}

impl DaaCli {
    /// Execute the CLI command
    pub async fn execute(&self) -> Result<()> {
        // Load configuration
        let config = config::load_config(self.config.as_ref()).await?;
        
        // Disable colors if requested
        if self.no_color {
            colored::control::set_override(false);
        }
        
        // Execute command
        match &self.command {
            Commands::Init { force, template } => {
                commands::init::execute(*force, template, &self.output).await
            }
            
            Commands::Status { detailed } => {
                commands::status::execute(*detailed, &config, &self.output).await
            }
            
            Commands::Config { action } => {
                commands::config::execute(action, &self.output).await
            }
            
            #[cfg(feature = "chain")]
            Commands::Chain { action } => {
                chain::execute(action, &config, &self.output).await
            }
            
            #[cfg(feature = "economy")]
            Commands::Economy { action } => {
                economy::execute(action, &config, &self.output).await
            }
            
            #[cfg(feature = "rules")]
            Commands::Rules { action } => {
                rules::execute(action, &config, &self.output).await
            }
            
            #[cfg(feature = "ai")]
            Commands::AI { action } => {
                ai::execute(action, &config, &self.output).await
            }
            
            #[cfg(feature = "orchestrator")]
            Commands::Orchestrator { action } => {
                orchestrator::execute(action, &config, &self.output).await
            }
        }
    }
}

/// Format and display output based on the requested format
pub fn display_output<T>(data: &T, format: &OutputFormat) -> Result<()>
where
    T: Serialize + Tabled,
{
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(data)?;
            println!("{}", json);
        }
        
        OutputFormat::Table => {
            let table = Table::new(std::slice::from_ref(data));
            println!("{}", table);
        }
        
        OutputFormat::Yaml => {
            // For now, use JSON as YAML fallback
            let json = serde_json::to_string_pretty(data)?;
            println!("{}", json);
        }
    }
    
    Ok(())
}

/// Display a simple message with optional styling
pub fn display_message(message: &str, style: MessageStyle) {
    match style {
        MessageStyle::Success => println!("{} {}", "✓".green().bold(), message),
        MessageStyle::Warning => println!("{} {}", "⚠".yellow().bold(), message),
        MessageStyle::Error => println!("{} {}", "✗".red().bold(), message),
        MessageStyle::Info => println!("{} {}", "ℹ".blue().bold(), message),
        MessageStyle::Plain => println!("{}", message),
    }
}

/// Message styling options
pub enum MessageStyle {
    Success,
    Warning,
    Error,
    Info,
    Plain,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_structure() {
        // Verify CLI can be constructed
        DaaCli::command().debug_assert();
    }

    #[test]
    fn test_output_format_parsing() {
        assert!(matches!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json));
        assert!(matches!("table".parse::<OutputFormat>().unwrap(), OutputFormat::Table));
        assert!(matches!("yaml".parse::<OutputFormat>().unwrap(), OutputFormat::Yaml));
        assert!("invalid".parse::<OutputFormat>().is_err());
    }
}