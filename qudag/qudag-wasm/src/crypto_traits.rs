//! Crypto trait abstractions for cross-platform compatibility
//!
//! This module defines traits that allow the same API to work on both
//! native and WASM targets, with graceful fallbacks and clear error messages.

use std::error::Error;
use std::fmt;

/// Error type for crypto operations
#[derive(Debug, Clone)]
pub enum CryptoAbstractionError {
    /// Operation not supported on this platform
    UnsupportedPlatform(String),
    /// Feature not available (e.g., missing WASM capability)
    FeatureNotAvailable(String),
    /// Underlying crypto operation failed
    CryptoOperationFailed(String),
    /// Invalid key format or size
    InvalidKey(String),
    /// Invalid data format
    InvalidData(String),
}

impl fmt::Display for CryptoAbstractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform(msg) => write!(f, "Platform not supported: {}", msg),
            Self::FeatureNotAvailable(msg) => write!(f, "Feature not available: {}", msg),
            Self::CryptoOperationFailed(msg) => write!(f, "Crypto operation failed: {}", msg),
            Self::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            Self::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl Error for CryptoAbstractionError {}

pub type Result<T> = std::result::Result<T, CryptoAbstractionError>;

/// Public key abstraction
pub trait PublicKey: Clone + Send + Sync {
    /// Get the raw bytes of the public key
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from raw bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
    
    /// Get the algorithm identifier
    fn algorithm(&self) -> &str;
    
    /// Get the key size in bytes
    fn size(&self) -> usize;
}

/// Private/Secret key abstraction
pub trait PrivateKey: Clone + Send + Sync {
    /// Get the raw bytes of the private key (handle with care!)
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from raw bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
    
    /// Securely wipe the key from memory
    fn zeroize(&mut self);
    
    /// Get the algorithm identifier
    fn algorithm(&self) -> &str;
    
    /// Get the key size in bytes
    fn size(&self) -> usize;
}

/// Digital signature abstraction
pub trait Signature: Clone + Send + Sync {
    /// Get the raw bytes of the signature
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from raw bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
    
    /// Get the algorithm identifier
    fn algorithm(&self) -> &str;
    
    /// Get the signature size in bytes
    fn size(&self) -> usize;
}

/// Quantum-resistant digital signature operations
pub trait QuantumResistantSigning {
    type PublicKey: PublicKey;
    type PrivateKey: PrivateKey;
    type Signature: Signature;
    
    /// Generate a new key pair
    fn generate_keypair() -> Result<(Self::PublicKey, Self::PrivateKey)>;
    
    /// Sign a message
    fn sign(message: &[u8], private_key: &Self::PrivateKey) -> Result<Self::Signature>;
    
    /// Verify a signature
    fn verify(
        message: &[u8],
        signature: &Self::Signature,
        public_key: &Self::PublicKey,
    ) -> Result<bool>;
    
    /// Get the algorithm name (e.g., "ML-DSA-65")
    fn algorithm_name() -> &'static str;
    
    /// Check if this algorithm is available on the current platform
    fn is_available() -> bool;
}

/// Ciphertext abstraction for KEM
pub trait Ciphertext: Clone + Send + Sync {
    /// Get the raw bytes of the ciphertext
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from raw bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
    
    /// Get the ciphertext size in bytes
    fn size(&self) -> usize;
}

/// Shared secret abstraction for KEM
pub trait SharedSecret: Clone + Send + Sync {
    /// Get the raw bytes of the shared secret
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from raw bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
    
    /// Securely wipe the secret from memory
    fn zeroize(&mut self);
    
    /// Get the secret size in bytes
    fn size(&self) -> usize;
}

/// Quantum-resistant key encapsulation mechanism
pub trait KeyEncapsulation {
    type PublicKey: PublicKey;
    type PrivateKey: PrivateKey;
    type Ciphertext: Ciphertext;
    type SharedSecret: SharedSecret;
    
    /// Generate a new key pair
    fn generate_keypair() -> Result<(Self::PublicKey, Self::PrivateKey)>;
    
    /// Encapsulate a shared secret using a public key
    fn encapsulate(
        public_key: &Self::PublicKey,
    ) -> Result<(Self::Ciphertext, Self::SharedSecret)>;
    
    /// Decapsulate a shared secret using a private key
    fn decapsulate(
        ciphertext: &Self::Ciphertext,
        private_key: &Self::PrivateKey,
    ) -> Result<Self::SharedSecret>;
    
    /// Get the algorithm name (e.g., "ML-KEM-768")
    fn algorithm_name() -> &'static str;
    
    /// Check if this algorithm is available on the current platform
    fn is_available() -> bool;
}

/// Hash function abstraction
pub trait HashFunction {
    /// Hash data and return the digest
    fn hash(data: &[u8]) -> Vec<u8>;
    
    /// Get the algorithm name (e.g., "BLAKE3")
    fn algorithm_name() -> &'static str;
    
    /// Get the output size in bytes
    fn output_size() -> usize;
    
    /// Check if this hash function is available
    fn is_available() -> bool;
}

/// Streaming hash operations
pub trait StreamingHash: Send + Sync {
    /// Update the hash with new data
    fn update(&mut self, data: &[u8]);
    
    /// Finalize and get the hash digest
    fn finalize(self) -> Vec<u8>;
    
    /// Reset the hasher to initial state
    fn reset(&mut self);
}

/// Factory for creating streaming hashers
pub trait StreamingHashFactory {
    type Hasher: StreamingHash;
    
    /// Create a new streaming hasher
    fn new_hasher() -> Self::Hasher;
}

/// Feature detection trait for runtime capability checking
pub trait CryptoFeatureDetection {
    /// Check if ML-DSA signatures are available
    fn has_ml_dsa() -> bool;
    
    /// Check if ML-KEM is available
    fn has_ml_kem() -> bool;
    
    /// Check if HQC encryption is available
    fn has_hqc() -> bool;
    
    /// Check if BLAKE3 hashing is available
    fn has_blake3() -> bool;
    
    /// Check if quantum fingerprinting is available
    fn has_quantum_fingerprint() -> bool;
    
    /// Get a list of all available crypto features
    fn available_features() -> Vec<&'static str>;
    
    /// Get platform-specific limitations or notes
    fn platform_notes() -> Option<&'static str>;
}

/// Platform-specific crypto provider
pub trait CryptoProvider {
    /// Get the provider name (e.g., "native", "wasm", "wasm-fallback")
    fn name() -> &'static str;
    
    /// Get the provider version
    fn version() -> &'static str;
    
    /// Check if this is a fallback/limited implementation
    fn is_fallback() -> bool;
    
    /// Initialize the crypto provider (if needed)
    fn initialize() -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CryptoAbstractionError::UnsupportedPlatform("WASM32".to_string());
        assert_eq!(err.to_string(), "Platform not supported: WASM32");
        
        let err = CryptoAbstractionError::FeatureNotAvailable("ML-KEM".to_string());
        assert_eq!(err.to_string(), "Feature not available: ML-KEM");
    }
}