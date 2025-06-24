use std::alloc::System;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom allocator that tracks memory allocations
pub struct TrackedAllocator {
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
    #[allow(dead_code)]
    inner: System,
}

impl Default for TrackedAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackedAllocator {
    pub const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            deallocated: AtomicUsize::new(0),
            inner: System,
        }
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated.load(Ordering::SeqCst)
    }

    pub fn deallocated_bytes(&self) -> usize {
        self.deallocated.load(Ordering::SeqCst)
    }

    pub fn current_usage(&self) -> usize {
        self.allocated_bytes()
            .saturating_sub(self.deallocated_bytes())
    }
}

// TODO: Unsafe allocator implementation disabled due to #![deny(unsafe_code)]
// This would require enabling unsafe code in specific modules
/*
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            self.allocated.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocated.fetch_add(layout.size(), Ordering::SeqCst);
        self.inner.dealloc(ptr, layout)
    }
}
*/

// #[global_allocator]
// static ALLOCATOR: TrackedAllocator = TrackedAllocator::new();

/// Get the current memory usage in bytes
pub fn get_memory_usage() -> usize {
    // TODO: Implement without global allocator
    0
}

/// Get the total number of bytes allocated
pub fn get_total_allocated() -> usize {
    // TODO: Implement without global allocator
    0
}

/// Get the total number of bytes deallocated
pub fn get_total_deallocated() -> usize {
    // TODO: Implement without global allocator
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_memory_tracking() {
        // Record initial values
        let start_allocated = get_total_allocated();
        let start_deallocated = get_total_deallocated();

        // Allocate some memory
        let data = vec![0u8; 1024];
        let allocated_size = mem::size_of_val(data.as_slice());

        // Check allocation was tracked
        assert!(get_total_allocated() > start_allocated);
        assert!(get_total_allocated() - start_allocated >= allocated_size);

        // Drop the allocation
        drop(data);

        // Check deallocation was tracked
        assert!(get_total_deallocated() > start_deallocated);
        assert!(get_total_deallocated() - start_deallocated >= allocated_size);
    }
}
