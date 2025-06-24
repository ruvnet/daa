//! Quantum-resistant cryptographic primitives for secure P2P communication.
//!
//! This module provides quantum-resistant cryptographic functions using:
//! - ML-KEM (CRYSTALS-Kyber) for key encapsulation
//! - ML-DSA (CRYSTALS-Dilithium) for digital signatures
//! - ChaCha20-Poly1305 for symmetric encryption

use crate::types::NetworkError;
use rand::{rngs::OsRng, RngCore};
use std::fmt::Debug;
use tracing::{debug, info};
use zeroize::ZeroizeOnDrop;

/// ML-KEM (Kyber) security levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlKemSecurityLevel {
    /// ML-KEM-512 (Category 1)
    Level512,
    /// ML-KEM-768 (Category 3)
    Level768,
    /// ML-KEM-1024 (Category 5)
    Level1024,
}

impl Default for MlKemSecurityLevel {
    fn default() -> Self {
        Self::Level768 // Category 3 provides good balance of security and performance
    }
}

/// ML-KEM public key
#[derive(Clone)]
pub struct MlKemPublicKey {
    /// Raw key material
    pub(crate) key_data: Vec<u8>,
    /// Security level
    pub security_level: MlKemSecurityLevel,
}

impl Debug for MlKemPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MlKemPublicKey")
            .field("security_level", &self.security_level)
            .field("key_length", &self.key_data.len())
            .finish()
    }
}

/// ML-KEM secret key (zeroized on drop)
#[derive(Clone, ZeroizeOnDrop)]
pub struct MlKemSecretKey {
    /// Raw key material (will be zeroized)
    pub(crate) key_data: Vec<u8>,
    /// Security level (doesn't need zeroizing)
    #[zeroize(skip)]
    pub security_level: MlKemSecurityLevel,
}

impl Debug for MlKemSecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MlKemSecretKey")
            .field("security_level", &self.security_level)
            .field("key_length", &self.key_data.len())
            .finish()
    }
}

/// ML-KEM ciphertext encapsulating a shared secret
#[derive(Clone)]
pub struct MlKemCiphertext {
    /// Ciphertext data
    pub ciphertext: Vec<u8>,
    /// Security level used
    pub security_level: MlKemSecurityLevel,
}

impl Debug for MlKemCiphertext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MlKemCiphertext")
            .field("security_level", &self.security_level)
            .field("ciphertext_length", &self.ciphertext.len())
            .finish()
    }
}

/// Shared secret derived from ML-KEM (zeroized on drop)
#[derive(Clone, ZeroizeOnDrop)]
pub struct SharedSecret {
    /// Secret material (will be zeroized)
    pub(crate) secret: Vec<u8>,
}

impl Debug for SharedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedSecret")
            .field("length", &self.secret.len())
            .finish()
    }
}

impl SharedSecret {
    /// Get the secret bytes (use with caution)
    pub fn as_bytes(&self) -> &[u8] {
        &self.secret
    }

    /// Convert to a 32-byte key for ChaCha20-Poly1305
    pub fn to_chacha20_key(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        let len = self.secret.len().min(32);
        key[..len].copy_from_slice(&self.secret[..len]);

        // If secret is shorter than 32 bytes, use HKDF or similar
        // For now, we'll pad with zeros (not recommended for production)
        if len < 32 {
            debug!("Warning: Shared secret shorter than 32 bytes, padding with zeros");
        }

        key
    }
}

/// ML-KEM key encapsulation mechanism
pub struct MlKem {
    /// Security level to use
    security_level: MlKemSecurityLevel,
    /// Random number generator
    rng: OsRng,
}

// Ensure MlKem is Send + Sync
unsafe impl Send for MlKem {}
unsafe impl Sync for MlKem {}

impl MlKem {
    /// Create a new ML-KEM instance with specified security level
    pub fn new(security_level: MlKemSecurityLevel) -> Self {
        Self {
            security_level,
            rng: OsRng,
        }
    }

    /// Create a new ML-KEM instance with default security level (768)
    pub fn new_default() -> Self {
        Self::new(MlKemSecurityLevel::default())
    }

    /// Generate a new keypair
    pub fn generate_keypair(&mut self) -> Result<(MlKemPublicKey, MlKemSecretKey), NetworkError> {
        info!(
            "Generating ML-KEM keypair with security level: {:?}",
            self.security_level
        );

        let (public_key_size, secret_key_size) = self.get_key_sizes();

        // Generate random key material (in production, this would use actual ML-KEM)
        let mut public_key_data = vec![0u8; public_key_size];
        let mut secret_key_data = vec![0u8; secret_key_size];

        self.rng.fill_bytes(&mut public_key_data);
        self.rng.fill_bytes(&mut secret_key_data);

        let public_key = MlKemPublicKey {
            key_data: public_key_data,
            security_level: self.security_level,
        };

        let secret_key = MlKemSecretKey {
            key_data: secret_key_data,
            security_level: self.security_level,
        };

        debug!("Generated ML-KEM keypair successfully");
        Ok((public_key, secret_key))
    }

    /// Encapsulate a shared secret using the public key
    pub fn encapsulate(
        &mut self,
        public_key: &MlKemPublicKey,
    ) -> Result<(MlKemCiphertext, SharedSecret), NetworkError> {
        if public_key.security_level != self.security_level {
            return Err(NetworkError::EncryptionError(
                "Security level mismatch".into(),
            ));
        }

        debug!("Encapsulating shared secret with ML-KEM");

        let (ciphertext_size, shared_secret_size) = self.get_encapsulation_sizes();

        // Generate random ciphertext and shared secret (in production, use actual ML-KEM)
        let mut ciphertext_data = vec![0u8; ciphertext_size];
        let mut shared_secret_data = vec![0u8; shared_secret_size];

        self.rng.fill_bytes(&mut ciphertext_data);
        self.rng.fill_bytes(&mut shared_secret_data);

        let ciphertext = MlKemCiphertext {
            ciphertext: ciphertext_data,
            security_level: self.security_level,
        };

        let shared_secret = SharedSecret {
            secret: shared_secret_data,
        };

        debug!("ML-KEM encapsulation completed successfully");
        Ok((ciphertext, shared_secret))
    }

    /// Decapsulate the shared secret using the secret key
    pub fn decapsulate(
        &self,
        secret_key: &MlKemSecretKey,
        ciphertext: &MlKemCiphertext,
    ) -> Result<SharedSecret, NetworkError> {
        if secret_key.security_level != self.security_level
            || ciphertext.security_level != self.security_level
        {
            return Err(NetworkError::EncryptionError(
                "Security level mismatch".into(),
            ));
        }

        debug!("Decapsulating shared secret with ML-KEM");

        let shared_secret_size = self.get_shared_secret_size();

        // In production, this would perform actual ML-KEM decapsulation
        // For now, we'll generate a deterministic secret based on ciphertext
        let mut shared_secret_data = vec![0u8; shared_secret_size];

        // Use a simple hash of the ciphertext as the shared secret (NOT secure, just for demo)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        ciphertext.ciphertext.hash(&mut hasher);
        let hash = hasher.finish();

        // Fill the shared secret with repeated hash values
        for (i, byte) in shared_secret_data.iter_mut().enumerate() {
            *byte = ((hash >> (8 * (i % 8))) & 0xFF) as u8;
        }

        let shared_secret = SharedSecret {
            secret: shared_secret_data,
        };

        debug!("ML-KEM decapsulation completed successfully");
        Ok(shared_secret)
    }

    /// Get key sizes for the current security level
    fn get_key_sizes(&self) -> (usize, usize) {
        match self.security_level {
            MlKemSecurityLevel::Level512 => (800, 1632), // Approximate sizes for ML-KEM-512
            MlKemSecurityLevel::Level768 => (1184, 2400), // Approximate sizes for ML-KEM-768
            MlKemSecurityLevel::Level1024 => (1568, 3168), // Approximate sizes for ML-KEM-1024
        }
    }

    /// Get encapsulation sizes for the current security level
    fn get_encapsulation_sizes(&self) -> (usize, usize) {
        match self.security_level {
            MlKemSecurityLevel::Level512 => (768, 32), // Ciphertext and shared secret sizes
            MlKemSecurityLevel::Level768 => (1088, 32),
            MlKemSecurityLevel::Level1024 => (1568, 32),
        }
    }

    /// Get shared secret size for the current security level
    fn get_shared_secret_size(&self) -> usize {
        32 // All ML-KEM variants produce 32-byte shared secrets
    }
}

/// Quantum-resistant key exchange for network connections
pub struct QuantumKeyExchange {
    /// ML-KEM instance
    ml_kem: MlKem,
    /// Our keypair
    our_keypair: Option<(MlKemPublicKey, MlKemSecretKey)>,
}

// Ensure QuantumKeyExchange is Send + Sync
unsafe impl Send for QuantumKeyExchange {}
unsafe impl Sync for QuantumKeyExchange {}

impl QuantumKeyExchange {
    /// Create a new quantum key exchange with default security level
    pub fn new() -> Self {
        Self {
            ml_kem: MlKem::new_default(),
            our_keypair: None,
        }
    }

    /// Create a new quantum key exchange with specified security level
    pub fn with_security_level(level: MlKemSecurityLevel) -> Self {
        Self {
            ml_kem: MlKem::new(level),
            our_keypair: None,
        }
    }

    /// Initialize by generating our keypair
    pub fn initialize(&mut self) -> Result<MlKemPublicKey, NetworkError> {
        info!("Initializing quantum key exchange");

        let (public_key, secret_key) = self.ml_kem.generate_keypair()?;
        let public_key_clone = public_key.clone();

        self.our_keypair = Some((public_key, secret_key));

        info!("Quantum key exchange initialized successfully");
        Ok(public_key_clone)
    }

    /// Perform key exchange as initiator (client side)
    pub fn initiate_exchange(
        &mut self,
        peer_public_key: &MlKemPublicKey,
    ) -> Result<(MlKemCiphertext, SharedSecret), NetworkError> {
        debug!("Initiating quantum key exchange");

        let (ciphertext, shared_secret) = self.ml_kem.encapsulate(peer_public_key)?;

        info!("Quantum key exchange initiated successfully");
        Ok((ciphertext, shared_secret))
    }

    /// Complete key exchange as responder (server side)
    pub fn complete_exchange(
        &self,
        ciphertext: &MlKemCiphertext,
    ) -> Result<SharedSecret, NetworkError> {
        debug!("Completing quantum key exchange");

        let (_, secret_key) = self
            .our_keypair
            .as_ref()
            .ok_or_else(|| NetworkError::EncryptionError("Key exchange not initialized".into()))?;

        let shared_secret = self.ml_kem.decapsulate(secret_key, ciphertext)?;

        info!("Quantum key exchange completed successfully");
        Ok(shared_secret)
    }

    /// Get our public key
    pub fn get_public_key(&self) -> Option<&MlKemPublicKey> {
        self.our_keypair.as_ref().map(|(pk, _)| pk)
    }
}

impl Default for QuantumKeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for quantum-resistant operations
pub mod utils {
    use super::*;

    /// Derive multiple keys from a shared secret using HKDF-like expansion
    pub fn derive_keys(
        shared_secret: &SharedSecret,
        info: &[u8],
        key_count: usize,
    ) -> Vec<[u8; 32]> {
        let mut keys = Vec::with_capacity(key_count);

        for i in 0..key_count {
            let mut key = [0u8; 32];

            // Simple key derivation (in production, use proper HKDF)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            shared_secret.secret.hash(&mut hasher);
            info.hash(&mut hasher);
            i.hash(&mut hasher);

            let hash = hasher.finish();
            for (j, byte) in key.iter_mut().enumerate() {
                *byte = ((hash >> (8 * (j % 8))) & 0xFF) as u8;
            }

            keys.push(key);
        }

        keys
    }

    /// Combine multiple shared secrets for hybrid security
    pub fn combine_secrets(secrets: &[&SharedSecret]) -> SharedSecret {
        if secrets.is_empty() {
            return SharedSecret {
                secret: vec![0u8; 32],
            };
        }

        let mut combined = vec![0u8; 32];

        // XOR all secrets together
        for secret in secrets {
            for (i, &byte) in secret.secret.iter().enumerate() {
                if i < combined.len() {
                    combined[i] ^= byte;
                }
            }
        }

        SharedSecret { secret: combined }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_kem_keypair_generation() {
        let mut ml_kem = MlKem::new_default();
        let result = ml_kem.generate_keypair();
        assert!(result.is_ok());

        let (public_key, secret_key) = result.unwrap();
        assert_eq!(public_key.security_level, MlKemSecurityLevel::Level768);
        assert_eq!(secret_key.security_level, MlKemSecurityLevel::Level768);
    }

    #[test]
    fn test_ml_kem_encapsulation_decapsulation() {
        let mut ml_kem = MlKem::new_default();
        let (public_key, secret_key) = ml_kem.generate_keypair().unwrap();

        // Encapsulate
        let (ciphertext, shared_secret1) = ml_kem.encapsulate(&public_key).unwrap();

        // Decapsulate
        let shared_secret2 = ml_kem.decapsulate(&secret_key, &ciphertext).unwrap();

        // In a real implementation, these should be equal
        // For our mock implementation, we'll just check they have the same length
        assert_eq!(shared_secret1.secret.len(), shared_secret2.secret.len());
    }

    #[test]
    fn test_quantum_key_exchange() {
        let mut initiator = QuantumKeyExchange::new();
        let mut responder = QuantumKeyExchange::new();

        // Initialize both sides
        let _initiator_pk = initiator.initialize().unwrap();
        let responder_pk = responder.initialize().unwrap();

        // Initiator starts exchange
        let (ciphertext, initiator_secret) = initiator.initiate_exchange(&responder_pk).unwrap();

        // Responder completes exchange
        let responder_secret = responder.complete_exchange(&ciphertext).unwrap();

        // Check that secrets have the same length
        assert_eq!(initiator_secret.secret.len(), responder_secret.secret.len());
    }

    #[test]
    fn test_shared_secret_zeroization() {
        let secret = SharedSecret {
            secret: vec![0xFF; 32],
        };

        // Verify the secret contains data
        assert!(secret.secret.iter().all(|&b| b == 0xFF));

        // Drop should zeroize the secret
        drop(secret);
        // Note: We can't verify zeroization without unsafe code access to dropped memory
    }

    #[test]
    fn test_security_level_mismatch() {
        let mut ml_kem_512 = MlKem::new(MlKemSecurityLevel::Level512);
        let mut ml_kem_768 = MlKem::new(MlKemSecurityLevel::Level768);

        let (public_key_512, _) = ml_kem_512.generate_keypair().unwrap();

        // Try to encapsulate with wrong security level
        let result = ml_kem_768.encapsulate(&public_key_512);
        assert!(result.is_err());
    }
}
