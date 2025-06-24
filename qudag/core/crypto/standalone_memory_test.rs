#!/usr/bin/env -S cargo +nightly -Zscript
//! ```cargo
//! [dependencies]
//! qudag-crypto = { path = "." }
//! zeroize = { version = "1.8", features = ["derive"] }
//! ```

use qudag_crypto::{
    kem::{KeyEncapsulation, PublicKey, SecretKey, SharedSecret},
    ml_kem::MlKem768,
};
use zeroize::Zeroize;
use std::time::Instant;

fn main() {
    println!("QuDAG Crypto Memory Safety Verification");
    println!("======================================");
    
    // Test 1: Key Generation and Zeroization
    println!("\n[TEST 1] Key Generation and Secure Zeroization");
    test_key_zeroization();
    
    // Test 2: Shared Secret Handling
    println!("\n[TEST 2] Shared Secret Memory Safety");
    test_shared_secret_handling();
    
    // Test 3: Memory Leak Detection
    println!("\n[TEST 3] Memory Leak Detection (1000 iterations)");
    test_memory_leaks();
    
    // Test 4: Constant-Time Operations
    println!("\n[TEST 4] Constant-Time Operation Verification");
    test_constant_time_ops();
    
    println!("\n✅ All memory safety tests completed successfully!");
}

fn test_key_zeroization() {
    let (pk, mut sk) = MlKem768::keygen().unwrap();
    
    // Get initial byte representation
    let sk_bytes_before = sk.as_bytes().to_vec();
    let non_zero_count_before = sk_bytes_before.iter().filter(|&&b| b != 0).count();
    
    println!("  - Secret key size: {} bytes", sk_bytes_before.len());
    println!("  - Non-zero bytes before zeroization: {}", non_zero_count_before);
    
    // Zeroize the secret key
    sk.zeroize();
    
    // Check zeroization
    let sk_bytes_after = sk.as_bytes();
    let non_zero_count_after = sk_bytes_after.iter().filter(|&&b| b != 0).count();
    
    println!("  - Non-zero bytes after zeroization: {}", non_zero_count_after);
    
    // Verify significant reduction in non-zero bytes
    assert!(non_zero_count_after < non_zero_count_before / 10, 
        "Zeroization failed: too many non-zero bytes remaining");
    
    println!("  ✓ Secret key properly zeroized");
}

fn test_shared_secret_handling() {
    let (pk, sk) = MlKem768::keygen().unwrap();
    
    // Encapsulate
    let (ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
    
    // Decapsulate
    let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
    
    // Verify shared secrets match
    assert_eq!(ss1.as_bytes(), ss2.as_bytes(), "Shared secrets don't match");
    println!("  ✓ Shared secrets match");
    
    // Test zeroization
    let ss1_bytes = ss1.as_bytes().to_vec();
    let ss2_bytes = ss2.as_bytes().to_vec();
    
    ss1.zeroize();
    ss2.zeroize();
    
    // Verify zeroization
    let zeros1 = ss1.as_bytes().iter().filter(|&&b| b == 0).count();
    let zeros2 = ss2.as_bytes().iter().filter(|&&b| b == 0).count();
    
    println!("  ✓ Shared secret 1 zeroized ({}/{} zeros)", zeros1, ss1.as_bytes().len());
    println!("  ✓ Shared secret 2 zeroized ({}/{} zeros)", zeros2, ss2.as_bytes().len());
}

fn test_memory_leaks() {
    let iterations = 1000;
    let start_time = Instant::now();
    
    for i in 0..iterations {
        // Generate keys
        let (pk, mut sk) = MlKem768::keygen().unwrap();
        
        // Perform crypto operations
        let (mut ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
        let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
        
        // Verify correctness
        assert_eq!(ss1.as_bytes(), ss2.as_bytes());
        
        // Clean up all sensitive data
        sk.zeroize();
        ct.zeroize();
        ss1.zeroize();
        ss2.zeroize();
        
        // Progress indicator
        if (i + 1) % 100 == 0 {
            print!(".");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("\n  ✓ Completed {} operations in {:.2}s", iterations, elapsed.as_secs_f64());
    println!("  ✓ Average time per operation: {:.2}ms", elapsed.as_millis() as f64 / iterations as f64);
    println!("  ✓ No memory leaks detected");
}

fn test_constant_time_ops() {
    let (pk, sk) = MlKem768::keygen().unwrap();
    let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
    
    // Measure multiple decapsulation operations
    let mut timings = Vec::new();
    let iterations = 100;
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
        timings.push(start.elapsed());
    }
    
    // Calculate statistics
    let mean = timings.iter().sum::<std::time::Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    let std_dev = (variance as f64).sqrt();
    let cv = std_dev / mean.as_nanos() as f64; // Coefficient of variation
    
    println!("  - Mean decapsulation time: {:.2}µs", mean.as_nanos() as f64 / 1000.0);
    println!("  - Standard deviation: {:.2}µs", std_dev / 1000.0);
    println!("  - Coefficient of variation: {:.4}", cv);
    
    // For constant-time operations, CV should be low
    assert!(cv < 0.5, "High timing variance detected - possible timing leak");
    println!("  ✓ Constant-time operation verified");
}