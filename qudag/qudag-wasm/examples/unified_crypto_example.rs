//! Example demonstrating the unified crypto API that works on both native and WASM

use qudag_wasm::crypto_unified::{
    get_crypto_capabilities, CryptoFeatureDetection, CryptoProvider, CurrentProvider, HashFunction,
    PlatformFeatures, QuantumResistantSigning, UnifiedBlake3, UnifiedMlDsa,
};

fn main() {
    println!("QuDAG Unified Crypto API Example\n");

    // Initialize the crypto provider
    match CurrentProvider::initialize() {
        Ok(_) => println!("✓ Crypto provider initialized successfully"),
        Err(e) => {
            eprintln!("✗ Failed to initialize crypto provider: {}", e);
            return;
        }
    }

    // Display platform information
    println!("\n=== Platform Information ===");
    println!("Provider: {}", CurrentProvider::name());
    println!("Version: {}", CurrentProvider::version());
    println!("Is Fallback: {}", CurrentProvider::is_fallback());

    // Check available features
    println!("\n=== Feature Detection ===");
    println!("ML-DSA: {}", PlatformFeatures::has_ml_dsa());
    println!("ML-KEM: {}", PlatformFeatures::has_ml_kem());
    println!("HQC: {}", PlatformFeatures::has_hqc());
    println!("BLAKE3: {}", PlatformFeatures::has_blake3());
    println!(
        "Quantum Fingerprint: {}",
        PlatformFeatures::has_quantum_fingerprint()
    );

    if let Some(notes) = PlatformFeatures::platform_notes() {
        println!("\nPlatform Notes: {}", notes);
    }

    // Display capability summary
    println!("\n=== Capability Summary ===");
    println!("{}", get_crypto_capabilities());

    // Demonstrate BLAKE3 hashing (available on all platforms)
    println!("\n=== BLAKE3 Hashing Demo ===");
    let data = b"Hello, QuDAG!";
    let hash = UnifiedBlake3::hash(data);
    println!("Data: {:?}", std::str::from_utf8(data).unwrap());
    println!("Hash: {}", hex::encode(&hash));
    println!("Hash Length: {} bytes", hash.len());

    // Demonstrate ML-DSA if available
    if PlatformFeatures::has_ml_dsa() {
        println!("\n=== ML-DSA Digital Signatures Demo ===");

        match UnifiedMlDsa::generate_keypair() {
            Ok((public_key, private_key)) => {
                println!("✓ Generated ML-DSA keypair");
                println!("  Public Key Size: {} bytes", public_key.size());
                println!("  Private Key Size: {} bytes", private_key.size());
                println!("  Algorithm: {}", public_key.algorithm());

                // Try to sign a message (might fail on some platforms)
                let message = b"Sign this message";
                match UnifiedMlDsa::sign(message, &private_key) {
                    Ok(signature) => {
                        println!("✓ Signed message successfully");
                        println!("  Signature Size: {} bytes", signature.size());

                        // Verify the signature
                        match UnifiedMlDsa::verify(message, &signature, &public_key) {
                            Ok(valid) => {
                                println!(
                                    "✓ Signature verification: {}",
                                    if valid { "VALID" } else { "INVALID" }
                                );
                            }
                            Err(e) => {
                                println!("✗ Verification failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Signing failed: {}", e);
                        println!("  (This is expected on some platforms)");
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to generate keypair: {}", e);
            }
        }
    } else {
        println!("\n✗ ML-DSA not available on this platform");
    }

    // List all available features
    println!("\n=== Available Features ===");
    let features = PlatformFeatures::available_features();
    for (i, feature) in features.iter().enumerate() {
        println!("{}. {}", i + 1, feature);
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_unified_crypto_demo() -> String {
    let mut output = String::new();

    // Initialize
    if let Err(e) = CurrentProvider::initialize() {
        return format!("Failed to initialize: {}", e);
    }

    // Get capabilities
    output.push_str(&get_crypto_capabilities());
    output.push_str("\n\n");

    // Test BLAKE3
    let hash = UnifiedBlake3::hash(b"Hello from WASM!");
    output.push_str(&format!("BLAKE3 Hash: {}\n", hex::encode(hash)));

    // Test ML-DSA if available
    if PlatformFeatures::has_ml_dsa() {
        match UnifiedMlDsa::generate_keypair() {
            Ok((pk, _sk)) => {
                output.push_str(&format!("ML-DSA Public Key Size: {} bytes\n", pk.size()));
            }
            Err(e) => {
                output.push_str(&format!("ML-DSA Error: {}\n", e));
            }
        }
    }

    output
}
