//! Adaptive batching for optimal message throughput and latency

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Adaptive batching configuration
#[derive(Clone, Debug)]
pub struct BatchConfig {
    /// Minimum batch size before forcing flush
    pub min_batch_size: usize,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Base latency target in microseconds
    pub base_latency_micros: u64,
    /// Maximum acceptable latency in microseconds
    pub max_latency_micros: u64,
    /// Load factor threshold for aggressive batching
    pub load_threshold: f64,
    /// Smoothing factor for exponential moving average
    pub smoothing_factor: f64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            min_batch_size: 1,
            max_batch_size: 1000,
            base_latency_micros: 100, // 100 microseconds base target
            max_latency_micros: 1000, // 1 millisecond max
            load_threshold: 0.7,      // 70% load threshold
            smoothing_factor: 0.1,    // 10% smoothing
        }
    }
}

/// Adaptive message batcher that optimizes for both throughput and latency
pub struct AdaptiveBatcher<T> {
    /// Configuration parameters
    config: BatchConfig,
    /// Current batch of messages
    current_batch: VecDeque<T>,
    /// When the current batch was started
    batch_start_time: Instant,
    /// Queue pressure metrics
    queue_metrics: QueuePressureMetrics,
    /// Adaptive thresholds
    adaptive_thresholds: AdaptiveThresholds,
    /// Performance history for learning
    performance_history: PerformanceHistory,
}

/// Queue pressure tracking
#[derive(Clone, Debug)]
struct QueuePressureMetrics {
    current_queue_length: usize,
    max_queue_length: usize,
    average_queue_length: f64,
    queue_growth_rate: f64,
    last_update: Instant,
}

/// Adaptive threshold calculations
#[derive(Clone, Debug)]
struct AdaptiveThresholds {
    current_latency_target: Duration,
    current_batch_size_target: usize,
    load_factor: f64,
    #[allow(dead_code)]
    pressure_coefficient: f64,
}

/// Performance history for machine learning-like optimization
#[derive(Debug)]
struct PerformanceHistory {
    /// Recent latency measurements
    latency_samples: VecDeque<Duration>,
    /// Recent throughput measurements
    throughput_samples: VecDeque<f64>,
    /// Batch size vs performance correlation
    batch_performance_map: std::collections::HashMap<usize, f64>,
    /// Maximum samples to keep
    max_samples: usize,
}

impl<T> AdaptiveBatcher<T> {
    /// Create a new adaptive batcher
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config: config.clone(),
            current_batch: VecDeque::with_capacity(config.max_batch_size),
            batch_start_time: Instant::now(),
            queue_metrics: QueuePressureMetrics {
                current_queue_length: 0,
                max_queue_length: config.max_batch_size * 10,
                average_queue_length: 0.0,
                queue_growth_rate: 0.0,
                last_update: Instant::now(),
            },
            adaptive_thresholds: AdaptiveThresholds {
                current_latency_target: Duration::from_micros(config.base_latency_micros),
                current_batch_size_target: config.min_batch_size,
                load_factor: 0.0,
                pressure_coefficient: 1.0,
            },
            performance_history: PerformanceHistory {
                latency_samples: VecDeque::with_capacity(1000),
                throughput_samples: VecDeque::with_capacity(1000),
                batch_performance_map: std::collections::HashMap::new(),
                max_samples: 1000,
            },
        }
    }

    /// Add a message to the current batch
    pub fn add_message(&mut self, message: T) -> Option<Vec<T>> {
        self.current_batch.push_back(message);
        self.update_queue_metrics();

        if self.should_flush() {
            Some(self.flush_batch())
        } else {
            None
        }
    }

    /// Force flush the current batch
    pub fn flush_batch(&mut self) -> Vec<T> {
        let batch_size = self.current_batch.len();
        let batch_latency = self.batch_start_time.elapsed();

        // Record performance metrics
        self.record_batch_performance(batch_size, batch_latency);

        // Extract batch
        let batch: Vec<T> = self.current_batch.drain(..).collect();

        // Reset for next batch
        self.batch_start_time = Instant::now();
        self.update_adaptive_thresholds();

        batch
    }

    /// Check if the current batch should be flushed
    pub fn should_flush(&self) -> bool {
        let current_latency = self.batch_start_time.elapsed();
        let batch_size = self.current_batch.len();

        // Hard limits
        if batch_size >= self.config.max_batch_size {
            return true;
        }

        if current_latency >= Duration::from_micros(self.config.max_latency_micros) {
            return true;
        }

        // Adaptive thresholds
        if batch_size >= self.adaptive_thresholds.current_batch_size_target {
            return true;
        }

        if current_latency >= self.adaptive_thresholds.current_latency_target {
            return true;
        }

        // Queue pressure-based flushing
        if self.adaptive_thresholds.load_factor > self.config.load_threshold {
            // Under high load, flush more aggressively
            let pressure_adjusted_latency = Duration::from_micros(
                (self.config.base_latency_micros as f64
                    * (1.0 - self.adaptive_thresholds.load_factor * 0.5)) as u64,
            );

            if current_latency >= pressure_adjusted_latency {
                return true;
            }
        }

        false
    }

    /// Update queue pressure metrics
    fn update_queue_metrics(&mut self) {
        let now = Instant::now();
        let time_delta = now
            .duration_since(self.queue_metrics.last_update)
            .as_secs_f64();

        if time_delta > 0.0 {
            let previous_length = self.queue_metrics.current_queue_length;
            self.queue_metrics.current_queue_length = self.current_batch.len();

            // Calculate growth rate
            let length_delta =
                self.queue_metrics.current_queue_length as f64 - previous_length as f64;
            self.queue_metrics.queue_growth_rate = length_delta / time_delta;

            // Update running average
            let new_length = self.queue_metrics.current_queue_length as f64;
            self.queue_metrics.average_queue_length = self.queue_metrics.average_queue_length
                * (1.0 - self.config.smoothing_factor)
                + new_length * self.config.smoothing_factor;

            // Calculate load factor
            self.adaptive_thresholds.load_factor = self.queue_metrics.current_queue_length as f64
                / self.queue_metrics.max_queue_length as f64;

            self.queue_metrics.last_update = now;
        }
    }

    /// Record batch performance for learning
    fn record_batch_performance(&mut self, batch_size: usize, latency: Duration) {
        // Record latency sample
        if self.performance_history.latency_samples.len() >= self.performance_history.max_samples {
            self.performance_history.latency_samples.pop_front();
        }
        self.performance_history.latency_samples.push_back(latency);

        // Calculate throughput
        let throughput = if latency.as_secs_f64() > 0.0 {
            batch_size as f64 / latency.as_secs_f64()
        } else {
            0.0
        };

        // Record throughput sample
        if self.performance_history.throughput_samples.len() >= self.performance_history.max_samples
        {
            self.performance_history.throughput_samples.pop_front();
        }
        self.performance_history
            .throughput_samples
            .push_back(throughput);

        // Update batch size performance mapping
        let current_score = self
            .performance_history
            .batch_performance_map
            .get(&batch_size)
            .copied()
            .unwrap_or(0.0);

        // Use a weighted average to update performance score
        let latency_score = 1.0 / (latency.as_secs_f64() + 0.001); // Inverse latency
        let new_score = current_score * 0.9 + latency_score * 0.1;

        self.performance_history
            .batch_performance_map
            .insert(batch_size, new_score);
    }

    /// Update adaptive thresholds based on performance history
    fn update_adaptive_thresholds(&mut self) {
        // Calculate average latency and throughput
        let avg_latency = self.calculate_average_latency();
        let _avg_throughput = self.calculate_average_throughput();

        // Adjust latency target based on current performance
        if avg_latency > self.adaptive_thresholds.current_latency_target {
            // Performance is worse than target, be more aggressive
            self.adaptive_thresholds.current_latency_target = Duration::from_micros(
                (self.adaptive_thresholds.current_latency_target.as_micros() as f64 * 0.9) as u64,
            );
        } else {
            // Performance is good, can be more relaxed
            self.adaptive_thresholds.current_latency_target = Duration::from_micros(
                (self.adaptive_thresholds.current_latency_target.as_micros() as f64 * 1.05) as u64,
            );
        }

        // Clamp latency target to configured bounds
        self.adaptive_thresholds.current_latency_target = self
            .adaptive_thresholds
            .current_latency_target
            .max(Duration::from_micros(self.config.base_latency_micros / 10))
            .min(Duration::from_micros(self.config.max_latency_micros));

        // Adjust batch size target based on load and performance
        let optimal_batch_size = self.find_optimal_batch_size();
        self.adaptive_thresholds.current_batch_size_target = optimal_batch_size
            .max(self.config.min_batch_size)
            .min(self.config.max_batch_size);
    }

    /// Find optimal batch size based on performance history
    fn find_optimal_batch_size(&self) -> usize {
        if self.performance_history.batch_performance_map.is_empty() {
            return self.config.min_batch_size;
        }

        // Find batch size with best performance score
        let (best_batch_size, _score) = self
            .performance_history
            .batch_performance_map
            .iter()
            .max_by(|(_, score_a), (_, score_b)| score_a.partial_cmp(score_b).unwrap())
            .unwrap();

        *best_batch_size
    }

    /// Calculate average latency from recent samples
    fn calculate_average_latency(&self) -> Duration {
        if self.performance_history.latency_samples.is_empty() {
            return Duration::from_micros(self.config.base_latency_micros);
        }

        let total_nanos: u128 = self
            .performance_history
            .latency_samples
            .iter()
            .map(|d| d.as_nanos())
            .sum();

        Duration::from_nanos(
            (total_nanos / self.performance_history.latency_samples.len() as u128) as u64,
        )
    }

    /// Calculate average throughput from recent samples
    fn calculate_average_throughput(&self) -> f64 {
        if self.performance_history.throughput_samples.is_empty() {
            return 0.0;
        }

        self.performance_history
            .throughput_samples
            .iter()
            .sum::<f64>()
            / self.performance_history.throughput_samples.len() as f64
    }

    /// Get current batch size
    pub fn current_batch_size(&self) -> usize {
        self.current_batch.len()
    }

    /// Get current batch age
    pub fn current_batch_age(&self) -> Duration {
        self.batch_start_time.elapsed()
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> BatcherStats {
        BatcherStats {
            current_batch_size: self.current_batch.len(),
            current_batch_age: self.batch_start_time.elapsed(),
            queue_load_factor: self.adaptive_thresholds.load_factor,
            adaptive_latency_target: self.adaptive_thresholds.current_latency_target,
            adaptive_batch_size_target: self.adaptive_thresholds.current_batch_size_target,
            average_latency: self.calculate_average_latency(),
            average_throughput: self.calculate_average_throughput(),
            total_samples: self.performance_history.latency_samples.len(),
        }
    }

    /// Reset performance history
    pub fn reset_history(&mut self) {
        self.performance_history.latency_samples.clear();
        self.performance_history.throughput_samples.clear();
        self.performance_history.batch_performance_map.clear();
    }
}

/// Statistics for monitoring batcher performance
#[derive(Debug, Clone)]
pub struct BatcherStats {
    pub current_batch_size: usize,
    pub current_batch_age: Duration,
    pub queue_load_factor: f64,
    pub adaptive_latency_target: Duration,
    pub adaptive_batch_size_target: usize,
    pub average_latency: Duration,
    pub average_throughput: f64,
    pub total_samples: usize,
}

impl std::fmt::Display for BatcherStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Batcher Stats: batch_size={}, age={:?}, load={:.2}%, target_latency={:?}, target_batch={}, avg_latency={:?}, throughput={:.1}/s",
            self.current_batch_size,
            self.current_batch_age,
            self.queue_load_factor * 100.0,
            self.adaptive_latency_target,
            self.adaptive_batch_size_target,
            self.average_latency,
            self.average_throughput
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_batcher_basic() {
        let config = BatchConfig::default();
        let mut batcher = AdaptiveBatcher::new(config);

        // Add messages
        assert!(batcher.add_message("msg1").is_none());
        assert!(batcher.add_message("msg2").is_none());
        assert!(batcher.add_message("msg3").is_none());

        // Force flush
        let batch = batcher.flush_batch();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch, vec!["msg1", "msg2", "msg3"]);
    }

    #[test]
    fn test_adaptive_batcher_size_limit() {
        let config = BatchConfig {
            max_batch_size: 2,
            ..Default::default()
        };
        let mut batcher = AdaptiveBatcher::new(config);

        // Add messages up to limit
        assert!(batcher.add_message("msg1").is_none());
        let batch = batcher.add_message("msg2").unwrap();

        assert_eq!(batch.len(), 2);
        assert_eq!(batch, vec!["msg1", "msg2"]);
    }

    #[test]
    fn test_adaptive_batcher_time_limit() {
        let config = BatchConfig {
            max_latency_micros: 1, // Very short timeout
            ..Default::default()
        };
        let mut batcher = AdaptiveBatcher::new(config);

        batcher.add_message("msg1");

        // Wait a bit
        std::thread::sleep(Duration::from_millis(1));

        // Should flush due to timeout
        let batch = batcher.add_message("msg2").unwrap();
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_batcher_stats() {
        let config = BatchConfig::default();
        let mut batcher = AdaptiveBatcher::new(config);

        batcher.add_message("msg1");
        batcher.add_message("msg2");

        let stats = batcher.get_stats();
        assert_eq!(stats.current_batch_size, 2);
        assert!(stats.current_batch_age > Duration::ZERO);
    }

    #[test]
    fn test_performance_learning() {
        let config = BatchConfig::default();
        let mut batcher = AdaptiveBatcher::new(config);

        // Simulate multiple batches with different sizes
        for batch_size in [1, 5, 10, 20] {
            for _ in 0..batch_size {
                batcher.add_message("msg");
            }
            batcher.flush_batch();
        }

        let stats = batcher.get_stats();
        assert!(stats.total_samples > 0);
        assert!(stats.average_throughput >= 0.0);
    }

    #[test]
    fn test_queue_pressure_adaptation() {
        let config = BatchConfig {
            load_threshold: 0.5,
            ..Default::default()
        };
        let mut batcher = AdaptiveBatcher::new(config);

        // Fill up to create pressure
        for i in 0..100 {
            batcher.add_message(format!("msg{}", i));
        }

        let stats = batcher.get_stats();
        assert!(stats.queue_load_factor > 0.0);
    }
}
