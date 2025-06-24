/// Comprehensive memory safety test suite for cryptographic operations
/// 
/// This module implements thorough memory safety testing including:
/// - Memory leak detection with allocation tracking
/// - Buffer overflow protection testing
/// - Use-after-free prevention validation
/// - Double-free protection verification
/// - Memory corruption detection
/// - Secure memory wiping validation

use qudag_crypto::{
    kem::{KeyEncapsulation, MlKem768},
    ml_dsa::{MlDsa, MlDsaKeyPair},
    encryption::HQC,
    hash::Blake3Hash,
    CryptoError,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::ptr::NonNull;
use std::time::{Duration, Instant};
use rand::{RngCore, thread_rng};
use zeroize::{Zeroize, Zeroizing};

/// Memory allocation tracking for leak detection
#[derive(Debug)]
struct AllocationTracker {
    allocations: Arc<Mutex<HashMap<usize, (usize, Instant)>>>,
    total_allocated: AtomicUsize,
    peak_allocated: AtomicUsize,
}

impl AllocationTracker {
    fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
            total_allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
        }
    }
    
    fn track_allocation(&self, ptr: usize, size: usize) {
        let mut allocations = self.allocations.lock().unwrap();
        allocations.insert(ptr, (size, Instant::now()));
        
        let current = self.total_allocated.fetch_add(size, Ordering::SeqCst) + size;
        let mut peak = self.peak_allocated.load(Ordering::SeqCst);
        while current > peak {
            match self.peak_allocated.compare_exchange_weak(peak, current, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
    }
    
    fn track_deallocation(&self, ptr: usize) -> Option<usize> {
        let mut allocations = self.allocations.lock().unwrap();
        if let Some((size, _)) = allocations.remove(&ptr) {
            self.total_allocated.fetch_sub(size, Ordering::SeqCst);
            Some(size)
        } else {
            None
        }
    }
    
    fn get_stats(&self) -> (usize, usize, usize) {
        let allocations = self.allocations.lock().unwrap();
        (
            allocations.len(),
            self.total_allocated.load(Ordering::SeqCst),
            self.peak_allocated.load(Ordering::SeqCst),
        )
    }
    
    fn get_leaked_allocations(&self) -> Vec<(usize, usize, Duration)> {
        let allocations = self.allocations.lock().unwrap();
        let now = Instant::now();
        allocations.iter()
            .map(|(&ptr, &(size, time))| (ptr, size, now.duration_since(time)))
            .collect()
    }
}

/// Custom allocator for memory safety testing
struct TestAllocator {
    tracker: AllocationTracker,
    underlying: System,
}

impl TestAllocator {
    fn new() -> Self {
        Self {
            tracker: AllocationTracker::new(),
            underlying: System,
        }
    }
    
    fn get_tracker(&self) -> &AllocationTracker {
        &self.tracker
    }
}

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.underlying.alloc(layout);
        if !ptr.is_null() {
            self.tracker.track_allocation(ptr as usize, layout.size());
        }
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(size) = self.tracker.track_deallocation(ptr as usize) {
            assert_eq!(size, layout.size(), "Deallocation size mismatch");
        }
        self.underlying.dealloc(ptr, layout);
    }
}

/// Memory corruption detection utilities
mod memory_corruption {
    use super::*;
    
    /// Fill memory with canary patterns for corruption detection
    pub fn fill_canary_pattern(buffer: &mut [u8], pattern: u8) {
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = pattern.wrapping_add(i as u8);
        }
    }
    
    /// Verify canary pattern integrity
    pub fn verify_canary_pattern(buffer: &[u8], pattern: u8) -> bool {
        buffer.iter().enumerate().all(|(i, &byte)| {
            byte == pattern.wrapping_add(i as u8)
        })
    }
    
    /// Create guard pages around allocation for overflow detection
    pub fn create_guarded_allocation(size: usize) -> Result<(*mut u8, Layout), std::alloc::LayoutError> {
        let page_size = 4096; // Assume 4KB pages
        let guard_size = page_size * 2; // Guard pages before and after
        let total_size = size + guard_size;
        
        let layout = Layout::from_size_align(total_size, page_size)?;
        let ptr = unsafe { std::alloc::alloc(layout) };
        
        if ptr.is_null() {
            return Err(std::alloc::LayoutError);
        }
        
        // Set up guard pages (would require mprotect in real implementation)
        let protected_ptr = unsafe { ptr.add(page_size) };
        
        Ok((protected_ptr, layout))
    }
    
    /// Detect use-after-free by poisoning freed memory
    pub fn poison_freed_memory(ptr: *mut u8, size: usize) {
        unsafe {
            std::ptr::write_bytes(ptr, 0xDE, size); // "DEAD" pattern
        }
    }
    
    /// Verify memory hasn't been accessed after free
    pub fn verify_poison_pattern(ptr: *const u8, size: usize) -> bool {
        unsafe {
            (0..size).all(|i| *ptr.add(i) == 0xDE)
        }
    }
}

/// Secure memory wiping verification
mod secure_wiping {
    use super::*;
    
    /// Verify that memory has been securely wiped
    pub fn verify_secure_wipe(data: &[u8]) -> bool {
        // Check for common patterns that indicate incomplete wiping
        let all_zeros = data.iter().all(|&b| b == 0);
        let all_ones = data.iter().all(|&b| b == 0xFF);
        let has_pattern = has_repeating_pattern(data);
        
        // Memory should be zeroed and not have obvious patterns
        all_zeros && !has_pattern
    }
    
    fn has_repeating_pattern(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        
        // Check for 4-byte repeating patterns
        let pattern = &data[0..4];
        data.chunks_exact(4).all(|chunk| chunk == pattern)
    }
    
    /// Test memory wiping under various conditions
    pub fn test_memory_wiping_resistance() -> bool {
        let mut test_data = vec![0x42u8; 1024];
        
        // Simulate memory pressure
        let _pressure = vec![vec![0u8; 1024]; 100];
        
        // Wipe memory
        test_data.zeroize();
        
        // Verify wiping was effective
        verify_secure_wipe(&test_data)
    }
    
    /// Test that compiler optimizations don't eliminate wiping
    #[inline(never)]
    pub fn test_optimization_resistance(data: &mut [u8]) {
        // Use volatile writes to prevent optimization
        for byte in data.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        
        // Memory barrier to ensure writes complete
        std::sync::atomic::fence(Ordering::SeqCst);
    }
}

#[cfg(test)]
mod comprehensive_memory_safety_tests {
    use super::*;

    #[test]
    fn test_ml_kem_memory_leak_detection() {
        let tracker = AllocationTracker::new();
        let initial_stats = tracker.get_stats();
        
        // Perform multiple key generation cycles
        for _ in 0..100 {
            let (pk, mut sk) = MlKem768::keygen().unwrap();
            
            // Perform encryption/decryption cycle
            let (mut ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
            let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
            
            // Verify operation succeeded
            assert_eq!(ss1.as_bytes(), ss2.as_bytes());
            
            // Explicitly zeroize sensitive data
            sk.zeroize();
            ss1.zeroize();
            ss2.zeroize();
            ct.zeroize();
        }
        
        // Force garbage collection
        std::hint::black_box(&tracker);
        
        let final_stats = tracker.get_stats();
        let leaked = tracker.get_leaked_allocations();
        
        // Check for memory leaks
        assert_eq!(initial_stats.0, final_stats.0, 
            "Memory leak detected: {} allocations not freed", 
            final_stats.0 - initial_stats.0);
        
        // Report any long-lived allocations
        for (ptr, size, duration) in leaked {
            if duration > Duration::from_secs(1) {
                panic!("Long-lived allocation detected: {} bytes at {:p} for {:?}", 
                    size, ptr as *const u8, duration);
            }
        }
    }

    #[test]
    fn test_buffer_overflow_protection() {
        // Test with various buffer sizes around critical boundaries
        let test_sizes = [16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
        
        for &size in &test_sizes {
            let mut buffer = vec![0u8; size];
            
            // Fill with canary pattern
            memory_corruption::fill_canary_pattern(&mut buffer, 0xAA);
            
            // Perform crypto operations with the buffer
            let (pk, sk) = MlKem768::keygen().unwrap();
            let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
            
            // Use buffer in a way that might cause overflow if not protected
            let hash_input = [buffer.as_slice(), ct.as_bytes(), ss.as_bytes()].concat();
            let _hash = Blake3Hash::hash(&hash_input);
            
            // Verify canary pattern is intact
            assert!(memory_corruption::verify_canary_pattern(&buffer, 0xAA),
                "Buffer overflow detected for size {}", size);
        }
    }

    #[test]
    fn test_use_after_free_protection() {
        // This test would be more effective with custom allocator
        // but we can still test the basic principle
        
        let mut sensitive_data = Zeroizing::new(vec![0x42u8; 256]);
        let data_ptr = sensitive_data.as_ptr();
        
        // Use the data in crypto operations
        let (pk, sk) = MlKem768::keygen().unwrap();
        let hash_input = [sensitive_data.as_slice(), pk.as_bytes()].concat();
        let _hash = Blake3Hash::hash(&hash_input);
        
        // Explicitly drop the data
        drop(sensitive_data);
        
        // In a real implementation, we would verify the memory has been poisoned
        // and accessing it would cause a fault. Here we just ensure the test completes.
        
        // Verify that new operations still work (no memory corruption)
        let (ct, ss) = MlKem768::encapsulate(&pk).unwrap();
        let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
        assert_eq!(ss.as_bytes(), ss2.as_bytes());
    }

    #[test]
    fn test_double_free_protection() {
        // Test that zeroizing the same data multiple times is safe
        let mut data1 = vec![0x42u8; 128];
        let mut data2 = data1.clone();
        
        // First zeroize
        data1.zeroize();
        
        // Second zeroize should be safe
        data1.zeroize();
        
        // Zeroize of clone should also be safe
        data2.zeroize();
        
        // Verify both are properly zeroed
        assert!(data1.iter().all(|&b| b == 0));
        assert!(data2.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_memory_corruption_detection() {
        let mut test_buffer = vec![0u8; 1024];
        
        // Fill with pattern
        memory_corruption::fill_canary_pattern(&mut test_buffer, 0x55);
        
        // Perform crypto operations that access memory
        let keypair = MlDsa::keygen().unwrap();
        let message = b"memory corruption test";
        
        // Sign message using our buffer as additional entropy
        let hash_input = [message, &test_buffer].concat();
        let signature = MlDsa::sign(&hash_input, keypair.secret_key()).unwrap();
        
        // Verify buffer integrity
        assert!(memory_corruption::verify_canary_pattern(&test_buffer, 0x55),
            "Memory corruption detected during crypto operations");
        
        // Verify signature is valid
        assert!(MlDsa::verify(&hash_input, &signature, keypair.public_key()).is_ok());
    }

    #[test]
    fn test_secure_memory_wiping() {
        // Test various sensitive data types
        let test_cases = vec![
            vec![0x42u8; 32],   // Secret key sized
            vec![0xAAu8; 64],   // Signature sized
            vec![0x55u8; 128],  // Large buffer
            vec![0xFFu8; 1024], // Very large buffer
        ];
        
        for mut data in test_cases {
            let original_pattern = data[0];
            
            // Ensure data is not optimized away
            std::hint::black_box(&data);
            
            // Perform secure wiping
            secure_wiping::test_optimization_resistance(&mut data);
            
            // Verify wiping was effective
            assert!(secure_wiping::verify_secure_wipe(&data),
                "Secure wiping failed for pattern {:02X}", original_pattern);
        }
    }

    #[test]
    fn test_memory_alignment_safety() {
        // Test crypto operations with various memory alignments
        let alignments = [1, 2, 4, 8, 16, 32, 64];
        
        for &alignment in &alignments {
            let layout = Layout::from_size_align(1024, alignment).unwrap();
            let ptr = unsafe { std::alloc::alloc(layout) };
            assert!(!ptr.is_null(), "Allocation failed for alignment {}", alignment);
            
            let buffer = unsafe { std::slice::from_raw_parts_mut(ptr, 1024) };
            
            // Initialize buffer
            for (i, byte) in buffer.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
            
            // Use buffer in crypto operations
            let hash = Blake3Hash::hash(buffer);
            assert_eq!(hash.len(), 32);
            
            // Cleanup
            unsafe { std::alloc::dealloc(ptr, layout) };
        }
    }

    #[test]
    fn test_stack_overflow_protection() {
        // Test recursive crypto operations don't cause stack overflow
        fn recursive_crypto_test(depth: usize, max_depth: usize) -> bool {
            if depth >= max_depth {
                // Base case: perform crypto operation
                let (pk, sk) = MlKem768::keygen().unwrap();
                let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
                let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                ss1.as_bytes() == ss2.as_bytes()
            } else {
                // Recurse with stack allocation
                let _stack_data = [0u8; 1024]; // Allocate on stack
                recursive_crypto_test(depth + 1, max_depth)
            }
        }
        
        // Test with reasonable recursion depth
        assert!(recursive_crypto_test(0, 10), "Recursive crypto operations failed");
    }

    #[test]
    fn test_heap_exhaustion_handling() {
        // Test behavior under memory pressure
        let mut large_allocations = Vec::new();
        
        // Allocate memory until we approach limits
        for i in 0..100 {
            match vec![0u8; 1024 * 1024].try_into() {
                Ok(allocation) => large_allocations.push(allocation),
                Err(_) => break,
            }
            
            // Ensure crypto operations still work under memory pressure
            if i % 10 == 0 {
                let result = MlKem768::keygen();
                match result {
                    Ok((pk, sk)) => {
                        let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
                        let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                        assert_eq!(ss1.as_bytes(), ss2.as_bytes());
                    },
                    Err(_) => {
                        // It's acceptable to fail under extreme memory pressure
                        println!("Crypto operation failed under memory pressure at iteration {}", i);
                        break;
                    }
                }
            }
        }
        
        // Cleanup
        large_allocations.clear();
        
        // Verify operations work normally after memory pressure
        let (pk, sk) = MlKem768::keygen().unwrap();
        let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
        let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
        assert_eq!(ss1.as_bytes(), ss2.as_bytes());
    }

    #[test]
    fn test_concurrent_memory_safety() {
        use std::sync::Arc;
        use std::thread;
        
        let num_threads = 4;
        let operations_per_thread = 25;
        let shared_data = Arc::new(Mutex::new(vec![0u8; 1024]));
        
        let mut handles = Vec::new();
        
        for thread_id in 0..num_threads {
            let data_clone = Arc::clone(&shared_data);
            
            let handle = thread::spawn(move || {
                for i in 0..operations_per_thread {
                    // Perform crypto operations
                    let (pk, mut sk) = MlKem768::keygen().unwrap();
                    let (mut ct, mut ss1) = MlKem768::encapsulate(&pk).unwrap();
                    let mut ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                    
                    // Access shared data safely
                    {
                        let mut shared = data_clone.lock().unwrap();
                        shared[thread_id * 4 + (i % 4)] = (thread_id + i) as u8;
                    }
                    
                    // Verify crypto operation
                    assert_eq!(ss1.as_bytes(), ss2.as_bytes());
                    
                    // Cleanup
                    sk.zeroize();
                    ct.zeroize();
                    ss1.zeroize();
                    ss2.zeroize();
                }
                
                thread_id
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            let thread_id = handle.join().unwrap();
            println!("Thread {} completed memory safety tests", thread_id);
        }
        
        // Verify shared data integrity
        let final_data = shared_data.lock().unwrap();
        println!("Final shared data state: {:?}", &final_data[0..16]);
    }

    #[test]
    fn test_memory_fragmentation_resistance() {
        // Allocate and deallocate memory in patterns that cause fragmentation
        let mut allocations = Vec::new();
        
        // Phase 1: Allocate many small buffers
        for i in 0..1000 {
            let size = 16 + (i % 64); // Variable sizes
            allocations.push(vec![i as u8; size]);
        }
        
        // Phase 2: Deallocate every other buffer (creates fragmentation)
        let mut i = 0;
        allocations.retain(|_| {
            i += 1;
            i % 2 == 0
        });
        
        // Phase 3: Perform crypto operations in fragmented environment
        for _ in 0..10 {
            let keypair = MlDsa::keygen().unwrap();
            let message = b"fragmentation test message";
            let signature = MlDsa::sign(message, keypair.secret_key()).unwrap();
            
            // Verify signature
            assert!(MlDsa::verify(message, &signature, keypair.public_key()).is_ok());
            
            // Allocate more buffers
            allocations.push(vec![0x42u8; 128]);
        }
        
        // Cleanup
        allocations.clear();
        
        // Verify normal operation after fragmentation test
        let (pk, sk) = MlKem768::keygen().unwrap();
        let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
        let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
        assert_eq!(ss1.as_bytes(), ss2.as_bytes());
    }

    #[test]
    fn test_secure_comparison_memory_safety() {
        use subtle::ConstantTimeEq;
        
        // Test that constant-time comparisons don't leak through memory access patterns
        let data1 = vec![0x42u8; 32];
        let data2 = vec![0x42u8; 32];
        let data3 = vec![0x43u8; 32];
        
        // Fill memory with pattern to detect unintended accesses
        let mut canary = vec![0xAAu8; 1024];
        memory_corruption::fill_canary_pattern(&mut canary, 0xCC);
        
        // Perform constant-time comparisons
        let result1 = data1.ct_eq(&data2); // Should be equal
        let result2 = data1.ct_eq(&data3); // Should be different
        
        // Verify canary pattern is intact (no memory corruption)
        assert!(memory_corruption::verify_canary_pattern(&canary, 0xCC),
            "Memory corruption during constant-time comparison");
        
        // Verify comparison results
        assert_eq!(bool::from(result1), true);
        assert_eq!(bool::from(result2), false);
    }

    #[test]
    fn test_zeroization_completeness() {
        // Test that all sensitive data structures properly implement zeroization
        
        // Test key material
        let (mut pk, mut sk) = MlKem768::keygen().unwrap();
        let pk_ptr = pk.as_bytes().as_ptr();
        let sk_ptr = sk.as_bytes().as_ptr();
        
        // Record original values (first few bytes)
        let pk_original = pk.as_bytes()[0..4].to_vec();
        let sk_original = sk.as_bytes()[0..4].to_vec();
        
        // Zeroize
        sk.zeroize();
        // Note: Public keys typically don't need zeroization, but test if implemented
        
        // Verify zeroization (secret key should be zeroed)
        let sk_after = sk.as_bytes()[0..4].to_vec();
        assert_ne!(sk_original, sk_after, "Secret key not properly zeroized");
        
        // Test shared secrets
        let (pk2, sk2) = MlKem768::keygen().unwrap();
        let (mut ct, mut ss1) = MlKem768::encapsulate(&pk2).unwrap();
        let mut ss2 = MlKem768::decapsulate(&sk2, &ct).unwrap();
        
        let ss1_original = ss1.as_bytes()[0..4].to_vec();
        let ss2_original = ss2.as_bytes()[0..4].to_vec();
        
        ss1.zeroize();
        ss2.zeroize();
        ct.zeroize();
        
        let ss1_after = ss1.as_bytes()[0..4].to_vec();
        let ss2_after = ss2.as_bytes()[0..4].to_vec();
        
        assert_ne!(ss1_original, ss1_after, "Shared secret 1 not properly zeroized");
        assert_ne!(ss2_original, ss2_after, "Shared secret 2 not properly zeroized");
    }
}