#![deny(unsafe_code)]

use crate::traffic_obfuscation::{TrafficObfuscationConfig, TrafficObfuscator};
use crate::types::{MessagePriority, NetworkError, NetworkMessage};
use anyhow::Result;
use blake3::Hash;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex, RwLock};

/// Serializable wrapper for blake3::Hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerializableHash(pub [u8; 32]);

impl From<Hash> for SerializableHash {
    fn from(hash: Hash) -> Self {
        SerializableHash(*hash.as_bytes())
    }
}

impl From<SerializableHash> for Hash {
    fn from(hash: SerializableHash) -> Self {
        Hash::from(hash.0)
    }
}

impl SerializableHash {
    /// Get the hash as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// High-performance message queue for network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// The actual message
    pub message: NetworkMessage,
    /// Message hash for integrity
    pub hash: SerializableHash,
    /// Timestamp
    pub timestamp: u64,
    /// Signature
    pub signature: Option<Vec<u8>>,
}

impl MessageEnvelope {
    pub fn new(message: NetworkMessage) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut hasher = blake3::Hasher::new();
        hasher.update(&bincode::serialize(&message).unwrap());
        hasher.update(&timestamp.to_le_bytes());

        Self {
            message,
            hash: hasher.finalize().into(),
            timestamp,
            signature: None,
        }
    }

    pub fn verify(&self) -> bool {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bincode::serialize(&self.message).unwrap());
        hasher.update(&self.timestamp.to_le_bytes());

        self.hash == hasher.finalize().into()
    }

    pub fn sign(&mut self, key: &[u8]) -> Result<(), NetworkError> {
        // Sign the message hash
        let signature = ring::signature::Ed25519KeyPair::from_seed_unchecked(key)
            .map_err(|e| NetworkError::EncryptionError(e.to_string()))?;

        self.signature = Some(signature.sign(self.hash.as_bytes()).as_ref().to_vec());
        Ok(())
    }

    pub fn verify_signature(&self, public_key: &[u8]) -> Result<bool, NetworkError> {
        match &self.signature {
            Some(sig) => {
                let peer_public_key =
                    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key);

                peer_public_key
                    .verify(self.hash.as_bytes(), sig)
                    .map(|_| true)
                    .map_err(|e| NetworkError::EncryptionError(e.to_string()))
            }
            None => Ok(false),
        }
    }
}

pub struct MessageQueue {
    /// High priority message queue
    high_priority: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    /// Normal priority message queue  
    normal_priority: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    /// Low priority message queue
    low_priority: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    /// Channel for message notifications
    notify_tx: mpsc::Sender<()>,
    /// Traffic obfuscator for message processing
    obfuscator: Option<Arc<TrafficObfuscator>>,
    /// Configuration for traffic obfuscation
    obfuscation_config: Arc<RwLock<TrafficObfuscationConfig>>,
}

impl MessageQueue {
    /// Creates a new message queue
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(1000);

        let queue = Self {
            high_priority: Arc::new(Mutex::new(VecDeque::with_capacity(10000))),
            normal_priority: Arc::new(Mutex::new(VecDeque::with_capacity(50000))),
            low_priority: Arc::new(Mutex::new(VecDeque::with_capacity(100000))),
            notify_tx: tx,
            obfuscator: None,
            obfuscation_config: Arc::new(RwLock::new(TrafficObfuscationConfig::default())),
        };

        (queue, rx)
    }

    /// Creates a new message queue with traffic obfuscation
    pub fn with_obfuscation(config: TrafficObfuscationConfig) -> (Self, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(1000);
        let obfuscator = Arc::new(TrafficObfuscator::new(config.clone()));

        let queue = Self {
            high_priority: Arc::new(Mutex::new(VecDeque::with_capacity(10000))),
            normal_priority: Arc::new(Mutex::new(VecDeque::with_capacity(50000))),
            low_priority: Arc::new(Mutex::new(VecDeque::with_capacity(100000))),
            notify_tx: tx,
            obfuscator: Some(obfuscator),
            obfuscation_config: Arc::new(RwLock::new(config)),
        };

        (queue, rx)
    }

    /// Enable traffic obfuscation
    pub async fn enable_obfuscation(&mut self, config: TrafficObfuscationConfig) {
        self.obfuscator = Some(Arc::new(TrafficObfuscator::new(config.clone())));
        *self.obfuscation_config.write().await = config;

        // Start the obfuscator
        if let Some(obfuscator) = &self.obfuscator {
            obfuscator.start().await;
        }
    }

    /// Enqueues a message with the specified priority
    pub async fn enqueue(&self, mut msg: NetworkMessage) -> Result<(), NetworkError> {
        // Apply obfuscation if enabled
        if let Some(obfuscator) = &self.obfuscator {
            // Process message through obfuscation pipeline
            let obfuscated_payload = obfuscator.obfuscate_message(msg.clone()).await?;

            // If obfuscation returns empty (batching), don't enqueue directly
            if obfuscated_payload.is_empty() {
                return Ok(());
            }

            // Update message with obfuscated payload
            msg.payload = obfuscated_payload;
        }

        let envelope = MessageEnvelope::new(msg.clone());

        // Verify message integrity
        if !envelope.verify() {
            return Err(NetworkError::Internal(
                "Message integrity check failed".into(),
            ));
        }
        let queue = match msg.priority {
            MessagePriority::High => &self.high_priority,
            MessagePriority::Normal => &self.normal_priority,
            MessagePriority::Low => &self.low_priority,
        };

        queue.lock().await.push_back(envelope);
        let _ = self.notify_tx.send(()).await;
        Ok(())
    }

    /// Dequeues the next message by priority
    pub async fn dequeue(&self) -> Option<MessageEnvelope> {
        if let Some(msg) = self.high_priority.lock().await.pop_front() {
            return Some(msg);
        }

        if let Some(msg) = self.normal_priority.lock().await.pop_front() {
            return Some(msg);
        }

        self.low_priority.lock().await.pop_front()
    }

    /// Returns the total number of queued messages
    pub async fn len(&self) -> usize {
        let high = self.high_priority.lock().await.len();
        let normal = self.normal_priority.lock().await.len();
        let low = self.low_priority.lock().await.len();
        high + normal + low
    }

    /// Returns true if the queue is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// Purge expired messages
    pub async fn purge_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Purge high priority
        let mut high = self.high_priority.lock().await;
        high.retain(|env| env.message.ttl.as_secs() + env.timestamp > now);

        // Purge normal priority
        let mut normal = self.normal_priority.lock().await;
        normal.retain(|env| env.message.ttl.as_secs() + env.timestamp > now);

        // Purge low priority
        let mut low = self.low_priority.lock().await;
        low.retain(|env| env.message.ttl.as_secs() + env.timestamp > now);
    }

    /// Process batched messages if obfuscation is enabled
    pub async fn process_batch(&self) -> Result<Vec<MessageEnvelope>, NetworkError> {
        if let Some(obfuscator) = &self.obfuscator {
            let obfuscated_messages = obfuscator.process_batch().await?;

            let mut envelopes = Vec::new();
            for obfuscated_data in obfuscated_messages {
                // Create a dummy message envelope for obfuscated data
                let msg = NetworkMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    source: vec![],
                    destination: vec![],
                    payload: obfuscated_data,
                    priority: MessagePriority::Normal,
                    ttl: std::time::Duration::from_secs(300),
                };
                envelopes.push(MessageEnvelope::new(msg));
            }

            Ok(envelopes)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get obfuscation statistics
    pub async fn get_obfuscation_stats(
        &self,
    ) -> Option<crate::traffic_obfuscation::ObfuscationStats> {
        if let Some(obfuscator) = &self.obfuscator {
            Some(obfuscator.get_stats().await)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_message_queue() {
        use std::thread;

        let (queue, _rx) = MessageQueue::new();

        // Create test messages
        let msg1 = NetworkMessage {
            id: "1".into(),
            source: vec![1],
            destination: vec![2],
            payload: vec![0; 100],
            priority: MessagePriority::High,
            ttl: Duration::from_secs(60),
        };

        let msg2 = NetworkMessage {
            id: "2".into(),
            source: vec![1],
            destination: vec![2],
            payload: vec![0; 100],
            priority: MessagePriority::Normal,
            ttl: Duration::from_secs(60),
        };

        // Test enqueue
        assert!(queue.enqueue(msg1.clone()).await.is_ok());

        // Test message verification
        let envelope = queue.dequeue().await.unwrap();
        assert!(envelope.verify());
        assert!(queue.enqueue(msg2.clone()).await.is_ok());
        assert_eq!(queue.len().await, 2);

        // Test priority dequeue
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.message.id, "1"); // High priority dequeued first

        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.message.id, "2"); // Normal priority dequeued second

        // Test message expiry
        let msg3 = NetworkMessage {
            id: "3".into(),
            source: vec![1],
            destination: vec![2],
            payload: vec![0; 100],
            priority: MessagePriority::Low,
            ttl: Duration::from_secs(1), // Short TTL
        };

        assert!(queue.enqueue(msg3).await.is_ok());
        assert_eq!(queue.len().await, 1);

        // Wait for message to expire
        thread::sleep(Duration::from_secs(2));
        queue.purge_expired().await;
        assert_eq!(queue.len().await, 0);
    }
}
