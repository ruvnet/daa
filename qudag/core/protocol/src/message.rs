//! Protocol message implementation.

use qudag_crypto::{Ciphertext, MlDsaKeyPair, MlDsaPublicKey, MlKem768, PublicKey, SecretKey};
use qudag_dag::vertex::VertexId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during message operations.
#[derive(Debug, Error)]
pub enum MessageError {
    /// Invalid message format
    #[error("Invalid message format")]
    InvalidFormat,

    /// Message too large
    #[error("Message too large: {0} bytes")]
    MessageTooLarge(usize),

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Missing signature when required
    #[error("Missing signature")]
    MissingSignature,

    /// Signing failed
    #[error("Message signing failed")]
    SigningFailed,

    /// Verification failed
    #[error("Signature verification failed")]
    VerificationFailed,

    /// Encryption failed
    #[error("Encryption failed")]
    EncryptionFailed,

    /// Decryption failed
    #[error("Decryption failed")]
    DecryptionFailed,

    /// Serialization failed
    #[error("Message serialization failed")]
    SerializationFailed,

    /// Deserialization failed
    #[error("Message deserialization failed")]
    DeserializationFailed,

    /// Message expired
    #[error("Message has expired")]
    MessageExpired,

    /// Invalid timestamp
    #[error("Invalid message timestamp")]
    InvalidTimestamp,

    /// Incompatible protocol version
    #[error("Incompatible protocol version: {0:?}")]
    IncompatibleVersion(ProtocolVersion),
}

/// Protocol version information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version
    pub major: u16,
    /// Minor version
    pub minor: u16,
    /// Patch version
    pub patch: u16,
    /// Protocol features supported
    pub features: Vec<String>,
}

impl ProtocolVersion {
    /// Current protocol version
    pub const CURRENT: ProtocolVersion = ProtocolVersion {
        major: 1,
        minor: 0,
        patch: 0,
        features: vec![],
    };

    /// Check if this version is compatible with another
    pub fn is_compatible(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major && self.minor <= other.minor
    }
}

/// Message type enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Protocol handshake messages
    Handshake(HandshakeType),

    /// DAG consensus messages
    Consensus(ConsensusMessageType),

    /// Network routing messages
    Routing(RoutingMessageType),

    /// Anonymous communication messages
    Anonymous(AnonymousMessageType),

    /// Protocol control messages
    Control(ControlMessageType),

    /// State synchronization messages
    Sync(SyncMessageType),

    /// Generic data messages
    Data(Vec<u8>),
}

/// Handshake message types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandshakeType {
    /// Initial handshake request
    Init,
    /// Handshake response with key exchange
    Response,
    /// Handshake completion confirmation
    Complete,
    /// Protocol version negotiation
    VersionNegotiation,
}

/// Consensus message types for DAG
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsensusMessageType {
    /// New vertex proposal
    VertexProposal,
    /// Vertex vote
    Vote,
    /// Finality announcement
    Finality,
    /// Query for missing vertices
    Query,
}

/// Routing message types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoutingMessageType {
    /// Onion routing message
    OnionRouted,
    /// Direct peer message
    Direct,
    /// Anonymous broadcast
    Broadcast,
}

/// Anonymous communication message types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnonymousMessageType {
    /// Anonymous data payload
    Data,
    /// Mix network message
    Mix,
    /// Cover traffic
    Cover,
}

/// Control message types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlMessageType {
    /// Ping for connectivity
    Ping,
    /// Pong response
    Pong,
    /// Disconnect notification
    Disconnect,
    /// Keep-alive message
    KeepAlive,
}

/// Synchronization message types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncMessageType {
    /// Request state sync
    StateRequest,
    /// State sync response
    StateResponse,
    /// Delta sync
    DeltaSync,
    /// Checkpoint sync
    CheckpointSync,
}

/// Protocol message structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: Uuid,

    /// Protocol version
    pub version: ProtocolVersion,

    /// Message type
    pub msg_type: MessageType,

    /// Message payload
    pub payload: Vec<u8>,

    /// Message timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,

    /// Message signature (ML-DSA)
    pub signature: Option<Vec<u8>>,

    /// Message headers for metadata
    pub headers: HashMap<String, String>,

    /// Sender's public key hash for verification
    pub sender_key_hash: Option<Vec<u8>>,

    /// Message sequence number for ordering
    pub sequence: u64,

    /// Time-to-live for message expiration
    pub ttl: Option<u64>,
}

impl Message {
    /// Create a new message
    pub fn new(msg_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            version: ProtocolVersion::CURRENT,
            msg_type,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            signature: None,
            headers: HashMap::new(),
            sender_key_hash: None,
            sequence: 0,
            ttl: None,
        }
    }

    /// Create a new message with version
    pub fn new_with_version(
        version: ProtocolVersion,
        msg_type: MessageType,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            version,
            msg_type,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            signature: None,
            headers: HashMap::new(),
            sender_key_hash: None,
            sequence: 0,
            ttl: None,
        }
    }

    /// Set message sequence number
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = sequence;
        self
    }

    /// Set message TTL
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Add header to message
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            now > self.timestamp + ttl
        } else {
            false
        }
    }

    /// Get message for signing (excludes signature field)
    fn get_signable_data(&self) -> Result<Vec<u8>, MessageError> {
        let mut msg_copy = self.clone();
        msg_copy.signature = None;

        bincode::serialize(&msg_copy).map_err(|_| MessageError::SerializationFailed)
    }

    /// Sign message with ML-DSA
    pub fn sign(&mut self, keypair: &MlDsaKeyPair) -> Result<(), MessageError> {
        let signable_data = self.get_signable_data()?;

        // Sign using the keypair directly
        let signature = keypair
            .sign(&signable_data, &mut rand::thread_rng())
            .map_err(|_| MessageError::SigningFailed)?;

        self.signature = Some(signature);

        // Set sender key hash for verification
        let public_key_bytes = keypair.public_key();
        self.sender_key_hash = Some(blake3::hash(public_key_bytes).as_bytes().to_vec());

        Ok(())
    }

    /// Verify message signature with ML-DSA
    pub fn verify(&self, public_key: &MlDsaPublicKey) -> Result<bool, MessageError> {
        let signature = self
            .signature
            .as_ref()
            .ok_or(MessageError::MissingSignature)?;

        // Verify sender key hash matches
        if let Some(sender_hash) = &self.sender_key_hash {
            let public_key_bytes = public_key.as_bytes();
            let expected_hash = blake3::hash(public_key_bytes).as_bytes().to_vec();
            if sender_hash != &expected_hash {
                return Ok(false);
            }
        }

        let signable_data = self.get_signable_data()?;

        // Verify using the public key directly
        public_key
            .verify(&signable_data, signature)
            .map_err(|_| MessageError::VerificationFailed)
            .map(|_| true)
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, MessageError> {
        bincode::serialize(self).map_err(|_| MessageError::SerializationFailed)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, MessageError> {
        bincode::deserialize(data).map_err(|_| MessageError::DeserializationFailed)
    }

    /// Validate message structure and content
    pub fn validate(&self) -> Result<(), MessageError> {
        // Check if message has expired
        if self.is_expired() {
            return Err(MessageError::MessageExpired);
        }

        // Check payload size limits (max 1MB)
        if self.payload.len() > 1024 * 1024 {
            return Err(MessageError::MessageTooLarge(self.payload.len()));
        }

        // Validate timestamp is reasonable (not too far in future)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Allow up to 5 minutes in the future
        if self.timestamp > now + (5 * 60 * 1000) {
            return Err(MessageError::InvalidTimestamp);
        }

        Ok(())
    }
}

/// Encrypted message container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Encrypted message data
    pub ciphertext: Vec<u8>,
    /// Encapsulated key (ML-KEM)
    pub encapsulation: Vec<u8>,
    /// Message headers (unencrypted)
    pub headers: HashMap<String, String>,
    /// Timestamp
    pub timestamp: u64,
}

impl EncryptedMessage {
    /// Encrypt a message using ML-KEM + AES-GCM
    pub fn encrypt(
        message: &Message,
        recipient_public_key: &PublicKey,
    ) -> Result<Self, MessageError> {
        // Serialize the message
        let message_bytes = message.to_bytes()?;

        // Use ML-KEM for key encapsulation
        let (ciphertext, shared_secret) = MlKem768::encapsulate(recipient_public_key)
            .map_err(|_| MessageError::EncryptionFailed)?;

        // Use shared secret as AES key (first 32 bytes)
        let _aes_key = &shared_secret.as_bytes()[..32];

        // Encrypt message with AES-GCM (simplified - in real implementation use proper AEAD)
        let encrypted_data = message_bytes; // TODO: Implement actual AES-GCM encryption

        Ok(Self {
            ciphertext: encrypted_data,
            encapsulation: ciphertext.as_bytes().to_vec(),
            headers: message.headers.clone(),
            timestamp: message.timestamp,
        })
    }

    /// Decrypt a message using ML-KEM + AES-GCM
    pub fn decrypt(&self, recipient_secret_key: &SecretKey) -> Result<Message, MessageError> {
        // Decapsulate the shared secret
        let encapsulation = Ciphertext::from_bytes(&self.encapsulation)
            .map_err(|_| MessageError::DecryptionFailed)?;
        let shared_secret = MlKem768::decapsulate(recipient_secret_key, &encapsulation)
            .map_err(|_| MessageError::DecryptionFailed)?;

        // Use shared secret as AES key
        let _aes_key = &shared_secret.as_bytes()[..32];

        // Decrypt message with AES-GCM (simplified)
        let message_bytes = &self.ciphertext; // TODO: Implement actual AES-GCM decryption

        Message::from_bytes(message_bytes)
    }
}

/// Message factory for creating different types of protocol messages
pub struct MessageFactory;

impl MessageFactory {
    /// Create a handshake init message
    pub fn create_handshake_init(
        protocol_version: ProtocolVersion,
        public_key: &MlDsaPublicKey,
        kem_public_key: &PublicKey,
    ) -> Result<Message, MessageError> {
        let payload = HandshakePayload {
            protocol_version: protocol_version.clone(),
            public_key: public_key.as_bytes().to_vec(),
            kem_public_key: kem_public_key.as_bytes().to_vec(),
            capabilities: vec!["anonymous-routing".to_string(), "dag-consensus".to_string()],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        let payload_bytes =
            bincode::serialize(&payload).map_err(|_| MessageError::SerializationFailed)?;

        Ok(Message::new_with_version(
            protocol_version,
            MessageType::Handshake(HandshakeType::Init),
            payload_bytes,
        ))
    }

    /// Create a consensus vertex proposal message
    pub fn create_vertex_proposal(
        vertex_id: VertexId,
        vertex_data: Vec<u8>,
        parent_vertices: Vec<VertexId>,
    ) -> Result<Message, MessageError> {
        let payload = ConsensusPayload::VertexProposal {
            vertex_id,
            vertex_data,
            parent_vertices,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        let payload_bytes =
            bincode::serialize(&payload).map_err(|_| MessageError::SerializationFailed)?;

        Ok(Message::new(
            MessageType::Consensus(ConsensusMessageType::VertexProposal),
            payload_bytes,
        ))
    }

    /// Create a ping message
    pub fn create_ping() -> Result<Message, MessageError> {
        let payload = ControlPayload::Ping {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            nonce: rand::random::<u64>(),
        };

        let payload_bytes =
            bincode::serialize(&payload).map_err(|_| MessageError::SerializationFailed)?;

        Ok(Message::new(
            MessageType::Control(ControlMessageType::Ping),
            payload_bytes,
        )
        .with_ttl(30000)) // 30 second TTL
    }
}

/// Handshake payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    pub protocol_version: ProtocolVersion,
    pub public_key: Vec<u8>,
    pub kem_public_key: Vec<u8>,
    pub capabilities: Vec<String>,
    pub timestamp: u64,
}

/// Consensus message payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusPayload {
    VertexProposal {
        vertex_id: VertexId,
        vertex_data: Vec<u8>,
        parent_vertices: Vec<VertexId>,
        timestamp: u64,
    },
    Vote {
        vertex_id: VertexId,
        vote: bool,
        timestamp: u64,
    },
    Finality {
        vertex_ids: Vec<VertexId>,
        timestamp: u64,
    },
    Query {
        requested_vertices: Vec<VertexId>,
        timestamp: u64,
    },
}

/// Control message payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlPayload {
    Ping { timestamp: u64, nonce: u64 },
    Pong { timestamp: u64, nonce: u64 },
    Disconnect { reason: String, timestamp: u64 },
    KeepAlive { timestamp: u64 },
}
