use qudag_crypto::kem::{PublicKey as KEMPublicKey, SecretKey as KEMSecretKey};
use qudag_crypto::ml_kem::MlKem768;
use rand::{thread_rng, Rng, RngCore};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::Mutex as TokioMutex;

/// Error types for onion routing operations
#[derive(Error, Debug)]
pub enum OnionError {
    /// Layer encryption failed
    #[error("layer encryption failed: {0}")]
    EncryptionError(String),

    /// Layer decryption failed
    #[error("layer decryption failed: {0}")]
    DecryptionError(String),

    /// Invalid layer format
    #[error("invalid layer format: {0}")]
    InvalidFormat(String),

    /// ML-KEM operation failed
    #[error("ML-KEM error: {0}")]
    MLKEMError(String),

    /// Random number generation failed
    #[error("RNG error: {0}")]
    RngError(String),

    /// Route construction failed
    #[error("route construction failed: {0}")]
    RouteError(String),

    /// Timing constraint violation
    #[error("timing constraint violated: {0}")]
    TimingError(String),
}

/// Onion routing layer containing encrypted next hop information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionLayer {
    /// Encrypted next hop public key
    pub next_hop: Vec<u8>,
    /// Encrypted payload for next hop
    pub payload: Vec<u8>,
    /// Encrypted routing metadata
    pub metadata: Vec<u8>,
    /// ML-KEM ciphertext for key encapsulation
    pub kem_ciphertext: Vec<u8>,
    /// Nonce for AEAD encryption
    pub nonce: [u8; 12],
    /// Authentication tag
    pub auth_tag: Vec<u8>,
    /// Layer creation timestamp for timing analysis resistance
    pub timestamp: u64,
    /// Dummy padding for size normalization
    pub padding: Vec<u8>,
}

impl OnionLayer {
    /// Creates a new onion layer with quantum-resistant encryption
    pub fn new(next_hop: Vec<u8>, payload: Vec<u8>, metadata: Vec<u8>) -> Self {
        let rng = SystemRandom::new();
        let mut nonce = [0u8; 12];
        rng.fill(&mut nonce).expect("RNG failure");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Add random padding to normalize layer sizes (defense against traffic analysis)
        let mut padding = vec![0u8; thread_rng().next_u32() as usize % 256];
        thread_rng().fill_bytes(&mut padding);

        Self {
            next_hop,
            payload,
            metadata,
            kem_ciphertext: Vec::new(),
            nonce,
            auth_tag: Vec::new(),
            timestamp,
            padding,
        }
    }

    /// Validates layer format and timing constraints
    pub fn validate(&self) -> Result<(), OnionError> {
        if self.next_hop.is_empty() {
            return Err(OnionError::InvalidFormat("empty next hop key".into()));
        }
        if self.payload.is_empty() {
            return Err(OnionError::InvalidFormat("empty payload".into()));
        }
        if self.kem_ciphertext.is_empty() {
            return Err(OnionError::InvalidFormat("missing KEM ciphertext".into()));
        }

        // Check timing constraints (prevent replay attacks)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if now.saturating_sub(self.timestamp) > 300_000 {
            // 5 minute window
            return Err(OnionError::TimingError("layer too old".into()));
        }

        Ok(())
    }

    /// Get total layer size including padding (for traffic analysis resistance)
    pub fn total_size(&self) -> usize {
        self.next_hop.len()
            + self.payload.len()
            + self.metadata.len()
            + self.kem_ciphertext.len()
            + self.auth_tag.len()
            + self.padding.len()
            + 12
            + 8 // nonce + timestamp
    }

    /// Normalize layer size to standard size (anti-traffic analysis)
    pub fn normalize_size(&mut self, target_size: usize) {
        let current_size = self.total_size();
        if current_size < target_size {
            let padding_needed = target_size - current_size;
            let mut additional_padding = vec![0u8; padding_needed];
            thread_rng().fill_bytes(&mut additional_padding);
            self.padding.extend(additional_padding);
        }
    }
}

/// Onion router interface for handling layered encryption/decryption
pub trait OnionRouter: Send + Sync {
    /// Encrypts a message with multiple onion layers
    fn encrypt_layers(
        &self,
        message: Vec<u8>,
        route: Vec<Vec<u8>>,
    ) -> Result<Vec<OnionLayer>, OnionError>;

    /// Decrypts the outer layer of an onion-routed message
    fn decrypt_layer(&self, layer: OnionLayer)
        -> Result<(Vec<u8>, Option<OnionLayer>), OnionError>;

    /// Creates routing metadata for a layer
    fn create_metadata(&self, route_info: Vec<u8>) -> Result<Vec<u8>, OnionError>;
}

/// Implementation of ML-KEM-based onion routing with quantum resistance
pub struct MLKEMOnionRouter {
    /// Node's ML-KEM secret key for decryption
    ml_kem_secret_key: KEMSecretKey,
    /// Node's ML-KEM public key
    ml_kem_public_key: KEMPublicKey,
    /// Random number generator
    rng: SystemRandom,
    /// Standard layer size for traffic analysis resistance
    standard_layer_size: usize,
    /// Circuit management
    #[allow(dead_code)]
    circuit_manager: Arc<TokioMutex<CircuitManager>>,
    /// Directory authority client
    directory_client: Arc<DirectoryClient>,
}

impl MLKEMOnionRouter {
    /// Creates a new ML-KEM onion router
    pub async fn new() -> Result<Self, OnionError> {
        // Generate ML-KEM keypair for this node
        let (public_key, secret_key) = MlKem768::keygen()
            .map_err(|e| OnionError::MLKEMError(format!("Key generation failed: {:?}", e)))?;

        let circuit_manager = Arc::new(TokioMutex::new(CircuitManager::new()));
        let directory_client = Arc::new(DirectoryClient::new());

        Ok(Self {
            ml_kem_secret_key: secret_key,
            ml_kem_public_key: public_key,
            rng: SystemRandom::new(),
            standard_layer_size: 4096, // 4KB standard layer size
            circuit_manager,
            directory_client,
        })
    }

    /// Creates a new ML-KEM onion router with custom layer size
    pub async fn with_layer_size(layer_size: usize) -> Result<Self, OnionError> {
        let mut router = Self::new().await?;
        router.standard_layer_size = layer_size;
        Ok(router)
    }

    /// Get the node's ML-KEM public key
    pub fn get_public_key(&self) -> &KEMPublicKey {
        &self.ml_kem_public_key
    }

    /// Derive symmetric key from ML-KEM shared secret using KDF
    fn derive_symmetric_key(&self, shared_secret: &[u8]) -> Result<[u8; 32], OnionError> {
        use ring::hkdf;

        let salt = ring::hkdf::Salt::new(hkdf::HKDF_SHA256, b"QuDAG-Onion-v1");
        let prk = salt.extract(shared_secret);

        let mut key = [0u8; 32];
        let info = [&b"symmetric-key"[..]];
        prk.expand(&info[..], hkdf::HKDF_SHA256)
            .map_err(|_| OnionError::EncryptionError("Key derivation failed".into()))?
            .fill(&mut key)
            .map_err(|_| OnionError::EncryptionError("Key derivation failed".into()))?;

        Ok(key)
    }

    /// Encrypt data with ChaCha20-Poly1305
    fn encrypt_aead(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        data: &[u8],
    ) -> Result<Vec<u8>, OnionError> {
        let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, key)
            .map_err(|e| OnionError::EncryptionError(e.to_string()))?;
        let sealing_key = LessSafeKey::new(unbound_key);

        let mut encrypted_data = data.to_vec();
        sealing_key
            .seal_in_place_append_tag(
                Nonce::assume_unique_for_key(*nonce),
                Aad::empty(),
                &mut encrypted_data,
            )
            .map_err(|e| OnionError::EncryptionError(e.to_string()))?;

        Ok(encrypted_data)
    }

    /// Decrypt data with ChaCha20-Poly1305
    fn decrypt_aead(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        encrypted_data: &mut [u8],
    ) -> Result<Vec<u8>, OnionError> {
        let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, key)
            .map_err(|e| OnionError::DecryptionError(e.to_string()))?;
        let opening_key = LessSafeKey::new(unbound_key);

        let decrypted = opening_key
            .open_in_place(
                Nonce::assume_unique_for_key(*nonce),
                Aad::empty(),
                encrypted_data,
            )
            .map_err(|e| OnionError::DecryptionError(e.to_string()))?;

        Ok(decrypted.to_vec())
    }

    /// Add timing obfuscation delay
    #[allow(dead_code)]
    async fn add_timing_obfuscation(&self) {
        // Random delay between 10-100ms to prevent timing analysis
        let delay_ms = (thread_rng().next_u32() % 90) + 10;
        tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;
    }
}

impl MLKEMOnionRouter {
    /// Encrypt layers for onion routing (async version)
    pub async fn encrypt_layers(
        &self,
        message: Vec<u8>,
        route: Vec<Vec<u8>>,
    ) -> Result<Vec<OnionLayer>, OnionError> {
        if route.is_empty() {
            return Err(OnionError::RouteError("empty route".into()));
        }

        let mut layers = Vec::new();
        let mut current_payload = message;

        // Build layers from innermost to outermost (reverse order)
        for (i, _hop_pubkey) in route.iter().rev().enumerate() {
            // Create nonce for this layer
            let mut nonce = [0u8; 12];
            self.rng
                .fill(&mut nonce)
                .map_err(|e| OnionError::RngError(e.to_string()))?;

            // Get public key for this hop
            let hop_public_key = self
                .directory_client
                .get_public_key(&route[route.len() - i - 1])
                .await
                .map_err(|e| OnionError::RouteError(format!("Failed to get public key: {}", e)))?;

            // ML-KEM encapsulation for this layer
            let (kem_ciphertext, shared_secret) = MlKem768::encapsulate(&hop_public_key)
                .map_err(|e| OnionError::MLKEMError(format!("Encapsulation failed: {:?}", e)))?;

            // Derive symmetric key from shared secret
            let symmetric_key = self.derive_symmetric_key(shared_secret.as_bytes())?;

            // Create routing metadata with circuit ID and hop info
            let circuit_id = thread_rng().next_u64();
            let hop_info = HopMetadata {
                circuit_id,
                hop_number: i as u8,
                next_hop: if i == 0 {
                    None
                } else {
                    Some(route[route.len() - i].clone())
                },
                flags: LayerFlags::default(),
            };
            let metadata = self.create_metadata(
                bincode::serialize(&hop_info)
                    .map_err(|e| OnionError::InvalidFormat(e.to_string()))?,
            )?;

            // Determine next hop (empty for last layer)
            let next_hop = if i == 0 {
                Vec::new() // Final destination
            } else {
                route[route.len() - i].clone()
            };

            // Create layer
            let mut layer = OnionLayer::new(next_hop, current_payload.clone(), metadata);
            layer.kem_ciphertext = kem_ciphertext.as_bytes().to_vec();
            layer.nonce = nonce;

            // Encrypt the layer payload
            let encrypted_payload = self.encrypt_aead(&symmetric_key, &nonce, &current_payload)?;
            layer.payload = encrypted_payload;

            // Normalize layer size for traffic analysis resistance
            layer.normalize_size(self.standard_layer_size);

            // Validate layer
            layer.validate()?;

            // For next iteration, current_payload becomes the serialized current layer
            current_payload = bincode::serialize(&layer)
                .map_err(|e| OnionError::EncryptionError(e.to_string()))?;

            layers.push(layer);
        }

        // Reverse to get correct order (outermost first)
        layers.reverse();
        Ok(layers)
    }

    /// Decrypt an onion layer
    pub fn decrypt_layer(
        &self,
        layer: OnionLayer,
    ) -> Result<(Vec<u8>, Option<OnionLayer>), OnionError> {
        // Validate layer before processing
        layer.validate()?;

        // ML-KEM decapsulation using the node's secret key
        let kem_ciphertext = qudag_crypto::kem::Ciphertext::from_bytes(&layer.kem_ciphertext)
            .map_err(|_| OnionError::MLKEMError("Invalid KEM ciphertext".into()))?;

        let shared_secret = MlKem768::decapsulate(&self.ml_kem_secret_key, &kem_ciphertext)
            .map_err(|e| OnionError::MLKEMError(format!("Decapsulation failed: {:?}", e)))?;

        // Derive symmetric key from shared secret
        let symmetric_key = self.derive_symmetric_key(shared_secret.as_bytes())?;

        // Decrypt the payload using the derived symmetric key
        let mut encrypted_payload = layer.payload.clone();
        let decrypted_payload =
            self.decrypt_aead(&symmetric_key, &layer.nonce, &mut encrypted_payload)?;

        // Try to deserialize as next layer (if this isn't the final layer)
        if !layer.next_hop.is_empty() {
            match bincode::deserialize::<OnionLayer>(&decrypted_payload) {
                Ok(next_layer) => Ok((decrypted_payload, Some(next_layer))),
                Err(_) => {
                    // Not a layer, must be final payload
                    Ok((decrypted_payload, None))
                }
            }
        } else {
            // Final layer - return the original message
            Ok((decrypted_payload, None))
        }
    }

    fn create_metadata(&self, route_info: Vec<u8>) -> Result<Vec<u8>, OnionError> {
        // Create metadata with timing information and flags
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut metadata = Vec::new();
        metadata.extend_from_slice(&timestamp.to_le_bytes());
        metadata.extend_from_slice(&route_info);

        // Add random padding to metadata for traffic analysis resistance
        let mut padding = vec![0u8; thread_rng().next_u32() as usize % 128];
        thread_rng().fill_bytes(&mut padding);
        metadata.extend(padding);

        Ok(metadata)
    }
}

impl OnionRouter for MLKEMOnionRouter {
    fn encrypt_layers(
        &self,
        message: Vec<u8>,
        route: Vec<Vec<u8>>,
    ) -> Result<Vec<OnionLayer>, OnionError> {
        // Block on async version
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.encrypt_layers(message, route))
        })
    }

    fn decrypt_layer(
        &self,
        layer: OnionLayer,
    ) -> Result<(Vec<u8>, Option<OnionLayer>), OnionError> {
        self.decrypt_layer(layer)
    }

    fn create_metadata(&self, route_info: Vec<u8>) -> Result<Vec<u8>, OnionError> {
        self.create_metadata(route_info)
    }
}

/// Mix network node for batch processing and traffic shaping
#[derive(Debug)]
pub struct MixNode {
    /// Node identifier
    pub id: Vec<u8>,
    /// Batch configuration
    config: MixConfig,
    /// Message buffer for batching
    message_buffer: Vec<MixMessage>,
    /// Last flush time
    last_flush: SystemTime,
    /// Dummy traffic generator
    dummy_generator: DummyTrafficGenerator,
    /// Traffic shaper
    traffic_shaper: TrafficShaper,
}

/// Configuration for mix node behavior
#[derive(Debug, Clone)]
pub struct MixConfig {
    /// Batch size for message processing
    pub batch_size: usize,
    /// Maximum batch wait time
    pub batch_timeout: Duration,
    /// Target output rate (messages per second)
    pub target_rate: f64,
    /// Dummy traffic probability (0.0 to 1.0)
    pub dummy_probability: f64,
    /// Enable timing obfuscation
    pub timing_obfuscation: bool,
}

impl Default for MixConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout: Duration::from_millis(500),
            target_rate: 50.0,      // 50 messages per second
            dummy_probability: 0.1, // 10% dummy traffic
            timing_obfuscation: true,
        }
    }
}

/// Message in the mix network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixMessage {
    /// Message content (encrypted)
    pub content: Vec<u8>,
    /// Message priority
    pub priority: u8,
    /// Creation timestamp
    pub timestamp: u64,
    /// Message type indicator
    pub message_type: MixMessageType,
    /// Normalized size for traffic analysis resistance
    pub normalized_size: usize,
}

/// Type of message in mix network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MixMessageType {
    /// Real message with payload
    Real,
    /// Dummy message for traffic analysis resistance
    Dummy,
    /// Heartbeat message
    Heartbeat,
}

impl MixNode {
    /// Create a new mix node with default configuration
    pub fn new(id: Vec<u8>) -> Self {
        Self::with_config(id, MixConfig::default())
    }

    /// Create a new mix node with custom configuration
    pub fn with_config(id: Vec<u8>, config: MixConfig) -> Self {
        Self {
            id,
            config: config.clone(),
            message_buffer: Vec::with_capacity(config.batch_size * 2),
            last_flush: SystemTime::now(),
            dummy_generator: DummyTrafficGenerator::new(config.dummy_probability),
            traffic_shaper: TrafficShaper::new(config.target_rate),
        }
    }

    /// Add a message to the mix node buffer
    pub async fn add_message(&mut self, mut message: MixMessage) -> Result<(), OnionError> {
        // Normalize message size for traffic analysis resistance
        message.normalized_size = self.normalize_message_size(&message);

        self.message_buffer.push(message);

        // Check if we should flush the batch
        if self.should_flush() {
            self.flush_batch().await?;
        }

        Ok(())
    }

    /// Check if batch should be flushed
    fn should_flush(&self) -> bool {
        self.message_buffer.len() >= self.config.batch_size
            || self.last_flush.elapsed().unwrap_or(Duration::ZERO) >= self.config.batch_timeout
    }

    /// Flush the current batch of messages
    pub async fn flush_batch(&mut self) -> Result<Vec<MixMessage>, OnionError> {
        if self.message_buffer.is_empty() {
            return Ok(Vec::new());
        }

        // Add dummy messages to fill batch if needed
        self.add_dummy_messages();

        // Shuffle messages for anonymity
        use rand::seq::SliceRandom;
        self.message_buffer.shuffle(&mut thread_rng());

        // Apply traffic shaping delay
        self.traffic_shaper.apply_shaping().await;

        // Apply timing obfuscation if enabled
        if self.config.timing_obfuscation {
            self.apply_timing_obfuscation().await;
        }

        // Flush the batch
        let batch = std::mem::take(&mut self.message_buffer);
        self.last_flush = SystemTime::now();

        Ok(batch)
    }

    /// Add dummy messages to maintain consistent batch sizes
    fn add_dummy_messages(&mut self) {
        while self.message_buffer.len() < self.config.batch_size {
            if let Some(dummy_msg) = self.dummy_generator.generate_dummy() {
                self.message_buffer.push(dummy_msg);
            } else {
                break; // Don't add more dummies if generator says no
            }
        }
    }

    /// Normalize message size for traffic analysis resistance
    fn normalize_message_size(&self, message: &MixMessage) -> usize {
        // Standard sizes: 512B, 1KB, 2KB, 4KB, 8KB
        let standard_sizes = [512, 1024, 2048, 4096, 8192];
        let content_size = message.content.len();

        // Find the smallest standard size that fits the content
        for &size in &standard_sizes {
            if content_size <= size {
                return size;
            }
        }

        // If content is larger than largest standard size, round up to next KB
        content_size.div_ceil(1024) * 1024
    }

    /// Apply timing obfuscation
    async fn apply_timing_obfuscation(&self) {
        // Random delay between 50-150ms
        let delay_ms = (thread_rng().next_u32() % 100) + 50;
        tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;
    }

    /// Get current buffer statistics
    pub fn get_stats(&self) -> MixNodeStats {
        MixNodeStats {
            buffer_size: self.message_buffer.len(),
            last_flush_elapsed: self.last_flush.elapsed().unwrap_or(Duration::ZERO),
            dummy_ratio: self.dummy_generator.get_dummy_ratio(),
            target_rate: self.config.target_rate,
        }
    }
}

/// Mix node statistics
#[derive(Debug, Clone)]
pub struct MixNodeStats {
    /// Current buffer size
    pub buffer_size: usize,
    /// Time since last flush
    pub last_flush_elapsed: Duration,
    /// Ratio of dummy messages (0.0 to 1.0)
    pub dummy_ratio: f64,
    /// Target output rate
    pub target_rate: f64,
}

/// Dummy traffic generator for anonymity
#[derive(Debug)]
struct DummyTrafficGenerator {
    /// Probability of generating dummy traffic
    probability: f64,
    /// Counter for statistics
    dummy_count: usize,
    /// Total message count
    total_count: usize,
}

impl DummyTrafficGenerator {
    fn new(probability: f64) -> Self {
        Self {
            probability: probability.clamp(0.0, 1.0),
            dummy_count: 0,
            total_count: 0,
        }
    }

    /// Generate a dummy message if probability allows
    fn generate_dummy(&mut self) -> Option<MixMessage> {
        self.total_count += 1;

        if thread_rng().gen::<f64>() < self.probability {
            self.dummy_count += 1;

            // Generate dummy content of random size
            let size = (thread_rng().next_u32() % 4096) + 256; // 256B to 4KB
            let mut content = vec![0u8; size as usize];
            thread_rng().fill_bytes(&mut content);

            Some(MixMessage {
                content,
                priority: 0,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                message_type: MixMessageType::Dummy,
                normalized_size: 0, // Will be set by normalize_message_size
            })
        } else {
            None
        }
    }

    /// Get current dummy message ratio
    fn get_dummy_ratio(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.dummy_count as f64 / self.total_count as f64
        }
    }
}

/// Traffic shaper for maintaining consistent output rates
#[derive(Debug)]
#[allow(dead_code)]
struct TrafficShaper {
    /// Target rate in messages per second
    target_rate: f64,
    /// Last message time
    last_message_time: SystemTime,
    /// Message interval
    message_interval: Duration,
}

impl TrafficShaper {
    fn new(target_rate: f64) -> Self {
        let message_interval = Duration::from_secs_f64(1.0 / target_rate.max(0.1));

        Self {
            target_rate,
            last_message_time: SystemTime::now(),
            message_interval,
        }
    }

    /// Apply traffic shaping delay
    async fn apply_shaping(&mut self) {
        let now = SystemTime::now();
        let elapsed = now
            .duration_since(self.last_message_time)
            .unwrap_or(Duration::ZERO);

        if elapsed < self.message_interval {
            let delay = self.message_interval - elapsed;
            tokio::time::sleep(delay).await;
        }

        self.last_message_time = SystemTime::now();
    }
}

/// Metadata protection and anonymization utilities
pub struct MetadataProtector {
    /// Random number generator for obfuscation
    rng: SystemRandom,
    /// Configuration for metadata protection
    config: MetadataConfig,
}

/// Configuration for metadata protection
#[derive(Debug, Clone)]
pub struct MetadataConfig {
    /// Enable IP address anonymization
    pub anonymize_ip: bool,
    /// Enable timestamp obfuscation
    pub obfuscate_timing: bool,
    /// Enable packet size normalization
    pub normalize_size: bool,
    /// Enable header randomization
    pub randomize_headers: bool,
    /// Timing bucket size in milliseconds
    pub timing_bucket_ms: u64,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            anonymize_ip: true,
            obfuscate_timing: true,
            normalize_size: true,
            randomize_headers: true,
            timing_bucket_ms: 100, // 100ms buckets
        }
    }
}

/// Protected metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedMetadata {
    /// Obfuscated timestamp (rounded to bucket)
    pub obfuscated_timestamp: u64,
    /// Randomized header fields
    pub random_headers: Vec<(String, Vec<u8>)>,
    /// Normalized packet size
    pub normalized_size: usize,
    /// Anonymous routing identifiers
    pub anonymous_ids: Vec<Vec<u8>>,
    /// Padding for size normalization
    pub padding: Vec<u8>,
}

impl Default for MetadataProtector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataProtector {
    /// Create a new metadata protector with default configuration
    pub fn new() -> Self {
        Self::with_config(MetadataConfig::default())
    }

    /// Create a new metadata protector with custom configuration
    pub fn with_config(config: MetadataConfig) -> Self {
        Self {
            rng: SystemRandom::new(),
            config,
        }
    }

    /// Protect metadata for a message
    pub fn protect_metadata(
        &self,
        original_metadata: &[u8],
    ) -> Result<ProtectedMetadata, OnionError> {
        let timestamp = if self.config.obfuscate_timing {
            self.obfuscate_timestamp()?
        } else {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        };

        let random_headers = if self.config.randomize_headers {
            self.generate_random_headers()?
        } else {
            Vec::new()
        };

        let normalized_size = if self.config.normalize_size {
            self.normalize_packet_size(original_metadata.len())
        } else {
            original_metadata.len()
        };

        let anonymous_ids = self.generate_anonymous_ids()?;
        let padding = self.generate_padding(normalized_size, original_metadata.len())?;

        Ok(ProtectedMetadata {
            obfuscated_timestamp: timestamp,
            random_headers,
            normalized_size,
            anonymous_ids,
            padding,
        })
    }

    /// Obfuscate timestamp by rounding to nearest bucket
    fn obfuscate_timestamp(&self) -> Result<u64, OnionError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Round to nearest bucket
        let bucket_size = self.config.timing_bucket_ms;
        let obfuscated = (now / bucket_size) * bucket_size;

        // Add random jitter within the bucket
        let mut jitter_bytes = [0u8; 8];
        self.rng
            .fill(&mut jitter_bytes)
            .map_err(|e| OnionError::RngError(e.to_string()))?;
        let jitter = u64::from_le_bytes(jitter_bytes) % bucket_size;

        Ok(obfuscated + jitter)
    }

    /// Generate random headers for metadata obfuscation
    fn generate_random_headers(&self) -> Result<Vec<(String, Vec<u8>)>, OnionError> {
        let header_names = [
            "X-Request-ID",
            "X-Correlation-ID",
            "X-Session-ID",
            "X-Trace-ID",
            "X-Custom-Header",
            "X-Client-ID",
        ];

        let mut headers = Vec::new();
        let num_headers = (thread_rng().next_u32() % 4) + 2; // 2-5 headers

        for _ in 0..num_headers {
            let name = header_names[thread_rng().next_u32() as usize % header_names.len()];
            let mut value = vec![0u8; 16];
            self.rng
                .fill(&mut value)
                .map_err(|e| OnionError::RngError(e.to_string()))?;
            headers.push((name.to_string(), value));
        }

        Ok(headers)
    }

    /// Normalize packet size to standard sizes
    fn normalize_packet_size(&self, original_size: usize) -> usize {
        // Standard packet sizes for traffic analysis resistance
        let standard_sizes = [512, 1024, 1536, 2048, 3072, 4096, 6144, 8192, 12288, 16384];

        // Find the smallest standard size that fits the original
        for &size in &standard_sizes {
            if original_size <= size {
                return size;
            }
        }

        // If larger than largest standard size, round up to next 4KB
        original_size.div_ceil(4096) * 4096
    }

    /// Generate anonymous routing identifiers
    fn generate_anonymous_ids(&self) -> Result<Vec<Vec<u8>>, OnionError> {
        let mut ids = Vec::new();
        let num_ids = (thread_rng().next_u32() % 3) + 1; // 1-3 IDs

        for _ in 0..num_ids {
            let mut id = vec![0u8; 32]; // 256-bit anonymous ID
            self.rng
                .fill(&mut id)
                .map_err(|e| OnionError::RngError(e.to_string()))?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// Generate padding to reach normalized size
    fn generate_padding(
        &self,
        target_size: usize,
        current_size: usize,
    ) -> Result<Vec<u8>, OnionError> {
        if target_size <= current_size {
            return Ok(Vec::new());
        }

        let padding_size = target_size - current_size;
        let mut padding = vec![0u8; padding_size];
        self.rng
            .fill(&mut padding)
            .map_err(|e| OnionError::RngError(e.to_string()))?;

        Ok(padding)
    }

    /// Anonymize IP addresses using proxy pools
    pub fn anonymize_ip(&self, original_ip: &str) -> Result<String, OnionError> {
        if !self.config.anonymize_ip {
            return Ok(original_ip.to_string());
        }

        // In a real implementation, this would use a pool of proxy IPs
        // For now, we'll generate a plausible looking IP
        let mut ip_bytes = [0u8; 4];
        self.rng
            .fill(&mut ip_bytes)
            .map_err(|e| OnionError::RngError(e.to_string()))?;

        // Ensure it's a private IP range for testing
        ip_bytes[0] = 10; // Use 10.x.x.x range

        Ok(format!(
            "{}.{}.{}.{}",
            ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]
        ))
    }

    /// Remove identifying information from packets
    pub fn scrub_packet_headers(&self, packet: &mut Vec<u8>) -> Result<(), OnionError> {
        // In a real implementation, this would:
        // 1. Remove or randomize TCP/IP headers that could identify the source
        // 2. Normalize packet timing
        // 3. Remove application-specific identifiers
        // 4. Add cover traffic patterns

        // For now, we'll add some random bytes at the beginning as dummy headers
        let mut dummy_headers = vec![0u8; 20]; // 20 byte dummy header
        self.rng
            .fill(&mut dummy_headers)
            .map_err(|e| OnionError::RngError(e.to_string()))?;

        // Prepend dummy headers
        let mut new_packet = dummy_headers;
        new_packet.extend_from_slice(packet);
        *packet = new_packet;

        Ok(())
    }
}

/// Circuit management for onion routing
#[derive(Debug)]
pub struct CircuitManager {
    /// Active circuits
    circuits: HashMap<u64, Circuit>,
    /// Circuit creation rate limiter
    creation_rate: f64,
    /// Last circuit creation time
    last_creation: Instant,
    /// Maximum concurrent circuits
    max_circuits: usize,
    /// Circuit lifetime
    circuit_lifetime: Duration,
    /// Circuit rotation interval
    rotation_interval: Duration,
}

/// Onion routing circuit
#[derive(Debug, Clone)]
pub struct Circuit {
    /// Circuit ID
    pub id: u64,
    /// Circuit hops (node public keys)
    pub hops: Vec<Vec<u8>>,
    /// Circuit state
    pub state: CircuitState,
    /// Creation timestamp
    pub created_at: Instant,
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Circuit bandwidth usage
    pub bandwidth_used: u64,
    /// Circuit quality score
    pub quality_score: f64,
}

/// Circuit state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is being built
    Building,
    /// Circuit is active and ready
    Active,
    /// Circuit is being torn down
    TearingDown,
    /// Circuit is closed
    Closed,
    /// Circuit failed
    Failed(String),
}

impl CircuitManager {
    /// Create new circuit manager
    pub fn new() -> Self {
        Self {
            circuits: HashMap::new(),
            creation_rate: 1.0, // 1 circuit per second max
            last_creation: Instant::now(),
            max_circuits: 100,
            circuit_lifetime: Duration::from_secs(600), // 10 minutes
            rotation_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Build a new circuit with specified number of hops
    pub async fn build_circuit(
        &mut self,
        hops: usize,
        directory: &DirectoryClient,
    ) -> Result<u64, OnionError> {
        // Rate limiting
        let elapsed = self.last_creation.elapsed().as_secs_f64();
        if elapsed < 1.0 / self.creation_rate {
            return Err(OnionError::RouteError(
                "Circuit creation rate limit exceeded".into(),
            ));
        }

        // Check circuit limit
        if self.circuits.len() >= self.max_circuits {
            // Remove oldest inactive circuit
            self.cleanup_inactive_circuits();
            if self.circuits.len() >= self.max_circuits {
                return Err(OnionError::RouteError("Maximum circuits reached".into()));
            }
        }

        // Select random nodes for the circuit
        let nodes = directory.select_random_nodes(hops).await?;

        // Create circuit
        let circuit_id = thread_rng().next_u64();
        let circuit = Circuit {
            id: circuit_id,
            hops: nodes,
            state: CircuitState::Building,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            bandwidth_used: 0,
            quality_score: 1.0,
        };

        self.circuits.insert(circuit_id, circuit);
        self.last_creation = Instant::now();

        Ok(circuit_id)
    }

    /// Activate a circuit after successful building
    pub fn activate_circuit(&mut self, circuit_id: u64) -> Result<(), OnionError> {
        let circuit = self
            .circuits
            .get_mut(&circuit_id)
            .ok_or_else(|| OnionError::RouteError("Circuit not found".into()))?;

        if circuit.state != CircuitState::Building {
            return Err(OnionError::RouteError(
                "Circuit not in building state".into(),
            ));
        }

        circuit.state = CircuitState::Active;
        circuit.last_activity = Instant::now();
        Ok(())
    }

    /// Tear down a circuit
    pub async fn teardown_circuit(&mut self, circuit_id: u64) -> Result<(), OnionError> {
        let circuit = self
            .circuits
            .get_mut(&circuit_id)
            .ok_or_else(|| OnionError::RouteError("Circuit not found".into()))?;

        circuit.state = CircuitState::TearingDown;

        // In a real implementation, this would send teardown messages to all hops
        // For now, just mark as closed
        circuit.state = CircuitState::Closed;

        Ok(())
    }

    /// Get an active circuit for use
    pub fn get_active_circuit(&mut self) -> Option<&mut Circuit> {
        // Find best active circuit based on quality score and age
        self.circuits
            .values_mut()
            .filter(|c| c.state == CircuitState::Active)
            .filter(|c| c.created_at.elapsed() < self.circuit_lifetime)
            .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
    }

    /// Update circuit metrics
    pub fn update_circuit_metrics(&mut self, circuit_id: u64, bytes_sent: u64, success: bool) {
        if let Some(circuit) = self.circuits.get_mut(&circuit_id) {
            circuit.last_activity = Instant::now();
            circuit.bandwidth_used += bytes_sent;

            // Update quality score based on success rate
            if success {
                circuit.quality_score = (circuit.quality_score * 0.95 + 1.0 * 0.05).min(1.0);
            } else {
                circuit.quality_score = (circuit.quality_score * 0.95).max(0.0);
            }
        }
    }

    /// Check if circuits need rotation
    pub fn needs_rotation(&self) -> bool {
        self.circuits
            .values()
            .filter(|c| c.state == CircuitState::Active)
            .any(|c| c.created_at.elapsed() >= self.rotation_interval)
    }

    /// Cleanup inactive or expired circuits
    pub fn cleanup_inactive_circuits(&mut self) {
        let _now = Instant::now();
        self.circuits.retain(|_, circuit| match circuit.state {
            CircuitState::Closed | CircuitState::Failed(_) => false,
            _ => circuit.created_at.elapsed() < self.circuit_lifetime * 2,
        });
    }

    /// Get circuit statistics
    pub fn get_stats(&self) -> CircuitStats {
        let active_count = self
            .circuits
            .values()
            .filter(|c| c.state == CircuitState::Active)
            .count();

        let total_bandwidth = self.circuits.values().map(|c| c.bandwidth_used).sum();

        let avg_quality = if active_count > 0 {
            self.circuits
                .values()
                .filter(|c| c.state == CircuitState::Active)
                .map(|c| c.quality_score)
                .sum::<f64>()
                / active_count as f64
        } else {
            0.0
        };

        CircuitStats {
            total_circuits: self.circuits.len(),
            active_circuits: active_count,
            total_bandwidth,
            average_quality: avg_quality,
        }
    }
}

/// Circuit statistics
#[derive(Debug, Clone)]
pub struct CircuitStats {
    /// Total number of circuits
    pub total_circuits: usize,
    /// Number of active circuits
    pub active_circuits: usize,
    /// Total bandwidth used
    pub total_bandwidth: u64,
    /// Average circuit quality
    pub average_quality: f64,
}

/// Directory authority client for node discovery
#[derive(Debug)]
pub struct DirectoryClient {
    /// Known nodes with their public keys
    nodes: Arc<TokioMutex<HashMap<Vec<u8>, NodeInfo>>>,
    /// Directory servers
    #[allow(dead_code)]
    directory_servers: Vec<String>,
    /// Last directory update
    last_update: Arc<TokioMutex<Instant>>,
    /// Update interval
    update_interval: Duration,
}

/// Node information in directory
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Node ID
    pub id: Vec<u8>,
    /// ML-KEM public key
    pub public_key: KEMPublicKey,
    /// Node address
    pub address: String,
    /// Node bandwidth (bytes/sec)
    pub bandwidth: u64,
    /// Node uptime
    pub uptime: Duration,
    /// Node flags
    pub flags: NodeFlags,
    /// Last seen timestamp
    pub last_seen: Instant,
}

/// Node capability flags
#[derive(Debug, Clone, Default)]
pub struct NodeFlags {
    /// Node can be used as entry guard
    pub guard: bool,
    /// Node can be used as exit
    pub exit: bool,
    /// Node is a directory server
    pub directory: bool,
    /// Node supports high bandwidth
    pub fast: bool,
    /// Node is stable
    pub stable: bool,
}

impl DirectoryClient {
    /// Create new directory client
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(TokioMutex::new(HashMap::new())),
            directory_servers: vec![
                "dir1.qudag.net:9030".to_string(),
                "dir2.qudag.net:9030".to_string(),
                "dir3.qudag.net:9030".to_string(),
            ],
            last_update: Arc::new(TokioMutex::new(Instant::now())),
            update_interval: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Get public key for a node
    pub async fn get_public_key(&self, node_id: &[u8]) -> Result<KEMPublicKey, String> {
        let nodes = self.nodes.lock().await;
        nodes
            .get(node_id)
            .map(|info| info.public_key.clone())
            .ok_or_else(|| "Node not found in directory".to_string())
    }

    /// Select random nodes for circuit building
    pub async fn select_random_nodes(&self, count: usize) -> Result<Vec<Vec<u8>>, OnionError> {
        // Update directory if needed
        self.update_directory_if_needed().await?;

        let nodes = self.nodes.lock().await;

        // Filter active nodes
        let active_nodes: Vec<_> = nodes
            .values()
            .filter(|n| n.last_seen.elapsed() < Duration::from_secs(3600))
            .filter(|n| n.flags.stable)
            .collect();

        if active_nodes.len() < count {
            return Err(OnionError::RouteError("Not enough active nodes".into()));
        }

        // Select nodes with bandwidth weighting
        let mut selected = Vec::new();
        let mut available = active_nodes.clone();

        for i in 0..count {
            // For entry guard, prefer nodes with guard flag
            if i == 0 {
                let guards: Vec<_> = available
                    .iter()
                    .filter(|n| n.flags.guard)
                    .copied()
                    .collect();

                if !guards.is_empty() {
                    available = guards;
                }
            }

            // For exit node, require exit flag
            if i == count - 1 {
                available.retain(|n| n.flags.exit);
                if available.is_empty() {
                    return Err(OnionError::RouteError("No exit nodes available".into()));
                }
            }

            // Select node weighted by bandwidth
            let total_bandwidth: u64 = available.iter().map(|n| n.bandwidth).sum();
            let mut target = thread_rng().gen_range(0..total_bandwidth);

            for (idx, node) in available.iter().enumerate() {
                if target < node.bandwidth {
                    selected.push(node.public_key.as_bytes().to_vec());
                    available.remove(idx);
                    break;
                }
                target -= node.bandwidth;
            }
        }

        Ok(selected)
    }

    /// Update directory from directory servers
    async fn update_directory_if_needed(&self) -> Result<(), OnionError> {
        let last_update = *self.last_update.lock().await;

        if last_update.elapsed() < self.update_interval {
            return Ok(());
        }

        // In a real implementation, this would fetch from directory servers
        // For now, we'll simulate with some dummy nodes
        self.populate_dummy_nodes().await;

        *self.last_update.lock().await = Instant::now();
        Ok(())
    }

    /// Populate dummy nodes for testing
    async fn populate_dummy_nodes(&self) {
        let mut nodes = self.nodes.lock().await;

        for i in 0..20 {
            let (public_key, _) = MlKem768::keygen().unwrap();
            let node_id = vec![i as u8; 32];

            let info = NodeInfo {
                id: node_id.clone(),
                public_key,
                address: format!("node{}.qudag.net:9001", i),
                bandwidth: 1_000_000 * (i as u64 + 1), // 1-20 MB/s
                uptime: Duration::from_secs(3600 * (i as u64 + 1)),
                flags: NodeFlags {
                    guard: i < 5,
                    exit: i >= 15,
                    directory: i % 5 == 0,
                    fast: i % 2 == 0,
                    stable: true,
                },
                last_seen: Instant::now(),
            };

            nodes.insert(node_id, info);
        }
    }

    /// Measure bandwidth to a node
    pub async fn measure_bandwidth(&self, node_id: &[u8]) -> Result<u64, OnionError> {
        // In a real implementation, this would perform actual bandwidth testing
        // For now, return the stored bandwidth
        let nodes = self.nodes.lock().await;
        nodes
            .get(node_id)
            .map(|info| info.bandwidth)
            .ok_or_else(|| OnionError::RouteError("Node not found".into()))
    }

    /// Get load balancing weights for nodes
    pub async fn get_load_balancing_weights(&self) -> HashMap<Vec<u8>, f64> {
        let nodes = self.nodes.lock().await;
        let total_bandwidth: u64 = nodes.values().map(|n| n.bandwidth).sum();

        nodes
            .iter()
            .map(|(id, info)| {
                let weight = info.bandwidth as f64 / total_bandwidth as f64;
                (id.clone(), weight)
            })
            .collect()
    }
}

/// Hop metadata for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopMetadata {
    /// Circuit ID for this hop
    pub circuit_id: u64,
    /// Hop number in circuit
    pub hop_number: u8,
    /// Next hop node ID (None for exit)
    pub next_hop: Option<Vec<u8>>,
    /// Layer processing flags
    pub flags: LayerFlags,
}

/// Layer processing flags
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayerFlags {
    /// This is an exit layer
    pub is_exit: bool,
    /// Apply traffic padding
    pub padding_enabled: bool,
    /// Apply timing obfuscation
    pub timing_obfuscation: bool,
    /// Acknowledgment required
    pub ack_required: bool,
}

/// Traffic analysis resistance utilities
pub struct TrafficAnalysisResistance {
    /// Configuration for traffic analysis resistance
    config: TrafficAnalysisConfig,
    /// Pattern database for normal traffic
    pattern_db: TrafficPatternDatabase,
}

/// Configuration for traffic analysis resistance
#[derive(Debug, Clone)]
pub struct TrafficAnalysisConfig {
    /// Enable pattern mimicking
    pub enable_pattern_mimicking: bool,
    /// Enable burst obfuscation
    pub enable_burst_obfuscation: bool,
    /// Enable flow correlation resistance
    pub enable_flow_correlation_resistance: bool,
    /// Minimum inter-packet delay (milliseconds)
    pub min_inter_packet_delay: u64,
    /// Maximum inter-packet delay (milliseconds)
    pub max_inter_packet_delay: u64,
}

impl Default for TrafficAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_pattern_mimicking: true,
            enable_burst_obfuscation: true,
            enable_flow_correlation_resistance: true,
            min_inter_packet_delay: 10,
            max_inter_packet_delay: 100,
        }
    }
}

/// Database of traffic patterns for mimicking
#[derive(Debug)]
struct TrafficPatternDatabase {
    /// Known traffic patterns
    patterns: Vec<TrafficPattern>,
}

/// A traffic pattern for mimicking normal traffic
#[derive(Debug, Clone)]
struct TrafficPattern {
    /// Packet sizes in the pattern
    packet_sizes: Vec<usize>,
    /// Inter-packet delays in milliseconds
    inter_packet_delays: Vec<u64>,
    /// Pattern frequency weight
    weight: f64,
}

impl Default for TrafficAnalysisResistance {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficAnalysisResistance {
    /// Create a new traffic analysis resistance module
    pub fn new() -> Self {
        Self::with_config(TrafficAnalysisConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: TrafficAnalysisConfig) -> Self {
        Self {
            config,
            pattern_db: TrafficPatternDatabase::new(),
        }
    }

    /// Apply traffic analysis resistance to a message stream
    pub async fn apply_resistance(&self, messages: &mut [MixMessage]) -> Result<(), OnionError> {
        if self.config.enable_pattern_mimicking {
            self.apply_pattern_mimicking(messages).await?;
        }

        if self.config.enable_burst_obfuscation {
            self.apply_burst_obfuscation(messages).await?;
        }

        if self.config.enable_flow_correlation_resistance {
            self.apply_flow_correlation_resistance(messages).await?;
        }

        Ok(())
    }

    /// Apply pattern mimicking to make traffic look normal
    async fn apply_pattern_mimicking(&self, messages: &mut [MixMessage]) -> Result<(), OnionError> {
        let pattern = self.pattern_db.select_random_pattern();

        // Adjust message sizes to match pattern
        for (i, message) in messages.iter_mut().enumerate() {
            if let Some(&target_size) = pattern.packet_sizes.get(i % pattern.packet_sizes.len()) {
                message.normalized_size = target_size;

                // Pad or truncate content to match target size
                if message.content.len() < target_size {
                    let padding_size = target_size - message.content.len();
                    let mut padding = vec![0u8; padding_size];
                    thread_rng().fill_bytes(&mut padding);
                    message.content.extend(padding);
                } else if message.content.len() > target_size {
                    message.content.truncate(target_size);
                }
            }
        }

        // Apply inter-packet delays from pattern
        for (i, &delay) in pattern.inter_packet_delays.iter().enumerate() {
            if i > 0 && i <= messages.len() {
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }

        Ok(())
    }

    /// Apply burst obfuscation to break up traffic bursts
    async fn apply_burst_obfuscation(
        &self,
        _messages: &mut [MixMessage],
    ) -> Result<(), OnionError> {
        // Add random delays between burst detection and mitigation
        let burst_delay = thread_rng().next_u64()
            % (self.config.max_inter_packet_delay - self.config.min_inter_packet_delay)
            + self.config.min_inter_packet_delay;

        tokio::time::sleep(Duration::from_millis(burst_delay)).await;
        Ok(())
    }

    /// Apply flow correlation resistance
    async fn apply_flow_correlation_resistance(
        &self,
        messages: &mut [MixMessage],
    ) -> Result<(), OnionError> {
        // Randomize message order to prevent flow correlation
        use rand::seq::SliceRandom;
        messages.shuffle(&mut thread_rng());

        // Add variable delays to prevent timing correlation
        for _ in 0..messages.len() {
            let delay = thread_rng().next_u64()
                % (self.config.max_inter_packet_delay - self.config.min_inter_packet_delay)
                + self.config.min_inter_packet_delay;
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        Ok(())
    }
}

impl TrafficPatternDatabase {
    fn new() -> Self {
        // Initialize with some common traffic patterns
        let patterns = vec![
            TrafficPattern {
                packet_sizes: vec![1024, 1024, 512, 2048, 1024],
                inter_packet_delays: vec![50, 75, 25, 100, 30],
                weight: 1.0,
            },
            TrafficPattern {
                packet_sizes: vec![512, 512, 1024, 512, 4096],
                inter_packet_delays: vec![25, 25, 50, 25, 200],
                weight: 0.8,
            },
            TrafficPattern {
                packet_sizes: vec![2048, 1024, 1024, 1024, 2048],
                inter_packet_delays: vec![100, 50, 50, 50, 150],
                weight: 0.6,
            },
        ];

        Self { patterns }
    }

    fn select_random_pattern(&self) -> &TrafficPattern {
        // Weight-based selection
        let total_weight: f64 = self.patterns.iter().map(|p| p.weight).sum();
        let mut target = thread_rng().gen::<f64>() * total_weight;

        for pattern in &self.patterns {
            target -= pattern.weight;
            if target <= 0.0 {
                return pattern;
            }
        }

        // Fallback to first pattern
        &self.patterns[0]
    }
}
