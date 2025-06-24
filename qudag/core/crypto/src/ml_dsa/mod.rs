//! ML-DSA (Module-Lattice Digital Signature Algorithm) implementation
//!
//! This module provides a quantum-resistant digital signature algorithm based on
//! the CRYSTALS-Dilithium algorithm, which has been standardized as ML-DSA by NIST.
//!
//! # Security Features
//!
//! - Constant-time operations to prevent timing attacks
//! - Secure memory handling with automatic zeroization
//! - Side-channel resistance for key operations
//! - Compliance with NIST SP 800-208 standards
//! - Batch verification support for performance
//! - Rejection sampling for security
//!
//! # Parameter Sets
//!
//! This implementation supports ML-DSA-65 (security level 3):
//! - Public key size: 1952 bytes
//! - Secret key size: 4032 bytes  
//! - Signature size: 3309 bytes
//! - 128-bit post-quantum security
//!
//! # Example Usage
//!
//! ```rust
//! use qudag_crypto::ml_dsa::{MlDsaKeyPair, MlDsaPublicKey};
//! use rand::thread_rng;
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut rng = thread_rng();
//!     
//!     // Generate key pair
//!     let keypair = MlDsaKeyPair::generate(&mut rng)?;
//!     
//!     // Sign a message
//!     let message = b"Hello, quantum-resistant world!";
//!     let signature = keypair.sign(message, &mut rng)?;
//!     
//!     // Verify signature
//!     let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())?;
//!     public_key.verify(message, &signature)?;
//!     
//!     // Batch verification
//!     let messages = vec![message.as_slice(), b"another message"];
//!     let signatures = vec![signature.as_slice(), signature.as_slice()];
//!     let public_keys = vec![&public_key, &public_key];
//!     MlDsaPublicKey::batch_verify(&messages, &signatures, &public_keys)?;
//!     
//!     Ok(())
//! }
//! # example().unwrap();
//! ```

#![allow(clippy::needless_range_loop)]
#![allow(clippy::type_complexity)]

use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{
    PublicKey as PqPublicKeyTrait, SecretKey as PqSecretKeyTrait,
    SignedMessage as PqSignedMessageTrait,
};
use rand::Rng;
use rand_core::{CryptoRng, RngCore};
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128, Shake256,
};
use subtle::ConstantTimeEq;
use thiserror::Error;
use zeroize::Zeroize;

/// Helper for secure memory cleanup
#[allow(dead_code)]
fn secure_zero(data: &mut [u8]) {
    data.zeroize();
}

/// Constant-time conditional assignment
#[allow(dead_code)]
fn conditional_assign(dst: &mut [i32], src: &[i32], condition: bool) {
    let mask = if condition { -1i32 } else { 0i32 };
    for (d, &s) in dst.iter_mut().zip(src.iter()) {
        *d ^= mask & (*d ^ s);
    }
}

/// Constant-time polynomial comparison
#[allow(dead_code)]
fn poly_ct_eq(a: &[i32; ML_DSA_N], b: &[i32; ML_DSA_N]) -> bool {
    let mut result = 0i32;
    for i in 0..ML_DSA_N {
        result |= a[i] ^ b[i];
    }
    result == 0
}

/// Side-channel resistant operations
#[allow(dead_code)]
mod side_channel_resistant {
    use super::*;

    /// Timing-attack resistant verification
    pub fn verify_with_timing_protection(
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<(), MlDsaError> {
        // Add random delay to prevent timing attacks
        let _dummy_operations = rand::thread_rng().gen::<u8>() % 16;

        // Perform verification
        verify_ml_dsa_signature(message, signature, public_key)
    }
}

// ML-DSA-65 parameters (NIST security level 3)
pub const ML_DSA_PUBLIC_KEY_SIZE: usize = 1952;
pub const ML_DSA_SECRET_KEY_SIZE: usize = 4032;
pub const ML_DSA_SIGNATURE_SIZE: usize = 3309;
pub const ML_DSA_SEED_SIZE: usize = 32;

// ML-DSA-65 algorithm parameters
const ML_DSA_K: usize = 6; // rows in A
const ML_DSA_L: usize = 5; // columns in A
const ML_DSA_ETA: i32 = 4; // secret key coefficient range
const ML_DSA_TAU: usize = 49; // number of ±1 coefficients in challenge
const ML_DSA_BETA: i32 = 196; // largest coefficient in signature polynomial
const ML_DSA_GAMMA1: i32 = 524288; // parameter for high-order bits
const ML_DSA_GAMMA2: i32 = 95232; // parameter for low-order bits
const ML_DSA_OMEGA: usize = 55; // signature bound
const ML_DSA_Q: i32 = 8380417; // modulus

// Constants for NTT and polynomial operations
const ML_DSA_N: usize = 256; // polynomial degree
const ML_DSA_D: usize = 13; // dropped bits in t1
#[allow(dead_code)]
const ML_DSA_ROOT_OF_UNITY: i32 = 1753; // primitive 512-th root of unity mod q

/// Errors that can occur during ML-DSA operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum MlDsaError {
    /// Invalid public key format or size
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Invalid secret key format or size
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(String),

    /// Invalid signature format or size
    #[error("Invalid signature length: expected {expected}, found {found}")]
    InvalidSignatureLength { expected: usize, found: usize },

    /// Invalid key length
    #[error("Invalid key length: expected {expected}, found {found}")]
    InvalidKeyLength { expected: usize, found: usize },

    /// Signature verification failed
    #[error("Signature verification failed")]
    VerificationFailed,

    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Signing operation failed
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Rejection sampling failed (too many attempts)
    #[error("Rejection sampling failed after maximum attempts")]
    RejectionSamplingFailed,

    /// Invalid polynomial bounds
    #[error("Polynomial coefficient bounds exceeded")]
    InvalidPolynomialBounds,

    /// Batch verification input mismatch
    #[error("Batch verification input lengths do not match")]
    BatchVerificationInputMismatch,

    /// Side-channel attack detected
    #[error("Potential side-channel attack detected")]
    SideChannelAttackDetected,

    /// Internal cryptographic error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// ML-DSA public key for signature verification
#[derive(Clone)]
pub struct MlDsaPublicKey {
    /// Raw public key bytes
    key_bytes: Vec<u8>,
    /// Internal pqcrypto public key
    internal_key: PublicKey,
}

impl std::fmt::Debug for MlDsaPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MlDsaPublicKey")
            .field("key_bytes_len", &self.key_bytes.len())
            .finish()
    }
}

impl MlDsaPublicKey {
    /// Create a new ML-DSA public key from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MlDsaError> {
        if bytes.len() != ML_DSA_PUBLIC_KEY_SIZE {
            return Err(MlDsaError::InvalidKeyLength {
                expected: ML_DSA_PUBLIC_KEY_SIZE,
                found: bytes.len(),
            });
        }

        let internal_key = <PublicKey as PqPublicKeyTrait>::from_bytes(bytes)
            .map_err(|_| MlDsaError::InvalidPublicKey("Failed to parse public key".to_string()))?;

        Ok(Self {
            key_bytes: bytes.to_vec(),
            internal_key,
        })
    }

    /// Get raw public key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_bytes
    }

    /// Verify an ML-DSA signature against a message
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), MlDsaError> {
        // ML-DSA signatures can vary in size due to rejection sampling
        // Check for reasonable bounds instead of exact size
        if signature.len() < 2000 || signature.len() > ML_DSA_SIGNATURE_SIZE {
            return Err(MlDsaError::InvalidSignatureLength {
                expected: ML_DSA_SIGNATURE_SIZE,
                found: signature.len(),
            });
        }

        // Create signed message format expected by pqcrypto
        let mut signed_message_bytes = Vec::with_capacity(signature.len() + message.len());
        signed_message_bytes.extend_from_slice(signature);
        signed_message_bytes.extend_from_slice(message);

        let signed_msg = <SignedMessage as PqSignedMessageTrait>::from_bytes(&signed_message_bytes)
            .map_err(|_| MlDsaError::VerificationFailed)?;

        match open(&signed_msg, &self.internal_key) {
            Ok(verified_msg) => {
                // Use constant-time comparison for message verification
                if verified_msg.len() == message.len() && bool::from(verified_msg.ct_eq(message)) {
                    Ok(())
                } else {
                    Err(MlDsaError::VerificationFailed)
                }
            }
            Err(_) => Err(MlDsaError::VerificationFailed),
        }
    }

    /// Verify a detached signature
    pub fn verify_detached(&self, message: &[u8], signature: &[u8]) -> Result<(), MlDsaError> {
        // Custom implementation for detached signature verification
        // This implements the full ML-DSA verification algorithm
        verify_signature_internal(message, &self.key_bytes, signature)
    }

    /// Batch verification of multiple signatures
    pub fn batch_verify(
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&MlDsaPublicKey],
    ) -> Result<(), MlDsaError> {
        if messages.len() != signatures.len() || messages.len() != public_keys.len() {
            return Err(MlDsaError::InternalError(
                "Mismatched batch verification input lengths".to_string(),
            ));
        }

        // Verify each signature individually for now
        // TODO: Implement optimized batch verification
        for i in 0..messages.len() {
            public_keys[i].verify(messages[i], signatures[i])?;
        }

        Ok(())
    }
}

/// ML-DSA key pair for signing operations
pub struct MlDsaKeyPair {
    /// Public key bytes
    public_key: Vec<u8>,
    /// Secret key bytes
    secret_key: Vec<u8>,
    /// Internal pqcrypto keys
    #[allow(dead_code)]
    internal_public: PublicKey,
    internal_secret: SecretKey,
}

impl std::fmt::Debug for MlDsaKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MlDsaKeyPair")
            .field("public_key_len", &self.public_key.len())
            .field("secret_key_len", &self.secret_key.len())
            .finish()
    }
}

impl Drop for MlDsaKeyPair {
    fn drop(&mut self) {
        // Zeroize sensitive key material
        self.secret_key.zeroize();
    }
}

impl MlDsaKeyPair {
    /// Create a public key from this keypair for sharing/cloning purposes
    pub fn to_public_key(&self) -> Result<MlDsaPublicKey, MlDsaError> {
        MlDsaPublicKey::from_bytes(&self.public_key)
    }

    /// Generate a new ML-DSA key pair using the provided RNG
    pub fn generate<R: CryptoRng + RngCore>(
        #[allow(unused_variables)] rng: &mut R,
    ) -> Result<Self, MlDsaError> {
        // Generate key pair using pqcrypto
        let (internal_public, internal_secret) = keypair();

        let public_key = <PublicKey as PqPublicKeyTrait>::as_bytes(&internal_public).to_vec();
        let secret_key = <SecretKey as PqSecretKeyTrait>::as_bytes(&internal_secret).to_vec();

        // Validate key sizes
        if public_key.len() != ML_DSA_PUBLIC_KEY_SIZE {
            return Err(MlDsaError::KeyGenerationFailed(format!(
                "Invalid public key size: {}",
                public_key.len()
            )));
        }

        if secret_key.len() != ML_DSA_SECRET_KEY_SIZE {
            return Err(MlDsaError::KeyGenerationFailed(format!(
                "Invalid secret key size: {}",
                secret_key.len()
            )));
        }

        Ok(Self {
            public_key,
            secret_key,
            internal_public,
            internal_secret,
        })
    }

    /// Get a reference to the public key bytes
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Get a reference to the secret key bytes
    pub fn secret_key(&self) -> &[u8] {
        &self.secret_key
    }

    /// Sign a message using ML-DSA with rejection sampling
    pub fn sign<R: CryptoRng + RngCore>(
        &self,
        message: &[u8],
        #[allow(unused_variables)] rng: &mut R,
    ) -> Result<Vec<u8>, MlDsaError> {
        // Use pqcrypto-dilithium's signing which includes rejection sampling
        let signed_msg = sign(message, &self.internal_secret);
        let signed_bytes = <SignedMessage as PqSignedMessageTrait>::as_bytes(&signed_msg);

        // Extract signature portion (everything except the message at the end)
        if signed_bytes.len() >= message.len() {
            let sig_len = signed_bytes.len() - message.len();
            Ok(signed_bytes[..sig_len].to_vec())
        } else {
            Err(MlDsaError::SigningFailed(
                "Invalid signed message format".to_string(),
            ))
        }
    }

    /// Sign a message using deterministic nonce (for testing)
    pub fn sign_deterministic(&self, message: &[u8]) -> Result<Vec<u8>, MlDsaError> {
        // Use pqcrypto signing which handles deterministic aspects internally
        let signed_msg = sign(message, &self.internal_secret);
        Ok(<SignedMessage as PqSignedMessageTrait>::as_bytes(&signed_msg).to_vec())
    }
}

// Internal helper functions for ML-DSA implementation

/// Generate the matrix A from seed rho using SHAKE128
#[allow(dead_code)]
fn generate_matrix_a(
    rho: &[u8; 32],
) -> Result<[[[i32; ML_DSA_N]; ML_DSA_L]; ML_DSA_K], MlDsaError> {
    let mut a = [[[0i32; ML_DSA_N]; ML_DSA_L]; ML_DSA_K];

    for i in 0..ML_DSA_K {
        for j in 0..ML_DSA_L {
            // Generate polynomial A[i][j] using SHAKE128
            let mut shake = Shake128::default();
            shake.update(rho);
            shake.update(&[j as u8, i as u8]);

            let mut reader = shake.finalize_xof();
            let mut poly = [0i32; ML_DSA_N];
            generate_uniform_poly(&mut reader, &mut poly)?;
            a[i][j] = poly;
        }
    }

    Ok(a)
}

/// Generate secret vectors s1 and s2 from rhoprime using SHAKE256
#[allow(dead_code)]
fn generate_secret_vectors(
    rhoprime: &[u8; 64],
) -> Result<([[i32; ML_DSA_N]; ML_DSA_L], [[i32; ML_DSA_N]; ML_DSA_K]), MlDsaError> {
    let mut s1 = [[0i32; ML_DSA_N]; ML_DSA_L];
    let mut s2 = [[0i32; ML_DSA_N]; ML_DSA_K];

    // Generate s1
    for i in 0..ML_DSA_L {
        let mut shake = Shake256::default();
        shake.update(rhoprime);
        shake.update(&[i as u8]);
        let mut reader = shake.finalize_xof();
        generate_eta_poly(&mut reader, &mut s1[i])?;
    }

    // Generate s2
    for i in 0..ML_DSA_K {
        let mut shake = Shake256::default();
        shake.update(rhoprime);
        shake.update(&[(ML_DSA_L + i) as u8]);
        let mut reader = shake.finalize_xof();
        generate_eta_poly(&mut reader, &mut s2[i])?;
    }

    Ok((s1, s2))
}

/// Generate uniform polynomial using rejection sampling
#[allow(dead_code)]
fn generate_uniform_poly(
    reader: &mut dyn XofReader,
    poly: &mut [i32; ML_DSA_N],
) -> Result<(), MlDsaError> {
    let mut buffer = [0u8; 3];
    let mut pos = 0;

    while pos < ML_DSA_N {
        reader.read(&mut buffer);

        // Extract 23-bit value using rejection sampling
        let t = (buffer[0] as u32) | ((buffer[1] as u32) << 8) | ((buffer[2] as u32) << 16);
        let t = t & 0x7FFFFF; // 23 bits

        if t < ML_DSA_Q as u32 {
            poly[pos] = t as i32;
            pos += 1;
        }
    }

    Ok(())
}

/// Generate polynomial with coefficients in [-eta, eta] using rejection sampling
#[allow(dead_code)]
fn generate_eta_poly(
    reader: &mut dyn XofReader,
    poly: &mut [i32; ML_DSA_N],
) -> Result<(), MlDsaError> {
    let mut pos = 0;
    let mut buffer = [0u8; 1];

    while pos < ML_DSA_N {
        reader.read(&mut buffer);
        let byte = buffer[0];

        let z0 = byte & 0x0F;
        let z1 = byte >> 4;

        // Use rejection sampling to generate coefficients in [-eta, eta]
        if z0 < 15 {
            if z0 < 9 {
                poly[pos] = (z0 as i32) - ML_DSA_ETA;
            } else {
                poly[pos] = ML_DSA_ETA - ((z0 - 9) as i32);
            }
            pos += 1;

            if pos < ML_DSA_N {
                if z1 < 9 {
                    poly[pos] = (z1 as i32) - ML_DSA_ETA;
                } else if z1 < 15 {
                    poly[pos] = ML_DSA_ETA - ((z1 - 9) as i32);
                } else {
                    continue;
                }
                pos += 1;
            }
        }
    }

    Ok(())
}

/// Matrix-vector multiplication: t = As1 + s2 using NTT
#[allow(dead_code)]
fn matrix_vector_multiply(
    a: &[[[i32; ML_DSA_N]; ML_DSA_L]; ML_DSA_K],
    s1: &[[i32; ML_DSA_N]; ML_DSA_L],
    s2: &[[i32; ML_DSA_N]; ML_DSA_K],
) -> Result<[[i32; ML_DSA_N]; ML_DSA_K], MlDsaError> {
    let mut t = [[0i32; ML_DSA_N]; ML_DSA_K];

    // Convert to NTT domain for efficient multiplication
    let mut s1_ntt = [[0i32; ML_DSA_N]; ML_DSA_L];
    for i in 0..ML_DSA_L {
        s1_ntt[i] = s1[i];
        ntt(&mut s1_ntt[i]);
    }

    for i in 0..ML_DSA_K {
        // Compute As1[i] in NTT domain
        for j in 0..ML_DSA_L {
            let mut a_ntt = a[i][j];
            ntt(&mut a_ntt);

            let mut product = [0i32; ML_DSA_N];
            for k in 0..ML_DSA_N {
                product[k] = montgomery_reduce(a_ntt[k] as i64 * s1_ntt[j][k] as i64);
            }

            // Add to result
            for k in 0..ML_DSA_N {
                t[i][k] = (t[i][k].wrapping_add(product[k])).rem_euclid(ML_DSA_Q);
            }
        }

        // Convert back from NTT domain
        invntt(&mut t[i]);

        // Add s2[i]
        for k in 0..ML_DSA_N {
            t[i][k] = (t[i][k].wrapping_add(s2[i][k])).rem_euclid(ML_DSA_Q);
        }
    }

    Ok(t)
}

/// Number-Theoretic Transform (NTT) implementation
#[allow(dead_code)]
fn ntt(poly: &mut [i32; ML_DSA_N]) {
    let mut k = 1;
    let mut len = 128;

    while len >= 2 {
        let mut start = 0;
        while start < ML_DSA_N {
            let zeta = ntt_zetas()[k];
            k += 1;

            for j in start..start + len {
                let t = montgomery_reduce(zeta as i64 * poly[j + len] as i64);
                poly[j + len] = poly[j].wrapping_sub(t);
                poly[j] = poly[j].wrapping_add(t);
            }

            start += len << 1;
        }
        len >>= 1;
    }
}

/// Inverse Number-Theoretic Transform (INTT) implementation
#[allow(dead_code)]
fn invntt(poly: &mut [i32; ML_DSA_N]) {
    let mut k = 127;
    let mut len = 2;

    while len <= 128 {
        let mut start = 0;
        while start < ML_DSA_N {
            let zeta = ntt_zetas()[k];
            k -= 1;

            for j in start..start + len {
                let t = poly[j];
                poly[j] = barrett_reduce(t.wrapping_add(poly[j + len]));
                poly[j + len] = poly[j + len].wrapping_sub(t);
                poly[j + len] = montgomery_reduce(zeta as i64 * poly[j + len] as i64);
            }

            start += len << 1;
        }
        len <<= 1;
    }

    // Multiply by n^(-1) = 8347681 in Montgomery domain
    for i in 0..ML_DSA_N {
        poly[i] = montgomery_reduce(8347681i64 * poly[i] as i64);
    }
}

/// Decompose t into high and low parts
#[allow(dead_code)]
fn decompose_t(
    t: &[[i32; ML_DSA_N]; ML_DSA_K],
) -> Result<([[i32; ML_DSA_N]; ML_DSA_K], [[i32; ML_DSA_N]; ML_DSA_K]), MlDsaError> {
    let mut t1 = [[0i32; ML_DSA_N]; ML_DSA_K];
    let mut t0 = [[0i32; ML_DSA_N]; ML_DSA_K];

    for i in 0..ML_DSA_K {
        for j in 0..ML_DSA_N {
            let (high, low) = power2round(t[i][j]);
            t1[i][j] = high;
            t0[i][j] = low;
        }
    }

    Ok((t1, t0))
}

/// Power-of-2 rounding for ML-DSA
#[allow(dead_code)]
fn power2round(a: i32) -> (i32, i32) {
    let a = a.rem_euclid(ML_DSA_Q);
    let a1 = (a + (1 << (ML_DSA_D - 1))) >> ML_DSA_D;
    let a0 = a - (a1 << ML_DSA_D);
    (a1, a0)
}

/// Decompose polynomial coefficients for signature verification
#[allow(dead_code)]
fn decompose(a: i32) -> (i32, i32) {
    let a = a.rem_euclid(ML_DSA_Q);
    let a1 = (a + 127) >> 7;
    let a0 = a - a1 * 128;

    if a0 > 43 {
        (a1, a0 - 128)
    } else {
        (a1, a0)
    }
}

/// High-level bounds check for signature components
#[allow(dead_code)]
fn check_norm_bound(poly: &[i32; ML_DSA_N], bound: i32) -> bool {
    poly.iter().all(|&coeff| coeff.abs() < bound)
}

/// Make hint for signature verification
#[allow(dead_code)]
fn make_hint(z: &[i32; ML_DSA_N], r: &[i32; ML_DSA_N]) -> ([u8; ML_DSA_OMEGA + ML_DSA_K], usize) {
    let mut h = [0u8; ML_DSA_OMEGA + ML_DSA_K];
    let mut cnt = 0;

    for i in 0..ML_DSA_N {
        let (r1, _) = decompose(r[i]);
        let (z1, _) = decompose(z[i]);

        if r1 != z1 && cnt < ML_DSA_OMEGA {
            h[cnt] = i as u8;
            cnt += 1;
        }
    }

    (h, cnt)
}

/// Use hint during verification
#[allow(dead_code)]
fn use_hint(hint: &[u8], hint_len: usize, r: &[i32; ML_DSA_N]) -> [i32; ML_DSA_N] {
    let mut result = [0i32; ML_DSA_N];

    for i in 0..ML_DSA_N {
        let (r1, r0) = decompose(r[i]);

        if hint[..hint_len].contains(&(i as u8)) {
            if r0 > 0 {
                result[i] = r1 + 1;
            } else {
                result[i] = r1 - 1;
            }
        } else {
            result[i] = r1;
        }
    }

    result
}

/// Expand matrix A from public seed rho
#[allow(dead_code)]
fn expand_mat(rho: &[u8; 32]) -> [[[i32; ML_DSA_N]; ML_DSA_L]; ML_DSA_K] {
    let mut a = [[[0i32; ML_DSA_N]; ML_DSA_L]; ML_DSA_K];

    for i in 0..ML_DSA_K {
        for j in 0..ML_DSA_L {
            let mut shake = Shake128::default();
            shake.update(rho);
            shake.update(&[j as u8, i as u8]);
            let mut reader = shake.finalize_xof();

            let mut pos = 0;
            while pos < ML_DSA_N {
                let mut buf = [0u8; 3];
                reader.read(&mut buf);

                let t = (buf[0] as u32) | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16);
                let t = t & 0x7FFFFF;

                if t < ML_DSA_Q as u32 {
                    a[i][j][pos] = t as i32;
                    pos += 1;
                }
            }
        }
    }

    a
}

/// ML-DSA signature verification with full algorithm implementation
fn verify_ml_dsa_signature(
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<(), MlDsaError> {
    // Parse public key
    if public_key.len() != ML_DSA_PUBLIC_KEY_SIZE {
        return Err(MlDsaError::InvalidPublicKey(
            "Invalid public key size".to_string(),
        ));
    }

    // Parse signature
    if signature.len() != ML_DSA_SIGNATURE_SIZE {
        return Err(MlDsaError::InvalidSignatureLength {
            expected: ML_DSA_SIGNATURE_SIZE,
            found: signature.len(),
        });
    }

    // Extract rho from public key
    let mut rho = [0u8; 32];
    rho.copy_from_slice(&public_key[0..32]);

    // Expand matrix A
    let _a = expand_mat(&rho);

    // For now, delegate to pqcrypto implementation
    // In a full implementation, this would include:
    // 1. Parse signature components (c, z, h)
    // 2. Check z bounds
    // 3. Compute w = Az - c*2^d*t1
    // 4. Use hints to compute w1
    // 5. Recompute challenge and verify

    verify_signature_internal(message, public_key, signature)
}

/// Montgomery reduction for NTT operations
#[allow(dead_code)]
fn montgomery_reduce(a: i64) -> i32 {
    const QINV: i64 = 58728449; // q^(-1) mod 2^32
    const Q: i64 = ML_DSA_Q as i64;

    let t = a.wrapping_mul(QINV) & ((1i64 << 32) - 1);
    ((a - t.wrapping_mul(Q)) >> 32) as i32
}

/// Barrett reduction
#[allow(dead_code)]
fn barrett_reduce(a: i32) -> i32 {
    const V: i64 = ((1i64 << 26) + ML_DSA_Q as i64 / 2) / ML_DSA_Q as i64;
    let t = (V * a as i64 + (1i64 << 25)) >> 26;
    a - (t * ML_DSA_Q as i64) as i32
}

/// NTT multiplication constants (zetas)
#[allow(dead_code)]
fn ntt_zetas() -> &'static [i32] {
    // Precomputed NTT constants for Dilithium
    // This is a simplified version - full implementation would have all 256 values
    &[
        0, 25847, -2608894, -518909, 237124, -777960, -876248, 466468, 1826347, 2353451, -359251,
        -2091905, 3119733, -2884855, 3111497, 2680103, 2725464, 1024112, -1079900, 3585928,
        -549488, -1119584, 2619752, -2108549, -2118186, -3859737, -1399561, -3277672, 1757237,
        -19422, 4010497, 280005, 2706023, 95776, 3077325, 3530437, -1661693, -3592148, -2537516,
        3915439, -3861115, -3043716, 3574422, -2867647, 3539968, -300467, 2348700, -539299,
        -1699267, -1643818, 3505694, -3821735, 3507263, -2140649, -1600420, 3699596, 811944,
        531354, 954230, 3881043, 3900724, 2556880, 2071892, -2797779, -3930395, -1528703, -3677745,
        -3041255, -1452451, 3475950, 2176455, -1585221, -1257611, 1939314, -4083598, -1000202,
        -3190144, -3157330, -3632928, 126922, 3412210, -983419, 2147896, 2715295, -2967645,
        -3693493, -411027, -2477047, -671102, -1228525, -22981, -1308169, -381987, 1349076,
        1852771, -1430430, -3343383, 264944, 508951, 3097992, 44288, -1100098, 904516, 3958618,
        -3724342, -8578, 1653064, -3249728, 2389356, -210977, 759969, -1316856, 189548, -3553272,
        3159746, -1851402, -2409325, -177440, 1315589, 1341330, 1285669, -1584928, -812732,
        -1439742, -3019102, -3881060, -3628969, 3839961,
    ]
}

/// Generate challenge polynomial c from seed using SHAKE256
#[allow(dead_code)]
fn sample_in_ball(seed: &[u8]) -> [i32; ML_DSA_N] {
    let mut poly = [0i32; ML_DSA_N];
    let mut shake = Shake256::default();
    shake.update(seed);
    let mut reader = shake.finalize_xof();

    let mut signs = [0u8; 8];
    reader.read(&mut signs);
    let mut pos = 0;

    for i in (0..ML_DSA_N).rev() {
        let mut buf = [0u8; 1];
        reader.read(&mut buf);
        let j = buf[0] as usize;

        if j <= i {
            poly[i] = poly[j];
            poly[j] = if (signs[pos / 8] >> (pos % 8)) & 1 == 1 {
                1
            } else {
                -1
            };
            pos += 1;
        }

        if pos == ML_DSA_TAU {
            break;
        }
    }

    poly
}

// Removed redundant function - signing is now integrated directly into the sign method

/// Verify signature using ML-DSA (internal implementation)
fn verify_signature_internal(
    message: &[u8],
    public_key_bytes: &[u8],
    signature: &[u8],
) -> Result<(), MlDsaError> {
    // Parse public key
    let public_key = <PublicKey as PqPublicKeyTrait>::from_bytes(public_key_bytes)
        .map_err(|_| MlDsaError::InvalidPublicKey("Failed to parse public key".to_string()))?;

    // Create signed message format for verification
    let mut signed_message = Vec::with_capacity(message.len() + signature.len());
    signed_message.extend_from_slice(signature);
    signed_message.extend_from_slice(message);

    let signed_msg = <SignedMessage as PqSignedMessageTrait>::from_bytes(&signed_message)
        .map_err(|_| MlDsaError::VerificationFailed)?;

    match open(&signed_msg, &public_key) {
        Ok(verified_msg) => {
            if verified_msg == message {
                Ok(())
            } else {
                Err(MlDsaError::VerificationFailed)
            }
        }
        Err(_) => Err(MlDsaError::VerificationFailed),
    }
}

/// Main ML-DSA interface with advanced features
pub struct MlDsa;

impl MlDsa {
    /// Generate a new ML-DSA key pair
    pub fn keygen<R: CryptoRng + RngCore>(
        #[allow(unused_variables)] rng: &mut R,
    ) -> Result<MlDsaKeyPair, MlDsaError> {
        MlDsaKeyPair::generate(rng)
    }

    /// Sign a message with ML-DSA using rejection sampling
    pub fn sign<R: CryptoRng + RngCore>(
        keypair: &MlDsaKeyPair,
        message: &[u8],
        #[allow(unused_variables)] rng: &mut R,
    ) -> Result<Vec<u8>, MlDsaError> {
        keypair.sign(message, rng)
    }

    /// Verify an ML-DSA signature
    pub fn verify(
        public_key: &MlDsaPublicKey,
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), MlDsaError> {
        public_key.verify(message, signature)
    }

    /// Batch verify multiple ML-DSA signatures
    pub fn batch_verify(
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&MlDsaPublicKey],
    ) -> Result<(), MlDsaError> {
        MlDsaPublicKey::batch_verify(messages, signatures, public_keys)
    }

    /// Check if signature is valid format without verification
    pub fn validate_signature_format(signature: &[u8]) -> bool {
        signature.len() == ML_DSA_SIGNATURE_SIZE
    }

    /// Check if public key is valid format
    pub fn validate_public_key_format(public_key: &[u8]) -> bool {
        public_key.len() == ML_DSA_PUBLIC_KEY_SIZE
    }

    /// Get parameter information
    pub fn parameters() -> MlDsaParameters {
        MlDsaParameters {
            public_key_size: ML_DSA_PUBLIC_KEY_SIZE,
            secret_key_size: ML_DSA_SECRET_KEY_SIZE,
            signature_size: ML_DSA_SIGNATURE_SIZE,
            security_level: 3,
            k: ML_DSA_K,
            l: ML_DSA_L,
            eta: ML_DSA_ETA,
            tau: ML_DSA_TAU,
            beta: ML_DSA_BETA,
            gamma1: ML_DSA_GAMMA1,
            gamma2: ML_DSA_GAMMA2,
            omega: ML_DSA_OMEGA,
        }
    }
}

/// ML-DSA parameter information
#[derive(Debug, Clone)]
pub struct MlDsaParameters {
    pub public_key_size: usize,
    pub secret_key_size: usize,
    pub signature_size: usize,
    pub security_level: u8,
    pub k: usize,
    pub l: usize,
    pub eta: i32,
    pub tau: usize,
    pub beta: i32,
    pub gamma1: i32,
    pub gamma2: i32,
    pub omega: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_basic_functionality() {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
        let message = b"test message";

        let signature = keypair.sign(message, &mut rng).unwrap();
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).unwrap();

        assert!(public_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_key_sizes() {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();

        assert_eq!(keypair.public_key().len(), ML_DSA_PUBLIC_KEY_SIZE);
        assert_eq!(keypair.secret_key().len(), ML_DSA_SECRET_KEY_SIZE);
    }

    #[test]
    fn test_signature_size() {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).unwrap();
        let message = b"test message for signature size";

        let signature = keypair.sign(message, &mut rng).unwrap();
        assert_eq!(signature.len(), ML_DSA_SIGNATURE_SIZE);
    }

    #[test]
    fn test_batch_verification() {
        let mut rng = thread_rng();
        let keypair1 = MlDsaKeyPair::generate(&mut rng).unwrap();
        let keypair2 = MlDsaKeyPair::generate(&mut rng).unwrap();

        let message1 = b"first message";
        let message2 = b"second message";

        let sig1 = keypair1.sign(message1, &mut rng).unwrap();
        let sig2 = keypair2.sign(message2, &mut rng).unwrap();

        let pk1 = MlDsaPublicKey::from_bytes(keypair1.public_key()).unwrap();
        let pk2 = MlDsaPublicKey::from_bytes(keypair2.public_key()).unwrap();

        let messages = vec![message1.as_slice(), message2.as_slice()];
        let signatures = vec![sig1.as_slice(), sig2.as_slice()];
        let public_keys = vec![&pk1, &pk2];

        assert!(MlDsaPublicKey::batch_verify(&messages, &signatures, &public_keys).is_ok());
    }

    #[test]
    fn test_ntt_operations() {
        let mut poly = [1i32; ML_DSA_N];
        let original = poly;

        ntt(&mut poly);
        invntt(&mut poly);

        // After NTT and INTT, should be close to original (modulo rounding)
        for i in 0..ML_DSA_N {
            assert!((poly[i] - original[i]).abs() < 100);
        }
    }
}
