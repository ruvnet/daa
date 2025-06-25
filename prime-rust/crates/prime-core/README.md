# DAA Prime Core

[![Crates.io](https://img.shields.io/crates/v/daa-prime-core.svg)](https://crates.io/crates/daa-prime-core)
[![Documentation](https://docs.rs/daa-prime-core/badge.svg)](https://docs.rs/daa-prime-core)
[![License](https://img.shields.io/crates/l/daa-prime-core.svg)](https://github.com/yourusername/daa/blob/main/LICENSE)

Core shared structures and protocol definitions for the Prime distributed machine learning framework. This crate provides the foundational types, protocols, and message formats used across all Prime components.

## Overview

DAA Prime Core is the foundational layer of the Prime distributed ML system, providing:

- **Core Data Types**: Essential types for nodes, models, training configurations, and metrics
- **Protocol Definitions**: Network message formats and protocol handling
- **Serialization**: JSON-based serialization for all network communications
- **Error Handling**: Unified error types and result handling
- **Testing Support**: Property-based testing utilities and test fixtures

## Features

- 🔧 **Type Safety**: Strongly typed interfaces for all ML operations
- 📡 **Network Protocol**: Comprehensive message types for distributed coordination
- ⚡ **Performance**: Zero-copy serialization where possible
- 🧪 **Testing**: Built-in property-based testing support
- 📚 **Documentation**: Comprehensive API documentation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
daa-prime-core = "0.2.1"
```

## Quick Start

### Basic Types

```rust
use daa_prime_core::{NodeId, TrainingConfig, OptimizerType, AggregationStrategy};

// Create a node identifier
let node_id = NodeId::new("trainer-001");

// Configure training parameters
let config = TrainingConfig {
    batch_size: 32,
    learning_rate: 0.001,
    epochs: 100,
    optimizer: OptimizerType::Adam { beta1: 0.9, beta2: 0.999 },
    aggregation_strategy: AggregationStrategy::FederatedAveraging,
};
```

### Protocol Messages

```rust
use daa_prime_core::{ProtocolMessage, MessageType, NodeId};

// Create a protocol message
let sender = NodeId::new("coordinator-1");
let message = ProtocolMessage::new(sender, MessageType::Ping);

// Serialize for network transmission
let serialized = serde_json::to_string(&message)?;

// Send to specific recipient
let recipient = NodeId::new("trainer-1");
let targeted_message = ProtocolMessage::new(sender, MessageType::Ping)
    .with_recipient(recipient);
```

### Gradient Updates

```rust
use daa_prime_core::{GradientUpdate, TrainingMetrics, NodeId};
use std::collections::HashMap;

// Create a gradient update
let update = GradientUpdate {
    node_id: NodeId::new("trainer-001"),
    model_version: 42,
    round: 10,
    gradients: HashMap::from([
        ("layer1.weight".to_string(), vec![0.1, -0.2, 0.05]),
        ("layer1.bias".to_string(), vec![0.01]),
    ]),
    metrics: TrainingMetrics {
        loss: 0.234,
        accuracy: 0.892,
        samples_processed: 1000,
        computation_time_ms: 150,
    },
    timestamp: 1634567890,
};
```

## Core Types

### Node Management

- **`NodeId`**: Unique identifier for network participants
- **`ModelMetadata`**: Model version and architecture information
- **`TrainingConfig`**: Comprehensive training parameter configuration

### ML Operations

- **`OptimizerType`**: Support for SGD, Adam, and AdamW optimizers
- **`AggregationStrategy`**: Federated averaging, secure aggregation, and Byzantine-fault-tolerant methods
- **`GradientUpdate`**: Structured gradient sharing between nodes
- **`TrainingMetrics`**: Training performance and timing metrics

### Network Protocol

- **`MessageType`**: All supported message types for distributed coordination
- **`ProtocolMessage`**: Signed, versioned message wrapper
- **`ProtocolHandler`**: Trait for implementing custom message handlers

## Message Types

The protocol supports several categories of messages:

### Training Messages
- `GradientUpdate`: Share model gradients with coordinators
- `ModelSync`: Distribute updated model parameters
- `TrainingRequest`: Request participation in training rounds

### Consensus Messages
- `ConsensusProposal`: Propose values for consensus
- `ConsensusVote`: Vote on consensus proposals
- `ConsensusCommit`: Commit agreed-upon values

### DHT Messages
- `DhtPut`: Store key-value pairs in distributed hash table
- `DhtGet`: Retrieve values from DHT
- `DhtResponse`: Response with requested DHT values

### Control Messages
- `Ping`/`Pong`: Network connectivity testing
- `JoinRequest`/`JoinResponse`: Network participation coordination

## Testing

The crate includes comprehensive testing utilities:

```rust
use daa_prime_core::*;
use proptest::prelude::*;

// Property-based testing for gradient updates
proptest! {
    #[test]
    fn test_gradient_serialization(
        node_id in "[a-zA-Z0-9]{5,20}",
        version in 0u64..1000u64,
    ) {
        let update = GradientUpdate {
            node_id: NodeId::new(node_id),
            model_version: version,
            // ... rest of fields
        };
        
        let serialized = serde_json::to_string(&update)?;
        let deserialized: GradientUpdate = serde_json::from_str(&serialized)?;
        assert_eq!(update.node_id, deserialized.node_id);
    }
}
```

## Error Handling

All operations return `Result<T, Error>` for comprehensive error handling:

```rust
use daa_prime_core::{Result, Error};

fn process_message(data: &[u8]) -> Result<ProtocolMessage> {
    let message: ProtocolMessage = serde_json::from_slice(data)
        .map_err(|e| Error::Serialization(e.to_string()))?;
    
    // Validate message format
    if message.version != PROTOCOL_VERSION {
        return Err(Error::UnsupportedVersion(message.version));
    }
    
    Ok(message)
}
```

## Integration with DAA Ecosystem

Prime Core integrates seamlessly with the broader DAA ecosystem:

- **DAA AI**: ML model management and inference
- **DAA Rules**: Governance and compliance checking
- **DAA Economy**: Token-based incentive mechanisms

## Advanced Usage

### Custom Message Handlers

```rust
use daa_prime_core::{ProtocolHandler, ProtocolMessage, Result};
use async_trait::async_trait;

struct CustomHandler;

#[async_trait]
impl ProtocolHandler for CustomHandler {
    async fn handle_message(&self, message: ProtocolMessage) -> Result<Option<ProtocolMessage>> {
        match message.message_type {
            MessageType::Ping => {
                // Respond with Pong
                Ok(Some(ProtocolMessage::new(
                    message.sender.clone(),
                    MessageType::Pong
                )))
            },
            _ => Ok(None)
        }
    }
    
    async fn validate_message(&self, message: &ProtocolMessage) -> Result<()> {
        // Custom validation logic
        if message.signature.is_none() {
            return Err(Error::InvalidMessage("Missing signature".to_string()));
        }
        Ok(())
    }
}
```

### Model Metadata Management

```rust
use daa_prime_core::ModelMetadata;

let metadata = ModelMetadata {
    id: "resnet50-v1".to_string(),
    version: 1,
    architecture: "ResNet".to_string(),
    parameters_count: 25_557_032,
    created_at: 1634567890,
    updated_at: 1634567890,
};

// Track model evolution
let updated_metadata = ModelMetadata {
    version: metadata.version + 1,
    updated_at: current_timestamp(),
    ..metadata
};
```

## Performance Considerations

- **Zero-Copy Serialization**: Use `serde_json::from_slice` for network data
- **Message Batching**: Combine multiple operations into single messages
- **Compression**: Consider compressing large gradient updates
- **Connection Pooling**: Reuse network connections where possible

## Security

- **Message Signing**: All protocol messages support cryptographic signatures
- **Version Validation**: Protocol version checking prevents compatibility issues
- **Input Validation**: Comprehensive validation for all message types

## Roadmap

- [ ] Protocol buffer support for binary serialization
- [ ] Built-in message compression
- [ ] Advanced aggregation strategies (Byzantine fault tolerance)
- [ ] Differential privacy support
- [ ] Homomorphic encryption integration

## Contributing

Contributions are welcome! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Related Crates

- [`daa-prime-dht`](../prime-dht): Distributed hash table implementation
- [`daa-prime-trainer`](../prime-trainer): Training node implementation  
- [`daa-prime-coordinator`](../prime-coordinator): Coordination and governance
- [`daa-prime-cli`](../prime-cli): Command-line interface

## Support

- 📖 [Documentation](https://docs.rs/daa-prime-core)
- 🐛 [Issue Tracker](https://github.com/yourusername/daa/issues)
- 💬 [Discussions](https://github.com/yourusername/daa/discussions)