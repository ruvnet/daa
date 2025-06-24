//! Request signing and verification for QuDAG MCP security.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use blake3;
use hex;
use zeroize::{Zeroize, ZeroizeOnDrop};
use tracing::{debug, warn};

use crate::error::{McpError, McpResult};

/// Request signer for integrity verification
pub struct RequestSigner {
    /// Signing key
    signing_key: SigningKey,
    
    /// Signature algorithm
    algorithm: SignatureAlgorithm,
    
    /// Configuration
    config: SigningConfig,
}

/// Signature verifier for request integrity
pub struct SignatureVerifier {
    /// Verification keys (multiple keys for rotation)
    verification_keys: HashMap<String, VerificationKey>,
    
    /// Default algorithm
    algorithm: SignatureAlgorithm,
    
    /// Configuration
    config: VerificationConfig,
}

/// Signing configuration
#[derive(Debug, Clone)]
pub struct SigningConfig {
    /// Include timestamp in signature
    pub include_timestamp: bool,
    
    /// Include nonce for uniqueness
    pub include_nonce: bool,
    
    /// Signature expiration time in seconds
    pub signature_expiration: u64,
    
    /// Key ID for identification
    pub key_id: String,
    
    /// Additional headers to include in signature
    pub signed_headers: Vec<String>,
}

/// Verification configuration
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Allow clock skew in seconds
    pub clock_skew_tolerance: u64,
    
    /// Require timestamp in signature
    pub require_timestamp: bool,
    
    /// Require nonce in signature
    pub require_nonce: bool,
    
    /// Maximum signature age in seconds
    pub max_signature_age: u64,
    
    /// Required headers in signature
    pub required_headers: Vec<String>,
}

/// Signature algorithms supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// HMAC with BLAKE3
    HmacBlake3,
    
    /// HMAC with SHA-256
    HmacSha256,
    
    /// Ed25519 digital signature
    Ed25519,
    
    /// ECDSA with P-256
    EcdsaP256,
}

/// Signing key material
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SigningKey {
    /// Key material
    key_data: Vec<u8>,
    
    /// Key type
    key_type: KeyType,
    
    /// Key ID
    key_id: String,
    
    /// Key creation time
    created_at: SystemTime,
}

/// Verification key material
#[derive(Debug, Clone)]
pub struct VerificationKey {
    /// Key material (public key for asymmetric, shared key for symmetric)
    key_data: Vec<u8>,
    
    /// Key type
    key_type: KeyType,
    
    /// Key ID
    key_id: String,
    
    /// Key validity period
    valid_from: SystemTime,
    valid_until: Option<SystemTime>,
}

/// Key type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum KeyType {
    /// Symmetric key for HMAC
    Symmetric,
    
    /// Ed25519 private/public key pair
    Ed25519,
    
    /// ECDSA P-256 private/public key pair
    EcdsaP256,
}

/// Signed request container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedRequest {
    /// Original request data
    pub data: Vec<u8>,
    
    /// Signature
    pub signature: String,
    
    /// Signature metadata
    pub metadata: SignatureMetadata,
    
    /// Signed headers
    pub signed_headers: HashMap<String, String>,
}

/// Signature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    /// Signature algorithm
    pub algorithm: SignatureAlgorithm,
    
    /// Key ID used for signing
    pub key_id: String,
    
    /// Timestamp when signature was created
    pub timestamp: Option<u64>,
    
    /// Nonce for uniqueness
    pub nonce: Option<String>,
    
    /// Signature version
    pub version: String,
    
    /// Additional metadata
    pub extra: HashMap<String, String>,
}

/// Signature verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether signature is valid
    pub valid: bool,
    
    /// Verification details
    pub details: VerificationDetails,
    
    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,
    
    /// Key used for verification
    pub key_id: Option<String>,
}

/// Detailed verification information
#[derive(Debug, Clone)]
pub struct VerificationDetails {
    /// Signature algorithm used
    pub algorithm: SignatureAlgorithm,
    
    /// Timestamp verification result
    pub timestamp_valid: bool,
    
    /// Nonce verification result
    pub nonce_valid: bool,
    
    /// Header verification result
    pub headers_valid: bool,
    
    /// Signature age in seconds
    pub signature_age: Option<u64>,
    
    /// Clock skew in seconds
    pub clock_skew: Option<i64>,
}

impl RequestSigner {
    /// Create new request signer with HMAC-BLAKE3
    pub fn new() -> McpResult<Self> {
        let key_data = Self::generate_symmetric_key(32)?;
        let key_id = Self::generate_key_id();
        
        let signing_key = SigningKey {
            key_data,
            key_type: KeyType::Symmetric,
            key_id: key_id.clone(),
            created_at: SystemTime::now(),
        };
        
        let config = SigningConfig {
            include_timestamp: true,
            include_nonce: true,
            signature_expiration: 300, // 5 minutes
            key_id,
            signed_headers: vec![
                "content-type".to_string(),
                "user-agent".to_string(),
                "authorization".to_string(),
            ],
        };
        
        Ok(Self {
            signing_key,
            algorithm: SignatureAlgorithm::HmacBlake3,
            config,
        })
    }
    
    /// Create signer with specific key
    pub fn with_key(key_data: Vec<u8>, key_type: KeyType, key_id: String) -> McpResult<Self> {
        let signing_key = SigningKey {
            key_data,
            key_type: key_type.clone(),
            key_id: key_id.clone(),
            created_at: SystemTime::now(),
        };
        
        let algorithm = match key_type {
            KeyType::Symmetric => SignatureAlgorithm::HmacBlake3,
            KeyType::Ed25519 => SignatureAlgorithm::Ed25519,
            KeyType::EcdsaP256 => SignatureAlgorithm::EcdsaP256,
        };
        
        let config = SigningConfig {
            include_timestamp: true,
            include_nonce: true,
            signature_expiration: 300,
            key_id,
            signed_headers: vec!["content-type".to_string()],
        };
        
        Ok(Self {
            signing_key,
            algorithm,
            config,
        })
    }
    
    /// Sign request data
    pub async fn sign_request(&self, data: &[u8]) -> McpResult<SignedRequest> {
        self.sign_request_with_headers(data, &HashMap::new()).await
    }
    
    /// Sign request with specific headers
    pub async fn sign_request_with_headers(
        &self,
        data: &[u8],
        headers: &HashMap<String, String>,
    ) -> McpResult<SignedRequest> {
        let timestamp = if self.config.include_timestamp {
            Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
        } else {
            None
        };
        
        let nonce = if self.config.include_nonce {
            Some(Self::generate_nonce())
        } else {
            None
        };
        
        // Extract signed headers
        let mut signed_headers = HashMap::new();
        for header_name in &self.config.signed_headers {
            if let Some(header_value) = headers.get(header_name) {
                signed_headers.insert(header_name.clone(), header_value.clone());
            }
        }
        
        let metadata = SignatureMetadata {
            algorithm: self.algorithm.clone(),
            key_id: self.config.key_id.clone(),
            timestamp,
            nonce: nonce.clone(),
            version: "1.0".to_string(),
            extra: HashMap::new(),
        };
        
        // Create signature payload
        let signature_payload = self.create_signature_payload(data, &metadata, &signed_headers)?;
        
        // Generate signature
        let signature = self.generate_signature(&signature_payload)?;
        
        debug!("Request signed with key ID: {}", self.config.key_id);
        
        Ok(SignedRequest {
            data: data.to_vec(),
            signature,
            metadata,
            signed_headers,
        })
    }
    
    /// Verify signed request
    pub async fn verify_request(&self, signed_request: &SignedRequest) -> McpResult<bool> {
        // Create signature payload
        let signature_payload = self.create_signature_payload(
            &signed_request.data,
            &signed_request.metadata,
            &signed_request.signed_headers,
        )?;
        
        // Verify signature
        let valid = self.verify_signature(&signature_payload, &signed_request.signature)?;
        
        if valid {
            // Additional validations
            if let Some(timestamp) = signed_request.metadata.timestamp {
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                let age = now.saturating_sub(timestamp);
                
                if age > self.config.signature_expiration {
                    warn!("Signature expired: {} seconds old", age);
                    return Ok(false);
                }
            }
        }
        
        Ok(valid)
    }
    
    /// Create signature payload for signing/verification
    fn create_signature_payload(
        &self,
        data: &[u8],
        metadata: &SignatureMetadata,
        headers: &HashMap<String, String>,
    ) -> McpResult<Vec<u8>> {
        let mut payload = Vec::new();
        
        // Add algorithm
        payload.extend_from_slice(format!("{:?}", metadata.algorithm).as_bytes());
        payload.push(b'\n');
        
        // Add key ID
        payload.extend_from_slice(metadata.key_id.as_bytes());
        payload.push(b'\n');
        
        // Add timestamp if present
        if let Some(timestamp) = metadata.timestamp {
            payload.extend_from_slice(timestamp.to_string().as_bytes());
        }
        payload.push(b'\n');
        
        // Add nonce if present
        if let Some(nonce) = &metadata.nonce {
            payload.extend_from_slice(nonce.as_bytes());
        }
        payload.push(b'\n');
        
        // Add signed headers (sorted for consistency)
        let mut sorted_headers: Vec<_> = headers.iter().collect();
        sorted_headers.sort_by_key(|(k, _)| *k);
        
        for (name, value) in sorted_headers {
            payload.extend_from_slice(name.as_bytes());
            payload.push(b':');
            payload.extend_from_slice(value.as_bytes());
            payload.push(b'\n');
        }
        
        // Add request data
        payload.extend_from_slice(data);
        
        Ok(payload)
    }
    
    /// Generate signature for payload
    fn generate_signature(&self, payload: &[u8]) -> McpResult<String> {
        match self.algorithm {
            SignatureAlgorithm::HmacBlake3 => {
                let signature = self.hmac_blake3(&self.signing_key.key_data, payload);
                Ok(hex::encode(signature))
            }
            SignatureAlgorithm::HmacSha256 => {
                let signature = self.hmac_sha256(&self.signing_key.key_data, payload)?;
                Ok(hex::encode(signature))
            }
            _ => Err(McpError::crypto("Unsupported signature algorithm")),
        }
    }
    
    /// Verify signature for payload
    fn verify_signature(&self, payload: &[u8], signature: &str) -> McpResult<bool> {
        let expected_signature = self.generate_signature(payload)?;
        Ok(constant_time_eq::constant_time_eq(
            signature.as_bytes(),
            expected_signature.as_bytes(),
        ))
    }
    
    /// HMAC with BLAKE3
    fn hmac_blake3(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new_keyed(
            &key[..32].try_into().unwrap_or([0u8; 32])
        );
        hasher.update(data);
        hasher.finalize().as_bytes().to_vec()
    }
    
    /// HMAC with SHA-256
    fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> McpResult<Vec<u8>> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|e| McpError::crypto(format!("HMAC key error: {}", e)))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }
    
    /// Generate symmetric key
    fn generate_symmetric_key(length: usize) -> McpResult<Vec<u8>> {
        use rand::RngCore;
        let mut key = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut key);
        Ok(key)
    }
    
    /// Generate key ID
    fn generate_key_id() -> String {
        let mut id_bytes = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut id_bytes);
        hex::encode(id_bytes)
    }
    
    /// Generate nonce
    fn generate_nonce() -> String {
        let mut nonce_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        hex::encode(nonce_bytes)
    }
    
    /// Get signing key ID
    pub fn get_key_id(&self) -> &str {
        &self.config.key_id
    }
    
    /// Get verification key for sharing
    pub fn get_verification_key(&self) -> VerificationKey {
        VerificationKey {
            key_data: self.signing_key.key_data.clone(),
            key_type: self.signing_key.key_type.clone(),
            key_id: self.signing_key.key_id.clone(),
            valid_from: self.signing_key.created_at,
            valid_until: None,
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: SigningConfig) {
        self.config = config;
    }
}

impl SignatureVerifier {
    /// Create new signature verifier
    pub fn new() -> Self {
        Self {
            verification_keys: HashMap::new(),
            algorithm: SignatureAlgorithm::HmacBlake3,
            config: VerificationConfig {
                clock_skew_tolerance: 60, // 1 minute
                require_timestamp: true,
                require_nonce: false,
                max_signature_age: 300, // 5 minutes
                required_headers: Vec::new(),
            },
        }
    }
    
    /// Add verification key
    pub fn add_key(&mut self, key: VerificationKey) {
        self.verification_keys.insert(key.key_id.clone(), key);
    }
    
    /// Remove verification key
    pub fn remove_key(&mut self, key_id: &str) {
        self.verification_keys.remove(key_id);
    }
    
    /// Verify signed request with detailed result
    pub async fn verify_request_detailed(&self, signed_request: &SignedRequest) -> McpResult<VerificationResult> {
        let key_id = &signed_request.metadata.key_id;
        
        // Find verification key
        let verification_key = self.verification_keys.get(key_id)
            .ok_or_else(|| McpError::crypto(format!("Unknown key ID: {}", key_id)))?;
        
        let mut warnings = Vec::new();
        let mut details = VerificationDetails {
            algorithm: signed_request.metadata.algorithm.clone(),
            timestamp_valid: true,
            nonce_valid: true,
            headers_valid: true,
            signature_age: None,
            clock_skew: None,
        };
        
        // Check key validity
        let now = SystemTime::now();
        if now < verification_key.valid_from {
            return Ok(VerificationResult {
                valid: false,
                details,
                warnings,
                key_id: Some(key_id.clone()),
            });
        }
        
        if let Some(valid_until) = verification_key.valid_until {
            if now > valid_until {
                return Ok(VerificationResult {
                    valid: false,
                    details,
                    warnings,
                    key_id: Some(key_id.clone()),
                });
            }
        }
        
        // Verify timestamp
        if let Some(timestamp) = signed_request.metadata.timestamp {
            let now_timestamp = now.duration_since(UNIX_EPOCH)?.as_secs();
            let age = now_timestamp.saturating_sub(timestamp);
            let clock_skew = (now_timestamp as i64) - (timestamp as i64);
            
            details.signature_age = Some(age);
            details.clock_skew = Some(clock_skew);
            
            if age > self.config.max_signature_age {
                details.timestamp_valid = false;
            }
            
            if clock_skew.abs() > self.config.clock_skew_tolerance as i64 {
                warnings.push(format!("Clock skew detected: {} seconds", clock_skew));
            }
        } else if self.config.require_timestamp {
            details.timestamp_valid = false;
        }
        
        // Verify nonce
        if signed_request.metadata.nonce.is_none() && self.config.require_nonce {
            details.nonce_valid = false;
        }
        
        // Verify required headers
        for required_header in &self.config.required_headers {
            if !signed_request.signed_headers.contains_key(required_header) {
                details.headers_valid = false;
                warnings.push(format!("Missing required header: {}", required_header));
            }
        }
        
        // Create temporary signer for verification
        let temp_signer = RequestSigner::with_key(
            verification_key.key_data.clone(),
            verification_key.key_type.clone(),
            verification_key.key_id.clone(),
        )?;
        
        // Verify signature
        let signature_valid = temp_signer.verify_request(signed_request).await?;
        
        let overall_valid = signature_valid 
            && details.timestamp_valid 
            && details.nonce_valid 
            && details.headers_valid;
        
        Ok(VerificationResult {
            valid: overall_valid,
            details,
            warnings,
            key_id: Some(key_id.clone()),
        })
    }
    
    /// Simple verification (returns only boolean)
    pub async fn verify_request(&self, signed_request: &SignedRequest) -> McpResult<bool> {
        let result = self.verify_request_detailed(signed_request).await?;
        Ok(result.valid)
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: VerificationConfig) {
        self.config = config;
    }
    
    /// Get all key IDs
    pub fn get_key_ids(&self) -> Vec<String> {
        self.verification_keys.keys().cloned().collect()
    }
}

impl SignedRequest {
    /// Create unsigned request (for testing or when signing is disabled)
    pub fn unsigned(data: Vec<u8>) -> Self {
        Self {
            data,
            signature: String::new(),
            metadata: SignatureMetadata {
                algorithm: SignatureAlgorithm::HmacBlake3,
                key_id: "unsigned".to_string(),
                timestamp: None,
                nonce: None,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            signed_headers: HashMap::new(),
        }
    }
    
    /// Check if request is actually signed
    pub fn is_signed(&self) -> bool {
        !self.signature.is_empty() && self.metadata.key_id != "unsigned"
    }
    
    /// Get request size
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> McpResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| McpError::crypto(format!("Serialization failed: {}", e)))
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> McpResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| McpError::crypto(format!("Deserialization failed: {}", e)))
    }
}

impl Default for SigningConfig {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_nonce: true,
            signature_expiration: 300,
            key_id: "default".to_string(),
            signed_headers: vec!["content-type".to_string()],
        }
    }
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            clock_skew_tolerance: 60,
            require_timestamp: true,
            require_nonce: false,
            max_signature_age: 300,
            required_headers: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_request_signing_and_verification() {
        let signer = RequestSigner::new().unwrap();
        let request_data = b"Test request data";
        
        // Sign request
        let signed_request = signer.sign_request(request_data).await.unwrap();
        assert!(signed_request.is_signed());
        assert!(!signed_request.signature.is_empty());
        
        // Verify request
        let is_valid = signer.verify_request(&signed_request).await.unwrap();
        assert!(is_valid);
    }
    
    #[tokio::test]
    async fn test_signature_verifier() {
        let signer = RequestSigner::new().unwrap();
        let mut verifier = SignatureVerifier::new();
        
        // Add verification key
        let verification_key = signer.get_verification_key();
        verifier.add_key(verification_key);
        
        let request_data = b"Test request for verifier";
        let signed_request = signer.sign_request(request_data).await.unwrap();
        
        // Verify with verifier
        let is_valid = verifier.verify_request(&signed_request).await.unwrap();
        assert!(is_valid);
        
        // Detailed verification
        let result = verifier.verify_request_detailed(&signed_request).await.unwrap();
        assert!(result.valid);
        assert!(result.details.timestamp_valid);
        assert_eq!(result.key_id, Some(signer.get_key_id().to_string()));
    }
    
    #[tokio::test]
    async fn test_request_with_headers() {
        let signer = RequestSigner::new().unwrap();
        let request_data = b"Test request with headers";
        
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("user-agent".to_string(), "QuDAG-MCP/1.0".to_string());
        
        let signed_request = signer.sign_request_with_headers(request_data, &headers).await.unwrap();
        
        // Should include signed headers
        assert!(signed_request.signed_headers.contains_key("content-type"));
        assert!(signed_request.signed_headers.contains_key("user-agent"));
        
        // Verify request
        let is_valid = signer.verify_request(&signed_request).await.unwrap();
        assert!(is_valid);
    }
    
    #[tokio::test]
    async fn test_signature_expiration() {
        let mut config = SigningConfig::default();
        config.signature_expiration = 1; // 1 second expiration
        
        let mut signer = RequestSigner::new().unwrap();
        signer.update_config(config);
        
        let request_data = b"Test expiring signature";
        let signed_request = signer.sign_request(request_data).await.unwrap();
        
        // Should be valid immediately
        let is_valid = signer.verify_request(&signed_request).await.unwrap();
        assert!(is_valid);
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Should be invalid after expiration
        let is_valid = signer.verify_request(&signed_request).await.unwrap();
        assert!(!is_valid);
    }
    
    #[tokio::test]
    async fn test_unsigned_request() {
        let unsigned = SignedRequest::unsigned(b"test data".to_vec());
        assert!(!unsigned.is_signed());
        assert_eq!(unsigned.signature, "");
        assert_eq!(unsigned.metadata.key_id, "unsigned");
    }
    
    #[tokio::test]
    async fn test_signature_tampering() {
        let signer = RequestSigner::new().unwrap();
        let request_data = b"Original request data";
        
        let mut signed_request = signer.sign_request(request_data).await.unwrap();
        
        // Tamper with data
        signed_request.data = b"Tampered request data".to_vec();
        
        // Verification should fail
        let is_valid = signer.verify_request(&signed_request).await.unwrap();
        assert!(!is_valid);
    }
    
    #[test]
    fn test_serialization() {
        let signed_request = SignedRequest::unsigned(b"test data".to_vec());
        
        let serialized = signed_request.to_bytes().unwrap();
        let deserialized = SignedRequest::from_bytes(&serialized).unwrap();
        
        assert_eq!(signed_request.data, deserialized.data);
        assert_eq!(signed_request.signature, deserialized.signature);
        assert_eq!(signed_request.metadata.key_id, deserialized.metadata.key_id);
    }
}