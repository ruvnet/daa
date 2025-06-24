# DAG Module API

The `qudag_dag` module implements a directed acyclic graph (DAG) consensus mechanism using QR-Avalanche for quantum-resistant distributed consensus with Byzantine fault tolerance.

## Core Types

### Dag

Main DAG structure with asynchronous message processing and conflict resolution.

```rust
pub struct Dag {
    // private fields
}

impl Dag {
    pub async fn new() -> Self;
    pub async fn add_message(&self, message: DagMessage) -> Result<(), DagError>;
    pub async fn get_tips(&self) -> Vec<VertexId>;
    pub async fn get_finalized_vertices(&self) -> Vec<VertexId>;
    pub async fn process_pending(&self) -> Result<(), DagError>;
    pub async fn resolve_conflicts(&self) -> Result<(), DagError>;
    pub fn get_metrics(&self) -> DagMetrics;
}
```

### DagMessage

Message structure for DAG processing with quantum-resistant signatures.

```rust
pub struct DagMessage {
    pub id: VertexId,
    pub payload: Vec<u8>,
    pub parents: HashSet<VertexId>,
    pub timestamp: u64,
}

impl DagMessage {
    pub fn new(payload: Vec<u8>, parents: HashSet<VertexId>) -> Self;
    pub fn sign(&mut self, keypair: &MlDsaKeyPair) -> Result<(), DagError>;
    pub fn verify(&self, public_key: &MlDsaPublicKey) -> Result<bool, DagError>;
}
```

### Consensus

QR-Avalanche consensus implementation with Byzantine fault tolerance.

```rust
pub struct Consensus {
    // private fields
}

impl Consensus {
    pub fn new(config: ConsensusConfig) -> Self;
    pub async fn query_confidence(&self, vertex_id: VertexId) -> Result<ConsensusStatus, ConsensusError>;
    pub async fn determine_finality(&self, vertex_id: VertexId) -> Result<bool, ConsensusError>;
    pub async fn handle_query(&self, query: ConsensusQuery) -> Result<ConsensusResponse, ConsensusError>;
    pub fn get_status(&self, vertex_id: VertexId) -> Option<ConsensusStatus>;
}
```

### ConsensusStatus

Status of vertex in consensus process.

```rust
pub enum ConsensusStatus {
    Pending,
    Preferred,
    Accepted,
    Finalized,
    Rejected,
}
```

### DAGConsensus

The main consensus engine handling vertex ordering and finality (legacy interface).

```rust
pub struct DAGConsensus {
    // private fields
}

impl DAGConsensus {
    pub fn new() -> Self;
    pub fn with_config(config: ConsensusConfig) -> Self;
    pub fn add_vertex(&mut self, vertex: Vertex) -> Result<(), ConsensusError>;
    pub fn get_tips(&self) -> HashSet<String>;
    pub fn get_confidence(&self, id: &str) -> Option<Confidence>;
    pub fn get_total_order(&self) -> Result<Vec<String>, ConsensusError>;
}
```

### Vertex

Represents a message or transaction in the DAG.

```rust
pub struct Vertex {
    pub id: VertexId,
    pub parents: Vec<VertexId>,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
    pub payload: Vec<u8>,
    pub quantum_signature: Option<MlDsaSignature>,
}

impl Vertex {
    pub fn new(payload: Vec<u8>, parents: Vec<VertexId>) -> Self;
    pub fn sign_quantum(&mut self, keypair: &MlDsaKeyPair) -> Result<(), VertexError>;
    pub fn verify_quantum(&self, public_key: &MlDsaPublicKey) -> Result<bool, VertexError>;
    pub fn is_genesis(&self) -> bool;
    pub fn get_depth(&self) -> usize;
}
```

### VertexId

Unique identifier for vertices in the DAG.

```rust
pub struct VertexId(pub [u8; 32]);

impl VertexId {
    pub fn new() -> Self;
    pub fn from_hash(data: &[u8]) -> Self;
    pub fn as_bytes(&self) -> &[u8];
}
```

### TipSelection

Algorithm for selecting tips when creating new vertices.

```rust
pub struct TipSelection {
    // private fields
}

impl TipSelection {
    pub fn new() -> Self;
    pub async fn select_tips(&self, dag: &Dag, max_tips: usize) -> Result<Vec<VertexId>, TipSelectionError>;
    pub fn validate_tip_selection(&self, tips: &[VertexId]) -> Result<bool, TipSelectionError>;
}
```

### DagMetrics

Performance and status metrics for the DAG.

```rust
pub struct DagMetrics {
    pub vertex_count: usize,
    pub tip_count: usize,
    pub finalized_count: usize,
    pub pending_count: usize,
    pub average_confirmation_time: Duration,
    pub throughput_tps: f64,
}
```

### Confidence

Finality status for vertices (legacy).

```rust
pub enum Confidence {
    Pending,
    HighConfidence,
    Final,
}
```

### ConsensusConfig

Configuration parameters for the consensus algorithm.

```rust
pub struct ConsensusConfig {
    pub query_sample_size: usize,
    pub finality_threshold: f64,
    pub finality_timeout: Duration,
    pub confirmation_depth: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            query_sample_size: 20,
            finality_threshold: 0.80,
            finality_timeout: Duration::from_secs(1),
            confirmation_depth: 4,
        }
    }
}
```

## Error Types

### DagError

Main error type for DAG operations.

```rust
pub enum DagError {
    VertexError(VertexError),
    ConsensusError(ConsensusError),
    ChannelClosed,
    ConflictDetected,
    StateSyncFailed,
}
```

### ConsensusError

Errors in the consensus mechanism.

```rust
pub enum ConsensusError {
    InvalidVertex(String),
    ForkDetected(String),
    ValidationError(String),
    ConsensusTimeout,
    FinalityError(String),
    QueryFailed(String),
    NetworkError(String),
}
```

### VertexError

Errors related to vertex operations.

```rust
pub enum VertexError {
    InvalidSignature,
    InvalidParents,
    InvalidTimestamp,
    SigningFailed,
    VerificationFailed,
    MissingQuantumSignature,
}
```

### TipSelectionError

Errors in tip selection algorithm.

```rust
pub enum TipSelectionError {
    NoTipsAvailable,
    MaxTipsExceeded,
    InvalidTipSet,
    AlgorithmFailed,
}
```

## Example Usage

### Modern DAG Operations

```rust
use qudag_dag::{Dag, DagMessage, VertexId, DagError};
use qudag_crypto::MlDsaKeyPair;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<(), DagError> {
    // Create a new DAG instance
    let dag = Dag::new().await;
    
    // Create ML-DSA keypair for signing
    let keypair = MlDsaKeyPair::generate()?;
    
    // Create and add a message to the DAG
    let mut message = DagMessage::new(
        b"Hello, DAG!".to_vec(),
        HashSet::new(), // No parents for genesis message
    );
    
    // Sign the message with quantum-resistant signature
    message.sign(&keypair)?;
    
    // Add message to DAG
    dag.add_message(message).await?;
    
    // Get current tips
    let tips = dag.get_tips().await;
    println!("Current tips: {:?}", tips);
    
    // Create a child message
    let mut child_message = DagMessage::new(
        b"Child message".to_vec(),
        tips.into_iter().collect(), // Reference current tips as parents
    );
    child_message.sign(&keypair)?;
    dag.add_message(child_message).await?;
    
    // Process pending messages and resolve conflicts
    dag.process_pending().await?;
    dag.resolve_conflicts().await?;
    
    // Get finalized vertices
    let finalized = dag.get_finalized_vertices().await;
    println!("Finalized vertices: {:?}", finalized);
    
    // Get performance metrics
    let metrics = dag.get_metrics();
    println!("DAG has {} vertices, {} finalized", 
             metrics.vertex_count, metrics.finalized_count);
    
    Ok(())
}
```

### Consensus Operations

```rust
use qudag_dag::{Consensus, ConsensusConfig, ConsensusStatus, VertexId};

async fn consensus_example() -> Result<(), ConsensusError> {
    // Configure consensus parameters
    let config = ConsensusConfig {
        query_sample_size: 20,
        finality_threshold: 0.80,
        finality_timeout: Duration::from_secs(2),
        confirmation_depth: 4,
    };
    
    let consensus = Consensus::new(config);
    let vertex_id = VertexId::new();
    
    // Query confidence from network peers
    let status = consensus.query_confidence(vertex_id).await?;
    
    match status {
        ConsensusStatus::Finalized => {
            println!("Vertex achieved finality");
        }
        ConsensusStatus::Accepted => {
            println!("Vertex accepted but not yet final");
        }
        ConsensusStatus::Preferred => {
            println!("Vertex preferred by network");
        }
        ConsensusStatus::Pending => {
            println!("Vertex still pending consensus");
        }
        ConsensusStatus::Rejected => {
            println!("Vertex rejected by network");
        }
    }
    
    // Check if vertex has achieved finality
    let is_final = consensus.determine_finality(vertex_id).await?;
    if is_final {
        println!("Vertex is now finalized");
    }
    
    Ok(())
}
```

### Tip Selection

```rust
use qudag_dag::{TipSelection, Dag, VertexId};

async fn tip_selection_example(dag: &Dag) -> Result<Vec<VertexId>, TipSelectionError> {
    let tip_selector = TipSelection::new();
    
    // Select optimal tips for new vertex
    let max_tips = 2; // Select up to 2 tips
    let selected_tips = tip_selector.select_tips(dag, max_tips).await?;
    
    // Validate the tip selection
    if tip_selector.validate_tip_selection(&selected_tips)? {
        println!("Selected {} valid tips", selected_tips.len());
        Ok(selected_tips)
    } else {
        Err(TipSelectionError::InvalidTipSet)
    }
}
```

### Basic DAG Operations (Legacy)

```rust
use qudag_dag::{DAGConsensus, Vertex, ConsensusError};

// Create a new DAG consensus instance
let mut dag = DAGConsensus::new();

// Create and add a vertex
let vertex = Vertex {
    id: VertexId::new(),
    parents: vec![],
    timestamp: 0,
    signature: None,
    payload: b"Hello".to_vec(),
    quantum_signature: None,
};

// Add vertex to DAG
dag.add_vertex(vertex)?;

// Get current tips (vertices with no children)
let tips = dag.get_tips();

// Check vertex finality
if let Some(confidence) = dag.get_confidence("vertex1") {
    match confidence {
        Confidence::Final => println!("Vertex is final"),
        Confidence::HighConfidence => println!("Vertex has high confidence"),
        Confidence::Pending => println!("Vertex is still pending"),
    }
}
```

### Custom Configuration

```rust
use qudag_dag::{DAGConsensus, ConsensusConfig};
use std::time::Duration;

let config = ConsensusConfig {
    query_sample_size: 30,
    finality_threshold: 0.85,
    finality_timeout: Duration::from_secs(2),
    confirmation_depth: 5,
};

let dag = DAGConsensus::with_config(config);
```

### Error Handling

```rust
use qudag_dag::{DAGConsensus, ConsensusError};

fn handle_vertex_addition(dag: &mut DAGConsensus, vertex: Vertex) {
    match dag.add_vertex(vertex) {
        Ok(()) => println!("Vertex added successfully"),
        Err(ConsensusError::InvalidVertex(msg)) => {
            eprintln!("Invalid vertex: {}", msg);
        }
        Err(ConsensusError::ForkDetected(msg)) => {
            eprintln!("Fork detected: {}", msg);
            // Implement fork resolution strategy
        }
        Err(e) => eprintln!("Error adding vertex: {}", e),
    }
}
```

## Best Practices

1. **Vertex Creation**
   - Always ensure parent vertices exist before adding new vertices
   - Validate vertex signatures before addition
   - Use monotonically increasing timestamps

2. **Performance Optimization**
   - Monitor the DAG size and prune old vertices when possible
   - Adjust consensus parameters based on network conditions
   - Cache commonly accessed vertices

3. **Fork Handling**
   - Implement proper fork detection and resolution
   - Consider using a fork choice rule
   - Maintain consistent total ordering

## Configuration Guidelines

1. **Query Sample Size**
   - Larger values increase security but reduce performance
   - Recommended range: 20-50 peers
   - Adjust based on network size

2. **Finality Threshold**
   - Higher values increase security but may delay finality
   - Recommended range: 0.75-0.85
   - Consider network latency when adjusting

3. **Confirmation Depth**
   - Affects confidence in finality
   - Recommended range: 4-6 confirmations
   - Balance between security and latency