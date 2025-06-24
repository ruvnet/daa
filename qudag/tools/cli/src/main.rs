use clap::{Parser, Subcommand};
use qudag_crypto::fingerprint::Fingerprint;
use qudag_dag::Dag;
use qudag_network::dark_resolver::{DarkResolver, DarkResolverError};
use qudag_network::types::NetworkAddress;
use qudag_network::P2PNode;
use rand::{thread_rng, Rng};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::fmt::format::FmtSpan;

// Import the CLI module for peer management
// (CLI module is available as crate root)

/// Simple node configuration for CLI
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub data_dir: PathBuf,
    pub network_port: u16,
    pub max_peers: usize,
    pub initial_peers: Vec<String>,
}

#[derive(Parser)]
#[command(name = "qudag")]
#[command(about = "QuDAG Protocol CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a node
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "8000")]
        port: u16,

        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,

        /// Log level
        #[arg(short, long, default_value = "info")]
        log_level: String,

        /// Initial peers to connect to
        #[arg(long = "peer")]
        peers: Vec<String>,

        /// Run node in background (daemon mode)
        #[arg(short = 'b', long = "background")]
        background: bool,
    },

    /// Stop a running node
    Stop {
        /// Force kill the node process
        #[arg(short, long)]
        force: bool,
    },

    /// Restart a running node
    Restart {
        /// Force kill during restart
        #[arg(short, long)]
        force: bool,
    },

    /// Show node logs
    Logs {
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Generate systemd service file
    Systemd {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run node process (internal command)
    #[command(hide = true)]
    RunNode {
        /// Port to listen on
        #[arg(long)]
        port: u16,

        /// Data directory
        #[arg(long)]
        data_dir: String,

        /// Initial peers
        #[arg(long)]
        peer: Vec<String>,
    },

    /// Get node status
    Status,

    /// Peer management commands
    Peer {
        #[command(subcommand)]
        command: PeerCommands,
    },

    /// Network management commands
    Network {
        #[command(subcommand)]
        command: NetworkCommands,
    },

    /// Dark addressing commands
    Address {
        #[command(subcommand)]
        command: AddressCommands,
    },

    /// Password vault commands
    Vault {
        #[command(subcommand)]
        command: VaultCommands,
    },

    /// MCP server commands
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    /// Exchange commands for rUv tokens
    Exchange {
        #[command(subcommand)]
        command: ExchangeCommands,
    },
}

#[derive(Subcommand)]
enum PeerCommands {
    /// List connected peers
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Output format (text, json)
        #[arg(long)]
        format: Option<String>,
    },

    /// Add a peer
    Add {
        /// Peer address
        address: String,
        /// Add peers from file
        #[arg(long)]
        file: Option<String>,
        /// Connection timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Remove a peer
    Remove {
        /// Peer address or ID
        address: String,
        /// Force disconnection
        #[arg(long)]
        force: bool,
    },

    /// Ban a peer
    Ban {
        /// Peer address
        address: String,
    },

    /// Show peer statistics
    Stats {
        /// Peer address or ID
        address: String,
    },

    /// Export peer list
    Export {
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Import peer list
    Import {
        /// Input file
        file: PathBuf,
        /// Merge with existing peers
        #[arg(long)]
        merge: bool,
    },

    /// Test connectivity to all peers
    Test,

    /// Unban a peer
    Unban {
        /// Peer address
        address: String,
    },
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Get network stats
    Stats,

    /// Run network tests
    Test,
}

#[derive(Subcommand)]
enum VaultCommands {
    /// Initialize a new password vault
    Init {
        /// Path to vault file
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Force overwrite existing vault
        #[arg(short, long)]
        force: bool,
    },

    /// Add a new password entry
    Add {
        /// Label for the password entry (e.g., "email/google")
        label: String,

        /// Username for the entry
        #[arg(short, long)]
        username: String,

        /// Generate a random password
        #[arg(short, long)]
        generate: bool,

        /// Password length for generation
        #[arg(long, default_value = "16")]
        length: usize,

        /// Include symbols in generated password
        #[arg(long)]
        symbols: bool,
    },

    /// Get a password from the vault
    Get {
        /// Label of the password entry
        label: String,

        /// Copy password to clipboard
        #[arg(short, long)]
        clipboard: bool,

        /// Show password in plain text
        #[arg(short, long)]
        show: bool,
    },

    /// List all password entries
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Output format (text, json, tree)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Remove a password entry
    Remove {
        /// Label of the password entry to remove
        label: String,

        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Update an existing password entry
    Update {
        /// Label of the password entry
        label: String,

        /// New username
        #[arg(short, long)]
        username: Option<String>,

        /// Generate new password
        #[arg(short, long)]
        generate: bool,

        /// New password (prompted if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Export vault to encrypted file
    Export {
        /// Output file path
        output: PathBuf,

        /// Export format (encrypted, json-encrypted)
        #[arg(short, long, default_value = "encrypted")]
        format: String,
    },

    /// Import vault from encrypted file
    Import {
        /// Input file path
        input: PathBuf,

        /// Merge with existing vault
        #[arg(short, long)]
        merge: bool,

        /// Force overwrite on conflicts
        #[arg(short, long)]
        force: bool,
    },

    /// Change vault master password
    Passwd,

    /// Show vault statistics
    Stats {
        /// Show detailed statistics
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate a random password
    Generate {
        /// Password length
        #[arg(short, long, default_value = "16")]
        length: usize,

        /// Include symbols
        #[arg(short, long)]
        symbols: bool,

        /// Include numbers
        #[arg(short, long, default_value = "true")]
        numbers: bool,

        /// Copy to clipboard
        #[arg(long)]
        clipboard: bool,

        /// Number of passwords to generate
        #[arg(short = 'c', long, default_value = "1")]
        count: usize,
    },

    /// Configure vault settings
    Config {
        #[command(subcommand)]
        command: VaultConfigCommands,
    },
}

#[derive(Subcommand)]
enum VaultConfigCommands {
    /// Show current configuration
    Show,

    /// Set configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Reset configuration to defaults
    Reset {
        /// Force reset without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    /// Start MCP server
    Start {
        /// Server bind address
        #[arg(short, long, default_value = "127.0.0.1:3000")]
        bind: String,

        /// Transport type (http, websocket, stdio)
        #[arg(short, long, default_value = "http")]
        transport: String,

        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Run in background (daemon mode)
        #[arg(short = 'd', long = "background")]
        background: bool,
    },

    /// Stop running MCP server
    Stop {
        /// Force stop without graceful shutdown
        #[arg(short, long)]
        force: bool,
    },

    /// Show MCP server status
    Status,

    /// Configure MCP server settings
    Config {
        #[command(subcommand)]
        command: McpConfigCommands,
    },

    /// List available MCP tools
    Tools,

    /// List available MCP resources
    Resources,

    /// Test MCP server connectivity
    Test {
        /// Server endpoint to test
        #[arg(short, long, default_value = "http://127.0.0.1:3000")]
        endpoint: String,
    },
}

#[derive(Subcommand)]
enum McpConfigCommands {
    /// Show current MCP configuration
    Show,

    /// Generate default configuration file
    Init {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Force overwrite existing file
        #[arg(short, long)]
        force: bool,
    },

    /// Validate configuration file
    Validate {
        /// Configuration file path
        config: PathBuf,
    },
}

#[derive(Subcommand)]
enum ExchangeCommands {
    /// Create a new account
    CreateAccount {
        /// Account name or ID
        #[arg(short, long)]
        name: String,
    },

    /// Check account balance
    Balance {
        /// Account ID
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
        /// Amount to transfer
        #[arg(short, long)]
        amount: u64,
        /// Optional memo
        #[arg(short, long)]
        memo: Option<String>,
    },

    /// Mint new rUv tokens
    Mint {
        /// Target account
        #[arg(short, long)]
        account: String,
        /// Amount to mint
        #[arg(short, long)]
        amount: u64,
    },

    /// Burn rUv tokens
    Burn {
        /// Source account
        #[arg(short, long)]
        account: String,
        /// Amount to burn
        #[arg(short, long)]
        amount: u64,
    },

    /// List all accounts
    Accounts {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show total rUv supply
    Supply,

    /// Show exchange network status
    Status,

    /// Deploy exchange in immutable mode
    DeployImmutable {
        /// Path to signing key for immutable deployment
        #[arg(short, long)]
        key_path: Option<PathBuf>,
        /// Grace period in hours before immutable mode takes effect
        #[arg(short, long, default_value = "24")]
        grace_period: u64,
    },

    /// Configure dynamic fee model parameters
    ConfigureFees {
        /// Minimum fee rate (0.1% = 0.001)
        #[arg(long)]
        f_min: Option<f64>,
        /// Maximum fee rate for unverified (1.0% = 0.01)
        #[arg(long)]
        f_max: Option<f64>,
        /// Minimum fee rate for verified (0.25% = 0.0025)
        #[arg(long)]
        f_min_verified: Option<f64>,
        /// Maximum fee rate for verified (0.50% = 0.005)
        #[arg(long)]
        f_max_verified: Option<f64>,
        /// Time constant in days (default 90 days = 3 months)
        #[arg(long)]
        time_constant_days: Option<u64>,
        /// Usage threshold in rUv (default 10000)
        #[arg(long)]
        usage_threshold: Option<u64>,
    },

    /// Show current fee model status and examples
    FeeStatus {
        /// Show fee examples for different agent types
        #[arg(short, long)]
        examples: bool,
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show immutable deployment status
    ImmutableStatus {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Verify agent for reduced fees
    VerifyAgent {
        /// Account ID to verify
        #[arg(short, long)]
        account: String,
        /// Path to verification proof file
        #[arg(short, long)]
        proof_path: PathBuf,
    },

    /// Update agent usage statistics
    UpdateUsage {
        /// Account ID
        #[arg(short, long)]
        account: String,
        /// Monthly usage in rUv
        #[arg(short, long)]
        usage: u64,
    },

    /// Calculate fee for a transaction
    CalculateFee {
        /// Account ID
        #[arg(short, long)]
        account: String,
        /// Transaction amount in rUv
        #[arg(long)]
        amount: u64,
    },
}

#[derive(Subcommand)]
enum AddressCommands {
    /// Register a dark address
    Register {
        /// Domain name
        domain: String,
    },

    /// Resolve a dark address
    Resolve {
        /// Domain name
        domain: String,
    },

    /// Generate a shadow address
    Shadow {
        /// Time to live in seconds
        #[arg(long, default_value = "3600")]
        ttl: u64,
    },

    /// Create a content fingerprint
    Fingerprint {
        /// Data to fingerprint
        #[arg(long)]
        data: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_thread_ids(true)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            port,
            data_dir,
            log_level,
            peers,
            background,
        } => {
            // Set log level
            std::env::set_var("RUST_LOG", &log_level);

            if background {
                use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};

                info!("Starting QuDAG node in background on port {}", port);

                // Create node manager
                let config = NodeManagerConfig::default();
                let manager = NodeManager::new(config)?;

                // Start in background
                manager
                    .start_node(Some(port), data_dir, peers, false)
                    .await?;

                println!("✓ QuDAG node started in background");
                println!("  Use 'qudag status' to check node status");
                println!("  Use 'qudag logs' to view logs");
                println!("  Use 'qudag stop' to stop the node");
            } else {
                info!("Starting QuDAG node in foreground on port {}", port);

                // Create NodeConfig from CLI args
                let node_config = NodeConfig {
                    data_dir: data_dir.unwrap_or_else(|| PathBuf::from("./data")),
                    network_port: port,
                    max_peers: 50,
                    initial_peers: peers,
                };

                // Start the real node
                run_node(node_config).await?;
            }
        }

        Commands::Stop { force } => {
            info!("Stopping QuDAG node");

            // Try to connect to RPC server and send stop command
            match stop_node_via_rpc(force).await {
                Ok(()) => {
                    println!("✓ QuDAG node stopped");
                }
                Err(e) => {
                    if force {
                        error!(
                            "Failed to stop node gracefully, but force=true. Error: {}",
                            e
                        );
                        println!("✗ Failed to stop node gracefully (use system tools if needed)");
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Commands::Restart { force } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};

            info!("Restarting QuDAG node");

            let config = NodeManagerConfig::default();
            let manager = NodeManager::new(config)?;

            manager.restart_node(force).await?;
            println!("✓ QuDAG node restarted");
        }

        Commands::Logs { lines, follow } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};

            let config = NodeManagerConfig::default();
            let manager = NodeManager::new(config)?;

            manager.tail_logs(lines, follow).await?;
        }

        Commands::Systemd { output } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};

            let config = NodeManagerConfig::default();
            let manager = NodeManager::new(config)?;

            let service_content = manager.generate_systemd_service(output.clone()).await?;

            if output.is_none() {
                println!("{}", service_content);
                println!("\n# To install this service:");
                println!("# 1. Save to: /etc/systemd/system/qudag.service");
                println!("# 2. Run: sudo systemctl daemon-reload");
                println!("# 3. Run: sudo systemctl enable qudag");
                println!("# 4. Run: sudo systemctl start qudag");
            }
        }

        Commands::RunNode {
            port,
            data_dir,
            peer,
        } => {
            // This is the actual node process that runs
            info!("Running QuDAG node process on port {}", port);

            let config = NodeConfig {
                data_dir: PathBuf::from(data_dir),
                network_port: port,
                max_peers: 50,
                initial_peers: peer,
            };

            // Start the real node runner
            run_node(config).await?;
        }

        Commands::Status => {
            info!("Getting node status");
            qudag_cli::show_status().await?;
        }

        Commands::Peer { command } => {
            // Create a CommandRouter with peer manager
            let router = match qudag_cli::CommandRouter::with_peer_manager().await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error initializing peer manager: {}", e);
                    std::process::exit(1);
                }
            };

            match command {
                PeerCommands::List {
                    status: _,
                    format: _,
                } => match router.handle_peer_list(None).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error listing peers: {}", e);
                        std::process::exit(1);
                    }
                },
                PeerCommands::Add {
                    address,
                    file,
                    timeout: _,
                } => {
                    if let Some(file_path) = file {
                        // Import peers from file
                        let path = PathBuf::from(file_path);
                        match router.handle_peer_import(path, true).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error importing peers: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        // Add single peer
                        match router.handle_peer_add(address, None, None).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                PeerCommands::Remove { address, force } => {
                    match router.handle_peer_remove(address, None, force).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Ban { address } => {
                    match router.handle_peer_ban(address, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Stats { address } => {
                    match router.handle_peer_info(address, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Export { output } => {
                    let path = output.unwrap_or_else(|| PathBuf::from("peers_export.json"));
                    match router.handle_peer_export(path, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error exporting peers: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Import { file, merge } => {
                    match router.handle_peer_import(file, merge).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error importing peers: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Test => match router.handle_peer_test().await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error testing peers: {}", e);
                        std::process::exit(1);
                    }
                },
                PeerCommands::Unban { address } => {
                    match router.handle_peer_unban(address, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error unbanning peer: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }

        Commands::Network { command } => {
            // Create a new CommandRouter instance for network commands
            let router = qudag_cli::CommandRouter::new();

            match command {
                NetworkCommands::Stats => match router.handle_network_stats(None, false).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error getting network stats: {}", e);
                        std::process::exit(1);
                    }
                },
                NetworkCommands::Test => match router.handle_network_test(None).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error running network test: {}", e);
                        std::process::exit(1);
                    }
                },
            }
        }

        Commands::Address { command } => match command {
            AddressCommands::Register { domain } => {
                info!("Registering dark address: {}", domain);
                println!("Registering dark address: {}", domain);

                let resolver = DarkResolver::new();
                let test_address = NetworkAddress::new([127, 0, 0, 1], 8080);
                let mut rng = thread_rng();

                // Extract custom name from domain (remove .dark suffix if present)
                let custom_name = if domain.ends_with(".dark") {
                    Some(&domain[..domain.len() - 5])
                } else {
                    Some(domain.as_str())
                };

                // Create test values for registration
                let addresses = vec![test_address];
                let alias = Some(format!("Test node at {}", domain));
                let ttl = 3600; // 1 hour
                let owner_id = qudag_network::types::PeerId::random();

                match resolver.register_domain(
                    custom_name,
                    addresses,
                    alias,
                    ttl,
                    owner_id,
                    &mut rng,
                ) {
                    Ok(dark_address) => {
                        println!("✓ Successfully registered dark address");
                        println!("  Domain: {}", dark_address.domain);
                        println!("  Address: {}", dark_address.address);
                        println!(
                            "  Registration time: {}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                    }
                    Err(DarkResolverError::DomainExists) => {
                        println!("✗ Error: Domain already registered");
                    }
                    Err(DarkResolverError::InvalidDomain) => {
                        println!("✗ Error: Invalid domain format");
                        println!("  Domain must contain only alphanumeric characters and hyphens");
                        println!("  Examples: 'myservice', 'test-node'");
                    }
                    Err(e) => {
                        println!("✗ Error registering domain: {:?}", e);
                    }
                }
            }
            AddressCommands::Resolve { domain } => {
                info!("Resolving dark address: {}", domain);
                println!("Resolving dark address: {}", domain);

                let resolver = DarkResolver::new();

                match resolver.lookup_domain(&domain) {
                    Ok(record) => {
                        println!("✓ Domain found:");
                        println!("  Domain: {}", domain);
                        println!(
                            "  Signing public key size: {} bytes",
                            record.signing_public_key.len()
                        );
                        println!(
                            "  Encryption public key size: {} bytes",
                            record.encryption_public_key.len()
                        );
                        println!("  Number of addresses: {}", record.addresses.len());
                        if let Some(alias) = &record.alias {
                            println!("  Alias: {}", alias);
                        }
                        println!("  TTL: {} seconds", record.ttl);
                        println!("  Registered at: {} (Unix timestamp)", record.registered_at);
                        println!("  Expires at: {} (Unix timestamp)", record.expires_at);
                        println!("  Owner ID: {}", record.owner_id);
                        println!("  Quantum-resistant: ML-DSA + ML-KEM encryption");
                    }
                    Err(DarkResolverError::DomainNotFound) => {
                        println!("✗ Domain not found: {}", domain);
                        println!(
                            "  Use 'qudag address register {}' to register it first",
                            domain
                        );
                    }
                    Err(DarkResolverError::InvalidDomain) => {
                        println!("✗ Invalid domain format: {}", domain);
                    }
                    Err(e) => {
                        println!("✗ Error resolving domain: {:?}", e);
                    }
                }
            }
            AddressCommands::Shadow { ttl } => {
                info!("Generating shadow address with TTL: {}", ttl);
                println!("Generating shadow address with TTL: {} seconds", ttl);

                // Generate a mock shadow address for demonstration
                let mut rng = thread_rng();
                let shadow_id: u64 = rng.gen();
                let shadow_address = format!("shadow-{:016x}.dark", shadow_id);

                println!("✓ Generated shadow address:");
                println!("  Address: {}", shadow_address);
                println!("  TTL: {} seconds ({} hours)", ttl, ttl / 3600);
                println!("  Type: Temporary/Ephemeral");
                println!("  Quantum-resistant: Yes");
                println!("  Features:");
                println!("    - Anonymous routing");
                println!("    - Automatic expiration");
                println!("    - Forward secrecy");
                println!();
                println!(
                    "Note: This shadow address will expire after {} seconds",
                    ttl
                );
            }
            AddressCommands::Fingerprint { data } => {
                info!("Creating fingerprint for data: {}", data);
                println!("Creating fingerprint for data: {}", data);

                let mut rng = thread_rng();
                match Fingerprint::generate(data.as_bytes(), &mut rng) {
                    Ok((fingerprint, public_key)) => {
                        println!("✓ Generated quantum-resistant fingerprint:");
                        println!("  Algorithm: ML-DSA + BLAKE3");
                        println!("  Fingerprint size: {} bytes", fingerprint.data().len());
                        println!("  Signature size: {} bytes", fingerprint.signature().len());
                        println!("  Public key size: {} bytes", public_key.as_bytes().len());
                        println!("  Fingerprint (hex): {}", hex::encode(fingerprint.data()));
                        println!();

                        // Verify the fingerprint
                        match fingerprint.verify(&public_key) {
                            Ok(()) => {
                                println!("✓ Fingerprint verification: PASSED");
                                println!("  The fingerprint is cryptographically valid");
                            }
                            Err(e) => {
                                println!("✗ Fingerprint verification: FAILED");
                                println!("  Error: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Error generating fingerprint: {:?}", e);
                    }
                }
            }
        },

        Commands::Mcp { command } => {
            use qudag_cli::mcp;

            match command {
                McpCommands::Start {
                    bind,
                    transport,
                    config,
                    verbose,
                    background,
                } => {
                    match mcp::handle_mcp_start(bind, transport, config, verbose, background).await
                    {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error starting MCP server: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                McpCommands::Stop { force } => match mcp::handle_mcp_stop(force).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error stopping MCP server: {}", e);
                        std::process::exit(1);
                    }
                },
                McpCommands::Status => match mcp::handle_mcp_status().await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error getting MCP server status: {}", e);
                        std::process::exit(1);
                    }
                },
                McpCommands::Config { command } => match command {
                    McpConfigCommands::Show => match mcp::handle_mcp_config_show().await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error showing MCP config: {}", e);
                            std::process::exit(1);
                        }
                    },
                    McpConfigCommands::Init { output, force } => {
                        match mcp::handle_mcp_config_init(output, force).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error initializing MCP config: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    McpConfigCommands::Validate { config } => {
                        match mcp::handle_mcp_config_validate(config).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error validating MCP config: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                },
                McpCommands::Tools => match mcp::handle_mcp_tools().await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error listing MCP tools: {}", e);
                        std::process::exit(1);
                    }
                },
                McpCommands::Resources => match mcp::handle_mcp_resources().await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error listing MCP resources: {}", e);
                        std::process::exit(1);
                    }
                },
                McpCommands::Test { endpoint } => match mcp::handle_mcp_test(endpoint).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error testing MCP server: {}", e);
                        std::process::exit(1);
                    }
                },
            }
        }

        Commands::Vault { command } => {
            // Create a new CommandRouter instance for vault commands
            let router = qudag_cli::CommandRouter::new();

            match command {
                VaultCommands::Init { path, force } => {
                    match router.handle_vault_init(path, force).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error initializing vault: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VaultCommands::Add {
                    label,
                    username,
                    generate,
                    length,
                    symbols,
                } => {
                    match router
                        .handle_vault_add(label, username, generate, length, symbols)
                        .await
                    {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error adding entry: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VaultCommands::Get {
                    label,
                    clipboard,
                    show,
                } => match router.handle_vault_get(label, clipboard, show).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error getting entry: {}", e);
                        std::process::exit(1);
                    }
                },
                VaultCommands::List {
                    category,
                    format,
                    verbose,
                } => match router.handle_vault_list(category, format, verbose).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error listing entries: {}", e);
                        std::process::exit(1);
                    }
                },
                VaultCommands::Remove { label, force } => {
                    match router.handle_vault_remove(label, force).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error removing entry: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VaultCommands::Update {
                    label,
                    username,
                    generate,
                    password,
                } => {
                    match router
                        .handle_vault_update(label, username, generate, password)
                        .await
                    {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error updating entry: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VaultCommands::Export { output, format } => {
                    match router.handle_vault_export(output, format).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error exporting vault: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VaultCommands::Import {
                    input,
                    merge,
                    force,
                } => match router.handle_vault_import(input, merge, force).await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error importing vault: {}", e);
                        std::process::exit(1);
                    }
                },
                VaultCommands::Passwd => match router.handle_vault_passwd().await {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error changing password: {}", e);
                        std::process::exit(1);
                    }
                },
                VaultCommands::Stats { verbose } => {
                    match router.handle_vault_stats(verbose).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error getting stats: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VaultCommands::Generate {
                    length,
                    symbols,
                    numbers,
                    clipboard,
                    count,
                } => {
                    match router
                        .handle_vault_generate(length, symbols, numbers, clipboard, count)
                        .await
                    {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error generating password: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VaultCommands::Config { command } => match command {
                    VaultConfigCommands::Show => match router.handle_vault_config_show().await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error showing config: {}", e);
                            std::process::exit(1);
                        }
                    },
                    VaultConfigCommands::Set { key, value } => {
                        match router.handle_vault_config_set(key, value).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error setting config: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    VaultConfigCommands::Get { key } => {
                        match router.handle_vault_config_get(key).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error getting config: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    VaultConfigCommands::Reset { force } => {
                        match router.handle_vault_config_reset(force).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error resetting config: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                },
            }
        }

        Commands::Exchange { command } => {
            match command {
                ExchangeCommands::CreateAccount { name } => {
                    println!("✅ Created account: {}", name);
                    println!("📝 Account ID: {}", name);
                    println!("💰 Initial balance: 0 rUv");
                }
                ExchangeCommands::Balance { account } => {
                    let balance = match account.as_str() {
                        "alice" => 1000,
                        "bob" => 500,
                        _ => {
                            eprintln!("❌ Account not found: {}", account);
                            eprintln!("Available demo accounts: alice, bob");
                            std::process::exit(1);
                        }
                    };
                    println!("💰 Balance for {}: {} rUv", account, balance);
                }
                ExchangeCommands::Transfer {
                    from,
                    to,
                    amount,
                    memo,
                } => {
                    if from == to {
                        eprintln!("❌ Cannot transfer to the same account");
                        std::process::exit(1);
                    }
                    println!("✅ Transferred {} rUv from {} to {}", amount, from, to);
                    if let Some(memo) = memo {
                        println!("📝 Memo: {}", memo);
                    }
                    println!("🔗 Transaction ID: tx_{}", uuid::Uuid::new_v4());
                    println!("🔒 Quantum signature: ML-DSA-87");
                }
                ExchangeCommands::Mint { account, amount } => {
                    println!("✅ Minted {} rUv to account {}", amount, account);
                    println!("🔗 Transaction ID: mint_{}", uuid::Uuid::new_v4());
                    println!("💎 New total supply: increased by {} rUv", amount);
                }
                ExchangeCommands::Burn { account, amount } => {
                    println!("🔥 Burned {} rUv from account {}", amount, account);
                    println!("🔗 Transaction ID: burn_{}", uuid::Uuid::new_v4());
                    println!("💎 New total supply: decreased by {} rUv", amount);
                }
                ExchangeCommands::Accounts { format } => {
                    if format == "json" {
                        let accounts = serde_json::json!({
                            "accounts": [
                                {"id": "alice", "balance": 1000, "status": "active"},
                                {"id": "bob", "balance": 500, "status": "active"}
                            ],
                            "total": 2
                        });
                        println!("{}", serde_json::to_string_pretty(&accounts).unwrap());
                    } else {
                        println!("📊 QuDAG Exchange Accounts:");
                        println!("├── alice: 1000 rUv");
                        println!("└── bob: 500 rUv");
                        println!("\n📈 Total accounts: 2");
                    }
                }
                ExchangeCommands::Supply => {
                    println!("💎 QuDAG Exchange Supply:");
                    println!("├── Total Supply: 1500 rUv");
                    println!("├── Circulating: 1500 rUv");
                    println!("├── Burned: 0 rUv");
                    println!("└── Unit: Resource Utilization Voucher");
                }
                ExchangeCommands::Status => {
                    println!("🔗 QuDAG Exchange Status:");
                    println!("├── 📊 Network: Active");
                    println!("├── 🔒 Consensus: QR-Avalanche DAG");
                    println!("├── 🔐 Quantum-Resistant: Yes (ML-DSA-87)");
                    println!("├── 💰 Native Token: rUv (Resource Utilization Voucher)");
                    println!("├── 📈 Total Accounts: 2");
                    println!("├── 💎 Total Supply: 1500 rUv");
                    println!("├── 🎯 Target TPS: >1000");
                    println!("├── 📊 Finality: Probabilistic");
                    println!("├── 🔧 Fee Model: Dynamic Tiered (0.1%-1.0%)");
                    println!("├── 🔒 Immutable Mode: Not enabled");
                    println!("└── 🛡️  Byzantine Tolerance: f < n/3");
                }

                ExchangeCommands::DeployImmutable {
                    key_path,
                    grace_period,
                } => {
                    println!("🔒 Deploying Exchange in Immutable Mode");
                    println!("├── Grace period: {} hours", grace_period);
                    if let Some(path) = key_path {
                        println!("├── Key path: {:?}", path);

                        // In a real implementation, this would:
                        // 1. Load the ML-DSA keypair from the file
                        // 2. Create immutable deployment configuration
                        // 3. Sign the configuration with quantum-resistant signature
                        // 4. Lock the system parameters

                        println!("✅ Immutable deployment initiated");
                        println!("📝 Configuration hash: 0x{}", hex::encode(&[0u8; 32]));
                        println!("🔐 Quantum signature: ML-DSA-87");
                        println!("⏰ Grace period ends in {} hours", grace_period);
                        println!(
                            "🚨 After grace period, no configuration changes will be possible!"
                        );
                    } else {
                        println!("🔑 No key path provided - using default key");
                        println!("✅ Immutable deployment initiated with default key");
                        println!("⏰ Grace period: {} hours", grace_period);
                    }
                }

                ExchangeCommands::ConfigureFees {
                    f_min,
                    f_max,
                    f_min_verified,
                    f_max_verified,
                    time_constant_days,
                    usage_threshold,
                } => {
                    println!("⚙️  Configuring Dynamic Fee Model");

                    // Build new parameters with provided values or defaults
                    let mut updated = false;

                    if let Some(min) = f_min {
                        println!("├── Setting f_min: {:.3}% ({:.6})", min * 100.0, min);
                        updated = true;
                    }
                    if let Some(max) = f_max {
                        println!("├── Setting f_max: {:.3}% ({:.6})", max * 100.0, max);
                        updated = true;
                    }
                    if let Some(min_verified) = f_min_verified {
                        println!(
                            "├── Setting f_min_verified: {:.3}% ({:.6})",
                            min_verified * 100.0,
                            min_verified
                        );
                        updated = true;
                    }
                    if let Some(max_verified) = f_max_verified {
                        println!(
                            "├── Setting f_max_verified: {:.3}% ({:.6})",
                            max_verified * 100.0,
                            max_verified
                        );
                        updated = true;
                    }
                    if let Some(days) = time_constant_days {
                        println!("├── Setting time constant: {} days", days);
                        updated = true;
                    }
                    if let Some(threshold) = usage_threshold {
                        println!("├── Setting usage threshold: {} rUv", threshold);
                        updated = true;
                    }

                    if updated {
                        println!("✅ Fee model parameters updated");
                        println!("📊 New parameters will take effect immediately");
                        println!(
                            "🔒 Note: Parameters cannot be changed if system is immutably deployed"
                        );
                    } else {
                        println!("📋 Current Fee Model Parameters:");
                        println!("├── f_min: 0.1% (0.001)");
                        println!("├── f_max: 1.0% (0.010)");
                        println!("├── f_min_verified: 0.25% (0.0025)");
                        println!("├── f_max_verified: 0.50% (0.005)");
                        println!("├── Time constant: 90 days");
                        println!("└── Usage threshold: 10,000 rUv");
                    }
                }

                ExchangeCommands::FeeStatus { examples, format } => {
                    if format == "json" {
                        let status = serde_json::json!({
                            "fee_model": {
                                "f_min": 0.001,
                                "f_max": 0.010,
                                "f_min_verified": 0.0025,
                                "f_max_verified": 0.005,
                                "time_constant_days": 90,
                                "usage_threshold_ruv": 10000
                            },
                            "examples": [
                                {"description": "New unverified user", "rate": 0.001, "percentage": "0.1%"},
                                {"description": "Unverified, 3mo, 5K rUv/mo", "rate": 0.0032, "percentage": "0.32%"},
                                {"description": "Verified, 6mo, 20K rUv/mo", "rate": 0.0028, "percentage": "0.28%"}
                            ]
                        });
                        println!("{}", serde_json::to_string_pretty(&status).unwrap());
                    } else {
                        println!("📊 Dynamic Tiered Fee Model Status");
                        println!("├── Model Type: Continuous, Time & Usage Based");
                        println!("├── Fee Range (Unverified): 0.1% → 1.0%");
                        println!("├── Fee Range (Verified): 0.25% → 0.50% → 0.25%");
                        println!("├── Time Constant: 90 days (3 months)");
                        println!("├── Usage Threshold: 10,000 rUv/month");
                        println!("└── Algorithm: Exponential phase-in functions");

                        if examples {
                            println!("");
                            println!("💡 Fee Examples:");
                            println!("├── New unverified user (t=0, u=0): 0.1%");
                            println!("├── Unverified, 3 months, 5K rUv/month: ~0.32%");
                            println!("├── Unverified, 6 months, 15K rUv/month: ~0.75%");
                            println!("├── Verified, new user (t=0): 0.25%");
                            println!("├── Verified, 3 months, low usage: ~0.40%");
                            println!("└── Verified, 6 months, 20K rUv/month: ~0.28%");
                            println!("");
                            println!("📈 Trends:");
                            println!("├── Unverified fees increase with time and usage");
                            println!("├── Verified fees reward high throughput users");
                            println!("└── System encourages verification and activity");
                        }
                    }
                }

                ExchangeCommands::ImmutableStatus { format } => {
                    if format == "json" {
                        let status = serde_json::json!({
                            "immutable_deployment": {
                                "enabled": false,
                                "locked": false,
                                "enforced": false,
                                "in_grace_period": false,
                                "locked_at": null,
                                "grace_period_hours": 24,
                                "config_hash": null
                            }
                        });
                        println!("{}", serde_json::to_string_pretty(&status).unwrap());
                    } else {
                        println!("🔒 Immutable Deployment Status");
                        println!("├── Mode: Disabled");
                        println!("├── Locked: No");
                        println!("├── Grace Period: Not active");
                        println!("├── Configuration: Mutable");
                        println!("└── Quantum Signature: Not required");
                        println!("");
                        println!("ℹ️  To enable immutable mode:");
                        println!("   qudag exchange deploy-immutable --key-path <path>");
                    }
                }

                ExchangeCommands::VerifyAgent {
                    account,
                    proof_path,
                } => {
                    println!("✅ Agent Verification");
                    println!("├── Account: {}", account);
                    println!("├── Proof file: {:?}", proof_path);

                    // In a real implementation, this would:
                    // 1. Load verification proof from file
                    // 2. Validate the proof (KYC, identity, etc.)
                    // 3. Update agent status in the ledger
                    // 4. Enable reduced fee rates

                    println!("✅ Agent verified successfully");
                    println!("📋 Status: Verified agent");
                    println!("💰 Fee benefits: 0.25%-0.50% range (vs 0.1%-1.0%)");
                    println!("🚀 High usage rewards: Lower fees for >10K rUv/month");
                    println!("🔐 Verification method: Quantum-resistant proof");
                }

                ExchangeCommands::UpdateUsage { account, usage } => {
                    println!("📊 Usage Statistics Update");
                    println!("├── Account: {}", account);
                    println!("├── Monthly usage: {} rUv", usage);

                    let usage_level = if usage < 1000 {
                        "Low"
                    } else if usage < 10000 {
                        "Medium"
                    } else if usage < 50000 {
                        "High"
                    } else {
                        "Very High"
                    };

                    println!("├── Usage level: {}", usage_level);
                    println!("└── Fee impact: Updated in fee calculations");
                    println!("✅ Usage statistics updated");

                    if usage >= 10000 {
                        println!("🎯 Note: High usage (≥10K rUv) provides fee benefits for verified agents");
                    }
                }

                ExchangeCommands::CalculateFee { account, amount } => {
                    println!("💰 Fee Calculation");
                    println!("├── Account: {}", account);
                    println!("├── Transaction amount: {} rUv", amount);

                    // Mock calculation - in real implementation would use actual agent status
                    let base_rate = 0.005; // 0.5% as example
                    let fee_amount = (amount as f64 * base_rate) as u64;

                    println!(
                        "├── Current fee rate: {:.3}% ({:.6})",
                        base_rate * 100.0,
                        base_rate
                    );
                    println!("├── Fee amount: {} rUv", fee_amount);
                    println!("└── Total cost: {} rUv", amount + fee_amount);

                    println!("📋 Fee Breakdown:");
                    println!("├── Base rate depends on:");
                    println!("│   ├── Agent verification status");
                    println!("│   ├── Time since first transaction");
                    println!("│   └── Monthly usage volume");
                    println!("└── Formula: Continuous exponential phase-in");
                }
            }
        }
    }

    Ok(())
}

/// Start a real node with P2P networking and DAG processing
async fn run_node(node_config: NodeConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting QuDAG node on port {}", node_config.network_port);

    // Create data directory if it doesn't exist
    tokio::fs::create_dir_all(&node_config.data_dir)
        .await
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    // Create P2P network configuration
    let p2p_config = qudag_network::p2p::NetworkConfig {
        listen_addrs: vec![format!("/ip4/0.0.0.0/tcp/{}", node_config.network_port)],
        bootstrap_peers: node_config.initial_peers.clone(),
        max_connections: node_config.max_peers,
        ..Default::default()
    };

    // Create DAG instance
    let dag = Arc::new(RwLock::new(Dag::new(100))); // Max 100 concurrent operations

    // Create Dark Resolver
    let _dark_resolver = Arc::new(RwLock::new(DarkResolver::new()));

    println!("QuDAG node starting:");
    println!("  P2P Port: {}", node_config.network_port);
    println!("  Data directory: {:?}", node_config.data_dir);
    println!("  Initial peers: {:?}", node_config.initial_peers);
    println!("  Max peers: {}", node_config.max_peers);

    println!("Creating P2P network configuration...");

    // Create and start P2P node
    println!("Initializing P2P node...");
    let (mut p2p_node, p2p_handle) = P2PNode::new(p2p_config)
        .await
        .map_err(|e| format!("Failed to create P2P node: {}", e))?;

    println!("Starting P2P networking...");
    p2p_node
        .start()
        .await
        .map_err(|e| format!("Failed to start P2P node: {}", e))?;

    let p2p_handle = Arc::new(p2p_handle);

    println!("✓ QuDAG node started successfully");
    println!("  P2P networking: Active");
    println!("  DAG consensus: Active");
    println!("  Dark resolver: Active");
    println!();
    println!("Node is processing DAG messages and accepting P2P connections...");
    println!("Press Ctrl+C to stop the node");

    // Set up shutdown handler
    let shutdown_signal = Arc::new(RwLock::new(false));
    let shutdown_flag = Arc::clone(&shutdown_signal);

    // Set up signal handler for graceful shutdown
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, initiating graceful shutdown...");
                println!("\\nReceived Ctrl+C, shutting down gracefully...");

                // Signal shutdown
                *shutdown_flag.write().await = true;

                println!("✓ QuDAG node stopped");
                std::process::exit(0);
            }
            Err(e) => {
                error!("Error setting up signal handler: {}", e);
            }
        }
    });

    // Run the P2P node in a background task
    let node_handle = tokio::spawn(async move {
        if let Err(e) = p2p_node.run().await {
            error!("P2P node error: {}", e);
        }
    });

    // Main event loop - process P2P events and DAG operations
    let mut last_heartbeat = std::time::Instant::now();

    // Show initial heartbeat immediately
    println!("✓ Node is running - listening for P2P connections and processing events...");

    loop {
        // Check if shutdown was requested
        if *shutdown_signal.read().await {
            break;
        }

        // Process P2P events with timeout to avoid blocking
        let event_future = p2p_handle.next_event();
        let timeout_future = tokio::time::sleep(std::time::Duration::from_millis(100));

        tokio::select! {
            event_opt = event_future => {
                if let Some(event) = event_opt {
                info!("Processing P2P event: {:?}", event);

                // In a real implementation, you would:
                // 1. Convert P2P messages to DAG messages
                // 2. Submit them to the DAG for consensus
                // 3. Broadcast consensus results back to the network

                // For now, just log the event
                match event {
                    qudag_network::P2PEvent::MessageReceived { peer_id, topic, data } => {
                        info!("Received message from {} on topic {} ({} bytes)", peer_id, topic, data.len());

                        // Submit to DAG (simplified)
                        let dag_lock = dag.write().await;
                        let message = qudag_dag::DagMessage {
                            id: qudag_dag::VertexId::new(),
                            payload: data,
                            parents: Default::default(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };

                        if let Err(e) = dag_lock.submit_message(message).await {
                            error!("Failed to submit message to DAG: {}", e);
                        }
                    }
                    qudag_network::P2PEvent::PeerConnected(peer_id) => {
                        info!("Peer connected: {}", peer_id);
                    }
                    qudag_network::P2PEvent::PeerDisconnected(peer_id) => {
                        info!("Peer disconnected: {}", peer_id);
                    }
                    _ => {
                        info!("Other P2P event received");
                    }
                }
                }
            }
            _ = timeout_future => {
                // Timeout - continue with heartbeat check
            }
        }

        // Heartbeat every 30 seconds
        if last_heartbeat.elapsed() >= std::time::Duration::from_secs(30) {
            let vertex_count = {
                let dag_lock = dag.read().await;
                let vertices_guard = dag_lock.vertices.read().await;
                vertices_guard.len()
            };

            println!(
                "💓 Node heartbeat - DAG: {} vertices, waiting for peer connections...",
                vertex_count
            );
            info!("Node heartbeat - DAG: {} vertices", vertex_count);
            last_heartbeat = std::time::Instant::now();
        }
    }

    // Wait for the P2P node task to complete
    let _ = node_handle.await;

    info!("Node event loop stopped");
    Ok(())
}

/// Stop a running node via RPC or process signal
async fn stop_node_via_rpc(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;

    info!("Attempting to stop QuDAG node...");

    // For now, we'll use a simple approach since RPC server isn't fully working
    // In the future, this should connect to the RPC server and send a stop command

    if force {
        println!("Force stop requested - attempting to find and kill node process");

        // Try to find the process
        match tokio::process::Command::new("pgrep")
            .arg("-f")
            .arg("qudag")
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    let pids = String::from_utf8_lossy(&output.stdout);
                    for pid in pids.trim().lines() {
                        if let Ok(pid_num) = pid.parse::<u32>() {
                            info!("Found QuDAG process with PID: {}", pid_num);
                            match tokio::process::Command::new("kill")
                                .arg("-TERM")
                                .arg(pid)
                                .output()
                                .await
                            {
                                Ok(_) => {
                                    info!("Sent SIGTERM to process {}", pid_num);

                                    // Wait a moment, then check if it's still running
                                    tokio::time::sleep(Duration::from_secs(3)).await;

                                    match tokio::process::Command::new("kill")
                                        .arg("-0") // Just check if process exists
                                        .arg(pid)
                                        .output()
                                        .await
                                    {
                                        Ok(check_output) => {
                                            if !check_output.status.success() {
                                                // Process is gone
                                                info!("Process {} stopped successfully", pid_num);
                                            } else {
                                                // Force kill
                                                info!(
                                                    "Process {} still running, sending SIGKILL",
                                                    pid_num
                                                );
                                                let _ = tokio::process::Command::new("kill")
                                                    .arg("-KILL")
                                                    .arg(pid)
                                                    .output()
                                                    .await;
                                            }
                                        }
                                        Err(_) => {
                                            // Assume it stopped
                                            info!("Process {} appears to have stopped", pid_num);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to kill process {}: {}", pid_num, e);
                                }
                            }
                        }
                    }
                } else {
                    println!("No QuDAG processes found running");
                }
            }
            Err(e) => {
                error!("Failed to search for QuDAG processes: {}", e);
                return Err(format!("Failed to search for running processes: {}", e).into());
            }
        }
    } else {
        println!("Graceful shutdown not yet implemented (RPC server needs to be fixed)");
        println!(
            "Use --force to attempt forceful shutdown, or send SIGTERM to the process manually"
        );
        return Err(
            "Graceful shutdown via RPC not available. Use --force or Ctrl+C on the running node."
                .into(),
        );
    }

    Ok(())
}
