# ML-KEM Based Onion Routing for QuDAG

This module implements a quantum-resistant onion routing system using ML-KEM-768 for anonymous communication within the QuDAG network.

## Key Features

### 1. **ML-KEM-768 Layer Encryption**
- Quantum-resistant key encapsulation using NIST-standardized ML-KEM
- Each hop uses independent ML-KEM keypairs
- Symmetric key derivation from shared secrets using HKDF
- ChaCha20-Poly1305 for payload encryption

### 2. **Circuit Construction and Management**
- Minimum 3-hop circuits for anonymity
- Automatic circuit rotation every 5 minutes
- Circuit quality scoring based on performance
- Rate-limited circuit creation (1 per second max)
- Bandwidth-weighted node selection

### 3. **Traffic Analysis Resistance**
- Mix nodes with batch processing
- Automatic dummy traffic injection
- Message size normalization to standard sizes
- Timing obfuscation with random delays
- Traffic pattern mimicking

### 4. **Directory Authority System**
- Node discovery and public key distribution
- Bandwidth measurement and load balancing
- Node capability flags (guard, exit, fast, stable)
- Hourly directory updates
- Weighted node selection based on bandwidth

### 5. **Advanced Features**
- Circuit teardown and cleanup
- Metadata protection with anonymization
- IP address anonymization
- Packet header scrubbing
- Flow correlation resistance

## Architecture

```
┌─────────────────────┐
│   Application       │
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│   Router (ML-KEM)   │
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│  Circuit Manager    │
├─────────────────────┤
│ • Build circuits    │
│ • Rotate circuits   │
│ • Quality scoring   │
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│  Onion Router       │
├─────────────────────┤
│ • ML-KEM encrypt    │
│ • Layer creation    │
│ • Metadata protect  │
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│    Mix Network      │
├─────────────────────┤
│ • Batch processing  │
│ • Dummy traffic     │
│ • Traffic shaping   │
└─────────────────────┘
```

## Usage Example

```rust
use qudag_network::onion::{MLKEMOnionRouter, CircuitManager, DirectoryClient};
use qudag_network::router::Router;
use qudag_network::types::{NetworkMessage, RoutingStrategy};

// Initialize components
let router = Router::new().await?;
let mut circuit_manager = CircuitManager::new();
let directory_client = DirectoryClient::new();

// Build anonymous circuit
let circuit_id = circuit_manager.build_circuit(3, &directory_client).await?;
circuit_manager.activate_circuit(circuit_id)?;

// Send anonymous message
let message = NetworkMessage {
    id: "anon-001".to_string(),
    source: vec![0u8; 32], // Anonymous source
    destination: vec![255u8; 32],
    payload: b"Secret message".to_vec(),
    priority: MessagePriority::High,
    ttl: Duration::from_secs(60),
};

let route = router.route(&message, RoutingStrategy::Anonymous { hops: 3 }).await?;
```

## Security Considerations

1. **Quantum Resistance**: ML-KEM-768 provides NIST security level 3
2. **Anonymity Set**: Minimum 3 hops prevents trivial traffic analysis
3. **Timing Attacks**: Random delays and batch processing resist timing correlation
4. **Traffic Analysis**: Size normalization and dummy traffic prevent pattern detection
5. **Metadata Leakage**: All metadata is protected and anonymized

## Performance

- Circuit creation: ~100ms (including ML-KEM operations)
- Per-hop latency: ~10-50ms (including obfuscation delays)
- Throughput: Limited by mix node batching (configurable)
- Memory: ~4KB per layer (normalized size)

## Configuration

Key parameters can be tuned:
- `max_circuits`: Maximum concurrent circuits (default: 100)
- `circuit_lifetime`: Circuit expiration time (default: 10 minutes)
- `rotation_interval`: Circuit rotation period (default: 5 minutes)
- `batch_size`: Mix node batch size (default: 100 messages)
- `dummy_probability`: Dummy traffic ratio (default: 0.1)

## Testing

Comprehensive test coverage includes:
- ML-KEM encryption/decryption
- Circuit lifecycle management
- Traffic analysis resistance
- Concurrent message processing
- Failure recovery
- Load balancing

Run tests with:
```bash
cargo test -p qudag-network onion
```

## Future Enhancements

- [ ] Pluggable transports for censorship resistance
- [ ] Bridge nodes for network access
- [ ] Hidden services support
- [ ] Congestion control
- [ ] Stream isolation
- [ ] Guard node persistence