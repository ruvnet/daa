# QuDAG Dark Addressing System Design

## Overview

The Dark Addressing System provides anonymous, unlinkable network addresses for QuDAG nodes, enabling privacy-preserving communication without revealing real network endpoints. This system integrates with the onion routing layer and uses post-quantum cryptography for long-term security.

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                  Dark Addressing System                      │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │ Address Generator│  │ Address Encoder │  │ Checksum Gen ││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │  DHT Publisher  │  │  Resolver Cache │  │ Proof System ││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐│
│ │ Privacy Manager │  │ Rotation Policy │  │ Revocation   ││
│ └─────────────────┘  └─────────────────┘  └──────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## 1. Address Format and Structure

### 1.1 Address Components

```rust
/// Dark address structure (human-readable when encoded)
pub struct DarkAddress {
    /// Version byte for future compatibility
    version: u8,
    
    /// Network identifier (mainnet, testnet, etc.)
    network: NetworkType,
    
    /// Identity commitment (20 bytes)
    commitment: [u8; 20],
    
    /// Error correction checksum (4 bytes)
    checksum: [u8; 4],
}

/// Total size: 26 bytes
/// Encoded size: 52 characters (base32) or 35 characters (base58)

impl DarkAddress {
    /// Human-readable encoding using base32 (Tor-style)
    pub fn to_human_readable(&self) -> String {
        let mut data = Vec::with_capacity(26);
        data.push(self.version);
        data.push(self.network as u8);
        data.extend_from_slice(&self.commitment);
        data.extend_from_slice(&self.checksum);
        
        // Use base32 encoding for readability
        // Result looks like: "qd1a2b3c4d5e6f7g8h9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z"
        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &data)
            .to_lowercase()
    }
    
    /// Alternative encoding using Bech32 (Bitcoin-style)
    pub fn to_bech32(&self) -> String {
        let hrp = match self.network {
            NetworkType::Mainnet => "qd",
            NetworkType::Testnet => "tqd",
            NetworkType::Devnet => "dqd",
        };
        
        bech32::encode(hrp, self.to_bytes(), bech32::Variant::Bech32)
            .expect("Valid bech32 encoding")
    }
}
```

### 1.2 Address Generation Process

```rust
pub struct DarkAddressGenerator {
    /// Master identity key (ML-DSA)
    master_key: MLDSAKeyPair,
    
    /// Address derivation counter
    counter: AtomicU64,
    
    /// Network configuration
    network: NetworkType,
}

impl DarkAddressGenerator {
    /// Generate a new dark address
    pub fn generate(&self) -> (DarkAddress, AddressSecret) {
        // Step 1: Generate ephemeral keypair
        let ephemeral_key = MLDSAKeyPair::generate();
        
        // Step 2: Create commitment
        let commitment = self.create_commitment(&ephemeral_key);
        
        // Step 3: Generate address
        let address = DarkAddress {
            version: CURRENT_ADDRESS_VERSION,
            network: self.network,
            commitment: commitment.into(),
            checksum: self.calculate_checksum(&commitment),
        };
        
        // Step 4: Create secret for later proving ownership
        let secret = AddressSecret {
            ephemeral_key,
            master_signature: self.sign_address(&address),
            derivation_path: self.counter.fetch_add(1, Ordering::SeqCst),
        };
        
        (address, secret)
    }
    
    fn create_commitment(&self, ephemeral_key: &MLDSAKeyPair) -> [u8; 32] {
        let mut hasher = Blake3::new();
        
        // Commit to ephemeral public key
        hasher.update(&ephemeral_key.public_key().to_bytes());
        
        // Commit to master public key
        hasher.update(&self.master_key.public_key().to_bytes());
        
        // Add timestamp for freshness
        hasher.update(&SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_le_bytes());
        
        // Add network-specific salt
        hasher.update(self.network.as_bytes());
        
        hasher.finalize().into()
    }
    
    fn calculate_checksum(&self, commitment: &[u8; 32]) -> [u8; 4] {
        // Double-hash for checksum (like Bitcoin)
        let hash1 = Blake3::hash(commitment);
        let hash2 = Blake3::hash(hash1.as_bytes());
        
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&hash2.as_bytes()[..4]);
        checksum
    }
}
```

## 2. Address Resolution Protocol

### 2.1 DHT Storage Format

```rust
/// Record stored in DHT for address resolution
#[derive(Serialize, Deserialize)]
pub struct DarkAddressRecord {
    /// The dark address being mapped
    address: DarkAddress,
    
    /// Encrypted endpoint information
    encrypted_endpoint: EncryptedEndpoint,
    
    /// Proof of address ownership
    ownership_proof: OwnershipProof,
    
    /// Onion routing introduction points
    introduction_points: Vec<IntroductionPoint>,
    
    /// Record expiration time
    expires_at: SystemTime,
    
    /// Anti-spam proof of work
    proof_of_work: ProofOfWork,
}

/// Encrypted network endpoint
#[derive(Serialize, Deserialize)]
pub struct EncryptedEndpoint {
    /// ML-KEM ciphertext containing endpoint
    kem_ciphertext: Vec<u8>,
    
    /// Encrypted endpoint data
    encrypted_data: Vec<u8>,
    
    /// Encryption nonce
    nonce: [u8; 24],
}

/// Introduction point for establishing connection
#[derive(Serialize, Deserialize)]
pub struct IntroductionPoint {
    /// Relay node's public key
    relay_pubkey: MLKEMPublicKey,
    
    /// Relay node's onion address
    relay_address: OnionAddress,
    
    /// Authentication token
    auth_token: [u8; 32],
    
    /// Expiration time
    expires_at: SystemTime,
}
```

### 2.2 Resolution Process

```rust
pub struct DarkResolver {
    /// DHT client
    dht: Arc<EnhancedKademlia>,
    
    /// Resolution cache
    cache: Arc<ResolutionCache>,
    
    /// Privacy configuration
    privacy_config: PrivacyConfig,
    
    /// Proof verifier
    proof_verifier: ProofVerifier,
}

impl DarkResolver {
    /// Resolve dark address to network endpoint
    pub async fn resolve(&self, address: &DarkAddress) -> Result<ResolvedEndpoint> {
        // Step 1: Check cache
        if let Some(cached) = self.cache.get(address).await {
            return Ok(cached);
        }
        
        // Step 2: Privacy measures
        self.apply_privacy_measures().await?;
        
        // Step 3: Query DHT
        let record = self.query_dht_with_privacy(address).await?;
        
        // Step 4: Verify record
        self.verify_record(&record, address)?;
        
        // Step 5: Decrypt endpoint
        let endpoint = self.decrypt_endpoint(&record.encrypted_endpoint)?;
        
        // Step 6: Establish connection via introduction points
        let connection = self.connect_via_introduction(&record.introduction_points).await?;
        
        // Step 7: Cache result
        let resolved = ResolvedEndpoint {
            endpoint,
            connection,
            introduction_points: record.introduction_points,
        };
        
        self.cache.insert(address.clone(), resolved.clone()).await;
        
        Ok(resolved)
    }
    
    async fn query_dht_with_privacy(&self, address: &DarkAddress) -> Result<DarkAddressRecord> {
        // Convert address to DHT key
        let key = self.address_to_dht_key(address);
        
        // Perform dummy queries for privacy
        if self.privacy_config.enable_dummy_queries {
            self.send_dummy_queries(3).await;
        }
        
        // Add random delay
        let delay = self.privacy_config.random_delay();
        sleep(delay).await;
        
        // Query with timeout
        match timeout(DHT_QUERY_TIMEOUT, self.dht.get_value(&key)).await {
            Ok(Ok(value)) => {
                // Deserialize record
                let record: DarkAddressRecord = bincode::deserialize(&value)?;
                Ok(record)
            }
            Ok(Err(e)) => Err(DarkAddressError::DHTError(e)),
            Err(_) => Err(DarkAddressError::ResolutionTimeout),
        }
    }
    
    async fn connect_via_introduction(&self, intro_points: &[IntroductionPoint]) -> Result<SecureConnection> {
        // Try introduction points in random order
        let mut points = intro_points.to_vec();
        points.shuffle(&mut thread_rng());
        
        for point in points {
            match self.try_introduction_point(&point).await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    warn!("Introduction point failed: {:?}", e);
                    continue;
                }
            }
        }
        
        Err(DarkAddressError::NoValidIntroductionPoints)
    }
}
```

## 3. Privacy Protection Mechanisms

### 3.1 Dummy Query Generation

```rust
pub struct PrivacyManager {
    /// Dummy query generator
    dummy_generator: DummyQueryGenerator,
    
    /// Query timing randomizer
    timing_randomizer: TimingRandomizer,
    
    /// Cover traffic generator
    cover_traffic: CoverTrafficGenerator,
}

impl PrivacyManager {
    /// Generate and send dummy queries
    pub async fn send_dummy_queries(&self, count: usize) -> Result<()> {
        let futures: Vec<_> = (0..count)
            .map(|_| {
                let address = self.dummy_generator.generate_dummy_address();
                let dht = self.dht.clone();
                
                async move {
                    // Fire and forget - we don't care about results
                    let _ = dht.get_value(&address.to_dht_key()).await;
                }
            })
            .collect();
        
        // Execute dummy queries in parallel
        join_all(futures).await;
        
        Ok(())
    }
    
    /// Apply timing randomization
    pub async fn randomize_timing(&self) {
        let delay = self.timing_randomizer.get_delay();
        sleep(delay).await;
    }
}

pub struct DummyQueryGenerator {
    /// Random number generator
    rng: Mutex<ChaCha20Rng>,
}

impl DummyQueryGenerator {
    /// Generate realistic-looking dummy address
    pub fn generate_dummy_address(&self) -> DarkAddress {
        let mut rng = self.rng.lock().unwrap();
        
        // Generate random commitment that looks real
        let mut commitment = [0u8; 20];
        rng.fill_bytes(&mut commitment);
        
        // Create valid-looking address
        let mut address = DarkAddress {
            version: CURRENT_ADDRESS_VERSION,
            network: NetworkType::Mainnet,
            commitment,
            checksum: [0u8; 4],
        };
        
        // Calculate proper checksum so it passes validation
        address.checksum = address.calculate_checksum();
        
        address
    }
}
```

### 3.2 Introduction Point Protocol

```rust
pub struct IntroductionProtocol {
    /// Our introduction points
    our_intro_points: Arc<RwLock<Vec<IntroductionPoint>>>,
    
    /// Active introduction circuits
    intro_circuits: Arc<DashMap<[u8; 32], IntroductionCircuit>>,
}

impl IntroductionProtocol {
    /// Create new introduction point
    pub async fn create_introduction_point(&self) -> Result<IntroductionPoint> {
        // Select relay for introduction
        let relay = self.select_introduction_relay().await?;
        
        // Build circuit to relay
        let circuit = self.circuit_manager
            .build_circuit_to(relay.peer_id, CircuitType::Introduction)
            .await?;
        
        // Generate authentication token
        let auth_token = self.generate_auth_token();
        
        // Send ESTABLISH_INTRO cell
        let establish_msg = EstablishIntroMessage {
            auth_token,
            public_key: self.identity.public_key(),
            expiration: SystemTime::now() + INTRO_POINT_LIFETIME,
        };
        
        circuit.send_message(establish_msg).await?;
        
        // Wait for confirmation
        let intro_established = circuit.receive_message().await?;
        
        // Create introduction point
        let intro_point = IntroductionPoint {
            relay_pubkey: relay.ml_kem_public_key,
            relay_address: relay.onion_address,
            auth_token,
            expires_at: SystemTime::now() + INTRO_POINT_LIFETIME,
        };
        
        // Store circuit for handling introductions
        self.intro_circuits.insert(auth_token, IntroductionCircuit {
            circuit,
            created_at: Instant::now(),
        });
        
        Ok(intro_point)
    }
    
    /// Handle introduction request
    pub async fn handle_introduction(&self, request: IntroductionRequest) -> Result<()> {
        // Verify authentication
        let circuit = self.intro_circuits
            .get(&request.auth_token)
            .ok_or(IntroductionError::InvalidAuthToken)?;
        
        // Verify proof of work (anti-spam)
        self.verify_proof_of_work(&request.proof_of_work)?;
        
        // Create rendezvous point
        let rendezvous = self.create_rendezvous_point().await?;
        
        // Send rendezvous info to introducer
        let response = IntroductionResponse {
            rendezvous_point: rendezvous.public_info(),
            rendezvous_cookie: rendezvous.cookie,
            ephemeral_key: self.generate_ephemeral_key(),
        };
        
        circuit.send_message(response).await?;
        
        // Wait at rendezvous point
        self.await_rendezvous(rendezvous).await
    }
}
```

## 4. Address Lifecycle Management

### 4.1 Address Rotation

```rust
pub struct AddressRotationPolicy {
    /// Rotation interval
    rotation_interval: Duration,
    
    /// Maximum address age
    max_age: Duration,
    
    /// Usage-based rotation
    usage_threshold: UsageThreshold,
}

pub struct UsageThreshold {
    /// Maximum number of connections
    max_connections: u64,
    
    /// Maximum data transferred
    max_bytes: u64,
    
    /// Maximum time active
    max_active_time: Duration,
}

pub struct AddressManager {
    /// Current active addresses
    active_addresses: Arc<RwLock<HashMap<DarkAddress, AddressMetadata>>>,
    
    /// Address rotation policy
    rotation_policy: AddressRotationPolicy,
    
    /// Address generator
    generator: Arc<DarkAddressGenerator>,
}

impl AddressManager {
    /// Rotation background task
    pub async fn rotation_loop(&self) {
        let mut interval = interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            // Check each active address
            let addresses = self.active_addresses.read().await.clone();
            
            for (address, metadata) in addresses {
                if self.should_rotate(&address, &metadata) {
                    self.rotate_address(&address).await;
                }
            }
        }
    }
    
    fn should_rotate(&self, address: &DarkAddress, metadata: &AddressMetadata) -> bool {
        let age = Instant::now() - metadata.created_at;
        
        // Check rotation conditions
        age > self.rotation_policy.rotation_interval ||
        age > self.rotation_policy.max_age ||
        metadata.connection_count > self.rotation_policy.usage_threshold.max_connections ||
        metadata.bytes_transferred > self.rotation_policy.usage_threshold.max_bytes
    }
    
    async fn rotate_address(&self, old_address: &DarkAddress) {
        info!("Rotating dark address: {}", old_address.to_human_readable());
        
        // Generate new address
        let (new_address, secret) = self.generator.generate();
        
        // Publish new address to DHT
        self.publish_address(&new_address, &secret).await.ok();
        
        // Mark old address for deprecation
        if let Some(mut metadata) = self.active_addresses.write().await.get_mut(old_address) {
            metadata.deprecated_at = Some(Instant::now());
            metadata.replacement = Some(new_address.clone());
        }
        
        // Add new address
        self.active_addresses.write().await.insert(new_address, AddressMetadata {
            created_at: Instant::now(),
            secret,
            connection_count: 0,
            bytes_transferred: 0,
            deprecated_at: None,
            replacement: None,
        });
    }
}
```

### 4.2 Address Revocation

```rust
pub struct RevocationManager {
    /// Revocation list stored in DHT
    revocation_list: Arc<RwLock<HashSet<DarkAddress>>>,
    
    /// Revocation proofs
    revocation_proofs: Arc<DashMap<DarkAddress, RevocationProof>>,
}

#[derive(Serialize, Deserialize)]
pub struct RevocationProof {
    /// Address being revoked
    address: DarkAddress,
    
    /// Reason for revocation
    reason: RevocationReason,
    
    /// Signature by address owner
    signature: MLDSASignature,
    
    /// Timestamp
    timestamp: SystemTime,
}

#[derive(Serialize, Deserialize)]
pub enum RevocationReason {
    /// Key compromise
    KeyCompromise,
    
    /// Voluntary rotation
    Rotation,
    
    /// Address expiration
    Expiration,
    
    /// Security precaution
    SecurityPrecaution,
}

impl RevocationManager {
    /// Revoke an address
    pub async fn revoke_address(&self, address: &DarkAddress, secret: &AddressSecret, reason: RevocationReason) -> Result<()> {
        // Create revocation proof
        let proof = RevocationProof {
            address: address.clone(),
            reason,
            signature: secret.sign_revocation(address, &reason)?,
            timestamp: SystemTime::now(),
        };
        
        // Add to local revocation list
        self.revocation_list.write().await.insert(address.clone());
        self.revocation_proofs.insert(address.clone(), proof.clone());
        
        // Publish revocation to DHT
        self.publish_revocation(proof).await?;
        
        // Remove address record from DHT
        self.remove_address_record(address).await?;
        
        Ok(())
    }
    
    /// Check if address is revoked
    pub async fn is_revoked(&self, address: &DarkAddress) -> bool {
        // Check local cache first
        if self.revocation_list.read().await.contains(address) {
            return true;
        }
        
        // Check DHT for revocation
        if let Ok(proof) = self.check_dht_revocation(address).await {
            // Verify and cache
            if self.verify_revocation_proof(&proof).is_ok() {
                self.revocation_list.write().await.insert(address.clone());
                return true;
            }
        }
        
        false
    }
}
```

## 5. Integration with Network Stack

### 5.1 Network Manager Integration

```rust
impl NetworkManager {
    /// Get or create dark address for communication
    pub async fn get_dark_address(&self) -> Result<DarkAddress> {
        let manager = &self.address_manager;
        
        // Get current active address
        if let Some(address) = manager.get_active_address().await {
            return Ok(address);
        }
        
        // Generate new address
        let (address, secret) = manager.generate_new_address().await?;
        
        // Publish to DHT
        manager.publish_address(&address, &secret).await?;
        
        Ok(address)
    }
    
    /// Connect to peer via dark address
    pub async fn connect_dark(&self, dark_address: &DarkAddress) -> Result<PeerId> {
        // Resolve address
        let resolved = self.dark_resolver.resolve(dark_address).await?;
        
        // Connect via introduction points
        let connection = self.establish_dark_connection(&resolved).await?;
        
        // Get peer ID from connection
        let peer_id = connection.peer_id();
        
        // Store connection
        self.connection_manager.add_connection(peer_id, connection);
        
        Ok(peer_id)
    }
}
```

### 5.2 Onion Routing Integration

```rust
/// Dark addressing with onion routing
pub struct DarkOnionIntegration {
    /// Onion router
    onion_router: Arc<OnionRouter>,
    
    /// Dark address manager
    address_manager: Arc<AddressManager>,
}

impl DarkOnionIntegration {
    /// Send message to dark address via onion routing
    pub async fn send_to_dark_address(&self, message: &[u8], dark_addr: &DarkAddress) -> Result<()> {
        // Resolve to introduction points
        let resolved = self.resolve_dark_address(dark_addr).await?;
        
        // Build circuit to introduction point
        let intro_circuit = self.onion_router
            .build_circuit_to(&resolved.introduction_points[0])
            .await?;
        
        // Send introduction request
        let intro_request = self.create_introduction_request(message);
        intro_circuit.send(intro_request).await?;
        
        // Receive rendezvous info
        let rendezvous_info = intro_circuit.receive().await?;
        
        // Build circuit to rendezvous
        let rendezvous_circuit = self.onion_router
            .build_circuit_to_rendezvous(&rendezvous_info)
            .await?;
        
        // Send actual message
        rendezvous_circuit.send(message).await?;
        
        Ok(())
    }
}
```

## 6. Security Analysis

### 6.1 Threat Model

1. **Address Linkability**: Adversary tries to link dark addresses to real identities
2. **Traffic Analysis**: Adversary monitors DHT queries to infer communication patterns  
3. **Sybil Attacks**: Adversary creates many addresses to flood the system
4. **Quantum Attacks**: Future quantum computers breaking cryptography

### 6.2 Security Properties

1. **Unlinkability**: Different dark addresses cannot be linked to same entity
2. **Forward Secrecy**: Compromise of long-term keys doesn't reveal past communications
3. **Post-Quantum Security**: ML-KEM and ML-DSA provide quantum resistance
4. **Anti-Enumeration**: Cannot enumerate all valid addresses
5. **Revocation**: Compromised addresses can be revoked

### 6.3 Privacy Guarantees

1. **Query Privacy**: DHT lookups don't reveal communication intent
2. **Timing Privacy**: Random delays prevent timing correlation
3. **Traffic Privacy**: Dummy queries hide real lookups
4. **Introduction Privacy**: Introduction points don't learn final destinations

## 7. Performance Optimizations

### 7.1 Caching Strategy

```rust
pub struct ResolutionCache {
    /// LRU cache for resolved addresses
    cache: Arc<Mutex<LruCache<DarkAddress, CachedResolution>>>,
    
    /// Cache configuration
    config: CacheConfig,
}

pub struct CacheConfig {
    /// Maximum cache entries
    max_entries: usize,
    
    /// TTL for cache entries
    ttl: Duration,
    
    /// Negative cache TTL
    negative_ttl: Duration,
}

pub struct CachedResolution {
    /// Resolved endpoint
    endpoint: ResolvedEndpoint,
    
    /// Cache insertion time
    cached_at: Instant,
    
    /// Number of uses
    use_count: u64,
    
    /// Last used time
    last_used: Instant,
}

impl ResolutionCache {
    /// Get cached resolution with freshness check
    pub async fn get(&self, address: &DarkAddress) -> Option<ResolvedEndpoint> {
        let mut cache = self.cache.lock().await;
        
        if let Some(cached) = cache.get_mut(address) {
            // Check TTL
            if cached.cached_at.elapsed() > self.config.ttl {
                cache.pop(address);
                return None;
            }
            
            // Update usage stats
            cached.use_count += 1;
            cached.last_used = Instant::now();
            
            Some(cached.endpoint.clone())
        } else {
            None
        }
    }
}
```

### 7.2 Batch Operations

```rust
pub struct BatchResolver {
    /// Pending resolution requests
    pending: Arc<Mutex<HashMap<DarkAddress, Vec<oneshot::Sender<Result<ResolvedEndpoint>>>>>>,
    
    /// Batch configuration
    batch_config: BatchConfig,
}

impl BatchResolver {
    /// Resolve address with batching
    pub async fn resolve(&self, address: DarkAddress) -> Result<ResolvedEndpoint> {
        // Check if resolution already pending
        let rx = {
            let mut pending = self.pending.lock().await;
            
            if let Some(waiters) = pending.get_mut(&address) {
                // Add to existing waiters
                let (tx, rx) = oneshot::channel();
                waiters.push(tx);
                rx
            } else {
                // Start new resolution
                let (tx, rx) = oneshot::channel();
                pending.insert(address.clone(), vec![tx]);
                
                // Spawn resolution task
                self.spawn_resolution(address);
                
                rx
            }
        };
        
        // Wait for resolution
        rx.await?
    }
}
```

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_address_generation() {
        let generator = DarkAddressGenerator::new_test();
        let (addr1, _) = generator.generate();
        let (addr2, _) = generator.generate();
        
        // Addresses should be different
        assert_ne!(addr1, addr2);
        
        // Checksums should be valid
        assert!(addr1.verify_checksum());
        assert!(addr2.verify_checksum());
    }
    
    #[tokio::test]
    async fn test_address_resolution() {
        let resolver = DarkResolver::new_test();
        let (address, secret) = generate_test_address();
        
        // Publish address
        publish_test_address(&address, &secret).await;
        
        // Resolve should succeed
        let resolved = resolver.resolve(&address).await.unwrap();
        assert_eq!(resolved.endpoint.peer_id, secret.peer_id);
    }
    
    #[tokio::test] 
    async fn test_privacy_measures() {
        let resolver = DarkResolver::new_test();
        let monitor = NetworkMonitor::new();
        
        // Resolve with monitoring
        let _ = resolver.resolve(&test_address()).await;
        
        // Should see dummy queries
        let queries = monitor.get_dht_queries();
        assert!(queries.len() > 1);
        
        // Should have timing randomization
        let intervals = monitor.get_query_intervals();
        assert!(intervals.iter().any(|&i| i > Duration::from_millis(100)));
    }
}
```

### 8.2 Integration Tests

```rust
#[tokio::test]
async fn test_full_dark_communication() {
    // Create two nodes
    let node1 = TestNode::new().await;
    let node2 = TestNode::new().await;
    
    // Node 1 generates dark address
    let dark_addr = node1.generate_dark_address().await;
    
    // Node 2 connects via dark address
    let peer_id = node2.connect_dark(&dark_addr).await.unwrap();
    
    // Send message
    let message = b"Hello via dark address";
    node2.send_message(peer_id, message).await.unwrap();
    
    // Node 1 receives message
    let received = node1.receive_message().await.unwrap();
    assert_eq!(received.data, message);
}
```

## Conclusion

The Dark Addressing System provides strong privacy guarantees through:
- Unlinkable addresses with cryptographic commitments
- Integration with onion routing for anonymous connections
- Post-quantum security with ML-KEM/ML-DSA
- Multiple privacy protection mechanisms
- Efficient caching and batch resolution

This design ensures that QuDAG nodes can communicate privately without revealing network-level identities or locations.