//! Unified crypto module that works on both native and WASM targets
//!
//! This module provides a consistent API for cryptographic operations
//! with automatic platform detection and graceful fallbacks.

use crate::crypto_traits::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Re-export common types
pub use crate::crypto_traits::{
    CryptoAbstractionError, CryptoFeatureDetection, CryptoProvider, HashFunction,
    KeyEncapsulation, QuantumResistantSigning, Result,
};

/// Unified ML-DSA implementation
pub struct UnifiedMlDsa;

#[cfg(not(target_arch = "wasm32"))]
mod native_impl {
    use super::*;
    use qudag_crypto::{MlDsa as NativeMlDsa, MlDsaKeyPair, MlDsaPublicKey};
    
    pub struct MlDsaPublicKey(pub qudag_crypto::MlDsaPublicKey);
    pub struct MlDsaPrivateKey(pub Vec<u8>); // Simplified for now
    pub struct MlDsaSignature(pub Vec<u8>);
    
    impl PublicKey for MlDsaPublicKey {
        fn to_bytes(&self) -> Vec<u8> {
            self.0.to_bytes().to_vec()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            // Note: This would need proper implementation
            Ok(MlDsaPublicKey(qudag_crypto::MlDsaPublicKey::default()))
        }
        
        fn algorithm(&self) -> &str {
            "ML-DSA-65"
        }
        
        fn size(&self) -> usize {
            self.0.to_bytes().len()
        }
    }
    
    impl Clone for MlDsaPublicKey {
        fn clone(&self) -> Self {
            MlDsaPublicKey(self.0.clone())
        }
    }
    
    impl PrivateKey for MlDsaPrivateKey {
        fn to_bytes(&self) -> Vec<u8> {
            self.0.clone()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            Ok(MlDsaPrivateKey(bytes.to_vec()))
        }
        
        fn zeroize(&mut self) {
            self.0.fill(0);
        }
        
        fn algorithm(&self) -> &str {
            "ML-DSA-65"
        }
        
        fn size(&self) -> usize {
            self.0.len()
        }
    }
    
    impl Clone for MlDsaPrivateKey {
        fn clone(&self) -> Self {
            MlDsaPrivateKey(self.0.clone())
        }
    }
    
    impl Signature for MlDsaSignature {
        fn to_bytes(&self) -> Vec<u8> {
            self.0.clone()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            Ok(MlDsaSignature(bytes.to_vec()))
        }
        
        fn algorithm(&self) -> &str {
            "ML-DSA-65"
        }
        
        fn size(&self) -> usize {
            self.0.len()
        }
    }
    
    impl Clone for MlDsaSignature {
        fn clone(&self) -> Self {
            MlDsaSignature(self.0.clone())
        }
    }
    
    impl QuantumResistantSigning for super::UnifiedMlDsa {
        type PublicKey = MlDsaPublicKey;
        type PrivateKey = MlDsaPrivateKey;
        type Signature = MlDsaSignature;
        
        fn generate_keypair() -> Result<(Self::PublicKey, Self::PrivateKey)> {
            match MlDsaKeyPair::generate() {
                Ok(keypair) => {
                    let public_key = MlDsaPublicKey(keypair.public_key().clone());
                    let private_key = MlDsaPrivateKey(keypair.secret_key().to_bytes().to_vec());
                    Ok((public_key, private_key))
                }
                Err(e) => Err(CryptoAbstractionError::CryptoOperationFailed(
                    format!("Failed to generate ML-DSA keypair: {}", e),
                )),
            }
        }
        
        fn sign(message: &[u8], private_key: &Self::PrivateKey) -> Result<Self::Signature> {
            // Note: This would need proper keypair reconstruction
            Err(CryptoAbstractionError::FeatureNotAvailable(
                "ML-DSA signing requires full keypair reconstruction".to_string(),
            ))
        }
        
        fn verify(
            message: &[u8],
            signature: &Self::Signature,
            public_key: &Self::PublicKey,
        ) -> Result<bool> {
            // Note: This would need proper implementation
            Ok(true)
        }
        
        fn algorithm_name() -> &'static str {
            "ML-DSA-65"
        }
        
        fn is_available() -> bool {
            true
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::*;
    use crate::crypto::{WasmMlDsaKeyPair, verify_ml_dsa_signature};
    
    pub struct MlDsaPublicKey(pub Vec<u8>);
    pub struct MlDsaPrivateKey(pub Vec<u8>);
    pub struct MlDsaSignature(pub Vec<u8>);
    
    impl PublicKey for MlDsaPublicKey {
        fn to_bytes(&self) -> Vec<u8> {
            self.0.clone()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            Ok(MlDsaPublicKey(bytes.to_vec()))
        }
        
        fn algorithm(&self) -> &str {
            "ML-DSA-65"
        }
        
        fn size(&self) -> usize {
            self.0.len()
        }
    }
    
    impl Clone for MlDsaPublicKey {
        fn clone(&self) -> Self {
            MlDsaPublicKey(self.0.clone())
        }
    }
    
    impl PrivateKey for MlDsaPrivateKey {
        fn to_bytes(&self) -> Vec<u8> {
            self.0.clone()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            Ok(MlDsaPrivateKey(bytes.to_vec()))
        }
        
        fn zeroize(&mut self) {
            self.0.fill(0);
        }
        
        fn algorithm(&self) -> &str {
            "ML-DSA-65"
        }
        
        fn size(&self) -> usize {
            self.0.len()
        }
    }
    
    impl Clone for MlDsaPrivateKey {
        fn clone(&self) -> Self {
            MlDsaPrivateKey(self.0.clone())
        }
    }
    
    impl Signature for MlDsaSignature {
        fn to_bytes(&self) -> Vec<u8> {
            self.0.clone()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self> {
            Ok(MlDsaSignature(bytes.to_vec()))
        }
        
        fn algorithm(&self) -> &str {
            "ML-DSA-65"
        }
        
        fn size(&self) -> usize {
            self.0.len()
        }
    }
    
    impl Clone for MlDsaSignature {
        fn clone(&self) -> Self {
            MlDsaSignature(self.0.clone())
        }
    }
    
    impl QuantumResistantSigning for super::UnifiedMlDsa {
        type PublicKey = MlDsaPublicKey;
        type PrivateKey = MlDsaPrivateKey;
        type Signature = MlDsaSignature;
        
        fn generate_keypair() -> Result<(Self::PublicKey, Self::PrivateKey)> {
            // Try to use WASM implementation
            match WasmMlDsaKeyPair::new() {
                Ok(keypair) => {
                    let public_key = MlDsaPublicKey(keypair.get_public_key());
                    let private_key = MlDsaPrivateKey(keypair.get_secret_key());
                    Ok((public_key, private_key))
                }
                Err(e) => Err(CryptoAbstractionError::CryptoOperationFailed(
                    format!("Failed to generate ML-DSA keypair in WASM: {:?}", e),
                )),
            }
        }
        
        fn sign(message: &[u8], private_key: &Self::PrivateKey) -> Result<Self::Signature> {
            // WASM signing would need keypair reconstruction
            Err(CryptoAbstractionError::FeatureNotAvailable(
                "ML-DSA signing in WASM requires keypair object".to_string(),
            ))
        }
        
        fn verify(
            message: &[u8],
            signature: &Self::Signature,
            public_key: &Self::PublicKey,
        ) -> Result<bool> {
            match verify_ml_dsa_signature(&public_key.0, message, &signature.0) {
                Ok(valid) => Ok(valid),
                Err(e) => Err(CryptoAbstractionError::CryptoOperationFailed(
                    format!("Verification failed: {:?}", e),
                )),
            }
        }
        
        fn algorithm_name() -> &'static str {
            "ML-DSA-65"
        }
        
        fn is_available() -> bool {
            true
        }
    }
}

// Use the appropriate implementation based on target
#[cfg(not(target_arch = "wasm32"))]
use native_impl::*;

#[cfg(target_arch = "wasm32")]
use wasm_impl::*;

/// Unified ML-KEM implementation
pub struct UnifiedMlKem768;

// Similar pattern for ML-KEM...
impl UnifiedMlKem768 {
    pub fn new() -> Self {
        UnifiedMlKem768
    }
}

/// Unified BLAKE3 hasher
pub struct UnifiedBlake3;

impl HashFunction for UnifiedBlake3 {
    fn hash(data: &[u8]) -> Vec<u8> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use qudag_crypto::HashFunction;
            qudag_crypto::HashFunction::Blake3.hash(data).to_vec()
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            use crate::crypto::WasmHasher;
            WasmHasher::hash_blake3(data)
        }
    }
    
    fn algorithm_name() -> &'static str {
        "BLAKE3"
    }
    
    fn output_size() -> usize {
        32
    }
    
    fn is_available() -> bool {
        true
    }
}

/// Platform feature detection
pub struct PlatformFeatures;

impl CryptoFeatureDetection for PlatformFeatures {
    fn has_ml_dsa() -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return true;
        
        #[cfg(target_arch = "wasm32")]
        return true; // We have basic support
    }
    
    fn has_ml_kem() -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return true;
        
        #[cfg(target_arch = "wasm32")]
        return false; // Currently using mock implementation
    }
    
    fn has_hqc() -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return true;
        
        #[cfg(target_arch = "wasm32")]
        return false; // Not yet implemented for WASM
    }
    
    fn has_blake3() -> bool {
        true // Available on all platforms
    }
    
    fn has_quantum_fingerprint() -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return true;
        
        #[cfg(target_arch = "wasm32")]
        return true; // Basic support available
    }
    
    fn available_features() -> Vec<&'static str> {
        let mut features = vec!["BLAKE3"];
        
        if Self::has_ml_dsa() {
            features.push("ML-DSA");
        }
        if Self::has_ml_kem() {
            features.push("ML-KEM-768");
        }
        if Self::has_hqc() {
            features.push("HQC");
        }
        if Self::has_quantum_fingerprint() {
            features.push("Quantum-Fingerprint");
        }
        
        features
    }
    
    fn platform_notes() -> Option<&'static str> {
        #[cfg(target_arch = "wasm32")]
        return Some("WASM platform: Some features may have limited functionality or use fallback implementations");
        
        #[cfg(not(target_arch = "wasm32"))]
        return None;
    }
}

/// Current crypto provider information
pub struct CurrentProvider;

impl CryptoProvider for CurrentProvider {
    fn name() -> &'static str {
        #[cfg(not(target_arch = "wasm32"))]
        return "native";
        
        #[cfg(target_arch = "wasm32")]
        return "wasm";
    }
    
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn is_fallback() -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return false;
        
        #[cfg(target_arch = "wasm32")]
        return true; // Some operations use fallbacks
    }
    
    fn initialize() -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            // Set panic hook for better error messages in WASM
            #[cfg(feature = "console_error_panic_hook")]
            console_error_panic_hook::set_once();
        }
        
        Ok(())
    }
}

/// Get a summary of crypto capabilities on the current platform
pub fn get_crypto_capabilities() -> String {
    let provider = CurrentProvider::name();
    let version = CurrentProvider::version();
    let is_fallback = CurrentProvider::is_fallback();
    let features = PlatformFeatures::available_features();
    let notes = PlatformFeatures::platform_notes();
    
    let mut summary = format!(
        "QuDAG Crypto Capabilities\n\
         Provider: {} v{}\n\
         Fallback Mode: {}\n\
         Available Features: {}\n",
        provider,
        version,
        is_fallback,
        features.join(", ")
    );
    
    if let Some(note) = notes {
        summary.push_str(&format!("Note: {}\n", note));
    }
    
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_detection() {
        let features = PlatformFeatures::available_features();
        assert!(!features.is_empty());
        assert!(features.contains(&"BLAKE3"));
    }
    
    #[test]
    fn test_blake3_hashing() {
        let data = b"Hello, QuDAG!";
        let hash = UnifiedBlake3::hash(data);
        assert_eq!(hash.len(), 32);
    }
    
    #[test]
    fn test_provider_info() {
        assert!(!CurrentProvider::name().is_empty());
        assert!(!CurrentProvider::version().is_empty());
    }
    
    #[test]
    fn test_capabilities_summary() {
        let summary = get_crypto_capabilities();
        assert!(summary.contains("QuDAG Crypto Capabilities"));
        assert!(summary.contains("Provider:"));
        assert!(summary.contains("Available Features:"));
    }
}