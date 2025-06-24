//! Timing attack resistance tests for QuDAG Exchange

use std::time::{Duration, Instant};
use qudag_exchange::security::{SecurityConfig, TimingGuard, constant_time};
use subtle::ConstantTimeEq;
use rand::{thread_rng, Rng};

/// Test that cryptographic operations complete in constant time
#[test]
fn test_constant_time_crypto_operations() {
    use qudag_crypto::ml_dsa::MlDsaKeyPair;
    use qudag_crypto::ml_kem::MlKem768;
    
    let mut rng = thread_rng();
    const ITERATIONS: usize = 100;
    
    // Test ML-KEM decapsulation timing
    let (pk, sk) = MlKem768::keygen().expect("Key generation failed");
    let mut decap_times = Vec::with_capacity(ITERATIONS);
    
    for _ in 0..ITERATIONS {
        let (ct, _) = MlKem768::encapsulate(&pk).expect("Encapsulation failed");
        
        let start = Instant::now();
        let _ = MlKem768::decapsulate(&sk, &ct);
        let elapsed = start.elapsed();
        
        decap_times.push(elapsed.as_nanos());
    }
    
    // Calculate variance
    let mean = decap_times.iter().sum::<u128>() / ITERATIONS as u128;
    let variance = decap_times.iter()
        .map(|&time| {
            let diff = if time > mean { time - mean } else { mean - time };
            diff * diff
        })
        .sum::<u128>() / ITERATIONS as u128;
    
    let std_dev = (variance as f64).sqrt();
    let cv = std_dev / mean as f64; // Coefficient of variation
    
    // Timing should be consistent (low coefficient of variation)
    assert!(cv < 0.05, "ML-KEM timing variance too high: {:.2}%", cv * 100.0);
    
    // Test ML-DSA signing timing
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation failed");
    let mut sign_times = Vec::with_capacity(ITERATIONS);
    
    for _ in 0..ITERATIONS {
        let message: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        
        let start = Instant::now();
        let _ = keypair.sign(&message, &mut rng);
        let elapsed = start.elapsed();
        
        sign_times.push(elapsed.as_nanos());
    }
    
    // Calculate variance for signing
    let mean = sign_times.iter().sum::<u128>() / ITERATIONS as u128;
    let variance = sign_times.iter()
        .map(|&time| {
            let diff = if time > mean { time - mean } else { mean - time };
            diff * diff
        })
        .sum::<u128>() / ITERATIONS as u128;
    
    let std_dev = (variance as f64).sqrt();
    let cv = std_dev / mean as f64;
    
    assert!(cv < 0.05, "ML-DSA timing variance too high: {:.2}%", cv * 100.0);
}

/// Test constant-time comparison operations
#[test]
fn test_constant_time_comparison() {
    let mut rng = thread_rng();
    const ITERATIONS: usize = 1000;
    
    // Generate random data
    let data1: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
    let data2: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
    let data3 = data1.clone();
    
    // Time equal comparisons
    let mut equal_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = constant_time::compare_bytes(&data1, &data3);
        equal_times.push(start.elapsed().as_nanos());
    }
    
    // Time unequal comparisons
    let mut unequal_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = constant_time::compare_bytes(&data1, &data2);
        unequal_times.push(start.elapsed().as_nanos());
    }
    
    // Calculate means
    let equal_mean = equal_times.iter().sum::<u128>() / ITERATIONS as u128;
    let unequal_mean = unequal_times.iter().sum::<u128>() / ITERATIONS as u128;
    
    // Means should be very close (within 5%)
    let diff = if equal_mean > unequal_mean {
        equal_mean - unequal_mean
    } else {
        unequal_mean - equal_mean
    };
    
    let diff_percent = (diff as f64 / equal_mean as f64) * 100.0;
    assert!(diff_percent < 5.0, 
        "Timing difference between equal and unequal comparisons: {:.2}%", diff_percent);
}

/// Test timing guard functionality
#[test]
fn test_timing_guard() {
    let expected = Duration::from_millis(100);
    let tolerance = Duration::from_millis(10);
    
    // Test operation within bounds
    let guard = TimingGuard::new(expected, tolerance);
    std::thread::sleep(Duration::from_millis(100));
    assert!(guard.check().is_ok());
    
    // Test operation too fast
    let guard = TimingGuard::new(expected, tolerance);
    std::thread::sleep(Duration::from_millis(50));
    assert!(guard.check().is_err());
    
    // Test operation too slow
    let guard = TimingGuard::new(expected, tolerance);
    std::thread::sleep(Duration::from_millis(150));
    assert!(guard.check().is_err());
}

/// Test for timing leaks in error paths
#[test]
fn test_error_path_timing() {
    use qudag_crypto::ml_dsa::{MlDsaKeyPair, MlDsaPublicKey};
    
    let mut rng = thread_rng();
    const ITERATIONS: usize = 100;
    
    // Generate keypair and valid signature
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation failed");
    let message = b"test message";
    let valid_signature = keypair.sign(message, &mut rng).expect("Signing failed");
    let public_key = keypair.to_public_key().expect("Public key conversion failed");
    
    // Generate invalid signature
    let mut invalid_signature = valid_signature.clone();
    invalid_signature[0] ^= 0xFF; // Flip bits to make invalid
    
    // Time valid signature verification
    let mut valid_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = public_key.verify(message, &valid_signature);
        valid_times.push(start.elapsed().as_nanos());
    }
    
    // Time invalid signature verification
    let mut invalid_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = public_key.verify(message, &invalid_signature);
        invalid_times.push(start.elapsed().as_nanos());
    }
    
    // Calculate means
    let valid_mean = valid_times.iter().sum::<u128>() / ITERATIONS as u128;
    let invalid_mean = invalid_times.iter().sum::<u128>() / ITERATIONS as u128;
    
    // Timing should be similar for valid and invalid signatures
    let diff = if valid_mean > invalid_mean {
        valid_mean - invalid_mean
    } else {
        invalid_mean - valid_mean
    };
    
    let diff_percent = (diff as f64 / valid_mean as f64) * 100.0;
    assert!(diff_percent < 10.0, 
        "Timing difference between valid and invalid signatures: {:.2}%", diff_percent);
}

/// Test for cache timing attacks
#[test]
fn test_cache_timing_resistance() {
    use qudag_crypto::ml_kem::MlKem768;
    
    const ITERATIONS: usize = 50;
    
    // Generate keys
    let (pk1, sk1) = MlKem768::keygen().expect("Key generation failed");
    let (pk2, sk2) = MlKem768::keygen().expect("Key generation failed");
    
    // Warm up cache with first key
    for _ in 0..10 {
        let (ct, _) = MlKem768::encapsulate(&pk1).expect("Encapsulation failed");
        let _ = MlKem768::decapsulate(&sk1, &ct);
    }
    
    // Time operations with cached key
    let mut cached_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let (ct, _) = MlKem768::encapsulate(&pk1).expect("Encapsulation failed");
        
        let start = Instant::now();
        let _ = MlKem768::decapsulate(&sk1, &ct);
        cached_times.push(start.elapsed().as_nanos());
    }
    
    // Time operations with uncached key
    let mut uncached_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let (ct, _) = MlKem768::encapsulate(&pk2).expect("Encapsulation failed");
        
        let start = Instant::now();
        let _ = MlKem768::decapsulate(&sk2, &ct);
        uncached_times.push(start.elapsed().as_nanos());
    }
    
    // Calculate means
    let cached_mean = cached_times.iter().sum::<u128>() / ITERATIONS as u128;
    let uncached_mean = uncached_times.iter().sum::<u128>() / ITERATIONS as u128;
    
    // Cache effects should be minimal
    let diff = if cached_mean > uncached_mean {
        cached_mean - uncached_mean
    } else {
        uncached_mean - cached_mean
    };
    
    let diff_percent = (diff as f64 / uncached_mean as f64) * 100.0;
    assert!(diff_percent < 15.0, 
        "Cache timing difference too high: {:.2}%", diff_percent);
}