use proptest::prelude::*;
use qudag_crypto::hqc::{Hqc256, PublicKey};
use rand::RngCore;

#[test]
fn test_hqc_key_generation() {
    let (pk, sk) = Hqc256::keygen().expect("Key generation should succeed");
    assert!(!pk.as_bytes().is_empty());
    assert!(!sk.as_bytes().is_empty());
}

#[test]
fn test_hqc_encryption_decryption() {
    let message = b"Test message for HQC encryption";
    let (pk, sk) = Hqc256::keygen().expect("Key generation should succeed");

    let ciphertext = Hqc256::encrypt(&pk, message).expect("Encryption should succeed");
    let decrypted = Hqc256::decrypt(&sk, &ciphertext).expect("Decryption should succeed");

    // Check that message was properly encoded/decoded (may have padding)
    assert!(decrypted.len() >= message.len());
    assert_eq!(&decrypted[..message.len()], message);
}

#[test]
fn test_hqc_invalid_ciphertext() {
    let (_, sk) = Hqc256::keygen().expect("Key generation should succeed");
    let mut invalid_ciphertext = vec![0u8; Hqc256::CIPHERTEXT_SIZE];
    rand::thread_rng().fill_bytes(&mut invalid_ciphertext);

    let result = Hqc256::decrypt(&sk, &invalid_ciphertext);
    // Invalid ciphertext should either fail or produce different plaintext
    // In our implementation, it will likely succeed but produce incorrect data
    if let Ok(decrypted) = result {
        // Should be very unlikely to match our known message patterns
        let test_message = b"Test message for HQC encryption";
        assert_ne!(
            &decrypted[..test_message.len().min(decrypted.len())],
            test_message
        );
    }
}

#[test]
fn test_hqc_long_message() {
    let message = vec![0u8; 1024];
    let (pk, sk) = Hqc256::keygen().expect("Key generation should succeed");

    let ciphertext = Hqc256::encrypt(&pk, &message).expect("Encryption should succeed");
    let decrypted = Hqc256::decrypt(&sk, &ciphertext).expect("Decryption should succeed");

    // Check that message was properly encoded/decoded (may have padding)
    assert!(decrypted.len() >= message.len());
    assert_eq!(&decrypted[..message.len()], message);
}

proptest! {
    #[test]
    fn test_hqc_random_keys_and_messages(
        message in prop::collection::vec(any::<u8>(), 1..1000),
        pk_bytes in prop::collection::vec(0u8..255, Hqc256::PUBLIC_KEY_SIZE),
        ct_bytes in prop::collection::vec(0u8..255, Hqc256::CIPHERTEXT_SIZE)
    ) {
        // Ensure we can handle random/malformed inputs without panicking
        if let Ok(pk) = PublicKey::from_bytes(&pk_bytes) {
            // Attempt encryption with random public key - should not panic
            let _ = Hqc256::encrypt(&pk, &message);
        }
    }
}
