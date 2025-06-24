//! QuDAG Exchange CLI
//!
//! Command-line interface for interacting with the QuDAG Exchange system.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use qudag_exchange_core::{AccountId, Balance, Exchange, ExchangeConfig};
use std::path::PathBuf;

mod commands;
mod config;
mod output;

use commands::*;
use config::CliConfig;
use output::OutputFormat;

/// QuDAG Exchange - Quantum-secure resource exchange protocol
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    output: OutputFormat,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new account
    CreateAccount {
        /// Account name/identifier
        #[arg(short, long)]
        name: String,
        
        /// Initial balance (if bootstrapping)
        #[arg(short, long)]
        initial_balance: Option<Balance>,
    },
    
    /// Check account balance
    Balance {
        /// Account identifier (defaults to current account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Transfer rUv tokens between accounts
    Transfer {
        /// Recipient account
        #[arg(short, long)]
        to: String,
        
        /// Amount to transfer
        #[arg(short, long)]
        amount: Balance,
        
        /// Optional memo/message
        #[arg(short, long)]
        memo: Option<String>,
    },
    
    /// Display resource usage status
    ResourceStatus {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Display consensus information
    ConsensusInfo {
        /// Show peer details
        #[arg(short, long)]
        peers: bool,
    },
    
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Initialize configuration
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
    
    /// Set configuration value
    Set {
        key: String,
        value: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .init();
    }
    
    // Load configuration
    let config_path = cli.config
        .or_else(|| CliConfig::default_path().ok())
        .context("Could not determine config path")?;
        
    let cli_config = if config_path.exists() {
        CliConfig::load(&config_path)?
    } else {
        println!("{}", "No configuration found. Run 'qudag-exchange config init' to create one.".yellow());
        CliConfig::default()
    };
    
    // Execute command
    match cli.command {
        Commands::CreateAccount { name, initial_balance } => {
            create_account::execute(name, initial_balance, &cli_config, cli.output).await?;
        }
        
        Commands::Balance { account } => {
            balance::execute(account, &cli_config, cli.output).await?;
        }
        
        Commands::Transfer { to, amount, memo } => {
            transfer::execute(to, amount, memo, &cli_config, cli.output).await?;
        }
        
        Commands::ResourceStatus { detailed } => {
            resource_status::execute(detailed, &cli_config, cli.output).await?;
        }
        
        Commands::ConsensusInfo { peers } => {
            consensus_info::execute(peers, &cli_config, cli.output).await?;
        }
        
        Commands::Config { action } => {
            match action {
                ConfigAction::Show => {
                    config::show(&cli_config, cli.output)?;
                }
                ConfigAction::Init { force } => {
                    config::init(&config_path, force)?;
                }
                ConfigAction::Set { key, value } => {
                    config::set(&config_path, &key, &value)?;
                }
            }
        }
    }
    
    Ok(())
}