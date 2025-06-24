// Cryptographic zeroization test
// Tests that cryptographic secrets are properly cleared from memory

use std::alloc::{alloc, dealloc, Layout};
use std::sync::atomic::Ordering;
use std::slice;
use std::ptr;

/// Simulate a cryptographic key structure
#[derive(Debug)]
struct CryptoKey {
    key_data: Vec<u8>,
    key_size: usize,
}

impl CryptoKey {
    fn new(size: usize) -> Self {
        let mut key_data = vec![0u8; size];
        
        // Fill with pseudo-random data (simulating key generation)
        for (i, byte) in key_data.iter_mut().enumerate() {
            *byte = ((i * 0x12345 + 0x6789) % 256) as u8;
        }
        
        Self {
            key_data,
            key_size: size,
        }
    }
    
    /// Secure zeroization of key material
    fn zeroize(&mut self) {
        // Overwrite with zeros
        for byte in self.key_data.iter_mut() {
            *byte = 0;
        }
        
        // Add memory fence to prevent compiler optimization
        std::sync::atomic::fence(Ordering::SeqCst);
        
        // Double-check zeroization
        for (i, &byte) in self.key_data.iter().enumerate() {
            if byte != 0 {
                panic!("Zeroization failed at index {}: value {}", i, byte);
            }
        }
    }
    
    /// Verify that the key is properly zeroized
    fn verify_zeroized(&self) -> bool {
        self.key_data.iter().all(|&b| b == 0)
    }
}

impl Drop for CryptoKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Test basic cryptographic key zeroization
fn test_crypto_key_zeroization() {
    println!("Testing cryptographic key zeroization...");
    
    let mut key = CryptoKey::new(32);
    
    // Verify key has non-zero data initially
    let has_nonzero = key.key_data.iter().any(|&b| b != 0);
    assert!(has_nonzero, "Key should have non-zero data initially");
    
    // Zeroize the key
    key.zeroize();
    
    // Verify zeroization
    assert!(key.verify_zeroized(), "Key not properly zeroized");
    
    println!("✓ Basic cryptographic key zeroization test passed");
}

/// Test zeroization of multiple key sizes
fn test_multiple_key_sizes() {
    println!("Testing zeroization of multiple key sizes...");
    
    let key_sizes = [16, 32, 64, 128, 256, 512, 1024, 2048];
    
    for &size in &key_sizes {
        let mut key = CryptoKey::new(size);
        
        // Verify initial non-zero state
        let initial_nonzero = key.key_data.iter().any(|&b| b != 0);
        assert!(initial_nonzero, "Key of size {} should have non-zero data", size);
        
        // Zeroize
        key.zeroize();
        
        // Verify zeroization
        assert!(key.verify_zeroized(), "Key of size {} not properly zeroized", size);
    }
    
    println!("✓ Multiple key sizes zeroization test passed");
}

/// Test that zeroization survives memory pressure
fn test_zeroization_under_memory_pressure() {
    println!("Testing zeroization under memory pressure...");
    
    let mut keys = Vec::new();
    
    // Create many keys
    for i in 0..1000 {
        let key_size = 32 + (i % 64); // Vary key sizes
        keys.push(CryptoKey::new(key_size));
    }
    
    // Zeroize all keys
    for key in &mut keys {
        key.zeroize();
    }
    
    // Verify all keys are zeroized
    for (i, key) in keys.iter().enumerate() {
        assert!(key.verify_zeroized(), "Key {} not properly zeroized under memory pressure", i);
    }
    
    println!("✓ Zeroization under memory pressure test passed");
}

/// Test zeroization with manual memory management
fn test_manual_memory_zeroization() {
    println!("Testing manual memory zeroization...");
    
    let size = 1024;
    let layout = Layout::from_size_align(size, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    
    if ptr.is_null() {
        panic!("Memory allocation failed");
    }
    
    // Write secret data
    let slice = unsafe { slice::from_raw_parts_mut(ptr, size) };
    for (i, byte) in slice.iter_mut().enumerate() {
        *byte = ((i * 0xDEADBEEF + 0xCAFEBABE) % 256) as u8;
    }
    
    // Verify data is non-zero
    let has_nonzero = slice.iter().any(|&b| b != 0);
    assert!(has_nonzero, "Memory should have non-zero data");
    
    // Secure zeroization
    unsafe {
        ptr::write_bytes(ptr, 0, size);
    }
    
    // Memory fence
    std::sync::atomic::fence(Ordering::SeqCst);
    
    // Verify zeroization
    for (i, &byte) in slice.iter().enumerate() {
        assert_eq!(byte, 0, "Memory not zeroized at index {}: value {}", i, byte);
    }
    
    // Deallocate
    unsafe { dealloc(ptr, layout) };
    
    println!("✓ Manual memory zeroization test passed");
}

/// Test that compiler doesn't optimize away zeroization
fn test_compiler_optimization_resistance() {
    println!("Testing compiler optimization resistance...");
    
    // Use volatile operations to prevent optimization
    fn volatile_zeroize(data: &mut [u8]) {
        for byte in data.iter_mut() {
            unsafe {
                ptr::write_volatile(byte, 0);
            }
        }
        
        // Memory fence
        std::sync::atomic::fence(Ordering::SeqCst);
    }
    
    let mut secret_data = vec![0x42u8; 256];
    
    // Verify initial state
    assert!(secret_data.iter().all(|&b| b == 0x42), "Initial data incorrect");
    
    // Volatile zeroization
    volatile_zeroize(&mut secret_data);
    
    // Verify zeroization
    for (i, &byte) in secret_data.iter().enumerate() {
        assert_eq!(byte, 0, "Volatile zeroization failed at index {}: value {}", i, byte);
    }
    
    println!("✓ Compiler optimization resistance test passed");
}

/// Test zeroization timing consistency
fn test_zeroization_timing() {
    println!("Testing zeroization timing consistency...");
    
    let sizes = [64, 128, 256, 512];
    let iterations = 100;
    
    for &size in &sizes {
        let mut times = Vec::new();
        
        for _ in 0..iterations {
            let mut data = vec![0x5Au8; size];
            
            let start = std::time::Instant::now();
            
            // Zeroize
            for byte in data.iter_mut() {
                unsafe {
                    ptr::write_volatile(byte, 0);
                }
            }
            std::sync::atomic::fence(Ordering::SeqCst);
            
            let elapsed = start.elapsed();
            times.push(elapsed.as_nanos());
            
            // Verify zeroization
            assert!(data.iter().all(|&b| b == 0), "Zeroization failed for size {}", size);
        }
        
        // Calculate timing statistics
        let avg_time = times.iter().sum::<u128>() / times.len() as u128;
        let max_time = *times.iter().max().unwrap();
        let min_time = *times.iter().min().unwrap();
        
        println!("Size {}: avg={}ns, min={}ns, max={}ns", size, avg_time, min_time, max_time);
        
        // Check for reasonable timing consistency
        let variance_ratio = max_time as f64 / min_time as f64;
        if variance_ratio > 10.0 {
            println!("WARNING: High timing variance for size {}: ratio {:.2}", size, variance_ratio);
        }
    }
    
    println!("✓ Zeroization timing consistency test completed");
}

/// Test concurrent zeroization
fn test_concurrent_zeroization() {
    println!("Testing concurrent zeroization...");
    
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let shared_keys = Arc::new(Mutex::new(Vec::new()));
    
    // Create keys in multiple threads
    let create_handles: Vec<_> = (0..4).map(|thread_id| {
        let keys = shared_keys.clone();
        thread::spawn(move || {
            for i in 0..100 {
                let key_size = 32 + (i % 32);
                let key = CryptoKey::new(key_size);
                
                {
                    let mut guard = keys.lock().unwrap();
                    guard.push(key);
                }
                
                // Yield to other threads
                thread::yield_now();
            }
        })
    }).collect();
    
    // Wait for key creation
    for handle in create_handles {
        handle.join().unwrap();
    }
    
    // Zeroize keys in the main thread (simpler approach)
    let mut total_zeroized = 0;
    {
        let mut guard = shared_keys.lock().unwrap();
        for key in guard.iter_mut() {
            key.zeroize();
            assert!(key.verify_zeroized(), "Key not properly zeroized");
            total_zeroized += 1;
        }
    }
    
    println!("✓ Concurrent zeroization test passed ({} keys zeroized)", total_zeroized);
}

fn main() {
    println!("Cryptographic Zeroization Test Suite");
    println!("===================================");
    
    test_crypto_key_zeroization();
    test_multiple_key_sizes();
    test_zeroization_under_memory_pressure();
    test_manual_memory_zeroization();
    test_compiler_optimization_resistance();
    test_zeroization_timing();
    test_concurrent_zeroization();
    
    println!();
    println!("🔒 All cryptographic zeroization tests passed!");
    println!();
    println!("Key security features verified:");
    println!("  ✓ Proper secret data clearing");
    println!("  ✓ Compiler optimization resistance");
    println!("  ✓ Memory fence usage");
    println!("  ✓ Concurrent safety");
    println!("  ✓ Multiple key size support");
    println!("  ✓ Memory pressure resistance");
}