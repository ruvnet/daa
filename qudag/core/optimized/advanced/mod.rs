//! Advanced performance optimizations for QuDAG
//!
//! This module implements cutting-edge optimizations including:
//! - SIMD acceleration for cryptographic operations
//! - Zero-copy buffer management for networking
//! - GPU acceleration research and proof-of-concepts
//! - Memory-mapped file storage for large DAGs
//! - Kernel bypass networking options

pub mod simd_crypto;
pub mod zero_copy_buffers;
pub mod gpu_acceleration;
pub mod mmap_storage;
pub mod kernel_bypass;
pub mod benchmarks;

pub use simd_crypto::{SimdBlake3, SimdSignatureVerifier, SimdBatchOps};
pub use zero_copy_buffers::{ZeroCopyBuffer, ZeroCopyPool, NetworkBuffer};
pub use gpu_acceleration::{GpuAccelerator, GpuDagValidator, GpuCryptoEngine};
pub use mmap_storage::{MmapDagStorage, MmapConfig};
pub use kernel_bypass::{DpdkAdapter, IoUringTransport, XdpPacketProcessor};

/// Performance metrics for advanced optimizations
#[derive(Debug, Clone, Default)]
pub struct OptimizationMetrics {
    pub simd_speedup: f64,
    pub zero_copy_saves: u64,
    pub gpu_operations: u64,
    pub mmap_hits: u64,
    pub kernel_bypasses: u64,
}

impl OptimizationMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn report(&self) -> String {
        format!(
            "Optimization Metrics:\n\
             - SIMD Speedup: {:.2}x\n\
             - Zero-Copy Memory Saves: {} MB\n\
             - GPU Operations: {}\n\
             - Memory-Mapped Hits: {}\n\
             - Kernel Bypasses: {}",
            self.simd_speedup,
            self.zero_copy_saves / 1_048_576,
            self.gpu_operations,
            self.mmap_hits,
            self.kernel_bypasses
        )
    }
}