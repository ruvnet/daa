# DAA Chain

**🚀 FULL IMPLEMENTATION - This is the complete, production-ready implementation of the DAA Chain module, not a placeholder.**

Blockchain integration layer for the Decentralized Autonomous Agents (DAA) system, built on top of QuDAG network infrastructure.

## Overview

DAA Chain provides the blockchain foundation for the DAA ecosystem, enabling secure, scalable, and decentralized operations for autonomous agents. It integrates with QuDAG's advanced networking and consensus mechanisms while adding DAA-specific transaction types and validation logic.

## Features

### Core Blockchain Functionality
- **Block Management**: Complete block creation, validation, and storage
- **Transaction Processing**: DAA-specific transaction types with cryptographic validation
- **Consensus Integration**: Pluggable consensus using QuDAG consensus primitives
- **Network Layer**: P2P networking with agent discovery and capability advertisement

### DAA-Specific Features
- **Agent Registration**: On-chain agent identity and capability management
- **Resource Allocation**: Decentralized resource distribution and tracking
- **Task Assignment**: Blockchain-based task coordination and execution
- **Reward Distribution**: Automated reward calculations and distribution
- **Rules Integration**: Optional integration with DAA Rules engine

### QuDAG Integration
- Built on QuDAG network infrastructure
- Uses QuDAG consensus mechanisms
- Leverages QuDAG protocol messaging
- Integrates with QuDAG storage primitives

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   DaaChain      │    │  NetworkManager │    │ ConsensusEngine │
│                 │    │                 │    │                 │
│ - Configuration │◄──►│ - P2P Network   │◄──►│ - Validator Set │
│ - Block/TX Mgmt │    │ - Agent Discovery│    │ - Vote Processing│
│ - Storage       │    │ - Capabilities  │    │ - Leader Election│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│    Storage      │    │ QuDAG Network   │    │ QuDAG Consensus │
│                 │    │                 │    │                 │
│ - File Storage  │    │ - P2P Protocol  │    │ - BFT Algorithm │
│ - Block Cache   │    │ - Message Route │    │ - State Machine │
│ - TX Pool       │    │ - Peer Discovery│    │ - Safety Rules  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Transaction Types

### Agent Registration
```rust
TransactionType::AgentRegistration {
    agent_id: String,
    public_key: Vec<u8>,
    capabilities: Vec<String>,
}
```

### Resource Allocation
```rust
TransactionType::ResourceAllocation {
    agent_id: String,
    resource_type: String,
    amount: u64,
}
```

### Task Assignment
```rust
TransactionType::TaskAssignment {
    task_id: String,
    agent_id: String,
    parameters: HashMap<String, String>,
}
```

### Reward Distribution
```rust
TransactionType::RewardDistribution {
    agent_id: String,
    amount: u64,
    reason: String,
}
```

### Data Storage
```rust
TransactionType::Data {
    data: Vec<u8>,
    metadata: HashMap<String, String>,
}
```

## Usage

### Basic Chain Setup

```rust
use daa_chain::{DaaChain, ChainConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create chain configuration
    let config = ChainConfig {
        chain_id: "daa-main".to_string(),
        max_block_size: 1024 * 1024, // 1MB
        block_time: 15, // 15 seconds
        enable_consensus: true,
        storage_path: "./data/chain".to_string(),
        ..Default::default()
    };

    // Initialize chain
    let mut chain = DaaChain::new(config).await?;
    
    // Start processing
    chain.start().await?;
    
    Ok(())
}
```

### Creating Transactions

```rust
use daa_chain::transaction::{TransactionBuilder, TransactionType};
use ed25519_dalek::SigningKey;

// Create signing key
let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);

// Build transaction
let tx = TransactionBuilder::new()
    .with_type(TransactionType::AgentRegistration {
        agent_id: "agent-123".to_string(),
        public_key: signing_key.verifying_key().to_bytes().to_vec(),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
    })
    .with_nonce(1)
    .with_gas(21000, 1)
    .build_and_sign(&signing_key)?;

// Submit to chain
let tx_hash = chain.submit_transaction(tx).await?;
println!("Transaction submitted: {}", tx_hash);
```

### Network Operations

```rust
use daa_chain::network::NetworkManager;

// Create network manager
let mut network = NetworkManager::new(config.network).await?;
network.start().await?;

// Broadcast agent registration
network.broadcast_agent_registration(
    "agent-123".to_string(),
    vec!["compute".to_string(), "storage".to_string()],
).await?;

// Find agents with specific capabilities
let compute_agents = network.get_peers_with_capability("compute").await;
println!("Found {} compute agents", compute_agents.len());
```

## Configuration

### Chain Configuration

```rust
ChainConfig {
    chain_id: "daa-main".to_string(),
    network: NetworkConfig::default(),
    max_block_size: 1024 * 1024,
    max_transactions_per_block: 1000,
    block_time: 15,
    enable_consensus: true,
    storage_path: "./data/chain".to_string(),
}
```

### Network Configuration

```rust
NetworkConfig {
    listen_port: 8000,
    bootstrap_peers: vec![
        "127.0.0.1:8001".to_string(),
        "127.0.0.1:8002".to_string(),
    ],
    max_peers: 50,
    enable_discovery: true,
}
```

## Features

The crate supports several feature flags:

- `default`: Includes network and consensus features
- `network`: Enables QuDAG network integration
- `consensus`: Enables QuDAG consensus integration
- `rules-integration`: Enables integration with DAA Rules engine
- `full`: Includes all features

```toml
[dependencies]
daa-chain = { version = "0.1.0", features = ["full"] }
```

## Integration with Other DAA Components

### DAA Rules Integration
When the `rules-integration` feature is enabled, DAA Chain can validate transactions and blocks using the DAA Rules engine:

```rust
use daa_chain::rules_bridge::{RulesBridge, ChainRule};

// Create rules bridge
let mut bridge = RulesBridge::new(rule_engine);

// Add validation rule
let rule = ChainRule {
    id: "max-allocation".to_string(),
    name: "Maximum Allocation Limit".to_string(),
    trigger: RuleTrigger::BeforeTransactionValidation,
    conditions: vec![
        RuleCondition::TransactionType("ResourceAllocation".to_string()),
        RuleCondition::AmountGreaterThan(1000),
    ],
    actions: vec![RuleAction::Deny("Allocation exceeds limit".to_string())],
    enabled: true,
};

bridge.add_rule(rule);
```

## Storage

DAA Chain uses a pluggable storage system with file-based storage as the default implementation:

- **Blocks**: Stored as JSON files with hash-based naming
- **Transactions**: Individual transaction files with caching
- **Metadata**: Chain state and configuration data
- **Height Mapping**: Block height to hash mapping for quick access

## Consensus

The consensus system integrates with QuDAG's BFT consensus while adding DAA-specific features:

- **Validator Management**: Dynamic validator set with stake-based selection
- **Vote Types**: Proposal, Prevote, Precommit, and Commit votes
- **Leader Selection**: Round-robin leader selection based on stake
- **Finality**: 2/3+ majority required for block commitment

## Testing

Run the test suite:

```bash
cargo test --package daa-chain
```

Run with all features:

```bash
cargo test --package daa-chain --all-features
```

## Dependencies

### QuDAG Dependencies
- `qudag-core`: Core blockchain primitives
- `qudag-network`: P2P networking layer
- `qudag-protocol`: Protocol message handling
- `qudag-consensus`: BFT consensus implementation

### Cryptographic Dependencies
- `blake3`: Fast hashing algorithm
- `ed25519-dalek`: Digital signatures
- `hex`: Hexadecimal encoding

### Async Runtime
- `tokio`: Async runtime with full features
- `async-trait`: Async traits support

## License

MIT OR Apache-2.0