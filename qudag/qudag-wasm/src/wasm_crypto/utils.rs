//! Utility functions for WASM crypto operations
//!
//! This module provides helper functions and utilities for crypto operations in WASM.

use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::*;

/// Buffer pool for reducing allocations in WASM
pub struct CryptoBufferPool {
    small_buffers: Vec<Vec<u8>>,  // 4KB buffers
    medium_buffers: Vec<Vec<u8>>, // 64KB buffers
    large_buffers: Vec<Vec<u8>>,  // 1MB buffers
}

impl CryptoBufferPool {
    /// Create a new buffer pool
    pub fn new() -> Self {
        Self {
            small_buffers: Vec::with_capacity(10),
            medium_buffers: Vec::with_capacity(5),
            large_buffers: Vec::with_capacity(2),
        }
    }

    /// Acquire a buffer of the requested size
    pub fn acquire(&mut self, size: usize) -> Vec<u8> {
        if size <= 4096 {
            self.small_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.resize(size, 0);
                    buf
                })
                .unwrap_or_else(|| vec![0u8; size])
        } else if size <= 65536 {
            self.medium_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.resize(size, 0);
                    buf
                })
                .unwrap_or_else(|| vec![0u8; size])
        } else {
            self.large_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.resize(size, 0);
                    buf
                })
                .unwrap_or_else(|| vec![0u8; size])
        }
    }

    /// Release a buffer back to the pool
    pub fn release(&mut self, mut buffer: Vec<u8>) {
        // Clear sensitive data
        for byte in buffer.iter_mut() {
            *byte = 0;
        }

        let capacity = buffer.capacity();
        buffer.clear();

        if capacity <= 4096 && self.small_buffers.len() < 10 {
            self.small_buffers.push(buffer);
        } else if capacity <= 65536 && self.medium_buffers.len() < 5 {
            self.medium_buffers.push(buffer);
        } else if capacity <= 1048576 && self.large_buffers.len() < 2 {
            self.large_buffers.push(buffer);
        }
        // Otherwise, let the buffer be dropped
    }
}

/// Timing-safe comparison
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

/// Secure random number generation for WASM
#[wasm_bindgen]
pub struct SecureRandom;

#[wasm_bindgen]
impl SecureRandom {
    /// Generate random bytes using the best available source
    #[wasm_bindgen(js_name = "getRandomBytes")]
    pub fn get_random_bytes(len: usize) -> Result<Vec<u8>, JsError> {
        let mut buffer = vec![0u8; len];

        // Try Web Crypto first
        if let Some(window) = web_sys::window() {
            if let Ok(crypto) = window.crypto() {
                let mut array = vec![0u8; len];
                crypto
                    .get_random_values_with_u8_array(&mut array)
                    .map_err(|_| JsError::new("Failed to get random values"))?;
                return Ok(array);
            }
        }

        // Fallback to getrandom
        getrandom::getrandom(&mut buffer)
            .map_err(|e| JsError::new(&format!("Random generation failed: {}", e)))?;

        Ok(buffer)
    }

    /// Generate a random u32
    #[wasm_bindgen(js_name = "randomU32")]
    pub fn random_u32() -> Result<u32, JsError> {
        let bytes = Self::get_random_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Generate a random u64
    #[wasm_bindgen(js_name = "randomU64")]
    pub fn random_u64() -> Result<u64, JsError> {
        let bytes = Self::get_random_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

/// Memory zeroization utilities
pub mod zeroize {
    use core::ptr;

    /// Securely zero memory
    pub fn zeroize_bytes(data: &mut [u8]) {
        for i in 0..data.len() {
            unsafe {
                ptr::write_volatile(&mut data[i], 0);
            }
        }

        // Memory fence to prevent reordering
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }

    /// Zeroizing wrapper for sensitive data
    pub struct ZeroizingBytes {
        data: Vec<u8>,
    }

    impl ZeroizingBytes {
        pub fn new(data: Vec<u8>) -> Self {
            Self { data }
        }

        pub fn as_slice(&self) -> &[u8] {
            &self.data
        }

        pub fn as_mut_slice(&mut self) -> &mut [u8] {
            &mut self.data
        }
    }

    impl Drop for ZeroizingBytes {
        fn drop(&mut self) {
            zeroize_bytes(&mut self.data);
        }
    }
}

/// Performance monitoring for crypto operations
#[wasm_bindgen]
pub struct CryptoPerformance;

#[wasm_bindgen]
impl CryptoPerformance {
    /// Measure operation timing
    #[wasm_bindgen(js_name = "measureTiming")]
    pub fn measure_timing(operation_name: &str) -> f64 {
        if let Some(window) = web_sys::window() {
            if let Some(performance) = window.performance() {
                let start = performance.now();
                // Operation would be performed here
                let end = performance.now();
                return end - start;
            }
        }
        0.0
    }

    /// Get high-resolution timestamp
    #[wasm_bindgen(js_name = "now")]
    pub fn now() -> f64 {
        if let Some(window) = web_sys::window() {
            if let Some(performance) = window.performance() {
                return performance.now();
            }
        }
        0.0
    }
}

/// Hex encoding/decoding utilities
pub mod hex {
    use anyhow::{anyhow, Result};

    /// Encode bytes to hex string
    pub fn encode(data: &[u8]) -> String {
        hex::encode(data)
    }

    /// Decode hex string to bytes
    pub fn decode(hex_str: &str) -> Result<Vec<u8>> {
        hex::decode(hex_str).map_err(|e| anyhow!("Hex decode error: {}", e))
    }
}

/// Base64 encoding/decoding utilities
pub mod base64 {
    use anyhow::{anyhow, Result};
    use base64::{engine::general_purpose, Engine as _};

    /// Encode bytes to base64 string
    pub fn encode(data: &[u8]) -> String {
        general_purpose::STANDARD.encode(data)
    }

    /// Encode bytes to URL-safe base64 string
    pub fn encode_url_safe(data: &[u8]) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(data)
    }

    /// Decode base64 string to bytes
    pub fn decode(b64_str: &str) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(b64_str)
            .map_err(|e| anyhow!("Base64 decode error: {}", e))
    }

    /// Decode URL-safe base64 string to bytes
    pub fn decode_url_safe(b64_str: &str) -> Result<Vec<u8>> {
        general_purpose::URL_SAFE_NO_PAD
            .decode(b64_str)
            .map_err(|e| anyhow!("Base64 decode error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool() {
        let mut pool = CryptoBufferPool::new();

        let buf1 = pool.acquire(1024);
        assert_eq!(buf1.len(), 1024);

        pool.release(buf1);

        let buf2 = pool.acquire(1024);
        assert_eq!(buf2.len(), 1024);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hello!"));
    }

    #[test]
    fn test_zeroize() {
        let mut data = vec![1, 2, 3, 4, 5];
        zeroize::zeroize_bytes(&mut data);
        assert_eq!(data, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_hex_encoding() {
        let data = b"Hello";
        let encoded = hex::encode(data);
        assert_eq!(encoded, "48656c6c6f");

        let decoded = hex::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"Hello, World!";
        let encoded = base64::encode(data);
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        let decoded = base64::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
