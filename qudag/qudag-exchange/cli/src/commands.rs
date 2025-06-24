//! CLI command implementations

use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use qudag_exchange_core::{Ledger, RuvAmount, Transaction, TransactionType};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::display;

/// Wallet-related commands
#[derive(Subcommand)]
pub enum WalletCommand {
    /// Create a new wallet
    Create {
        /// Optional wallet address (generated if not provided)
        #[arg(short, long)]
        address: Option<String>,
        
        /// Create as vault-backed wallet
        #[arg(short, long)]
        vault: bool,
    },
    
    /// Show wallet balance
    Balance {
        /// Wallet address
        address: String,
    },
    
    /// List all wallets
    List,
}

/// Transaction-related commands
#[derive(Subcommand)]
pub enum TransactionCommand {
    /// Send rUv to another wallet
    Send {
        /// Sender address
        #[arg(short, long)]
        from: String,
        
        /// Recipient address
        #[arg(short, long)]
        to: String,
        
        /// Amount to send
        #[arg(short, long)]
        amount: u64,
        
        /// Transaction fee (default: 1 rUv)
        #[arg(short, long, default_value = "1")]
        fee: u64,
    },
    
    /// Show transaction details
    Show {
        /// Transaction ID
        tx_id: String,
    },
    
    /// List pending transactions
    Pending,
}

/// Resource contribution commands
#[derive(Subcommand)]
pub enum ResourceCommand {
    /// Start resource contribution
    Start {
        /// Agent ID
        agent_id: String,
    },
    
    /// Submit resource metrics
    Submit {
        /// Agent ID
        agent_id: String,
        
        /// Resource type (cpu, gpu, memory, storage, bandwidth)
        #[arg(short, long)]
        resource_type: String,
        
        /// Amount of resource
        #[arg(short, long)]
        amount: f64,
        
        /// Duration in seconds
        #[arg(short, long)]
        duration: u64,
        
        /// Quality score (0.0-1.0)
        #[arg(short, long, default_value = "1.0")]
        quality: f64,
    },
    
    /// Finalize resource contribution and mint rUv
    Finalize {
        /// Agent ID
        agent_id: String,
    },
}

/// Handle wallet commands
pub async fn handle_wallet_command(
    cmd: WalletCommand,
    ledger: Arc<RwLock<Ledger>>,
    _config: Config,
) -> Result<()> {
    match cmd {
        WalletCommand::Create { address, vault } => {
            let address = address.unwrap_or_else(|| {
                // Generate address (simplified)
                format!("qudag1{}", hex::encode(&rand::random::<[u8; 8]>()))
            });
            
            let ledger = ledger.read().await;
            let wallet = ledger.get_or_create_wallet(address.clone(), vault);
            
            println!("{}", "Wallet created successfully!".green());
            println!("Address: {}", wallet.address.bright_blue());
            println!("Type: {}", if vault { "Vault-backed" } else { "Standard" });
        }
        
        WalletCommand::Balance { address } => {
            let ledger = ledger.read().await;
            if let Some(balance) = ledger.get_balance(&address) {
                println!("Wallet: {}", address.bright_blue());
                println!("Balance: {}", format!("{}", balance).bright_green());
            } else {
                println!("{}", "Wallet not found".red());
            }
        }
        
        WalletCommand::List => {
            let ledger = ledger.read().await;
            let stats = ledger.stats();
            println!("Total wallets: {}", stats.wallet_count);
            // In a real implementation, we'd iterate through wallets
        }
    }
    
    Ok(())
}

/// Handle transaction commands
pub async fn handle_transaction_command(
    cmd: TransactionCommand,
    ledger: Arc<RwLock<Ledger>>,
    _config: Config,
) -> Result<()> {
    match cmd {
        TransactionCommand::Send { from, to, amount, fee } => {
            let tx = Transaction::new(
                TransactionType::Transfer {
                    from: from.clone(),
                    to: to.clone(),
                    amount: RuvAmount::from_ruv(amount),
                },
                RuvAmount::from_ruv(fee),
            );
            
            let ledger = ledger.write().await;
            match ledger.submit_transaction(tx) {
                Ok(tx_id) => {
                    println!("{}", "Transaction submitted successfully!".green());
                    println!("Transaction ID: {}", tx_id.bright_blue());
                    println!("Status: {}", "Pending".yellow());
                    
                    // Auto-process for demo
                    if let Err(e) = ledger.process_transaction(&tx_id) {
                        println!("{}: {}", "Failed to process".red(), e);
                    } else {
                        println!("Status: {}", "Confirmed".green());
                    }
                }
                Err(e) => {
                    println!("{}: {}", "Transaction failed".red(), e);
                }
            }
        }
        
        TransactionCommand::Show { tx_id } => {
            let ledger = ledger.read().await;
            if let Some(tx) = ledger.get_transaction(&tx_id) {
                display::show_transaction(&tx);
            } else {
                println!("{}", "Transaction not found".red());
            }
        }
        
        TransactionCommand::Pending => {
            let ledger = ledger.read().await;
            let pool_size = ledger.tx_pool_size();
            println!("Pending transactions: {}", pool_size);
        }
    }
    
    Ok(())
}

/// Handle resource commands
pub async fn handle_resource_command(
    cmd: ResourceCommand,
    ledger: Arc<RwLock<Ledger>>,
    _config: Config,
) -> Result<()> {
    use qudag_exchange_core::resource::{ResourceMetrics, ResourceType};
    
    match cmd {
        ResourceCommand::Start { agent_id } => {
            let ledger = ledger.write().await;
            ledger.start_resource_contribution(agent_id.clone());
            println!("{}", "Resource contribution started!".green());
            println!("Agent ID: {}", agent_id.bright_blue());
        }
        
        ResourceCommand::Submit { 
            agent_id, 
            resource_type, 
            amount, 
            duration, 
            quality 
        } => {
            let resource_type = match resource_type.to_lowercase().as_str() {
                "cpu" => ResourceType::Cpu,
                "gpu" => ResourceType::Gpu,
                "memory" => ResourceType::Memory,
                "storage" => ResourceType::Storage,
                "bandwidth" => ResourceType::Bandwidth,
                other => ResourceType::Custom(other.to_string()),
            };
            
            let metric = ResourceMetrics {
                resource_type,
                amount,
                duration,
                quality_score: quality.clamp(0.0, 1.0),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            
            let ruv_value = metric.calculate_ruv_value();
            
            let ledger = ledger.write().await;
            ledger.record_resource_metric(&agent_id, metric)?;
            
            println!("{}", "Resource metric recorded!".green());
            println!("Estimated value: {}", format!("{}", ruv_value).bright_green());
        }
        
        ResourceCommand::Finalize { agent_id } => {
            let ledger = ledger.write().await;
            match ledger.finalize_resource_contribution(&agent_id)? {
                Some(tx_id) => {
                    println!("{}", "Resource contribution finalized!".green());
                    println!("Mint transaction ID: {}", tx_id.bright_blue());
                    
                    // Auto-process the mint
                    if let Err(e) = ledger.process_transaction(&tx_id) {
                        println!("{}: {}", "Failed to mint".red(), e);
                    } else {
                        println!("Status: {}", "rUv minted successfully!".green());
                    }
                }
                None => {
                    println!("{}", "No contribution found for agent".yellow());
                }
            }
        }
    }
    
    Ok(())
}

// Add rand dependency for address generation
use rand;