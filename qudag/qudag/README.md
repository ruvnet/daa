# QuDAG: The Future of Autonomous, Quantum-Resistant, Zero-Person Businesses

[![Crates.io](https://img.shields.io/crates/v/qudag.svg)](https://crates.io/crates/qudag)
[![Documentation](https://docs.rs/qudag/badge.svg)](https://docs.rs/qudag)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/ruvnet/QuDAG)

**QuDAG** is a revolutionary **quantum-resistant distributed communication platform** built for the quantum age, designed to support the next generation of **autonomous AI agents**, **swarm intelligence**, and **zero-person businesses**. This is the core library that provides the foundation for building **Agentic Organizations**—where AI-driven systems run businesses entirely on their own.

## 🚀 The Platform for Autonomous Businesses

QuDAG enables businesses to operate **autonomously** and **decentrally**, without human intervention. The platform provides:

* **Quantum-Resistant Security**: ML-KEM-768 & ML-DSA cryptography protects against future quantum threats
* **Agent Swarm Coordination**: Built-in support for autonomous AI agent communication and coordination
* **Resource Exchange Economy**: rUv tokens enable agents to trade computational resources
* **Zero-Person Operations**: Immutable deployments for stable, long-term autonomous operations
* **Decentralized Infrastructure**: P2P network ensures resilience without central points of failure

## 🌟 Key Features

### Core Components

* **Quantum-Resistant Cryptography** - ML-KEM-768, ML-DSA, HQC, and BLAKE3
* **DAG Consensus** - QR-Avalanche algorithm for Byzantine fault tolerance
* **Dark Addressing** - Decentralized .dark domain system
* **Anonymous Routing** - Multi-hop onion routing with ChaCha20Poly1305
* **Resource Exchange** - rUv token system with dynamic fee models
* **MCP Integration** - Native Model Context Protocol server for AI agents
* **Password Vault** - Quantum-resistant credential management
* **P2P Networking** - LibP2P with Kademlia DHT

## 📦 Installation

Add QuDAG to your Rust project:

```toml
[dependencies]
qudag = "1.3"
```

Or install specific components:

```toml
[dependencies]
qudag-crypto = "0.4"      # Quantum-resistant cryptography
qudag-network = "0.4"     # P2P networking
qudag-dag = "0.4"         # DAG consensus
qudag-exchange-core = "0.3"  # Resource exchange
```

## 🚀 Quick Start

### Basic Usage

```rust
use qudag::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create quantum-resistant keys
    let keypair = MlDsaKeyPair::generate()?;
    
    // Initialize DAG
    let dag = Dag::new();
    
    // Create network manager
    let network = NetworkManager::new()?;
    
    // Initialize exchange for resource trading
    let exchange = Exchange::new()?;
    
    println!("QuDAG node ready for autonomous operations!");
    Ok(())
}
```

### Autonomous Agent Example

```rust
use qudag::{Exchange, NetworkManager, MlDsaKeyPair};

// Create an autonomous agent
pub struct AutonomousAgent {
    keypair: MlDsaKeyPair,
    exchange: Exchange,
    network: NetworkManager,
}

impl AutonomousAgent {
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Register on the network
        self.network.register_domain("agent.dark").await?;
        
        // Create exchange account
        let account = self.exchange.create_account("autonomous-agent")?;
        
        // Start trading resources
        loop {
            // Buy computational resources when needed
            if self.needs_resources() {
                self.exchange.transfer(
                    &account,
                    &resource_provider,
                    rUv::new(100),
                    Some("Buying CPU time")
                )?;
            }
            
            // Sell excess resources
            if self.has_excess_resources() {
                self.exchange.transfer(
                    &other_agent,
                    &account,
                    rUv::new(50),
                    Some("Selling bandwidth")
                )?;
            }
            
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
```

### Zero-Person Business Example

```rust
use qudag::{BusinessConfig, Exchange, ImmutableDeployment};

// Configure a zero-person business
let config = BusinessConfig::builder()
    .enable_auto_distribution(true)
    .enable_vault_management(true)
    .enable_role_earnings(true)
    .set_payout_threshold(rUv::new(100))
    .build()?;

// Deploy immutably for autonomous operation
let deployment = ImmutableDeployment::new(config)
    .with_grace_period(Duration::from_hours(24))
    .lock(keypair)?;

// Business now runs autonomously without human intervention
```

## 💡 Use Cases

### Autonomous AI Services
Build AI agents that provide services and generate revenue independently:
- Customer service bots that earn tokens per interaction
- Content generation agents that sell their output
- Analysis agents that trade insights for resources

### Decentralized Compute Markets
Create marketplaces where agents trade computational resources:
- CPU/GPU time for AI training
- Storage space for distributed data
- Bandwidth for content delivery

### Self-Sustaining Infrastructure
Deploy infrastructure that maintains itself:
- Nodes that earn revenue to pay for hosting
- Services that scale based on demand
- Systems that upgrade autonomously

## 🔧 Core Modules

### Cryptography (`qudag-crypto`)
```rust
use qudag_crypto::{MlKem768, MlDsaKeyPair, Blake3Hash};

// Quantum-resistant encryption
let (ciphertext, shared_secret) = MlKem768::encapsulate(&public_key)?;

// Digital signatures
let signature = keypair.sign(message)?;

// Quantum-resistant hashing
let hash = Blake3Hash::hash(data);
```

### Networking (`qudag-network`)
```rust
use qudag_network::{NetworkManager, DarkResolver};

// P2P networking
let network = NetworkManager::new()?;
network.listen_on("/ip4/0.0.0.0/tcp/8000").await?;

// Dark addressing
network.register_domain("myservice.dark").await?;
let addresses = network.resolve_domain("other.dark").await?;
```

### DAG Consensus (`qudag-dag`)
```rust
use qudag_dag::{Dag, QRAvalanche};

// Create DAG with consensus
let mut dag = Dag::with_consensus(QRAvalanche::new());

// Add vertices
let vertex = dag.create_vertex(message, parents)?;
dag.add_vertex(vertex).await?;
```

### Exchange (`qudag-exchange-core`)
```rust
use qudag_exchange::{Exchange, rUv};

// Resource trading
let mut exchange = Exchange::new()?;
let alice = exchange.create_account("alice")?;

// Transfer with automatic fee calculation
exchange.transfer(&alice, &bob, rUv::new(1000), None)?;
```

## 🛡️ Security Features

- **Post-Quantum Cryptography**: NIST-approved ML-KEM and ML-DSA algorithms
- **Memory Safety**: Written in Rust with zero unsafe code
- **Anonymous Communication**: Onion routing with traffic obfuscation
- **Byzantine Fault Tolerance**: QR-Avalanche consensus algorithm

## 📊 Performance

- **Crypto Operations**: 5,000+ signatures/sec, 1,000+ key exchanges/sec
- **Network Throughput**: 10,000+ messages/sec
- **DAG Consensus**: Sub-second finality
- **Exchange TPS**: 10,000+ transactions/sec

## 🔗 Ecosystem

- [qudag-cli](https://crates.io/crates/qudag-cli) - Command-line interface
- [qudag-mcp](https://crates.io/crates/qudag-mcp) - MCP server for AI integration
- [qudag-wasm](https://www.npmjs.com/package/qudag) - WebAssembly bindings
- [qudag-exchange-cli](https://crates.io/crates/qudag-exchange-standalone-cli) - Exchange CLI

## 📚 Documentation

- [API Documentation](https://docs.rs/qudag)
- [GitHub Repository](https://github.com/ruvnet/QuDAG)
- [Examples](https://github.com/ruvnet/QuDAG/tree/main/examples)
- [Architecture Guide](https://github.com/ruvnet/QuDAG/tree/main/docs/architecture)

## 🌟 Building the Future

QuDAG is the foundation for the next generation of autonomous businesses. Whether you're building AI agent swarms, decentralized marketplaces, or self-sustaining infrastructure, QuDAG provides the secure, scalable platform you need.

The age of zero-person businesses is here. Build yours with QuDAG.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.