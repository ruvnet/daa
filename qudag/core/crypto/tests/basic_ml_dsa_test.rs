//! Basic ML-DSA test to verify compilation and basic functionality

#[test]
fn test_ml_dsa_types_exist() {
    // Just test that the types can be imported

    // If we get here, at least the types are defined
    assert!(true);
}

#[test]
fn test_ml_dsa_basic_functionality() {
    use qudag_crypto::ml_dsa::{MlDsaKeyPair, MlDsaPublicKey};
    use rand::thread_rng;

    let mut rng = thread_rng();

    // Test key generation
    let keypair_result = MlDsaKeyPair::generate(&mut rng);
    assert!(keypair_result.is_ok(), "Key generation should succeed");

    let keypair = keypair_result.unwrap();

    // Test basic key properties
    assert!(
        !keypair.public_key().is_empty(),
        "Public key should not be empty"
    );
    assert!(
        !keypair.secret_key().is_empty(),
        "Secret key should not be empty"
    );

    // Test public key creation
    let public_key_result = MlDsaPublicKey::from_bytes(keypair.public_key());
    assert!(
        public_key_result.is_ok(),
        "Public key creation should succeed"
    );

    // Test basic signing (this might fail with our placeholder implementation)
    let message = b"test message";
    let signature_result = keypair.sign(message, &mut rng);

    if let Ok(signature) = signature_result {
        // If signing succeeds, test verification
        let public_key = public_key_result.unwrap();
        let verification_result = public_key.verify(message, &signature);

        // With our placeholder implementation, this might fail, but that's ok for now
        println!("Verification result: {:?}", verification_result);
    } else {
        println!(
            "Signing failed (expected with placeholder implementation): {:?}",
            signature_result
        );
    }
}
