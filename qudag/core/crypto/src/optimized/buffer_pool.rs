//! High-performance buffer pool for reduced memory allocations

use std::sync::Arc;
use std::collections::VecDeque;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Buffer pool for reusing memory allocations
pub struct BufferPool {
    /// Small buffers (up to 1KB)
    small_buffers: Mutex<VecDeque<Vec<u8>>>,
    /// Medium buffers (1KB - 16KB)
    medium_buffers: Mutex<VecDeque<Vec<u8>>>,
    /// Large buffers (16KB+)
    large_buffers: Mutex<VecDeque<Vec<u8>>>,
    /// Pool statistics
    stats: PoolStats,
}

#[derive(Default)]
struct PoolStats {
    small_hits: AtomicUsize,
    small_misses: AtomicUsize,
    medium_hits: AtomicUsize,
    medium_misses: AtomicUsize,
    large_hits: AtomicUsize,
    large_misses: AtomicUsize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new() -> Self {
        Self {
            small_buffers: Mutex::new(VecDeque::with_capacity(1000)),
            medium_buffers: Mutex::new(VecDeque::with_capacity(500)),
            large_buffers: Mutex::new(VecDeque::with_capacity(100)),
            stats: PoolStats::default(),
        }
    }

    /// Acquire a buffer of the specified size
    pub fn acquire(&self, size: usize) -> PooledBuffer {
        let buffer = match size {
            0..=1024 => self.acquire_small(size),
            1025..=16384 => self.acquire_medium(size),
            _ => self.acquire_large(size),
        };
        
        PooledBuffer {
            buffer,
            pool: self,
            original_size: size,
        }
    }

    fn acquire_small(&self, size: usize) -> Vec<u8> {
        let mut buffers = self.small_buffers.lock();
        if let Some(mut buffer) = buffers.pop_front() {
            self.stats.small_hits.fetch_add(1, Ordering::Relaxed);
            buffer.clear();
            buffer.resize(size, 0);
            buffer
        } else {
            self.stats.small_misses.fetch_add(1, Ordering::Relaxed);
            vec![0u8; size]
        }
    }

    fn acquire_medium(&self, size: usize) -> Vec<u8> {
        let mut buffers = self.medium_buffers.lock();
        if let Some(mut buffer) = buffers.pop_front() {
            self.stats.medium_hits.fetch_add(1, Ordering::Relaxed);
            buffer.clear();
            buffer.resize(size, 0);
            buffer
        } else {
            self.stats.medium_misses.fetch_add(1, Ordering::Relaxed);
            vec![0u8; size]
        }
    }

    fn acquire_large(&self, size: usize) -> Vec<u8> {
        let mut buffers = self.large_buffers.lock();
        if let Some(mut buffer) = buffers.pop_front() {
            self.stats.large_hits.fetch_add(1, Ordering::Relaxed);
            buffer.clear();
            buffer.resize(size, 0);
            buffer
        } else {
            self.stats.large_misses.fetch_add(1, Ordering::Relaxed);
            vec![0u8; size]
        }
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, mut buffer: Vec<u8>, original_size: usize) {
        // Securely clear the buffer
        buffer.fill(0);
        
        // Prevent over-accumulation of buffers
        match original_size {
            0..=1024 => {
                let mut buffers = self.small_buffers.lock();
                if buffers.len() < 1000 {
                    buffers.push_back(buffer);
                }
            }
            1025..=16384 => {
                let mut buffers = self.medium_buffers.lock();
                if buffers.len() < 500 {
                    buffers.push_back(buffer);
                }
            }
            _ => {
                let mut buffers = self.large_buffers.lock();
                if buffers.len() < 100 {
                    buffers.push_back(buffer);
                }
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStatistics {
        PoolStatistics {
            small_hit_rate: self.calculate_hit_rate(
                self.stats.small_hits.load(Ordering::Relaxed),
                self.stats.small_misses.load(Ordering::Relaxed),
            ),
            medium_hit_rate: self.calculate_hit_rate(
                self.stats.medium_hits.load(Ordering::Relaxed),
                self.stats.medium_misses.load(Ordering::Relaxed),
            ),
            large_hit_rate: self.calculate_hit_rate(
                self.stats.large_hits.load(Ordering::Relaxed),
                self.stats.large_misses.load(Ordering::Relaxed),
            ),
            total_buffers: self.small_buffers.lock().len() + 
                          self.medium_buffers.lock().len() + 
                          self.large_buffers.lock().len(),
        }
    }

    fn calculate_hit_rate(&self, hits: usize, misses: usize) -> f64 {
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

/// Statistics for buffer pool performance
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    pub small_hit_rate: f64,
    pub medium_hit_rate: f64,
    pub large_hit_rate: f64,
    pub total_buffers: usize,
}

/// RAII wrapper for pooled buffers
pub struct PooledBuffer<'a> {
    buffer: Vec<u8>,
    pool: &'a BufferPool,
    original_size: usize,
}

impl<'a> PooledBuffer<'a> {
    /// Get a mutable reference to the buffer
    pub fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    /// Get an immutable reference to the buffer
    pub fn as_ref(&self) -> &Vec<u8> {
        &self.buffer
    }

    /// Get the buffer as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Get the buffer as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Resize the buffer
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.buffer.resize(new_len, value);
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl<'a> std::ops::Deref for PooledBuffer<'a> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a> std::ops::DerefMut for PooledBuffer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl<'a> Drop for PooledBuffer<'a> {
    fn drop(&mut self) {
        // Return the buffer to the pool
        let buffer = std::mem::take(&mut self.buffer);
        self.pool.return_buffer(buffer, self.original_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_basic() {
        let pool = BufferPool::new();
        
        // Acquire a small buffer
        let mut buffer = pool.acquire(512);
        assert_eq!(buffer.len(), 512);
        
        // Modify the buffer
        buffer.as_mut_slice()[0] = 42;
        assert_eq!(buffer.as_slice()[0], 42);
        
        // Buffer should be returned to pool when dropped
        drop(buffer);
        
        // Acquire another buffer - should reuse the first one
        let buffer2 = pool.acquire(512);
        assert_eq!(buffer2.as_slice()[0], 0); // Should be cleared
    }

    #[test]
    fn test_buffer_pool_size_categories() {
        let pool = BufferPool::new();
        
        let small = pool.acquire(500);
        let medium = pool.acquire(5000);
        let large = pool.acquire(50000);
        
        assert_eq!(small.len(), 500);
        assert_eq!(medium.len(), 5000);
        assert_eq!(large.len(), 50000);
    }

    #[test]
    fn test_buffer_pool_stats() {
        let pool = BufferPool::new();
        
        // First acquisition should be a miss
        let _buffer1 = pool.acquire(1000);
        drop(_buffer1);
        
        // Second acquisition should be a hit
        let _buffer2 = pool.acquire(1000);
        
        let stats = pool.stats();
        assert!(stats.small_hit_rate > 0.0);
    }
}