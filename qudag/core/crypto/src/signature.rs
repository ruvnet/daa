//! ML-DSA (Dilithium) digital signature implementation.

use thiserror::Error;

/// Errors that can occur during signature operations.
#[derive(Debug, Error)]
pub enum SignatureError {
    /// Invalid public key format
    #[error("Invalid public key format")]
    InvalidPublicKey,

    /// Invalid secret key format
    #[error("Invalid secret key format")]
    InvalidSecretKey,

    /// Invalid signature format
    #[error("Invalid signature format")]
    InvalidSignature,

    /// Signature verification failed
    #[error("Signature verification failed")]
    VerificationFailed,

    /// Key generation failed
    #[error("Key generation failed")]
    KeyGenerationFailed,
}

/// ML-DSA public key.
#[derive(Debug, Clone)]
pub struct PublicKey(#[allow(dead_code)] Vec<u8>);

/// ML-DSA secret key.
#[derive(Debug, Clone)]
pub struct SecretKey(#[allow(dead_code)] Vec<u8>);

/// ML-DSA signature.
#[derive(Debug, Clone)]
pub struct Signature(#[allow(dead_code)] Vec<u8>);

/// ML-DSA digital signature trait.
pub trait DigitalSignature {
    /// Generate a new key pair.
    fn keygen() -> Result<(PublicKey, SecretKey), SignatureError>;

    /// Sign a message using a secret key.
    fn sign(secret_key: &SecretKey, message: &[u8]) -> Result<Signature, SignatureError>;

    /// Verify a signature using a public key and message.
    fn verify(
        public_key: &PublicKey,
        message: &[u8],
        signature: &Signature,
    ) -> Result<bool, SignatureError>;
}
