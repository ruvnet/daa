use thiserror::Error;

/// Errors that can occur during cryptographic operations
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid key size")]
    InvalidKeySize,

    #[error("Invalid signature size")]
    InvalidSignatureSize,

    #[error("Invalid ciphertext size")]
    InvalidCiphertextSize,

    #[error("Invalid shared secret size")]
    InvalidSharedSecretSize,

    #[error("Key generation failed")]
    KeyGenerationError,

    #[error("Signature verification failed")]
    VerificationError,

    #[error("Encryption failed")]
    EncryptionError,

    #[error("Decryption failed")]
    DecryptionError,

    #[error("Invalid message size")]
    InvalidMessageSize,

    #[error("Invalid parameters")]
    InvalidParameters,

    #[error("Operation not supported")]
    UnsupportedOperation,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Invalid length")]
    InvalidLength,

    #[error("Encapsulation failed")]
    EncapsulationError,

    #[error("Decapsulation failed")]
    DecapsulationError,
}
