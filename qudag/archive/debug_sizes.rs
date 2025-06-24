use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};

fn main() {
    println!("Debugging Dilithium3 sizes...");
    
    // Generate keypair
    let (pk, sk) = keypair();
    let message = b"test message";
    
    println!("Key sizes:");
    println!("  Public key: {} bytes", pk.as_bytes().len());
    println!("  Secret key: {} bytes", sk.as_bytes().len());
    
    // Sign message
    let signed_msg = sign(message, &sk);
    let signed_bytes = signed_msg.as_bytes();
    
    println!("Signed message:");
    println!("  Total size: {} bytes", signed_bytes.len());
    println!("  Message size: {} bytes", message.len());
    println!("  Estimated signature size: {} bytes", signed_bytes.len() - message.len());
    
    // Try to verify
    match open(&signed_msg, &pk) {
        Ok(recovered) => {
            println!("Verification successful, recovered {} bytes", recovered.len());
            if recovered == message {
                println!("✅ Message matches!");
            } else {
                println!("❌ Message mismatch");
            }
        }
        Err(e) => {
            println!("❌ Verification failed: {:?}", e);
        }
    }
}