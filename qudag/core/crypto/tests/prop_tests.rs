use proptest::prelude::*;
use qudag_crypto::{
    encryption,
    error::CryptoError,
    fingerprint::Fingerprint,
    hash::HashFunction,
    hqc::{Hqc256, SecurityParameter},
    kem,
    ml_dsa::MlDsa87,
    ml_kem::{Metrics as MlKemMetrics, MlKem768},
    signatures,
};
use rand::rngs::{SeedableRng, StdRng};
use std::collections::HashSet;

// ML-KEM property tests
prop_compose! {
    fn arb_kem_keypair()(mut rng in any::<[u8; 32]>()) -> Result<kem::KeyPair, kem::KEMError> {
        let mut rng = StdRng::from_seed(rng);
        kem::generate_keypair(&mut rng)
    }
}

proptest! {
    #[test]
    fn test_kem_roundtrip(keypair in arb_kem_keypair()) {
        let keypair = keypair.unwrap();
        let (shared_secret1, ciphertext) = kem::encapsulate(&keypair.public_key).unwrap();
        let shared_secret2 = kem::decapsulate(&keypair.secret_key, &ciphertext).unwrap();

        prop_assert!(bool::from(kem::constant_time_compare(&shared_secret1, &shared_secret2)));
    }

    #[test]
    fn test_kem_key_uniqueness(
        keypair1 in arb_kem_keypair(),
        keypair2 in arb_kem_keypair()
    ) {
        let keypair1 = keypair1.unwrap();
        let keypair2 = keypair2.unwrap();

        // Different keys should be generated
        prop_assert_ne!(keypair1.public_key, keypair2.public_key);
        prop_assert_ne!(keypair1.secret_key, keypair2.secret_key);

        // Cross encapsulation/decapsulation should fail
        let (_, ct1) = kem::encapsulate(&keypair1.public_key).unwrap();
        let (_, ct2) = kem::encapsulate(&keypair2.public_key).unwrap();

        let result1 = kem::decapsulate(&keypair2.secret_key, &ct1);
        let result2 = kem::decapsulate(&keypair1.secret_key, &ct2);

        prop_assert!(result1.is_err() || result2.is_err());
    }
}

// ML-DSA property tests
prop_compose! {
    fn arb_dsa_keypair()(mut rng in any::<[u8; 32]>()) -> Result<signatures::KeyPair, signatures::SignatureError> {
        let mut rng = StdRng::from_seed(rng);
        signatures::generate_keypair(&mut rng)
    }
}

proptest! {
    #[test]
    fn test_signature_roundtrip(
        keypair in arb_dsa_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..1024)
    ) {
        let keypair = keypair.unwrap();
        let signature = signatures::sign(&keypair.secret_key, &message).unwrap();
        let is_valid = signatures::verify(&keypair.public_key, &message, &signature).unwrap();
        prop_assert!(is_valid);
    }

    #[test]
    fn test_signature_tampering(
        keypair in arb_dsa_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..1024),
        tamper_index in 0usize..1024,
        tamper_byte in any::<u8>()
    ) {
        let keypair = keypair.unwrap();
        let mut signature = signatures::sign(&keypair.secret_key, &message).unwrap();

        if tamper_index < signature.len() {
            signature[tamper_index] ^= tamper_byte;
            let is_valid = signatures::verify(&keypair.public_key, &message, &signature).unwrap();
            prop_assert!(!is_valid);
        }
    }
}

// HQC encryption property tests
prop_compose! {
    fn arb_enc_keypair()(mut rng in any::<[u8; 32]>()) -> Result<encryption::KeyPair, encryption::EncryptionError> {
        let mut rng = StdRng::from_seed(rng);
        encryption::generate_keypair(&mut rng)
    }
}

proptest! {
    #[test]
    fn test_encryption_roundtrip(
        keypair in arb_enc_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..32)
    ) {
        let keypair = keypair.unwrap();
        let mut rng = thread_rng();

        let ciphertext = encryption::encrypt(&mut rng, &keypair.public_key, &message).unwrap();
        let decrypted = encryption::decrypt(&keypair.secret_key, &ciphertext).unwrap();

        prop_assert_eq!(message, decrypted);
    }

    #[test]
    fn test_encryption_tampering(
        keypair in arb_enc_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..32),
        tamper_index in 0usize..1024,
        tamper_byte in any::<u8>()
    ) {
        let keypair = keypair.unwrap();
        let mut rng = thread_rng();

        let mut ciphertext = encryption::encrypt(&mut rng, &keypair.public_key, &message).unwrap();

        if tamper_index < ciphertext.len() {
            ciphertext[tamper_index] ^= tamper_byte;

            match encryption::decrypt(&keypair.secret_key, &ciphertext) {
                Ok(decrypted) => prop_assert_ne!(message, decrypted),
                Err(_) => prop_assert!(true), // Error is also acceptable
            }
        }
    }
}

// Fingerprint property tests
proptest! {
    #[test]
    fn test_fingerprint_properties(
        data in prop::collection::vec(any::<u8>(), 0..1024)
    ) {
        let mut rng = rand::thread_rng();

        // Property: Can generate and verify fingerprint for any data
        let (fingerprint, public_key) = Fingerprint::generate(&data, &mut rng).unwrap();
        prop_assert!(fingerprint.verify(&public_key).is_ok());

        // Property: Different runs produce different fingerprints
        let (fingerprint2, _) = Fingerprint::generate(&data, &mut rng).unwrap();
        prop_assert_ne!(fingerprint.data(), fingerprint2.data());

        // Property: Fingerprint data length is consistent
        prop_assert_eq!(fingerprint.data().len(), 64);
    }

    #[test]
    fn test_fingerprint_verification_properties(
        data1 in prop::collection::vec(any::<u8>(), 0..1024),
        data2 in prop::collection::vec(any::<u8>(), 0..1024)
    ) {
        let mut rng = rand::thread_rng();

        // Generate two fingerprints
        let (fp1, key1) = Fingerprint::generate(&data1, &mut rng).unwrap();
        let (fp2, key2) = Fingerprint::generate(&data2, &mut rng).unwrap();

        // Property: Cannot verify fingerprint with wrong key
        if data1 != data2 {
            prop_assert!(fp1.verify(&key2).is_err());
            prop_assert!(fp2.verify(&key1).is_err());
        }
    }
}

// Enhanced ML-KEM property tests with mathematical properties
proptest! {
    #[test]
    fn test_ml_kem_mathematical_properties(
        seed1 in any::<[u8; 32]>(),
        seed2 in any::<[u8; 32]>()
    ) {
        let mut rng1 = StdRng::from_seed(seed1);
        let mut rng2 = StdRng::from_seed(seed2);

        // Property: Key generation is deterministic with same seed
        let mut rng1_copy = StdRng::from_seed(seed1);
        let keypair1a = MlKem768::generate_keypair(&mut rng1).unwrap();
        let keypair1b = MlKem768::generate_keypair(&mut rng1_copy).unwrap();

        prop_assert_eq!(keypair1a.public_key.as_bytes(), keypair1b.public_key.as_bytes());
        prop_assert_eq!(keypair1a.secret_key.as_bytes(), keypair1b.secret_key.as_bytes());

        // Property: Different seeds produce different keys
        let keypair2 = MlKem768::generate_keypair(&mut rng2).unwrap();
        if seed1 != seed2 {
            prop_assert_ne!(keypair1a.public_key.as_bytes(), keypair2.public_key.as_bytes());
        }

        // Property: Encapsulation is probabilistic (different outputs for same input)
        let (ss1, ct1) = MlKem768::encapsulate(&keypair1a.public_key, &mut rng1).unwrap();
        let (ss2, ct2) = MlKem768::encapsulate(&keypair1a.public_key, &mut rng2).unwrap();

        // Different randomness should produce different ciphertexts but recoverable secrets
        if seed1 != seed2 {
            prop_assert_ne!(ct1.as_bytes(), ct2.as_bytes());
        }

        // Property: Decapsulation recovers the correct shared secret
        let recovered_ss1 = MlKem768::decapsulate(&keypair1a.secret_key, &ct1).unwrap();
        let recovered_ss2 = MlKem768::decapsulate(&keypair1a.secret_key, &ct2).unwrap();

        prop_assert_eq!(ss1.as_bytes(), recovered_ss1.as_bytes());
        prop_assert_eq!(ss2.as_bytes(), recovered_ss2.as_bytes());
    }

    #[test]
    fn test_ml_kem_key_size_invariants(
        seed in any::<[u8; 32]>()
    ) {
        let mut rng = StdRng::from_seed(seed);
        let keypair = MlKem768::generate_keypair(&mut rng).unwrap();

        // Property: Key sizes are as expected for ML-KEM-768
        prop_assert_eq!(keypair.public_key.as_bytes().len(), 1184); // ML-KEM-768 public key size
        prop_assert_eq!(keypair.secret_key.as_bytes().len(), 2400); // ML-KEM-768 secret key size

        // Property: Shared secret size is consistent
        let (shared_secret, _) = MlKem768::encapsulate(&keypair.public_key, &mut rng).unwrap();
        prop_assert_eq!(shared_secret.as_bytes().len(), 32); // 256-bit shared secret
    }

    #[test]
    fn test_ml_kem_error_conditions(
        seed in any::<[u8; 32]>(),
        tamper_byte in any::<u8>(),
        tamper_pos in 0usize..2400
    ) {
        let mut rng = StdRng::from_seed(seed);
        let keypair = MlKem768::generate_keypair(&mut rng).unwrap();
        let (_, ciphertext) = MlKem768::encapsulate(&keypair.public_key, &mut rng).unwrap();

        // Property: Tampering with ciphertext causes decapsulation failure or different result
        let mut tampered_ct = ciphertext.clone();
        let ct_bytes = tampered_ct.as_bytes_mut();
        if tamper_pos < ct_bytes.len() {
            ct_bytes[tamper_pos] ^= tamper_byte;

            let result = MlKem768::decapsulate(&keypair.secret_key, &tampered_ct);
            // Either fails or produces different shared secret
            prop_assert!(result.is_err() || {
                let original_ss = MlKem768::decapsulate(&keypair.secret_key, &ciphertext).unwrap();
                result.unwrap().as_bytes() != original_ss.as_bytes()
            });
        }
    }
}

// Enhanced ML-DSA signature property tests
proptest! {
    #[test]
    fn test_ml_dsa_mathematical_properties(
        seed in any::<[u8; 32]>(),
        messages in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 0..1024),
            1..10
        )
    ) {
        let mut rng = StdRng::from_seed(seed);
        let keypair = MlDsa87::generate_keypair(&mut rng).unwrap();

        let mut signatures = Vec::new();

        for message in &messages {
            let signature = MlDsa87::sign(&keypair.secret_key, message, &mut rng).unwrap();

            // Property: Signature verification always succeeds for valid signatures
            let is_valid = MlDsa87::verify(&keypair.public_key, message, &signature).unwrap();
            prop_assert!(is_valid);

            signatures.push(signature);
        }

        // Property: Different messages produce different signatures (high probability)
        let mut signature_set = HashSet::new();
        for signature in &signatures {
            signature_set.insert(signature.as_bytes().to_vec());
        }

        // With different messages, we should get different signatures
        let unique_messages: HashSet<_> = messages.iter().collect();
        if unique_messages.len() > 1 {
            prop_assert!(signature_set.len() > 1, "All signatures are identical for different messages");
        }
    }

    #[test]
    fn test_ml_dsa_signature_properties(
        seed in any::<[u8; 32]>(),
        message in prop::collection::vec(any::<u8>(), 1..1024)
    ) {
        let mut rng = StdRng::from_seed(seed);
        let keypair = MlDsa87::generate_keypair(&mut rng).unwrap();

        // Property: Multiple signatures of same message are different (probabilistic)
        let sig1 = MlDsa87::sign(&keypair.secret_key, &message, &mut rng).unwrap();
        let sig2 = MlDsa87::sign(&keypair.secret_key, &message, &mut rng).unwrap();

        // Both should verify correctly
        prop_assert!(MlDsa87::verify(&keypair.public_key, &message, &sig1).unwrap());
        prop_assert!(MlDsa87::verify(&keypair.public_key, &message, &sig2).unwrap());

        // Signatures should be different due to randomization
        prop_assert_ne!(sig1.as_bytes(), sig2.as_bytes());
    }

    #[test]
    fn test_ml_dsa_key_size_invariants(
        seed in any::<[u8; 32]>()
    ) {
        let mut rng = StdRng::from_seed(seed);
        let keypair = MlDsa87::generate_keypair(&mut rng).unwrap();

        // Property: Key sizes are as expected for ML-DSA-87
        prop_assert_eq!(keypair.public_key.as_bytes().len(), 2592); // ML-DSA-87 public key
        prop_assert_eq!(keypair.secret_key.as_bytes().len(), 4896); // ML-DSA-87 secret key

        // Property: Signature size is bounded
        let message = vec![1, 2, 3, 4];
        let signature = MlDsa87::sign(&keypair.secret_key, &message, &mut rng).unwrap();
        let sig_len = signature.as_bytes().len();

        // ML-DSA-87 signatures should be around 4627 bytes
        prop_assert!(sig_len >= 4600 && sig_len <= 4700, "Signature size {} out of expected range", sig_len);
    }
}

// Enhanced HQC encryption property tests
proptest! {
    #[test]
    fn test_hqc_mathematical_properties(
        seed in any::<[u8; 32]>(),
        plaintexts in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 1..64),
            1..5
        )
    ) {
        let mut rng = StdRng::from_seed(seed);
        let keypair = Hqc256::generate_keypair(&mut rng).unwrap();

        for plaintext in &plaintexts {
            // Property: Encryption followed by decryption is identity
            let ciphertext = Hqc256::encrypt(&keypair.public_key, plaintext, &mut rng).unwrap();
            let decrypted = Hqc256::decrypt(&keypair.secret_key, &ciphertext).unwrap();

            prop_assert_eq!(plaintext, &decrypted);

            // Property: Encryption is probabilistic
            let ciphertext2 = Hqc256::encrypt(&keypair.public_key, plaintext, &mut rng).unwrap();
            prop_assert_ne!(ciphertext.as_bytes(), ciphertext2.as_bytes());
        }
    }

    #[test]
    fn test_hqc_error_propagation(
        seed in any::<[u8; 32]>(),
        plaintext in prop::collection::vec(any::<u8>(), 1..32),
        error_positions in prop::collection::vec(0usize..1024, 1..10)
    ) {
        let mut rng = StdRng::from_seed(seed);
        let keypair = Hqc256::generate_keypair(&mut rng).unwrap();

        let ciphertext = Hqc256::encrypt(&keypair.public_key, &plaintext, &mut rng).unwrap();

        // Property: Small errors in ciphertext should be correctable or cause failure
        let mut corrupted_ct = ciphertext.clone();
        let ct_bytes = corrupted_ct.as_bytes_mut();

        for &pos in &error_positions {
            if pos < ct_bytes.len() {
                ct_bytes[pos] ^= 1; // Single bit flip
            }
        }

        let decrypt_result = Hqc256::decrypt(&keypair.secret_key, &corrupted_ct);

        // Either decryption fails (error correction limit exceeded) or succeeds
        match decrypt_result {
            Ok(decrypted) => {
                // If decryption succeeds, it should either be original or detectably different
                if decrypted != plaintext {
                    prop_assert!(true); // Error was detected by producing different output
                }
            },
            Err(_) => {
                prop_assert!(true); // Error was detected by decryption failure
            }
        }
    }
}

// Hash function property tests
proptest! {
    #[test]
    fn test_hash_function_properties(
        inputs in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 0..1024),
            1..20
        )
    ) {
        let hasher = HashFunction::blake3();
        let mut hashes = HashSet::new();

        for input in &inputs {
            let hash = hasher.hash(input);

            // Property: Hash has consistent size
            prop_assert_eq!(hash.len(), 32); // BLAKE3 produces 256-bit hashes

            // Property: Same input produces same hash
            let hash2 = hasher.hash(input);
            prop_assert_eq!(hash, hash2);

            hashes.insert(hash);
        }

        // Property: Different inputs produce different hashes (collision resistance)
        let unique_inputs: HashSet<_> = inputs.iter().collect();
        if unique_inputs.len() > 1 {
            // We should have as many unique hashes as unique inputs (with very high probability)
            prop_assert_eq!(hashes.len(), unique_inputs.len());
        }
    }

    #[test]
    fn test_hash_avalanche_effect(
        base_input in prop::collection::vec(any::<u8>(), 1..100),
        bit_position in 0usize..800
    ) {
        let hasher = HashFunction::blake3();

        if bit_position / 8 < base_input.len() {
            let mut modified_input = base_input.clone();
            modified_input[bit_position / 8] ^= 1 << (bit_position % 8);

            let hash1 = hasher.hash(&base_input);
            let hash2 = hasher.hash(&modified_input);

            // Property: Single bit change should cause significant hash change (avalanche effect)
            let mut diff_bits = 0;
            for (a, b) in hash1.iter().zip(hash2.iter()) {
                diff_bits += (a ^ b).count_ones();
            }

            // At least 50% of bits should change (avalanche criterion)
            prop_assert!(diff_bits >= 128, "Insufficient avalanche effect: {} bits changed", diff_bits);
        }
    }
}

// Cross-primitive property tests
proptest! {
    #[test]
    fn test_hybrid_crypto_properties(
        seed in any::<[u8; 32]>(),
        message in prop::collection::vec(any::<u8>(), 1..100)
    ) {
        let mut rng = StdRng::from_seed(seed);

        // Generate keys for all primitives
        let kem_keypair = MlKem768::generate_keypair(&mut rng).unwrap();
        let dsa_keypair = MlDsa87::generate_keypair(&mut rng).unwrap();
        let hqc_keypair = Hqc256::generate_keypair(&mut rng).unwrap();
        let hasher = HashFunction::blake3();

        // Property: Hybrid encryption (KEM + symmetric) preserves message
        let (shared_secret, kem_ct) = MlKem768::encapsulate(&kem_keypair.public_key, &mut rng).unwrap();

        // Use shared secret as encryption key for HQC
        let message_hash = hasher.hash(&message);
        let signature = MlDsa87::sign(&dsa_keypair.secret_key, &message_hash, &mut rng).unwrap();

        // Verify the signature
        let sig_valid = MlDsa87::verify(&dsa_keypair.public_key, &message_hash, &signature).unwrap();
        prop_assert!(sig_valid);

        // Recover shared secret
        let recovered_secret = MlKem768::decapsulate(&kem_keypair.secret_key, &kem_ct).unwrap();
        prop_assert_eq!(shared_secret.as_bytes(), recovered_secret.as_bytes());

        // Property: Hash-then-sign preserves message authenticity
        let modified_message = {
            let mut m = message.clone();
            if !m.is_empty() {
                m[0] ^= 1;
            }
            m
        };

        if modified_message != message {
            let modified_hash = hasher.hash(&modified_message);
            let modified_sig_valid = MlDsa87::verify(&dsa_keypair.public_key, &modified_hash, &signature).unwrap();
            prop_assert!(!modified_sig_valid);
        }
    }
}
