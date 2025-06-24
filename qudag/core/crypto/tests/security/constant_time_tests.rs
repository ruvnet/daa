use criterion::black_box;
use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};
use test_utils::timing::*;
use subtle::{Choice, ConstantTimeEq};

/// Test suite for constant-time operations validation
#[cfg(test)]
mod constant_time_tests {
    use super::*;

    const ITERATIONS: usize = 10000; // Increased for better statistical significance

    /// Helper function to verify constant-time behavior with different inputs
    fn verify_constant_time<F>(func: F, description: &str)
    where
        F: Fn() + Send + 'static
    {
        let time_variance = measure_time_variance(func, ITERATIONS);
        assert!(time_variance < TIMING_THRESHOLD,
            "{} showed timing variation above threshold: {}", description, time_variance);
    }

    #[test]
    fn test_mlkem_operations_constant_time() {
        // Test key generation
        verify_constant_time(
            || {
                let (pk, sk) = MLKem::keygen();
                black_box((pk, sk));
            },
            "ML-KEM key generation"
        );

        // Test encapsulation with different public keys
        let (pk1, _) = MLKem::keygen();
        let (pk2, _) = MLKem::keygen();
        verify_constant_time(
            || {
                let choice = Choice::from(1u8);
                let pk = Choice::from(0u8).select(&pk2, &pk1);
                let (ct, ss) = MLKem::encapsulate(&pk).unwrap();
                black_box((ct, ss));
            },
            "ML-KEM encapsulation"
        );

        // Test decapsulation with valid and invalid ciphertexts
        let (pk, sk) = MLKem::keygen();
        let (ct, _) = MLKem::encapsulate(&pk).unwrap();
        let mut invalid_ct = ct.clone();
        invalid_ct[0] ^= 1; // Flip one bit

        verify_constant_time(
            || {
                let _ = MLKem::decapsulate(&ct, &sk);
                let _ = MLKem::decapsulate(&invalid_ct, &sk);
                black_box(());
            },
            "ML-KEM decapsulation"
        );
    }

    #[test]
    fn test_mldsa_operations_constant_time() {
        let message1 = b"test message 1";
        let message2 = b"test message 2";
        let (_, sk) = MLDsa::keygen();

        // Test signing with different messages
        verify_constant_time(
            || {
                let msg = Choice::from(0u8).select(message2, message1);
                let signature = MLDsa::sign(msg, &sk);
                black_box(signature);
            },
            "ML-DSA signing"
        );

        // Test verification with valid and invalid signatures
        let (pk, _) = MLDsa::keygen();
        let signature = MLDsa::sign(message1, &sk);
        let mut invalid_sig = signature.clone();
        invalid_sig[0] ^= 1; // Flip one bit

        verify_constant_time(
            || {
                let _ = MLDsa::verify(message1, &signature, &pk);
                let _ = MLDsa::verify(message1, &invalid_sig, &pk);
                black_box(());
            },
            "ML-DSA verification"
        );
    }

    #[test]
    fn test_hqc_operations_constant_time() {
        let message1 = b"test message 1";
        let message2 = b"test message 2";
        let (pk, sk) = HQC::keygen();

        // Test encryption with different messages
        verify_constant_time(
            || {
                let msg = Choice::from(0u8).select(message2, message1);
                let ciphertext = HQC::encrypt(msg, &pk);
                black_box(ciphertext);
            },
            "HQC encryption"
        );

        // Test decryption with valid and invalid ciphertexts
        let ct = HQC::encrypt(message1, &pk).unwrap();
        let mut invalid_ct = ct.clone();
        invalid_ct[0] ^= 1; // Flip one bit

        verify_constant_time(
            || {
                let _ = HQC::decrypt(&ct, &sk);
                let _ = HQC::decrypt(&invalid_ct, &sk);
                black_box(());
            },
            "HQC decryption"
        );
    }
}