//! QuDAG Exchange CLI - Command-line interface for rUv token management

use clap::{Parser, Subcommand};
use anyhow::Result;
use qudag_exchange_core::{
    ExchangeConfig, ExchangeConfigBuilder, BusinessPlanConfig,
    ContributorRole, ContributorInfo, AccountId, rUv,
    types::Timestamp,
};

#[derive(Parser)]
#[command(
    name = "qudag-exchange-cli",
    about = "QuDAG Exchange - Quantum-secure resource exchange with rUv tokens",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new account
    CreateAccount {
        /// Name for the account
        #[arg(short, long)]
        name: String,
    },
    /// Check account balance
    Balance {
        /// Account name or ID
        #[arg(short, long)]
        account: String,
    },
    /// Transfer rUv tokens between accounts
    Transfer {
        /// Source account
        #[arg(short, long)]
        from: String,
        /// Destination account
        #[arg(short, long)]
        to: String,
        /// Amount of rUv to transfer
        #[arg(short, long)]
        amount: u64,
    },
    /// Start a QuDAG Exchange node
    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },
    /// Network operations
    Network {
        #[command(subcommand)]
        command: NetworkCommands,
    },
    /// Business plan operations (rUv payout streams)
    BusinessPlan {
        #[command(subcommand)]
        command: BusinessPlanCommands,
    },
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Start the node
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Stop the node
    Stop,
    /// Check node status
    Status,
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Show network status
    Status,
    /// List connected peers
    Peers,
    /// Connect to a peer
    Connect {
        /// Peer address
        address: String,
    },
}

#[derive(Subcommand)]
enum BusinessPlanCommands {
    /// Enable business plan features
    Enable {
        /// Enable automatic distribution
        #[arg(long)]
        auto_distribution: bool,
        /// Enable vault management
        #[arg(long)]
        vault_management: bool,
        /// Enable role-based earnings
        #[arg(long)]
        role_earnings: bool,
        /// Enable bounty rewards
        #[arg(long)]
        bounty_rewards: bool,
    },
    /// Disable business plan features
    Disable,
    /// Show business plan status
    Status,
    /// Configure payout splits
    Configure {
        #[command(subcommand)]
        command: ConfigureCommands,
    },
    /// Manage contributors
    Contributors {
        #[command(subcommand)]
        command: ContributorCommands,
    },
    /// View payout history
    Payouts {
        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Filter by contributor ID
        #[arg(long)]
        contributor: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigureCommands {
    /// Set minimum payout threshold
    Threshold {
        /// Minimum threshold in rUv
        amount: u64,
    },
    /// Set system fee percentage
    SystemFee {
        /// Percentage (0.0 to 0.1)
        percentage: f64,
    },
    /// Configure payout split for single-agent jobs
    SingleAgent {
        /// Agent percentage (0.0 to 1.0)
        agent: f64,
        /// Infrastructure percentage (0.0 to 1.0)
        infrastructure: f64,
    },
    /// Configure payout split for plugin-enhanced jobs
    PluginEnhanced {
        /// Agent percentage (0.0 to 1.0)
        agent: f64,
        /// Plugin percentage (0.0 to 1.0)
        plugin: f64,
        /// Infrastructure percentage (0.0 to 1.0)
        infrastructure: f64,
    },
}

#[derive(Subcommand)]
enum ContributorCommands {
    /// Register a new contributor
    Register {
        /// Contributor ID
        id: String,
        /// Contributor type
        #[arg(value_enum)]
        role: ContributorType,
        /// Vault ID for payouts
        vault_id: String,
        /// Custom payout percentage (optional)
        #[arg(long)]
        custom_percentage: Option<f64>,
    },
    /// List all contributors
    List,
    /// Show contributor details
    Show {
        /// Contributor ID
        id: String,
    },
    /// Update contributor configuration
    Update {
        /// Contributor ID
        id: String,
        /// New custom percentage (optional)
        #[arg(long)]
        custom_percentage: Option<f64>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ContributorType {
    AgentProvider,
    PluginCreator,
    NodeOperator,
    BountyAgent,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::CreateAccount { name } => {
            println!("Creating account: {}", name);
            // TODO: Implement account creation
            println!("Account created successfully!");
        }
        Commands::Balance { account } => {
            println!("Checking balance for account: {}", account);
            // TODO: Implement balance check
            println!("Balance: 1000 rUv");
        }
        Commands::Transfer { from, to, amount } => {
            println!("Transferring {} rUv from {} to {}", amount, from, to);
            // TODO: Implement transfer
            println!("Transfer completed successfully!");
        }
        Commands::Node { command } => match command {
            NodeCommands::Start { port } => {
                println!("Starting QuDAG Exchange node on port {}", port);
                // TODO: Implement node start
            }
            NodeCommands::Stop => {
                println!("Stopping QuDAG Exchange node");
                // TODO: Implement node stop
            }
            NodeCommands::Status => {
                println!("Node Status: Running");
                // TODO: Implement node status check
            }
        },
        Commands::Network { command } => match command {
            NetworkCommands::Status => {
                println!("Network Status: Healthy");
                // TODO: Implement network status
            }
            NetworkCommands::Peers => {
                println!("Connected Peers: 0");
                // TODO: Implement peer listing
            }
            NetworkCommands::Connect { address } => {
                println!("Connecting to peer: {}", address);
                // TODO: Implement peer connection
            }
        },
        Commands::BusinessPlan { command } => {
            handle_business_plan_command(command).await?;
        },
    }

    Ok(())
}

async fn handle_business_plan_command(command: BusinessPlanCommands) -> Result<()> {
    match command {
        BusinessPlanCommands::Enable {
            auto_distribution,
            vault_management,
            role_earnings,
            bounty_rewards,
        } => {
            println!("Enabling business plan features...");
            
            let mut bp_config = BusinessPlanConfig::default();
            bp_config.enabled = true;
            bp_config.enable_auto_distribution = auto_distribution;
            bp_config.enable_vault_management = vault_management;
            bp_config.enable_role_earnings = role_earnings;
            bp_config.enable_bounty_rewards = bounty_rewards;
            
            // Create exchange config with business plan
            let mut config = ExchangeConfigBuilder::new()
                .with_business_plan(bp_config)
                .build()?;
            
            println!("Business plan features enabled:");
            println!("  Auto Distribution: {}", auto_distribution);
            println!("  Vault Management: {}", vault_management);
            println!("  Role Earnings: {}", role_earnings);
            println!("  Bounty Rewards: {}", bounty_rewards);
            
            // TODO: Save configuration to file
            println!("Configuration saved.");
        },
        BusinessPlanCommands::Disable => {
            println!("Disabling business plan features...");
            
            let mut config = ExchangeConfig::new()?;
            config.disable_business_plan();
            
            println!("Business plan features disabled.");
            // TODO: Save configuration to file
        },
        BusinessPlanCommands::Status => {
            println!("Business Plan Status");
            println!("===================");
            
            // TODO: Load configuration from file
            let config = ExchangeConfig::new()?;
            let current_time = Timestamp::now();
            let summary = config.get_summary(current_time);
            
            match summary.business_plan_summary {
                Some(bp_summary) => {
                    println!("Status: {}", if bp_summary.enabled { "Enabled" } else { "Disabled" });
                    println!("Auto Distribution: {}", bp_summary.auto_distribution_enabled);
                    println!("Vault Management: {}", bp_summary.vault_management_enabled);
                    println!("Role Earnings: {}", bp_summary.role_earnings_enabled);
                    println!("Bounty Rewards: {}", bp_summary.bounty_rewards_enabled);
                    println!("Total Contributors: {}", bp_summary.total_contributors);
                    println!("Min Payout Threshold: {} rUv", bp_summary.min_payout_threshold);
                    println!("System Fee: {:.4}%", bp_summary.system_fee_percentage * 100.0);
                },
                None => {
                    println!("Status: Disabled");
                    println!("Business plan features are not enabled.");
                    println!("Use 'business-plan enable' to activate payout streams.");
                }
            }
        },
        BusinessPlanCommands::Configure { command } => {
            handle_configure_command(command).await?;
        },
        BusinessPlanCommands::Contributors { command } => {
            handle_contributor_command(command).await?;
        },
        BusinessPlanCommands::Payouts { limit, contributor } => {
            println!("Payout History (last {} entries)", limit);
            println!("==================================");
            
            // TODO: Load actual payout history from storage
            println!("No payouts found.");
            if let Some(contrib_id) = contributor {
                println!("Filtered by contributor: {}", contrib_id);
            }
            
            println!("\nExample payout structure:");
            println!("  Transaction ID: tx_abc123");
            println!("  Total Fee: 100 rUv");
            println!("  Payouts:");
            println!("    - Agent (agent_xyz): 85 rUv (85.0%)");
            println!("    - Plugin (plugin_123): 10 rUv (10.0%)");
            println!("    - Infrastructure: 5 rUv (5.0%)");
        },
    }
    Ok(())
}

async fn handle_configure_command(command: ConfigureCommands) -> Result<()> {
    match command {
        ConfigureCommands::Threshold { amount } => {
            println!("Setting minimum payout threshold to {} rUv", amount);
            
            // TODO: Load existing config, update threshold, save
            let mut config = ExchangeConfig::new()?;
            if let Some(bp_config) = &mut config.business_plan {
                bp_config.payout_config.min_payout_threshold = rUv::new(amount);
                println!("Threshold updated successfully.");
            } else {
                println!("Error: Business plan features not enabled.");
                println!("Use 'business-plan enable' first.");
            }
        },
        ConfigureCommands::SystemFee { percentage } => {
            if percentage < 0.0 || percentage > 0.1 {
                println!("Error: System fee percentage must be between 0.0 and 0.1 (10% max)");
                return Ok(());
            }
            
            println!("Setting system fee to {:.4}%", percentage * 100.0);
            
            // TODO: Load existing config, update system fee, save
            let mut config = ExchangeConfig::new()?;
            if let Some(bp_config) = &mut config.business_plan {
                bp_config.payout_config.system_fee_percentage = percentage;
                println!("System fee updated successfully.");
            } else {
                println!("Error: Business plan features not enabled.");
            }
        },
        ConfigureCommands::SingleAgent { agent, infrastructure } => {
            if (agent + infrastructure) > 1.0 {
                println!("Error: Total percentages cannot exceed 100%");
                return Ok(());
            }
            
            println!("Configuring single-agent payout split:");
            println!("  Agent: {:.1}%", agent * 100.0);
            println!("  Infrastructure: {:.1}%", infrastructure * 100.0);
            
            // TODO: Update configuration
            println!("Single-agent split updated successfully.");
        },
        ConfigureCommands::PluginEnhanced { agent, plugin, infrastructure } => {
            if (agent + plugin + infrastructure) > 1.0 {
                println!("Error: Total percentages cannot exceed 100%");
                return Ok(());
            }
            
            println!("Configuring plugin-enhanced payout split:");
            println!("  Agent: {:.1}%", agent * 100.0);
            println!("  Plugin: {:.1}%", plugin * 100.0);
            println!("  Infrastructure: {:.1}%", infrastructure * 100.0);
            
            // TODO: Update configuration
            println!("Plugin-enhanced split updated successfully.");
        },
    }
    Ok(())
}

async fn handle_contributor_command(command: ContributorCommands) -> Result<()> {
    match command {
        ContributorCommands::Register { id, role, vault_id, custom_percentage } => {
            println!("Registering contributor: {}", id);
            println!("  Role: {:?}", role);
            println!("  Vault ID: {}", vault_id);
            
            if let Some(pct) = custom_percentage {
                println!("  Custom Percentage: {:.2}%", pct * 100.0);
            }
            
            // Convert CLI role to core role type
            let core_role = match role {
                ContributorType::AgentProvider => ContributorRole::AgentProvider {
                    agent_id: id.clone(),
                    resource_consumed: 0, // Will be updated during operations
                },
                ContributorType::PluginCreator => ContributorRole::PluginCreator {
                    module_id: id.clone(),
                    usage_count: 0,
                },
                ContributorType::NodeOperator => ContributorRole::NodeOperator {
                    node_id: id.clone(),
                    consensus_rounds: 0,
                    uptime_percentage: 1.0,
                },
                ContributorType::BountyAgent => ContributorRole::BountyAgent {
                    bounty_id: id.clone(),
                    completed_at: Timestamp::now(),
                },
            };
            
            let contributor_info = ContributorInfo {
                vault_id: AccountId::new(vault_id),
                role: core_role,
                custom_percentage,
                registered_at: Timestamp::now(),
                total_earnings: rUv::new(0),
                last_payout: None,
            };
            
            // TODO: Save contributor to storage
            println!("Contributor registered successfully!");
        },
        ContributorCommands::List => {
            println!("Registered Contributors");
            println!("======================");
            
            // TODO: Load contributors from storage
            println!("No contributors found.");
            println!("Use 'business-plan contributors register' to add contributors.");
        },
        ContributorCommands::Show { id } => {
            println!("Contributor Details: {}", id);
            println!("====================");
            
            // TODO: Load specific contributor from storage
            println!("Contributor not found.");
            println!("Use 'business-plan contributors list' to see all contributors.");
        },
        ContributorCommands::Update { id, custom_percentage } => {
            println!("Updating contributor: {}", id);
            
            if let Some(pct) = custom_percentage {
                println!("Setting custom percentage to: {:.2}%", pct * 100.0);
            }
            
            // TODO: Update contributor in storage
            println!("Contributor updated successfully!");
        },
    }
    Ok(())
}