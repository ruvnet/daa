use crate::ml_dsa::{MlDsaError, MlDsaKeyPair, MlDsaPublicKey};
use blake3::Hasher;
use rand_core::{CryptoRng, RngCore};
use subtle::{Choice, ConstantTimeEq};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Errors that can occur during fingerprint operations
#[derive(Debug, Error)]
pub enum FingerprintError {
    #[error("Invalid fingerprint data")]
    InvalidData,
    #[error("Failed to generate fingerprint")]
    GenerationFailed,
    #[error("Failed to verify fingerprint")]
    VerificationFailed,
    #[error("ML-DSA error: {0}")]
    MlDsaError(#[from] MlDsaError),
}

/// A quantum-resistant fingerprint using ML-DSA signatures
///
/// # Examples
///
/// ```rust
/// use qudag_crypto::fingerprint::Fingerprint;
/// use rand::thread_rng;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut rng = thread_rng();
///     let data = b"some data to fingerprint";
///     
///     // Generate a fingerprint
///     let (fingerprint, public_key) = Fingerprint::generate(data, &mut rng)?;
///     
///     // Verify the fingerprint
///     fingerprint.verify(&public_key)?;
///     
///     // Access the fingerprint data
///     let fp_data = fingerprint.data();
///     assert_eq!(fp_data.len(), 64); // BLAKE3 outputs 64 bytes in XOF mode
///     
///     Ok(())
/// }
/// # example().unwrap();
/// ```
#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct Fingerprint {
    /// The actual fingerprint data
    data: Vec<u8>,
    /// The ML-DSA signature over the fingerprint data
    signature: Vec<u8>,
}

impl Fingerprint {
    /// Generate a new fingerprint from the given data using ML-DSA
    pub fn generate<R: CryptoRng + RngCore>(
        data: &[u8],
        rng: &mut R,
    ) -> Result<(Self, MlDsaPublicKey), FingerprintError> {
        // Generate ML-DSA keypair for signing
        let keypair = MlDsaKeyPair::generate(rng)?;
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())?;

        // Hash the input data to create fingerprint
        let mut hasher = Hasher::new();
        hasher.update(data);
        let mut fingerprint_data = vec![0u8; 64];
        hasher.finalize_xof().fill(&mut fingerprint_data);

        // Sign the fingerprint data
        let signature = keypair.sign(&fingerprint_data, rng)?;

        Ok((
            Self {
                data: fingerprint_data,
                signature,
            },
            public_key,
        ))
    }

    /// Verify a fingerprint using the provided public key
    pub fn verify(&self, public_key: &MlDsaPublicKey) -> Result<(), FingerprintError> {
        public_key.verify(&self.data, &self.signature)?;
        Ok(())
    }

    /// Get a reference to the fingerprint data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get a reference to the signature
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

impl ConstantTimeEq for Fingerprint {
    fn ct_eq(&self, other: &Self) -> Choice {
        let data_eq = self.data.ct_eq(&other.data);
        let sig_eq = self.signature.ct_eq(&other.signature);
        data_eq & sig_eq
    }
}

impl PartialEq for Fingerprint {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl Eq for Fingerprint {}
