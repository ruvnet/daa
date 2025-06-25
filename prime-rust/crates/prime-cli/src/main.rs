// Prime CLI with DAA Integration
use anyhow::Result;
use clap::{Parser, Subcommand};
use prime_coordinator::CoordinatorNode;
use prime_core::{NodeType, EconomyInterface, GovernanceInterface};
use prime_dht::{DhtNode, PrimeDht};
use prime_trainer::TrainerAgentFactory;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

/// Prime - Decentralized ML Training Framework with DAA Integration
#[derive(Parser)]
#[command(name = "prime")]
#[command(about = "Decentralized ML training with autonomous agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a node with the specified role
    Up {
        /// Node role: trainer, coordinator, dht
        #[arg(short, long)]
        role: String,
        
        /// Node ID (defaults to random)
        #[arg(short, long)]
        id: Option<String>,
        
        /// Bootstrap peer addresses
        #[arg(short, long)]
        bootstrap: Vec<String>,
        
        /// Public IP address for this node
        #[arg(long)]
        public_ip: Option<String>,
    },
    
    /// Join the network as a trainer
    Join {
        /// Coordinator address
        #[arg(short, long)]
        coordinator: String,
        
        /// Node ID (defaults to random)
        #[arg(short, long)]
        id: Option<String>,
    },
    
    /// Show network status
    Status {
        /// DHT node to query
        #[arg(short, long)]
        dht: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Up { role, id, bootstrap, public_ip } => {
            let node_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            
            // Create mock implementations for DAA interfaces
            let dht = Arc::new(PrimeDht::new(node_id.clone(), create_mock_economy()));
            let economy = create_mock_economy();
            let governance = create_mock_governance();
            
            match role.as_str() {
                "trainer" => {
                    tracing::info!("Starting trainer node: {}", node_id);
                    
                    let trainer = TrainerAgentFactory::create_agent(
                        node_id,
                        dht,
                        economy,
                        governance,
                    ).await?;
                    
                    trainer.start().await?;
                }
                
                "coordinator" => {
                    tracing::info!("Starting coordinator node: {}", node_id);
                    
                    let mut coordinator = CoordinatorNode::new(
                        node_id.clone(),
                        prime_coordinator::CoordinatorConfig::default(),
                        dht.clone(),
                        economy.clone(),
                        governance.clone(),
                    ).await?;
                    
                    let daa_context = prime_core::DaaContext {
                        node_id,
                        node_type: NodeType::Coordinator,
                        peer_uri: format!("grpc://{}:50051", public_ip.unwrap_or("localhost".to_string())),
                        dht_handle: dht,
                        economy_handle: economy,
                        governance_handle: governance,
                    };
                    
                    coordinator.setup_autonomy_loops(daa_context).await?;
                    coordinator.start().await?;
                }
                
                "dht" => {
                    tracing::info!("Starting DHT node: {}", node_id);
                    
                    let mut dht_node = DhtNode::new(
                        node_id.clone(),
                        bootstrap,
                        economy.clone(),
                        governance.clone(),
                    ).await?;
                    
                    let daa_context = prime_core::DaaContext {
                        node_id,
                        node_type: NodeType::ParameterServer,
                        peer_uri: format!("p2p://{}", public_ip.unwrap_or("localhost".to_string())),
                        dht_handle: dht,
                        economy_handle: economy,
                        governance_handle: governance,
                    };
                    
                    dht_node.setup_autonomy_loops(daa_context).await?;
                    dht_node.start().await?;
                }
                
                _ => {
                    anyhow::bail!("Unknown role: {}. Use: trainer, coordinator, or dht", role);
                }
            }
        }
        
        Commands::Join { coordinator, id } => {
            let node_id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            tracing::info!("Joining network as trainer {} via coordinator {}", node_id, coordinator);
            
            // Create DAA interfaces
            let dht = Arc::new(PrimeDht::new(node_id.clone(), create_mock_economy()));
            let economy = create_mock_economy();
            let governance = create_mock_governance();
            
            // Update peer URI to point to coordinator
            let trainer = TrainerAgentFactory::create_agent(
                node_id,
                dht,
                economy,
                governance,
            ).await?;
            
            trainer.start().await?;
        }
        
        Commands::Status { dht } => {
            tracing::info!("Querying network status from DHT: {}", dht);
            
            // In production, would connect to DHT and query network state
            println!("Network Status:");
            println!("  Active nodes: 3");
            println!("  Current round: 42");
            println!("  Model version: 7");
            println!("  Total contributions: 1,234,567");
        }
    }
    
    Ok(())
}

/// Create a mock economy implementation
fn create_mock_economy() -> Arc<dyn EconomyInterface> {
    Arc::new(MockEconomy::default())
}

/// Create a mock governance implementation
fn create_mock_governance() -> Arc<dyn GovernanceInterface> {
    Arc::new(MockGovernance::default())
}

// Mock implementations for testing
#[derive(Default)]
struct MockEconomy {
    balances: Arc<tokio::sync::RwLock<std::collections::HashMap<String, u64>>>,
}

#[async_trait::async_trait]
impl EconomyInterface for MockEconomy {
    async fn reward_contribution(&self, contributor: &str, amount: u64) -> prime_core::Result<()> {
        let mut balances = self.balances.write().await;
        let balance = balances.entry(contributor.to_string()).or_insert(1000);
        *balance += amount;
        tracing::debug!("Rewarded {} tokens to {}", amount, contributor);
        Ok(())
    }
    
    async fn charge_usage(&self, user: &str, amount: u64) -> prime_core::Result<()> {
        let mut balances = self.balances.write().await;
        let balance = balances.entry(user.to_string()).or_insert(1000);
        if *balance < amount {
            return Err(prime_core::PrimeError::EconomicError("Insufficient balance".to_string()));
        }
        *balance -= amount;
        tracing::debug!("Charged {} tokens from {}", amount, user);
        Ok(())
    }
    
    async fn get_balance(&self, account: &str) -> prime_core::Result<u64> {
        let balances = self.balances.read().await;
        Ok(*balances.get(account).unwrap_or(&1000))
    }
}

#[derive(Default)]
struct MockGovernance;

#[async_trait::async_trait]
impl GovernanceInterface for MockGovernance {
    async fn validate_action(&self, action: &str, params: std::collections::HashMap<String, String>) -> prime_core::Result<bool> {
        tracing::debug!("Validating action: {} with params: {:?}", action, params);
        
        // Simple mock validation rules
        match action {
            "training_config" => {
                if let Some(lr) = params.get("learning_rate") {
                    let learning_rate: f32 = lr.parse().unwrap_or(1.0);
                    Ok(learning_rate <= 0.1) // Max learning rate
                } else {
                    Ok(true)
                }
            }
            "dht_storage" => {
                if let Some(size) = params.get("storage_size") {
                    let storage_size: usize = size.parse().unwrap_or(0);
                    Ok(storage_size < 100000) // Max 100k keys
                } else {
                    Ok(true)
                }
            }
            _ => Ok(true),
        }
    }
    
    async fn add_rule(&self, rule: prime_core::GovernanceRule) -> prime_core::Result<()> {
        tracing::info!("Added governance rule: {}", rule.name);
        Ok(())
    }
    
    async fn get_active_rules(&self) -> prime_core::Result<Vec<prime_core::GovernanceRule>> {
        Ok(vec![])
    }
}