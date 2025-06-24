//! Cryptographic hash functions implementation.

use thiserror::Error;

/// Errors that can occur during hash operations.
#[derive(Debug, Error)]
pub enum HashError {
    /// Input data is too large
    #[error("Input data is too large")]
    InputTooLarge,

    /// Hash computation failed
    #[error("Hash computation failed")]
    ComputationFailed,
}

/// Hash function output.
///
/// # Examples
///
/// ```rust
/// use qudag_crypto::hash::Digest;
///
/// let digest = Digest(vec![0x12, 0x34, 0x56, 0x78]);
/// let bytes = digest.as_bytes();
/// assert_eq!(bytes, &[0x12, 0x34, 0x56, 0x78]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest(Vec<u8>);

impl Digest {
    /// Get the digest as a byte slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qudag_crypto::hash::Digest;
    ///
    /// let digest = Digest(vec![0x12, 0x34, 0x56, 0x78]);
    /// let bytes = digest.as_bytes();
    /// assert_eq!(bytes, &[0x12, 0x34, 0x56, 0x78]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert the digest into a vector of bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qudag_crypto::hash::Digest;
    ///
    /// let digest = Digest(vec![0x12, 0x34, 0x56, 0x78]);
    /// let bytes = digest.into_bytes();
    /// assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78]);
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Cryptographic hash function trait.
///
/// # Examples
///
/// ```rust
/// use qudag_crypto::hash::{HashFunction, HashError};
///
/// // Implement a simple hash function for demonstration
/// struct SimpleHash {
///     state: Vec<u8>,
/// }
///
/// impl HashFunction for SimpleHash {
///     fn new() -> Self {
///         Self { state: Vec::new() }
///     }
///     
///     fn update(&mut self, data: &[u8]) -> Result<(), HashError> {
///         self.state.extend_from_slice(data);
///         Ok(())
///     }
///     
///     fn finalize(self) -> Result<qudag_crypto::hash::Digest, HashError> {
///         // Simple checksum for example
///         let sum = self.state.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
///         Ok(qudag_crypto::hash::Digest(vec![sum]))
///     }
///     
///     fn hash(data: &[u8]) -> Result<qudag_crypto::hash::Digest, HashError> {
///         let mut hasher = Self::new();
///         hasher.update(data)?;
///         hasher.finalize()
///     }
/// }
///
/// // Use the hash function
/// let data = b"hello world";
/// let digest = SimpleHash::hash(data).unwrap();
/// ```
pub trait HashFunction {
    /// Create a new hash instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut hasher = Blake3Hash::new();
    /// ```
    fn new() -> Self;

    /// Update the hash state with input data.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut hasher = Blake3Hash::new();
    /// hasher.update(b"hello").unwrap();
    /// hasher.update(b" world").unwrap();
    /// ```
    fn update(&mut self, data: &[u8]) -> Result<(), HashError>;

    /// Finalize the hash computation and return the digest.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut hasher = Blake3Hash::new();
    /// hasher.update(b"hello world").unwrap();
    /// let digest = hasher.finalize().unwrap();
    /// ```
    fn finalize(self) -> Result<Digest, HashError>;

    /// Compute hash of input data in one step.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let digest = Blake3Hash::hash(b"hello world").unwrap();
    /// ```
    fn hash(data: &[u8]) -> Result<Digest, HashError>;
}
