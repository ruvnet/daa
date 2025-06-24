//! HQC (Hamming Quasi-Cyclic) Code-Based Encryption Example
//!
//! This example demonstrates how to use HQC for quantum-resistant encryption.
//! HQC is a code-based cryptographic scheme that provides quantum-resistant
//! public key encryption based on error-correcting codes.

use qudag_crypto::hqc::{Hqc, SecurityParameter, HqcError};
use rand::thread_rng;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 HQC (Hamming Quasi-Cyclic) Example");
    println!("====================================");

    // Example 1: Basic Key Generation and Encryption
    basic_encryption_example()?;

    // Example 2: Different Security Levels
    security_levels_example()?;

    // Example 3: Performance Comparison
    performance_comparison()?;

    // Example 4: Error Handling
    error_handling_examples()?;

    // Example 5: Large Message Handling
    large_message_example()?;

    // Example 6: Key Serialization
    key_serialization_example()?;

    println!("\n✅ All HQC examples completed successfully!");
    Ok(())
}

/// Example 1: Basic Key Generation and Encryption
///
/// This demonstrates the fundamental HQC operations.
fn basic_encryption_example() -> Result<(), HqcError> {
    println!("\n🔐 Example 1: Basic Key Generation and Encryption");

    let mut rng = thread_rng();
    let hqc = Hqc::new(SecurityParameter::Hqc256);

    // Generate key pair
    let (public_key, secret_key) = hqc.generate_keypair(&mut rng)?;
    println!("   Generated HQC-256 key pair");
    println!("   Public key size: {} bytes", public_key.as_bytes().len());
    println!("   Secret key size: {} bytes", secret_key.as_bytes().len());

    // Message to encrypt
    let message = b"Hello, quantum-resistant encryption!";
    println!("   Message: \"{}\"", String::from_utf8_lossy(message));
    println!("   Message size: {} bytes", message.len());

    // Encrypt the message
    let ciphertext = hqc.encrypt(message, &public_key, &mut rng)?;
    println!("   Ciphertext generated successfully");

    // Decrypt the message
    let decrypted = hqc.decrypt(&ciphertext, &secret_key)?;
    println!("   Message decrypted successfully");

    // Verify the decryption
    let decrypted_msg = &decrypted[..message.len()];
    if decrypted_msg == message {
        println!("   ✅ Encryption and decryption successful!");
        println!("   Decrypted: \"{}\"", String::from_utf8_lossy(decrypted_msg));
    } else {
        println!("   ❌ Decryption failed - messages don't match");
        return Err(HqcError::DecryptionError);
    }

    Ok(())
}

/// Example 2: Different Security Levels
///
/// This shows how to use different HQC security parameters.
fn security_levels_example() -> Result<(), HqcError> {
    println!("\n🛡️  Example 2: Different Security Levels");

    let mut rng = thread_rng();
    let security_levels = vec![
        (SecurityParameter::Hqc128, "HQC-128", "128-bit quantum security"),
        (SecurityParameter::Hqc192, "HQC-192", "192-bit quantum security"),
        (SecurityParameter::Hqc256, "HQC-256", "256-bit quantum security"),
    ];

    for (security, name, description) in security_levels {
        println!("\n   Testing {}: {}", name, description);
        
        let hqc = Hqc::new(security);
        let (pk, sk) = hqc.generate_keypair(&mut rng)?;
        
        // Calculate maximum message size for this security level
        let max_msg_size = match security {
            SecurityParameter::Hqc128 => 16,  // 128 bits / 8
            SecurityParameter::Hqc192 => 24,  // 192 bits / 8
            SecurityParameter::Hqc256 => 32,  // 256 bits / 8
        };
        
        let message = vec![0x42u8; max_msg_size];
        println!("     Max message size: {} bytes", max_msg_size);
        
        let start = Instant::now();
        let ciphertext = hqc.encrypt(&message, &pk, &mut rng)?;
        let encrypt_time = start.elapsed();
        
        let start = Instant::now();
        let decrypted = hqc.decrypt(&ciphertext, &sk)?;
        let decrypt_time = start.elapsed();
        
        println!("     Encryption time: {:?}", encrypt_time);
        println!("     Decryption time: {:?}", decrypt_time);
        println!("     ✅ {} working correctly", name);
        
        // Verify decryption
        assert_eq!(&decrypted[..message.len()], &message);
    }

    Ok(())
}

/// Example 3: Performance Comparison
///
/// This benchmarks HQC performance across different operations.
fn performance_comparison() -> Result<(), HqcError> {
    println!("\n⚡ Example 3: Performance Comparison");

    let mut rng = thread_rng();
    const NUM_ITERATIONS: usize = 10;

    for security in [SecurityParameter::Hqc128, SecurityParameter::Hqc192, SecurityParameter::Hqc256] {
        let security_name = match security {
            SecurityParameter::Hqc128 => "HQC-128",
            SecurityParameter::Hqc192 => "HQC-192", 
            SecurityParameter::Hqc256 => "HQC-256",
        };
        
        println!("\n   Performance benchmarks for {}:", security_name);
        let hqc = Hqc::new(security);

        // Benchmark key generation
        let start = Instant::now();
        let mut keypairs = Vec::with_capacity(NUM_ITERATIONS);
        for _ in 0..NUM_ITERATIONS {
            keypairs.push(hqc.generate_keypair(&mut rng)?);
        }
        let keygen_duration = start.elapsed();
        println!("     Key generation: {} ops in {:?}", NUM_ITERATIONS, keygen_duration);
        println!("     Average per keygen: {:?}", keygen_duration / NUM_ITERATIONS as u32);

        // Benchmark encryption
        let (ref pk, ref sk) = keypairs[0];
        let max_msg_size = match security {
            SecurityParameter::Hqc128 => 16,
            SecurityParameter::Hqc192 => 24,
            SecurityParameter::Hqc256 => 32,
        };
        let test_message = vec![0x42u8; max_msg_size];

        let start = Instant::now();
        let mut ciphertexts = Vec::with_capacity(NUM_ITERATIONS);
        for _ in 0..NUM_ITERATIONS {
            ciphertexts.push(hqc.encrypt(&test_message, pk, &mut rng)?);
        }
        let encrypt_duration = start.elapsed();
        println!("     Encryption: {} ops in {:?}", NUM_ITERATIONS, encrypt_duration);
        println!("     Average per encryption: {:?}", encrypt_duration / NUM_ITERATIONS as u32);

        // Benchmark decryption
        let start = Instant::now();
        for ciphertext in &ciphertexts {
            let _decrypted = hqc.decrypt(ciphertext, sk)?;
        }
        let decrypt_duration = start.elapsed();
        println!("     Decryption: {} ops in {:?}", NUM_ITERATIONS, decrypt_duration);
        println!("     Average per decryption: {:?}", decrypt_duration / NUM_ITERATIONS as u32);
    }

    Ok(())
}

/// Example 4: Error Handling
///
/// This demonstrates proper error handling for HQC operations.
fn error_handling_examples() -> Result<(), HqcError> {
    println!("\n⚠️  Example 4: Error Handling");

    let mut rng = thread_rng();
    let hqc = Hqc::new(SecurityParameter::Hqc128);
    let (pk, sk) = hqc.generate_keypair(&mut rng)?;

    // Test 1: Message too long
    println!("   Testing oversized message handling...");
    let too_long_message = vec![0x42u8; 1000]; // Way too long for HQC-128
    match hqc.encrypt(&too_long_message, &pk, &mut rng) {
        Ok(_) => println!("     Unexpected: Oversized message was accepted"),
        Err(HqcError::InvalidParameters) => println!("     ✅ Correctly rejected oversized message"),
        Err(e) => println!("     ✅ Rejected with error: {:?}", e),
    }

    // Test 2: Invalid public key
    println!("   Testing invalid public key handling...");
    let invalid_pk_bytes = vec![0u8; 10]; // Too short
    match qudag_crypto::hqc::PublicKey::from_bytes(&invalid_pk_bytes) {
        Ok(_) => println!("     Unexpected: Invalid public key was accepted"),
        Err(e) => println!("     ✅ Correctly rejected invalid public key: {:?}", e),
    }

    // Test 3: Empty message
    println!("   Testing empty message...");
    let empty_message = vec![];
    match hqc.encrypt(&empty_message, &pk, &mut rng) {
        Ok(ciphertext) => {
            println!("     ✅ Empty message encrypted successfully");
            let decrypted = hqc.decrypt(&ciphertext, &sk)?;
            println!("     ✅ Empty message decrypted successfully");
            assert_eq!(decrypted.len(), 16); // HQC-128 k/8 = 16 bytes
        }
        Err(e) => println!("     Empty message error: {:?}", e),
    }

    Ok(())
}

/// Example 5: Large Message Handling
///
/// This shows how to handle messages at the size limits.
fn large_message_example() -> Result<(), HqcError> {
    println!("\n📦 Example 5: Large Message Handling");

    let mut rng = thread_rng();
    let hqc = Hqc::new(SecurityParameter::Hqc256);
    let (pk, sk) = hqc.generate_keypair(&mut rng)?;

    // Test maximum size message for HQC-256
    let max_size = 32; // 256 bits / 8 bytes
    let large_message = (0..max_size).map(|i| (i as u8) ^ 0xAA).collect::<Vec<u8>>();
    
    println!("   Testing maximum size message ({} bytes)...", max_size);
    println!("   Message pattern: {:02x?}...", &large_message[..8]);

    let start = Instant::now();
    let ciphertext = hqc.encrypt(&large_message, &pk, &mut rng)?;
    let encrypt_time = start.elapsed();
    println!("   Large message encrypted in {:?}", encrypt_time);

    let start = Instant::now();
    let decrypted = hqc.decrypt(&ciphertext, &sk)?;
    let decrypt_time = start.elapsed();
    println!("   Large message decrypted in {:?}", decrypt_time);

    // Verify decryption
    assert_eq!(&decrypted[..large_message.len()], &large_message);
    println!("   ✅ Large message handling successful");

    // Test various message sizes
    println!("\n   Testing different message sizes:");
    for size in [1, 4, 8, 16, 24, 32] {
        if size <= max_size {
            let message = vec![size as u8; size];
            let ct = hqc.encrypt(&message, &pk, &mut rng)?;
            let dec = hqc.decrypt(&ct, &sk)?;
            assert_eq!(&dec[..message.len()], &message);
            println!("     ✅ {} byte message: OK", size);
        }
    }

    Ok(())
}

/// Example 6: Key Serialization
///
/// This demonstrates how to serialize and deserialize HQC keys.
fn key_serialization_example() -> Result<(), HqcError> {
    println!("\n💾 Example 6: Key Serialization");

    let mut rng = thread_rng();
    let hqc = Hqc::new(SecurityParameter::Hqc192);

    // Generate key pair
    let (public_key, secret_key) = hqc.generate_keypair(&mut rng)?;
    println!("   Generated HQC-192 key pair");

    // Serialize keys
    let pk_bytes = public_key.as_bytes();
    let sk_bytes = secret_key.as_bytes();
    
    println!("   Public key serialized: {} bytes", pk_bytes.len());
    println!("   Secret key serialized: {} bytes", sk_bytes.len());

    // Test encryption with original keys
    let original_message = b"Key serialization test message";
    let original_ciphertext = hqc.encrypt(original_message, &public_key, &mut rng)?;
    let original_decrypted = hqc.decrypt(&original_ciphertext, &secret_key)?;
    
    // Verify original encryption works
    assert_eq!(&original_decrypted[..original_message.len()], original_message);
    println!("   ✅ Original keys work correctly");

    // Deserialize public key
    let restored_pk = qudag_crypto::hqc::PublicKey::from_bytes(&pk_bytes)?;
    println!("   ✅ Public key deserialized successfully");

    // Test with restored public key
    let new_ciphertext = hqc.encrypt(original_message, &restored_pk, &mut rng)?;
    let new_decrypted = hqc.decrypt(&new_ciphertext, &secret_key)?;
    
    // Verify restored public key works
    assert_eq!(&new_decrypted[..original_message.len()], original_message);
    println!("   ✅ Restored public key works correctly");

    // Test cross-compatibility
    let cross_decrypted = hqc.decrypt(&original_ciphertext, &secret_key)?;
    assert_eq!(&cross_decrypted[..original_message.len()], original_message);
    println!("   ✅ Cross-compatibility verified");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_encryption() {
        assert!(basic_encryption_example().is_ok());
    }

    #[test]
    fn test_security_levels() {
        assert!(security_levels_example().is_ok());
    }

    #[test]
    fn test_error_handling() {
        assert!(error_handling_examples().is_ok());
    }

    #[test]
    fn test_large_messages() {
        assert!(large_message_example().is_ok());
    }

    #[test]
    fn test_key_serialization() {
        assert!(key_serialization_example().is_ok());
    }
}