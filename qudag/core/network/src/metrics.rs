use std::time::{Duration, Instant};

/// Queue performance metrics
#[derive(Debug)]
pub struct QueueMetrics {
    /// Current queue depth
    pub depth: usize,
    /// Peak queue depth
    pub peak_depth: usize,
    /// Total messages processed
    pub total_messages: u64,
    /// Messages processed per second
    pub messages_per_second: f64,
    /// Last metrics update
    pub last_update: Instant,
}

impl Default for QueueMetrics {
    fn default() -> Self {
        Self {
            depth: 0,
            peak_depth: 0,
            total_messages: 0,
            messages_per_second: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// Latency metrics
#[derive(Debug, Default)]
pub struct LatencyMetrics {
    /// Average message latency
    pub avg_latency: Duration,
    /// Peak message latency
    pub peak_latency: Duration,
    /// Last 100 latency samples
    latency_samples: Vec<Duration>,
}

/// Throughput metrics
#[derive(Debug)]
pub struct ThroughputMetrics {
    /// Messages per second
    pub messages_per_second: f64,
    /// Bytes per second
    pub bytes_per_second: f64,
    /// Peak messages per second
    pub peak_messages_per_second: f64,
    /// Total bytes transferred
    pub total_bytes: u64,
    /// Last update timestamp
    pub last_update: Instant,
}

impl Default for ThroughputMetrics {
    fn default() -> Self {
        Self {
            messages_per_second: 0.0,
            bytes_per_second: 0.0,
            peak_messages_per_second: 0.0,
            total_bytes: 0,
            last_update: Instant::now(),
        }
    }
}

/// Network performance metrics
#[derive(Debug, Default)]
pub struct NetworkMetrics {
    /// Active connections
    pub connections: usize,
    /// Failed connection attempts
    pub connection_failures: u64,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// Route cache hits
    pub route_cache_hits: u64,
    /// Average latency
    pub avg_latency: Duration,
}

impl QueueMetrics {
    /// Record message processing
    pub fn record_message(&mut self) {
        self.total_messages += 1;
        self.depth = self.depth.saturating_add(1);
        self.peak_depth = self.peak_depth.max(self.depth);

        let elapsed = self.last_update.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.messages_per_second = self.total_messages as f64 / elapsed.as_secs_f64();
            self.last_update = Instant::now();
        }
    }

    /// Record message completion
    pub fn record_completion(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

impl LatencyMetrics {
    /// Record message latency
    pub fn record_latency(&mut self, latency: Duration) {
        // Update running average
        self.avg_latency = if self.latency_samples.is_empty() {
            latency
        } else {
            Duration::from_nanos(
                ((self.avg_latency.as_nanos() as f64 * self.latency_samples.len() as f64)
                    + latency.as_nanos() as f64) as u64
                    / (self.latency_samples.len() + 1) as f64 as u64,
            )
        };

        // Update peak latency
        self.peak_latency = self.peak_latency.max(latency);

        // Add to samples
        if self.latency_samples.len() >= 100 {
            self.latency_samples.remove(0);
        }
        self.latency_samples.push(latency);
    }

    /// Get latency percentile
    pub fn get_percentile(&self, percentile: f64) -> Duration {
        if self.latency_samples.is_empty() {
            return Duration::default();
        }

        let mut samples = self.latency_samples.clone();
        samples.sort();

        let index =
            ((samples.len() as f64 * percentile / 100.0).round() as usize).min(samples.len() - 1);

        samples[index]
    }
}

impl ThroughputMetrics {
    /// Record message throughput
    pub fn record_throughput(&mut self, bytes: u64) {
        self.total_bytes += bytes;

        let elapsed = self.last_update.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let seconds = elapsed.as_secs_f64();

            self.messages_per_second = self.total_bytes as f64 / seconds;
            self.bytes_per_second = self.total_bytes as f64 / seconds;

            self.peak_messages_per_second =
                self.peak_messages_per_second.max(self.messages_per_second);

            self.last_update = Instant::now();
        }
    }
}

impl NetworkMetrics {
    /// Record connection attempt
    pub fn record_connection(&mut self, success: bool) {
        if success {
            self.connections += 1;
        } else {
            self.connection_failures += 1;
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.route_cache_hits += 1;
        self.cache_hit_ratio = self.route_cache_hits as f64
            / (self.route_cache_hits + self.connection_failures) as f64;
    }

    /// Record latency
    pub fn record_latency(&mut self, latency: Duration) {
        self.avg_latency = if self.avg_latency.is_zero() {
            latency
        } else {
            (self.avg_latency + latency) / 2
        };
    }

    /// Get metrics summary
    pub fn get_summary(&self) -> NetworkMetricsSummary {
        NetworkMetricsSummary {
            active_connections: self.connections,
            connection_failures: self.connection_failures,
            cache_hit_ratio: self.cache_hit_ratio,
            avg_latency_ms: self.avg_latency.as_millis() as f64,
        }
    }
}

/// Network metrics summary
#[derive(Debug, Clone)]
pub struct NetworkMetricsSummary {
    pub active_connections: usize,
    pub connection_failures: u64,
    pub cache_hit_ratio: f64,
    pub avg_latency_ms: f64,
}
