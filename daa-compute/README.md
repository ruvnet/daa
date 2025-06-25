# DAA Compute - P2P Communication Layer

A high-performance peer-to-peer communication layer for decentralized AI training, built with Rust and libp2p. This implementation enables distributed gradient sharing across cloud servers, edge devices, and browser nodes.

## Features

### 🌐 Networking
- **libp2p Swarm**: Full implementation with Kademlia DHT and Gossipsub protocols
- **Multi-Transport Support**: TCP, WebSocket, and WebRTC (for browsers)
- **NAT Traversal**: STUN/TURN support with automatic hole punching
- **Discovery Methods**: Bootstrap nodes, mDNS, DHT, and capability-based filtering

### 🔄 Gradient Sharing
- **All-Reduce Algorithms**: Ring, Tree, Butterfly, and Hierarchical implementations
- **Compression**: Multiple methods including Zstandard, LZ4, Snappy, and int8 quantization
- **Fault Tolerance**: Automatic handling of node failures during aggregation
- **Optimized Routing**: Topology-aware message routing for efficient gradient distribution

### 🌍 Browser Support
- **WASM Compatible**: Full WebAssembly support for in-browser training
- **WebRTC Transport**: Direct browser-to-browser communication
- **WebSocket Fallback**: For environments where WebRTC is unavailable

### 🔒 Security
- **Encrypted Connections**: Noise protocol for all peer connections
- **Privacy Options**: Onion routing support for anonymous gradient sharing
- **Post-Quantum Ready**: Prepared for integration with QuDAG's quantum-resistant crypto

## Architecture

```
daa-compute/
├── src/
│   ├── lib.rs              # Main library interface
│   └── p2p/
│       ├── mod.rs          # P2P network manager
│       ├── behavior.rs    # libp2p protocol composition
│       ├── transport.rs    # Multi-transport configuration
│       ├── gradient.rs     # Gradient aggregation algorithms
│       ├── compression.rs  # Bandwidth optimization
│       ├── routing.rs      # Message routing protocols
│       ├── discovery.rs    # Peer discovery mechanisms
│       └── nat.rs          # NAT traversal implementation
└── examples/
    └── p2p_gradient_sharing.rs  # Usage example
```

## Usage

### Basic Setup

```rust
use daa_compute::{P2PNetwork, SwarmConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure the P2P network
    let config = SwarmConfig {
        compression_level: 3,
        enable_nat_traversal: true,
        ..Default::default()
    };
    
    // Create and start the network
    let mut network = P2PNetwork::new(config).await?;
    network.bootstrap().await?;
    
    // Share gradients
    let gradient = compute_gradient();
    network.broadcast_gradient(gradient).await?;
    
    // Run the network
    network.run().await
}
```

### Cloud Node Configuration

```rust
let config = SwarmConfig {
    listen_addresses: vec![
        "/ip4/0.0.0.0/tcp/9000".parse()?,
        "/ip4/0.0.0.0/tcp/9001/ws".parse()?,
    ],
    compression_level: 5,
    ..Default::default()
};
```

### Browser Node (WASM)

```rust
#[cfg(target_arch = "wasm32")]
let config = SwarmConfig {
    listen_addresses: vec![
        "/ip4/0.0.0.0/tcp/0/ws".parse()?,
    ],
    enable_nat_traversal: false, // Handled by browser
    ..Default::default()
};
```

### Edge Node with NAT

```rust
let config = SwarmConfig {
    compression_level: 9, // Maximum compression
    enable_nat_traversal: true,
    enable_relay: true, // Use relay if direct connection fails
    ..Default::default()
};
```

## All-Reduce Algorithms

The implementation provides four gradient aggregation algorithms:

1. **Ring All-Reduce**: Bandwidth-efficient, nodes form a ring
2. **Tree All-Reduce**: Latency-efficient, hierarchical aggregation
3. **Butterfly All-Reduce**: Balanced bandwidth and latency
4. **Hierarchical All-Reduce**: Optimized for geo-distributed nodes

## Compression Methods

Multiple compression strategies for different scenarios:

- **Quantization**: Float32 → Int8 (4x compression)
- **Sparse Format**: For gradients with many zeros
- **Delta Compression**: For sequential updates
- **Zstandard/LZ4/Snappy**: General-purpose compression

## Building

### Native Build

```bash
cd daa-compute
cargo build --release
```

### WASM Build

```bash
cargo build --target wasm32-unknown-unknown --features browser
```

### Run Examples

```bash
cargo run --example p2p_gradient_sharing
```

## Integration with DAA

This P2P layer integrates seamlessly with the DAA orchestrator:

```rust
use daa_orchestrator::DaaOrchestrator;
use daa_compute::P2PNetwork;

// Use P2P network as transport for DAA
let p2p = P2PNetwork::new(config).await?;
let orchestrator = DaaOrchestrator::with_transport(p2p);
```

## Performance

- **Bandwidth Optimization**: 4x compression with int8 quantization
- **Latency**: Sub-second gradient aggregation for <100 nodes
- **Scalability**: Tested with 1000+ peers in simulation
- **GPU Memory**: Zero-copy gradient sharing when possible

## Security Considerations

- All connections are encrypted with Noise protocol
- Peer identities are cryptographically verified
- Optional onion routing for privacy-sensitive deployments
- Ready for post-quantum cryptography via QuDAG integration

## Contributing

See the main DAA repository for contribution guidelines.

## License

This project is part of the DAA (Decentralized Autonomous Applications) SDK.