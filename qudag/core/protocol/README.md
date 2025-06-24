# QuDAG Protocol

Protocol implementation and coordination layer for the QuDAG system.

## Features

- **Component Orchestration**: Coordinates crypto, DAG, and network layers
- **Node Management**: Complete node lifecycle management
- **Message Handling**: Protocol message processing and routing
- **State Management**: Persistent state storage and synchronization
- **RPC Server**: JSON-RPC API for external communication
- **Configuration Management**: Centralized configuration system

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
qudag-protocol = "0.1"
```

## Examples

### Basic Node Setup

```rust
use qudag_protocol::{Node, NodeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NodeConfig {
        node_id: "my-node".to_string(),
        listen_port: 8080,
        rpc_port: 9090,
        data_dir: "./data".into(),
        ..Default::default()
    };
    
    let node = Node::new(config).await?;
    node.start().await?;
    
    Ok(())
}
```

### Message Processing

```rust
use qudag_protocol::{Message, MessageType, ProtocolVersion};

// Create a protocol message
let message = Message {
    version: ProtocolVersion::V1,
    message_type: MessageType::Transaction,
    payload: b"transaction data".to_vec(),
    timestamp: std::time::SystemTime::now(),
};

// Process the message
// protocol.process_message(message).await?;
```

### RPC Server

```rust
use qudag_protocol::{RpcServer, RpcTransport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = RpcServer::new("127.0.0.1:9090").await?;
    
    // Start RPC server
    server.start().await?;
    
    // Server will handle JSON-RPC requests
    Ok(())
}
```

### State Persistence

```rust
use qudag_protocol::{StateStore, MemoryStateStore, FileStateStore};

// In-memory storage for testing
let memory_store = MemoryStateStore::new();

// File-based storage for production
let file_store = FileStateStore::new("./state")?;

// Store and retrieve state
memory_store.save_state("key", &data).await?;
let retrieved = memory_store.load_state("key").await?;
```

## Architecture

### Protocol Stack

```
┌─────────────────────┐
│   RPC Server        │  ← External API
├─────────────────────┤
│   Protocol Layer    │  ← Message coordination
├─────────────────────┤
│   DAG Consensus     │  ← Consensus algorithm
├─────────────────────┤
│   P2P Network       │  ← Peer communication
├─────────────────────┤
│   Cryptography      │  ← Quantum-resistant crypto
└─────────────────────┘
```

### Core Components

- **Node**: Main coordinator for all subsystems
- **Message Factory**: Creates and validates protocol messages
- **State Machine**: Manages node state transitions
- **RPC Server**: Provides JSON-RPC API
- **Persistence**: State storage and recovery
- **Configuration**: System configuration management

## Node Configuration

```rust
use qudag_protocol::{NodeConfig, ProtocolConfig};

let config = NodeConfig {
    node_id: "node-1".to_string(),
    listen_port: 8080,
    rpc_port: 9090,
    data_dir: "./data".into(),
    max_peers: 50,
    protocol_config: ProtocolConfig {
        version: ProtocolVersion::V1,
        handshake_timeout: Duration::from_secs(30),
        message_timeout: Duration::from_secs(60),
        ..Default::default()
    },
    ..Default::default()
};
```

## Message Types

The protocol supports various message types:

```rust
use qudag_protocol::{MessageType, Message};

// Transaction messages
let tx_message = Message::new(MessageType::Transaction, payload);

// Consensus messages
let consensus_message = Message::new(MessageType::Consensus, payload);

// Network discovery
let discovery_message = Message::new(MessageType::Discovery, payload);

// Administrative commands
let admin_message = Message::new(MessageType::Admin, payload);
```

## RPC API

The protocol provides a JSON-RPC API:

### Available Methods

- `node.status` - Get node status and health
- `node.peers` - List connected peers
- `network.send_message` - Send message to peer
- `dag.get_tips` - Get current DAG tips
- `dag.add_vertex` - Add vertex to DAG

### Example RPC Call

```bash
curl -X POST http://localhost:9090 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "node.status",
    "id": 1
  }'
```

## State Management

```rust
use qudag_protocol::{ProtocolState, ProtocolStateMachine};

let state_machine = ProtocolStateMachine::new();

// Transition states
state_machine.transition_to(ProtocolState::Connecting).await?;
state_machine.transition_to(ProtocolState::Connected).await?;

// Get current state
let current = state_machine.current_state().await;
```

## Error Handling

Comprehensive error types for protocol operations:

```rust
use qudag_protocol::{ProtocolError, MessageError, StateError};

match result {
    Err(ProtocolError::MessageError(e)) => {
        eprintln!("Message error: {}", e);
    }
    Err(ProtocolError::StateError(e)) => {
        eprintln!("State error: {}", e);
    }
    Ok(value) => {
        // Handle success
    }
}
```

## Features

- `rocksdb`: Enable RocksDB backend for state storage
- `compression`: Enable message compression
- `metrics`: Enable performance metrics collection

## Memory Management

The protocol includes memory tracking and optimization:

```rust
use qudag_protocol::{get_memory_usage, MemoryTracker};

// Get current memory usage
let usage = get_memory_usage();
println!("Memory usage: {} bytes", usage);

// Track memory for specific operations
let tracker = MemoryTracker::new();
// ... perform operations ...
let delta = tracker.delta();
```

## Documentation

- [API Documentation](https://docs.rs/qudag-protocol)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)

## License

Licensed under either MIT or Apache-2.0 at your option.