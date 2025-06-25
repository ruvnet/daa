//! Common test utilities for Prime-Rust integration tests

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tempfile::TempDir;
use libp2p::{Multiaddr, PeerId};
use fake::{Fake, Faker};

pub mod network;
pub mod fixtures;
pub mod generators;
pub mod assertions;
pub mod mock_nodes;

/// Test configuration builder
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub num_nodes: usize,
    pub network_size: usize,
    pub enable_logging: bool,
    pub timeout: Duration,
    pub temp_dir: Option<TempDir>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            num_nodes: 3,
            network_size: 10,
            enable_logging: false,
            timeout: Duration::from_secs(30),
            temp_dir: None,
        }
    }
}

impl TestConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_nodes(mut self, n: usize) -> Self {
        self.num_nodes = n;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_logging(mut self) -> Self {
        self.enable_logging = true;
        self
    }

    pub fn with_temp_dir(mut self) -> Self {
        self.temp_dir = Some(TempDir::new().expect("Failed to create temp dir"));
        self
    }
}

/// Initialize test environment
pub fn init_test_env() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("prime=debug,libp2p=info")
        .with_test_writer()
        .try_init();
}

/// Generate a random peer ID
pub fn random_peer_id() -> PeerId {
    PeerId::random()
}

/// Generate a random socket address
pub fn random_socket_addr() -> SocketAddr {
    let port: u16 = (49152..65535).fake();
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

/// Generate a random multiaddr
pub fn random_multiaddr() -> Multiaddr {
    let addr = random_socket_addr();
    format!("/ip4/{}/tcp/{}", addr.ip(), addr.port())
        .parse()
        .unwrap()
}

/// Test result helper
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Retry helper for flaky network operations
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut f: F,
    max_attempts: usize,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;
    
    for attempt in 1..=max_attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts => return Err(e),
            _ => {
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }
    
    unreachable!()
}

/// Performance measurement helper
pub struct PerfMeasure {
    name: String,
    start: std::time::Instant,
}

impl PerfMeasure {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for PerfMeasure {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        tracing::info!(
            "{} completed in {:.3}s",
            self.name,
            elapsed.as_secs_f64()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = TestConfig::new()
            .with_nodes(5)
            .with_timeout(Duration::from_secs(60))
            .with_logging();

        assert_eq!(config.num_nodes, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.enable_logging);
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let mut attempts = 0;
        let result = retry_with_backoff(
            || {
                attempts += 1;
                async move {
                    if attempts < 3 {
                        Err("not yet")
                    } else {
                        Ok(42)
                    }
                }
            },
            5,
            Duration::from_millis(10),
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 3);
    }
}