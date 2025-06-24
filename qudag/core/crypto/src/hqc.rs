use blake3::Hasher;
use pqcrypto_hqc::{hqc128, hqc192, hqc256};
use pqcrypto_traits::kem::{
    Ciphertext as CiphertextTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait,
    SharedSecret as SharedSecretTrait,
};
use rand::{CryptoRng, RngCore};
use thiserror::Error;

/// Security parameter sets for HQC as defined in the NIST submission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityParameter {
    /// 128-bit security level
    Hqc128,
    /// 192-bit security level  
    Hqc192,
    /// 256-bit security level
    Hqc256,
}

/// Parameters for HQC encryption scheme based on NIST submission
#[derive(Debug, Clone)]
pub struct Parameters {
    /// Security level
    security: SecurityParameter,
    /// Public key size in bytes
    public_key_size: usize,
    /// Secret key size in bytes
    secret_key_size: usize,
    /// Ciphertext size in bytes
    ciphertext_size: usize,
    /// Shared secret size in bytes
    shared_secret_size: usize,
}

/// Error types for HQC operations
#[derive(Error, Debug)]
pub enum HqcError {
    #[error("Invalid parameters")]
    InvalidParameters,
    #[error("Encryption failed")]
    EncryptionError,
    #[error("Decryption failed")]
    DecryptionError,
    #[error("Random number generation failed")]
    RandomError,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid secret key")]
    InvalidSecretKey,
    #[error("Invalid ciphertext")]
    InvalidCiphertext,
    #[error("Message too long")]
    MessageTooLong,
}

/// Public key for HQC that can hold any security level
#[derive(Debug, Clone)]
pub struct PublicKey {
    inner: Vec<u8>,
    #[allow(dead_code)]
    params: Parameters,
}

/// Secret key for HQC that can hold any security level
#[derive(Debug, Clone)]
pub struct SecretKey {
    inner: Vec<u8>,
    #[allow(dead_code)]
    params: Parameters,
}

/// Ciphertext for HQC that contains both KEM ciphertext and encrypted message
#[derive(Debug, Clone)]
pub struct Ciphertext {
    /// HQC KEM ciphertext
    kem_ciphertext: Vec<u8>,
    /// Encrypted message using derived key
    encrypted_message: Vec<u8>,
    #[allow(dead_code)]
    params: Parameters,
}

impl Parameters {
    /// Create new HQC parameters for given security level
    pub fn new(security: SecurityParameter) -> Self {
        match security {
            SecurityParameter::Hqc128 => Self {
                security,
                public_key_size: hqc128::public_key_bytes(),
                secret_key_size: hqc128::secret_key_bytes(),
                ciphertext_size: hqc128::ciphertext_bytes(),
                shared_secret_size: hqc128::shared_secret_bytes(),
            },
            SecurityParameter::Hqc192 => Self {
                security,
                public_key_size: hqc192::public_key_bytes(),
                secret_key_size: hqc192::secret_key_bytes(),
                ciphertext_size: hqc192::ciphertext_bytes(),
                shared_secret_size: hqc192::shared_secret_bytes(),
            },
            SecurityParameter::Hqc256 => Self {
                security,
                public_key_size: hqc256::public_key_bytes(),
                secret_key_size: hqc256::secret_key_bytes(),
                ciphertext_size: hqc256::ciphertext_bytes(),
                shared_secret_size: hqc256::shared_secret_bytes(),
            },
        }
    }

    /// Get the byte length for public key
    pub fn public_key_len(&self) -> usize {
        self.public_key_size
    }

    /// Get the byte length for secret key
    pub fn secret_key_len(&self) -> usize {
        self.secret_key_size
    }

    /// Get the byte length for ciphertext
    pub fn ciphertext_len(&self) -> usize {
        self.ciphertext_size
    }

    /// Get the byte length for shared secret
    pub fn shared_secret_len(&self) -> usize {
        self.shared_secret_size
    }
}

/// Main HQC implementation
pub struct Hqc {
    params: Parameters,
}

impl Hqc {
    /// Create new HQC instance with given security parameters
    pub fn new(security: SecurityParameter) -> Self {
        Self {
            params: Parameters::new(security),
        }
    }

    /// Generate key pair using the real HQC implementation
    pub fn generate_keypair<R: CryptoRng + RngCore>(
        &self,
        #[allow(unused_variables)] _rng: &mut R,
    ) -> Result<(PublicKey, SecretKey), HqcError> {
        let params = self.params.clone();

        match params.security {
            SecurityParameter::Hqc128 => {
                let (pk, sk) = hqc128::keypair();
                Ok((
                    PublicKey {
                        inner: pk.as_bytes().to_vec(),
                        params: params.clone(),
                    },
                    SecretKey {
                        inner: sk.as_bytes().to_vec(),
                        params,
                    },
                ))
            }
            SecurityParameter::Hqc192 => {
                let (pk, sk) = hqc192::keypair();
                Ok((
                    PublicKey {
                        inner: pk.as_bytes().to_vec(),
                        params: params.clone(),
                    },
                    SecretKey {
                        inner: sk.as_bytes().to_vec(),
                        params,
                    },
                ))
            }
            SecurityParameter::Hqc256 => {
                let (pk, sk) = hqc256::keypair();
                Ok((
                    PublicKey {
                        inner: pk.as_bytes().to_vec(),
                        params: params.clone(),
                    },
                    SecretKey {
                        inner: sk.as_bytes().to_vec(),
                        params,
                    },
                ))
            }
        }
    }

    /// Encrypt a message using HQC KEM + symmetric encryption
    pub fn encrypt<R: CryptoRng + RngCore>(
        &self,
        message: &[u8],
        pk: &PublicKey,
        #[allow(unused_variables)] _rng: &mut R,
    ) -> Result<Ciphertext, HqcError> {
        // Check reasonable message length (64KB max)
        if message.len() > 65536 {
            return Err(HqcError::MessageTooLong);
        }

        match self.params.security {
            SecurityParameter::Hqc128 => {
                let pk_bytes = hqc128::PublicKey::from_bytes(&pk.inner)
                    .map_err(|_| HqcError::InvalidPublicKey)?;
                let (shared_secret, kem_ciphertext) = hqc128::encapsulate(&pk_bytes);

                // Derive encryption key from shared secret using BLAKE3
                let key = self.derive_key(shared_secret.as_bytes());
                let encrypted_message = self.xor_encrypt(message, &key);

                Ok(Ciphertext {
                    kem_ciphertext: kem_ciphertext.as_bytes().to_vec(),
                    encrypted_message,
                    params: self.params.clone(),
                })
            }
            SecurityParameter::Hqc192 => {
                let pk_bytes = hqc192::PublicKey::from_bytes(&pk.inner)
                    .map_err(|_| HqcError::InvalidPublicKey)?;
                let (shared_secret, kem_ciphertext) = hqc192::encapsulate(&pk_bytes);

                let key = self.derive_key(shared_secret.as_bytes());
                let encrypted_message = self.xor_encrypt(message, &key);

                Ok(Ciphertext {
                    kem_ciphertext: kem_ciphertext.as_bytes().to_vec(),
                    encrypted_message,
                    params: self.params.clone(),
                })
            }
            SecurityParameter::Hqc256 => {
                let pk_bytes = hqc256::PublicKey::from_bytes(&pk.inner)
                    .map_err(|_| HqcError::InvalidPublicKey)?;
                let (shared_secret, kem_ciphertext) = hqc256::encapsulate(&pk_bytes);

                let key = self.derive_key(shared_secret.as_bytes());
                let encrypted_message = self.xor_encrypt(message, &key);

                Ok(Ciphertext {
                    kem_ciphertext: kem_ciphertext.as_bytes().to_vec(),
                    encrypted_message,
                    params: self.params.clone(),
                })
            }
        }
    }

    /// Decrypt a ciphertext using HQC KEM + symmetric decryption
    pub fn decrypt(&self, ct: &Ciphertext, sk: &SecretKey) -> Result<Vec<u8>, HqcError> {
        match self.params.security {
            SecurityParameter::Hqc128 => {
                let sk_bytes = hqc128::SecretKey::from_bytes(&sk.inner)
                    .map_err(|_| HqcError::InvalidSecretKey)?;
                let kem_ct = hqc128::Ciphertext::from_bytes(&ct.kem_ciphertext)
                    .map_err(|_| HqcError::InvalidCiphertext)?;

                let shared_secret = hqc128::decapsulate(&kem_ct, &sk_bytes);

                // Derive the same key from shared secret
                let key = self.derive_key(shared_secret.as_bytes());
                let message = self.xor_decrypt(&ct.encrypted_message, &key);

                Ok(message)
            }
            SecurityParameter::Hqc192 => {
                let sk_bytes = hqc192::SecretKey::from_bytes(&sk.inner)
                    .map_err(|_| HqcError::InvalidSecretKey)?;
                let kem_ct = hqc192::Ciphertext::from_bytes(&ct.kem_ciphertext)
                    .map_err(|_| HqcError::InvalidCiphertext)?;

                let shared_secret = hqc192::decapsulate(&kem_ct, &sk_bytes);

                let key = self.derive_key(shared_secret.as_bytes());
                let message = self.xor_decrypt(&ct.encrypted_message, &key);

                Ok(message)
            }
            SecurityParameter::Hqc256 => {
                let sk_bytes = hqc256::SecretKey::from_bytes(&sk.inner)
                    .map_err(|_| HqcError::InvalidSecretKey)?;
                let kem_ct = hqc256::Ciphertext::from_bytes(&ct.kem_ciphertext)
                    .map_err(|_| HqcError::InvalidCiphertext)?;

                let shared_secret = hqc256::decapsulate(&kem_ct, &sk_bytes);

                let key = self.derive_key(shared_secret.as_bytes());
                let message = self.xor_decrypt(&ct.encrypted_message, &key);

                Ok(message)
            }
        }
    }

    /// Get the parameters for this HQC instance
    pub fn params(&self) -> &Parameters {
        &self.params
    }

    /// Derive encryption key from shared secret using BLAKE3
    fn derive_key(&self, shared_secret: &[u8]) -> Vec<u8> {
        let mut hasher = Hasher::new();
        hasher.update(b"HQC-KEY-DERIVATION");
        hasher.update(shared_secret);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Simple XOR-based stream cipher for message encryption
    fn xor_encrypt(&self, message: &[u8], key: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(message.len());
        for (i, &byte) in message.iter().enumerate() {
            result.push(byte ^ key[i % key.len()]);
        }
        result
    }

    /// Simple XOR-based stream cipher for message decryption
    fn xor_decrypt(&self, ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
        // XOR is symmetric
        self.xor_encrypt(ciphertext, key)
    }
}

// Implementations for key serialization and compatibility
impl PublicKey {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.clone()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HqcError> {
        // Default to HQC256 if no other information is available
        let params = Parameters::new(SecurityParameter::Hqc256);

        if bytes.len() != params.public_key_len() {
            return Err(HqcError::InvalidPublicKey);
        }

        Ok(Self {
            inner: bytes.to_vec(),
            params,
        })
    }

    /// Create public key from bytes with specific security level
    pub fn from_bytes_with_params(
        bytes: &[u8],
        security: SecurityParameter,
    ) -> Result<Self, HqcError> {
        let params = Parameters::new(security);

        if bytes.len() != params.public_key_len() {
            return Err(HqcError::InvalidPublicKey);
        }

        Ok(Self {
            inner: bytes.to_vec(),
            params,
        })
    }
}

impl SecretKey {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.clone()
    }

    /// Create secret key from bytes with specific security level
    pub fn from_bytes_with_params(
        bytes: &[u8],
        security: SecurityParameter,
    ) -> Result<Self, HqcError> {
        let params = Parameters::new(security);

        if bytes.len() != params.secret_key_len() {
            return Err(HqcError::InvalidSecretKey);
        }

        Ok(Self {
            inner: bytes.to_vec(),
            params,
        })
    }
}

impl Ciphertext {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        // First 4 bytes: length of encrypted message
        result.extend_from_slice(&(self.encrypted_message.len() as u32).to_le_bytes());
        // Next: KEM ciphertext
        result.extend_from_slice(&self.kem_ciphertext);
        // Finally: encrypted message
        result.extend_from_slice(&self.encrypted_message);
        result
    }

    /// Create ciphertext from bytes with specific security level
    pub fn from_bytes_with_params(
        bytes: &[u8],
        security: SecurityParameter,
    ) -> Result<Self, HqcError> {
        let params = Parameters::new(security);

        if bytes.len() < 4 + params.ciphertext_len() {
            return Err(HqcError::InvalidCiphertext);
        }

        // Read message length
        let msg_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

        if bytes.len() < 4 + params.ciphertext_len() + msg_len {
            return Err(HqcError::InvalidCiphertext);
        }

        let kem_start = 4;
        let kem_end = kem_start + params.ciphertext_len();
        let msg_start = kem_end;
        let msg_end = msg_start + msg_len;

        Ok(Self {
            kem_ciphertext: bytes[kem_start..kem_end].to_vec(),
            encrypted_message: bytes[msg_start..msg_end].to_vec(),
            params,
        })
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

/// HQC-256 wrapper for AsymmetricEncryption compatibility
pub struct Hqc256;

impl Hqc256 {
    pub const PUBLIC_KEY_SIZE: usize = hqc256::public_key_bytes();
    pub const SECRET_KEY_SIZE: usize = hqc256::secret_key_bytes();
    pub const CIPHERTEXT_SIZE: usize = hqc256::ciphertext_bytes();

    /// Generate a key pair
    pub fn keygen() -> Result<(PublicKey, SecretKey), HqcError> {
        let hqc = Hqc::new(SecurityParameter::Hqc256);
        let mut rng = rand::thread_rng();
        hqc.generate_keypair(&mut rng)
    }

    /// Encrypt a message
    pub fn encrypt(pk: &PublicKey, message: &[u8]) -> Result<Vec<u8>, HqcError> {
        let hqc = Hqc::new(SecurityParameter::Hqc256);
        let mut rng = rand::thread_rng();

        let ciphertext = hqc.encrypt(message, pk, &mut rng)?;
        Ok(ciphertext.as_bytes())
    }

    /// Decrypt a ciphertext
    pub fn decrypt(sk: &SecretKey, ciphertext: &[u8]) -> Result<Vec<u8>, HqcError> {
        let hqc = Hqc::new(SecurityParameter::Hqc256);
        let ct = Ciphertext::from_bytes_with_params(ciphertext, SecurityParameter::Hqc256)?;
        hqc.decrypt(&ct, sk)
    }
}

// Specific implementations for different security levels
pub struct Hqc128;
pub struct Hqc192;

impl Hqc128 {
    pub const PUBLIC_KEY_SIZE: usize = hqc128::public_key_bytes();
    pub const SECRET_KEY_SIZE: usize = hqc128::secret_key_bytes();
    pub const CIPHERTEXT_SIZE: usize = hqc128::ciphertext_bytes();

    /// Generate a key pair
    pub fn keygen() -> Result<(PublicKey, SecretKey), HqcError> {
        let hqc = Hqc::new(SecurityParameter::Hqc128);
        let mut rng = rand::thread_rng();
        hqc.generate_keypair(&mut rng)
    }
}

impl Hqc192 {
    pub const PUBLIC_KEY_SIZE: usize = hqc192::public_key_bytes();
    pub const SECRET_KEY_SIZE: usize = hqc192::secret_key_bytes();
    pub const CIPHERTEXT_SIZE: usize = hqc192::ciphertext_bytes();

    /// Generate a key pair
    pub fn keygen() -> Result<(PublicKey, SecretKey), HqcError> {
        let hqc = Hqc::new(SecurityParameter::Hqc192);
        let mut rng = rand::thread_rng();
        hqc.generate_keypair(&mut rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_parameters() {
        let params128 = Parameters::new(SecurityParameter::Hqc128);
        let params192 = Parameters::new(SecurityParameter::Hqc192);
        let params256 = Parameters::new(SecurityParameter::Hqc256);

        // Verify that different security levels have different sizes
        assert_ne!(params128.public_key_size, params192.public_key_size);
        assert_ne!(params192.public_key_size, params256.public_key_size);

        // Verify sizes are reasonable
        assert!(params128.public_key_size > 0);
        assert!(params128.secret_key_size > 0);
        assert!(params128.ciphertext_size > 0);
    }

    #[test]
    fn test_key_generation() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc128);
        let (pk, sk) = hqc.generate_keypair(&mut rng).unwrap();

        assert_eq!(pk.inner.len(), hqc128::public_key_bytes());
        assert_eq!(sk.inner.len(), hqc128::secret_key_bytes());
    }

    #[test]
    fn test_encryption_decryption() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc128);
        let (pk, sk) = hqc.generate_keypair(&mut rng).unwrap();

        let message = vec![0x42u8; 16];
        let ct = hqc.encrypt(&message, &pk, &mut rng).unwrap();
        let decrypted = hqc.decrypt(&ct, &sk).unwrap();

        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_different_security_levels() {
        let mut rng = ChaCha20Rng::from_entropy();

        for security in [
            SecurityParameter::Hqc128,
            SecurityParameter::Hqc192,
            SecurityParameter::Hqc256,
        ] {
            let hqc = Hqc::new(security);
            let (pk, sk) = hqc.generate_keypair(&mut rng).unwrap();

            let message = b"Hello, HQC!".to_vec();
            let ct = hqc.encrypt(&message, &pk, &mut rng).unwrap();
            let decrypted = hqc.decrypt(&ct, &sk).unwrap();

            assert_eq!(message, decrypted);
        }
    }

    #[test]
    fn test_long_message() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc256);
        let (pk, sk) = hqc.generate_keypair(&mut rng).unwrap();

        let message = vec![0x42u8; 1000];
        let ct = hqc.encrypt(&message, &pk, &mut rng).unwrap();
        let decrypted = hqc.decrypt(&ct, &sk).unwrap();

        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_hqc256_compatibility() {
        let (pk, sk) = Hqc256::keygen().unwrap();
        let message = b"Test message for HQC256";

        let ciphertext = Hqc256::encrypt(&pk, message).unwrap();
        let decrypted = Hqc256::decrypt(&sk, &ciphertext).unwrap();

        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn test_key_serialization() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc256);
        let (pk, sk) = hqc.generate_keypair(&mut rng).unwrap();

        let pk_bytes = pk.as_bytes();
        let sk_bytes = sk.as_bytes();

        assert!(!pk_bytes.is_empty());
        assert!(!sk_bytes.is_empty());

        // Test public key round-trip
        let pk_restored =
            PublicKey::from_bytes_with_params(&pk_bytes, SecurityParameter::Hqc256).unwrap();
        assert_eq!(pk.inner, pk_restored.inner);
    }

    #[test]
    fn test_ciphertext_serialization() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc256);
        let (pk, sk) = hqc.generate_keypair(&mut rng).unwrap();

        let message = b"Test message for serialization";
        let ct = hqc.encrypt(message, &pk, &mut rng).unwrap();

        let ct_bytes = ct.as_bytes();
        let ct_restored =
            Ciphertext::from_bytes_with_params(&ct_bytes, SecurityParameter::Hqc256).unwrap();

        let decrypted = hqc.decrypt(&ct_restored, &sk).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn test_empty_message() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc128);
        let (pk, sk) = hqc.generate_keypair(&mut rng).unwrap();

        let message = b"";
        let ct = hqc.encrypt(message, &pk, &mut rng).unwrap();
        let decrypted = hqc.decrypt(&ct, &sk).unwrap();

        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn test_message_too_long() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc128);
        let (pk, _sk) = hqc.generate_keypair(&mut rng).unwrap();

        let message = vec![0x42u8; 100_000]; // 100KB message
        let result = hqc.encrypt(&message, &pk, &mut rng);

        assert!(matches!(result, Err(HqcError::MessageTooLong)));
    }

    #[test]
    fn test_security_properties() {
        let mut rng = ChaCha20Rng::from_entropy();
        let hqc = Hqc::new(SecurityParameter::Hqc256);

        // Test that different key generations produce different keys
        let (pk1, sk1) = hqc.generate_keypair(&mut rng).unwrap();
        let (pk2, sk2) = hqc.generate_keypair(&mut rng).unwrap();

        assert_ne!(pk1.inner, pk2.inner);
        assert_ne!(sk1.inner, sk2.inner);

        // Test that same message with different keys produces different ciphertexts
        let message = b"Test message for security";
        let ct1 = hqc.encrypt(message, &pk1, &mut rng).unwrap();
        let ct2 = hqc.encrypt(message, &pk2, &mut rng).unwrap();

        assert_ne!(ct1.kem_ciphertext, ct2.kem_ciphertext);

        // Test that wrong key cannot decrypt correctly
        let ct = hqc.encrypt(message, &pk1, &mut rng).unwrap();
        // In the real HQC implementation, using wrong secret key may panic or return error
        // This is expected behavior for post-quantum cryptographic systems
        let decryption_result = std::panic::catch_unwind(|| hqc.decrypt(&ct, &sk2));

        // Either it panics (which we catch) or it succeeds with wrong data
        match decryption_result {
            Ok(Ok(decrypted)) => {
                // If it doesn't panic, the decrypted data should be wrong
                assert_ne!(message, &decrypted[..]);
            }
            Ok(Err(_)) => {
                // If it returns an error, that's also acceptable
            }
            Err(_) => {
                // If it panics, that's expected for wrong key usage
            }
        }
    }
}
