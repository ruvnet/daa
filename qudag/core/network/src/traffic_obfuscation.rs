//! Comprehensive traffic obfuscation module for QuDAG
//!
//! This module implements advanced traffic obfuscation techniques including:
//! - Message size normalization
//! - Dummy traffic generation
//! - Traffic shaping and padding
//! - Mix network batching
//! - Protocol obfuscation
//! - Traffic analysis resistance

use crate::onion::{MixMessage, MixMessageType, MixNode, TrafficAnalysisResistance};
use crate::types::{MessagePriority, NetworkError, NetworkMessage};
use base64::{engine::general_purpose, Engine as _};
use rand::{thread_rng, Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use tracing::{info, warn};

/// Standard message sizes for normalization (in bytes)
pub const STANDARD_MESSAGE_SIZES: [usize; 8] = [
    512,   // 512B
    1024,  // 1KB
    2048,  // 2KB
    4096,  // 4KB (default)
    8192,  // 8KB
    16384, // 16KB
    32768, // 32KB
    65536, // 64KB
];

/// Default standard message size (4KB)
pub const DEFAULT_MESSAGE_SIZE: usize = 4096;

/// Traffic obfuscation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficObfuscationConfig {
    /// Enable message size normalization
    pub enable_size_normalization: bool,
    /// Standard message size (default: 4KB)
    pub standard_message_size: usize,
    /// Enable dummy traffic generation
    pub enable_dummy_traffic: bool,
    /// Dummy traffic ratio (0.0 to 1.0)
    pub dummy_traffic_ratio: f64,
    /// Enable traffic shaping
    pub enable_traffic_shaping: bool,
    /// Traffic shaping delay range (min, max) in milliseconds
    pub traffic_delay_range: (u64, u64),
    /// Enable mix network batching
    pub enable_mix_batching: bool,
    /// Mix batch size
    pub mix_batch_size: usize,
    /// Mix batch timeout
    pub mix_batch_timeout: Duration,
    /// Enable protocol obfuscation
    pub enable_protocol_obfuscation: bool,
    /// Protocol obfuscation patterns
    pub obfuscation_patterns: Vec<ObfuscationPattern>,
    /// Enable burst prevention
    pub enable_burst_prevention: bool,
    /// Maximum burst size
    pub max_burst_size: usize,
    /// Burst prevention delay (milliseconds)
    pub burst_prevention_delay: u64,
}

impl Default for TrafficObfuscationConfig {
    fn default() -> Self {
        Self {
            enable_size_normalization: true,
            standard_message_size: DEFAULT_MESSAGE_SIZE,
            enable_dummy_traffic: true,
            dummy_traffic_ratio: 0.15, // 15% dummy traffic
            enable_traffic_shaping: true,
            traffic_delay_range: (10, 100), // 10-100ms delays
            enable_mix_batching: true,
            mix_batch_size: 50,
            mix_batch_timeout: Duration::from_millis(500),
            enable_protocol_obfuscation: true,
            obfuscation_patterns: vec![
                ObfuscationPattern::Http,
                ObfuscationPattern::Https,
                ObfuscationPattern::WebSocket,
                ObfuscationPattern::Custom(vec![0x00, 0x01, 0x02, 0x03]),
            ],
            enable_burst_prevention: true,
            max_burst_size: 100,
            burst_prevention_delay: 50,
        }
    }
}

/// Protocol obfuscation patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObfuscationPattern {
    /// HTTP-like traffic
    Http,
    /// HTTPS-like traffic
    Https,
    /// WebSocket-like traffic
    WebSocket,
    /// DNS-like traffic
    Dns,
    /// Custom pattern
    Custom(Vec<u8>),
}

/// Traffic obfuscation manager
pub struct TrafficObfuscator {
    /// Configuration
    config: Arc<RwLock<TrafficObfuscationConfig>>,
    /// Mix node for batching
    mix_node: Arc<Mutex<MixNode>>,
    /// Traffic analysis resistance
    traffic_resistance: Arc<TrafficAnalysisResistance>,
    /// Dummy traffic generator
    dummy_generator: Arc<DummyTrafficGenerator>,
    /// Traffic shaper
    traffic_shaper: Arc<Mutex<TrafficShaper>>,
    /// Protocol obfuscator
    protocol_obfuscator: Arc<ProtocolObfuscator>,
    /// Message statistics
    stats: Arc<RwLock<ObfuscationStats>>,
    /// Shutdown signal
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

/// Obfuscation statistics
#[derive(Debug, Clone, Default)]
pub struct ObfuscationStats {
    /// Total messages processed
    pub total_messages: u64,
    /// Dummy messages generated
    pub dummy_messages: u64,
    /// Messages normalized
    pub normalized_messages: u64,
    /// Batches processed
    pub batches_processed: u64,
    /// Average batch size
    pub avg_batch_size: f64,
    /// Total bytes padded
    pub total_padding_bytes: u64,
    /// Protocol obfuscations applied
    pub protocol_obfuscations: u64,
    /// Bursts prevented
    pub bursts_prevented: u64,
}

// Ensure TrafficObfuscator is Send + Sync
unsafe impl Send for TrafficObfuscator {}
unsafe impl Sync for TrafficObfuscator {}

impl TrafficObfuscator {
    /// Create a new traffic obfuscator
    pub fn new(config: TrafficObfuscationConfig) -> Self {
        let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);

        let mix_config = crate::onion::MixConfig {
            batch_size: config.mix_batch_size,
            batch_timeout: config.mix_batch_timeout,
            target_rate: 50.0,
            dummy_probability: config.dummy_traffic_ratio,
            timing_obfuscation: config.enable_traffic_shaping,
        };

        Self {
            config: Arc::new(RwLock::new(config.clone())),
            mix_node: Arc::new(Mutex::new(MixNode::with_config(
                vec![0u8; 32], // Node ID
                mix_config,
            ))),
            traffic_resistance: Arc::new(TrafficAnalysisResistance::new()),
            dummy_generator: Arc::new(DummyTrafficGenerator::new(config.dummy_traffic_ratio)),
            traffic_shaper: Arc::new(Mutex::new(TrafficShaper::new(config.traffic_delay_range))),
            protocol_obfuscator: Arc::new(ProtocolObfuscator::new(config.obfuscation_patterns)),
            stats: Arc::new(RwLock::new(ObfuscationStats::default())),
            shutdown_tx,
        }
    }

    /// Start the traffic obfuscator
    pub async fn start(&self) {
        info!("Starting traffic obfuscator");

        // Start dummy traffic generation
        if self.config.read().await.enable_dummy_traffic {
            self.start_dummy_traffic_generation().await;
        }

        // Start periodic batch flushing
        if self.config.read().await.enable_mix_batching {
            self.start_batch_flushing().await;
        }
    }

    /// Stop the traffic obfuscator
    pub async fn stop(&self) {
        info!("Stopping traffic obfuscator");
        let _ = self.shutdown_tx.send(());
    }

    /// Process a message through obfuscation pipeline
    pub async fn obfuscate_message(
        &self,
        mut message: NetworkMessage,
    ) -> Result<Vec<u8>, NetworkError> {
        let config = self.config.read().await;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_messages += 1;
        }

        // Apply message size normalization
        if config.enable_size_normalization {
            message = self.normalize_message_size(message).await?;
        }

        // Apply traffic shaping delay
        if config.enable_traffic_shaping {
            self.apply_traffic_shaping().await?;
        }

        // Convert to mix message
        let mix_message = self.to_mix_message(message).await?;

        // Add to mix node for batching
        if config.enable_mix_batching {
            self.mix_node
                .lock()
                .await
                .add_message(mix_message)
                .await
                .map_err(|e| NetworkError::Internal(format!("Mix batching failed: {}", e)))?;

            // Return empty for now - actual sending happens in batch
            return Ok(Vec::new());
        }

        // If not batching, serialize and apply protocol obfuscation
        let serialized = bincode::serialize(&mix_message)
            .map_err(|e| NetworkError::Internal(format!("Serialization failed: {}", e)))?;

        if config.enable_protocol_obfuscation {
            Ok(self.protocol_obfuscator.obfuscate(serialized).await?)
        } else {
            Ok(serialized)
        }
    }

    /// Process a batch of messages
    pub async fn process_batch(&self) -> Result<Vec<Vec<u8>>, NetworkError> {
        let config = self.config.read().await;
        let mut mix_node = self.mix_node.lock().await;

        // Flush the batch
        let batch = mix_node
            .flush_batch()
            .await
            .map_err(|e| NetworkError::Internal(format!("Batch flush failed: {}", e)))?;

        if batch.is_empty() {
            return Ok(Vec::new());
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.batches_processed += 1;
            stats.avg_batch_size = ((stats.avg_batch_size * (stats.batches_processed - 1) as f64)
                + batch.len() as f64)
                / stats.batches_processed as f64;
        }

        // Apply traffic analysis resistance
        let mut batch_messages = batch;
        self.traffic_resistance
            .apply_resistance(&mut batch_messages)
            .await
            .map_err(|e| NetworkError::Internal(format!("Traffic resistance failed: {}", e)))?;

        // Serialize and obfuscate each message
        let mut obfuscated_messages = Vec::new();
        for msg in batch_messages {
            let serialized = bincode::serialize(&msg)
                .map_err(|e| NetworkError::Internal(format!("Serialization failed: {}", e)))?;

            if config.enable_protocol_obfuscation {
                obfuscated_messages.push(self.protocol_obfuscator.obfuscate(serialized).await?);
            } else {
                obfuscated_messages.push(serialized);
            }
        }

        Ok(obfuscated_messages)
    }

    /// Normalize message size to standard size
    async fn normalize_message_size(
        &self,
        mut message: NetworkMessage,
    ) -> Result<NetworkMessage, NetworkError> {
        let config = self.config.read().await;
        let target_size = config.standard_message_size;
        let current_size = message.payload.len();

        if current_size < target_size {
            // Add padding
            let padding_size = target_size - current_size;
            let mut padding = vec![0u8; padding_size];
            thread_rng().fill_bytes(&mut padding);
            message.payload.extend(padding);

            // Update statistics
            let mut stats = self.stats.write().await;
            stats.normalized_messages += 1;
            stats.total_padding_bytes += padding_size as u64;
        } else if current_size > target_size {
            // For messages larger than standard size, round up to next standard size
            let next_size = STANDARD_MESSAGE_SIZES
                .iter()
                .find(|&&size| size >= current_size)
                .copied()
                .unwrap_or_else(|| {
                    // Round up to next multiple of largest standard size
                    let largest = STANDARD_MESSAGE_SIZES.last().unwrap();
                    current_size.div_ceil(*largest) * largest
                });

            if next_size > current_size {
                let padding_size = next_size - current_size;
                let mut padding = vec![0u8; padding_size];
                thread_rng().fill_bytes(&mut padding);
                message.payload.extend(padding);

                // Update statistics
                let mut stats = self.stats.write().await;
                stats.normalized_messages += 1;
                stats.total_padding_bytes += padding_size as u64;
            }
        }

        Ok(message)
    }

    /// Apply traffic shaping delay
    async fn apply_traffic_shaping(&self) -> Result<(), NetworkError> {
        self.traffic_shaper.lock().await.apply_delay().await;
        Ok(())
    }

    /// Convert network message to mix message
    async fn to_mix_message(&self, message: NetworkMessage) -> Result<MixMessage, NetworkError> {
        let content = bincode::serialize(&message)
            .map_err(|e| NetworkError::Internal(format!("Serialization failed: {}", e)))?;

        let priority = match message.priority {
            MessagePriority::High => 2,
            MessagePriority::Normal => 1,
            MessagePriority::Low => 0,
        };

        Ok(MixMessage {
            content,
            priority,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message_type: MixMessageType::Real,
            normalized_size: 0, // Will be set by mix node
        })
    }

    /// Start dummy traffic generation
    async fn start_dummy_traffic_generation(&self) {
        let dummy_generator = self.dummy_generator.clone();
        let mix_node = self.mix_node.clone();
        let stats = self.stats.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Some(dummy_msg) = dummy_generator.generate().await {
                            if let Err(e) = mix_node.lock().await.add_message(dummy_msg).await {
                                warn!("Failed to add dummy message: {}", e);
                            } else {
                                let mut stats = stats.write().await;
                                stats.dummy_messages += 1;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Stopping dummy traffic generation");
                        break;
                    }
                }
            }
        });
    }

    /// Start periodic batch flushing
    async fn start_batch_flushing(&self) {
        let obfuscator = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Check if batch should be flushed
                        let should_flush = {
                            let mix_node = obfuscator.mix_node.lock().await;
                            mix_node.get_stats().buffer_size > 0
                        };

                        if should_flush {
                            if let Err(e) = obfuscator.process_batch().await {
                                warn!("Failed to process batch: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Stopping batch flushing");
                        break;
                    }
                }
            }
        });
    }

    /// Get obfuscation statistics
    pub async fn get_stats(&self) -> ObfuscationStats {
        self.stats.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: TrafficObfuscationConfig) {
        *self.config.write().await = config;
    }
}

/// Dummy traffic generator
struct DummyTrafficGenerator {
    /// Dummy traffic ratio
    ratio: f64,
    /// Message counter for ratio calculation
    message_counter: Arc<Mutex<u64>>,
}

// Ensure DummyTrafficGenerator is Send + Sync
unsafe impl Send for DummyTrafficGenerator {}
unsafe impl Sync for DummyTrafficGenerator {}

impl DummyTrafficGenerator {
    fn new(ratio: f64) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            message_counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn generate(&self) -> Option<MixMessage> {
        let mut counter = self.message_counter.lock().await;
        *counter += 1;

        // Generate dummy based on ratio
        if thread_rng().gen::<f64>() < self.ratio {
            // Generate random dummy content
            let size =
                STANDARD_MESSAGE_SIZES[thread_rng().gen_range(0..STANDARD_MESSAGE_SIZES.len())];
            let mut content = vec![0u8; size];
            thread_rng().fill_bytes(&mut content);

            Some(MixMessage {
                content,
                priority: 0,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                message_type: MixMessageType::Dummy,
                normalized_size: size,
            })
        } else {
            None
        }
    }
}

/// Traffic shaper for controlling message timing
struct TrafficShaper {
    /// Delay range (min, max) in milliseconds
    delay_range: (u64, u64),
    /// Last message time
    last_message_time: Arc<Mutex<SystemTime>>,
    /// Burst prevention
    burst_counter: Arc<Mutex<usize>>,
    burst_reset_time: Arc<Mutex<SystemTime>>,
}

// Ensure TrafficShaper is Send + Sync
unsafe impl Send for TrafficShaper {}
unsafe impl Sync for TrafficShaper {}

impl TrafficShaper {
    fn new(delay_range: (u64, u64)) -> Self {
        Self {
            delay_range,
            last_message_time: Arc::new(Mutex::new(SystemTime::now())),
            burst_counter: Arc::new(Mutex::new(0)),
            burst_reset_time: Arc::new(Mutex::new(SystemTime::now())),
        }
    }

    async fn apply_delay(&self) {
        // Generate random delay within range
        let delay_ms = thread_rng().gen_range(self.delay_range.0..=self.delay_range.1);

        // Check for burst prevention
        let mut burst_counter = self.burst_counter.lock().await;
        let mut burst_reset_time = self.burst_reset_time.lock().await;

        let now = SystemTime::now();
        if now
            .duration_since(*burst_reset_time)
            .unwrap_or(Duration::ZERO)
            > Duration::from_secs(1)
        {
            // Reset burst counter every second
            *burst_counter = 0;
            *burst_reset_time = now;
        }

        *burst_counter += 1;
        if *burst_counter > 100 {
            // Burst detected, apply additional delay
            sleep(Duration::from_millis(delay_ms * 2)).await;
        } else {
            sleep(Duration::from_millis(delay_ms)).await;
        }

        // Update last message time
        *self.last_message_time.lock().await = SystemTime::now();
    }
}

/// Protocol obfuscator for disguising traffic patterns
struct ProtocolObfuscator {
    /// Obfuscation patterns
    patterns: Vec<ObfuscationPattern>,
}

// Ensure ProtocolObfuscator is Send + Sync
unsafe impl Send for ProtocolObfuscator {}
unsafe impl Sync for ProtocolObfuscator {}

impl ProtocolObfuscator {
    fn new(patterns: Vec<ObfuscationPattern>) -> Self {
        Self { patterns }
    }

    async fn obfuscate(&self, data: Vec<u8>) -> Result<Vec<u8>, NetworkError> {
        // Select random pattern
        let pattern = &self.patterns[thread_rng().gen_range(0..self.patterns.len())];

        match pattern {
            ObfuscationPattern::Http => self.obfuscate_as_http(data),
            ObfuscationPattern::Https => self.obfuscate_as_https(data),
            ObfuscationPattern::WebSocket => self.obfuscate_as_websocket(data),
            ObfuscationPattern::Dns => self.obfuscate_as_dns(data),
            ObfuscationPattern::Custom(header) => self.obfuscate_with_custom(data, header),
        }
    }

    fn obfuscate_as_http(&self, data: Vec<u8>) -> Result<Vec<u8>, NetworkError> {
        // Create HTTP-like request
        let encoded = general_purpose::STANDARD.encode(&data);
        let http_request = format!(
            "POST /api/v1/data HTTP/1.1\r\n\
            Host: example.com\r\n\
            User-Agent: Mozilla/5.0\r\n\
            Content-Type: application/octet-stream\r\n\
            Content-Length: {}\r\n\
            X-Request-ID: {}\r\n\
            \r\n\
            {}",
            encoded.len(),
            uuid::Uuid::new_v4(),
            encoded
        );

        Ok(http_request.into_bytes())
    }

    fn obfuscate_as_https(&self, data: Vec<u8>) -> Result<Vec<u8>, NetworkError> {
        // Simulate TLS record
        let mut obfuscated = Vec::new();

        // TLS record header
        obfuscated.push(0x17); // Application data
        obfuscated.push(0x03); // TLS version 1.2
        obfuscated.push(0x03);
        obfuscated.extend_from_slice(&(data.len() as u16).to_be_bytes());

        // Encrypted payload (just base64 encoded for simulation)
        obfuscated.extend_from_slice(&data);

        Ok(obfuscated)
    }

    fn obfuscate_as_websocket(&self, data: Vec<u8>) -> Result<Vec<u8>, NetworkError> {
        // Create WebSocket frame
        let mut frame = Vec::new();

        // FIN = 1, opcode = binary (2)
        frame.push(0x82);

        // Payload length
        if data.len() < 126 {
            frame.push(data.len() as u8 | 0x80); // Masked
        } else if data.len() < 65536 {
            frame.push(126 | 0x80);
            frame.extend_from_slice(&(data.len() as u16).to_be_bytes());
        } else {
            frame.push(127 | 0x80);
            frame.extend_from_slice(&(data.len() as u64).to_be_bytes());
        }

        // Masking key
        let mut mask = [0u8; 4];
        thread_rng().fill_bytes(&mut mask);
        frame.extend_from_slice(&mask);

        // Masked payload
        for (i, &byte) in data.iter().enumerate() {
            frame.push(byte ^ mask[i % 4]);
        }

        Ok(frame)
    }

    fn obfuscate_as_dns(&self, data: Vec<u8>) -> Result<Vec<u8>, NetworkError> {
        // Create DNS-like query with data encoded in subdomains
        let encoded = general_purpose::STANDARD
            .encode(&data)
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        // Split into DNS labels (max 63 chars each)
        let labels: Vec<String> = encoded
            .chars()
            .collect::<Vec<char>>()
            .chunks(63)
            .map(|chunk| chunk.iter().collect())
            .collect();

        // Create DNS query structure
        let mut dns_query = Vec::new();

        // DNS header
        dns_query.extend_from_slice(&thread_rng().next_u32().to_be_bytes()[..2]); // ID
        dns_query.extend_from_slice(&[0x01, 0x00]); // Flags (standard query)
        dns_query.extend_from_slice(&[0x00, 0x01]); // Questions
        dns_query.extend_from_slice(&[0x00, 0x00]); // Answers
        dns_query.extend_from_slice(&[0x00, 0x00]); // Authority
        dns_query.extend_from_slice(&[0x00, 0x00]); // Additional

        // Query section
        for label in labels.iter().take(4) {
            // Limit to 4 labels
            dns_query.push(label.len() as u8);
            dns_query.extend_from_slice(label.as_bytes());
        }
        dns_query.push(0); // Root label

        // Query type and class
        dns_query.extend_from_slice(&[0x00, 0x01]); // Type A
        dns_query.extend_from_slice(&[0x00, 0x01]); // Class IN

        Ok(dns_query)
    }

    fn obfuscate_with_custom(
        &self,
        mut data: Vec<u8>,
        header: &[u8],
    ) -> Result<Vec<u8>, NetworkError> {
        // Prepend custom header
        let mut obfuscated = header.to_vec();
        obfuscated.append(&mut data);
        Ok(obfuscated)
    }
}

impl Clone for TrafficObfuscator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            mix_node: self.mix_node.clone(),
            traffic_resistance: self.traffic_resistance.clone(),
            dummy_generator: self.dummy_generator.clone(),
            traffic_shaper: self.traffic_shaper.clone(),
            protocol_obfuscator: self.protocol_obfuscator.clone(),
            stats: self.stats.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_normalization() {
        let config = TrafficObfuscationConfig {
            enable_size_normalization: true,
            standard_message_size: 4096,
            ..Default::default()
        };

        let obfuscator = TrafficObfuscator::new(config);

        // Test small message padding
        let small_msg = NetworkMessage {
            id: "test1".to_string(),
            source: vec![1, 2, 3],
            destination: vec![4, 5, 6],
            payload: vec![0u8; 100], // 100 bytes
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let normalized = obfuscator.normalize_message_size(small_msg).await.unwrap();
        assert_eq!(normalized.payload.len(), 4096);

        // Test large message rounding
        let large_msg = NetworkMessage {
            id: "test2".to_string(),
            source: vec![1, 2, 3],
            destination: vec![4, 5, 6],
            payload: vec![0u8; 5000], // 5000 bytes
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let normalized = obfuscator.normalize_message_size(large_msg).await.unwrap();
        assert_eq!(normalized.payload.len(), 8192); // Next standard size
    }

    #[tokio::test]
    async fn test_dummy_traffic_generation() {
        let generator = DummyTrafficGenerator::new(0.5); // 50% dummy traffic

        let mut dummy_count = 0;
        for _ in 0..100 {
            if generator.generate().await.is_some() {
                dummy_count += 1;
            }
        }

        // Should be roughly 50% dummy messages (allow some variance)
        assert!(dummy_count > 30 && dummy_count < 70);
    }

    #[tokio::test]
    async fn test_protocol_obfuscation() {
        let obfuscator = ProtocolObfuscator::new(vec![
            ObfuscationPattern::Http,
            ObfuscationPattern::Https,
            ObfuscationPattern::WebSocket,
            ObfuscationPattern::Dns,
        ]);

        let data = vec![1, 2, 3, 4, 5];

        // Test HTTP obfuscation
        let http_result = obfuscator.obfuscate_as_http(data.clone()).unwrap();
        let http_str = String::from_utf8_lossy(&http_result);
        assert!(http_str.contains("HTTP/1.1"));
        assert!(http_str.contains("Content-Type: application/octet-stream"));

        // Test HTTPS obfuscation
        let https_result = obfuscator.obfuscate_as_https(data.clone()).unwrap();
        assert_eq!(https_result[0], 0x17); // Application data
        assert_eq!(https_result[1], 0x03); // TLS 1.2
        assert_eq!(https_result[2], 0x03);

        // Test WebSocket obfuscation
        let ws_result = obfuscator.obfuscate_as_websocket(data.clone()).unwrap();
        assert_eq!(ws_result[0], 0x82); // Binary frame

        // Test DNS obfuscation
        let dns_result = obfuscator.obfuscate_as_dns(data).unwrap();
        assert!(dns_result.len() > 12); // At least DNS header size
    }

    #[tokio::test]
    async fn test_traffic_shaping() {
        let shaper = TrafficShaper::new((10, 50));

        let start = SystemTime::now();
        shaper.apply_delay().await;
        let elapsed = start.elapsed().unwrap();

        // Should have delayed between 10-50ms
        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed <= Duration::from_millis(60)); // Allow some overhead
    }
}
