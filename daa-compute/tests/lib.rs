//! DAA Compute Test Library
//!
//! Comprehensive test suite for the DAA Compute distributed training system.
//! This library organizes all test modules and provides utilities for testing.

pub mod unit {
    pub mod p2p {
        pub mod test_compression;
        pub mod test_gradient;
    }

    pub mod protocols {
        pub mod test_aggregation;
    }
}

pub mod integration {
    pub mod test_p2p_networking;
}

pub mod simulation {
    pub mod test_node_churn;
}

pub mod benchmarks {
    pub mod test_training_performance;
}

// Common test utilities
pub mod common {
    use daa_compute::training::Gradient;
    use libp2p::PeerId;
    use std::time::Duration;

    /// Create a test gradient with specified parameters
    pub fn create_test_gradient(node_id: &str, values: Vec<f32>, round: u64) -> Gradient {
        Gradient {
            values,
            node_id: node_id.to_string(),
            round,
            compressed: false,
        }
    }

    /// Create multiple test gradients for load testing
    pub fn create_test_gradients(count: usize, size: usize) -> Vec<Gradient> {
        (0..count)
            .map(|i| {
                let values: Vec<f32> = (0..size).map(|j| ((i * size + j) as f32).sin()).collect();
                create_test_gradient(&format!("node_{}", i), values, 1)
            })
            .collect()
    }

    /// Assert that two floating point vectors are approximately equal
    pub fn assert_approx_eq(a: &[f32], b: &[f32], tolerance: f32) {
        assert_eq!(a.len(), b.len(), "Vector lengths don't match");

        for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
            assert!(
                (x - y).abs() < tolerance,
                "Values at index {} don't match: {} != {} (tolerance: {})",
                i,
                x,
                y,
                tolerance
            );
        }
    }

    /// Wait for a condition with timeout
    pub async fn wait_for_condition<F, Fut>(
        mut condition: F,
        timeout: Duration,
        check_interval: Duration,
    ) -> Result<(), &'static str>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if condition().await {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await;
        }

        Err("Condition not met within timeout")
    }

    /// Generate random peer ID for testing
    pub fn random_peer_id() -> PeerId {
        PeerId::random()
    }

    /// Helpers for tests that run real `P2PNetwork`s over loopback TCP.
    ///
    /// A swarm only makes progress while it is polled, so tests pump every
    /// network they own and poll for the state they expect against a deadline
    /// rather than sleeping for a fixed time.
    pub mod net {
        use anyhow::{anyhow, Result};
        use daa_compute::{Multiaddr, P2PNetwork, SwarmConfig};
        use futures::future::select_all;
        use std::time::{Duration, Instant};

        /// Upper bound for any single network condition to become true.
        pub const DEADLINE: Duration = Duration::from_secs(20);

        /// A network on an OS-assigned loopback port.
        pub async fn loopback_network() -> Result<P2PNetwork> {
            let config = SwarmConfig {
                listen_addresses: vec!["/ip4/127.0.0.1/tcp/0".parse()?],
                ..SwarmConfig::default()
            };
            P2PNetwork::new(config).await
        }

        /// Handle swarm events on all `networks` for up to `slice`.
        pub async fn pump(networks: &mut [&mut P2PNetwork], slice: Duration) -> Result<()> {
            let end = Instant::now() + slice;
            while Instant::now() < end {
                let events = networks
                    .iter_mut()
                    .map(|network| Box::pin(network.process_next_event()));
                // The first network with an event is handled; the others are
                // still waiting on their swarm, which is cancellation-safe.
                match tokio::time::timeout_at(
                    tokio::time::Instant::from_std(end),
                    select_all(events),
                )
                .await
                {
                    Ok((result, _, _)) => result?,
                    Err(_) => break,
                }
            }
            Ok(())
        }

        /// Pump `network` until it reports a listen address.
        pub async fn listen_address(network: &mut P2PNetwork) -> Result<Multiaddr> {
            let start = Instant::now();
            while start.elapsed() < DEADLINE {
                if let Some(addr) = network.listen_addresses().into_iter().next() {
                    return Ok(addr);
                }
                pump(&mut [&mut *network], Duration::from_millis(50)).await?;
            }
            Err(anyhow!("network did not start listening"))
        }

        /// Dial `hub` from each of `spokes` and pump until every spoke is connected.
        pub async fn connect_star(
            hub: &mut P2PNetwork,
            spokes: &mut [&mut P2PNetwork],
        ) -> Result<()> {
            let hub_addr = listen_address(hub).await?;
            let hub_id = hub.local_peer_id();
            for spoke in spokes.iter_mut() {
                spoke.connect_to_peer(hub_id, hub_addr.clone())?;
            }
            let start = Instant::now();
            while start.elapsed() < DEADLINE {
                if spokes
                    .iter()
                    .all(|spoke| spoke.connected_peers().contains(&hub_id))
                {
                    return Ok(());
                }
                let mut all: Vec<&mut P2PNetwork> = vec![&mut *hub];
                all.extend(spokes.iter_mut().map(|spoke| &mut **spoke));
                pump(&mut all, Duration::from_millis(50)).await?;
            }
            Err(anyhow!("spokes did not connect to the hub"))
        }

        /// Publish from `networks[sender]` once a peer is subscribed to the
        /// gradient topic (gossipsub rejects publishes with no subscribers).
        pub async fn broadcast(
            networks: &mut [&mut P2PNetwork],
            sender: usize,
            gradient: Vec<f32>,
        ) -> Result<()> {
            let start = Instant::now();
            loop {
                match networks[sender].broadcast_gradient(gradient.clone()).await {
                    Ok(()) => return Ok(()),
                    Err(e) if start.elapsed() > DEADLINE => return Err(e),
                    Err(_) => pump(networks, Duration::from_millis(100)).await?,
                }
            }
        }

        /// Pump until `networks[receiver]` holds gradients from at least `count` peers.
        pub async fn await_gradients(
            networks: &mut [&mut P2PNetwork],
            receiver: usize,
            count: usize,
        ) -> Result<Vec<Vec<f32>>> {
            let start = Instant::now();
            while start.elapsed() < DEADLINE {
                let received = networks[receiver].received_gradients().await;
                if received.len() >= count {
                    return Ok(received.into_iter().map(|(_, values)| values).collect());
                }
                pump(networks, Duration::from_millis(50)).await?;
            }
            Err(anyhow!("expected gradients from {count} peers"))
        }
    }
}

// Test configuration constants
pub mod config {
    use std::time::Duration;

    pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);
    pub const NETWORK_SETUP_DELAY: Duration = Duration::from_millis(500);
    pub const MESSAGE_PROPAGATION_DELAY: Duration = Duration::from_millis(200);
    pub const NODE_FAILURE_DETECTION_DELAY: Duration = Duration::from_secs(2);

    pub const SMALL_GRADIENT_SIZE: usize = 100;
    pub const MEDIUM_GRADIENT_SIZE: usize = 10_000;
    pub const LARGE_GRADIENT_SIZE: usize = 1_000_000;

    pub const BASE_TEST_PORT: u16 = 8000;
}

// Test result analysis utilities
pub mod analysis {
    use std::time::Duration;

    #[derive(Debug, Clone)]
    pub struct PerformanceMetrics {
        pub throughput: f64,        // operations per second
        pub latency_ms: f64,        // average latency in milliseconds
        pub memory_usage_mb: f64,   // memory usage in megabytes
        pub cpu_usage_percent: f64, // CPU usage percentage
    }

    impl PerformanceMetrics {
        pub fn calculate_throughput(operations: usize, duration: Duration) -> f64 {
            operations as f64 / duration.as_secs_f64()
        }

        pub fn calculate_latency_ms(total_duration: Duration, operations: usize) -> f64 {
            total_duration.as_millis() as f64 / operations as f64
        }
    }

    #[derive(Debug, Clone)]
    pub struct NetworkMetrics {
        pub bytes_sent: u64,
        pub bytes_received: u64,
        pub messages_sent: u64,
        pub messages_received: u64,
        pub connection_failures: u32,
        pub average_round_trip_ms: f64,
    }

    impl Default for NetworkMetrics {
        fn default() -> Self {
            Self {
                bytes_sent: 0,
                bytes_received: 0,
                messages_sent: 0,
                messages_received: 0,
                connection_failures: 0,
                average_round_trip_ms: 0.0,
            }
        }
    }
}

// Mock implementations for testing
pub mod mocks {
    use daa_compute::training::{Gradient, ModelInterface, ModelParameters};

    /// Mock model for testing purposes
    pub struct MockModel {
        pub parameters: Vec<f32>,
        pub version: u64,
    }

    impl MockModel {
        pub fn new(size: usize) -> Self {
            Self {
                parameters: vec![0.1; size],
                version: 0,
            }
        }
    }

    impl ModelInterface for MockModel {
        fn forward(&self, input: &[f32]) -> Vec<f32> {
            // Simple mock forward pass
            input
                .iter()
                .zip(self.parameters.iter())
                .map(|(x, w)| x * w)
                .collect()
        }

        fn backward(&mut self, loss: f32) -> Gradient {
            // Mock gradient calculation
            let gradient_values = self
                .parameters
                .iter()
                .map(|w| loss * w * 0.01) // Simple gradient computation
                .collect();

            Gradient {
                values: gradient_values,
                node_id: "mock_model".to_string(),
                round: self.version,
                compressed: false,
            }
        }

        fn apply_gradient(&mut self, gradient: &Gradient) {
            // Apply gradient with simple SGD
            for (param, grad) in self.parameters.iter_mut().zip(gradient.values.iter()) {
                *param -= 0.01 * grad; // Learning rate = 0.01
            }
            self.version += 1;
        }

        fn get_parameters(&self) -> ModelParameters {
            ModelParameters {
                weights: self.parameters.clone(),
                version: self.version,
                hash: format!("mock_hash_{}", self.version),
            }
        }

        fn set_parameters(&mut self, params: ModelParameters) {
            self.parameters = params.weights;
            self.version = params.version;
        }
    }

    /// Mock network for testing without actual networking
    pub struct MockNetwork {
        pub peer_id: libp2p::PeerId,
        pub received_gradients: Vec<Gradient>,
        pub connected_peers: Vec<libp2p::PeerId>,
    }

    impl Default for MockNetwork {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockNetwork {
        pub fn new() -> Self {
            Self {
                peer_id: libp2p::PeerId::random(),
                received_gradients: Vec::new(),
                connected_peers: Vec::new(),
            }
        }

        pub async fn simulate_gradient_broadcast(&mut self, gradient: Gradient) {
            // Simulate network delay
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            self.received_gradients.push(gradient);
        }

        pub fn simulate_peer_connection(&mut self, peer_id: libp2p::PeerId) {
            if !self.connected_peers.contains(&peer_id) {
                self.connected_peers.push(peer_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_utilities() {
        let gradient = common::create_test_gradient("test", vec![1.0, 2.0, 3.0], 42);
        assert_eq!(gradient.node_id, "test");
        assert_eq!(gradient.values, vec![1.0, 2.0, 3.0]);
        assert_eq!(gradient.round, 42);
    }

    #[test]
    fn test_gradient_creation() {
        let gradients = common::create_test_gradients(3, 5);
        assert_eq!(gradients.len(), 3);
        assert_eq!(gradients[0].values.len(), 5);
    }

    #[test]
    fn test_approx_eq() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.001, 1.999, 3.001];
        common::assert_approx_eq(&a, &b, 0.01);
    }

    #[test]
    fn test_performance_metrics() {
        use std::time::Duration;

        let throughput =
            analysis::PerformanceMetrics::calculate_throughput(1000, Duration::from_secs(1));
        assert_eq!(throughput, 1000.0);

        let latency =
            analysis::PerformanceMetrics::calculate_latency_ms(Duration::from_millis(1000), 10);
        assert_eq!(latency, 100.0);
    }

    #[test]
    fn test_mock_model() {
        use daa_compute::training::ModelInterface;

        let mut model = mocks::MockModel::new(3);

        let output = model.forward(&[1.0, 2.0, 3.0]);
        assert_eq!(output, vec![0.1, 0.2, 0.3]);

        let gradient = model.backward(1.0);
        assert_eq!(gradient.values.len(), 3);

        model.apply_gradient(&gradient);
        assert_eq!(model.version, 1);
    }

    #[tokio::test]
    async fn test_mock_network() {
        let mut network = mocks::MockNetwork::new();
        let gradient = common::create_test_gradient("test", vec![1.0], 1);

        network.simulate_gradient_broadcast(gradient).await;
        assert_eq!(network.received_gradients.len(), 1);

        let peer_id = libp2p::PeerId::random();
        network.simulate_peer_connection(peer_id);
        assert_eq!(network.connected_peers.len(), 1);
        assert_eq!(network.connected_peers[0], peer_id);
    }
}
