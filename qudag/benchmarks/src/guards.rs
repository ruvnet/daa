//! Benchmark guards and safety mechanisms
//!
//! This module provides safety guards and protective mechanisms for benchmarks,
//! including resource limits, timeouts, and error recovery.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Benchmark guard errors
#[derive(Error, Debug)]
pub enum GuardError {
    /// Timeout exceeded
    #[error("Benchmark timeout exceeded: {duration:?}")]
    TimeoutExceeded { duration: Duration },

    /// Memory limit exceeded
    #[error("Memory limit exceeded: {current} bytes > {limit} bytes")]
    MemoryLimitExceeded { current: u64, limit: u64 },

    /// CPU limit exceeded
    #[error("CPU usage limit exceeded: {current}% > {limit}%")]
    CpuLimitExceeded { current: f64, limit: f64 },

    /// Operation count limit exceeded
    #[error("Operation count limit exceeded: {current} > {limit}")]
    OperationLimitExceeded { current: u64, limit: u64 },
}

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum execution time
    pub max_duration: Option<Duration>,
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
    /// Maximum CPU usage percentage
    pub max_cpu: Option<f64>,
    /// Maximum number of operations
    pub max_operations: Option<u64>,
}

/// Benchmark execution guard
pub struct BenchmarkGuard {
    limits: ResourceLimits,
    start_time: Instant,
    operation_count: AtomicU64,
    aborted: AtomicBool,
}

/// Shared guard handle for multi-threaded benchmarks
pub type GuardHandle = Arc<BenchmarkGuard>;

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_duration: Some(Duration::from_secs(300)), // 5 minutes default
            max_memory: Some(1024 * 1024 * 1024),         // 1GB default
            max_cpu: Some(95.0),                          // 95% CPU default
            max_operations: Some(1_000_000),              // 1M operations default
        }
    }
}

impl ResourceLimits {
    /// Create unlimited resource configuration (use with caution)
    pub fn unlimited() -> Self {
        Self {
            max_duration: None,
            max_memory: None,
            max_cpu: None,
            max_operations: None,
        }
    }

    /// Create tight limits for testing
    pub fn tight() -> Self {
        Self {
            max_duration: Some(Duration::from_secs(30)),
            max_memory: Some(100 * 1024 * 1024), // 100MB
            max_cpu: Some(80.0),                 // 80%
            max_operations: Some(10_000),
        }
    }
}

impl BenchmarkGuard {
    /// Create a new benchmark guard with default limits
    pub fn new() -> Self {
        Self::with_limits(ResourceLimits::default())
    }

    /// Create a new benchmark guard with custom limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self {
            limits,
            start_time: Instant::now(),
            operation_count: AtomicU64::new(0),
            aborted: AtomicBool::new(false),
        }
    }

    /// Create a shared handle for multi-threaded access
    pub fn handle(self) -> GuardHandle {
        Arc::new(self)
    }

    /// Check if the benchmark should continue running
    pub fn should_continue(&self) -> Result<(), GuardError> {
        if self.aborted.load(Ordering::Relaxed) {
            return Err(GuardError::TimeoutExceeded {
                duration: self.start_time.elapsed(),
            });
        }

        // Check timeout
        if let Some(max_duration) = self.limits.max_duration {
            let elapsed = self.start_time.elapsed();
            if elapsed > max_duration {
                self.aborted.store(true, Ordering::Relaxed);
                return Err(GuardError::TimeoutExceeded { duration: elapsed });
            }
        }

        // Check operation count
        if let Some(max_ops) = self.limits.max_operations {
            let current_ops = self.operation_count.load(Ordering::Relaxed);
            if current_ops > max_ops {
                return Err(GuardError::OperationLimitExceeded {
                    current: current_ops,
                    limit: max_ops,
                });
            }
        }

        Ok(())
    }

    /// Record an operation
    pub fn record_operation(&self) -> Result<(), GuardError> {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.should_continue()
    }

    /// Check memory usage against limits
    pub fn check_memory(&self, current_memory: u64) -> Result<(), GuardError> {
        if let Some(max_memory) = self.limits.max_memory {
            if current_memory > max_memory {
                return Err(GuardError::MemoryLimitExceeded {
                    current: current_memory,
                    limit: max_memory,
                });
            }
        }
        Ok(())
    }

    /// Check CPU usage against limits
    pub fn check_cpu(&self, current_cpu: f64) -> Result<(), GuardError> {
        if let Some(max_cpu) = self.limits.max_cpu {
            if current_cpu > max_cpu {
                return Err(GuardError::CpuLimitExceeded {
                    current: current_cpu,
                    limit: max_cpu,
                });
            }
        }
        Ok(())
    }

    /// Abort the benchmark
    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

    /// Check if the benchmark was aborted
    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

    /// Get elapsed time since guard creation
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operation_count.load(Ordering::Relaxed)
    }

    /// Get resource limits
    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }
}

/// Convenience macro for checking guard status in benchmark loops
#[macro_export]
macro_rules! guard_check {
    ($guard:expr) => {
        $guard.should_continue()?
    };
}

/// Timeout guard for individual operations
pub struct TimeoutGuard {
    deadline: Instant,
}

impl TimeoutGuard {
    /// Create a new timeout guard
    pub fn new(timeout: Duration) -> Self {
        Self {
            deadline: Instant::now() + timeout,
        }
    }

    /// Check if the timeout has been exceeded
    pub fn check(&self) -> Result<(), GuardError> {
        let now = Instant::now();
        if now > self.deadline {
            let elapsed = now.duration_since(
                self.deadline - (self.deadline - (self.deadline - Duration::from_secs(0))),
            );
            Err(GuardError::TimeoutExceeded { duration: elapsed })
        } else {
            Ok(())
        }
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        let now = Instant::now();
        if now < self.deadline {
            self.deadline - now
        } else {
            Duration::from_secs(0)
        }
    }
}

/// Circuit breaker for error-prone operations
pub struct CircuitBreaker {
    failure_threshold: u32,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    state: AtomicBool, // true = open (failing), false = closed (working)
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32) -> Self {
        Self {
            failure_threshold,
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            state: AtomicBool::new(false),
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        // Reset failure count on success
        self.failure_count.store(0, Ordering::Relaxed);
        self.state.store(false, Ordering::Relaxed);
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= self.failure_threshold as u64 {
            self.state.store(true, Ordering::Relaxed);
        }
    }

    /// Check if operations are allowed
    pub fn is_open(&self) -> bool {
        self.state.load(Ordering::Relaxed)
    }

    /// Get failure count
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get success count
    pub fn success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }
}
