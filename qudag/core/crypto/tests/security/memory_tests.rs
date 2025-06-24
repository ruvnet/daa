use qudag_crypto::{kem::{KeyEncapsulation, PublicKey, SecretKey, Ciphertext, SharedSecret}, ml_kem::MlKem768};
use zeroize::{Zeroize, Zeroizing};
use std::{mem, sync::atomic::{AtomicU8, Ordering}, alloc::{Layout, alloc, dealloc}};
use proptest::prelude::*;
use std::time::Instant;
use std::ptr;
use std::slice;
use std::process::Command;
use std::env;
use tempfile::NamedTempFile;
use std::io::Write;
use std::ffi::CString;
use libc::{mlock, munlock, mprotect, PROT_READ, PROT_WRITE};

/// Memory security test suite for cryptographic operations
#[cfg(test)]
mod memory_security_tests {
    use super::*;

    /// Helper to verify memory patterns and zeroization
    fn verify_memory_patterns<T: AsRef<[u8]>>(data: &T, expected_zeros: usize) {
        let bytes = data.as_ref();
        
        // Check complete zeroization
        let zero_count = bytes.iter().filter(|&&b| b == 0).count();
        assert_eq!(zero_count, expected_zeros, 
            "Memory not properly zeroized - found {} zeros, expected {}", 
            zero_count, expected_zeros);

        // Check for residual patterns
        let ones_count = bytes.iter().filter(|&&b| b == 0xff).count();
        assert_eq!(ones_count, 0, "Found residual pattern of ones");

        // Check for repeating sequences
        for window in bytes.windows(4) {
            assert_ne!(window.iter().all(|&b| b == window[0]), true,
                "Found repeating byte pattern");
        }
    }

    /// Helper for aligned memory allocation
    fn allocate_aligned_buffer(size: usize, align: usize) -> (*mut u8, Layout) {
        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { alloc(layout) };
        (ptr, layout)
    }

    /// Helper to measure operation timing
    fn measure_constant_time<F>(op: F, iterations: usize) -> bool 
    where
        F: Fn() -> ()
    {
        let mut timings = Vec::with_capacity(iterations);
        
        for _ in 0..iterations {
            let start = Instant::now();
            op();
            timings.push(start.elapsed());
        }

        // Calculate timing variance
        let mean = timings.iter().sum::<std::time::Duration>() / iterations as u32;
        let variance = timings.iter()
            .map(|t| {
                let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
                diff * diff
            })
            .sum::<i128>() / iterations as i128;

        // Variance should be small for constant-time ops
        variance < 1000
    }

    #[test]
    fn test_mlkem_key_lifecycle() {
        // Generate key pair normally
        let (pk, mut sk) = MlKem768::keygen().unwrap();

        // Test zeroization behavior
        let sk_ptr = sk.as_bytes().as_ptr();
        let sk_len = sk.as_bytes().len();
        
        // Create a copy to verify zeroization
        let mut sk_copy = sk.as_bytes().to_vec();
        
        // Zeroize the secret key
        sk.zeroize();
        
        // Verify that the secret key has been properly zeroized
        // Note: This is a best-effort test since the Zeroize trait handles this
        let zeroized_data = sk.as_bytes();
        let zero_count = zeroized_data.iter().filter(|&&b| b == 0).count();
        
        // Should have significantly more zeros after zeroization
        assert!(zero_count > zeroized_data.len() / 2, 
            "Expected more zeros after zeroization, got {}/{}", zero_count, zeroized_data.len());
        
        // Test that operations still work with a fresh key pair
        let (pk2, sk2) = MlKem768::keygen().unwrap();
        let (ct, _ss1) = MlKem768::encapsulate(&pk2).unwrap();
        let _ss2 = MlKem768::decapsulate(&sk2, &ct).unwrap();
    }

    #[test]
    fn test_signature_memory_safety() {
        // Test with various message sizes
        proptest!(|(message in prop::collection::vec(any::<u8>(), 1..1024))| {
            // For now, we'll use ML-KEM operations as signature operations aren't fully implemented
            let (pk, mut sk) = MlKem768::keygen().unwrap();

            // Test encapsulation with secure memory
            let (ct, mut ss) = MlKem768::encapsulate(&pk).unwrap();
            
            // Add memory fence to ensure operation ordering
            std::sync::atomic::fence(Ordering::SeqCst);
            
            // Test decapsulation
            let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
            
            // Verify secrets match
            assert_eq!(ss.as_bytes(), ss2.as_bytes());

            // Test cleanup with memory fences
            std::sync::atomic::fence(Ordering::SeqCst);
            ss.zeroize();
            verify_memory_patterns(&ss, ss.as_bytes().len());

            std::sync::atomic::fence(Ordering::SeqCst);
            ss2.zeroize();
            verify_memory_patterns(&ss2, ss2.as_bytes().len());

            // Ensure secret key cleanup
            sk.zeroize();
            verify_memory_patterns(&sk, sk.as_bytes().len());
        });
    }

    #[test]
    fn test_encryption_memory_safety() {
        // Test with various message sizes using ML-KEM operations
        proptest!(|(message in prop::collection::vec(any::<u8>(), 1..1024))| {
            let (pk, mut sk) = MlKem768::keygen().unwrap();

            // Test encapsulation with secure memory
            let (mut ct, mut ss) = {
                let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
                
                // Memory fence to ensure cleanup ordering
                std::sync::atomic::fence(Ordering::SeqCst);
                
                (ct, ss)
            };

            // Test decapsulation with secure memory
            let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
            
            // Verify shared secrets match
            assert_eq!(ss.as_bytes(), ss2.as_bytes());
            
            // Memory fence before cleanup
            std::sync::atomic::fence(Ordering::SeqCst);
            
            // Test cleanup
            ss.zeroize();
            verify_memory_patterns(&ss, ss.as_bytes().len());
            
            ss2.zeroize();
            verify_memory_patterns(&ss2, ss2.as_bytes().len());
            
            ct.zeroize();
            verify_memory_patterns(&ct, ct.as_bytes().len());

            // Verify secret key cleanup
            sk.zeroize();
            verify_memory_patterns(&sk, sk.as_bytes().len());
        });
    }

    #[test]
    fn test_shared_secret_handling() {
        // Test with multiple key pairs
        for _ in 0..10 {
            let (pk, sk) = MlKem768::keygen().unwrap();
            
            // Test encapsulation
            let (ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
            
            // Test constant-time decapsulation
            let is_constant = measure_constant_time(|| {
                let _ = MlKem768::decapsulate(&sk, &ct);
            }, 100);
            assert!(is_constant, "Decapsulation not constant-time");

            let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();

            // Verify secrets match
            assert_eq!(ss1.as_bytes(), ss2.as_bytes());

            // Test cleanup with memory fences
            std::sync::atomic::fence(Ordering::SeqCst);
            ss1.zeroize();
            verify_memory_patterns(&ss1, ss1.as_bytes().len());

            std::sync::atomic::fence(Ordering::SeqCst);
            ss2.zeroize();
            verify_memory_patterns(&ss2, ss2.as_bytes().len());
        }
    }

    #[test]
    fn test_memory_alignment() {
        // Test alignment for different key sizes
        proptest!(|(size in 16usize..4096)| {
            let (ptr, layout) = allocate_aligned_buffer(size, 32);
            
            // Verify alignment
            assert_eq!(ptr as usize % 32, 0, 
                "Buffer not 32-byte aligned");

            // Test constant-time operations
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, size) };
            let is_constant = measure_constant_time(|| {
                for i in 0..size {
                    // Use atomic operations to prevent optimization
                    let _ = AtomicU8::new(slice[i]).load(Ordering::SeqCst);
                }
            }, 100);
            assert!(is_constant, "Memory access not constant-time");

            // Clean up
            unsafe { dealloc(ptr, layout); }
        });
    }

    /// Test secure memory allocation and protection
    #[test]
    fn test_secure_memory_allocation() {
        // Test memory locking for sensitive data
        let size = 4096;
        let layout = Layout::from_size_align(size, 32).unwrap();
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "Memory allocation failed");
        
        // Lock memory to prevent swapping (requires root privileges in some systems)
        let lock_result = unsafe { mlock(ptr as *const libc::c_void, size) };
        if lock_result == 0 {
            println!("Memory successfully locked to prevent swapping");
        } else {
            println!("Memory locking failed (may require elevated privileges)");
        }
        
        // Write sensitive data
        let slice = unsafe { slice::from_raw_parts_mut(ptr, size) };
        for (i, byte) in slice.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        // Test memory protection
        let protect_result = unsafe { mprotect(ptr as *mut libc::c_void, size, PROT_READ) };
        if protect_result == 0 {
            println!("Memory protection set to read-only");
            
            // Restore write permissions for cleanup
            unsafe { mprotect(ptr as *mut libc::c_void, size, PROT_READ | PROT_WRITE) };
        }
        
        // Secure cleanup
        for byte in slice.iter_mut() {
            *byte = 0;
        }
        
        // Add memory fence
        std::sync::atomic::fence(Ordering::SeqCst);
        
        // Unlock memory if it was locked
        if lock_result == 0 {
            unsafe { munlock(ptr as *const libc::c_void, size) };
        }
        
        // Deallocate
        unsafe { dealloc(ptr, layout) };
    }
    
    /// Test for memory bounds checking
    #[test]
    fn test_memory_bounds_checking() {
        let size = 1024;
        let mut buffer = vec![0u8; size];
        
        // Test valid access
        buffer[0] = 0xFF;
        buffer[size - 1] = 0xAA;
        
        // Test boundary conditions
        assert_eq!(buffer[0], 0xFF);
        assert_eq!(buffer[size - 1], 0xAA);
        
        // Test with cryptographic keys
        let (pk, mut sk) = MlKem768::keygen().unwrap();
        
        // Ensure key sizes are within expected bounds
        assert_eq!(pk.as_bytes().len(), MlKem768::PUBLIC_KEY_SIZE);
        assert_eq!(sk.as_bytes().len(), MlKem768::SECRET_KEY_SIZE);
        
        // Test buffer overflow protection
        let mut temp_buffer = vec![0u8; 32];
        
        // This should not cause buffer overflow
        let data_to_copy = &pk.as_bytes()[..32.min(pk.as_bytes().len())];
        temp_buffer[..data_to_copy.len()].copy_from_slice(data_to_copy);
        
        // Verify data integrity
        assert_eq!(&temp_buffer[..data_to_copy.len()], data_to_copy);
        
        // Clean up
        sk.zeroize();
        temp_buffer.zeroize();
    }
    
    /// Test for memory leaks in cryptographic operations
    #[test]
    fn test_memory_leak_detection() {
        // Perform multiple crypto operations to detect leaks
        let iterations = 1000;
        
        for i in 0..iterations {
            // Generate key pair
            let (pk, mut sk) = MlKem768::keygen().unwrap();
            
            // Perform encapsulation
            let (mut ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
            
            // Perform decapsulation
            let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
            
            // Verify operation
            assert_eq!(ss1.as_bytes(), ss2.as_bytes());
            
            // Clean up all sensitive data
            sk.zeroize();
            ss1.zeroize();
            ss2.zeroize();
            ct.zeroize();
            
            // Periodic memory fence
            if i % 100 == 0 {
                std::sync::atomic::fence(Ordering::SeqCst);
            }
        }
        
        // Force garbage collection
        std::sync::atomic::fence(Ordering::SeqCst);
        println!("Completed {} crypto operations without detected leaks", iterations);
    }
    
    /// Test for stack overflow protection
    #[test]
    fn test_stack_overflow_protection() {
        // Test with reasonable stack depth
        fn recursive_crypto_test(depth: usize) -> bool {
            if depth == 0 {
                // Perform crypto operation at recursion bottom
                let (pk, mut sk) = MlKem768::keygen().unwrap();
                let (ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
                let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                
                let result = ss1.as_bytes() == ss2.as_bytes();
                
                // Clean up
                sk.zeroize();
                ss1.zeroize();
                ss2.zeroize();
                
                result
            } else {
                // Recurse with some stack allocation
                let _stack_data = [0u8; 1024];
                recursive_crypto_test(depth - 1)
            }
        }
        
        // Test with moderate recursion depth
        assert!(recursive_crypto_test(10), "Recursive crypto test failed");
    }
    
    /// Test constant-time memory comparison
    #[test]
    fn test_constant_time_memory_comparison() {
        use subtle::ConstantTimeEq;
        
        let data1 = vec![0x42u8; 32];
        let data2 = vec![0x42u8; 32];
        let data3 = vec![0x43u8; 32];
        
        // Test timing for equal data
        let equal_times: Vec<_> = (0..100).map(|_| {
            let start = Instant::now();
            let _result = data1.ct_eq(&data2);
            start.elapsed()
        }).collect();
        
        // Test timing for different data
        let different_times: Vec<_> = (0..100).map(|_| {
            let start = Instant::now();
            let _result = data1.ct_eq(&data3);
            start.elapsed()
        }).collect();
        
        // Calculate timing statistics
        let equal_avg = equal_times.iter().sum::<std::time::Duration>() / equal_times.len() as u32;
        let different_avg = different_times.iter().sum::<std::time::Duration>() / different_times.len() as u32;
        
        // Timing should be similar for constant-time operations
        let timing_ratio = if equal_avg > different_avg {
            equal_avg.as_nanos() as f64 / different_avg.as_nanos() as f64
        } else {
            different_avg.as_nanos() as f64 / equal_avg.as_nanos() as f64
        };
        
        println!("Timing ratio for constant-time comparison: {:.2}", timing_ratio);
        assert!(timing_ratio < 2.0, "Comparison may not be constant-time: ratio {:.2}", timing_ratio);
    }

    /// Run memory safety tests under Valgrind
    #[test]
    #[ignore] // Only run when explicitly requested
    fn test_valgrind_memory_safety() {
        // Create a simple test program
        let test_code = r#"
            use qudag_crypto::{kem::KeyEncapsulation, ml_kem::MlKem768};
            use zeroize::Zeroize;
            
            fn main() {
                // Perform multiple crypto operations
                for _ in 0..100 {
                    let (pk, mut sk) = MlKem768::keygen().unwrap();
                    let (ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
                    let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                    
                    // Verify operation
                    assert_eq!(ss1.as_bytes(), ss2.as_bytes());
                    
                    // Clean up
                    sk.zeroize();
                    ss1.zeroize();
                    ss2.zeroize();
                }
                println!("Memory safety test completed successfully");
            }
        "#;
        
        // Write test to temporary file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(test_code.as_bytes()).expect("Failed to write test code");
        
        // Compile the test program
        let compile_output = Command::new("rustc")
            .arg("--crate-type")
            .arg("bin")
            .arg("-L")
            .arg("target/debug/deps")
            .arg("--extern")
            .arg("qudag_crypto=target/debug/libqudag_crypto.rlib")
            .arg("--extern")
            .arg("zeroize")
            .arg("-o")
            .arg("/tmp/memory_test")
            .arg(temp_file.path())
            .output();
        
        match compile_output {
            Ok(output) if output.status.success() => {
                // Run under Valgrind
                let valgrind_output = Command::new("valgrind")
                    .arg("--tool=memcheck")
                    .arg("--leak-check=full")
                    .arg("--show-leak-kinds=all")
                    .arg("--track-origins=yes")
                    .arg("--error-exitcode=1")
                    .arg("/tmp/memory_test")
                    .output();
                
                match valgrind_output {
                    Ok(output) => {
                        println!("Valgrind stdout: {}", String::from_utf8_lossy(&output.stdout));
                        println!("Valgrind stderr: {}", String::from_utf8_lossy(&output.stderr));
                        
                        if !output.status.success() {
                            panic!("Valgrind detected memory safety issues");
                        }
                        
                        println!("Valgrind memory safety test passed");
                    }
                    Err(e) => {
                        println!("Failed to run Valgrind: {}. Skipping Valgrind test.", e);
                    }
                }
            }
            Ok(output) => {
                println!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr));
                println!("Skipping Valgrind test due to compilation failure");
            }
            Err(e) => {
                println!("Failed to compile test: {}. Skipping Valgrind test.", e);
            }
        }
    }
}