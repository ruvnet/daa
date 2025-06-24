# QuDAG Protocol Architecture Documentation

This directory contains comprehensive architectural documentation for the QuDAG quantum-resistant DAG-based anonymous communication protocol implementation.

## System Architecture Overview

QuDAG is built as a modular workspace architecture with the following core components:

```
QuDAG Protocol Architecture
├── Core Modules
│   ├── crypto/      # Quantum-resistant cryptographic primitives
│   ├── dag/         # DAG consensus with QR-Avalanche algorithm
│   ├── network/     # P2P networking with anonymous routing
│   └── protocol/    # Main protocol coordination and management
├── Tools
│   ├── cli/         # Command-line interface for node operations
│   └── simulator/   # Network simulation and testing framework
├── Benchmarks       # Performance benchmarking and regression testing
└── Infrastructure   # Deployment configurations (Docker, K8s, Terraform)
```

## Component Interactions

### High-Level Protocol Flow

1. **Node Initialization**
   - Cryptographic key generation (ML-KEM, ML-DSA)
   - Network peer discovery via Kademlia DHT
   - DAG consensus initialization
   - Protocol coordinator startup

2. **Message Processing**
   - Message creation and quantum-resistant signing
   - DAG vertex creation and parent selection
   - Consensus validation via QR-Avalanche
   - Anonymous network routing and delivery

3. **Consensus Operation**
   - Vertex validation and conflict detection
   - Byzantine fault-tolerant consensus
   - Finality determination and ordering
   - State synchronization across nodes

### Inter-Module Communication

```
[Protocol] ←→ [Crypto]     # Key management and signatures
[Protocol] ←→ [DAG]        # Consensus and ordering
[Protocol] ←→ [Network]    # P2P communication
[DAG] ←→ [Crypto]          # Vertex signing and validation
[Network] ←→ [Crypto]      # Transport encryption
```

## Design Decisions

### 1. Quantum-Resistant Cryptography

**Decision**: Use NIST standardized post-quantum algorithms
- **ML-KEM-768**: Key encapsulation (NIST Level 3 security)
- **ML-DSA**: Digital signatures with constant-time operations
- **BLAKE3**: Quantum-resistant hashing

**Rationale**: Prepare for quantum computing threats while maintaining performance

### 2. DAG-Based Consensus

**Decision**: Implement QR-Avalanche consensus on DAG structure
- **Benefits**: High throughput, low latency, Byzantine fault tolerance
- **Trade-offs**: Complex implementation vs. linear blockchain

**Rationale**: Better scalability and performance than traditional blockchain

### 3. Anonymous Routing

**Decision**: Onion routing with traffic obfuscation
- **Layers**: Multi-hop routing with ChaCha20Poly1305 encryption
- **Mixing**: Random delays and padding for traffic analysis resistance

**Rationale**: Protect user privacy and prevent network analysis

### 4. Memory-Safe Implementation

**Decision**: Rust with zero unsafe code
- **Safety**: Automatic memory management and thread safety
- **Security**: Cryptographic memory zeroization

**Rationale**: Prevent memory-based attacks and vulnerabilities

## Protocol Specifications

### Message Format

```rust
struct DagMessage {
    id: VertexId,           // Unique message identifier
    payload: Vec<u8>,       // Message content
    parents: HashSet<VertexId>, // DAG parent references
    timestamp: u64,         // Creation timestamp
    signature: MlDsaSignature, // Quantum-resistant signature
}
```

### Consensus Algorithm

**QR-Avalanche Properties**:
- **Safety**: Byzantine fault tolerance (< 1/3 adversarial nodes)
- **Liveness**: Progress guarantee under network conditions
- **Finality**: Probabilistic finality with high confidence
- **Performance**: Sub-second finality, 10k+ TPS

### Network Protocol

**Anonymous Routing**:
- **Circuit Length**: 3-7 hops for anonymity/performance balance
- **Encryption**: Layered encryption with forward secrecy
- **Discovery**: Kademlia DHT for decentralized peer discovery

## Technical Diagrams

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                  QuDAG Protocol                         │
├─────────────────────────────────────────────────────────┤
│  Protocol Coordinator                                   │
│  ├── Message Validation     ├── State Management       │
│  ├── Component Coordination ├── Error Handling         │
│  └── Metrics Collection     └── Resource Management    │
├─────────────────────────────────────────────────────────┤
│  Core Modules                                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │   Crypto    │ │     DAG     │ │   Network   │      │
│  │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────── │      │
│  │ │ ML-KEM  │ │ │ │Consensus│ │ │ │ Anonymous │      │
│  │ │ ML-DSA  │ │ │ │ Vertex  │ │ │ │ Routing   │      │
│  │ │ BLAKE3  │ │ │ │ Tip Sel │ │ │ │ P2P Comm  │      │
│  │ └─────────┘ │ │ └─────────┘ │ │ └─────────── │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
├─────────────────────────────────────────────────────────┤
│  Infrastructure                                         │
│  ├── Async Runtime (Tokio)  ├── Metrics (Prometheus)   │
│  ├── Logging (Tracing)      ├── Serialization (Serde)  │
│  └── Memory Management      └── Error Handling         │
└─────────────────────────────────────────────────────────┘
```

### Message Flow Sequence

```
Node A                    Network                    Node B
  │                        │                         │
  │── Create Message ──────┼─────────────────────────┤
  │                        │                         │
  │── Sign with ML-DSA ────┼─────────────────────────┤
  │                        │                         │
  │── Add to DAG ──────────┼─────────────────────────┤
  │                        │                         │
  │── Encrypt & Route ─────┼──> Anonymous Routing ───┤
  │                        │                         │
  │                        ┼──> Hop 1 ──> Hop 2 ────┤
  │                        │                         │
  │                        ┼─────────────────────────┼──> Receive Message
  │                        │                         │
  │                        ┼─────────────────────────┼──> Verify Signature
  │                        │                         │
  │                        ┼─────────────────────────┼──> Add to DAG
  │                        │                         │
  │<─── Consensus Vote ────┼<────────────────────────┤
```

## Data Flow Documentation

### Cryptographic Operations Flow

1. **Key Generation**
   ```
   Entropy Source → ML-KEM Keygen → (Public Key, Secret Key)
                 → ML-DSA Keygen → (Signing Key, Verify Key)
   ```

2. **Message Signing**
   ```
   Message → Hash (BLAKE3) → Sign (ML-DSA) → Signature
   ```

3. **Encryption**
   ```
   Message → Encrypt (ML-KEM + ChaCha20) → Ciphertext
   ```

### DAG Consensus Flow

1. **Vertex Creation**
   ```
   Message → Create Vertex → Select Parents → Sign Vertex
   ```

2. **Consensus Process**
   ```
   Vertex → Validate → Query Network → Aggregate Votes → Finalize
   ```

3. **State Updates**
   ```
   Finalized Vertices → Update State → Sync Across Nodes
   ```

### Network Communication Flow

1. **Route Discovery**
   ```
   Destination → Query DHT → Build Circuit → Establish Route
   ```

2. **Message Transmission**
   ```
   Message → Encrypt Layers → Route Through Hops → Decrypt at Destination
   ```

3. **Peer Management**
   ```
   Bootstrap → Discover Peers → Maintain Connections → Handle Failures
   ```

## Documentation Structure

- **[Network Architecture](network/)**: Detailed networking implementation
  - [Anonymous Routing](network/anonymous_routing.md)
  - [Connection Management](network/connection_management.md)
  - [Message Handling](network/message_handling.md)
  - [Performance Optimizations](network/performance_optimizations.md)

## Performance Characteristics

### Throughput Metrics
- **Consensus**: 10,000+ transactions per second
- **Network**: 1,000+ messages per second per node
- **Crypto**: 2,000+ signatures per second

### Latency Metrics
- **Consensus Finality**: <1 second (99th percentile)
- **Network Routing**: <200ms average
- **Crypto Operations**: <2ms per operation

### Resource Usage
- **Base Memory**: ~50MB per node
- **Active Memory**: ~100MB under load
- **Network Bandwidth**: Adaptive based on load

## Development Guidelines

### Adding New Components

1. Follow the modular architecture pattern
2. Implement proper error handling with `thiserror`
3. Add comprehensive test coverage
4. Include performance benchmarks
5. Update documentation

### Security Considerations

1. Use quantum-resistant cryptography
2. Implement constant-time operations
3. Ensure memory safety and zeroization
4. Follow secure coding practices
5. Conduct security audits

### Performance Optimization

1. Profile critical paths
2. Implement efficient data structures
3. Use async/await for I/O operations
4. Minimize memory allocations
5. Benchmark against targets

This architecture documentation provides the foundation for understanding the QuDAG protocol implementation and serves as a reference for developers working on the system.