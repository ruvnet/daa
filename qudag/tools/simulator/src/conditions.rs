//! Network condition simulation for realistic testing environments.

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

/// Network condition profiles for different environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkProfile {
    /// Local area network (LAN) conditions
    Lan,
    /// Wide area network (WAN) conditions
    Wan,
    /// Mobile/cellular network conditions
    Mobile,
    /// Satellite network conditions
    Satellite,
    /// Unstable/congested network
    Congested,
    /// Custom network profile
    Custom(NetworkConditions),
}

/// Comprehensive network condition parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConditions {
    /// Base latency (constant component)
    pub base_latency: Duration,
    /// Latency variance (for jitter simulation)
    pub latency_variance: Duration,
    /// Packet loss rate (0.0-1.0)
    pub packet_loss_rate: f64,
    /// Bandwidth limit in bytes per second
    pub bandwidth_limit: u64,
    /// Burst capacity in bytes
    pub burst_capacity: u64,
    /// Network congestion factor (0.0-1.0)
    pub congestion_factor: f64,
    /// Probability of temporary outages
    pub outage_probability: f64,
    /// Duration of temporary outages
    pub outage_duration: Duration,
}

impl NetworkProfile {
    /// Get network conditions for this profile
    pub fn conditions(&self) -> NetworkConditions {
        match self {
            NetworkProfile::Lan => NetworkConditions {
                base_latency: Duration::from_millis(1),
                latency_variance: Duration::from_millis(1),
                packet_loss_rate: 0.0001,
                bandwidth_limit: 1_000_000_000, // 1 Gbps
                burst_capacity: 10_000_000,     // 10 MB
                congestion_factor: 0.01,
                outage_probability: 0.0001,
                outage_duration: Duration::from_millis(10),
            },
            NetworkProfile::Wan => NetworkConditions {
                base_latency: Duration::from_millis(50),
                latency_variance: Duration::from_millis(20),
                packet_loss_rate: 0.001,
                bandwidth_limit: 100_000_000, // 100 Mbps
                burst_capacity: 1_000_000,    // 1 MB
                congestion_factor: 0.05,
                outage_probability: 0.001,
                outage_duration: Duration::from_millis(100),
            },
            NetworkProfile::Mobile => NetworkConditions {
                base_latency: Duration::from_millis(100),
                latency_variance: Duration::from_millis(50),
                packet_loss_rate: 0.01,
                bandwidth_limit: 10_000_000, // 10 Mbps
                burst_capacity: 100_000,     // 100 KB
                congestion_factor: 0.15,
                outage_probability: 0.01,
                outage_duration: Duration::from_millis(500),
            },
            NetworkProfile::Satellite => NetworkConditions {
                base_latency: Duration::from_millis(600),
                latency_variance: Duration::from_millis(100),
                packet_loss_rate: 0.005,
                bandwidth_limit: 25_000_000, // 25 Mbps
                burst_capacity: 500_000,     // 500 KB
                congestion_factor: 0.1,
                outage_probability: 0.005,
                outage_duration: Duration::from_secs(2),
            },
            NetworkProfile::Congested => NetworkConditions {
                base_latency: Duration::from_millis(200),
                latency_variance: Duration::from_millis(300),
                packet_loss_rate: 0.05,
                bandwidth_limit: 1_000_000, // 1 Mbps
                burst_capacity: 50_000,     // 50 KB
                congestion_factor: 0.5,
                outage_probability: 0.02,
                outage_duration: Duration::from_secs(1),
            },
            NetworkProfile::Custom(conditions) => conditions.clone(),
        }
    }
}

/// Network condition simulator that applies realistic impairments
pub struct NetworkConditionSimulator {
    conditions: NetworkConditions,
    current_bandwidth_usage: u64,
    last_bandwidth_reset: Instant,
    is_in_outage: bool,
    outage_end_time: Option<Instant>,
}

impl NetworkConditionSimulator {
    /// Create a new network condition simulator
    pub fn new(profile: NetworkProfile) -> Self {
        Self {
            conditions: profile.conditions(),
            current_bandwidth_usage: 0,
            last_bandwidth_reset: Instant::now(),
            is_in_outage: false,
            outage_end_time: None,
        }
    }

    /// Create simulator with custom conditions
    pub fn with_conditions(conditions: NetworkConditions) -> Self {
        Self {
            conditions,
            current_bandwidth_usage: 0,
            last_bandwidth_reset: Instant::now(),
            is_in_outage: false,
            outage_end_time: None,
        }
    }

    /// Simulate message transmission with network conditions
    pub async fn transmit_message(
        &mut self,
        message_size: usize,
    ) -> Result<Duration, NetworkError> {
        // Check for outages
        self.update_outage_state().await;

        if self.is_in_outage {
            return Err(NetworkError::NetworkOutage);
        }

        // Simulate packet loss
        if self.should_drop_packet() {
            debug!("Packet dropped due to network conditions");
            return Err(NetworkError::PacketLoss);
        }

        // Calculate latency with jitter
        let latency = self.calculate_latency();

        // Apply bandwidth limiting
        let transmission_delay = self.apply_bandwidth_limit(message_size).await?;

        // Total delay is latency + transmission time
        let total_delay = latency + transmission_delay;

        debug!(
            "Message transmitted: size={} bytes, latency={:?}, transmission_delay={:?}",
            message_size, latency, transmission_delay
        );

        sleep(total_delay).await;
        Ok(total_delay)
    }

    /// Check if packet should be dropped based on loss rate
    fn should_drop_packet(&self) -> bool {
        let mut rng = thread_rng();
        let loss_rate =
            self.conditions.packet_loss_rate * (1.0 + self.conditions.congestion_factor);
        rng.gen::<f64>() < loss_rate
    }

    /// Calculate latency with jitter
    fn calculate_latency(&self) -> Duration {
        let mut rng = thread_rng();
        let jitter_ms = rng.gen_range(0..=self.conditions.latency_variance.as_millis() as u64);
        let congestion_factor = 1.0 + self.conditions.congestion_factor;

        Duration::from_millis(
            (self.conditions.base_latency.as_millis() as f64 * congestion_factor) as u64
                + jitter_ms,
        )
    }

    /// Apply bandwidth limiting and return transmission delay
    async fn apply_bandwidth_limit(
        &mut self,
        message_size: usize,
    ) -> Result<Duration, NetworkError> {
        // Reset bandwidth counter every second
        let now = Instant::now();
        if now.duration_since(self.last_bandwidth_reset) >= Duration::from_secs(1) {
            self.current_bandwidth_usage = 0;
            self.last_bandwidth_reset = now;
        }

        let message_size_u64 = message_size as u64;

        // Check if we would exceed bandwidth limit
        if self.current_bandwidth_usage + message_size_u64 > self.conditions.bandwidth_limit {
            // Check if burst capacity allows this message
            if message_size_u64 > self.conditions.burst_capacity {
                return Err(NetworkError::BandwidthExceeded);
            }

            // Wait until next bandwidth window
            let wait_time = Duration::from_secs(1) - now.duration_since(self.last_bandwidth_reset);
            sleep(wait_time).await;

            // Reset counters
            self.current_bandwidth_usage = 0;
            self.last_bandwidth_reset = Instant::now();
        }

        // Calculate transmission time based on effective bandwidth
        let effective_bandwidth =
            self.conditions.bandwidth_limit as f64 * (1.0 - self.conditions.congestion_factor);
        let transmission_time = Duration::from_secs_f64(message_size as f64 / effective_bandwidth);

        self.current_bandwidth_usage += message_size_u64;

        Ok(transmission_time)
    }

    /// Update outage state based on probability
    async fn update_outage_state(&mut self) {
        let now = Instant::now();

        // Check if current outage has ended
        if let Some(end_time) = self.outage_end_time {
            if now >= end_time {
                self.is_in_outage = false;
                self.outage_end_time = None;
                debug!("Network outage ended");
            }
        }

        // Check for new outages
        if !self.is_in_outage {
            let mut rng = thread_rng();
            if rng.gen::<f64>() < self.conditions.outage_probability {
                self.is_in_outage = true;
                self.outage_end_time = Some(now + self.conditions.outage_duration);
                warn!(
                    "Network outage started, duration: {:?}",
                    self.conditions.outage_duration
                );
            }
        }
    }

    /// Get current network statistics
    pub fn get_stats(&self) -> NetworkStats {
        NetworkStats {
            current_bandwidth_usage: self.current_bandwidth_usage,
            bandwidth_utilization: self.current_bandwidth_usage as f64
                / self.conditions.bandwidth_limit as f64,
            is_in_outage: self.is_in_outage,
            effective_packet_loss_rate: self.conditions.packet_loss_rate
                * (1.0 + self.conditions.congestion_factor),
            conditions: self.conditions.clone(),
        }
    }

    /// Update network conditions dynamically
    pub fn update_conditions(&mut self, conditions: NetworkConditions) {
        self.conditions = conditions;
        debug!("Network conditions updated: {:?}", self.conditions);
    }
}

/// Network statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Current bandwidth usage in bytes per second
    pub current_bandwidth_usage: u64,
    /// Bandwidth utilization ratio (0.0-1.0)
    pub bandwidth_utilization: f64,
    /// Whether network is currently in outage
    pub is_in_outage: bool,
    /// Effective packet loss rate including congestion
    pub effective_packet_loss_rate: f64,
    /// Current network conditions
    pub conditions: NetworkConditions,
}

/// Network errors that can occur during simulation
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// Packet was dropped due to network conditions
    #[error("Packet dropped due to network conditions")]
    PacketLoss,

    /// Network is currently experiencing an outage
    #[error("Network outage in progress")]
    NetworkOutage,

    /// Message exceeds bandwidth capacity
    #[error("Message size exceeds bandwidth capacity")]
    BandwidthExceeded,

    /// General network error
    #[error("Network error: {0}")]
    General(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_lan_profile() {
        let mut simulator = NetworkConditionSimulator::new(NetworkProfile::Lan);
        let result = simulator.transmit_message(1000).await;
        assert!(result.is_ok());

        let delay = result.unwrap();
        assert!(delay < Duration::from_millis(10)); // LAN should be fast
    }

    #[tokio::test]
    async fn test_satellite_profile() {
        let mut simulator = NetworkConditionSimulator::new(NetworkProfile::Satellite);
        let result = simulator.transmit_message(1000).await;
        assert!(result.is_ok());

        let delay = result.unwrap();
        assert!(delay > Duration::from_millis(500)); // Satellite should have high latency
    }

    #[tokio::test]
    async fn test_packet_loss() {
        let conditions = NetworkConditions {
            base_latency: Duration::from_millis(10),
            latency_variance: Duration::from_millis(1),
            packet_loss_rate: 1.0, // 100% loss rate
            bandwidth_limit: 1_000_000,
            burst_capacity: 10_000,
            congestion_factor: 0.0,
            outage_probability: 0.0,
            outage_duration: Duration::from_millis(100),
        };

        let mut simulator = NetworkConditionSimulator::with_conditions(conditions);
        let result = simulator.transmit_message(1000).await;
        assert!(matches!(result, Err(NetworkError::PacketLoss)));
    }

    #[tokio::test]
    async fn test_bandwidth_limiting() {
        let conditions = NetworkConditions {
            base_latency: Duration::from_millis(1),
            latency_variance: Duration::from_millis(1),
            packet_loss_rate: 0.0,
            bandwidth_limit: 1000, // Very low bandwidth
            burst_capacity: 500,
            congestion_factor: 0.0,
            outage_probability: 0.0,
            outage_duration: Duration::from_millis(100),
        };

        let mut simulator = NetworkConditionSimulator::with_conditions(conditions);

        // First small message should succeed
        let result = simulator.transmit_message(100).await;
        assert!(result.is_ok());

        // Large message should be rejected
        let result = simulator.transmit_message(2000).await;
        assert!(matches!(result, Err(NetworkError::BandwidthExceeded)));
    }

    #[tokio::test]
    async fn test_network_stats() {
        let mut simulator = NetworkConditionSimulator::new(NetworkProfile::Wan);
        let _ = simulator.transmit_message(1000).await;

        let stats = simulator.get_stats();
        assert!(stats.bandwidth_utilization >= 0.0);
        assert!(stats.bandwidth_utilization <= 1.0);
        assert_eq!(stats.conditions.base_latency, Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_profile_serialization() {
        let profile = NetworkProfile::Mobile;
        let conditions = profile.conditions();

        let serialized = serde_json::to_string(&conditions).unwrap();
        let deserialized: NetworkConditions = serde_json::from_str(&serialized).unwrap();

        assert_eq!(conditions.base_latency, deserialized.base_latency);
        assert_eq!(conditions.packet_loss_rate, deserialized.packet_loss_rate);
    }
}
