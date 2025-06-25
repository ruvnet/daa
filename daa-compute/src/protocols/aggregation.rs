use crate::training::{Gradient, ModelParameters};
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Gradient aggregation protocol supporting various strategies
pub struct GradientAggregator {
    compression_level: u8,
    aggregation_strategy: AggregationStrategy,
    verification_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum AggregationStrategy {
    /// Simple averaging of all gradients
    Average,
    /// Weighted average based on data contribution
    WeightedAverage(HashMap<String, f32>),
    /// Trimmed mean (remove outliers)
    TrimmedMean(f32), // Percentage to trim from each end
    /// Median-based aggregation
    Median,
    /// Byzantine-robust aggregation (Krum algorithm)
    Krum(usize), // Number of Byzantine nodes to tolerate
}

impl Default for AggregationStrategy {
    fn default() -> Self {
        AggregationStrategy::TrimmedMean(0.1) // Trim 10% from each end
    }
}

impl GradientAggregator {
    pub async fn new(compression_level: u8) -> anyhow::Result<Self> {
        Ok(Self {
            compression_level,
            aggregation_strategy: AggregationStrategy::default(),
            verification_enabled: true,
        })
    }

    /// Set aggregation strategy
    pub fn set_strategy(&mut self, strategy: AggregationStrategy) {
        self.aggregation_strategy = strategy;
    }

    /// Aggregate gradients from multiple nodes
    pub async fn aggregate(
        &self,
        gradients: Vec<Gradient>,
        round: u64,
    ) -> anyhow::Result<(Gradient, u64)> {
        if gradients.is_empty() {
            return Err(anyhow::anyhow!("No gradients to aggregate"));
        }

        info!(
            "Aggregating {} gradients for round {} using {:?} strategy",
            gradients.len(),
            round,
            self.aggregation_strategy
        );

        // Verify gradients if enabled
        let verified_gradients = if self.verification_enabled {
            self.verify_gradients(gradients).await?
        } else {
            gradients
        };

        // Decompress gradients if needed
        let decompressed = self.decompress_gradients(verified_gradients).await?;

        // Apply aggregation strategy
        let aggregated = match &self.aggregation_strategy {
            AggregationStrategy::Average => self.average_gradients(decompressed).await?,
            AggregationStrategy::WeightedAverage(weights) => {
                self.weighted_average_gradients(decompressed, weights).await?
            }
            AggregationStrategy::TrimmedMean(trim_pct) => {
                self.trimmed_mean_gradients(decompressed, *trim_pct).await?
            }
            AggregationStrategy::Median => self.median_gradients(decompressed).await?,
            AggregationStrategy::Krum(f) => self.krum_aggregation(decompressed, *f).await?,
        };

        // Calculate communication bytes
        let comm_bytes = self.calculate_communication_bytes(&aggregated);

        Ok((aggregated, comm_bytes))
    }

    /// Verify gradients for validity and security
    async fn verify_gradients(&self, gradients: Vec<Gradient>) -> anyhow::Result<Vec<Gradient>> {
        let mut verified = Vec::new();
        
        for grad in gradients {
            // Check basic validity
            if grad.values.is_empty() {
                warn!("Skipping empty gradient from {}", grad.node_id);
                continue;
            }

            // Check for NaN or Inf values
            if grad.values.iter().any(|v| !v.is_finite()) {
                warn!("Skipping gradient with non-finite values from {}", grad.node_id);
                continue;
            }

            // Check gradient norm bounds
            let norm = grad.values.iter().map(|v| v.powi(2)).sum::<f32>().sqrt();
            if norm > 1e6 {
                warn!("Skipping gradient with excessive norm {} from {}", norm, grad.node_id);
                continue;
            }

            verified.push(grad);
        }

        if verified.len() < gradients.len() / 2 {
            return Err(anyhow::anyhow!("Too many gradients failed verification"));
        }

        Ok(verified)
    }

    /// Decompress gradients if compressed
    async fn decompress_gradients(&self, gradients: Vec<Gradient>) -> anyhow::Result<Vec<Gradient>> {
        // In a real implementation, this would reverse the compression
        // For now, we just return as-is since compression is simplified
        Ok(gradients)
    }

    /// Simple average aggregation
    async fn average_gradients(&self, gradients: Vec<Gradient>) -> anyhow::Result<Gradient> {
        let grad_len = gradients[0].values.len();
        let mut sum = vec![0.0f32; grad_len];
        let count = gradients.len() as f32;

        for grad in &gradients {
            for (i, value) in grad.values.iter().enumerate() {
                sum[i] += value;
            }
        }

        for value in &mut sum {
            *value /= count;
        }

        Ok(Gradient {
            values: sum,
            node_id: "aggregator".to_string(),
            round: gradients[0].round,
            compressed: false,
        })
    }

    /// Weighted average aggregation
    async fn weighted_average_gradients(
        &self,
        gradients: Vec<Gradient>,
        weights: &HashMap<String, f32>,
    ) -> anyhow::Result<Gradient> {
        let grad_len = gradients[0].values.len();
        let mut weighted_sum = vec![0.0f32; grad_len];
        let mut total_weight = 0.0f32;

        for grad in &gradients {
            let weight = weights.get(&grad.node_id).unwrap_or(&1.0);
            total_weight += weight;

            for (i, value) in grad.values.iter().enumerate() {
                weighted_sum[i] += value * weight;
            }
        }

        if total_weight > 0.0 {
            for value in &mut weighted_sum {
                *value /= total_weight;
            }
        }

        Ok(Gradient {
            values: weighted_sum,
            node_id: "aggregator".to_string(),
            round: gradients[0].round,
            compressed: false,
        })
    }

    /// Trimmed mean aggregation (remove outliers)
    async fn trimmed_mean_gradients(
        &self,
        gradients: Vec<Gradient>,
        trim_pct: f32,
    ) -> anyhow::Result<Gradient> {
        let grad_len = gradients[0].values.len();
        let mut result = vec![0.0f32; grad_len];

        let trim_count = ((gradients.len() as f32 * trim_pct) as usize).max(1);
        let keep_count = gradients.len().saturating_sub(2 * trim_count);

        if keep_count == 0 {
            return Err(anyhow::anyhow!("All gradients trimmed"));
        }

        // For each gradient dimension
        for i in 0..grad_len {
            let mut values: Vec<f32> = gradients.iter().map(|g| g.values[i]).collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Calculate trimmed mean
            let sum: f32 = values[trim_count..values.len() - trim_count].iter().sum();
            result[i] = sum / keep_count as f32;
        }

        Ok(Gradient {
            values: result,
            node_id: "aggregator".to_string(),
            round: gradients[0].round,
            compressed: false,
        })
    }

    /// Median aggregation
    async fn median_gradients(&self, gradients: Vec<Gradient>) -> anyhow::Result<Gradient> {
        let grad_len = gradients[0].values.len();
        let mut result = vec![0.0f32; grad_len];

        for i in 0..grad_len {
            let mut values: Vec<f32> = gradients.iter().map(|g| g.values[i]).collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let mid = values.len() / 2;
            result[i] = if values.len() % 2 == 0 {
                (values[mid - 1] + values[mid]) / 2.0
            } else {
                values[mid]
            };
        }

        Ok(Gradient {
            values: result,
            node_id: "aggregator".to_string(),
            round: gradients[0].round,
            compressed: false,
        })
    }

    /// Krum aggregation (Byzantine-robust)
    async fn krum_aggregation(
        &self,
        gradients: Vec<Gradient>,
        f: usize,
    ) -> anyhow::Result<Gradient> {
        let n = gradients.len();
        if n <= 2 * f + 2 {
            return Err(anyhow::anyhow!(
                "Not enough gradients for Krum with f={} Byzantine nodes",
                f
            ));
        }

        // Calculate pairwise distances
        let mut scores = vec![0.0f32; n];
        for i in 0..n {
            let mut distances: Vec<f32> = Vec::new();
            
            for j in 0..n {
                if i != j {
                    let dist = self.gradient_distance(&gradients[i], &gradients[j]);
                    distances.push(dist);
                }
            }
            
            // Sort distances and sum the n-f-2 smallest
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
            scores[i] = distances[..n - f - 2].iter().sum();
        }

        // Select gradient with minimum score
        let best_idx = scores
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();

        Ok(gradients[best_idx].clone())
    }

    /// Calculate L2 distance between two gradients
    fn gradient_distance(&self, g1: &Gradient, g2: &Gradient) -> f32 {
        g1.values
            .iter()
            .zip(g2.values.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Calculate communication bytes for metrics
    fn calculate_communication_bytes(&self, gradient: &Gradient) -> u64 {
        let base_size = gradient.values.len() * 4; // f32 = 4 bytes
        
        if gradient.compressed && self.compression_level > 0 {
            // Estimate compressed size based on compression level
            let compression_ratio = 1.0 - (self.compression_level as f32 / 10.0) * 0.8;
            (base_size as f32 * compression_ratio) as u64
        } else {
            base_size as u64
        }
    }

    /// Parallel aggregation for large-scale scenarios
    pub async fn parallel_aggregate(
        &self,
        gradient_batches: Vec<Vec<Gradient>>,
        round: u64,
    ) -> anyhow::Result<Gradient> {
        let mut futures = FuturesUnordered::new();

        // Aggregate each batch in parallel
        for batch in gradient_batches {
            let aggregator = self.clone();
            futures.push(async move {
                aggregator.aggregate(batch, round).await
            });
        }

        // Collect results
        let mut intermediate_results = Vec::new();
        while let Some(result) = futures.next().await {
            match result {
                Ok((grad, _)) => intermediate_results.push(grad),
                Err(e) => error!("Batch aggregation failed: {}", e),
            }
        }

        // Final aggregation of intermediate results
        let (final_gradient, _) = self.aggregate(intermediate_results, round).await?;
        Ok(final_gradient)
    }
}

impl Clone for GradientAggregator {
    fn clone(&self) -> Self {
        Self {
            compression_level: self.compression_level,
            aggregation_strategy: self.aggregation_strategy.clone(),
            verification_enabled: self.verification_enabled,
        }
    }
}