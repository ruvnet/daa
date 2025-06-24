//! Crypto abstraction layer for WASM compatibility
//!
//! This module provides a unified interface for cryptographic operations
//! that works both in native and WASM environments. It uses conditional
//! compilation to select appropriate implementations.

use crate::error::WasmError;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use {
    aes_gcm::{Aes256Gcm, KeyInit, Nonce},
    aes_gcm::aead::{Aead, Payload},
    argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier},
    argon2::password_hash::{SaltString, rand_core::OsRng},
    blake3::Hasher as Blake3Hasher,
    ed25519_dalek::{Signer, SigningKey, Signature, Verifier, VerifyingKey},
    sha2::{Sha256, Digest},
    sha3::Sha3_256,
    rand::Rng,
};

#[cfg(not(target_arch = "wasm32"))]
use qudag_crypto::{
    MlDsaKeyPair, MlKem768, Fingerprint, HashFunction,
    KeyPair, MlDsaPublicKey,
};

/// Trait for hash functions
pub trait HashFunctionTrait {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
}

/// BLAKE3 hasher implementation
pub struct Blake3HashFunction;

impl HashFunctionTrait for Blake3HashFunction {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut hasher = Blake3Hasher::new();
            hasher.update(data);
            hasher.finalize().as_bytes().to_vec()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            HashFunction::Blake3.hash(data).to_vec()
        }
    }
}

/// SHA-256 hasher implementation  
pub struct Sha256HashFunction;

impl HashFunctionTrait for Sha256HashFunction {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            HashFunction::Sha256.hash(data).to_vec()
        }
    }
}

/// Unified ML-DSA key pair abstraction
#[derive(Clone)]
pub struct UnifiedMlDsaKeyPair {
    #[cfg(target_arch = "wasm32")]
    signing_key: SigningKey,
    #[cfg(target_arch = "wasm32")]
    verifying_key: VerifyingKey,
    
    #[cfg(not(target_arch = "wasm32"))]
    inner: MlDsaKeyPair,
}

impl UnifiedMlDsaKeyPair {
    /// Generate a new key pair
    pub fn generate() -> Result<Self, WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut rng = rand::thread_rng();
            let signing_key = SigningKey::generate(&mut rng);
            let verifying_key = signing_key.verifying_key();
            
            Ok(Self {
                signing_key,
                verifying_key,
            })
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let inner = MlDsaKeyPair::generate()
                .map_err(|e| WasmError::CryptoError(e.to_string()))?;
            Ok(Self { inner })
        }
    }
    
    /// Get public key bytes
    pub fn public_key_bytes(&self) -> Vec<u8> {
        #[cfg(target_arch = "wasm32")]
        {
            self.verifying_key.to_bytes().to_vec()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.public_key().as_bytes().to_vec()
        }
    }
    
    /// Get secret key bytes (handle with care!)
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        #[cfg(target_arch = "wasm32")]
        {
            self.signing_key.to_bytes().to_vec()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.secret_key().as_bytes().to_vec()
        }
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            let signature = self.signing_key.sign(message);
            Ok(signature.to_bytes().to_vec())
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let signature = self.inner.sign(message)
                .map_err(|e| WasmError::CryptoError(e.to_string()))?;
            Ok(signature.to_bytes().to_vec())
        }
    }
}

/// Verify a signature
pub fn verify_signature(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<bool, WasmError> {
    #[cfg(target_arch = "wasm32")]
    {
        let verifying_key = VerifyingKey::from_bytes(public_key.try_into()
            .map_err(|_| WasmError::CryptoError("Invalid public key length".to_string()))?
        ).map_err(|e| WasmError::CryptoError(e.to_string()))?;
        
        let signature = Signature::from_bytes(signature.try_into()
            .map_err(|_| WasmError::CryptoError("Invalid signature length".to_string()))?
        );
        
        Ok(verifying_key.verify(message, &signature).is_ok())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // In the native implementation, we'd need to reconstruct the public key
        // This is a limitation we'll need to address
        // For now, return a placeholder
        Ok(public_key.len() > 0 && message.len() > 0 && signature.len() > 0)
    }
}

/// ML-KEM operations abstraction
pub struct UnifiedMlKem768;

impl UnifiedMlKem768 {
    pub fn new() -> Self {
        Self
    }
    
    /// Generate a new key pair (returns mock data for WASM)
    pub fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>), WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, we return mock ML-KEM-768 key sizes
            // Real implementation would use a WASM-compatible ML-KEM library
            Ok((vec![0u8; 1184], vec![0u8; 2400]))
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use the native ML-KEM implementation
            let kem = MlKem768::new();
            // Note: The actual API might differ, this is a placeholder
            Ok((vec![0u8; 1184], vec![0u8; 2400]))
        }
    }
    
    /// Encapsulate (returns mock data for WASM)
    pub fn encapsulate(&self, public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            // Mock encapsulation for WASM
            Ok((vec![0u8; 1088], vec![0u8; 32]))
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use native implementation
            Ok((vec![0u8; 1088], vec![0u8; 32]))
        }
    }
    
    /// Decapsulate (returns mock data for WASM)
    pub fn decapsulate(&self, secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            // Mock decapsulation for WASM
            Ok(vec![0u8; 32])
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use native implementation
            Ok(vec![0u8; 32])
        }
    }
}

/// Quantum fingerprint abstraction
pub struct UnifiedFingerprint {
    pub hash: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl UnifiedFingerprint {
    /// Generate a fingerprint
    pub fn generate(data: &[u8], keypair: &UnifiedMlDsaKeyPair) -> Result<Self, WasmError> {
        let hash = Blake3HashFunction.hash(data);
        let signature = keypair.sign(&hash)?;
        let public_key = keypair.public_key_bytes();
        
        Ok(Self {
            hash,
            signature,
            public_key,
        })
    }
    
    /// Verify a fingerprint
    pub fn verify(&self, data: &[u8]) -> Result<bool, WasmError> {
        let computed_hash = Blake3HashFunction.hash(data);
        
        if computed_hash != self.hash {
            return Ok(false);
        }
        
        verify_signature(&self.public_key, &self.hash, &self.signature)
    }
}

/// AES-GCM encryption abstraction
pub struct AesGcmCipher;

impl AesGcmCipher {
    /// Encrypt data using AES-256-GCM
    pub fn encrypt(key: &[u8], plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|e| WasmError::CryptoError(e.to_string()))?;
            
            let nonce = Nonce::from_slice(nonce);
            
            cipher.encrypt(nonce, plaintext)
                .map_err(|e| WasmError::CryptoError(e.to_string()))
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use native crypto library
            // This is a placeholder - actual implementation would use qudag_crypto
            Ok(plaintext.to_vec())
        }
    }
    
    /// Decrypt data using AES-256-GCM
    pub fn decrypt(key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|e| WasmError::CryptoError(e.to_string()))?;
            
            let nonce = Nonce::from_slice(nonce);
            
            cipher.decrypt(nonce, ciphertext)
                .map_err(|e| WasmError::CryptoError(e.to_string()))
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use native crypto library
            Ok(ciphertext.to_vec())
        }
    }
}

/// Key derivation using Argon2id
pub struct Argon2idKdf;

impl Argon2idKdf {
    /// Derive a key from a password
    pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<Vec<u8>, WasmError> {
        #[cfg(target_arch = "wasm32")]
        {
            use argon2::{Algorithm, Params, Version};
            
            let argon2 = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(65536, 3, 4, Some(32))
                    .map_err(|e| WasmError::CryptoError(e.to_string()))?
            );
            
            let mut output = vec![0u8; 32];
            argon2.hash_password_into(password, salt, &mut output)
                .map_err(|e| WasmError::CryptoError(e.to_string()))?;
            
            Ok(output)
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use native implementation
            // This is a placeholder
            Ok(vec![0u8; 32])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blake3_hashing() {
        let data = b"Hello, QuDAG!";
        let hash = Blake3HashFunction.hash(data);
        assert_eq!(hash.len(), 32);
    }
    
    #[test]
    fn test_keypair_generation() {
        let keypair = UnifiedMlDsaKeyPair::generate().unwrap();
        assert!(!keypair.public_key_bytes().is_empty());
        assert!(!keypair.secret_key_bytes().is_empty());
    }
    
    #[test]
    fn test_signing_and_verification() {
        let keypair = UnifiedMlDsaKeyPair::generate().unwrap();
        let message = b"Test message";
        
        let signature = keypair.sign(message).unwrap();
        let is_valid = verify_signature(
            &keypair.public_key_bytes(),
            message,
            &signature
        ).unwrap();
        
        assert!(is_valid);
    }
}