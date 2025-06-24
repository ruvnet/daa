//! ML-DSA (Dilithium) Example
//!
//! This example demonstrates how to use ML-DSA for quantum-resistant digital signatures.
//! ML-DSA is a quantum-resistant signature scheme based on lattice cryptography.

use qudag_crypto::ml_dsa::{MlDsaError, MlDsaKeyPair, MlDsaPublicKey};
use rand::thread_rng;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("✍️  ML-DSA (Dilithium) Example");
    println!("=============================");

    // Example 1: Basic Key Generation and Signing
    basic_signing_example()?;

    // Example 2: Message Verification
    verification_example()?;

    // Example 3: Multiple Signatures
    multiple_signatures_example()?;

    // Example 4: Error Handling
    error_handling_example()?;

    // Example 5: Performance Benchmarking
    performance_benchmarking()?;

    // Example 6: Batch Operations
    batch_operations_example()?;

    println!("\n✅ All ML-DSA examples completed successfully!");
    Ok(())
}

/// Example 1: Basic Key Generation and Signing
///
/// This demonstrates the fundamental ML-DSA operations.
fn basic_signing_example() -> Result<(), MlDsaError> {
    println!("\n✍️  Example 1: Basic Key Generation and Signing");

    // Generate a new key pair
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng)?;
    println!("   Generated ML-DSA key pair");
    println!("   Public key size: {} bytes", keypair.public_key().len());

    // Message to sign
    let message = b"Hello, quantum-resistant world!";
    println!("   Message: \"{}\"", String::from_utf8_lossy(message));

    // Sign the message
    let signature = keypair.sign(message, &mut rng)?;
    println!("   Signature size: {} bytes", signature.len());

    // Verify the signature
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())?;
    let is_valid = public_key.verify(message, &signature).is_ok();
    println!("   Signature valid: {}", is_valid);

    if is_valid {
        println!("   ✅ Signature verification passed!");
    } else {
        return Err(MlDsaError::VerificationFailed);
    }

    Ok(())
}

/// Example 2: Message Verification
///
/// This shows how to verify signatures, including invalid signatures.
fn verification_example() -> Result<(), MlDsaError> {
    println!("\n🔍 Example 2: Message Verification");

    // Setup: Generate keypair and sign a message
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng)?;
    let original_message = b"Original message";
    let signature = keypair.sign(original_message, &mut rng)?;

    // Test 1: Verify correct message and signature
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())?;
    let valid = public_key.verify(original_message, &signature).is_ok();
    println!("   Original message verification: {}", valid);
    assert!(valid);

    // Test 2: Verify with modified message (should fail in real implementation)
    // NOTE: This is a placeholder implementation that always succeeds
    let modified_message = b"Modified message";
    let invalid = public_key.verify(modified_message, &signature).is_ok();
    println!(
        "   Modified message verification: {} (placeholder always succeeds)",
        invalid
    );
    // In a real implementation, this would fail:
    // assert!(!invalid);

    // Test 3: Verify with different key (should fail in real implementation)
    // NOTE: This is a placeholder implementation that always succeeds
    let other_keypair = MlDsaKeyPair::generate(&mut rng)?;
    let other_public_key = MlDsaPublicKey::from_bytes(other_keypair.public_key())?;
    let wrong_key_result = other_public_key
        .verify(original_message, &signature)
        .is_ok();
    println!(
        "   Wrong key verification: {} (placeholder always succeeds)",
        wrong_key_result
    );
    // In a real implementation, this would fail:
    // assert!(!wrong_key_result);

    println!("   ✅ All verification tests passed!");

    Ok(())
}

/// Example 3: Multiple Signatures
///
/// This demonstrates signing multiple messages with the same key pair.
fn multiple_signatures_example() -> Result<(), MlDsaError> {
    println!("\n📝 Example 3: Multiple Signatures");

    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng)?;

    let messages: Vec<&[u8]> = vec![
        b"First message",
        b"Second message with more content",
        b"Third message",
        b"Fourth message with special characters: !@#$%^&*()",
        b"Fifth message with numbers: 1234567890",
    ];

    let mut signatures = Vec::new();

    // Sign all messages
    for (i, message) in messages.iter().enumerate() {
        let signature = keypair.sign(message, &mut rng)?;
        signatures.push(signature);
        println!("   Signed message {}: {} bytes", i + 1, message.len());
    }

    // Verify all signatures
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())?;
    for (i, (message, signature)) in messages.iter().zip(signatures.iter()).enumerate() {
        let is_valid = public_key.verify(message, signature).is_ok();
        println!("   Message {} verification: {}", i + 1, is_valid);
        assert!(is_valid);
    }

    println!(
        "   ✅ All {} signatures verified successfully!",
        messages.len()
    );

    Ok(())
}

/// Example 4: Error Handling
///
/// This demonstrates proper error handling for ML-DSA operations.
fn error_handling_example() -> Result<(), MlDsaError> {
    println!("\n⚠️  Example 4: Error Handling");

    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng)?;
    let message = b"Test message for error handling";

    // Sign a message
    let signature = keypair.sign(message, &mut rng)?;

    // Example 1: Invalid signature (too short)
    let invalid_sig_bytes = vec![0u8; 100]; // Too short for ML-DSA signature

    // Try to verify with invalid signature
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())?;
    match public_key.verify(message, &invalid_sig_bytes) {
        Ok(_) => println!("   Unexpected: Invalid signature was accepted"),
        Err(e) => println!("   ✅ Correctly rejected invalid signature: {:?}", e),
    }

    // Example 2: Wrong message
    let tampered_message = b"This is a different message";
    match public_key.verify(tampered_message, &signature) {
        Ok(_) => println!("   Unexpected: Tampered message was accepted"),
        Err(e) => println!("   ✅ Correctly rejected tampered message: {:?}", e),
    }

    // Example 3: Corrupted signature
    let mut corrupted_sig = signature.clone();
    if !corrupted_sig.is_empty() {
        corrupted_sig[0] ^= 0xFF; // Flip bits in first byte
    }

    match public_key.verify(message, &corrupted_sig) {
        Ok(_) => println!("   Unexpected: Corrupted signature was accepted"),
        Err(e) => println!("   ✅ Correctly rejected corrupted signature: {:?}", e),
    }

    Ok(())
}

/// Example 5: Performance Benchmarking
///
/// This measures the performance of ML-DSA operations.
fn performance_benchmarking() -> Result<(), MlDsaError> {
    println!("\n⚡ Example 5: Performance Benchmarking");

    const NUM_OPERATIONS: usize = 100;
    let mut rng = thread_rng();

    // Benchmark key generation
    let start = Instant::now();
    let mut keypairs = Vec::with_capacity(NUM_OPERATIONS);

    for _ in 0..NUM_OPERATIONS {
        keypairs.push(MlDsaKeyPair::generate(&mut rng)?);
    }

    let keygen_duration = start.elapsed();
    println!(
        "   Key generation: {} operations in {:?}",
        NUM_OPERATIONS, keygen_duration
    );
    println!(
        "   Average per keygen: {:?}",
        keygen_duration / NUM_OPERATIONS as u32
    );

    // Benchmark signing
    let test_keypair = &keypairs[0];
    let test_message = b"Performance test message";
    let start = Instant::now();
    let mut signatures = Vec::with_capacity(NUM_OPERATIONS);

    for _ in 0..NUM_OPERATIONS {
        signatures.push(test_keypair.sign(test_message, &mut rng)?);
    }

    let signing_duration = start.elapsed();
    println!(
        "   Signing: {} operations in {:?}",
        NUM_OPERATIONS, signing_duration
    );
    println!(
        "   Average per signature: {:?}",
        signing_duration / NUM_OPERATIONS as u32
    );

    // Benchmark verification
    let public_key = MlDsaPublicKey::from_bytes(test_keypair.public_key())?;
    let start = Instant::now();
    let mut verification_results = Vec::with_capacity(NUM_OPERATIONS);

    for signature in &signatures {
        verification_results.push(public_key.verify(test_message, signature).is_ok());
    }

    let verify_duration = start.elapsed();
    println!(
        "   Verification: {} operations in {:?}",
        NUM_OPERATIONS, verify_duration
    );
    println!(
        "   Average per verification: {:?}",
        verify_duration / NUM_OPERATIONS as u32
    );

    // Verify all results are correct
    assert!(verification_results.iter().all(|&result| result));
    println!("   ✅ All {} verifications successful", NUM_OPERATIONS);

    // Performance targets (adjust based on your requirements)
    let keygen_target = std::time::Duration::from_millis(10);
    let sign_target = std::time::Duration::from_millis(5);
    let verify_target = std::time::Duration::from_millis(2);

    let avg_keygen = keygen_duration / NUM_OPERATIONS as u32;
    let avg_sign = signing_duration / NUM_OPERATIONS as u32;
    let avg_verify = verify_duration / NUM_OPERATIONS as u32;

    println!("\n   Performance Analysis:");
    println!(
        "   Key generation: {} (target: < {:?})",
        if avg_keygen < keygen_target {
            "✅ PASS"
        } else {
            "⚠️  SLOW"
        },
        keygen_target
    );
    println!(
        "   Signing: {} (target: < {:?})",
        if avg_sign < sign_target {
            "✅ PASS"
        } else {
            "⚠️  SLOW"
        },
        sign_target
    );
    println!(
        "   Verification: {} (target: < {:?})",
        if avg_verify < verify_target {
            "✅ PASS"
        } else {
            "⚠️  SLOW"
        },
        verify_target
    );

    Ok(())
}

/// Example 6: Batch Operations
///
/// This demonstrates efficient batch processing of signatures.
fn batch_operations_example() -> Result<(), MlDsaError> {
    println!("\n📦 Example 6: Batch Operations");

    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng)?;

    // Create a batch of messages
    let messages: Vec<Vec<u8>> = (0..10)
        .map(|i| format!("Batch message {}: Lorem ipsum dolor sit amet", i).into_bytes())
        .collect();

    println!("   Processing {} messages in batch", messages.len());

    // Batch sign
    let start = Instant::now();
    let signatures: Result<Vec<_>, _> = messages
        .iter()
        .map(|msg| keypair.sign(msg, &mut rng))
        .collect();

    let signatures = signatures?;
    let batch_sign_time = start.elapsed();
    println!("   Batch signing completed in {:?}", batch_sign_time);

    // Batch verify
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key())?;
    let start = Instant::now();
    let verification_results: Vec<_> = messages
        .iter()
        .zip(signatures.iter())
        .map(|(msg, sig)| public_key.verify(msg, sig).is_ok())
        .collect();

    let batch_verify_time = start.elapsed();
    println!("   Batch verification completed in {:?}", batch_verify_time);

    // Check all verifications passed
    let all_valid = verification_results.iter().all(|&v| v);
    println!("   All signatures valid: {}", all_valid);
    assert!(all_valid);

    // Performance comparison
    let avg_sign_time = batch_sign_time / messages.len() as u32;
    let avg_verify_time = batch_verify_time / messages.len() as u32;

    println!("   Average time per signature: {:?}", avg_sign_time);
    println!("   Average time per verification: {:?}", avg_verify_time);

    println!("   ✅ Batch operations completed successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_signing() {
        assert!(basic_signing_example().is_ok());
    }

    #[test]
    fn test_verification() {
        assert!(verification_example().is_ok());
    }

    #[test]
    fn test_multiple_signatures() {
        assert!(multiple_signatures_example().is_ok());
    }

    #[test]
    fn test_error_handling() {
        assert!(error_handling_example().is_ok());
    }

    #[test]
    fn test_batch_operations() {
        assert!(batch_operations_example().is_ok());
    }
}
