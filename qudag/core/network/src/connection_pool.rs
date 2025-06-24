#![deny(unsafe_code)]

use crate::connection::{ConnectionInfo, PooledConnection, WarmingState};
use crate::types::{ConnectionStatus, NetworkError, PeerId};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Notify, Semaphore};
use tokio::time::{interval, sleep};
use tracing::{debug, warn};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum connections in pool
    pub max_size: usize,
    /// Minimum connections to maintain
    pub min_size: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Connection max lifetime
    pub max_lifetime: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Connection acquisition timeout
    pub acquire_timeout: Duration,
    /// Enable connection warming
    pub enable_warming: bool,
    /// Connection validation on checkout
    pub validate_on_checkout: bool,
    /// Maximum connection reuse count
    pub max_reuse_count: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            min_size: 10,
            idle_timeout: Duration::from_secs(300), // 5 minutes
            max_lifetime: Duration::from_secs(3600), // 1 hour
            health_check_interval: Duration::from_secs(30),
            acquire_timeout: Duration::from_secs(10),
            enable_warming: true,
            validate_on_checkout: true,
            max_reuse_count: 1000,
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total connections created
    pub total_created: u64,
    /// Total connections destroyed
    pub total_destroyed: u64,
    /// Current pool size
    pub current_size: usize,
    /// Available connections
    pub available: usize,
    /// Active connections
    pub active: usize,
    /// Connection acquisition count
    pub acquisitions: u64,
    /// Connection release count
    pub releases: u64,
    /// Failed acquisition attempts
    pub failed_acquisitions: u64,
    /// Connection timeout count
    pub timeouts: u64,
    /// Average wait time for connection
    pub avg_wait_time: Duration,
    /// Pool hit rate
    pub hit_rate: f64,
}

/// Connection pool for efficient connection management
pub struct ConnectionPool {
    /// Pool configuration
    config: PoolConfig,
    /// Available connections
    available: Arc<DashMap<PeerId, VecDeque<PooledConnection>>>,
    /// Active connections (checked out)
    active: Arc<DashMap<PeerId, HashMap<u64, PooledConnection>>>,
    /// Connection semaphores per peer
    semaphores: Arc<DashMap<PeerId, Arc<Semaphore>>>,
    /// Pool statistics
    stats: Arc<RwLock<PoolStats>>,
    /// Connection ID counter
    connection_counter: AtomicUsize,
    /// Pool shutdown flag
    shutdown: AtomicBool,
    /// Connection waiters
    waiters: Arc<DashMap<PeerId, Arc<Notify>>>,
    /// Maintenance task handle
    #[allow(dead_code)]
    maintenance_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: PoolConfig) -> Self {
        let pool = Self {
            config: config.clone(),
            available: Arc::new(DashMap::new()),
            active: Arc::new(DashMap::new()),
            semaphores: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(PoolStats::default())),
            connection_counter: AtomicUsize::new(0),
            shutdown: AtomicBool::new(false),
            waiters: Arc::new(DashMap::new()),
            maintenance_handle: None,
        };

        // Start maintenance task
        let maintenance_pool = pool.clone();
        let handle = tokio::spawn(async move {
            maintenance_pool.run_maintenance().await;
        });

        Self {
            maintenance_handle: Some(handle),
            ..pool
        }
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self, peer_id: PeerId) -> Result<PooledConnection, NetworkError> {
        if self.shutdown.load(Ordering::Acquire) {
            return Err(NetworkError::ConnectionError(
                "Pool is shutting down".into(),
            ));
        }

        let start_time = Instant::now();

        // Get or create semaphore for this peer
        let semaphore = self
            .semaphores
            .entry(peer_id)
            .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_size)))
            .clone();

        // Try to acquire permit with timeout
        let permit = tokio::select! {
            result = semaphore.acquire() => {
                result.map_err(|_| NetworkError::ConnectionError("Semaphore closed".into()))?
            }
            _ = sleep(self.config.acquire_timeout) => {
                self.increment_timeouts();
                return Err(NetworkError::ConnectionError("Connection acquisition timeout".into()));
            }
        };

        // Check available connections
        if let Some(mut available_queue) = self.available.get_mut(&peer_id) {
            while let Some(mut conn) = available_queue.pop_front() {
                // Validate connection
                if self.is_connection_valid(&conn) {
                    if self.config.validate_on_checkout {
                        // Perform additional validation if needed
                        if !self.validate_connection(&conn).await {
                            continue;
                        }
                    }

                    // Update connection state
                    conn.last_used = Instant::now();
                    conn.usage_count += 1;

                    // Move to active connections
                    let conn_id = self.connection_counter.fetch_add(1, Ordering::Relaxed) as u64;
                    self.active
                        .entry(peer_id)
                        .or_insert_with(HashMap::new)
                        .insert(conn_id, conn.clone());

                    // Update statistics
                    self.update_acquisition_stats(start_time.elapsed());

                    // Forget the permit (keep it alive)
                    std::mem::forget(permit);

                    return Ok(conn);
                }
            }
        }

        // No available connection, create new one if under limit
        if self.get_peer_connection_count(peer_id) < self.config.max_size {
            let conn = self.create_connection(peer_id).await?;

            // Move to active connections
            let conn_id = self.connection_counter.fetch_add(1, Ordering::Relaxed) as u64;
            self.active
                .entry(peer_id)
                .or_insert_with(HashMap::new)
                .insert(conn_id, conn.clone());

            // Update statistics
            self.update_acquisition_stats(start_time.elapsed());
            self.increment_created();

            // Forget the permit (keep it alive)
            std::mem::forget(permit);

            Ok(conn)
        } else {
            // Wait for a connection to become available
            let waiter = self
                .waiters
                .entry(peer_id)
                .or_insert_with(|| Arc::new(Notify::new()))
                .clone();

            drop(permit); // Release permit while waiting

            tokio::select! {
                _ = waiter.notified() => {
                    // Retry acquisition
                    Box::pin(self.acquire(peer_id)).await
                }
                _ = sleep(self.config.acquire_timeout) => {
                    self.increment_failed_acquisitions();
                    Err(NetworkError::ConnectionError("No available connections".into()))
                }
            }
        }
    }

    /// Release a connection back to the pool
    pub fn release(&self, peer_id: PeerId, mut connection: PooledConnection) {
        if self.shutdown.load(Ordering::Acquire) {
            return;
        }

        // Update connection state
        connection.last_used = Instant::now();

        // Check if connection should be kept
        if !self.should_keep_connection(&connection) {
            self.destroy_connection(peer_id, connection);
            return;
        }

        // Return to available pool
        self.available
            .entry(peer_id)
            .or_insert_with(VecDeque::new)
            .push_back(connection);

        // Notify waiters
        if let Some(waiter) = self.waiters.get(&peer_id) {
            waiter.notify_one();
        }

        // Update statistics
        self.increment_releases();
    }

    /// Validate a connection
    async fn validate_connection(&self, conn: &PooledConnection) -> bool {
        // Basic validation - check if connection is healthy
        if !conn.info.is_healthy() {
            return false;
        }

        // Additional validation could include:
        // - Ping test
        // - Resource usage check
        // - Performance metrics validation

        true
    }

    /// Check if connection is valid for use
    fn is_connection_valid(&self, conn: &PooledConnection) -> bool {
        // Check lifetime
        if conn.created_at.elapsed() > self.config.max_lifetime {
            return false;
        }

        // Check idle time
        if conn.last_used.elapsed() > self.config.idle_timeout {
            return false;
        }

        // Check reuse count
        if conn.usage_count >= self.config.max_reuse_count {
            return false;
        }

        // Check health
        conn.info.is_healthy()
    }

    /// Check if connection should be kept in pool
    fn should_keep_connection(&self, conn: &PooledConnection) -> bool {
        self.is_connection_valid(conn) && self.get_total_connection_count() < self.config.max_size
    }

    /// Create a new connection
    async fn create_connection(&self, _peer_id: PeerId) -> Result<PooledConnection, NetworkError> {
        // Simulate connection creation (in real implementation, this would establish actual connection)
        let info = ConnectionInfo::new(ConnectionStatus::Connected);

        let mut conn = PooledConnection {
            info,
            created_at: Instant::now(),
            last_used: Instant::now(),
            usage_count: 0,
            weight: 1.0,
            max_streams: 100,
            active_streams: 0,
            warming_state: WarmingState::Cold,
            affinity_group: None,
        };

        // Warm connection if enabled
        if self.config.enable_warming {
            self.warm_connection(&mut conn).await?;
        }

        Ok(conn)
    }

    /// Warm a connection
    async fn warm_connection(&self, conn: &mut PooledConnection) -> Result<(), NetworkError> {
        conn.warming_state = WarmingState::Warming;

        // Simulate warming process
        sleep(Duration::from_millis(50)).await;

        // In real implementation, this would:
        // - Establish TLS handshake
        // - Perform protocol negotiation
        // - Prime any caches
        // - Run initial health checks

        conn.warming_state = WarmingState::Warm;
        Ok(())
    }

    /// Destroy a connection
    fn destroy_connection(&self, _peer_id: PeerId, _conn: PooledConnection) {
        // In real implementation, this would close the actual connection
        self.increment_destroyed();
    }

    /// Get connection count for a peer
    fn get_peer_connection_count(&self, peer_id: PeerId) -> usize {
        let available_count = self
            .available
            .get(&peer_id)
            .map(|queue| queue.len())
            .unwrap_or(0);

        let active_count = self.active.get(&peer_id).map(|map| map.len()).unwrap_or(0);

        available_count + active_count
    }

    /// Get total connection count
    fn get_total_connection_count(&self) -> usize {
        let available_count: usize = self.available.iter().map(|entry| entry.value().len()).sum();

        let active_count: usize = self.active.iter().map(|entry| entry.value().len()).sum();

        available_count + active_count
    }

    /// Run maintenance tasks
    async fn run_maintenance(&self) {
        let mut interval = interval(self.config.health_check_interval);

        while !self.shutdown.load(Ordering::Acquire) {
            interval.tick().await;

            // Clean up expired connections
            self.cleanup_expired_connections();

            // Maintain minimum pool size
            self.maintain_minimum_size().await;

            // Update pool statistics
            self.update_pool_stats();
        }
    }

    /// Clean up expired connections
    fn cleanup_expired_connections(&self) {
        for mut entry in self.available.iter_mut() {
            let peer_id = *entry.key();
            let queue = entry.value_mut();

            // Remove invalid connections
            queue.retain(|conn| {
                if self.is_connection_valid(conn) {
                    true
                } else {
                    self.destroy_connection(peer_id, conn.clone());
                    false
                }
            });
        }
    }

    /// Maintain minimum pool size
    async fn maintain_minimum_size(&self) {
        // This is a simplified version - in production, you'd want more sophisticated logic
        let total_count = self.get_total_connection_count();

        if total_count < self.config.min_size {
            let needed = self.config.min_size - total_count;
            debug!("Pool below minimum size, creating {} connections", needed);

            // Create connections for known peers
            for entry in self.available.iter() {
                let peer_id = *entry.key();
                for _ in 0..needed {
                    match self.create_connection(peer_id).await {
                        Ok(conn) => {
                            self.available
                                .entry(peer_id)
                                .or_insert_with(VecDeque::new)
                                .push_back(conn);
                            self.increment_created();
                        }
                        Err(e) => {
                            warn!("Failed to create connection during maintenance: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Update pool statistics
    fn update_pool_stats(&self) {
        let mut stats = self.stats.write();

        stats.current_size = self.get_total_connection_count();
        stats.available = self.available.iter().map(|entry| entry.value().len()).sum();
        stats.active = self.active.iter().map(|entry| entry.value().len()).sum();

        // Calculate hit rate
        if stats.acquisitions > 0 {
            stats.hit_rate = 1.0 - (stats.failed_acquisitions as f64 / stats.acquisitions as f64);
        }
    }

    /// Shutdown the pool
    pub async fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::Release);

        // Stop maintenance task
        if let Some(handle) = self.maintenance_handle.take() {
            handle.abort();
        }

        // Close all connections
        for entry in self.available.iter() {
            let peer_id = *entry.key();
            for conn in entry.value().iter() {
                self.destroy_connection(peer_id, conn.clone());
            }
        }

        for entry in self.active.iter() {
            let peer_id = *entry.key();
            for (_, conn) in entry.value().iter() {
                self.destroy_connection(peer_id, conn.clone());
            }
        }

        // Clear all data
        self.available.clear();
        self.active.clear();
        self.semaphores.clear();
        self.waiters.clear();
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        self.stats.read().clone()
    }

    // Statistics update methods
    fn increment_created(&self) {
        self.stats.write().total_created += 1;
    }

    fn increment_destroyed(&self) {
        self.stats.write().total_destroyed += 1;
    }

    fn increment_releases(&self) {
        self.stats.write().releases += 1;
    }

    fn increment_timeouts(&self) {
        self.stats.write().timeouts += 1;
    }

    fn increment_failed_acquisitions(&self) {
        self.stats.write().failed_acquisitions += 1;
    }

    fn update_acquisition_stats(&self, wait_time: Duration) {
        let mut stats = self.stats.write();
        stats.acquisitions += 1;

        // Update average wait time (exponential moving average)
        let alpha = 0.1;
        let current_avg = stats.avg_wait_time.as_millis() as f64;
        let new_wait = wait_time.as_millis() as f64;
        let updated_avg = alpha * new_wait + (1.0 - alpha) * current_avg;
        stats.avg_wait_time = Duration::from_millis(updated_avg as u64);
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            available: self.available.clone(),
            active: self.active.clone(),
            semaphores: self.semaphores.clone(),
            stats: self.stats.clone(),
            connection_counter: AtomicUsize::new(self.connection_counter.load(Ordering::Relaxed)),
            shutdown: AtomicBool::new(self.shutdown.load(Ordering::Relaxed)),
            waiters: self.waiters.clone(),
            maintenance_handle: None, // Don't clone the maintenance task
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(config);

        let stats = pool.get_stats();
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.available, 0);
        assert_eq!(stats.active, 0);
    }

    #[tokio::test]
    async fn test_connection_acquisition() {
        let config = PoolConfig {
            max_size: 10,
            min_size: 0,
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);
        let peer_id = PeerId::random();

        // Acquire connection
        let conn = pool.acquire(peer_id).await.unwrap();
        assert_eq!(conn.usage_count, 1);

        let stats = pool.get_stats();
        assert_eq!(stats.acquisitions, 1);
        assert_eq!(stats.total_created, 1);
    }

    #[tokio::test]
    async fn test_connection_release() {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(config);
        let peer_id = PeerId::random();

        // Acquire and release connection
        let conn = pool.acquire(peer_id).await.unwrap();
        pool.release(peer_id, conn);

        let stats = pool.get_stats();
        assert_eq!(stats.releases, 1);
        assert_eq!(stats.available, 1);
    }

    #[tokio::test]
    async fn test_connection_reuse() {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(config);
        let peer_id = PeerId::random();

        // First acquisition
        let conn1 = pool.acquire(peer_id).await.unwrap();
        let created_at = conn1.created_at;
        pool.release(peer_id, conn1);

        // Second acquisition should reuse
        let conn2 = pool.acquire(peer_id).await.unwrap();
        assert_eq!(conn2.created_at, created_at);
        assert_eq!(conn2.usage_count, 2);

        let stats = pool.get_stats();
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.acquisitions, 2);
    }

    #[tokio::test]
    async fn test_pool_limits() {
        let config = PoolConfig {
            max_size: 2,
            acquire_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);
        let peer_id = PeerId::random();

        // Acquire max connections
        let conn1 = pool.acquire(peer_id).await.unwrap();
        let conn2 = pool.acquire(peer_id).await.unwrap();

        // Third acquisition should timeout
        let result = pool.acquire(peer_id).await;
        assert!(result.is_err());

        // Release one and try again
        pool.release(peer_id, conn1);
        let conn3 = pool.acquire(peer_id).await;
        assert!(conn3.is_ok());

        // Cleanup
        pool.release(peer_id, conn2);
        pool.release(peer_id, conn3.unwrap());
    }

    #[tokio::test]
    async fn test_connection_expiration() {
        let config = PoolConfig {
            idle_timeout: Duration::from_millis(100),
            health_check_interval: Duration::from_millis(50),
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);
        let peer_id = PeerId::random();

        // Create and release connection
        let conn = pool.acquire(peer_id).await.unwrap();
        pool.release(peer_id, conn);

        // Wait for expiration
        sleep(Duration::from_millis(200)).await;

        // Connection should be cleaned up
        let stats = pool.get_stats();
        assert_eq!(stats.available, 0);
    }
}
