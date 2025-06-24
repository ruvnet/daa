//! Quantum-resistant handshake procedures for QuDAG protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, error, info};
use uuid::Uuid;

use qudag_crypto::{
    Ciphertext as KemCiphertext, KeyPair as KemKeyPair, MlDsaKeyPair, MlDsaPublicKey, MlKem768,
    PublicKey as KemPublicKey, SecretKey, SharedSecret,
};
use rand;

use crate::message::{HandshakeType, Message, MessageError, MessageType, ProtocolVersion};
use crate::state::{ProtocolStateMachine, StateError};

/// Handshake-related errors
#[derive(Debug, Error)]
pub enum HandshakeError {
    /// Cryptographic operation failed
    #[error("Cryptographic operation failed: {reason}")]
    CryptoError { reason: String },

    /// Invalid handshake message
    #[error("Invalid handshake message: {reason}")]
    InvalidMessage { reason: String },

    /// Handshake timeout
    #[error("Handshake timed out after {timeout:?}")]
    Timeout { timeout: Duration },

    /// Protocol version mismatch
    #[error("Protocol version mismatch: expected {expected:?}, got {actual:?}")]
    VersionMismatch {
        expected: ProtocolVersion,
        actual: ProtocolVersion,
    },

    /// Unsupported capabilities
    #[error("Unsupported capabilities: {capabilities:?}")]
    UnsupportedCapabilities { capabilities: Vec<String> },

    /// Invalid peer credentials
    #[error("Invalid peer credentials")]
    InvalidCredentials,

    /// State machine error
    #[error("State machine error: {0}")]
    StateMachine(#[from] StateError),

    /// Message error
    #[error("Message error: {0}")]
    Message(#[from] MessageError),

    /// Handshake already in progress
    #[error("Handshake already in progress with session {session_id}")]
    HandshakeInProgress { session_id: Uuid },

    /// Replay attack detected
    #[error("Replay attack detected: timestamp {timestamp} is too old")]
    ReplayAttack { timestamp: u64 },
}

/// Handshake configuration
#[derive(Debug, Clone)]
pub struct HandshakeConfig {
    /// Timeout for handshake completion
    pub timeout: Duration,
    /// Supported protocol versions
    pub supported_versions: Vec<ProtocolVersion>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Optional capabilities
    pub optional_capabilities: Vec<String>,
    /// Maximum timestamp skew allowed (to prevent replay attacks)
    pub max_timestamp_skew: Duration,
    /// Enable mutual authentication
    pub mutual_auth: bool,
}

impl Default for HandshakeConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            supported_versions: vec![ProtocolVersion::CURRENT],
            required_capabilities: vec![
                "dag-consensus".to_string(),
                "quantum-resistant-crypto".to_string(),
            ],
            optional_capabilities: vec![
                "anonymous-routing".to_string(),
                "dark-addressing".to_string(),
            ],
            max_timestamp_skew: Duration::from_secs(300), // 5 minutes
            mutual_auth: true,
        }
    }
}

/// Handshake session state
#[derive(Debug)]
pub struct HandshakeSession {
    /// Session identifier
    pub session_id: Uuid,
    /// Peer identifier (if known)
    pub peer_id: Option<Vec<u8>>,
    /// Handshake state
    pub state: HandshakeSessionState,
    /// Protocol version negotiated
    pub negotiated_version: Option<ProtocolVersion>,
    /// Peer capabilities
    pub peer_capabilities: Vec<String>,
    /// Our ephemeral keys for this session
    pub our_keys: HandshakeKeys,
    /// Peer's public keys
    pub peer_keys: Option<PeerKeys>,
    /// Shared secrets
    pub shared_secrets: Option<SharedSecrets>,
    /// Session start time
    pub started_at: SystemTime,
    /// Last activity
    pub last_activity: SystemTime,
    /// Nonce for this session
    pub nonce: u64,
}

/// Handshake session state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeSessionState {
    /// Waiting to start handshake
    Waiting,
    /// Sent initial handshake message
    InitSent,
    /// Received initial handshake message
    InitReceived,
    /// Sent handshake response
    ResponseSent,
    /// Received handshake response
    ResponseReceived,
    /// Handshake completed successfully
    Completed,
    /// Handshake failed
    Failed,
}

/// Handshake keys for a session
#[derive(Debug)]
pub struct HandshakeKeys {
    /// ML-DSA keypair for authentication
    pub signature_keypair: MlDsaKeyPair,
    /// ML-KEM keypair for key exchange
    pub kem_keypair: KemKeyPair,
}

/// Peer's public keys
#[derive(Debug, Clone)]
pub struct PeerKeys {
    /// Peer's ML-DSA public key
    pub signature_public_key: MlDsaPublicKey,
    /// Peer's ML-KEM public key
    pub kem_public_key: KemPublicKey,
}

/// Derived shared secrets
#[derive(Debug, Clone)]
pub struct SharedSecrets {
    /// Shared secret from ML-KEM
    pub kem_shared_secret: SharedSecret,
    /// Derived encryption key
    pub encryption_key: Vec<u8>,
    /// Derived MAC key
    pub mac_key: Vec<u8>,
    /// Session identifier
    pub session_key: Vec<u8>,
}

/// Handshake message payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeMessagePayload {
    /// Initial handshake message
    Init {
        protocol_version: ProtocolVersion,
        supported_versions: Vec<ProtocolVersion>,
        capabilities: Vec<String>,
        signature_public_key: Vec<u8>,
        kem_public_key: Vec<u8>,
        nonce: u64,
        timestamp: u64,
    },
    /// Handshake response message
    Response {
        protocol_version: ProtocolVersion,
        capabilities: Vec<String>,
        signature_public_key: Vec<u8>,
        kem_ciphertext: Vec<u8>, // Encapsulated shared secret
        nonce: u64,
        timestamp: u64,
    },
    /// Handshake completion message
    Complete { session_id: Vec<u8>, timestamp: u64 },
    /// Version negotiation message
    VersionNegotiation {
        supported_versions: Vec<ProtocolVersion>,
        preferred_version: ProtocolVersion,
    },
}

/// Handshake coordinator
pub struct HandshakeCoordinator {
    /// Handshake configuration
    config: HandshakeConfig,
    /// Active handshake sessions
    sessions: HashMap<Uuid, HandshakeSession>,
    /// Our long-term identity keys
    #[allow(dead_code)]
    identity_keys: HandshakeKeys,
    /// Protocol state machine
    state_machine: ProtocolStateMachine,
}

impl HandshakeCoordinator {
    /// Create a new handshake coordinator
    pub fn new(
        config: HandshakeConfig,
        identity_keys: HandshakeKeys,
        state_machine: ProtocolStateMachine,
    ) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            identity_keys,
            state_machine,
        }
    }

    /// Generate new handshake keys
    pub fn generate_keys() -> Result<HandshakeKeys, HandshakeError> {
        // Generate ML-DSA keypair
        let signature_keypair = MlDsaKeyPair::generate(&mut rand::thread_rng()).map_err(|e| {
            HandshakeError::CryptoError {
                reason: format!("Failed to generate ML-DSA keypair: {:?}", e),
            }
        })?;

        // Generate ML-KEM keypair
        let (kem_public_key, kem_secret_key) =
            MlKem768::keygen().map_err(|e| HandshakeError::CryptoError {
                reason: format!("Failed to generate ML-KEM keypair: {:?}", e),
            })?;
        let kem_keypair = KemKeyPair {
            public_key: kem_public_key.as_bytes().to_vec(),
            secret_key: kem_secret_key.as_bytes().to_vec(),
        };

        Ok(HandshakeKeys {
            signature_keypair,
            kem_keypair,
        })
    }

    /// Initiate handshake with a peer
    pub fn initiate_handshake(
        &mut self,
        peer_id: Option<Vec<u8>>,
    ) -> Result<(Uuid, Message), HandshakeError> {
        // Generate ephemeral keys for this session
        let session_keys = Self::generate_keys()?;
        let session_id = Uuid::new_v4();
        let nonce = rand::random::<u64>();

        // Create handshake session
        let session = HandshakeSession {
            session_id,
            peer_id,
            state: HandshakeSessionState::Waiting,
            negotiated_version: None,
            peer_capabilities: Vec::new(),
            our_keys: session_keys,
            peer_keys: None,
            shared_secrets: None,
            started_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            nonce,
        };

        // Create initial handshake message
        let payload = HandshakeMessagePayload::Init {
            protocol_version: ProtocolVersion::CURRENT,
            supported_versions: self.config.supported_versions.clone(),
            capabilities: [
                self.config.required_capabilities.clone(),
                self.config.optional_capabilities.clone(),
            ]
            .concat(),
            signature_public_key: session.our_keys.signature_keypair.public_key().to_vec(),
            kem_public_key: session.our_keys.kem_keypair.public_key().to_vec(),
            nonce,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        let payload_bytes =
            bincode::serialize(&payload).map_err(|e| HandshakeError::InvalidMessage {
                reason: format!("Failed to serialize handshake init: {:?}", e),
            })?;

        let mut message = Message::new(MessageType::Handshake(HandshakeType::Init), payload_bytes);

        // Sign the message
        message.sign(&session.our_keys.signature_keypair)?;

        // Update session state
        let mut session = session;
        session.state = HandshakeSessionState::InitSent;
        session.last_activity = SystemTime::now();

        // Store session
        self.sessions.insert(session_id, session);

        info!("Initiated handshake session: {}", session_id);
        Ok((session_id, message))
    }

    /// Process incoming handshake message
    pub fn process_handshake_message(
        &mut self,
        message: &Message,
        session_id: Option<Uuid>,
    ) -> Result<Option<Message>, HandshakeError> {
        // Validate message timestamp to prevent replay attacks
        self.validate_timestamp(message)?;

        match &message.msg_type {
            MessageType::Handshake(HandshakeType::Init) => self.process_handshake_init(message),
            MessageType::Handshake(HandshakeType::Response) => {
                self.process_handshake_response(message, session_id)
            }
            MessageType::Handshake(HandshakeType::Complete) => {
                self.process_handshake_complete(message, session_id)
            }
            MessageType::Handshake(HandshakeType::VersionNegotiation) => {
                self.process_version_negotiation(message, session_id)
            }
            _ => Err(HandshakeError::InvalidMessage {
                reason: "Not a handshake message".to_string(),
            }),
        }
    }

    /// Process handshake init message
    fn process_handshake_init(
        &mut self,
        message: &Message,
    ) -> Result<Option<Message>, HandshakeError> {
        let payload: HandshakeMessagePayload =
            bincode::deserialize(&message.payload).map_err(|e| HandshakeError::InvalidMessage {
                reason: format!("Failed to deserialize handshake init: {:?}", e),
            })?;

        if let HandshakeMessagePayload::Init {
            protocol_version,
            supported_versions,
            capabilities,
            signature_public_key,
            kem_public_key,
            nonce,
            timestamp: _,
        } = payload
        {
            // Verify protocol version compatibility
            let negotiated_version =
                self.negotiate_version(&supported_versions, &protocol_version)?;

            // Verify required capabilities
            self.verify_capabilities(&capabilities)?;

            // Parse peer keys
            let peer_signature_key =
                MlDsaPublicKey::from_bytes(&signature_public_key).map_err(|e| {
                    HandshakeError::CryptoError {
                        reason: format!("Invalid peer signature key: {:?}", e),
                    }
                })?;

            let peer_kem_key = KemPublicKey::from_bytes(&kem_public_key).map_err(|e| {
                HandshakeError::CryptoError {
                    reason: format!("Invalid peer KEM key: {:?}", e),
                }
            })?;

            // Verify message signature
            if !message.verify(&peer_signature_key)? {
                return Err(HandshakeError::InvalidCredentials);
            }

            // Generate ephemeral keys for this session
            let session_keys = Self::generate_keys()?;
            let session_id = Uuid::new_v4();
            let our_nonce = rand::random::<u64>();

            // Perform key exchange
            let (kem_ciphertext, shared_secret) =
                MlKem768::encapsulate(&peer_kem_key).map_err(|e| HandshakeError::CryptoError {
                    reason: format!("KEM encapsulation failed: {:?}", e),
                })?;

            // Derive session keys
            let shared_secrets = self.derive_session_keys(&shared_secret, nonce, our_nonce)?;

            // Create handshake session
            let session = HandshakeSession {
                session_id,
                peer_id: None, // Will be set later if needed
                state: HandshakeSessionState::InitReceived,
                negotiated_version: Some(negotiated_version.clone()),
                peer_capabilities: capabilities,
                our_keys: session_keys,
                peer_keys: Some(PeerKeys {
                    signature_public_key: peer_signature_key,
                    kem_public_key: peer_kem_key,
                }),
                shared_secrets: Some(shared_secrets),
                started_at: SystemTime::now(),
                last_activity: SystemTime::now(),
                nonce: our_nonce,
            };

            // Create response message
            let response_payload = HandshakeMessagePayload::Response {
                protocol_version: negotiated_version,
                capabilities: [
                    self.config.required_capabilities.clone(),
                    self.config.optional_capabilities.clone(),
                ]
                .concat(),
                signature_public_key: session.our_keys.signature_keypair.public_key().to_vec(),
                kem_ciphertext: kem_ciphertext.as_bytes().to_vec(),
                nonce: our_nonce,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };

            let response_bytes = bincode::serialize(&response_payload).map_err(|e| {
                HandshakeError::InvalidMessage {
                    reason: format!("Failed to serialize handshake response: {:?}", e),
                }
            })?;

            let mut response_message = Message::new(
                MessageType::Handshake(HandshakeType::Response),
                response_bytes,
            );

            // Sign the response
            response_message.sign(&session.our_keys.signature_keypair)?;

            // Update session state
            let mut session = session;
            session.state = HandshakeSessionState::ResponseSent;
            session.last_activity = SystemTime::now();

            // Store session
            self.sessions.insert(session_id, session);

            // Update protocol state machine
            self.state_machine
                .process_message(message, Some(session_id))?;

            info!(
                "Processed handshake init, sending response for session: {}",
                session_id
            );
            Ok(Some(response_message))
        } else {
            Err(HandshakeError::InvalidMessage {
                reason: "Expected handshake init payload".to_string(),
            })
        }
    }

    /// Process handshake response message
    fn process_handshake_response(
        &mut self,
        message: &Message,
        session_id: Option<Uuid>,
    ) -> Result<Option<Message>, HandshakeError> {
        let session_id = session_id.ok_or(HandshakeError::InvalidMessage {
            reason: "Session ID required for handshake response".to_string(),
        })?;

        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or(HandshakeError::InvalidMessage {
                reason: format!("Session not found: {}", session_id),
            })?;

        if session.state != HandshakeSessionState::InitSent {
            return Err(HandshakeError::InvalidMessage {
                reason: format!("Invalid session state for response: {:?}", session.state),
            });
        }

        let payload: HandshakeMessagePayload =
            bincode::deserialize(&message.payload).map_err(|e| HandshakeError::InvalidMessage {
                reason: format!("Failed to deserialize handshake response: {:?}", e),
            })?;

        if let HandshakeMessagePayload::Response {
            protocol_version,
            capabilities,
            signature_public_key,
            kem_ciphertext,
            nonce,
            timestamp: _,
        } = payload
        {
            // Parse peer keys
            let peer_signature_key =
                MlDsaPublicKey::from_bytes(&signature_public_key).map_err(|e| {
                    HandshakeError::CryptoError {
                        reason: format!("Invalid peer signature key: {:?}", e),
                    }
                })?;

            // Verify message signature
            if !message.verify(&peer_signature_key)? {
                return Err(HandshakeError::InvalidCredentials);
            }

            // Verify protocol version
            if !self.config.supported_versions.contains(&protocol_version) {
                return Err(HandshakeError::VersionMismatch {
                    expected: ProtocolVersion::CURRENT,
                    actual: protocol_version,
                });
            }

            // Verify capabilities first
            {
                // Check required capabilities
                for required_cap in &self.config.required_capabilities {
                    if !capabilities.contains(required_cap) {
                        return Err(HandshakeError::UnsupportedCapabilities {
                            capabilities: vec![required_cap.clone()],
                        });
                    }
                }
            }

            // Decapsulate shared secret
            let kem_ciphertext_bytes = KemCiphertext::from_bytes(&kem_ciphertext).map_err(|e| {
                HandshakeError::CryptoError {
                    reason: format!("Invalid KEM ciphertext: {:?}", e),
                }
            })?;

            let secret_key = SecretKey::from_bytes(session.our_keys.kem_keypair.secret_key())
                .map_err(|e| HandshakeError::CryptoError {
                    reason: format!("Invalid secret key: {:?}", e),
                })?;

            let shared_secret =
                MlKem768::decapsulate(&secret_key, &kem_ciphertext_bytes).map_err(|e| {
                    HandshakeError::CryptoError {
                        reason: format!("KEM decapsulation failed: {:?}", e),
                    }
                })?;

            // Derive session keys
            let session_nonce = session.nonce;
            let shared_secrets = {
                let secret_bytes = shared_secret.as_bytes();

                // Combine nonces for key derivation
                let combined_nonce = session_nonce ^ nonce;
                let nonce_bytes = combined_nonce.to_be_bytes();

                // Use HKDF for key derivation (simplified)
                let mut key_material = Vec::new();
                key_material.extend_from_slice(secret_bytes);
                key_material.extend_from_slice(&nonce_bytes);

                // Derive different keys using BLAKE3 with different contexts
                let encryption_key = blake3::keyed_hash(
                    blake3::hash(b"QuDAG-Encryption-Key").as_bytes(),
                    &key_material,
                )
                .as_bytes()
                .to_vec();

                let mac_key =
                    blake3::keyed_hash(blake3::hash(b"QuDAG-MAC-Key").as_bytes(), &key_material)
                        .as_bytes()
                        .to_vec();

                let session_key = blake3::keyed_hash(
                    blake3::hash(b"QuDAG-Session-Key").as_bytes(),
                    &key_material,
                )
                .as_bytes()
                .to_vec();

                SharedSecrets {
                    kem_shared_secret: shared_secret.clone(),
                    encryption_key,
                    mac_key,
                    session_key,
                }
            };

            // Update session
            session.negotiated_version = Some(protocol_version);
            session.peer_capabilities = capabilities;
            session.peer_keys = Some(PeerKeys {
                signature_public_key: peer_signature_key,
                kem_public_key: KemPublicKey::from_bytes(session.our_keys.kem_keypair.public_key())
                    .map_err(|e| HandshakeError::CryptoError {
                        reason: format!("Invalid public key: {:?}", e),
                    })?, // Placeholder
            });
            session.shared_secrets = Some(shared_secrets);
            session.state = HandshakeSessionState::ResponseReceived;
            session.last_activity = SystemTime::now();

            // Create completion message
            let complete_payload = HandshakeMessagePayload::Complete {
                session_id: session_id.as_bytes().to_vec(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };

            let complete_bytes = bincode::serialize(&complete_payload).map_err(|e| {
                HandshakeError::InvalidMessage {
                    reason: format!("Failed to serialize handshake complete: {:?}", e),
                }
            })?;

            let mut complete_message = Message::new(
                MessageType::Handshake(HandshakeType::Complete),
                complete_bytes,
            );

            // Sign the completion message
            complete_message.sign(&session.our_keys.signature_keypair)?;

            // Update session state
            session.state = HandshakeSessionState::Completed;

            // Update protocol state machine
            self.state_machine
                .process_message(message, Some(session_id))?;

            info!(
                "Processed handshake response, sending completion for session: {}",
                session_id
            );
            Ok(Some(complete_message))
        } else {
            Err(HandshakeError::InvalidMessage {
                reason: "Expected handshake response payload".to_string(),
            })
        }
    }

    /// Process handshake complete message
    fn process_handshake_complete(
        &mut self,
        message: &Message,
        session_id: Option<Uuid>,
    ) -> Result<Option<Message>, HandshakeError> {
        let session_id = session_id.ok_or(HandshakeError::InvalidMessage {
            reason: "Session ID required for handshake complete".to_string(),
        })?;

        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or(HandshakeError::InvalidMessage {
                reason: format!("Session not found: {}", session_id),
            })?;

        if session.state != HandshakeSessionState::ResponseSent {
            return Err(HandshakeError::InvalidMessage {
                reason: format!("Invalid session state for complete: {:?}", session.state),
            });
        }

        // Verify message signature
        if let Some(peer_keys) = &session.peer_keys {
            if !message.verify(&peer_keys.signature_public_key)? {
                return Err(HandshakeError::InvalidCredentials);
            }
        }

        // Update session state
        session.state = HandshakeSessionState::Completed;
        session.last_activity = SystemTime::now();

        // Update protocol state machine
        self.state_machine
            .process_message(message, Some(session_id))?;

        info!(
            "Handshake completed successfully for session: {}",
            session_id
        );
        Ok(None) // No response needed
    }

    /// Process version negotiation message
    fn process_version_negotiation(
        &mut self,
        message: &Message,
        _session_id: Option<Uuid>,
    ) -> Result<Option<Message>, HandshakeError> {
        let payload: HandshakeMessagePayload =
            bincode::deserialize(&message.payload).map_err(|e| HandshakeError::InvalidMessage {
                reason: format!("Failed to deserialize version negotiation: {:?}", e),
            })?;

        if let HandshakeMessagePayload::VersionNegotiation {
            supported_versions,
            preferred_version,
        } = payload
        {
            // Find compatible version
            let compatible_version =
                self.negotiate_version(&supported_versions, &preferred_version)?;

            // Update our supported versions if needed
            debug!("Negotiated protocol version: {:?}", compatible_version);

            // No response needed for version negotiation
            Ok(None)
        } else {
            Err(HandshakeError::InvalidMessage {
                reason: "Expected version negotiation payload".to_string(),
            })
        }
    }

    /// Negotiate protocol version
    fn negotiate_version(
        &self,
        peer_versions: &[ProtocolVersion],
        peer_preferred: &ProtocolVersion,
    ) -> Result<ProtocolVersion, HandshakeError> {
        // Try peer's preferred version first
        if self.config.supported_versions.contains(peer_preferred) {
            return Ok(peer_preferred.clone());
        }

        // Find highest compatible version
        for our_version in &self.config.supported_versions {
            for peer_version in peer_versions {
                if our_version.is_compatible(peer_version) {
                    return Ok(our_version.clone());
                }
            }
        }

        Err(HandshakeError::VersionMismatch {
            expected: ProtocolVersion::CURRENT,
            actual: peer_preferred.clone(),
        })
    }

    /// Verify peer capabilities
    fn verify_capabilities(&self, peer_capabilities: &[String]) -> Result<(), HandshakeError> {
        for required_cap in &self.config.required_capabilities {
            if !peer_capabilities.contains(required_cap) {
                return Err(HandshakeError::UnsupportedCapabilities {
                    capabilities: vec![required_cap.clone()],
                });
            }
        }
        Ok(())
    }

    /// Derive session keys from shared secret
    fn derive_session_keys(
        &self,
        shared_secret: &SharedSecret,
        our_nonce: u64,
        peer_nonce: u64,
    ) -> Result<SharedSecrets, HandshakeError> {
        let secret_bytes = shared_secret.as_bytes();

        // Combine nonces for key derivation
        let combined_nonce = our_nonce ^ peer_nonce;
        let nonce_bytes = combined_nonce.to_be_bytes();

        // Use HKDF for key derivation (simplified)
        let mut key_material = Vec::new();
        key_material.extend_from_slice(secret_bytes);
        key_material.extend_from_slice(&nonce_bytes);

        // Derive different keys using BLAKE3 with different contexts
        let encryption_key = blake3::keyed_hash(
            blake3::hash(b"QuDAG-Encryption-Key").as_bytes(),
            &key_material,
        )
        .as_bytes()
        .to_vec();

        let mac_key = blake3::keyed_hash(blake3::hash(b"QuDAG-MAC-Key").as_bytes(), &key_material)
            .as_bytes()
            .to_vec();

        let session_key =
            blake3::keyed_hash(blake3::hash(b"QuDAG-Session-Key").as_bytes(), &key_material)
                .as_bytes()
                .to_vec();

        Ok(SharedSecrets {
            kem_shared_secret: shared_secret.clone(),
            encryption_key,
            mac_key,
            session_key,
        })
    }

    /// Validate message timestamp to prevent replay attacks
    fn validate_timestamp(&self, message: &Message) -> Result<(), HandshakeError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let message_time = message.timestamp;
        let max_skew = self.config.max_timestamp_skew.as_millis() as u64;

        if now > message_time + max_skew || message_time > now + max_skew {
            return Err(HandshakeError::ReplayAttack {
                timestamp: message_time,
            });
        }

        Ok(())
    }

    /// Get handshake session
    pub fn get_session(&self, session_id: &Uuid) -> Option<&HandshakeSession> {
        self.sessions.get(session_id)
    }

    /// Remove completed or failed sessions
    pub fn cleanup_sessions(&mut self) {
        let now = SystemTime::now();
        let timeout = self.config.timeout;

        self.sessions.retain(|_, session| {
            let elapsed = now
                .duration_since(session.started_at)
                .unwrap_or(Duration::ZERO);

            match session.state {
                HandshakeSessionState::Completed | HandshakeSessionState::Failed => false,
                _ => elapsed < timeout,
            }
        });
    }

    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<&HandshakeSession> {
        self.sessions
            .values()
            .filter(|s| {
                !matches!(
                    s.state,
                    HandshakeSessionState::Completed | HandshakeSessionState::Failed
                )
            })
            .collect()
    }

    /// Check if handshake is completed for a session
    pub fn is_handshake_completed(&self, session_id: &Uuid) -> bool {
        self.sessions
            .get(session_id)
            .map(|s| s.state == HandshakeSessionState::Completed)
            .unwrap_or(false)
    }

    /// Get shared secrets for a completed session
    pub fn get_shared_secrets(&self, session_id: &Uuid) -> Option<&SharedSecrets> {
        self.sessions
            .get(session_id)
            .and_then(|s| s.shared_secrets.as_ref())
    }
}
