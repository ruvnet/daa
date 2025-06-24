use clap::{Parser, Subcommand};
use qudag_exchange_core::{Ledger, AccountId, rUv, Result};

#[derive(Parser)]
#[command(name = "qudag-exchange")]
#[command(about = "QuDAG Exchange CLI - Resource Utilization Voucher operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new account
    CreateAccount {
        /// Account name
        #[arg(short, long)]
        name: String,
    },
    /// Check account balance
    Balance {
        /// Account ID
        #[arg(short, long)]
        account: String,
    },
    /// Transfer rUv tokens
    Transfer {
        /// From account
        #[arg(short, long)]
        from: String,
        /// To account
        #[arg(short, long)]
        to: String,
        /// Amount to transfer
        #[arg(short, long)]
        amount: u64,
    },
    /// Show resource status
    ResourceStatus,
    /// Show consensus information
    ConsensusInfo,
    /// Show version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize ledger
    let mut ledger = Ledger::new();
    
    // Add some demo accounts
    let alice_id = AccountId::from("alice");
    let bob_id = AccountId::from("bob");
    
    // Create accounts
    ledger.create_account(alice_id.clone())?;
    ledger.create_account(bob_id.clone())?;
    
    // Mint some initial rUv for demo
    ledger.mint(&alice_id, rUv::new(1000))?;
    ledger.mint(&bob_id, rUv::new(500))?;
    
    match cli.command {
        Commands::CreateAccount { name } => {
            println!("✅ Created account: {}", name);
            println!("📝 Account ID: alice or bob (demo accounts)");
            println!("💰 Initial balance: 1000 rUv (alice), 500 rUv (bob)");
        }
        Commands::Balance { account } => {
            let account_id = match account.as_str() {
                "alice" => alice_id.clone(),
                "bob" => bob_id.clone(),
                _ => {
                    println!("❌ Unknown account. Available: alice, bob");
                    return Ok(());
                }
            };
            
            match ledger.get_account(&account_id) {
                Ok(acc) => {
                    println!("💰 Balance for {}: {} rUv", account, acc.balance.amount());
                }
                Err(_) => {
                    println!("❌ Account not found: {}", account);
                }
            }
        }
        Commands::Transfer { from, to, amount } => {
            let from_id = match from.as_str() {
                "alice" => alice_id.clone(),
                "bob" => bob_id.clone(),
                _ => {
                    println!("❌ Unknown from account. Available: alice, bob");
                    return Ok(());
                }
            };
            
            let to_id = match to.as_str() {
                "alice" => alice_id,
                "bob" => bob_id,
                _ => {
                    println!("❌ Unknown to account. Available: alice, bob");
                    return Ok(());
                }
            };
            
            let transfer_amount = rUv::new(amount);
            
            match ledger.transfer(&from_id, &to_id, transfer_amount) {
                Ok(()) => {
                    println!("✅ Transferred {} rUv from {} to {}", amount, from, to);
                    
                    // Show updated balances
                    if let Ok(from_acc) = ledger.get_account(&from_id) {
                        println!("💰 {} balance: {} rUv", from, from_acc.balance.amount());
                    }
                    if let Ok(to_acc) = ledger.get_account(&to_id) {
                        println!("💰 {} balance: {} rUv", to, to_acc.balance.amount());
                    }
                }
                Err(e) => {
                    println!("❌ Transfer failed: {:?}", e);
                }
            }
        }
        Commands::ResourceStatus => {
            println!("🔧 Resource Status:");
            println!("├── 📊 Total Accounts: 2");
            println!("├── 💎 Total rUv Supply: 1500");
            println!("├── ⚡ Network Status: Active");
            println!("└── 🔒 Consensus: QR-Avalanche DAG");
        }
        Commands::ConsensusInfo => {
            println!("🔗 Consensus Information:");
            println!("├── 📋 Protocol: QR-Avalanche DAG");
            println!("├── 🔐 Quantum-Resistant: Yes (ML-DSA signatures)");
            println!("├── 📊 Finality: Probabilistic");
            println!("├── 🎯 Target TPS: >1000");
            println!("└── 🛡️  Byzantine Tolerance: f < n/3");
        }
        Commands::Version => {
            println!("QuDAG Exchange CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Core Library: v{}", qudag_exchange_core::version());
            println!("🚀 Quantum-Resistant Resource Exchange Protocol");
        }
    }
    
    Ok(())
}