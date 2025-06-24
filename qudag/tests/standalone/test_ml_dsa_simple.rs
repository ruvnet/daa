extern crate pqcrypto_dilithium;
extern crate pqcrypto_traits;

use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};

fn main() {
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
            if verified_msg == message {
                println!("✅ Signature verification PASSED!");
            } else {
                println!("❌ Message mismatch after verification");
            }
        }
        Err(_) => {
            println!("❌ Signature verification FAILED!");
        }
    }
    
    // Test with different message (should fail)
    let wrong_message = b"Wrong message";
    let wrong_signed = sign(wrong_message, &sk);
    match open(&wrong_signed, &pk) {
        Ok(verified_msg) => {
            if verified_msg == message {
                println!("❌ Wrong message was verified as correct!");
            } else {
                println!("✅ Different message correctly verified as different");
            }
        }
        Err(_) => {
            println!("❌ Verification failed for different message");
        }
    }
    
    // Extract signature size
    let signature_size = signed_msg.as_bytes().len() - message.len();
    println!("Estimated signature size: {} bytes", signature_size);
}