//! ML-KEM (Kyber) 768 Example
//!
//! This example demonstrates how to use ML-KEM-768 for key encapsulation.
//! ML-KEM is a quantum-resistant key encapsulation mechanism that allows
//! two parties to establish a shared secret over an insecure channel.

use qudag_crypto::{ml_kem::MlKem768, kem::{Ciphertext, KEMError, PublicKey, SecretKey, SharedSecret, KeyEncapsulation}};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 ML-KEM-768 Example");
    println!("====================");

    // Example 1: Basic Key Generation
    basic_key_generation()?;

    // Example 2: Key Encapsulation and Decapsulation
    encapsulation_decapsulation()?;

    // Example 3: Error Handling
    error_handling_examples()?;

    // Example 4: Performance Measurement
    performance_measurement()?;

    // Example 5: Serialization/Deserialization
    serialization_examples()?;

    println!("\n✅ All ML-KEM examples completed successfully!");
    Ok(())
}

/// Example 1: Basic Key Generation
///
/// This shows how to generate ML-KEM key pairs securely.
fn basic_key_generation() -> Result<(), KEMError> {
    println!("\n🔑 Example 1: Basic Key Generation");

    // Generate a new key pair
    let (public_key, secret_key) = MlKem768::keygen()?;

    // Access public key
    println!("   Public key size: {} bytes", public_key.as_bytes().len());

    // Access secret key (normally kept private!)
    println!("   Secret key size: {} bytes", secret_key.as_bytes().len());

    // Keys are automatically zeroized when dropped for security
    println!("   ✓ Key pair generated successfully");

    Ok(())
}

/// Example 2: Key Encapsulation and Decapsulation
///
/// This demonstrates the core ML-KEM operation: establishing a shared secret.
fn encapsulation_decapsulation() -> Result<(), KEMError> {
    println!("\n🔐 Example 2: Key Encapsulation and Decapsulation");

    // Alice generates her key pair
    let (alice_public_key, alice_secret_key) = MlKem768::keygen()?;
    println!("   Alice generated her key pair");

    // Alice shares her public key with Bob (this can be done over insecure channel)

    // Bob encapsulates a shared secret using Alice's public key
    let (ciphertext, bob_shared_secret) = MlKem768::encapsulate(&alice_public_key)?;
    println!("   Bob encapsulated shared secret");
    println!("   Ciphertext size: {} bytes", ciphertext.as_bytes().len());
    println!(
        "   Shared secret size: {} bytes",
        bob_shared_secret.as_bytes().len()
    );

    // Alice decapsulates the shared secret using her secret key
    let alice_shared_secret = MlKem768::decapsulate(&alice_secret_key, &ciphertext)?;
    println!("   Alice decapsulated shared secret");

    // Verify both parties have the same shared secret
    assert_eq!(alice_shared_secret.as_bytes(), bob_shared_secret.as_bytes());
    println!("   ✓ Both parties have identical shared secrets!");

    // The shared secret can now be used for symmetric encryption
    println!("   💡 Tip: Use this shared secret for AES-GCM or ChaCha20-Poly1305");

    Ok(())
}

/// Example 3: Error Handling
///
/// This shows proper error handling for ML-KEM operations.
fn error_handling_examples() -> Result<(), KEMError> {
    println!("\n⚠️  Example 3: Error Handling");

    // Generate valid key pair first
    let (public_key, secret_key) = MlKem768::keygen()?;

    // Example: Invalid ciphertext handling
    let invalid_ciphertext = Ciphertext::from_bytes(&vec![0u8; 10]); // Too short
    match invalid_ciphertext {
        Ok(_) => println!("   Unexpected: Invalid ciphertext was accepted"),
        Err(e) => println!("   ✓ Correctly rejected invalid ciphertext: {:?}", e),
    }

    // Example: Using proper error handling in applications
    match attempt_decapsulation(&secret_key) {
        Ok(secret) => println!(
            "   Decapsulation succeeded, secret size: {}",
            secret.as_bytes().len()
        ),
        Err(KEMError::InvalidLength) => println!("   ✓ Handled invalid length error"),
        Err(KEMError::KeyGenerationError) => println!("   ✓ Handled key generation error"),
        Err(e) => println!("   ✓ Handled other error: {:?}", e),
    }

    Ok(())
}

/// Helper function for error handling example
fn attempt_decapsulation(secret_key: &SecretKey) -> Result<SharedSecret, KEMError> {
    // Create a malformed ciphertext for demonstration
    let bad_ciphertext_data = vec![0u8; 1568]; // Correct size but wrong content
    let bad_ciphertext = Ciphertext::from_bytes(&bad_ciphertext_data)?;

    // This will likely fail due to invalid ciphertext content
    MlKem768::decapsulate(secret_key, &bad_ciphertext)
}

/// Example 4: Performance Measurement
///
/// This shows how to measure ML-KEM performance for your application.
fn performance_measurement() -> Result<(), KEMError> {
    println!("\n⚡ Example 4: Performance Measurement");

    const NUM_ITERATIONS: usize = 100;

    // Measure key generation performance
    let start = Instant::now();
    let mut keypairs = Vec::with_capacity(NUM_ITERATIONS);

    for _ in 0..NUM_ITERATIONS {
        keypairs.push(MlKem768::keygen()?);
    }

    let keygen_duration = start.elapsed();
    println!(
        "   Key generation: {} operations in {:?}",
        NUM_ITERATIONS, keygen_duration
    );
    println!(
        "   Average per keygen: {:?}",
        keygen_duration / NUM_ITERATIONS as u32
    );

    // Measure encapsulation performance
    let (test_public_key, test_secret_key) = &keypairs[0];
    let start = Instant::now();
    let mut encap_results = Vec::with_capacity(NUM_ITERATIONS);

    for _ in 0..NUM_ITERATIONS {
        encap_results.push(MlKem768::encapsulate(test_public_key)?);
    }

    let encap_duration = start.elapsed();
    println!(
        "   Encapsulation: {} operations in {:?}",
        NUM_ITERATIONS, encap_duration
    );
    println!(
        "   Average per encap: {:?}",
        encap_duration / NUM_ITERATIONS as u32
    );

    // Measure decapsulation performance
    let start = Instant::now();

    for (ciphertext, _) in &encap_results {
        let _shared_secret = MlKem768::decapsulate(test_secret_key, ciphertext)?;
    }

    let decap_duration = start.elapsed();
    println!(
        "   Decapsulation: {} operations in {:?}",
        NUM_ITERATIONS, decap_duration
    );
    println!(
        "   Average per decap: {:?}",
        decap_duration / NUM_ITERATIONS as u32
    );

    println!("   💡 Tip: For production, aim for <1ms per operation");

    Ok(())
}

/// Example 5: Serialization/Deserialization
///
/// This shows how to serialize keys and ciphertexts for storage or transmission.
fn serialization_examples() -> Result<(), KEMError> {
    println!("\n💾 Example 5: Serialization/Deserialization");

    // Generate key pair
    let (public_key, secret_key) = MlKem768::keygen()?;

    // Serialize public key
    let public_key_bytes = public_key.as_bytes();
    println!("   Serialized public key: {} bytes", public_key_bytes.len());

    // Deserialize public key
    let reconstructed_public_key = PublicKey::from_bytes(public_key_bytes)?;
    println!("   ✓ Public key deserialized successfully");

    // Verify they're equivalent by using both for encapsulation
    let (ciphertext1, secret1) = MlKem768::encapsulate(&public_key)?;
    let (ciphertext2, secret2) = MlKem768::encapsulate(&reconstructed_public_key)?;

    // Both should work for decapsulation
    let decap_secret1 = MlKem768::decapsulate(&secret_key, &ciphertext1)?;
    let decap_secret2 = MlKem768::decapsulate(&secret_key, &ciphertext2)?;

    assert_eq!(secret1.as_bytes(), decap_secret1.as_bytes());
    assert_eq!(secret2.as_bytes(), decap_secret2.as_bytes());
    println!("   ✓ Both original and reconstructed keys work correctly");

    // Serialize ciphertext
    let ciphertext_bytes = ciphertext1.as_bytes();
    println!("   Serialized ciphertext: {} bytes", ciphertext_bytes.len());

    // Deserialize ciphertext
    let reconstructed_ciphertext = Ciphertext::from_bytes(ciphertext_bytes)?;
    let final_secret = MlKem768::decapsulate(&secret_key, &reconstructed_ciphertext)?;

    assert_eq!(secret1.as_bytes(), final_secret.as_bytes());
    println!("   ✓ Ciphertext serialization/deserialization works correctly");

    println!("   💡 Tip: Always verify deserialized data before use in production");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_key_generation() {
        assert!(basic_key_generation().is_ok());
    }

    #[test]
    fn test_encapsulation_decapsulation() {
        assert!(encapsulation_decapsulation().is_ok());
    }

    #[test]
    fn test_serialization_examples() {
        assert!(serialization_examples().is_ok());
    }

    #[test]
    fn test_performance_measurement() {
        // Use smaller iteration count for tests
        assert!(performance_measurement().is_ok());
    }
}
