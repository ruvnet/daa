use proptest::prelude::*;
use qudag_crypto::ml_dsa::{MlDsaError, MlDsaKeyPair, MlDsaPublicKey};
use rand::{thread_rng, RngCore};

#[test]
fn test_mldsa_key_generation() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    assert!(!keypair.public_key().is_empty());
}

#[test]
fn test_mldsa_sign_verify() {
    let message = b"Test message for ML-DSA signature";
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");

    let signature = keypair
        .sign(message, &mut rng)
        .expect("Signing should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Valid public key");
    let verification = public_key.verify(message, &signature);
    assert!(verification.is_ok());
}

#[test]
fn test_mldsa_invalid_signature() {
    let message = b"Test message for ML-DSA signature";
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Valid public key");

    let mut invalid_signature = vec![0u8; 2372]; // ML-DSA-65 signature size
    thread_rng().fill_bytes(&mut invalid_signature);

    let verification = public_key.verify(message, &invalid_signature);
    assert!(matches!(verification, Err(MlDsaError::VerificationFailed)));
}

#[test]
fn test_mldsa_message_tampering() {
    let message = b"Original message";
    let tampered_message = b"Tampered message";
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Valid public key");

    let signature = keypair
        .sign(message, &mut rng)
        .expect("Signing should succeed");
    let verification = public_key.verify(tampered_message, &signature);
    assert!(matches!(verification, Err(MlDsaError::VerificationFailed)));
}

proptest! {
    #[test]
    fn test_mldsa_random_inputs(
        message in prop::collection::vec(any::<u8>(), 1..1000),
        pk_bytes in prop::collection::vec(0u8..255, 1952), // ML-DSA-65 public key size
        sig_bytes in prop::collection::vec(0u8..255, 2372) // ML-DSA-65 signature size
    ) {
        // Ensure we can handle random/malformed inputs without panicking
        if let Ok(pk) = MlDsaPublicKey::from_bytes(&pk_bytes) {
            // Attempt verification with random inputs - should not panic
            let _ = pk.verify(&message, &sig_bytes);
        }
    }
}
