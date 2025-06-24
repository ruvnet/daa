use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Tracks memory usage patterns and churn metrics
pub struct MemoryTracker {
    peak_usage: AtomicUsize,
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
    start_time: Instant,
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            peak_usage: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn track_allocation(&self, _size: usize) {
        self.allocation_count.fetch_add(1, Ordering::SeqCst);
        let current = super::allocator::get_memory_usage();
        let mut peak = self.peak_usage.load(Ordering::SeqCst);
        while current > peak {
            match self.peak_usage.compare_exchange(
                peak,
                current,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
    }

    pub fn track_deallocation(&self, _size: usize) {
        self.deallocation_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_metrics(&self) -> MemoryMetrics {
        MemoryMetrics {
            current_usage: super::allocator::get_memory_usage(),
            peak_usage: self.peak_usage.load(Ordering::SeqCst),
            allocation_count: self.allocation_count.load(Ordering::SeqCst),
            deallocation_count: self.deallocation_count.load(Ordering::SeqCst),
            total_allocated: super::allocator::get_total_allocated(),
            total_deallocated: super::allocator::get_total_deallocated(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
}

/// Memory usage metrics
#[derive(Debug, Clone, Copy)]
pub struct MemoryMetrics {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub uptime_seconds: u64,
}

impl MemoryMetrics {
    pub fn allocation_rate(&self) -> f64 {
        self.allocation_count as f64 / self.uptime_seconds as f64
    }

    pub fn deallocation_rate(&self) -> f64 {
        self.deallocation_count as f64 / self.uptime_seconds as f64
    }

    pub fn churn_rate(&self) -> f64 {
        (self.total_allocated + self.total_deallocated) as f64 / self.uptime_seconds as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_memory_tracking() {
        let tracker = MemoryTracker::new();

        // Track some allocations
        tracker.track_allocation(1024);
        tracker.track_allocation(2048);

        // Track some deallocations
        tracker.track_deallocation(1024);

        // Sleep to get non-zero uptime
        thread::sleep(Duration::from_secs(1));

        let metrics = tracker.get_metrics();

        assert_eq!(metrics.allocation_count, 2);
        assert_eq!(metrics.deallocation_count, 1);
        assert!(metrics.uptime_seconds >= 1);
        assert!(metrics.allocation_rate() > 0.0);
        assert!(metrics.deallocation_rate() > 0.0);
        assert!(metrics.churn_rate() > 0.0);
    }
}
