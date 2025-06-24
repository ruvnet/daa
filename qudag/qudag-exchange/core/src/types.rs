//! Core types for QuDAG Exchange
//!
//! Defines fundamental types used throughout the system

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

/// rUv (Resource Utilization Voucher) - the fundamental unit of resource credits
///
/// Each rUv represents a unit of computational/storage capability within the system.
/// Users spend rUv to perform operations like transactions, data storage, or computations.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct rUv(pub u64);

impl rUv {
    /// Zero rUv tokens
    pub const ZERO: Self = Self(0);

    /// One rUv token
    pub const ONE: Self = Self(1);

    /// Create a new rUv amount
    pub const fn new(amount: u64) -> Self {
        Self(amount)
    }

    /// Get the raw amount
    pub const fn amount(&self) -> u64 {
        self.0
    }

    /// Check if zero
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Checked addition
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Checked subtraction
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Checked multiplication
    pub fn checked_mul(self, scalar: u64) -> Option<Self> {
        self.0.checked_mul(scalar).map(Self)
    }

    /// Saturating addition (caps at u64::MAX)
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction (floors at 0)
    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Add another rUv amount (returns Error on overflow)
    pub fn add(self, other: Self) -> Result<Self, &'static str> {
        self.checked_add(other).ok_or("rUv addition overflow")
    }

    /// Multiply by a percentage (0.0 to 1.0)
    pub fn multiply(self, percentage: f64) -> Result<Self, &'static str> {
        if percentage < 0.0 || percentage > 1.0 {
            return Err("Percentage must be between 0.0 and 1.0");
        }

        let result = (self.0 as f64 * percentage).round() as u64;
        if result > u64::MAX {
            return Err("rUv multiplication overflow");
        }

        Ok(Self(result))
    }
}

impl From<u64> for rUv {
    fn from(amount: u64) -> Self {
        Self(amount)
    }
}

impl From<rUv> for u64 {
    fn from(ruv: rUv) -> Self {
        ruv.0
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for rUv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} rUv", self.0)
    }
}

/// Timestamp type (milliseconds since epoch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Create a new timestamp
    pub const fn new(millis: u64) -> Self {
        Self(millis)
    }

    /// Get milliseconds value
    pub const fn millis(&self) -> u64 {
        self.0
    }

    /// Get the timestamp value (alias for millis)
    pub const fn value(&self) -> u64 {
        self.0
    }

    #[cfg(feature = "std")]
    /// Get current timestamp
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Self(duration.as_millis() as u64)
    }
}

/// Nonce type for transaction ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

impl Nonce {
    /// Zero nonce
    pub const ZERO: Self = Self(0);

    /// Create a new nonce
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get the nonce value
    pub const fn value(&self) -> u64 {
        self.0
    }

    /// Increment the nonce
    pub fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    /// Get next nonce
    pub fn next(&self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Hash type for content addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from slice (returns None if wrong length)
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(slice);
            Some(Self(bytes))
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruv_arithmetic() {
        let a = rUv::new(100);
        let b = rUv::new(50);

        assert_eq!(a.checked_add(b), Some(rUv::new(150)));
        assert_eq!(a.checked_sub(b), Some(rUv::new(50)));
        assert_eq!(b.checked_sub(a), None);

        assert_eq!(a.saturating_add(b), rUv::new(150));
        assert_eq!(b.saturating_sub(a), rUv::ZERO);
    }

    #[test]
    fn test_nonce_operations() {
        let mut nonce = Nonce::ZERO;
        assert_eq!(nonce.value(), 0);

        nonce.increment();
        assert_eq!(nonce.value(), 1);

        let next = nonce.next();
        assert_eq!(next.value(), 2);
        assert_eq!(nonce.value(), 1); // Original unchanged
    }

    #[test]
    fn test_hash_conversion() {
        let bytes = [1u8; 32];
        let hash = Hash::from_bytes(bytes);
        assert_eq!(hash.as_bytes(), &bytes);

        let slice: &[u8] = &bytes;
        let hash2 = Hash::from_slice(slice).unwrap();
        assert_eq!(hash, hash2);

        // Wrong size
        assert!(Hash::from_slice(&[1u8; 31]).is_none());
    }
}
