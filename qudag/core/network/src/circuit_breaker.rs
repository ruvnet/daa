#![deny(unsafe_code)]

use crate::types::PeerId;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tokio::time::interval;
use tracing::{info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - requests allowed
    Closed,
    /// Circuit is open - requests blocked
    Open,
    /// Circuit is half-open - testing recovery
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Timeout before attempting recovery
    pub timeout: Duration,
    /// Failure rate threshold (0.0 to 1.0)
    pub failure_rate_threshold: f64,
    /// Minimum number of requests for statistics
    pub min_requests: u32,
    /// Time window for rolling statistics
    pub window_duration: Duration,
    /// Maximum concurrent half-open requests
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            failure_rate_threshold: 0.5,
            min_requests: 10,
            window_duration: Duration::from_secs(60),
            half_open_max_requests: 1,
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerStats {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Rejected requests (circuit open)
    pub rejected_requests: u64,
    /// Current failure rate
    pub failure_rate: f64,
    /// Circuit state changes
    pub state_changes: u64,
    /// Last state change timestamp
    pub last_state_change: Option<Instant>,
    /// Time spent in each state
    pub time_in_closed: Duration,
    pub time_in_open: Duration,
    pub time_in_half_open: Duration,
}

/// Time-based sliding window for tracking request outcomes
#[derive(Debug)]
struct SlidingWindow {
    /// Window duration
    duration: Duration,
    /// Request outcomes (timestamp, success)
    outcomes: Vec<(Instant, bool)>,
    /// Success count in window
    success_count: usize,
    /// Failure count in window
    failure_count: usize,
}

impl SlidingWindow {
    fn new(duration: Duration) -> Self {
        Self {
            duration,
            outcomes: Vec::new(),
            success_count: 0,
            failure_count: 0,
        }
    }

    fn record(&mut self, success: bool) {
        let now = Instant::now();
        self.outcomes.push((now, success));

        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        self.cleanup();
    }

    fn cleanup(&mut self) {
        let cutoff = Instant::now() - self.duration;
        let mut i = 0;

        while i < self.outcomes.len() && self.outcomes[i].0 < cutoff {
            if self.outcomes[i].1 {
                self.success_count -= 1;
            } else {
                self.failure_count -= 1;
            }
            i += 1;
        }

        self.outcomes.drain(0..i);
    }

    fn total_requests(&self) -> usize {
        self.success_count + self.failure_count
    }

    fn failure_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.failure_count as f64 / total as f64
        }
    }

    fn reset(&mut self) {
        self.outcomes.clear();
        self.success_count = 0;
        self.failure_count = 0;
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    /// State transition timestamp
    state_changed_at: Arc<RwLock<Instant>>,
    /// Consecutive failures
    consecutive_failures: AtomicUsize,
    /// Consecutive successes
    consecutive_successes: AtomicUsize,
    /// Half-open request count
    half_open_requests: AtomicUsize,
    /// Request window
    window: Arc<RwLock<SlidingWindow>>,
    /// Statistics
    stats: Arc<RwLock<CircuitBreakerStats>>,
    /// State change notifier
    state_change_notify: Arc<Notify>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        let window_duration = config.window_duration;

        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            state_changed_at: Arc::new(RwLock::new(Instant::now())),
            consecutive_failures: AtomicUsize::new(0),
            consecutive_successes: AtomicUsize::new(0),
            half_open_requests: AtomicUsize::new(0),
            window: Arc::new(RwLock::new(SlidingWindow::new(window_duration))),
            stats: Arc::new(RwLock::new(CircuitBreakerStats::default())),
            state_change_notify: Arc::new(Notify::new()),
        }
    }

    /// Check if request should be allowed
    pub fn allow_request(&self) -> bool {
        let current_state = *self.state.read();

        match current_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                let elapsed = self.state_changed_at.read().elapsed();
                if elapsed >= self.config.timeout {
                    // Transition to half-open
                    self.transition_to_half_open();
                    true
                } else {
                    // Increment rejected count
                    self.stats.write().rejected_requests += 1;
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let current = self.half_open_requests.load(Ordering::Acquire);
                if current < self.config.half_open_max_requests as usize {
                    self.half_open_requests.fetch_add(1, Ordering::Release);
                    true
                } else {
                    self.stats.write().rejected_requests += 1;
                    false
                }
            }
        }
    }

    /// Record request outcome
    pub fn record_outcome(&self, success: bool) {
        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_requests += 1;
            if success {
                stats.successful_requests += 1;
            } else {
                stats.failed_requests += 1;
            }
        }

        // Update sliding window
        self.window.write().record(success);

        let current_state = *self.state.read();

        match current_state {
            CircuitState::Closed => {
                if success {
                    self.consecutive_failures.store(0, Ordering::Release);
                } else {
                    let failures = self.consecutive_failures.fetch_add(1, Ordering::AcqRel) + 1;

                    // Check failure threshold
                    if failures >= self.config.failure_threshold as usize {
                        self.check_and_open_circuit();
                    }
                }
            }
            CircuitState::Open => {
                // Should not happen - requests should be blocked
                warn!("Outcome recorded while circuit is open");
            }
            CircuitState::HalfOpen => {
                if success {
                    let successes = self.consecutive_successes.fetch_add(1, Ordering::AcqRel) + 1;

                    if successes >= self.config.success_threshold as usize {
                        self.transition_to_closed();
                    }
                } else {
                    // Single failure in half-open state reopens circuit
                    self.transition_to_open();
                }

                // Decrement half-open request count
                self.half_open_requests.fetch_sub(1, Ordering::Release);
            }
        }
    }

    /// Check failure rate and potentially open circuit
    fn check_and_open_circuit(&self) {
        let window = self.window.read();
        let total_requests = window.total_requests();
        let failure_rate = window.failure_rate();

        if total_requests >= self.config.min_requests as usize
            && failure_rate >= self.config.failure_rate_threshold
        {
            drop(window); // Release read lock before transitioning
            self.transition_to_open();
        }
    }

    /// Transition to open state
    fn transition_to_open(&self) {
        let mut state = self.state.write();
        let previous_state = *state;

        if previous_state != CircuitState::Open {
            *state = CircuitState::Open;
            *self.state_changed_at.write() = Instant::now();

            // Reset counters
            self.consecutive_failures.store(0, Ordering::Release);
            self.consecutive_successes.store(0, Ordering::Release);

            // Update statistics
            self.update_state_stats(previous_state, CircuitState::Open);

            info!(
                "Circuit breaker opened (failure rate: {:.2}%)",
                self.window.read().failure_rate() * 100.0
            );

            // Notify state change
            self.state_change_notify.notify_waiters();
        }
    }

    /// Transition to half-open state
    fn transition_to_half_open(&self) {
        let mut state = self.state.write();
        let previous_state = *state;

        if previous_state == CircuitState::Open {
            *state = CircuitState::HalfOpen;
            *self.state_changed_at.write() = Instant::now();

            // Reset counters
            self.consecutive_successes.store(0, Ordering::Release);
            self.half_open_requests.store(0, Ordering::Release);

            // Clear sliding window for fresh start
            self.window.write().reset();

            // Update statistics
            self.update_state_stats(previous_state, CircuitState::HalfOpen);

            info!("Circuit breaker half-opened for testing");

            // Notify state change
            self.state_change_notify.notify_waiters();
        }
    }

    /// Transition to closed state
    fn transition_to_closed(&self) {
        let mut state = self.state.write();
        let previous_state = *state;

        if previous_state != CircuitState::Closed {
            *state = CircuitState::Closed;
            *self.state_changed_at.write() = Instant::now();

            // Reset counters
            self.consecutive_failures.store(0, Ordering::Release);
            self.consecutive_successes.store(0, Ordering::Release);

            // Update statistics
            self.update_state_stats(previous_state, CircuitState::Closed);

            info!("Circuit breaker closed");

            // Notify state change
            self.state_change_notify.notify_waiters();
        }
    }

    /// Update state statistics
    fn update_state_stats(&self, from_state: CircuitState, _to_state: CircuitState) {
        let mut stats = self.stats.write();
        stats.state_changes += 1;

        if let Some(last_change) = stats.last_state_change {
            let duration = last_change.elapsed();

            match from_state {
                CircuitState::Closed => stats.time_in_closed += duration,
                CircuitState::Open => stats.time_in_open += duration,
                CircuitState::HalfOpen => stats.time_in_half_open += duration,
            }
        }

        stats.last_state_change = Some(Instant::now());
        stats.failure_rate = self.window.read().failure_rate();
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        *self.state.read()
    }

    /// Get statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        let mut stats = self.stats.read().clone();
        stats.failure_rate = self.window.read().failure_rate();
        stats
    }

    /// Wait for state change
    pub async fn wait_for_state_change(&self) {
        self.state_change_notify.notified().await;
    }

    /// Reset circuit breaker
    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        *self.state_changed_at.write() = Instant::now();

        self.consecutive_failures.store(0, Ordering::Release);
        self.consecutive_successes.store(0, Ordering::Release);
        self.half_open_requests.store(0, Ordering::Release);

        self.window.write().reset();

        *self.stats.write() = CircuitBreakerStats::default();

        self.state_change_notify.notify_waiters();
    }
}

/// Circuit breaker manager for multiple peers
pub struct CircuitBreakerManager {
    /// Circuit breakers per peer
    breakers: Arc<DashMap<PeerId, Arc<CircuitBreaker>>>,
    /// Default configuration
    default_config: CircuitBreakerConfig,
    /// Global statistics
    global_stats: Arc<RwLock<GlobalCircuitStats>>,
    /// Maintenance task handle
    maintenance_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Global circuit breaker statistics
#[derive(Debug, Clone, Default)]
pub struct GlobalCircuitStats {
    /// Total circuit breakers
    pub total_breakers: usize,
    /// Open circuits
    pub open_circuits: usize,
    /// Half-open circuits
    pub half_open_circuits: usize,
    /// Total requests across all circuits
    pub total_requests: u64,
    /// Total rejected requests
    pub total_rejected: u64,
    /// Average failure rate
    pub avg_failure_rate: f64,
}

impl CircuitBreakerManager {
    /// Create a new circuit breaker manager
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        let manager = Self {
            breakers: Arc::new(DashMap::new()),
            default_config,
            global_stats: Arc::new(RwLock::new(GlobalCircuitStats::default())),
            maintenance_handle: None,
        };

        // Start maintenance task
        let maintenance_manager = manager.clone();
        let handle = tokio::spawn(async move {
            maintenance_manager.run_maintenance().await;
        });

        Self {
            maintenance_handle: Some(handle),
            ..manager
        }
    }

    /// Get or create circuit breaker for a peer
    pub fn get_breaker(&self, peer_id: PeerId) -> Arc<CircuitBreaker> {
        self.breakers
            .entry(peer_id)
            .or_insert_with(|| Arc::new(CircuitBreaker::new(self.default_config.clone())))
            .clone()
    }

    /// Check if request should be allowed for a peer
    pub fn allow_request(&self, peer_id: PeerId) -> bool {
        self.get_breaker(peer_id).allow_request()
    }

    /// Record request outcome for a peer
    pub fn record_outcome(&self, peer_id: PeerId, success: bool) {
        self.get_breaker(peer_id).record_outcome(success);
    }

    /// Get circuit state for a peer
    pub fn get_state(&self, peer_id: PeerId) -> CircuitState {
        self.get_breaker(peer_id).state()
    }

    /// Get statistics for a peer
    pub fn get_stats(&self, peer_id: PeerId) -> CircuitBreakerStats {
        self.get_breaker(peer_id).stats()
    }

    /// Get global statistics
    pub fn get_global_stats(&self) -> GlobalCircuitStats {
        self.global_stats.read().clone()
    }

    /// Reset circuit breaker for a peer
    pub fn reset(&self, peer_id: PeerId) {
        if let Some(breaker) = self.breakers.get(&peer_id) {
            breaker.reset();
        }
    }

    /// Remove circuit breaker for a peer
    pub fn remove(&self, peer_id: PeerId) {
        self.breakers.remove(&peer_id);
    }

    /// Run maintenance tasks
    async fn run_maintenance(&self) {
        let mut interval = interval(Duration::from_secs(10));

        loop {
            interval.tick().await;
            self.update_global_stats();
        }
    }

    /// Update global statistics
    fn update_global_stats(&self) {
        let mut total_requests = 0u64;
        let mut total_rejected = 0u64;
        let mut open_circuits = 0;
        let mut half_open_circuits = 0;
        let mut total_failure_rate = 0.0;

        for entry in self.breakers.iter() {
            let breaker = entry.value();
            let stats = breaker.stats();

            total_requests += stats.total_requests;
            total_rejected += stats.rejected_requests;
            total_failure_rate += stats.failure_rate;

            match breaker.state() {
                CircuitState::Open => open_circuits += 1,
                CircuitState::HalfOpen => half_open_circuits += 1,
                _ => {}
            }
        }

        let total_breakers = self.breakers.len();
        let avg_failure_rate = if total_breakers > 0 {
            total_failure_rate / total_breakers as f64
        } else {
            0.0
        };

        let mut global_stats = self.global_stats.write();
        global_stats.total_breakers = total_breakers;
        global_stats.open_circuits = open_circuits;
        global_stats.half_open_circuits = half_open_circuits;
        global_stats.total_requests = total_requests;
        global_stats.total_rejected = total_rejected;
        global_stats.avg_failure_rate = avg_failure_rate;
    }

    /// Shutdown the manager
    pub fn shutdown(&mut self) {
        if let Some(handle) = self.maintenance_handle.take() {
            handle.abort();
        }
        self.breakers.clear();
    }
}

impl Clone for CircuitBreakerManager {
    fn clone(&self) -> Self {
        Self {
            breakers: self.breakers.clone(),
            default_config: self.default_config.clone(),
            global_stats: self.global_stats.clone(),
            maintenance_handle: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);

        // Circuit should start closed
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.allow_request());

        // Record success
        breaker.record_outcome(true);
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            min_requests: 1,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Record failures
        for _ in 0..3 {
            assert!(breaker.allow_request());
            breaker.record_outcome(false);
        }

        // Circuit should be open
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allow_request());

        let stats = breaker.stats();
        assert_eq!(stats.failed_requests, 3);
        assert_eq!(stats.rejected_requests, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Open circuit
        for _ in 0..2 {
            breaker.record_outcome(false);
        }
        assert_eq!(breaker.state(), CircuitState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;

        // Should transition to half-open
        assert!(breaker.allow_request());
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Open circuit
        for _ in 0..2 {
            breaker.record_outcome(false);
        }

        // Wait for timeout
        sleep(Duration::from_millis(100)).await;

        // Test recovery
        assert!(breaker.allow_request());
        breaker.record_outcome(true);
        assert!(breaker.allow_request());
        breaker.record_outcome(true);

        // Circuit should be closed
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_manager() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        // Test request allowance
        assert!(manager.allow_request(peer1));
        assert!(manager.allow_request(peer2));

        // Record outcomes
        manager.record_outcome(peer1, true);
        manager.record_outcome(peer2, false);

        // Check states
        assert_eq!(manager.get_state(peer1), CircuitState::Closed);

        // Check global stats
        let global_stats = manager.get_global_stats();
        assert_eq!(global_stats.total_breakers, 2);
    }

    #[test]
    fn test_sliding_window() {
        let mut window = SlidingWindow::new(Duration::from_secs(1));

        // Record some outcomes
        window.record(true);
        window.record(false);
        window.record(true);
        window.record(false);

        assert_eq!(window.total_requests(), 4);
        assert_eq!(window.failure_rate(), 0.5);

        // Test reset
        window.reset();
        assert_eq!(window.total_requests(), 0);
        assert_eq!(window.failure_rate(), 0.0);
    }
}
