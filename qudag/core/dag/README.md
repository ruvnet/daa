# QuDAG DAG

DAG consensus implementation with QR-Avalanche algorithm for the QuDAG protocol.

## Features

- **QR-Avalanche Consensus**: Quantum-resistant Byzantine fault-tolerant consensus
- **Parallel Processing**: Concurrent message validation and ordering
- **Conflict Resolution**: Automatic detection and resolution of conflicts
- **Tip Selection**: Intelligent parent selection algorithms
- **Performance Monitoring**: Real-time consensus metrics

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
qudag-dag = "0.1"
```

## Examples

### Basic DAG Operations

```rust
use qudag_dag::{Dag, Vertex, VertexId};
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new DAG
    let dag = Dag::new(100); // Max 100 concurrent messages
    
    // Create and submit a message
    let message = qudag_dag::DagMessage {
        id: VertexId::new(),
        payload: b"Hello, DAG!".to_vec(),
        parents: HashSet::new(), // Genesis message
        timestamp: std::time::SystemTime::now(),
    };
    
    dag.submit_message(message).await?;
    Ok(())
}
```

### Consensus Integration

```rust
use qudag_dag::{QrDag, Vertex, VertexId};
use std::collections::HashSet;

// Create a DAG consensus instance
let mut dag = QrDag::new();

// Add vertices to the DAG
let vertex_id = VertexId::new();
let vertex = Vertex::new(vertex_id, b"vertex data".to_vec(), HashSet::new());
dag.add_vertex(vertex)?;

// Get consensus status
if let Some(status) = dag.get_confidence("vertex_id") {
    println!("Consensus status: {:?}", status);
}

// Get current tips
let tips = dag.get_tips();
println!("Current tips: {:?}", tips);
```

### Tip Selection

```rust
use qudag_dag::{TipSelection, TipSelectionConfig};

let config = TipSelectionConfig {
    max_parents: 2,
    min_weight_threshold: 0.1,
    selection_strategy: Default::default(),
};

let tip_selector = TipSelection::new(config);
// Use tip selector for parent selection...
```

## Architecture

### Core Components

- **DAG Structure**: Directed acyclic graph for message ordering
- **QR-Avalanche**: Quantum-resistant consensus algorithm
- **Vertex Processing**: Parallel validation and state management
- **Conflict Detection**: Automatic fork detection and resolution
- **Tip Selection**: Algorithms for optimal parent selection

### Consensus Flow

1. **Message Submission**: New messages are submitted to the DAG
2. **Parent Validation**: Verify parent messages exist and are valid
3. **Conflict Detection**: Check for conflicting transactions
4. **Consensus Voting**: Apply QR-Avalanche voting mechanism
5. **Finalization**: Achieve consensus finality

## Performance Features

- **Asynchronous Processing**: Non-blocking message handling
- **Parallel Validation**: Concurrent vertex processing
- **Optimized Data Structures**: Efficient graph traversal
- **Memory Management**: Automatic cleanup of old data

## Configuration

```rust
use qudag_dag::{ConsensusConfig, QrDag};
use std::time::Duration;

let config = ConsensusConfig {
    query_sample_size: 10,
    finality_threshold: 0.8,
    finality_timeout: Duration::from_secs(5),
    confirmation_depth: 3,
};

let dag = QrDag::with_config(config);
```

## Error Handling

The crate provides comprehensive error types:

- `DagError`: Main error type for DAG operations
- `VertexError`: Vertex-specific validation errors
- `ConsensusError`: Consensus algorithm errors

## Features

- `optimizations`: Enable performance optimizations (validation cache, traversal index)
- `validation-cache`: Enable vertex validation caching
- `traversal-index`: Enable graph traversal indexing

## Documentation

- [API Documentation](https://docs.rs/qudag-dag)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)

## License

Licensed under either MIT or Apache-2.0 at your option.