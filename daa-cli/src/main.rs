//! DAA CLI - Command Line Interface for Decentralized Autonomous Applications

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colorful::Colorful;
use std::path::PathBuf;
use tracing::{info, error};

mod commands;
mod config;
mod utils;

use commands::*;
use config::CliConfig;

/// DAA CLI - Decentralized Autonomous Application Command Line Interface
#[derive(Parser)]
#[command(name = "daa")]
#[command(about = "A CLI for managing Decentralized Autonomous Applications with QuDAG integration")]
#[command(version)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// JSON output format
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new DAA configuration
    Init {
        /// Directory to initialize
        #[arg(short, long)]
        directory: Option<PathBuf>,
        
        /// Configuration template to use
        #[arg(short, long, default_value = "default")]
        template: String,
        
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },

    /// Start the DAA orchestrator
    Start {
        /// Run in daemon mode
        #[arg(short, long)]
        daemon: bool,
        
        /// PID file location (for daemon mode)
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },

    /// Get status of DAA components
    Status {
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
        
        /// Watch mode (continuous updates)
        #[arg(short, long)]
        watch: bool,
        
        /// Update interval in seconds for watch mode
        #[arg(long, default_value = "5")]
        interval: u64,
    },

    /// Stop the DAA orchestrator
    Stop {
        /// Force stop (kill process)
        #[arg(short, long)]
        force: bool,
        
        /// Grace period in seconds before force kill
        #[arg(long, default_value = "30")]
        grace_period: u64,
    },

    /// Add a new rule to the rules engine
    AddRule {
        /// Rule name/identifier
        #[arg(short, long)]
        name: String,
        
        /// Rule type
        #[arg(short, long)]
        rule_type: String,
        
        /// Rule parameters (JSON format)
        #[arg(short, long)]
        params: Option<String>,
        
        /// Rule description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// QuDAG network operations
    Network {
        #[command(subcommand)]
        action: NetworkAction,
    },

    /// Agent management
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },

    /// Logs management
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        
        /// Filter by log level
        #[arg(long)]
        level: Option<String>,
        
        /// Component to show logs for
        #[arg(long)]
        component: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Set a configuration value
    Set {
        /// Configuration key (dot notation)
        key: String,
        /// Configuration value
        value: String,
    },
    
    /// Get a configuration value
    Get {
        /// Configuration key (dot notation)
        key: String,
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

#[derive(Subcommand)]
pub enum NetworkAction {
    /// Show network status
    Status,
    
    /// Connect to QuDAG network
    Connect {
        /// Specific node to connect to
        #[arg(short, long)]
        node: Option<String>,
    },
    
    /// Disconnect from QuDAG network
    Disconnect,
    
    /// List connected peers
    Peers,
    
    /// Show network statistics
    Stats,
}

#[derive(Subcommand)]
pub enum AgentAction {
    /// List all agents
    List,
    
    /// Show agent details
    Show {
        /// Agent ID
        agent_id: String,
    },
    
    /// Create a new agent
    Create {
        /// Agent name
        #[arg(short, long)]
        name: String,
        
        /// Agent type
        #[arg(short, long)]
        agent_type: String,
        
        /// Agent capabilities (comma-separated)
        #[arg(short, long)]
        capabilities: Option<String>,
    },
    
    /// Stop an agent
    Stop {
        /// Agent ID
        agent_id: String,
        
        /// Force stop
        #[arg(short, long)]
        force: bool,
    },
    
    /// Restart an agent
    Restart {
        /// Agent ID
        agent_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli)?;

    // Load configuration
    let config = load_config(&cli).await?;

    // Handle commands
    match cli.command {
        Commands::Init { directory, template, force } => {
            init::handle_init(directory, template, force, &cli).await
        }
        Commands::Start { daemon, pid_file } => {
            start::handle_start(daemon, pid_file, &config, &cli).await
        }
        Commands::Status { detailed, watch, interval } => {
            status::handle_status(detailed, watch, interval, &config, &cli).await
        }
        Commands::Stop { force, grace_period } => {
            stop::handle_stop(force, grace_period, &config, &cli).await
        }
        Commands::AddRule { name, rule_type, params, description } => {
            rules::handle_add_rule(name, rule_type, params, description, &config, &cli).await
        }
        Commands::Config { action } => {
            config::handle_config(action, &config, &cli).await
        }
        Commands::Network { action } => {
            network::handle_network(action, &config, &cli).await
        }
        Commands::Agent { action } => {
            agent::handle_agent(action, &config, &cli).await
        }
        Commands::Logs { lines, follow, level, component } => {
            logs::handle_logs(lines, follow, level, component, &config, &cli).await
        }
    }
}

fn init_logging(cli: &Cli) -> Result<()> {
    let level = if cli.verbose { "debug" } else { "info" };
    
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(format!("daa={},daa_orchestrator={}", level, level))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    if cli.no_color {
        subscriber.without_time().init();
    } else {
        subscriber.init();
    }

    Ok(())
}

async fn load_config(cli: &Cli) -> Result<CliConfig> {
    let config_path = if let Some(ref path) = cli.config {
        path.clone()
    } else {
        utils::get_default_config_path()?
    };

    if config_path.exists() {
        info!("Loading configuration from: {}", config_path.display());
        CliConfig::from_file(&config_path)
            .with_context(|| format!("Failed to load config from {}", config_path.display()))
    } else {
        if cli.verbose {
            println!(
                "{}",
                format!("No configuration file found at {}, using defaults", config_path.display())
                    .yellow()
            );
        }
        Ok(CliConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert()
    }

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(&["daa", "status"]).unwrap();
        assert!(matches!(cli.command, Commands::Status { .. }));
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::try_parse_from(&["daa", "-v", "status"]).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_config_flag() {
        let cli = Cli::try_parse_from(&["daa", "-c", "/path/to/config.toml", "status"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.toml")));
    }
}