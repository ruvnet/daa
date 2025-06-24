//! Security utilities and validators for QuDAG Exchange

use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::error::{ExchangeError, Result};

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum allowed timing variance in microseconds
    pub max_timing_variance_us: u64,
    /// Minimum time between operations to prevent timing attacks
    pub min_operation_interval_ms: u64,
    /// Maximum transaction rate per second
    pub max_tx_rate_per_sec: u32,
    /// Enable timing attack detection
    pub timing_attack_detection: bool,
    /// Enable replay attack prevention
    pub replay_prevention: bool,
    /// Nonce lifetime in seconds
    pub nonce_lifetime_sec: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_timing_variance_us: 1000, // 1ms
            min_operation_interval_ms: 10,
            max_tx_rate_per_sec: 100,
            timing_attack_detection: true,
            replay_prevention: true,
            nonce_lifetime_sec: 300, // 5 minutes
        }
    }
}

/// Secure container for sensitive data
#[derive(Debug, ZeroizeOnDrop)]
pub struct SecureBytes {
    #[zeroize(drop)]
    data: Vec<u8>,
}

impl SecureBytes {
    /// Create new secure bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    /// Get a reference to the data
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    /// Get mutable reference (use with caution)
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
    
    /// Constant-time comparison
    pub fn ct_eq(&self, other: &Self) -> bool {
        self.data.ct_eq(&other.data).into()
    }
}

/// Timing attack detector
#[derive(Debug)]
pub struct TimingGuard {
    start: Instant,
    expected_duration: Duration,
    tolerance: Duration,
}

impl TimingGuard {
    /// Create a new timing guard
    pub fn new(expected_duration: Duration, tolerance: Duration) -> Self {
        Self {
            start: Instant::now(),
            expected_duration,
            tolerance,
        }
    }
    
    /// Check if operation completed within expected time
    pub fn check(&self) -> Result<()> {
        let elapsed = self.start.elapsed();
        let min_duration = self.expected_duration.saturating_sub(self.tolerance);
        let max_duration = self.expected_duration + self.tolerance;
        
        if elapsed < min_duration || elapsed > max_duration {
            return Err(ExchangeError::TimingAnomaly);
        }
        
        Ok(())
    }
}

/// Rate limiter for preventing DoS attacks
#[derive(Debug)]
pub struct RateLimiter {
    max_per_second: u32,
    window_start: Instant,
    count: u32,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_per_second: u32) -> Self {
        Self {
            max_per_second,
            window_start: Instant::now(),
            count: 0,
        }
    }
    
    /// Check if operation is allowed
    pub fn check_and_update(&mut self) -> Result<()> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start);
        
        // Reset window if more than 1 second has passed
        if elapsed >= Duration::from_secs(1) {
            self.window_start = now;
            self.count = 1;
            return Ok(());
        }
        
        // Check rate limit
        if self.count >= self.max_per_second {
            return Err(ExchangeError::RateLimitExceeded(
                format!("Maximum {} operations per second exceeded", self.max_per_second)
            ));
        }
        
        self.count += 1;
        Ok(())
    }
}

/// Nonce manager for replay prevention
#[derive(Debug)]
pub struct NonceManager {
    used_nonces: dashmap::DashMap<Vec<u8>, Instant>,
    lifetime: Duration,
}

impl NonceManager {
    /// Create a new nonce manager
    pub fn new(lifetime_sec: u64) -> Self {
        Self {
            used_nonces: dashmap::DashMap::new(),
            lifetime: Duration::from_secs(lifetime_sec),
        }
    }
    
    /// Check if nonce is valid (not used before)
    pub fn verify_nonce(&self, nonce: &[u8]) -> Result<()> {
        let now = Instant::now();
        
        // Clean up old nonces
        self.used_nonces.retain(|_, timestamp| {
            now.duration_since(*timestamp) < self.lifetime
        });
        
        // Check if nonce was already used
        if self.used_nonces.contains_key(nonce) {
            return Err(ExchangeError::ReplayAttack);
        }
        
        // Mark nonce as used
        self.used_nonces.insert(nonce.to_vec(), now);
        Ok(())
    }
}

/// Input validator to prevent injection attacks
pub struct InputValidator;

impl InputValidator {
    /// Validate transaction ID format
    pub fn validate_tx_id(id: &str) -> Result<()> {
        if id.is_empty() || id.len() > 64 {
            return Err(ExchangeError::TransactionValidation(
                "Invalid transaction ID length".to_string()
            ));
        }
        
        // Only allow alphanumeric and hyphen
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ExchangeError::TransactionValidation(
                "Invalid characters in transaction ID".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate address format
    pub fn validate_address(address: &str) -> Result<()> {
        if address.is_empty() || address.len() > 128 {
            return Err(ExchangeError::TransactionValidation(
                "Invalid address length".to_string()
            ));
        }
        
        // Basic format validation (can be expanded based on actual format)
        if !address.starts_with("qd") {
            return Err(ExchangeError::TransactionValidation(
                "Invalid address format".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate amount
    pub fn validate_amount(amount: u64) -> Result<()> {
        const MAX_AMOUNT: u64 = 1_000_000_000_000; // 1 trillion
        
        if amount == 0 {
            return Err(ExchangeError::TransactionValidation(
                "Amount must be greater than zero".to_string()
            ));
        }
        
        if amount > MAX_AMOUNT {
            return Err(ExchangeError::TransactionValidation(
                "Amount exceeds maximum allowed".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Constant-time operations utilities
pub mod constant_time {
    use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
    
    /// Constant-time byte array comparison
    pub fn compare_bytes(a: &[u8], b: &[u8]) -> bool {
        a.ct_eq(b).into()
    }
    
    /// Constant-time selection between two values
    pub fn select<T: ConditionallySelectable>(condition: bool, a: &T, b: &T) -> T {
        T::conditional_select(b, a, Choice::from(condition as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_bytes_zeroization() {
        let data = vec![1, 2, 3, 4, 5];
        let secure = SecureBytes::new(data);
        assert_eq!(secure.as_slice(), &[1, 2, 3, 4, 5]);
        // After drop, data should be zeroized
    }
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);
        
        // First two should succeed
        assert!(limiter.check_and_update().is_ok());
        assert!(limiter.check_and_update().is_ok());
        
        // Third should fail
        assert!(limiter.check_and_update().is_err());
    }
    
    #[test]
    fn test_nonce_manager() {
        let manager = NonceManager::new(60);
        let nonce = b"test-nonce-123";
        
        // First use should succeed
        assert!(manager.verify_nonce(nonce).is_ok());
        
        // Reuse should fail
        assert!(matches!(
            manager.verify_nonce(nonce),
            Err(ExchangeError::ReplayAttack)
        ));
    }
    
    #[test]
    fn test_input_validation() {
        // Valid inputs
        assert!(InputValidator::validate_tx_id("tx-123-abc").is_ok());
        assert!(InputValidator::validate_address("qd123abc").is_ok());
        assert!(InputValidator::validate_amount(1000).is_ok());
        
        // Invalid inputs
        assert!(InputValidator::validate_tx_id("").is_err());
        assert!(InputValidator::validate_tx_id("tx_123").is_err()); // underscore not allowed
        assert!(InputValidator::validate_address("invalid").is_err());
        assert!(InputValidator::validate_amount(0).is_err());
    }
}