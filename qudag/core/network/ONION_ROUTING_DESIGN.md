# QuDAG Onion Routing Design Specification

## Overview

This document details the design and implementation of the onion routing system for QuDAG, integrating ML-KEM (post-quantum) encryption with traditional onion routing techniques to provide anonymous, secure communication.

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Onion Routing System                      │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │ Circuit Builder │  │  Route Selector │  │ Mix Network  ││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │  ML-KEM Crypto  │  │ Cell Encryption │  │Traffic Mixer ││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │ Circuit Manager │  │ Relay Protocol  │  │ Exit Policy  ││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## 1. Circuit Construction Protocol

### 1.1 Circuit Types

```rust
pub enum CircuitType {
    /// Standard 3-hop circuit for general use
    Standard,
    
    /// Extended 5-hop circuit for high security
    Extended,
    
    /// Fast 2-hop circuit for low-latency applications
    Fast,
    
    /// Custom circuit with specified parameters
    Custom {
        min_hops: usize,
        max_hops: usize,
        selection_criteria: SelectionCriteria,
    },
}

pub struct SelectionCriteria {
    /// Minimum reputation score for relay nodes
    min_reputation: f64,
    
    /// Geographic diversity requirements
    geographic_diversity: bool,
    
    /// Bandwidth requirements
    min_bandwidth: u64,
    
    /// Latency constraints
    max_latency: Duration,
    
    /// Exclude certain jurisdictions
    excluded_countries: Vec<String>,
}
```

### 1.2 ML-KEM Handshake Protocol

```rust
/// Post-quantum secure handshake for circuit establishment
pub struct MLKEMHandshake {
    /// Our ML-KEM keypair
    our_keypair: MLKEMKeyPair,
    
    /// Security level for the handshake
    security_level: MLKEMSecurityLevel,
    
    /// Handshake state
    state: HandshakeState,
}

pub enum HandshakeState {
    /// Initial state
    Initial,
    
    /// Sent CREATE cell, waiting for CREATED
    SentCreate {
        circuit_id: CircuitId,
        timestamp: Instant,
    },
    
    /// Received CREATED, handshake complete
    Completed {
        shared_secret: SharedSecret,
        circuit_keys: CircuitKeys,
    },
}

impl MLKEMHandshake {
    /// Initiate handshake with a relay
    pub async fn initiate(&mut self, relay: &RelayInfo) -> Result<CreateCell> {
        // Generate ephemeral ML-KEM keypair for this circuit
        let ephemeral_keypair = MLKEMKeyPair::generate(self.security_level);
        
        // Encapsulate to relay's public key
        let (ciphertext, shared_secret) = relay.ml_kem_public_key.encapsulate()?;
        
        // Create handshake data
        let handshake_data = HandshakeData {
            protocol_version: ONION_PROTOCOL_VERSION,
            ephemeral_public_key: ephemeral_keypair.public_key(),
            supported_extensions: vec![
                Extension::QuantumResistant,
                Extension::TrafficPadding,
                Extension::CongestionControl,
            ],
        };
        
        // Sign handshake data with ML-DSA
        let signature = self.sign_handshake(&handshake_data)?;
        
        let create_cell = CreateCell {
            circuit_id: CircuitId::generate(),
            cell_type: CellType::Create,
            kem_ciphertext: ciphertext,
            handshake_data,
            signature,
            padding: generate_padding(CREATE_CELL_SIZE),
        };
        
        self.state = HandshakeState::SentCreate {
            circuit_id: create_cell.circuit_id,
            timestamp: Instant::now(),
        };
        
        Ok(create_cell)
    }
}
```

### 1.3 Circuit Building Algorithm

```rust
pub struct CircuitBuilder {
    /// Node selection strategy
    node_selector: Arc<NodeSelector>,
    
    /// Circuit manager
    circuit_manager: Arc<CircuitManager>,
    
    /// ML-KEM crypto
    ml_kem: Arc<QuantumKeyExchange>,
}

impl CircuitBuilder {
    pub async fn build_circuit(&self, params: CircuitParams) -> Result<Circuit> {
        // Step 1: Select nodes for the circuit
        let nodes = self.select_circuit_nodes(&params).await?;
        
        // Step 2: Build circuit incrementally
        let mut circuit = Circuit::new(params.circuit_type);
        
        for (i, node) in nodes.iter().enumerate() {
            // Extend circuit to next node
            self.extend_circuit(&mut circuit, node, i == 0).await?;
        }
        
        // Step 3: Finalize circuit
        circuit.finalize()?;
        
        // Step 4: Register with circuit manager
        self.circuit_manager.register_circuit(circuit.clone()).await;
        
        Ok(circuit)
    }
    
    async fn extend_circuit(&self, circuit: &mut Circuit, node: &NodeInfo, is_first: bool) -> Result<()> {
        if is_first {
            // Create new circuit with first hop
            let handshake = MLKEMHandshake::new(self.ml_kem.clone());
            let create_cell = handshake.initiate(node).await?;
            
            // Send CREATE cell and wait for CREATED
            let created_cell = self.send_and_wait_created(node, create_cell).await?;
            
            // Process CREATED response
            let (shared_secret, circuit_keys) = handshake.complete(created_cell)?;
            
            circuit.add_hop(CircuitHop {
                node_id: node.peer_id,
                shared_secret,
                circuit_keys,
                state: HopState::Active,
            });
        } else {
            // Extend existing circuit
            let extend_cell = self.create_extend_cell(circuit, node)?;
            
            // Send through existing circuit
            circuit.send_cell(extend_cell).await?;
            
            // Wait for EXTENDED response
            let extended_cell = circuit.receive_cell().await?;
            
            // Process EXTENDED response
            let hop = self.process_extended(extended_cell, node)?;
            circuit.add_hop(hop);
        }
        
        Ok(())
    }
}
```

## 2. Cell Encryption and Format

### 2.1 Cell Structure

```rust
/// Fixed-size onion cell for traffic analysis resistance
pub struct OnionCell {
    /// Circuit identifier
    pub circuit_id: CircuitId,
    
    /// Cell command type
    pub command: CellCommand,
    
    /// Encrypted payload (fixed size)
    pub payload: [u8; CELL_PAYLOAD_SIZE],
    
    /// AEAD authentication tag
    pub auth_tag: [u8; 16],
}

pub const CELL_SIZE: usize = 514; // Standard cell size
pub const CELL_PAYLOAD_SIZE: usize = 489; // Payload after headers

pub enum CellCommand {
    /// Padding cell for traffic analysis resistance
    Padding,
    
    /// Create new circuit
    Create,
    
    /// Circuit created successfully
    Created,
    
    /// Extend circuit
    Extend,
    
    /// Circuit extended successfully  
    Extended,
    
    /// Relay data
    Relay(RelayCommand),
    
    /// Destroy circuit
    Destroy,
}

pub enum RelayCommand {
    /// Begin new stream
    Begin,
    
    /// Stream data
    Data,
    
    /// End stream
    End,
    
    /// Send data and close
    SendMe,
    
    /// Extend circuit (relay)
    Extend,
    
    /// Circuit extended (relay)
    Extended,
    
    /// Truncate circuit
    Truncate,
    
    /// Circuit truncated
    Truncated,
    
    /// Connection refused
    ConnectRefused,
}
```

### 2.2 Layered Encryption

```rust
pub struct LayeredEncryption {
    /// Encryption keys for each hop
    hop_keys: Vec<HopKeys>,
    
    /// Current circuit state
    circuit_state: CircuitState,
}

pub struct HopKeys {
    /// Forward encryption key (client to relay)
    forward_key: ChaCha20Poly1305Key,
    
    /// Backward encryption key (relay to client)
    backward_key: ChaCha20Poly1305Key,
    
    /// Forward digest for integrity
    forward_digest: Blake3,
    
    /// Backward digest for integrity
    backward_digest: Blake3,
    
    /// Key derivation function
    kdf: HKDF,
}

impl LayeredEncryption {
    /// Apply onion encryption layers
    pub fn encrypt_outbound(&self, cell: &mut OnionCell) -> Result<()> {
        // Apply encryption in reverse order (exit to entry)
        for hop_keys in self.hop_keys.iter().rev() {
            // Encrypt payload
            let nonce = self.derive_nonce(&cell.circuit_id, Direction::Forward);
            hop_keys.forward_key.encrypt_in_place(&nonce, &[], &mut cell.payload)?;
            
            // Update digest
            hop_keys.forward_digest.update(&cell.payload);
            
            // Add authentication tag
            let tag = hop_keys.forward_digest.finalize_xof().take(16);
            cell.auth_tag.copy_from_slice(&tag);
        }
        
        Ok(())
    }
    
    /// Remove one layer of encryption (at relay)
    pub fn decrypt_layer(&self, cell: &mut OnionCell, hop_index: usize) -> Result<()> {
        let hop_keys = &self.hop_keys[hop_index];
        
        // Verify authentication tag
        let expected_tag = hop_keys.forward_digest.clone()
            .update(&cell.payload)
            .finalize_xof()
            .take(16);
            
        if !constant_time_eq(&cell.auth_tag, &expected_tag) {
            return Err(OnionError::AuthenticationFailed);
        }
        
        // Decrypt payload
        let nonce = self.derive_nonce(&cell.circuit_id, Direction::Forward);
        hop_keys.forward_key.decrypt_in_place(&nonce, &[], &mut cell.payload)?;
        
        Ok(())
    }
}
```

## 3. Traffic Analysis Resistance

### 3.1 Traffic Mixing

```rust
pub struct TrafficMixer {
    /// Mixing pool for cells
    cell_pool: Arc<Mutex<MixingPool>>,
    
    /// Mixing parameters
    config: MixingConfig,
    
    /// Statistics collector
    stats: Arc<MixingStats>,
}

pub struct MixingConfig {
    /// Minimum mixing delay
    min_delay: Duration,
    
    /// Maximum mixing delay
    max_delay: Duration,
    
    /// Batch size for sending
    batch_size: usize,
    
    /// Dummy traffic percentage
    dummy_traffic_rate: f64,
    
    /// Adaptive mixing based on traffic patterns
    adaptive_mixing: bool,
}

pub struct MixingPool {
    /// Cells waiting to be sent
    pending_cells: VecDeque<MixedCell>,
    
    /// Dummy cell generator
    dummy_generator: DummyGenerator,
    
    /// Send scheduler
    scheduler: SendScheduler,
}

pub struct MixedCell {
    /// The actual cell
    cell: OnionCell,
    
    /// When it was added to pool
    added_at: Instant,
    
    /// Scheduled send time
    send_at: Instant,
    
    /// Priority level
    priority: Priority,
}

impl TrafficMixer {
    /// Add cell to mixing pool
    pub async fn add_cell(&self, cell: OnionCell, priority: Priority) {
        let mut pool = self.cell_pool.lock().await;
        
        // Calculate send time based on mixing parameters
        let delay = self.calculate_mixing_delay(&pool, priority);
        let send_at = Instant::now() + delay;
        
        let mixed_cell = MixedCell {
            cell,
            added_at: Instant::now(),
            send_at,
            priority,
        };
        
        pool.pending_cells.push_back(mixed_cell);
        
        // Maybe add dummy cells
        if self.should_add_dummy_traffic(&pool) {
            self.add_dummy_cells(&mut pool).await;
        }
    }
    
    /// Background task for sending mixed cells
    pub async fn mixing_loop(&self) {
        let mut interval = interval(Duration::from_millis(10));
        
        loop {
            interval.tick().await;
            
            let cells_to_send = self.get_cells_to_send().await;
            
            if !cells_to_send.is_empty() {
                self.send_batch(cells_to_send).await;
            }
        }
    }
    
    fn calculate_mixing_delay(&self, pool: &MixingPool, priority: Priority) -> Duration {
        let base_delay = match priority {
            Priority::Critical => self.config.min_delay,
            Priority::High => self.config.min_delay * 2,
            Priority::Normal => (self.config.min_delay + self.config.max_delay) / 2,
            Priority::Low => self.config.max_delay,
        };
        
        if self.config.adaptive_mixing {
            // Adjust based on current traffic patterns
            let load_factor = pool.pending_cells.len() as f64 / 1000.0;
            let adjusted_delay = base_delay.mul_f64(1.0 + load_factor);
            adjusted_delay.min(self.config.max_delay)
        } else {
            base_delay
        }
    }
}
```

### 3.2 Padding and Cover Traffic

```rust
pub struct PaddingStrategy {
    /// Padding configuration
    config: PaddingConfig,
    
    /// Traffic pattern analyzer
    pattern_analyzer: TrafficPatternAnalyzer,
}

pub struct PaddingConfig {
    /// Enable adaptive padding
    adaptive: bool,
    
    /// Minimum padding cells per second
    min_padding_rate: f64,
    
    /// Maximum padding cells per second
    max_padding_rate: f64,
    
    /// Burst padding for activity spikes
    burst_padding: bool,
}

impl PaddingStrategy {
    /// Generate padding cells based on traffic patterns
    pub async fn generate_padding(&self, circuit: &Circuit) -> Vec<OnionCell> {
        let current_rate = self.pattern_analyzer.analyze_circuit(circuit);
        
        let padding_rate = if self.config.adaptive {
            self.calculate_adaptive_rate(current_rate)
        } else {
            self.config.min_padding_rate
        };
        
        let num_cells = (padding_rate * 0.01) as usize; // For 10ms interval
        
        (0..num_cells)
            .map(|_| self.create_padding_cell(circuit))
            .collect()
    }
    
    fn create_padding_cell(&self, circuit: &Circuit) -> OnionCell {
        let mut rng = thread_rng();
        let mut payload = [0u8; CELL_PAYLOAD_SIZE];
        rng.fill_bytes(&mut payload);
        
        OnionCell {
            circuit_id: circuit.id,
            command: CellCommand::Padding,
            payload,
            auth_tag: [0u8; 16], // Will be filled during encryption
        }
    }
}
```

## 4. Relay Node Implementation

### 4.1 Relay Protocol

```rust
pub struct RelayNode {
    /// Node identity
    identity: RelayIdentity,
    
    /// Active circuits
    circuits: Arc<DashMap<CircuitId, RelayCircuit>>,
    
    /// Exit policy
    exit_policy: ExitPolicy,
    
    /// Bandwidth limiter
    bandwidth_limiter: BandwidthLimiter,
    
    /// Statistics
    stats: Arc<RelayStats>,
}

pub struct RelayIdentity {
    /// ML-DSA signing key
    signing_key: MLDSAKeyPair,
    
    /// ML-KEM encryption key
    encryption_key: MLKEMKeyPair,
    
    /// Node fingerprint
    fingerprint: NodeFingerprint,
    
    /// Advertised capabilities
    capabilities: RelayCapabilities,
}

pub struct RelayCircuit {
    /// Circuit identifier
    circuit_id: CircuitId,
    
    /// Previous hop (or client)
    prev_hop: Option<PeerId>,
    
    /// Next hop (or exit)
    next_hop: Option<PeerId>,
    
    /// Circuit keys
    keys: CircuitKeys,
    
    /// Circuit state
    state: CircuitState,
    
    /// Creation time
    created_at: Instant,
}

impl RelayNode {
    /// Handle incoming cell
    pub async fn handle_cell(&self, cell: OnionCell, from: PeerId) -> Result<()> {
        match cell.command {
            CellCommand::Create => self.handle_create(cell, from).await,
            CellCommand::Extend => self.handle_extend(cell, from).await,
            CellCommand::Relay(relay_cmd) => self.handle_relay(cell, relay_cmd, from).await,
            CellCommand::Destroy => self.handle_destroy(cell, from).await,
            CellCommand::Padding => Ok(()), // Silently drop padding
            _ => Err(OnionError::UnexpectedCommand),
        }
    }
    
    async fn handle_create(&self, cell: OnionCell, from: PeerId) -> Result<()> {
        // Parse CREATE cell
        let create_data: CreateCell = cell.parse()?;
        
        // Perform ML-KEM decapsulation
        let shared_secret = self.identity.encryption_key
            .decapsulate(&create_data.kem_ciphertext)?;
        
        // Derive circuit keys
        let circuit_keys = self.derive_circuit_keys(&shared_secret);
        
        // Create relay circuit entry
        let relay_circuit = RelayCircuit {
            circuit_id: create_data.circuit_id,
            prev_hop: Some(from),
            next_hop: None,
            keys: circuit_keys.clone(),
            state: CircuitState::Open,
            created_at: Instant::now(),
        };
        
        self.circuits.insert(create_data.circuit_id, relay_circuit);
        
        // Send CREATED response
        let created_cell = self.create_created_cell(create_data.circuit_id, circuit_keys);
        self.send_cell(created_cell, from).await
    }
    
    async fn handle_relay(&self, mut cell: OnionCell, relay_cmd: RelayCommand, from: PeerId) -> Result<()> {
        // Get circuit
        let circuit = self.circuits.get(&cell.circuit_id)
            .ok_or(OnionError::CircuitNotFound)?;
        
        // Decrypt one layer
        self.decrypt_relay_cell(&mut cell, &circuit.keys)?;
        
        match relay_cmd {
            RelayCommand::Extend => {
                // Process EXTEND command
                let extend_data: ExtendCell = cell.parse()?;
                self.process_extend(circuit.value(), extend_data).await
            }
            RelayCommand::Data => {
                // Forward data to next hop or process as exit
                if let Some(next_hop) = &circuit.next_hop {
                    // Re-encrypt for next hop
                    self.encrypt_relay_cell(&mut cell, &circuit.keys)?;
                    self.send_cell(cell, *next_hop).await
                } else {
                    // We are the exit node
                    self.handle_exit_traffic(cell, circuit.value()).await
                }
            }
            _ => self.handle_other_relay_command(relay_cmd, cell, circuit.value()).await,
        }
    }
}
```

### 4.2 Exit Policy

```rust
pub struct ExitPolicy {
    /// Allowed ports
    allowed_ports: HashSet<u16>,
    
    /// Blocked addresses/ranges
    blocked_addresses: Vec<IpNetwork>,
    
    /// Rate limiting rules
    rate_limits: RateLimitRules,
    
    /// Traffic filtering
    traffic_filter: TrafficFilter,
}

impl ExitPolicy {
    /// Check if connection is allowed
    pub fn is_allowed(&self, addr: &SocketAddr) -> bool {
        // Check port
        if !self.allowed_ports.contains(&addr.port()) {
            return false;
        }
        
        // Check IP blocks
        for blocked in &self.blocked_addresses {
            if blocked.contains(addr.ip()) {
                return false;
            }
        }
        
        // Check rate limits
        if !self.rate_limits.check_allowed(addr) {
            return false;
        }
        
        true
    }
}
```

## 5. Circuit Management

### 5.1 Circuit Lifecycle

```rust
pub struct CircuitManager {
    /// Active circuits
    circuits: Arc<DashMap<CircuitId, ManagedCircuit>>,
    
    /// Circuit builder
    builder: Arc<CircuitBuilder>,
    
    /// Health monitor
    health_monitor: Arc<CircuitHealthMonitor>,
    
    /// Cleanup scheduler
    cleanup_scheduler: Arc<CleanupScheduler>,
}

pub struct ManagedCircuit {
    /// The circuit
    circuit: Circuit,
    
    /// Usage statistics
    stats: CircuitStats,
    
    /// Health status
    health: CircuitHealth,
    
    /// Last activity
    last_activity: Instant,
}

pub struct CircuitStats {
    /// Bytes sent through circuit
    bytes_sent: AtomicU64,
    
    /// Bytes received through circuit
    bytes_received: AtomicU64,
    
    /// Number of streams
    stream_count: AtomicU32,
    
    /// Circuit build time
    build_time: Duration,
    
    /// Average latency
    avg_latency: Duration,
}

impl CircuitManager {
    /// Get or create circuit for destination
    pub async fn get_circuit(&self, destination: &Destination) -> Result<Arc<Circuit>> {
        // Check for existing suitable circuit
        if let Some(circuit) = self.find_suitable_circuit(destination).await {
            return Ok(circuit);
        }
        
        // Build new circuit
        let params = self.determine_circuit_params(destination);
        let circuit = self.builder.build_circuit(params).await?;
        
        // Register circuit
        self.register_circuit(circuit.clone()).await;
        
        Ok(Arc::new(circuit))
    }
    
    /// Find existing circuit suitable for destination
    async fn find_suitable_circuit(&self, destination: &Destination) -> Option<Arc<Circuit>> {
        for entry in self.circuits.iter() {
            let managed = entry.value();
            
            // Check if circuit is healthy
            if managed.health != CircuitHealth::Healthy {
                continue;
            }
            
            // Check if circuit can handle more streams
            if managed.stats.stream_count.load(Ordering::Relaxed) >= MAX_STREAMS_PER_CIRCUIT {
                continue;
            }
            
            // Check if exit policy allows destination
            if !managed.circuit.exit_policy_allows(destination) {
                continue;
            }
            
            return Some(Arc::new(managed.circuit.clone()));
        }
        
        None
    }
    
    /// Background task for circuit maintenance
    pub async fn maintenance_loop(&self) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Clean up dead circuits
            self.cleanup_dead_circuits().await;
            
            // Refresh expiring circuits
            self.refresh_expiring_circuits().await;
            
            // Build preemptive circuits
            self.build_preemptive_circuits().await;
        }
    }
}
```

## 6. Integration with Network Stack

### 6.1 Network Manager Integration

```rust
impl NetworkManager {
    /// Send message through onion routing
    pub async fn send_anonymous(&self, message: NetworkMessage, destination: PeerId) -> Result<()> {
        // Get or create circuit
        let circuit = self.circuit_manager.get_circuit(&destination.into()).await?;
        
        // Serialize message
        let data = bincode::serialize(&message)?;
        
        // Fragment into cells
        let cells = self.fragment_into_cells(data, &circuit);
        
        // Send cells through circuit
        for cell in cells {
            circuit.send_cell(cell).await?;
        }
        
        Ok(())
    }
    
    /// Receive message from onion routing
    pub async fn receive_anonymous(&self) -> Result<NetworkMessage> {
        let cells = self.onion_receiver.receive_cells().await?;
        
        // Reassemble message
        let data = self.reassemble_from_cells(cells)?;
        
        // Deserialize
        let message = bincode::deserialize(&data)?;
        
        Ok(message)
    }
}
```

## 7. Security Considerations

### 7.1 Threat Model

1. **Network Adversary**: Can observe, modify, and inject traffic
2. **Compromised Relays**: Some fraction of relays may be malicious
3. **Timing Attacks**: Adversary can perform traffic correlation
4. **Quantum Adversary**: Future quantum computers may break classical crypto

### 7.2 Mitigations

1. **ML-KEM Integration**: Post-quantum secure key exchange
2. **Traffic Padding**: Constant-rate cover traffic
3. **Cell Size Normalization**: Fixed-size cells prevent size analysis
4. **Mixing Delays**: Random delays prevent timing correlation
5. **Circuit Rotation**: Regular circuit replacement
6. **Entry Guards**: Stable entry nodes reduce attack surface

## 8. Performance Optimizations

### 8.1 Circuit Reuse

```rust
pub struct CircuitPool {
    /// Pre-built circuits ready for use
    ready_circuits: Arc<Mutex<Vec<Circuit>>>,
    
    /// Target pool size
    target_size: usize,
    
    /// Circuit builder
    builder: Arc<CircuitBuilder>,
}

impl CircuitPool {
    /// Background task to maintain pool
    pub async fn maintain_pool(&self) {
        loop {
            let current_size = self.ready_circuits.lock().await.len();
            
            if current_size < self.target_size {
                // Build circuits in parallel
                let needed = self.target_size - current_size;
                let futures: Vec<_> = (0..needed)
                    .map(|_| self.builder.build_circuit(CircuitParams::default()))
                    .collect();
                
                let circuits = join_all(futures).await;
                
                let mut pool = self.ready_circuits.lock().await;
                for circuit in circuits.into_iter().flatten() {
                    pool.push(circuit);
                }
            }
            
            sleep(Duration::from_secs(10)).await;
        }
    }
}
```

### 8.2 Congestion Control

```rust
pub struct CongestionController {
    /// Circuit congestion states
    circuit_states: Arc<DashMap<CircuitId, CongestionState>>,
    
    /// Global rate limiter
    rate_limiter: Arc<RateLimiter>,
}

pub struct CongestionState {
    /// Current congestion window
    cwnd: usize,
    
    /// Slow start threshold
    ssthresh: usize,
    
    /// Round-trip time estimate
    rtt_estimate: Duration,
    
    /// Packets in flight
    inflight: usize,
}

impl CongestionController {
    /// Check if can send on circuit
    pub fn can_send(&self, circuit_id: CircuitId) -> bool {
        if let Some(state) = self.circuit_states.get(&circuit_id) {
            state.inflight < state.cwnd
        } else {
            true
        }
    }
    
    /// Update congestion state on ACK
    pub fn on_ack(&self, circuit_id: CircuitId, rtt: Duration) {
        if let Some(mut state) = self.circuit_states.get_mut(&circuit_id) {
            // Update RTT estimate
            state.rtt_estimate = (state.rtt_estimate * 7 + rtt) / 8;
            
            // Congestion window update
            if state.cwnd < state.ssthresh {
                // Slow start
                state.cwnd += 1;
            } else {
                // Congestion avoidance
                state.cwnd += 1 / state.cwnd;
            }
            
            state.inflight = state.inflight.saturating_sub(1);
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_construction() {
        let builder = CircuitBuilder::new_test();
        let circuit = builder.build_circuit(CircuitParams::default()).await.unwrap();
        
        assert_eq!(circuit.hop_count(), 3);
        assert_eq!(circuit.state(), CircuitState::Open);
    }
    
    #[tokio::test]
    async fn test_onion_encryption() {
        let circuit = create_test_circuit();
        let cell = create_test_cell();
        
        let encrypted = circuit.encrypt_cell(cell.clone()).unwrap();
        let decrypted = circuit.decrypt_cell(encrypted).unwrap();
        
        assert_eq!(cell.payload, decrypted.payload);
    }
    
    #[tokio::test]
    async fn test_traffic_mixing() {
        let mixer = TrafficMixer::new_test();
        let cell = create_test_cell();
        
        mixer.add_cell(cell, Priority::Normal).await;
        
        // Should not send immediately
        let sent = mixer.get_sent_cells();
        assert!(sent.is_empty());
        
        // Should send after delay
        sleep(mixer.config.max_delay).await;
        let sent = mixer.get_sent_cells();
        assert_eq!(sent.len(), 1);
    }
}
```

## Conclusion

This onion routing design provides strong anonymity guarantees through:
- Post-quantum secure encryption with ML-KEM
- Multi-hop circuits with careful node selection
- Traffic analysis resistance through padding and mixing
- Integration with the broader QuDAG network stack

The modular architecture allows for easy testing and future enhancements while maintaining security properties.