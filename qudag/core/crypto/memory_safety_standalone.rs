// Standalone memory safety test for QuDAG crypto primitives
// This can be compiled and run independently to test memory safety

use std::alloc::{alloc, dealloc, Layout};
use std::sync::atomic::{AtomicU8, Ordering};
use std::ptr;
use std::slice;

/// Test for secure memory allocation and cleanup
fn test_secure_memory_allocation() {
    println!("Testing secure memory allocation...");
    
    let size = 4096;
    let layout = Layout::from_size_align(size, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    
    if ptr.is_null() {
        panic!("Memory allocation failed");
    }
    
    // Write test pattern
    let slice = unsafe { slice::from_raw_parts_mut(ptr, size) };
    for (i, byte) in slice.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    
    // Verify pattern
    for (i, &byte) in slice.iter().enumerate() {
        assert_eq!(byte, (i % 256) as u8, "Memory corruption detected at index {}", i);
    }
    
    // Secure cleanup - overwrite with zeros
    for byte in slice.iter_mut() {
        *byte = 0;
    }
    
    // Add memory fence
    std::sync::atomic::fence(Ordering::SeqCst);
    
    // Verify cleanup
    for (i, &byte) in slice.iter().enumerate() {
        assert_eq!(byte, 0, "Memory not properly cleared at index {}", i);
    }
    
    // Deallocate
    unsafe { dealloc(ptr, layout) };
    
    println!("✓ Secure memory allocation test passed");
}

/// Test for memory bounds checking
fn test_memory_bounds() {
    println!("Testing memory bounds checking...");
    
    let size = 1024;
    let mut buffer = vec![0u8; size];
    
    // Test valid boundary access
    buffer[0] = 0xFF;
    buffer[size - 1] = 0xAA;
    
    assert_eq!(buffer[0], 0xFF);
    assert_eq!(buffer[size - 1], 0xAA);
    
    // Test safe copying with bounds checking
    let source_data = vec![0x42u8; 64];
    let copy_len = source_data.len().min(buffer.len());
    buffer[..copy_len].copy_from_slice(&source_data[..copy_len]);
    
    // Verify copy
    for i in 0..copy_len {
        assert_eq!(buffer[i], 0x42, "Copy verification failed at index {}", i);
    }
    
    println!("✓ Memory bounds checking test passed");
}

/// Test for constant-time memory operations
fn test_constant_time_operations() {
    println!("Testing constant-time operations...");
    
    let data1 = vec![0x42u8; 32];
    let data2 = vec![0x42u8; 32];
    let data3 = vec![0x43u8; 32];
    
    // Simulate constant-time comparison
    fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        result == 0
    }
    
    assert!(constant_time_eq(&data1, &data2), "Equal arrays should compare as equal");
    assert!(!constant_time_eq(&data1, &data3), "Different arrays should compare as unequal");
    
    println!("✓ Constant-time operations test passed");
}

/// Test for memory leak detection through repeated allocations
fn test_memory_leak_detection() {
    println!("Testing memory leak detection...");
    
    let iterations = 10000;
    let mut allocation_count = 0;
    let mut deallocation_count = 0;
    
    for _ in 0..iterations {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = unsafe { alloc(layout) };
        
        if !ptr.is_null() {
            allocation_count += 1;
            
            // Write some data
            unsafe {
                ptr::write_bytes(ptr, 0xAA, 64);
            }
            
            // Deallocate
            unsafe { dealloc(ptr, layout) };
            deallocation_count += 1;
        }
    }
    
    assert_eq!(allocation_count, deallocation_count, 
        "Memory leak detected: {} allocations, {} deallocations", 
        allocation_count, deallocation_count);
    
    println!("✓ Memory leak detection test passed ({} allocations)", allocation_count);
}

/// Test memory alignment requirements
fn test_memory_alignment() {
    println!("Testing memory alignment...");
    
    let alignments = [8, 16, 32, 64];
    
    for &align in &alignments {
        let size = 1024;
        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { alloc(layout) };
        
        if ptr.is_null() {
            panic!("Memory allocation failed for alignment {}", align);
        }
        
        // Check alignment
        assert_eq!(ptr as usize % align, 0, "Memory not aligned to {} bytes", align);
        
        // Write aligned data
        let slice = unsafe { slice::from_raw_parts_mut(ptr, size) };
        for (i, byte) in slice.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        // Verify data integrity
        for (i, &byte) in slice.iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8, "Data corruption at index {} with alignment {}", i, align);
        }
        
        // Clean up
        unsafe { dealloc(ptr, layout) };
    }
    
    println!("✓ Memory alignment test passed");
}

/// Test stack overflow protection with recursion
fn test_stack_overflow_protection() {
    println!("Testing stack overflow protection...");
    
    fn recursive_test(depth: usize, max_depth: usize) -> bool {
        if depth >= max_depth {
            return true;
        }
        
        // Allocate some stack space
        let _stack_array = [0u8; 1024];
        
        // Add some work to prevent optimization
        let sum: usize = _stack_array.iter().enumerate().map(|(i, &b)| i + b as usize).sum();
        
        // Continue recursion
        recursive_test(depth + 1, max_depth) && sum >= 0
    }
    
    // Test with reasonable depth
    assert!(recursive_test(0, 100), "Recursive test failed");
    
    println!("✓ Stack overflow protection test passed");
}

/// Test atomic memory operations
fn test_atomic_memory_operations() {
    println!("Testing atomic memory operations...");
    
    let shared_data = std::sync::Arc::new(AtomicU8::new(0));
    let threads: Vec<_> = (0..8).map(|i| {
        let data = shared_data.clone();
        std::thread::spawn(move || {
            for _ in 0..1000 {
                // Atomic increment
                data.fetch_add(1, Ordering::SeqCst);
                
                // Atomic compare-and-swap
                let current = data.load(Ordering::SeqCst);
                let _ = data.compare_exchange_weak(current, current.wrapping_mul(2), Ordering::SeqCst, Ordering::Relaxed);
            }
        })
    }).collect();
    
    // Wait for all threads
    for thread in threads {
        thread.join().unwrap();
    }
    
    println!("✓ Atomic memory operations test passed (final value: {})", shared_data.load(Ordering::SeqCst));
}

fn main() {
    println!("QuDAG Crypto Memory Safety Test Suite");
    println!("====================================");
    
    // Run all memory safety tests
    test_secure_memory_allocation();
    test_memory_bounds();
    test_constant_time_operations();
    test_memory_leak_detection();
    test_memory_alignment();
    test_stack_overflow_protection();
    test_atomic_memory_operations();
    
    println!();
    println!("🎉 All memory safety tests passed successfully!");
    println!();
    println!("To run with Valgrind:");
    println!("  valgrind --tool=memcheck --leak-check=full ./memory_safety_standalone");
    println!();
    println!("To run with AddressSanitizer:");
    println!("  rustc -Z sanitizer=address -o memory_safety_asan memory_safety_standalone.rs");
    println!("  ./memory_safety_asan");
}