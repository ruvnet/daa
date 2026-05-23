//! DAA CLI - Command Line Interface for Decentralized Autonomous Applications

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use tracing::{error, info};

mod commands;
mod config;
mod utils;

use config::CliConfig;

/// Lightweight context passed to command handlers so that `cli.command` can be
/// destructured without triggering borrow-after-partial-move.
pub struct CliContext {
    pub verbose: bool,
    pub json: bool,
    pub no_color: bool,
    pub config: Option<PathBuf>,
}

/// DAA CLI - Decentralized Autonomous Application Command Line Interface
#[derive(Parser)]
#[command(name = "daa")]
#[command(
    about = "A CLI for managing Decentralized Autonomous Applications with QuDAG integration"
)]
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
        #[arg(short = 'C', long)]
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

    // Snapshot CLI flags before consuming cli.command, avoiding partial-move issues.
    let verbose = cli.verbose;
    let json = cli.json;
    let no_color = cli.no_color;
    let config_path = cli.config.clone();

    // Build a lightweight context struct so command handlers don't need the full Cli.
    // This avoids Rust borrow-after-partial-move when we destructure cli.command.
    let cli_ctx = CliContext {
        verbose,
        json,
        no_color,
        config: config_path,
    };

    // Handle commands
    match cli.command {
        Commands::Init {
            directory,
            template,
            force,
        } => commands::init::handle_init(directory, template, force, &cli_ctx).await,
        Commands::Start { daemon, pid_file } => {
            commands::start::handle_start(daemon, pid_file, &config, &cli_ctx).await
        }
        Commands::Status {
            detailed,
            watch,
            interval,
        } => commands::status::handle_status(detailed, watch, interval, &config, &cli_ctx).await,
        Commands::Stop {
            force,
            grace_period,
        } => commands::stop::handle_stop(force, grace_period, &config, &cli_ctx).await,
        Commands::AddRule {
            name,
            rule_type,
            params,
            description,
        } => {
            commands::rules::handle_add_rule(
                name,
                rule_type,
                params,
                description,
                &config,
                &cli_ctx,
            )
            .await
        }
        Commands::Config { action } => handle_config_command(action, &config, &cli_ctx).await,
        Commands::Network { action } => {
            commands::network::handle_network(action, &config, &cli_ctx).await
        }
        Commands::Agent { action } => {
            commands::agent::handle_agent(action, &config, &cli_ctx).await
        }
        Commands::Logs {
            lines,
            follow,
            level,
            component,
        } => commands::logs::handle_logs(lines, follow, level, component, &config, &cli_ctx).await,
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

async fn handle_config_command(
    action: ConfigAction,
    config: &CliConfig,
    cli: &CliContext,
) -> Result<()> {
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
            let config_path = utils::get_default_config_path()?;
            new_config.to_file(&config_path)?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({ "key": key, "value": value, "status": "updated" })
                );
            } else {
                println!("Updated {}: {}", key, value);
                println!("Configuration saved to: {}", config_path.display());
            }
        }
        ConfigAction::Validate => match config.validate() {
            Ok(_) => {
                if cli.json {
                    println!("{}", serde_json::json!({ "status": "valid" }));
                } else {
                    println!("Configuration is valid");
                }
            }
            Err(e) => {
                if cli.json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "invalid", "error": e.to_string() })
                    );
                } else {
                    println!("Configuration is invalid: {}", e);
                }
                std::process::exit(1);
            }
        },
        ConfigAction::Reset { yes } => {
            if !yes {
                use std::io::{self, Write};
                print!("This will reset your configuration to defaults. Are you sure? (y/N): ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Configuration reset cancelled");
                    return Ok(());
                }
            }
            let default_config = CliConfig::default();
            let config_path = utils::get_default_config_path()?;
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
                format!(
                    "No configuration file found at {}, using defaults",
                    config_path.display()
                )
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
