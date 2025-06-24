# Anonymous Routing Implementation

The QuDAG network layer implements a sophisticated anonymous routing system combining onion routing, dark addressing, and traffic obfuscation to ensure comprehensive communication privacy.

## Core Components

### Router Implementation

The anonymous routing system is implemented through several key components:

#### MLKEMOnionRouter

The main onion router using quantum-resistant encryption:

- **Layered Encryption**: Uses ML-KEM for each routing layer
- **Circuit Length**: 3-7 hops for anonymity/performance balance
- **Forward Secrecy**: Fresh keys for each circuit
- **Performance Optimization**: Connection pooling and caching

```rust
pub struct MLKEMOnionRouter {
    // Quantum-resistant onion routing implementation
    circuits: HashMap<CircuitId, Circuit>,
    key_cache: LruCache<PeerId, MlKemKeys>,
}
```

#### Anonymous Routing Strategies

The system supports multiple routing strategies for different use cases:

1. **Direct**
   - Point-to-point communication
   - Uses routing table lookups
   - Minimal latency for known peers

2. **Flood**
   - Broadcast to all connected peers
   - Used for network announcements
   - Automatic peer discovery

3. **Random Subset**
   - Select random subset of peers
   - Load balancing across network
   - Network health probing

4. **Anonymous**
   - Full onion routing with multiple hops
   - Maximum privacy protection
   - Traffic analysis resistance

### Dark Addressing System

#### DarkResolver

Handles `.dark` domain resolution with quantum fingerprints:

```rust
pub struct DarkResolver {
    // Maps .dark domains to network addresses
    domain_cache: LruCache<String, DarkDomainRecord>,
    fingerprint_validator: QuantumFingerprint,
}
```

**Features**:
- Human-readable `.dark` domains (e.g., `myservice.dark`)
- Quantum fingerprint verification
- Decentralized domain resolution
- TTL-based caching

#### ShadowAddress System

Provides stealth addressing for enhanced privacy:

```rust
pub struct ShadowAddress {
    // Temporary stealth addresses
    address: String,
    network_type: NetworkType,
    metadata: ShadowMetadata,
    ttl: Duration,
}
```

**Capabilities**:
- Temporary `.shadow` addresses
- Time-based expiration
- Network type isolation
- Metadata protection

### Traffic Analysis Resistance

#### MetadataProtector

Protects against metadata analysis:

```rust
pub struct MetadataProtector {
    config: MetadataConfig,
    mix_strategy: MixingStrategy,
}
```

**Protection Methods**:
- Message size normalization
- Timing randomization
- Cover traffic generation
- Batch processing

#### TrafficAnalysisResistance

Advanced traffic analysis countermeasures:

```rust
pub struct TrafficAnalysisResistance {
    mix_nodes: Vec<MixNode>,
    batching_config: BatchConfig,
    timing_config: TimingConfig,
}
```

## Routing Implementation Details

### Circuit Building Process

1. **Peer Selection**
   ```rust
   // Select diverse peers for circuit
   let peers = select_diverse_peers(destination, min_hops, max_hops)?;
   ```

2. **Key Exchange**
   ```rust
   // Establish ML-KEM keys with each hop
   for peer in &peers {
       let (pk, sk) = ml_kem.keygen()?;
       let shared_secret = establish_shared_secret(peer, pk)?;
   }
   ```

3. **Circuit Construction**
   ```rust
   // Build layered encryption circuit
   let circuit = build_onion_circuit(&peers, &shared_secrets)?;
   ```

### Message Routing Flow

1. **Route Discovery**
   ```
   Client → DHT Query → Peer Discovery → Route Planning
   ```

2. **Circuit Establishment**
   ```
   Client → Hop 1 → Hop 2 → ... → Hop N → Destination
          ↳ ML-KEM Key Exchange at each hop
   ```

3. **Message Transmission**
   ```
   Message → Encrypt(Layer N) → ... → Encrypt(Layer 1) → Route
   ```

4. **Layer Peeling**
   ```
   Encrypted Onion → Hop 1 Decrypt → Hop 2 Decrypt → ... → Final Message
   ```

### Dark Domain Resolution

1. **Domain Query**
   ```rust
   let record = dark_resolver.resolve("myservice.dark").await?;
   ```

2. **Fingerprint Validation**
   ```rust
   let is_valid = quantum_fingerprint.verify(&record.data, &record.fingerprint)?;
   ```

3. **Address Resolution**
   ```rust
   let network_addr = NetworkAddress::from_record(&record)?;
   ```

### Shadow Address Generation

1. **Address Creation**
   ```rust
   let shadow = ShadowAddressGenerator::generate(
       NetworkType::Mainnet,
       Duration::from_secs(3600), // 1 hour TTL
   )?;
   ```

2. **Service Binding**
   ```rust
   shadow_resolver.bind_service(&shadow.address, service_endpoint).await?;
   ```

3. **Address Resolution**
   ```rust
   let endpoint = shadow_resolver.resolve(&shadow.address).await?;
   ```

## Performance Optimizations

### Connection Pooling

- Reuse established circuits when possible
- Connection warm-up for frequently accessed peers
- Graceful circuit teardown and cleanup

### Caching Strategies

- **Route Cache**: Cache successful routes for reuse
- **Key Cache**: Cache ML-KEM key pairs for performance
- **Domain Cache**: Cache resolved dark domains with TTL

### Batch Processing

- Batch multiple messages through same circuit
- Reduce circuit establishment overhead
- Improve overall throughput

## Security Features

### Quantum-Resistant Encryption

All routing layers use post-quantum cryptography:

- **ML-KEM**: Key encapsulation for each hop
- **ChaCha20Poly1305**: Message encryption with quantum-resistant keys
- **ML-DSA**: Route validation and integrity

### Forward Secrecy

- Fresh keys generated for each circuit
- No long-term circuit keys
- Automatic key rotation

### Traffic Analysis Protection

1. **Size Normalization**
   - Pad messages to standard sizes
   - Prevent size-based correlation

2. **Timing Randomization**
   - Random delays between hops
   - Prevent timing-based analysis

3. **Cover Traffic**
   - Generate dummy traffic
   - Hide real communication patterns

4. **Mixing**
   - Batch and reorder messages
   - Break temporal correlations

### Anonymity Guarantees

- **Sender Anonymity**: Source hidden through onion routing
- **Receiver Anonymity**: Dark/shadow addressing
- **Relationship Anonymity**: No single point knows both endpoints
- **Traffic Analysis Resistance**: Multiple countermeasures

## Error Handling and Resilience

### Circuit Failures

- Automatic circuit rebuilding on failure
- Fallback routes for reliability
- Graceful degradation strategies

### Peer Failures

- Dynamic peer replacement
- Route healing capabilities
- Network partition tolerance

### Attack Mitigation

- **Sybil Attack**: Diverse peer selection
- **Eclipse Attack**: Multiple route diversity
- **Traffic Analysis**: Comprehensive countermeasures
- **Timing Attacks**: Constant-time operations

## Configuration Options

### Router Configuration

```rust
pub struct RouterConfig {
    min_circuit_length: usize,      // Default: 3
    max_circuit_length: usize,      // Default: 7
    circuit_ttl: Duration,          // Default: 1 hour
    max_circuits: usize,            // Default: 100
    enable_cover_traffic: bool,     // Default: true
}
```

### Dark Addressing Configuration

```rust
pub struct DarkResolverConfig {
    cache_size: usize,              // Default: 1000
    default_ttl: Duration,          // Default: 1 hour
    enable_fingerprint_validation: bool, // Default: true
}
```

### Traffic Analysis Resistance Configuration

```rust
pub struct TrafficAnalysisConfig {
    enable_size_padding: bool,      // Default: true
    enable_timing_randomization: bool, // Default: true
    cover_traffic_rate: f64,        // Default: 0.1 (10%)
    batch_size: usize,              // Default: 10
}
```

## Best Practices

### For Applications

1. **Use appropriate routing strategy** for your privacy needs
2. **Enable cover traffic** for sensitive applications
3. **Monitor circuit health** and rebuild when necessary
4. **Use dark domains** for stable service addressing

### For Node Operators

1. **Configure adequate peer connections** for route diversity
2. **Enable traffic analysis resistance** features
3. **Monitor performance metrics** and adjust accordingly
4. **Maintain good network connectivity** for reliability

### For Developers

1. **Test anonymity properties** with network simulation
2. **Profile routing performance** under different loads
3. **Implement proper error handling** for circuit failures
4. **Follow constant-time programming** practices

This anonymous routing implementation provides comprehensive privacy protection while maintaining good performance characteristics for the QuDAG protocol.