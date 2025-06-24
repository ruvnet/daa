#![deny(unsafe_code)]

use crate::types::{MessagePriority, NetworkError, NetworkMessage, PeerId};
use blake3::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, warn};

/// Maximum chunk size (64KB)
const MAX_CHUNK_SIZE: usize = 65536;

/// Maximum chunks per message
const MAX_CHUNKS: usize = 10000;

/// Chunk timeout duration
const CHUNK_TIMEOUT: Duration = Duration::from_secs(30);

/// Message chunk header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHeader {
    /// Message ID
    pub message_id: String,
    /// Total number of chunks
    pub total_chunks: u32,
    /// Current chunk index
    pub chunk_index: u32,
    /// Chunk size
    pub chunk_size: usize,
    /// Message hash (for verification)
    pub message_hash: [u8; 32],
    /// Original message size
    pub original_size: usize,
}

/// Chunked message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkedMessage {
    /// Chunk header
    pub header: ChunkHeader,
    /// Chunk data
    pub data: Vec<u8>,
}

/// Streaming message chunk
#[derive(Debug)]
pub struct StreamingChunk {
    /// Chunk data
    pub data: Vec<u8>,
    /// Timestamp when received
    pub received_at: Instant,
}

/// Message reassembly state
#[derive(Debug)]
pub struct ReassemblyState {
    /// Received chunks
    pub chunks: HashMap<u32, StreamingChunk>,
    /// Total expected chunks
    pub total_chunks: u32,
    /// Original message size
    pub original_size: usize,
    /// Message hash for verification
    pub message_hash: [u8; 32],
    /// First chunk received time
    pub started_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
}

/// Message chunking and reassembly system
pub struct MessageChunker {
    /// Reassembly states for incoming messages
    reassembly_states: Arc<RwLock<HashMap<String, ReassemblyState>>>,
    /// Chunk cache for optimization
    chunk_cache: Arc<Mutex<lru::LruCache<String, Vec<u8>>>>,
    /// Configuration
    config: ChunkerConfig,
}

/// Chunker configuration
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Maximum chunk size
    pub max_chunk_size: usize,
    /// Chunk timeout
    pub chunk_timeout: Duration,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression threshold
    pub compression_threshold: usize,
    /// Cache size
    pub cache_size: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: MAX_CHUNK_SIZE,
            chunk_timeout: CHUNK_TIMEOUT,
            enable_compression: true,
            compression_threshold: 1024,
            cache_size: 1000,
        }
    }
}

impl MessageChunker {
    /// Create a new message chunker
    pub fn new(config: ChunkerConfig) -> Self {
        Self {
            reassembly_states: Arc::new(RwLock::new(HashMap::new())),
            chunk_cache: Arc::new(Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(config.cache_size).unwrap(),
            ))),
            config,
        }
    }

    /// Chunk a message for transmission
    pub async fn chunk_message(
        &self,
        message: &NetworkMessage,
    ) -> Result<Vec<ChunkedMessage>, NetworkError> {
        let payload = &message.payload;
        
        // Check if chunking is needed
        if payload.len() <= self.config.max_chunk_size {
            return Ok(vec![]); // No chunking needed
        }

        // Optionally compress the payload
        let data = if self.config.enable_compression && payload.len() > self.config.compression_threshold {
            self.compress_data(payload)?
        } else {
            payload.clone()
        };

        // Calculate message hash
        let message_hash = blake3::hash(&data);

        // Calculate number of chunks
        let total_chunks = ((data.len() + self.config.max_chunk_size - 1) / self.config.max_chunk_size) as u32;
        
        if total_chunks > MAX_CHUNKS as u32 {
            return Err(NetworkError::ValidationError(
                format!("Message too large: {} chunks exceeds maximum {}", total_chunks, MAX_CHUNKS)
            ));
        }

        // Create chunks
        let mut chunks = Vec::with_capacity(total_chunks as usize);
        
        for (index, chunk_data) in data.chunks(self.config.max_chunk_size).enumerate() {
            let header = ChunkHeader {
                message_id: message.id.clone(),
                total_chunks,
                chunk_index: index as u32,
                chunk_size: chunk_data.len(),
                message_hash: *message_hash.as_bytes(),
                original_size: payload.len(),
            };

            chunks.push(ChunkedMessage {
                header,
                data: chunk_data.to_vec(),
            });
        }

        debug!(
            "Chunked message {} into {} chunks (original: {} bytes, chunked: {} bytes)",
            message.id,
            chunks.len(),
            payload.len(),
            data.len()
        );

        Ok(chunks)
    }

    /// Process an incoming chunk
    pub async fn process_chunk(
        &self,
        chunk: ChunkedMessage,
    ) -> Result<Option<Vec<u8>>, NetworkError> {
        let message_id = chunk.header.message_id.clone();
        
        // Validate chunk
        self.validate_chunk(&chunk)?;

        // Check cache first
        if let Some(cached) = self.chunk_cache.lock().await.get(&message_id) {
            return Ok(Some(cached.clone()));
        }

        let mut states = self.reassembly_states.write().await;
        
        // Get or create reassembly state
        let state = states.entry(message_id.clone()).or_insert_with(|| {
            ReassemblyState {
                chunks: HashMap::new(),
                total_chunks: chunk.header.total_chunks,
                original_size: chunk.header.original_size,
                message_hash: chunk.header.message_hash,
                started_at: Instant::now(),
                last_activity: Instant::now(),
            }
        });

        // Update last activity
        state.last_activity = Instant::now();

        // Validate consistency
        if state.total_chunks != chunk.header.total_chunks {
            return Err(NetworkError::ValidationError(
                "Inconsistent chunk count".into()
            ));
        }

        // Add chunk to state
        state.chunks.insert(
            chunk.header.chunk_index,
            StreamingChunk {
                data: chunk.data,
                received_at: Instant::now(),
            },
        );

        // Check if all chunks received
        if state.chunks.len() == state.total_chunks as usize {
            // Reassemble message
            let reassembled = self.reassemble_message(state)?;
            
            // Cache the result
            self.chunk_cache.lock().await.put(message_id.clone(), reassembled.clone());
            
            // Clean up state
            states.remove(&message_id);
            
            Ok(Some(reassembled))
        } else {
            debug!(
                "Received chunk {}/{} for message {}",
                chunk.header.chunk_index + 1,
                state.total_chunks,
                message_id
            );
            Ok(None)
        }
    }

    /// Validate a chunk
    fn validate_chunk(&self, chunk: &ChunkedMessage) -> Result<(), NetworkError> {
        if chunk.header.chunk_index >= chunk.header.total_chunks {
            return Err(NetworkError::ValidationError(
                "Invalid chunk index".into()
            ));
        }

        if chunk.data.len() != chunk.header.chunk_size {
            return Err(NetworkError::ValidationError(
                "Chunk size mismatch".into()
            ));
        }

        if chunk.header.chunk_size > self.config.max_chunk_size {
            return Err(NetworkError::ValidationError(
                "Chunk size exceeds maximum".into()
            ));
        }

        Ok(())
    }

    /// Reassemble chunks into original message
    fn reassemble_message(&self, state: &ReassemblyState) -> Result<Vec<u8>, NetworkError> {
        let mut data = Vec::with_capacity(state.original_size);
        
        // Reassemble in order
        for i in 0..state.total_chunks {
            let chunk = state.chunks.get(&i)
                .ok_or_else(|| NetworkError::ValidationError(
                    format!("Missing chunk {}", i)
                ))?;
            
            data.extend_from_slice(&chunk.data);
        }

        // Verify hash
        let computed_hash = blake3::hash(&data);
        if *computed_hash.as_bytes() != state.message_hash {
            return Err(NetworkError::ValidationError(
                "Message hash verification failed".into()
            ));
        }

        // Decompress if needed
        if self.config.enable_compression && data.len() < state.original_size {
            self.decompress_data(&data)
        } else {
            Ok(data)
        }
    }

    /// Clean up expired reassembly states
    pub async fn cleanup_expired(&self) {
        let mut states = self.reassembly_states.write().await;
        let now = Instant::now();
        
        states.retain(|id, state| {
            let expired = now.duration_since(state.last_activity) < self.config.chunk_timeout;
            if !expired {
                warn!(
                    "Cleaning up expired message reassembly for {} ({}/{} chunks received)",
                    id,
                    state.chunks.len(),
                    state.total_chunks
                );
            }
            expired
        });
    }

    /// Compress data using zstd
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, NetworkError> {
        zstd::encode_all(data, 3)
            .map_err(|e| NetworkError::Internal(format!("Compression failed: {}", e)))
    }

    /// Decompress data using zstd
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, NetworkError> {
        zstd::decode_all(data)
            .map_err(|e| NetworkError::Internal(format!("Decompression failed: {}", e)))
    }

    /// Get chunking statistics
    pub async fn get_stats(&self) -> ChunkerStats {
        let states = self.reassembly_states.read().await;
        let cache = self.chunk_cache.lock().await;
        
        ChunkerStats {
            active_reassemblies: states.len(),
            cache_size: cache.len(),
            total_chunks_waiting: states.values()
                .map(|s| s.chunks.len())
                .sum(),
        }
    }
}

/// Chunker statistics
#[derive(Debug, Clone)]
pub struct ChunkerStats {
    /// Number of active reassembly operations
    pub active_reassemblies: usize,
    /// Cache size
    pub cache_size: usize,
    /// Total chunks waiting
    pub total_chunks_waiting: usize,
}

/// Extension trait for NetworkMessage to support chunking
pub trait ChunkableMessage {
    /// Check if message needs chunking
    fn needs_chunking(&self, max_size: usize) -> bool;
    
    /// Create chunked variant of the message
    fn into_chunked(self) -> ChunkedNetworkMessage;
}

impl ChunkableMessage for NetworkMessage {
    fn needs_chunking(&self, max_size: usize) -> bool {
        self.payload.len() > max_size
    }
    
    fn into_chunked(self) -> ChunkedNetworkMessage {
        ChunkedNetworkMessage {
            base: self,
            chunks: None,
        }
    }
}

/// Chunked network message wrapper
#[derive(Debug, Clone)]
pub struct ChunkedNetworkMessage {
    /// Base message
    pub base: NetworkMessage,
    /// Optional chunks if message was chunked
    pub chunks: Option<Vec<ChunkedMessage>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_message_chunking() {
        let config = ChunkerConfig {
            max_chunk_size: 1024,
            ..Default::default()
        };
        let chunker = MessageChunker::new(config);

        // Create large message
        let message = NetworkMessage {
            id: Uuid::new_v4().to_string(),
            source: vec![1],
            destination: vec![2],
            payload: vec![0u8; 3000], // 3KB
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Chunk the message
        let chunks = chunker.chunk_message(&message).await.unwrap();
        assert_eq!(chunks.len(), 3); // Should be 3 chunks

        // Verify chunk properties
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.header.chunk_index, i as u32);
            assert_eq!(chunk.header.total_chunks, 3);
            assert!(chunk.header.chunk_size <= 1024);
        }
    }

    #[tokio::test]
    async fn test_message_reassembly() {
        let config = ChunkerConfig {
            max_chunk_size: 1024,
            enable_compression: false,
            ..Default::default()
        };
        let chunker = MessageChunker::new(config);

        let original_data = vec![42u8; 2500];
        let message = NetworkMessage {
            id: Uuid::new_v4().to_string(),
            source: vec![1],
            destination: vec![2],
            payload: original_data.clone(),
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Chunk and reassemble
        let chunks = chunker.chunk_message(&message).await.unwrap();
        
        let mut reassembled_data = None;
        for chunk in chunks {
            if let Some(data) = chunker.process_chunk(chunk).await.unwrap() {
                reassembled_data = Some(data);
            }
        }

        assert_eq!(reassembled_data.unwrap(), original_data);
    }

    #[tokio::test]
    async fn test_out_of_order_reassembly() {
        let config = ChunkerConfig {
            max_chunk_size: 1024,
            enable_compression: false,
            ..Default::default()
        };
        let chunker = MessageChunker::new(config);

        let original_data = vec![99u8; 3072]; // Exactly 3 chunks
        let message = NetworkMessage {
            id: Uuid::new_v4().to_string(),
            source: vec![1],
            destination: vec![2],
            payload: original_data.clone(),
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        let chunks = chunker.chunk_message(&message).await.unwrap();
        
        // Process chunks out of order
        let mut reassembled_data = None;
        
        // Process chunk 2, then 0, then 1
        chunker.process_chunk(chunks[2].clone()).await.unwrap();
        chunker.process_chunk(chunks[0].clone()).await.unwrap();
        
        if let Some(data) = chunker.process_chunk(chunks[1].clone()).await.unwrap() {
            reassembled_data = Some(data);
        }

        assert_eq!(reassembled_data.unwrap(), original_data);
    }
}