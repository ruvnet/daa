// Sample implementation for WASM-compatible crypto abstraction layer

// In core/crypto/Cargo.toml:
/*
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
pqcrypto-dilithium = "0.5"
pqcrypto-kyber = "0.5"
pqcrypto-hqc = "0.2"

[target.'cfg(target_arch = "wasm32")'.dependencies]
ml-kem = "0.2.0"
fips204 = "0.3.0"
# Alternative: pqc_kyber = { version = "0.8.1", features = ["wasm"] }
# Alternative: ml-dsa = "0.2.0"
*/

// core/crypto/src/quantum/mod.rs
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

// Shared trait definitions
pub trait MlKemOps {
    fn keygen() -> Result<(Vec<u8>, Vec<u8>), CryptoError>;
    fn encapsulate(pk: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError>;
    fn decapsulate(sk: &[u8], ct: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

pub trait MlDsaOps {
    fn keygen() -> Result<(Vec<u8>, Vec<u8>), CryptoError>;
    fn sign(sk: &[u8], msg: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn verify(pk: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, CryptoError>;
}

// core/crypto/src/quantum/wasm/ml_kem.rs
#[cfg(target_arch = "wasm32")]
mod ml_kem_wasm {
    use ml_kem::{MlKem768, Encapsulate, Decapsulate, KemCore};
    use super::*;

    pub struct MlKemWrapper;

    impl MlKemOps for MlKemWrapper {
        fn keygen() -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
            let (dk, ek) = MlKem768::generate(&mut rand::thread_rng());
            Ok((ek.as_bytes().to_vec(), dk.as_bytes().to_vec()))
        }

        fn encapsulate(pk: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
            let ek = <MlKem768 as KemCore>::EncapsulationKey::from_bytes(pk)
                .map_err(|_| CryptoError::InvalidKey)?;
            let (ct, ss) = ek.encapsulate(&mut rand::thread_rng())
                .map_err(|_| CryptoError::EncapsulationError)?;
            Ok((ct.as_bytes().to_vec(), ss.as_bytes().to_vec()))
        }

        fn decapsulate(sk: &[u8], ct: &[u8]) -> Result<Vec<u8>, CryptoError> {
            let dk = <MlKem768 as KemCore>::DecapsulationKey::from_bytes(sk)
                .map_err(|_| CryptoError::InvalidKey)?;
            let ct_obj = <MlKem768 as KemCore>::Ciphertext::from_bytes(ct)
                .map_err(|_| CryptoError::InvalidCiphertext)?;
            let ss = dk.decapsulate(&ct_obj)
                .map_err(|_| CryptoError::DecapsulationError)?;
            Ok(ss.as_bytes().to_vec())
        }
    }
}

// core/crypto/src/quantum/wasm/ml_dsa.rs
#[cfg(target_arch = "wasm32")]
mod ml_dsa_wasm {
    use fips204::{ml_dsa_65, traits::{Signer, Verifier, SerDes}};
    use super::*;

    pub struct MlDsaWrapper;

    impl MlDsaOps for MlDsaWrapper {
        fn keygen() -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
            let (pk, sk) = ml_dsa_65::try_keygen()
                .map_err(|_| CryptoError::KeyGenerationError)?;
            Ok((pk.into_bytes(), sk.into_bytes()))
        }

        fn sign(sk: &[u8], msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
            let sk_obj = ml_dsa_65::PrivateKey::try_from_bytes(sk)
                .map_err(|_| CryptoError::InvalidKey)?;
            let sig = sk_obj.try_sign(msg, &[])
                .map_err(|_| CryptoError::SignError)?;
            Ok(sig.into_bytes())
        }

        fn verify(pk: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, CryptoError> {
            let pk_obj = ml_dsa_65::PublicKey::try_from_bytes(pk)
                .map_err(|_| CryptoError::InvalidKey)?;
            let sig_obj = ml_dsa_65::Signature::try_from_bytes(sig)
                .map_err(|_| CryptoError::InvalidSignature)?;
            Ok(pk_obj.verify(msg, &sig_obj, &[]))
        }
    }
}

// core/crypto/src/quantum/wasm/hqc.rs
#[cfg(target_arch = "wasm32")]
mod hqc_wasm {
    use super::*;

    pub struct HqcStub;

    impl HqcStub {
        pub fn new() -> Result<Self, CryptoError> {
            Err(CryptoError::UnsupportedInWasm("HQC is not available in WASM builds"))
        }

        pub fn encrypt(&self, _pk: &[u8], _msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
            Err(CryptoError::UnsupportedInWasm("HQC encryption not available in WASM"))
        }

        pub fn decrypt(&self, _sk: &[u8], _ct: &[u8]) -> Result<Vec<u8>, CryptoError> {
            Err(CryptoError::UnsupportedInWasm("HQC decryption not available in WASM"))
        }
    }
}

// Updated error type
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid key")]
    InvalidKey,
    #[error("Invalid ciphertext")]
    InvalidCiphertext,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Key generation failed")]
    KeyGenerationError,
    #[error("Encapsulation failed")]
    EncapsulationError,
    #[error("Decapsulation failed")]
    DecapsulationError,
    #[error("Signing failed")]
    SignError,
    #[error("Verification failed")]
    VerifyError,
    #[error("Unsupported in WASM: {0}")]
    UnsupportedInWasm(&'static str),
}

// Usage example that works on both native and WASM:
pub fn example_usage() -> Result<(), CryptoError> {
    // ML-KEM (works on both platforms)
    let (pk, sk) = MlKemWrapper::keygen()?;
    let (ct, ss1) = MlKemWrapper::encapsulate(&pk)?;
    let ss2 = MlKemWrapper::decapsulate(&sk, &ct)?;
    assert_eq!(ss1, ss2);

    // ML-DSA (works on both platforms)
    let (pk, sk) = MlDsaWrapper::keygen()?;
    let msg = b"Hello, quantum-resistant world!";
    let sig = MlDsaWrapper::sign(&sk, msg)?;
    let valid = MlDsaWrapper::verify(&pk, msg, &sig)?;
    assert!(valid);

    // HQC (only works on native, returns error on WASM)
    #[cfg(target_arch = "wasm32")]
    {
        match HqcStub::new() {
            Err(CryptoError::UnsupportedInWasm(_)) => {
                println!("HQC not available in WASM as expected");
            }
            _ => panic!("Expected UnsupportedInWasm error"),
        }
    }

    Ok(())
}