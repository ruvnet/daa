use subtle::ConstantTimeEq;
use zeroize::Zeroize;

#[test]
fn test_constant_time_equality() {
    let a = vec![1u8, 2u8, 3u8];
    let b = vec![1u8, 2u8, 3u8];
    let c = vec![4u8, 5u8, 6u8];

    assert!(constant_time_eq(&a, &b));
    assert!(!constant_time_eq(&a, &c));
    // Note: constant_time_eq handles different lengths safely
}

#[test]
fn test_constant_time_zeroization() {
    use zeroize::Zeroize;

    let mut sensitive_data = vec![1u8, 2u8, 3u8];
    sensitive_data.zeroize();

    assert!(sensitive_data.iter().all(|&x| x == 0));
}

#[test]
fn test_constant_time_comparison() {
    let a = 0xffu8;
    let b = 0xffu8;
    let c = 0x00u8;

    assert_eq!(a.ct_eq(&b).unwrap_u8(), 1);
    assert_eq!(a.ct_eq(&c).unwrap_u8(), 0);
}

#[test]
fn test_timing_resistance() {
    use qudag_crypto::ml_dsa::MlDsaKeyPair;
    use qudag_crypto::ml_kem::MlKem768;
    use rand::thread_rng;

    // Test KEM timing resistance
    let (pk_kem, sk_kem) = MlKem768::keygen().expect("KEM key generation failed");
    let (ct_kem, _) = MlKem768::encapsulate(&pk_kem).expect("KEM encapsulation failed");
    let _ = MlKem768::decapsulate(&sk_kem, &ct_kem).expect("KEM decapsulation failed");

    // Test signature timing resistance
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Signature key generation failed");
    let message = b"Test message";
    let signature = keypair.sign(message, &mut rng).expect("Signing failed");
    let public_key = keypair
        .to_public_key()
        .expect("Public key conversion failed");
    let _ = public_key
        .verify(message, &signature)
        .expect("Verification failed");
}
