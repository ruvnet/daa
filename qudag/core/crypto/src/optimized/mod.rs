//! Optimized cryptographic implementations for QuDAG
//! 
//! This module provides high-performance implementations of cryptographic
//! primitives with focus on:
//! - Reduced memory allocations
//! - Improved cache efficiency  
//! - SIMD optimizations where applicable
//! - Constant-time security properties

pub mod buffer_pool;
pub mod ml_kem_optimized;
pub mod cache;
pub mod simd_utils;

use std::sync::Arc;
use once_cell::sync::Lazy;

/// Global buffer pool for crypto operations
pub static CRYPTO_BUFFER_POOL: Lazy<Arc<buffer_pool::BufferPool>> = 
    Lazy::new(|| Arc::new(buffer_pool::BufferPool::new()));

/// Global key cache for frequently used keys
pub static KEY_CACHE: Lazy<Arc<cache::KeyCache>> = 
    Lazy::new(|| Arc::new(cache::KeyCache::new(10000)));

pub use buffer_pool::BufferPool;
pub use ml_kem_optimized::OptimizedMlKem768;
pub use cache::{KeyCache, CachedKey};