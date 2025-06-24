#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Quantum-resistant password vault library for QuDAG.
//!
//! This library provides a secure password management system using QuDAG's
//! quantum-resistant cryptographic primitives and DAG-based storage.
//!
//! ## Features
//!
//! - **Quantum-Resistant Encryption**: Uses ML-KEM (Kyber) for key encapsulation
//!   and ML-DSA (Dilithium) for digital signatures
//! - **DAG-Based Storage**: Stores secrets as nodes in a directed acyclic graph
//! - **Memory Safety**: Automatic zeroization of sensitive data
//! - **Secure Key Derivation**: Argon2id for password-based key derivation
//!
//! ## Example
//!
//! ```rust,no_run
//! use qudag_vault_core::{Vault, VaultError};
//!
//! # fn main() -> Result<(), VaultError> {
//! // Create a new vault
//! let mut vault = Vault::create("vault.qdag", "master_password")?;
//!
//! // Add a secret
//! vault.add_secret("email/work", "user@example.com", Some("password123"))?;
//!
//! // Retrieve a secret
//! let secret = vault.get_secret("email/work")?;
//! println!("Username: {}", secret.username);
//!
//! // Export the vault
//! vault.export("backup.qdag")?;
//! # Ok(())
//! # }
//! ```

pub mod crypto;
pub mod dag_storage;
pub mod error;
pub mod kdf;
pub mod secret;
pub mod utils;
pub mod vault;

pub use error::{VaultError, VaultResult};
pub use secret::{SecretEntry, SecretMetadata};
pub use vault::Vault;

// Re-export commonly used types
pub use crypto::{VaultCrypto, VaultKeyPair};

// Re-export from utils for convenience
pub use utils::CharacterSet;
