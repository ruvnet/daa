//! QuDAG Native Node.js Bindings
//!
//! High-performance NAPI-rs bindings for QuDAG quantum-resistant cryptography.
//!
//! # Features
//!
//! - ML-KEM-768 key encapsulation mechanism
//! - ML-DSA digital signatures
//! - BLAKE3 cryptographic hashing
//! - Password vault with quantum-resistant encryption
//! - Zero-copy operations for maximum performance
//!
//! # Example
//!
//! ```javascript
//! const { MlKem768 } = require('@daa/qudag-native');
//!
//! const mlkem = new MlKem768();
//! const { publicKey, secretKey } = mlkem.generateKeypair();
//!
//! const { ciphertext, sharedSecret } = mlkem.encapsulate(publicKey);
//! const decryptedSecret = mlkem.decapsulate(ciphertext, secretKey);
//!
//! console.assert(sharedSecret.equals(decryptedSecret));
//! ```

#![deny(clippy::all)]
#![warn(missing_docs)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

mod crypto;
// mod vault;  // TODO: Fix async runtime issues
// mod exchange;  // TODO: Fix async runtime issues
mod utils;

pub use crypto::*;
// pub use vault::*;
// pub use exchange::*;
pub use utils::*;

/// Initialize the QuDAG native module
#[napi]
pub fn init() -> Result<String> {
    Ok("QuDAG Native v0.1.0 - High-performance quantum-resistant cryptography".to_string())
}

/// Get module version
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get module information
#[napi(object)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
}

/// Get detailed module information
#[napi]
pub fn get_module_info() -> ModuleInfo {
    ModuleInfo {
        name: "qudag-native".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Native Node.js bindings for QuDAG quantum-resistant cryptography".to_string(),
        features: vec![
            "ML-KEM-768".to_string(),
            "ML-DSA".to_string(),
            "BLAKE3".to_string(),
            "Password Vault".to_string(),
            "Zero-copy operations".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_init() {
        let msg = init().unwrap();
        assert!(msg.contains("QuDAG Native"));
    }
}
