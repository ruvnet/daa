//! Utility functions for Prime core

use crate::error::{Error, Result};
use std::time::{Duration, Instant};

/// Timer for measuring execution time
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// Create a new timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
        }
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f32 {
        self.start.elapsed().as_secs_f32() * 1000.0
    }
    
    /// Get elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    
    /// Stop timer and return elapsed time
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        tracing::debug!("{} took {:?}", self.name, elapsed);
        elapsed
    }
}

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Format number with thousands separators
pub fn format_number(n: i64) -> String {
    let s = n.abs().to_string();
    let chunks: Vec<String> = s.chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| chunk.iter().rev().collect::<String>())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    
    let formatted = chunks.join(",");
    if n < 0 {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

/// Calculate hash of bytes
pub fn hash_bytes(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Exponential backoff for retries
pub struct ExponentialBackoff {
    current: Duration,
    max: Duration,
    factor: f32,
}

impl ExponentialBackoff {
    /// Create new backoff
    pub fn new(initial: Duration, max: Duration, factor: f32) -> Self {
        Self {
            current: initial,
            max,
            factor,
        }
    }
    
    /// Get next backoff duration
    pub fn next(&mut self) -> Duration {
        let current = self.current;
        let next = Duration::from_secs_f32(current.as_secs_f32() * self.factor);
        self.current = next.min(self.max);
        current
    }
    
    /// Reset backoff
    pub fn reset(&mut self) {
        self.current = Duration::from_secs(1);
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            2.0,
        )
    }
}

/// Retry an async operation with backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_retries: u32,
    mut backoff: ExponentialBackoff,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                
                if attempt < max_retries {
                    let delay = backoff.next();
                    tracing::warn!(
                        "Operation failed (attempt {}/{}), retrying in {:?}",
                        attempt + 1,
                        max_retries,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| Error::Other("Max retries exceeded".to_string())))
}

/// Progress tracker for long-running operations
pub struct ProgressTracker {
    total: u64,
    current: u64,
    start_time: Instant,
    last_update: Instant,
    update_interval: Duration,
}

impl ProgressTracker {
    /// Create new progress tracker
    pub fn new(total: u64) -> Self {
        let now = Instant::now();
        Self {
            total,
            current: 0,
            start_time: now,
            last_update: now,
            update_interval: Duration::from_secs(1),
        }
    }
    
    /// Update progress
    pub fn update(&mut self, current: u64) {
        self.current = current;
        
        if self.last_update.elapsed() >= self.update_interval {
            self.print_progress();
            self.last_update = Instant::now();
        }
    }
    
    /// Increment progress
    pub fn increment(&mut self) {
        self.update(self.current + 1);
    }
    
    /// Get progress percentage
    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f32 / self.total as f32) * 100.0
        }
    }
    
    /// Get estimated time remaining
    pub fn eta(&self) -> Option<Duration> {
        if self.current == 0 {
            return None;
        }
        
        let elapsed = self.start_time.elapsed();
        let rate = self.current as f32 / elapsed.as_secs_f32();
        let remaining = self.total - self.current;
        
        Some(Duration::from_secs_f32(remaining as f32 / rate))
    }
    
    /// Print progress to log
    fn print_progress(&self) {
        let pct = self.percentage();
        let eta = self.eta()
            .map(|d| format!(" ETA: {:?}", d))
            .unwrap_or_default();
        
        tracing::info!(
            "Progress: {}/{} ({:.1}%){}",
            format_number(self.current as i64),
            format_number(self.total as i64),
            pct,
            eta
        );
    }
}

/// Memory pool for tensor allocation
pub struct TensorMemoryPool {
    pools: std::collections::HashMap<(Vec<i64>, tch::Kind), Vec<tch::Tensor>>,
    device: tch::Device,
}

impl TensorMemoryPool {
    /// Create new memory pool
    pub fn new(device: tch::Device) -> Self {
        Self {
            pools: Default::default(),
            device,
        }
    }
    
    /// Allocate tensor from pool
    pub fn allocate(&mut self, shape: &[i64], kind: tch::Kind) -> tch::Tensor {
        let key = (shape.to_vec(), kind);
        
        if let Some(pool) = self.pools.get_mut(&key) {
            if let Some(tensor) = pool.pop() {
                return tensor;
            }
        }
        
        // Allocate new tensor
        tch::Tensor::zeros(shape, (kind, self.device))
    }
    
    /// Return tensor to pool
    pub fn deallocate(&mut self, tensor: tch::Tensor) {
        let shape = tensor.size();
        let kind = tensor.kind();
        let key = (shape, kind);
        
        self.pools.entry(key).or_insert_with(Vec::new).push(tensor);
    }
    
    /// Clear all pools
    pub fn clear(&mut self) {
        self.pools.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }
    
    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(-1234567), "-1,234,567");
        assert_eq!(format_number(1234567890), "1,234,567,890");
    }
    
    #[test]
    fn test_exponential_backoff() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_secs(1),
            Duration::from_secs(10),
            2.0,
        );
        
        assert_eq!(backoff.next(), Duration::from_secs(1));
        assert_eq!(backoff.next(), Duration::from_secs(2));
        assert_eq!(backoff.next(), Duration::from_secs(4));
        assert_eq!(backoff.next(), Duration::from_secs(8));
        assert_eq!(backoff.next(), Duration::from_secs(10)); // Capped at max
        assert_eq!(backoff.next(), Duration::from_secs(10));
    }
    
    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new(100);
        
        tracker.update(25);
        assert_eq!(tracker.percentage(), 25.0);
        
        tracker.update(50);
        assert_eq!(tracker.percentage(), 50.0);
        
        tracker.update(100);
        assert_eq!(tracker.percentage(), 100.0);
    }
}