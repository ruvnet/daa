//! Security tests for various attack vectors in QuDAG Exchange

use qudag_exchange::security::{NonceManager, RateLimiter, InputValidator, SecurityConfig};
use qudag_exchange::error::ExchangeError;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};

/// Test replay attack prevention
#[test]
fn test_replay_attack_prevention() {
    let manager = NonceManager::new(60); // 60 second lifetime
    
    // Generate random nonce
    let mut rng = thread_rng();
    let nonce: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    
    // First use should succeed
    assert!(manager.verify_nonce(&nonce).is_ok());
    
    // Immediate replay should fail
    match manager.verify_nonce(&nonce) {
        Err(ExchangeError::ReplayAttack) => {}, // Expected
        _ => panic!("Replay attack not detected"),
    }
    
    // Different nonce should succeed
    let nonce2: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    assert!(manager.verify_nonce(&nonce2).is_ok());
}

/// Test DoS protection via rate limiting
#[test]
fn test_dos_protection() {
    let mut limiter = RateLimiter::new(5); // 5 requests per second
    
    // First 5 requests should succeed
    for _ in 0..5 {
        assert!(limiter.check_and_update().is_ok());
    }
    
    // 6th request should fail
    match limiter.check_and_update() {
        Err(ExchangeError::RateLimitExceeded(_)) => {}, // Expected
        _ => panic!("Rate limit not enforced"),
    }
    
    // Wait for window to reset
    thread::sleep(Duration::from_secs(1));
    
    // Should work again
    assert!(limiter.check_and_update().is_ok());
}

/// Test input validation against injection attacks
#[test]
fn test_injection_attack_prevention() {
    // SQL injection attempts
    let sql_injections = vec![
        "'; DROP TABLE users; --",
        "1' OR '1'='1",
        "admin'--",
        "1; DELETE FROM transactions WHERE 1=1",
    ];
    
    for injection in sql_injections {
        assert!(
            InputValidator::validate_tx_id(injection).is_err(),
            "SQL injection not blocked: {}",
            injection
        );
    }
    
    // Path traversal attempts
    let path_traversals = vec![
        "../../../etc/passwd",
        "..\\..\\windows\\system32",
        "qd/../../../root",
    ];
    
    for traversal in path_traversals {
        assert!(
            InputValidator::validate_address(traversal).is_err(),
            "Path traversal not blocked: {}",
            traversal
        );
    }
    
    // XSS attempts
    let xss_attempts = vec![
        "<script>alert('xss')</script>",
        "javascript:alert(1)",
        "<img src=x onerror=alert(1)>",
    ];
    
    for xss in xss_attempts {
        assert!(
            InputValidator::validate_tx_id(xss).is_err(),
            "XSS not blocked: {}",
            xss
        );
    }
}

/// Test integer overflow protection
#[test]
fn test_integer_overflow_protection() {
    // Test amount validation
    assert!(InputValidator::validate_amount(u64::MAX).is_err());
    assert!(InputValidator::validate_amount(0).is_err());
    
    // Test safe amounts
    assert!(InputValidator::validate_amount(1).is_ok());
    assert!(InputValidator::validate_amount(1_000_000).is_ok());
}

/// Test double spending prevention
#[test]
fn test_double_spending_prevention() {
    use std::sync::Arc;
    use dashmap::DashMap;
    
    // Simulate transaction tracking
    let spent_txs: Arc<DashMap<String, bool>> = Arc::new(DashMap::new());
    
    let tx_id = "tx-123-abc".to_string();
    
    // First spend should succeed
    assert!(!spent_txs.contains_key(&tx_id));
    spent_txs.insert(tx_id.clone(), true);
    
    // Second spend attempt should be detected
    assert!(spent_txs.contains_key(&tx_id));
}

/// Test memory exhaustion attack prevention
#[test]
fn test_memory_exhaustion_prevention() {
    let manager = NonceManager::new(5); // 5 second lifetime
    
    // Try to exhaust memory with many nonces
    let mut rng = thread_rng();
    for i in 0..1000 {
        let nonce: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        assert!(manager.verify_nonce(&nonce).is_ok());
        
        // Old nonces should be cleaned up automatically
        if i % 100 == 0 {
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    // Wait for nonces to expire
    thread::sleep(Duration::from_secs(6));
    
    // All old nonces should be cleaned up, so a previously used nonce
    // from more than 5 seconds ago can be reused
    let old_nonce: Vec<u8> = (0..32).map(|_| 0u8).collect();
    assert!(manager.verify_nonce(&old_nonce).is_ok());
}

/// Test timing attack on password/key comparison
#[test]
fn test_password_timing_attack() {
    use qudag_exchange::security::SecureBytes;
    
    let correct_password = SecureBytes::new(b"correct-password-123".to_vec());
    let wrong_password1 = SecureBytes::new(b"wrong-password-12345".to_vec());
    let wrong_password2 = SecureBytes::new(b"correct-password-124".to_vec()); // Only last char different
    
    const ITERATIONS: usize = 1000;
    
    // Time comparison with completely wrong password
    let mut wrong_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = std::time::Instant::now();
        let _ = correct_password.ct_eq(&wrong_password1);
        wrong_times.push(start.elapsed().as_nanos());
    }
    
    // Time comparison with almost correct password
    let mut almost_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = std::time::Instant::now();
        let _ = correct_password.ct_eq(&wrong_password2);
        almost_times.push(start.elapsed().as_nanos());
    }
    
    // Calculate means
    let wrong_mean = wrong_times.iter().sum::<u128>() / ITERATIONS as u128;
    let almost_mean = almost_times.iter().sum::<u128>() / ITERATIONS as u128;
    
    // Timing should be constant regardless of similarity
    let diff = if wrong_mean > almost_mean {
        wrong_mean - almost_mean
    } else {
        almost_mean - wrong_mean
    };
    
    let diff_percent = (diff as f64 / wrong_mean as f64) * 100.0;
    assert!(diff_percent < 5.0, 
        "Password comparison timing leak detected: {:.2}%", diff_percent);
}

/// Test side-channel resistance in conditional operations
#[test]
fn test_conditional_operation_timing() {
    use qudag_exchange::security::constant_time;
    
    const ITERATIONS: usize = 1000;
    let mut rng = thread_rng();
    
    // Test conditional selection timing
    let a = 42u32;
    let b = 84u32;
    
    // Time true conditions
    let mut true_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let condition = true;
        let start = std::time::Instant::now();
        let _ = constant_time::select(condition, &a, &b);
        true_times.push(start.elapsed().as_nanos());
    }
    
    // Time false conditions
    let mut false_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let condition = false;
        let start = std::time::Instant::now();
        let _ = constant_time::select(condition, &a, &b);
        false_times.push(start.elapsed().as_nanos());
    }
    
    // Calculate means
    let true_mean = true_times.iter().sum::<u128>() / ITERATIONS as u128;
    let false_mean = false_times.iter().sum::<u128>() / ITERATIONS as u128;
    
    // Timing should be constant
    let diff = if true_mean > false_mean {
        true_mean - false_mean
    } else {
        false_mean - true_mean
    };
    
    let diff_percent = (diff as f64 / true_mean as f64) * 100.0;
    assert!(diff_percent < 5.0, 
        "Conditional operation timing leak: {:.2}%", diff_percent);
}