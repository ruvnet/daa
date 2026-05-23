//! Quantum-resistant cryptography operations
//!
//! This module provides NIST-compliant post-quantum cryptography:
//! - ML-KEM-768 (FIPS 203) - Key encapsulation mechanism
//! - ML-DSA-65 (FIPS 204) - Digital signatures
//! - BLAKE3 - Cryptographic hashing

use kem::{Decapsulate, Encapsulate};
use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{
    DetachedSignature as PqDetachedSignatureTrait, PublicKey as PqPublicKeyTrait,
    SecretKey as PqSecretKeyTrait,
};
use rand::rngs::OsRng;

/// Key pair containing public and secret keys
#[napi(object)]
pub struct KeyPair {
    /// DER-encoded public key bytes
    pub public_key: Buffer,
    /// DER-encoded secret key bytes
    pub secret_key: Buffer,
}

/// Encapsulated secret containing ciphertext and shared secret
#[napi(object)]
pub struct EncapsulatedSecret {
    /// KEM ciphertext bytes
    pub ciphertext: Buffer,
    /// Shared secret bytes derived from encapsulation
    pub shared_secret: Buffer,
}

// ============================================================================
// ML-KEM-768 (NIST FIPS 203) - Quantum-Resistant Key Encapsulation
// ============================================================================

/// Generate a new ML-KEM-768 keypair
///
/// Returns a KeyPair with:
/// - Public key: 1184 bytes
/// - Secret key: 2400 bytes
///
/// # Example
/// ```javascript
/// const { mlkem768GenerateKeypair } = require('@daa/qudag-native');
/// const { publicKey, secretKey } = mlkem768GenerateKeypair();
/// ```
#[napi]
pub fn mlkem768_generate_keypair() -> Result<KeyPair> {
    let mut rng = OsRng;

    // Generate keypair - returns (DecapsulationKey, EncapsulationKey)
    let (dk, ek) = MlKem768::generate(&mut rng);

    // Convert to bytes using as_bytes() from EncodedSizeUser
    let public_key = ek.as_bytes();
    let secret_key = dk.as_bytes();

    Ok(KeyPair {
        public_key: public_key.to_vec().into(),
        secret_key: secret_key.to_vec().into(),
    })
}

/// Encapsulate a shared secret using a public key
///
/// # Arguments
/// * `public_key` - ML-KEM-768 public key (1184 bytes)
///
/// # Returns
/// EncapsulatedSecret containing:
/// - Ciphertext: 1088 bytes
/// - Shared secret: 32 bytes
#[napi]
pub fn mlkem768_encapsulate(public_key: Buffer) -> Result<EncapsulatedSecret> {
    if public_key.len() != 1184 {
        return Err(Error::from_reason(format!(
            "Invalid public key length: expected 1184 bytes, got {}",
            public_key.len()
        )));
    }

    let mut rng = OsRng;

    // Convert to fixed-size array
    let ek_array: [u8; 1184] = public_key
        .as_ref()
        .try_into()
        .map_err(|_| Error::from_reason("Failed to convert public key"))?;

    // Create encapsulation key from bytes
    type EncapKey = <MlKem768 as KemCore>::EncapsulationKey;
    let ek = EncapKey::from_bytes((&ek_array).into());

    // Encapsulate to generate shared secret and ciphertext
    let (ct, ss) = ek
        .encapsulate(&mut rng)
        .map_err(|_| Error::from_reason("Encapsulation failed"))?;

    Ok(EncapsulatedSecret {
        ciphertext: ct.to_vec().into(),
        shared_secret: ss.to_vec().into(),
    })
}

/// Decapsulate a shared secret using a secret key
///
/// # Arguments
/// * `ciphertext` - Encapsulated ciphertext (1088 bytes)
/// * `secret_key` - ML-KEM-768 secret key (2400 bytes)
///
/// # Returns
/// Shared secret (32 bytes)
#[napi]
pub fn mlkem768_decapsulate(ciphertext: Buffer, secret_key: Buffer) -> Result<Buffer> {
    if ciphertext.len() != 1088 {
        return Err(Error::from_reason(format!(
            "Invalid ciphertext length: expected 1088 bytes, got {}",
            ciphertext.len()
        )));
    }

    if secret_key.len() != 2400 {
        return Err(Error::from_reason(format!(
            "Invalid secret key length: expected 2400 bytes, got {}",
            secret_key.len()
        )));
    }

    // Convert to fixed-size arrays
    let dk_array: [u8; 2400] = secret_key
        .as_ref()
        .try_into()
        .map_err(|_| Error::from_reason("Failed to convert secret key"))?;
    let ct_array: [u8; 1088] = ciphertext
        .as_ref()
        .try_into()
        .map_err(|_| Error::from_reason("Failed to convert ciphertext"))?;

    // Create keys from bytes
    type DecapKey = <MlKem768 as KemCore>::DecapsulationKey;
    let dk = DecapKey::from_bytes((&dk_array).into());
    let ct = ct_array.into();

    // Decapsulate to recover shared secret
    let ss = dk
        .decapsulate(&ct)
        .map_err(|_| Error::from_reason("Decapsulation failed"))?;

    Ok(ss.to_vec().into())
}

// ============================================================================
// ML-DSA-65 (NIST FIPS 204) - Quantum-Resistant Digital Signatures
// ============================================================================

/// Generate a new ML-DSA-65 keypair
///
/// Returns a KeyPair with:
/// - Public key: 1952 bytes
/// - Secret key: 4032 bytes
///
/// # Example
/// ```javascript
/// const { mldsa65GenerateKeypair } = require('@daa/qudag-native');
/// const { publicKey, secretKey } = mldsa65GenerateKeypair();
/// ```
#[napi]
pub fn mldsa65_generate_keypair() -> Result<KeyPair> {
    // Generate ML-DSA-65 keypair using pqcrypto-dilithium
    let (pk, sk) = keypair();

    Ok(KeyPair {
        public_key: pk.as_bytes().to_vec().into(),
        secret_key: sk.as_bytes().to_vec().into(),
    })
}

/// Sign a message with ML-DSA-65
///
/// # Arguments
/// * `message` - Message to sign
/// * `secret_key` - ML-DSA-65 secret key (4032 bytes)
///
/// # Returns
/// Signature (3309 bytes)
#[napi]
pub fn mldsa65_sign(message: Buffer, secret_key: Buffer) -> Result<Buffer> {
    if secret_key.len() != 4032 {
        return Err(Error::from_reason(format!(
            "Invalid secret key length: expected 4032 bytes, got {}",
            secret_key.len()
        )));
    }

    // Parse secret key from bytes
    let sk = SecretKey::from_bytes(secret_key.as_ref())
        .map_err(|_| Error::from_reason("Invalid secret key format"))?;

    // Sign the message
    let signature = detached_sign(message.as_ref(), &sk);

    Ok(signature.as_bytes().to_vec().into())
}

/// Verify a signature with ML-DSA-65
///
/// # Arguments
/// * `message` - Original message
/// * `signature` - Signature to verify (3309 bytes)
/// * `public_key` - ML-DSA-65 public key (1952 bytes)
///
/// # Returns
/// true if signature is valid, false otherwise
#[napi]
pub fn mldsa65_verify(message: Buffer, signature: Buffer, public_key: Buffer) -> Result<bool> {
    if public_key.len() != 1952 {
        return Err(Error::from_reason(format!(
            "Invalid public key length: expected 1952 bytes, got {}",
            public_key.len()
        )));
    }

    if signature.len() != 3309 {
        return Err(Error::from_reason(format!(
            "Invalid signature length: expected 3309 bytes, got {}",
            signature.len()
        )));
    }

    // Parse public key and signature from bytes
    let pk = PublicKey::from_bytes(public_key.as_ref())
        .map_err(|_| Error::from_reason("Invalid public key format"))?;
    let sig = DetachedSignature::from_bytes(signature.as_ref())
        .map_err(|_| Error::from_reason("Invalid signature format"))?;

    // Verify the signature
    Ok(verify_detached_signature(&sig, message.as_ref(), &pk).is_ok())
}

// ============================================================================
// BLAKE3 - Cryptographic Hashing
// ============================================================================

/// Compute BLAKE3 hash of data
///
/// # Arguments
/// * `data` - Data to hash
///
/// # Returns
/// 32-byte hash
#[napi]
pub fn blake3_hash(data: Buffer) -> Result<Buffer> {
    let hash = blake3::hash(data.as_ref());
    Ok(hash.as_bytes().to_vec().into())
}

/// Compute BLAKE3 hash as hex string
///
/// # Arguments
/// * `data` - Data to hash
///
/// # Returns
/// 64-character hex string
#[napi]
pub fn blake3_hash_hex(data: Buffer) -> Result<String> {
    let hash = blake3::hash(data.as_ref());
    Ok(hash.to_hex().to_string())
}

/// Generate quantum-resistant fingerprint
///
/// # Arguments
/// * `data` - Data to fingerprint
///
/// # Returns
/// Fingerprint string in format "qf:{hash}"
#[napi]
pub fn quantum_fingerprint(data: Buffer) -> Result<String> {
    let hash = blake3::hash(data.as_ref());
    Ok(format!("qf:{}", hash.to_hex()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ML-KEM-768 Tests
    #[test]
    fn test_mlkem_keygen() {
        let keypair = mlkem768_generate_keypair().unwrap();
        assert_eq!(keypair.public_key.len(), 1184);
        assert_eq!(keypair.secret_key.len(), 2400);
    }

    #[test]
    fn test_mlkem_encapsulate_decapsulate() {
        let keypair = mlkem768_generate_keypair().unwrap();
        let encapsulated = mlkem768_encapsulate(keypair.public_key.clone()).unwrap();

        assert_eq!(encapsulated.ciphertext.len(), 1088);
        assert_eq!(encapsulated.shared_secret.len(), 32);

        let decapsulated_secret =
            mlkem768_decapsulate(encapsulated.ciphertext.clone(), keypair.secret_key.clone())
                .unwrap();

        assert_eq!(decapsulated_secret.len(), 32);
        assert_eq!(
            encapsulated.shared_secret.as_ref(),
            decapsulated_secret.as_ref(),
            "Shared secrets must match after encapsulation/decapsulation"
        );
    }

    #[test]
    fn test_mlkem_invalid_public_key_length() {
        let invalid_key = vec![0u8; 100].into();
        assert!(mlkem768_encapsulate(invalid_key).is_err());
    }

    #[test]
    fn test_mlkem_invalid_secret_key_length() {
        let invalid_ciphertext = vec![0u8; 1088].into();
        let invalid_key = vec![0u8; 100].into();
        assert!(mlkem768_decapsulate(invalid_ciphertext, invalid_key).is_err());
    }

    // ML-DSA Tests
    #[test]
    fn test_mldsa_keygen() {
        let keypair = mldsa65_generate_keypair().unwrap();
        assert_eq!(keypair.public_key.len(), 1952);
        assert_eq!(keypair.secret_key.len(), 4032);
    }

    #[test]
    fn test_mldsa_sign_verify() {
        let keypair = mldsa65_generate_keypair().unwrap();
        let message = b"Hello, quantum-resistant world!";

        // Sign the message
        let signature = mldsa65_sign(message.to_vec().into(), keypair.secret_key.clone()).unwrap();

        assert_eq!(signature.len(), 3309);

        // Verify valid signature
        let is_valid = mldsa65_verify(
            message.to_vec().into(),
            signature.clone(),
            keypair.public_key.clone(),
        )
        .unwrap();

        assert!(is_valid, "Valid signature must verify successfully");

        // Verify tampered message is rejected
        let tampered_message = b"Tampered message";
        let is_invalid = mldsa65_verify(
            tampered_message.to_vec().into(),
            signature,
            keypair.public_key,
        )
        .unwrap();

        assert!(!is_invalid, "Tampered message must fail verification");
    }

    // BLAKE3 Tests
    #[test]
    fn test_blake3() {
        let data = vec![1, 2, 3, 4, 5];
        let hash = blake3_hash(data.into()).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_blake3_hex() {
        let data = b"test data";
        let hash_hex = blake3_hash_hex(data.to_vec().into()).unwrap();
        assert_eq!(hash_hex.len(), 64);
        assert!(hash_hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_quantum_fingerprint() {
        let data = b"fingerprint test";
        let fingerprint = quantum_fingerprint(data.to_vec().into()).unwrap();
        assert!(fingerprint.starts_with("qf:"));
        assert_eq!(fingerprint.len(), 67);
    }

    #[test]
    fn test_blake3_consistency() {
        let data = b"consistency test";
        let hash1 = blake3_hash(data.to_vec().into()).unwrap();
        let hash2 = blake3_hash(data.to_vec().into()).unwrap();
        assert_eq!(hash1.as_ref(), hash2.as_ref());
    }
}
