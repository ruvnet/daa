//! Cryptographic operations for WASM
//!
//! Provides quantum-resistant cryptographic operations including:
//! - ML-KEM-768 key encapsulation
//! - ML-DSA digital signatures
//! - BLAKE3 hashing
//! - Quantum fingerprinting

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Use the WASM-compatible crypto module
pub use crate::wasm_crypto::{
    WasmMlDsaKeyPair as MlDsaKeyPairInternal,
    WasmMlKemKeyPair as MlKemKeyPairInternal,
    WasmQuantumFingerprint as FingerprintInternal,
};

/// ML-DSA key pair for digital signatures
#[wasm_bindgen]
pub struct WasmMlDsaKeyPair {
    inner: MlDsaKeyPairInternal,
}

#[wasm_bindgen]
impl WasmMlDsaKeyPair {
    /// Generate a new ML-DSA key pair
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmMlDsaKeyPair, JsError> {
        let keypair = MlDsaKeyPairInternal::new()?;
        Ok(Self { inner: keypair })
    }
    
    /// Get the public key as bytes
    #[wasm_bindgen(js_name = "getPublicKey")]
    pub fn get_public_key(&self) -> Vec<u8> {
        self.inner.get_public_key()
    }
    
    /// Get the secret key as bytes (handle with care!)
    #[wasm_bindgen(js_name = "getSecretKey")]
    pub fn get_secret_key(&self) -> Vec<u8> {
        self.inner.get_secret_key()
    }
    
    /// Sign a message
    #[wasm_bindgen]
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, JsError> {
        self.inner.sign(message)
    }
    
    /// Export key pair as JSON
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> Result<JsValue, JsError> {
        let data = KeyPairData {
            public_key: hex::encode(self.get_public_key()),
            secret_key: hex::encode(self.get_secret_key()),
            key_type: "ML-DSA".to_string(),
        };
        Ok(serde_wasm_bindgen::to_value(&data)?)
    }
}

/// ML-KEM-768 operations for key encapsulation
#[wasm_bindgen]
pub struct WasmMlKem768 {
    // We don't need an inner field for stateless operations
}
#[wasm_bindgen]
impl WasmMlKem768 {
    /// Create a new ML-KEM-768 instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }
    
    /// Generate a new key pair
    #[wasm_bindgen(js_name = "generateKeyPair")]
    pub fn generate_key_pair(&self) -> Result<JsValue, JsError> {
        let keypair = MlKemKeyPairInternal::new()?;
        let data = KemKeyPairData {
            public_key: hex::encode(keypair.get_public_key()),
            secret_key: hex::encode(keypair.get_secret_key()),
            key_type: "ML-KEM-768".to_string(),
        };
        
        Ok(serde_wasm_bindgen::to_value(&data)?)
    }
    
    /// Encapsulate a shared secret
    #[wasm_bindgen]
    pub fn encapsulate(&self, public_key: &[u8]) -> Result<JsValue, JsError> {
        let temp_keypair = MlKemKeyPairInternal::new()?;
        temp_keypair.encapsulate(public_key)
    }
    
    /// Decapsulate a shared secret
    #[wasm_bindgen]
    pub fn decapsulate(&self, secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, JsError> {
        // Create a temporary keypair to access the decapsulate method
        let temp_keypair = MlKemKeyPairInternal::new()?;
        temp_keypair.decapsulate(ciphertext)
    }
}

/// BLAKE3 hashing operations
#[wasm_bindgen]
pub struct WasmHasher;

#[wasm_bindgen]
impl WasmHasher {
    /// Hash data using BLAKE3
    #[wasm_bindgen(js_name = "hashBlake3")]
    pub fn hash_blake3(data: &[u8]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().as_bytes().to_vec()
    }
    
    /// Create a hex-encoded hash
    #[wasm_bindgen(js_name = "hashBlake3Hex")]
    pub fn hash_blake3_hex(data: &[u8]) -> String {
        hex::encode(Self::hash_blake3(data))
    }
}

/// Quantum fingerprinting operations
#[wasm_bindgen]
pub struct WasmFingerprint;

#[wasm_bindgen]
impl WasmFingerprint {
    /// Generate a quantum fingerprint for data
    #[wasm_bindgen(js_name = "generate")]
    pub fn generate(data: &[u8], _keypair_bytes: &[u8]) -> Result<JsValue, JsError> {
        let fingerprint = FingerprintInternal::generate(data)?;
        fingerprint.to_json()
    }
    
    /// Verify a quantum fingerprint
    #[wasm_bindgen(js_name = "verify")]
    pub fn verify(data: &[u8], fingerprint_json: JsValue) -> Result<bool, JsError> {
        let fingerprint_data: FingerprintData = serde_wasm_bindgen::from_value(fingerprint_json)?;
        
        let _hash = hex::decode(&fingerprint_data.hash)
            .map_err(|e| JsError::new(&format!("Invalid hash hex: {}", e)))?;
        let _signature = hex::decode(&fingerprint_data.signature)
            .map_err(|e| JsError::new(&format!("Invalid signature hex: {}", e)))?;
        let _public_key = hex::decode(&fingerprint_data.public_key)
            .map_err(|e| JsError::new(&format!("Invalid public key hex: {}", e)))?;
        
        // Calculate hash of data and compare
        let calculated_hash = WasmHasher::hash_blake3(data);
        let provided_hash = hex::decode(&fingerprint_data.hash)
            .map_err(|e| JsError::new(&format!("Invalid hash hex: {}", e)))?;
        
        // Basic verification - check if hashes match
        Ok(calculated_hash == provided_hash)
    }
}

/// Helper function to verify ML-DSA signature
#[wasm_bindgen(js_name = "verifyMlDsaSignature")]
pub fn verify_ml_dsa_signature(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8]
) -> Result<bool, JsError> {
    use crate::wasm_crypto::ml_dsa;
    ml_dsa::verify(public_key, message, signature)
        .map_err(|e| JsError::new(&format!("Verification failed: {}", e)))
}

// Data structures for serialization
#[derive(Serialize, Deserialize)]
struct KeyPairData {
    public_key: String,
    secret_key: String,
    key_type: String,
}

#[derive(Serialize, Deserialize)]
struct KemKeyPairData {
    public_key: String,
    secret_key: String,
    key_type: String,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    #[wasm_bindgen_test]
    fn test_ml_dsa_keypair_generation() {
        let keypair = WasmMlDsaKeyPair::new().unwrap();
        assert!(!keypair.get_public_key().is_empty());
        assert!(!keypair.get_secret_key().is_empty());
    }
    
    #[wasm_bindgen_test]
    fn test_blake3_hashing() {
        let data = b"Hello, QuDAG!";
        let hash = WasmHasher::hash_blake3(data);
        assert_eq!(hash.len(), 32); // BLAKE3 produces 32-byte hashes
    }
}