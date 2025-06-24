#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Quantum-resistant cryptographic primitives for QuDAG protocol.
//!
//! This module implements the following primitives:
//! - ML-KEM: Key encapsulation mechanism
//! - ML-DSA: Digital signature algorithm
//! - HQC: Hamming Quasi-Cyclic code-based encryption
//! - BLAKE3: Cryptographic hash function
//! - Quantum Fingerprint: Data fingerprinting using ML-DSA

pub mod encryption;
pub mod error;
pub mod fingerprint;
pub mod hash;
pub mod hqc;
pub mod kem;
// mod optimized;
pub mod ml_dsa;
pub mod ml_kem;
pub mod signature;

pub use error::CryptoError;
pub use fingerprint::{Fingerprint, FingerprintError};
pub use hash::HashFunction;
pub use hqc::{Hqc, Hqc128, Hqc192, Hqc256, HqcError, SecurityParameter};
pub use kem::{
    Ciphertext, KEMError, KeyEncapsulation, KeyPair, PublicKey, SecretKey, SharedSecret,
};
pub use ml_dsa::{MlDsa, MlDsaError, MlDsaKeyPair, MlDsaPublicKey};
pub use ml_kem::{Metrics as MlKemMetrics, MlKem768};
pub use signature::{DigitalSignature, SignatureError};
