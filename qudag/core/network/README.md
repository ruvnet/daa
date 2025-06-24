# QuDAG Network

P2P networking layer with anonymous routing and dark addressing for the QuDAG protocol.

## Features

- **Anonymous Routing**: Multi-hop onion routing with ML-KEM encryption
- **Dark Addressing**: Decentralized `.dark` domain system
- **Traffic Obfuscation**: ChaCha20Poly1305 traffic disguising
- **NAT Traversal**: STUN/TURN/UPnP support for firewall penetration
- **Peer Discovery**: Kademlia DHT-based peer discovery
- **LibP2P Integration**: Production-ready P2P networking stack

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
qudag-network = "0.1"
```

## Examples

### Basic P2P Node

```rust
use qudag_network::{P2PNode, NetworkConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let (mut node, handle) = P2PNode::new(config).await?;
    
    // Start the node
    tokio::spawn(async move {
        node.run().await
    });
    
    // Use the handle to send commands
    // handle.connect_peer(peer_address).await?;
    
    Ok(())
}
```

### Dark Domain Resolution

```rust
use qudag_network::{DarkResolver, DarkDomainRecord};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = DarkResolver::new();
    
    // Register a dark domain
    let record = DarkDomainRecord {
        domain: "myservice.dark".to_string(),
        address: "12D3KooW...".to_string(), // Peer ID
        ttl: 3600,
        quantum_fingerprint: vec![],
    };
    
    resolver.register_domain(record).await?;
    
    // Resolve a domain
    if let Some(resolved) = resolver.resolve("myservice.dark").await? {
        println!("Resolved to: {}", resolved.address);
    }
    
    Ok(())
}
```

### Onion Routing

```rust
use qudag_network::{OnionRouter, Circuit};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = OnionRouter::new().await?;
    
    // Create a 3-hop circuit
    let hops = vec![
        "12D3KooW...".parse()?, // Peer 1
        "12D3KooW...".parse()?, // Peer 2  
        "12D3KooW...".parse()?, // Peer 3
    ];
    
    let circuit = router.create_circuit(hops).await?;
    
    // Send anonymous message through circuit
    let message = b"Anonymous message";
    router.send_through_circuit(&circuit, message).await?;
    
    Ok(())
}
```

### NAT Traversal

```rust
use qudag_network::{NatTraversalManager, NatTraversalConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NatTraversalConfig::default();
    let nat_manager = NatTraversalManager::new(config).await?;
    
    // Attempt to create port mapping
    let external_port = nat_manager.create_port_mapping(8080).await?;
    println!("External port: {}", external_port);
    
    Ok(())
}
```

## Architecture

### Core Components

- **P2P Node**: Main networking coordinator
- **Onion Router**: Anonymous multi-hop routing
- **Dark Resolver**: Decentralized domain name system
- **NAT Traversal**: Firewall and NAT penetration
- **Peer Discovery**: Kademlia DHT for peer finding
- **Traffic Obfuscation**: Message disguising and padding

### Network Stack

```
┌─────────────────────┐
│   Application       │
├─────────────────────┤
│   QuDAG Protocol    │
├─────────────────────┤
│   Onion Routing     │
├─────────────────────┤
│   LibP2P Transport  │
├─────────────────────┤
│   TCP/QUIC/WebSocket│
└─────────────────────┘
```

## Dark Addressing System

Create your own `.dark` domains:

```rust
use qudag_network::DarkResolver;

// Register domains
resolver.register_domain("chat.dark", peer_id).await?;
resolver.register_domain("files.dark", peer_id).await?;

// Create temporary shadow addresses
let shadow = resolver.create_shadow_address(3600).await?; // 1 hour TTL
println!("Temporary address: {}", shadow);
```

## Configuration

```rust
use qudag_network::NetworkConfig;
use std::time::Duration;

let config = NetworkConfig {
    listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse()?],
    bootstrap_peers: vec![],
    max_connections: 100,
    connection_timeout: Duration::from_secs(30),
    enable_mdns: true,
    enable_relay: true,
    enable_dcutr: true,
    timeout: Duration::from_secs(60),
};
```

## Security Features

- **Quantum-Resistant Encryption**: ML-KEM for session keys
- **Traffic Analysis Resistance**: Padding and timing obfuscation
- **Metadata Protection**: Onion routing hides communication patterns
- **Forward Secrecy**: Session keys are regularly rotated

## Features

- `optimizations`: Enable performance optimizations
- `message-chunking`: Large message splitting
- `adaptive-batching`: Intelligent message batching

## Peer Discovery

The network uses Kademlia DHT for decentralized peer discovery:

```rust
use qudag_network::KademliaDHT;

let dht = KademliaDHT::new(local_peer_id);
dht.bootstrap(&bootstrap_peers).await?;

// Find peers for content
let peers = dht.get_providers("content_hash").await?;
```

## Documentation

- [API Documentation](https://docs.rs/qudag-network)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)

## License

Licensed under either MIT or Apache-2.0 at your option.