//! Prime CLI - Command line interface for Prime distributed ML framework

use clap::{Parser, Subcommand};

/// Prime - Decentralized ML Training Framework
#[derive(Parser)]
#[command(name = "prime")]
#[command(about = "Decentralized ML training framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a trainer node
    Trainer {
        /// Node ID (defaults to random)
        #[arg(short, long)]
        id: Option<String>,
    },
    /// Start a coordinator node
    Coordinator {
        /// Node ID (defaults to random)
        #[arg(short, long)]
        id: Option<String>,
    },
    /// Show system status
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Trainer { id } => {
            let node_id = id.unwrap_or_else(|| format!("trainer-{}", uuid::Uuid::new_v4()));
            println!("Starting trainer node: {}", node_id);
            println!("Prime trainer functionality requires async runtime - use daa-prime-trainer crate directly");
        }
        Commands::Coordinator { id } => {
            let node_id = id.unwrap_or_else(|| format!("coordinator-{}", uuid::Uuid::new_v4()));
            println!("Starting coordinator node: {}", node_id);
            println!("Prime coordinator functionality requires async runtime - use daa-prime-coordinator crate directly");
        }
        Commands::Status => {
            println!("Prime system status:");
            println!("  - System: Ready");
            println!("  - Version: 0.2.0");
            println!("  - Framework: Distributed ML with DAA");
            println!("  - Available crates:");
            println!("    * daa-prime-core v0.2.0");
            println!("    * daa-prime-dht v0.2.0");
            println!("    * daa-prime-trainer v0.2.0");
            println!("    * daa-prime-coordinator v0.2.0");
        }
    }
}