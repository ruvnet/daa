//! Resource Utilization Voucher (rUv) types and operations

use num_bigint::BigUint;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};
use zeroize::Zeroize;

use crate::error::{Error, Result};
use crate::MAX_RUV_SUPPLY;

/// Represents an amount of rUv with 8 decimal places precision
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct RuvAmount {
    /// Internal representation in smallest units (1 rUv = 100_000_000 units)
    units: BigUint,
}

impl RuvAmount {
    /// Number of decimal places
    pub const DECIMALS: u32 = 8;
    
    /// Multiplier for decimal conversion
    pub const DECIMAL_MULTIPLIER: u64 = 100_000_000;

    /// Create a new RuvAmount from whole rUv units
    pub fn from_ruv(amount: u64) -> Self {
        let units = BigUint::from(amount) * BigUint::from(Self::DECIMAL_MULTIPLIER);
        Self { units }
    }

    /// Create a new RuvAmount from smallest units
    pub fn from_units(units: BigUint) -> Result<Self> {
        if units > BigUint::from(MAX_RUV_SUPPLY) {
            return Err(Error::InvalidTransaction {
                reason: "Amount exceeds maximum supply".to_string(),
            });
        }
        Ok(Self { units })
    }

    /// Get the amount in whole rUv units
    pub fn as_ruv(&self) -> u64 {
        (self.units.clone() / BigUint::from(Self::DECIMAL_MULTIPLIER))
            .try_into()
            .unwrap_or(u64::MAX)
    }

    /// Get the amount in smallest units
    pub fn as_units(&self) -> &BigUint {
        &self.units
    }

    /// Check if amount is zero
    pub fn is_zero(&self) -> bool {
        self.units.is_zero()
    }

    /// Add two amounts, checking for overflow
    pub fn checked_add(&self, other: &Self) -> Result<Self> {
        let sum = &self.units + &other.units;
        Self::from_units(sum)
    }

    /// Subtract two amounts, checking for underflow
    pub fn checked_sub(&self, other: &Self) -> Result<Self> {
        if self.units < other.units {
            return Err(Error::InsufficientBalance {
                required: other.as_ruv() as u128,
                available: self.as_ruv() as u128,
            });
        }
        Ok(Self {
            units: &self.units - &other.units,
        })
    }
}

impl fmt::Display for RuvAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.units.clone() / BigUint::from(Self::DECIMAL_MULTIPLIER);
        let fraction = self.units.clone() % BigUint::from(Self::DECIMAL_MULTIPLIER);
        
        if fraction.is_zero() {
            write!(f, "{} rUv", whole)
        } else {
            write!(f, "{}.{:08} rUv", whole, fraction)
        }
    }
}

impl Default for RuvAmount {
    fn default() -> Self {
        Self {
            units: BigUint::zero(),
        }
    }
}

/// Main rUv type representing a resource utilization voucher
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ruv {
    /// Unique identifier
    pub id: String,
    
    /// Amount of rUv
    pub amount: RuvAmount,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

impl Ruv {
    /// Create a new rUv instance
    pub fn new(id: String, amount: RuvAmount) -> Self {
        Self {
            id,
            amount,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        }
    }

    /// Create rUv with metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruv_amount_creation() {
        let amount = RuvAmount::from_ruv(100);
        assert_eq!(amount.as_ruv(), 100);
        assert_eq!(amount.as_units(), &BigUint::from(10_000_000_000u64));
    }

    #[test]
    fn test_ruv_amount_arithmetic() {
        let a = RuvAmount::from_ruv(100);
        let b = RuvAmount::from_ruv(50);
        
        let sum = a.checked_add(&b).unwrap();
        assert_eq!(sum.as_ruv(), 150);
        
        let diff = a.checked_sub(&b).unwrap();
        assert_eq!(diff.as_ruv(), 50);
        
        // Test underflow
        assert!(b.checked_sub(&a).is_err());
    }

    #[test]
    fn test_ruv_display() {
        let amount = RuvAmount::from_ruv(100);
        assert_eq!(format!("{}", amount), "100.00000000 rUv");
    }
}