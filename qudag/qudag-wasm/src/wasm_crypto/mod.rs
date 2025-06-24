//! WASM-compatible cryptography module for QuDAG
//!
//! This module provides quantum-resistant cryptographic operations that work in WASM environments.
//! It uses a hybrid approach: WebCrypto API where available, and pure Rust implementations as fallback.

use anyhow::Result;
use js_sys::{Object, Uint8Array};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub mod ml_dsa;
pub mod ml_kem;
pub mod utils;

// Simplified provider for now
pub struct PureRustCryptoProvider;

impl PureRustCryptoProvider {
    pub fn new() -> Self {
        PureRustCryptoProvider
    }

    pub fn blake3(&self, data: &[u8]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().as_bytes().to_vec()
    }

    pub fn random_bytes(&self, len: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0u8; len];
        getrandom::getrandom(&mut bytes)?;
        Ok(bytes)
    }
}

/// ML-KEM-768 key pair for WASM
#[wasm_bindgen]
pub struct WasmMlKemKeyPair {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

#[wasm_bindgen]
impl WasmMlKemKeyPair {
    /// Generate a new ML-KEM-768 key pair
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmMlKemKeyPair, JsError> {
        let (public_key, secret_key) = ml_kem::generate_keypair()
            .map_err(|e| JsError::new(&format!("Failed to generate ML-KEM keypair: {}", e)))?;

        Ok(Self {
            public_key,
            secret_key,
        })
    }

    /// Get the public key
    #[wasm_bindgen(js_name = "getPublicKey")]
    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    /// Get the secret key
    #[wasm_bindgen(js_name = "getSecretKey")]
    pub fn get_secret_key(&self) -> Vec<u8> {
        self.secret_key.clone()
    }

    /// Encapsulate a shared secret
    #[wasm_bindgen]
    pub fn encapsulate(&self, public_key: &[u8]) -> Result<JsValue, JsError> {
        let (ciphertext, shared_secret) = ml_kem::encapsulate(public_key)
            .map_err(|e| JsError::new(&format!("Encapsulation failed: {}", e)))?;

        let result = EncapsulationResult {
            ciphertext: hex::encode(&ciphertext),
            shared_secret: hex::encode(&shared_secret),
        };

        Ok(serde_wasm_bindgen::to_value(&result)?)
    }

    /// Decapsulate a shared secret
    #[wasm_bindgen]
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<Vec<u8>, JsError> {
        ml_kem::decapsulate(&self.secret_key, ciphertext)
            .map_err(|e| JsError::new(&format!("Decapsulation failed: {}", e)))
    }
}

/// ML-DSA key pair for WASM
#[wasm_bindgen]
pub struct WasmMlDsaKeyPair {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

#[wasm_bindgen]
impl WasmMlDsaKeyPair {
    /// Generate a new ML-DSA key pair
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmMlDsaKeyPair, JsError> {
        let (public_key, secret_key) = ml_dsa::generate_keypair()
            .map_err(|e| JsError::new(&format!("Failed to generate ML-DSA keypair: {}", e)))?;

        Ok(Self {
            public_key,
            secret_key,
        })
    }

    /// Get the public key
    #[wasm_bindgen(js_name = "getPublicKey")]
    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    /// Get the secret key
    #[wasm_bindgen(js_name = "getSecretKey")]
    pub fn get_secret_key(&self) -> Vec<u8> {
        self.secret_key.clone()
    }

    /// Sign a message
    #[wasm_bindgen]
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, JsError> {
        ml_dsa::sign(&self.secret_key, message)
            .map_err(|e| JsError::new(&format!("Signing failed: {}", e)))
    }

    /// Verify a signature
    #[wasm_bindgen]
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, JsError> {
        ml_dsa::verify(&self.public_key, message, signature)
            .map_err(|e| JsError::new(&format!("Verification failed: {}", e)))
    }
}

/// Key derivation functions for WASM
#[wasm_bindgen]
pub struct WasmKdf;

#[wasm_bindgen]
impl WasmKdf {
    /// Derive a key using simple hashing (simplified for WASM)
    #[wasm_bindgen(js_name = "deriveKey")]
    pub fn derive_key(password: &[u8], salt: &[u8], key_length: usize) -> Result<Vec<u8>, JsError> {
        // Simple key derivation using Blake3
        let mut hasher = blake3::Hasher::new();
        hasher.update(password);
        hasher.update(salt);
        let hash = hasher.finalize();

        let mut key = vec![0u8; key_length];
        for i in 0..key_length {
            key[i] = hash.as_bytes()[i % 32];
        }

        Ok(key)
    }

    /// Generate a random salt
    #[wasm_bindgen(js_name = "generateSalt")]
    pub fn generate_salt() -> Result<Vec<u8>, JsError> {
        let provider = PureRustCryptoProvider::new();
        provider
            .random_bytes(32)
            .map_err(|e| JsError::new(&format!("Salt generation failed: {}", e)))
    }
}

/// Quantum fingerprint for WASM
#[wasm_bindgen]
pub struct WasmQuantumFingerprint {
    hash: Vec<u8>,
    signature: Vec<u8>,
    public_key: Vec<u8>,
}

#[wasm_bindgen]
impl WasmQuantumFingerprint {
    /// Generate a quantum fingerprint
    #[wasm_bindgen(js_name = "generate")]
    pub fn generate(data: &[u8]) -> Result<WasmQuantumFingerprint, JsError> {
        // Generate a new ML-DSA keypair for the fingerprint
        let keypair = WasmMlDsaKeyPair::new()?;

        // Hash the data with Blake3
        let provider = PureRustCryptoProvider::new();
        let hash = provider.blake3(data);

        // Sign the hash
        let signature = keypair.sign(&hash)?;

        Ok(Self {
            hash,
            signature,
            public_key: keypair.get_public_key(),
        })
    }

    /// Get the hash
    #[wasm_bindgen(js_name = "getHash")]
    pub fn get_hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    /// Get the signature
    #[wasm_bindgen(js_name = "getSignature")]
    pub fn get_signature(&self) -> Vec<u8> {
        self.signature.clone()
    }

    /// Get the public key
    #[wasm_bindgen(js_name = "getPublicKey")]
    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    /// Export as JSON
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> Result<JsValue, JsError> {
        let data = FingerprintData {
            hash: hex::encode(&self.hash),
            signature: hex::encode(&self.signature),
            public_key: hex::encode(&self.public_key),
        };

        Ok(serde_wasm_bindgen::to_value(&data)?)
    }
}

// Helper structures for serialization
#[derive(Serialize, Deserialize)]
struct EncapsulationResult {
    ciphertext: String,
    shared_secret: String,
}

#[derive(Serialize, Deserialize)]
struct FingerprintData {
    hash: String,
    signature: String,
    public_key: String,
}

/// Secure memory wrapper that zeros on drop
pub struct SecureMemory<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> SecureMemory<N> {
    pub fn new() -> Self {
        Self { data: [0u8; N] }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl<const N: usize> Drop for SecureMemory<N> {
    fn drop(&mut self) {
        // Use volatile writes to prevent optimization
        for i in 0..N {
            unsafe {
                core::ptr::write_volatile(&mut self.data[i], 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_wasm_crypto_provider() {
        let provider = PureRustCryptoProvider::new();

        // Test random bytes
        let random = provider.random_bytes(32).unwrap();
        assert_eq!(random.len(), 32);

        // Test Blake3
        let data = b"Hello, QuDAG!";
        let hash = provider.blake3(data);
        assert_eq!(hash.len(), 32);
    }

    #[wasm_bindgen_test]
    fn test_ml_kem_keypair() {
        let keypair = WasmMlKemKeyPair::new().unwrap();
        assert!(!keypair.get_public_key().is_empty());
        assert!(!keypair.get_secret_key().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_ml_dsa_keypair() {
        let keypair = WasmMlDsaKeyPair::new().unwrap();
        let message = b"Test message";

        let signature = keypair.sign(message).unwrap();
        let valid = keypair.verify(message, &signature).unwrap();
        assert!(valid);
    }
}
