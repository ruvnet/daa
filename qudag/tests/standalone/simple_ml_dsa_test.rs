use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};

#[test]
fn test_simple_ml_dsa() {
    println!("Testing ML-DSA (Dilithium3) implementation...");
    
    // Generate keypair
    let (pk, sk) = keypair();
    println!("Generated keypair:");
    println!("  Public key size: {} bytes", pk.as_bytes().len());
    println!("  Secret key size: {} bytes", sk.as_bytes().len());
    
    // Sign a message
    let message = b"Hello, quantum-resistant world!";
    let signed_msg = sign(message, &sk);
    println!("Signed message size: {} bytes", signed_msg.as_bytes().len());
    
    // Verify signature
    match open(&signed_msg, &pk) {
        Ok(verified_msg) => {
            assert_eq!(verified_msg, message);
            println!("✅ Signature verification PASSED!");
        }
        Err(_) => {
            panic!("❌ Signature verification FAILED!");
        }
    }
    
    // Extract signature size
    let signature_size = signed_msg.as_bytes().len() - message.len();
    println!("Estimated signature size: {} bytes", signature_size);
    
    // Check expected sizes for ML-DSA-65 (dilithium3)
    assert_eq!(pk.as_bytes().len(), 1952, "Public key size should be 1952 bytes");
    assert_eq!(sk.as_bytes().len(), 4032, "Secret key size should be 4032 bytes");
}