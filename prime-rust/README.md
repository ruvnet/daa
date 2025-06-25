# Prime Rust - Distributed ML Framework

A distributed machine learning framework built with Rust, leveraging DAA for coordination and libp2p for networking.

## Workspace Structure

```
prime-rust/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── prime-core/           # Shared structures and protocol definitions
│   │   ├── Cargo.toml
│   │   ├── build.rs          # Proto compilation
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── tests/
│   │       └── core_test.rs
│   ├── prime-dht/            # Kademlia DHT implementation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── tests/
│   │       └── dht_test.rs
│   ├── prime-trainer/        # Distributed SGD/FSDP trainer
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── tests/
│   │       └── trainer_test.rs
│   ├── prime-coordinator/    # DAA-based governance layer
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── tests/
│   │       └── coordinator_test.rs
│   └── prime-cli/           # Command-line interface
│       ├── Cargo.toml
│       ├── src/
│       │   └── main.rs
│       └── tests/
│           └── cli_test.rs
└── README.md
```

## Crates Overview

### prime-core
- Shared data structures (ModelMetadata, NodeIdentity, TrainingConfig)
- Protocol buffer definitions
- Common types and utilities

### prime-dht
- Kademlia-based distributed hash table
- Peer discovery and routing
- Data replication and retrieval

### prime-trainer
- Distributed training orchestration
- SGD and FSDP implementations
- Gradient aggregation and compression
- Model sharding

### prime-coordinator
- DAA integration for governance
- Consensus mechanisms
- Task allocation
- Node management

### prime-cli
- Bootstrap and operations tooling
- Node management commands
- Training orchestration
- Configuration management

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p prime-core
cargo test -p prime-dht
cargo test -p prime-trainer
cargo test -p prime-coordinator
cargo test -p prime-cli
```

### Building

```bash
# Build all crates
cargo build --workspace

# Build release version
cargo build --release --workspace

# Build specific crate
cargo build -p prime-cli
```

### Running the CLI

```bash
# After building
./target/debug/prime --help

# Or install locally
cargo install --path crates/prime-cli
prime --help
```

## Dependencies

- **tokio**: Async runtime
- **libp2p**: P2P networking stack
- **tch**: PyTorch bindings for ML operations
- **daa**: Distributed Autonomous Agents
- **qudag**: QuDAG consensus
- **serde**: Serialization
- **clap**: CLI parsing
- **prost**: Protocol buffers

## Architecture

The Prime framework follows a modular architecture:

1. **Networking Layer** (prime-dht): Handles peer discovery and data distribution
2. **Training Layer** (prime-trainer): Manages distributed model training
3. **Coordination Layer** (prime-coordinator): Provides governance and consensus
4. **Core Layer** (prime-core): Shared types and protocols
5. **Interface Layer** (prime-cli): User interaction and operations

## TDD Approach

All crates follow Test-Driven Development:
- Tests are written first in `tests/` directories
- Implementation follows to make tests pass
- Integration tests validate cross-crate functionality