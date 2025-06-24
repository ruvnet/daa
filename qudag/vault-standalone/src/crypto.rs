//! Cryptographic operations for the vault with quantum-resistant primitives.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
// QuDAG integration disabled for initial standalone publishing
// #[cfg(feature = "qudag-integration")]
// use qudag_crypto::{
//     ml_kem::MlKem768, 
//     ml_dsa::MlDsaKeyPair,
//     kem::PublicKey as KemPublicKey,
// };
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{VaultError, VaultResult};

/// Size of the vault encryption key in bytes.
pub const VAULT_KEY_SIZE: usize = 32; // 256 bits for AES-256

/// Size of the AES-GCM nonce in bytes.
pub const NONCE_SIZE: usize = 12; // 96 bits for AES-GCM

/// Vault cryptographic operations wrapper.
pub struct VaultCrypto {
    /// The main vault encryption key.
    vault_key: VaultKey,
}

/// A zeroizable vault key.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct VaultKey([u8; VAULT_KEY_SIZE]);

/// Key pair for quantum-resistant operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultKeyPair {
    /// ML-KEM public key for key encapsulation.
    pub kem_public: Vec<u8>,
    /// ML-KEM secret key (encrypted).
    pub kem_secret_encrypted: Vec<u8>,
    /// ML-DSA public key for signatures (optional in standalone mode).
    pub dsa_public: Vec<u8>,
    /// ML-DSA secret key (encrypted).
    pub dsa_secret_encrypted: Vec<u8>,
}

impl VaultCrypto {
    /// Create a new vault crypto instance with a random key.
    pub fn new() -> VaultResult<Self> {
        let mut key = [0u8; VAULT_KEY_SIZE];
        getrandom::getrandom(&mut key)
            .map_err(|e| VaultError::Crypto(format!("Failed to generate random key: {}", e)))?;
        
        Ok(Self {
            vault_key: VaultKey(key),
        })
    }

    /// Create a vault crypto instance from an existing key.
    pub fn from_key(key: [u8; VAULT_KEY_SIZE]) -> Self {
        Self {
            vault_key: VaultKey(key),
        }
    }

    /// Get the vault key (be careful with this!).
    pub fn get_key(&self) -> &[u8; VAULT_KEY_SIZE] {
        &self.vault_key.0
    }

    /// Encrypt data using AES-256-GCM.
    pub fn encrypt(&self, plaintext: &[u8]) -> VaultResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.vault_key.0));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| VaultError::Crypto(format!("Encryption failed: {}", e)))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM.
    pub fn decrypt(&self, ciphertext: &[u8]) -> VaultResult<Vec<u8>> {
        if ciphertext.len() < NONCE_SIZE {
            return Err(VaultError::Crypto("Invalid ciphertext: too short".to_string()));
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.vault_key.0));
        
        cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| VaultError::Crypto(format!("Decryption failed: {}", e)))
    }

    /// Generate a placeholder key pair for standalone mode.
    /// In standalone mode, this creates placeholder keypair data.
    /// For real post-quantum crypto, use the 'qudag-integration' feature.
    pub fn generate_keypair(&self) -> VaultResult<VaultKeyPair> {
        // In standalone mode, create placeholder keypair
        // Users should enable qudag-integration feature for real post-quantum crypto
        let mut kem_public = vec![0u8; 1184]; // ML-KEM-768 public key size
        let mut dsa_public = vec![0u8; 1312]; // ML-DSA-65 public key size
        
        getrandom::getrandom(&mut kem_public)
            .map_err(|e| VaultError::Crypto(format!("Failed to generate KEM key: {}", e)))?;
        getrandom::getrandom(&mut dsa_public)
            .map_err(|e| VaultError::Crypto(format!("Failed to generate DSA key: {}", e)))?;
        
        let kem_secret_encrypted = self.encrypt(&[0u8; 2400])?; // ML-KEM-768 secret key size
        let dsa_secret_encrypted = self.encrypt(&[0u8; 4032])?; // ML-DSA-65 secret key size
        
        Ok(VaultKeyPair {
            kem_public,
            kem_secret_encrypted,
            dsa_public,
            dsa_secret_encrypted,
        })
    }

    /// Placeholder encapsulation for standalone mode.
    /// For real ML-KEM encapsulation, use the 'qudag-integration' feature.
    pub fn encapsulate_vault_key(
        &self,
        _recipient_public_key: &[u8],
    ) -> VaultResult<(Vec<u8>, Vec<u8>)> {
        Err(VaultError::Crypto(
            "Key encapsulation requires qudag-integration feature".to_string()
        ))
    }

    /// Compute BLAKE3 hash of data.
    pub fn hash(data: &[u8]) -> Vec<u8> {
        // Use blake3 from workspace dependencies
        blake3::hash(data).as_bytes().to_vec()
    }
}

impl Drop for VaultCrypto {
    fn drop(&mut self) {
        // VaultKey already implements ZeroizeOnDrop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = VaultCrypto::new().unwrap();
        let plaintext = b"Hello, Vault!";
        
        let ciphertext = crypto.encrypt(plaintext).unwrap();
        assert_ne!(ciphertext, plaintext);
        
        let decrypted = crypto.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_hash() {
        let data = b"test data";
        let hash1 = VaultCrypto::hash(data);
        let hash2 = VaultCrypto::hash(data);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // BLAKE3 output is 256 bits
    }
}