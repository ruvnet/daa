use crate::{DiLoCoConfig, training::{Gradient, ModelParameters}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Federated SGD implementation with periodic synchronization
pub struct FederatedSGD {
    config: DiLoCoConfig,
    node_id: String,
    peer_updates: Arc<RwLock<HashMap<String, PeerUpdate>>>,
    sync_state: Arc<Mutex<SyncState>>,
}

#[derive(Clone, Debug)]
struct PeerUpdate {
    gradient: Gradient,
    timestamp: std::time::Instant,
    verified: bool,
}

#[derive(Debug)]
struct SyncState {
    current_round: u64,
    participants: Vec<String>,
    sync_in_progress: bool,
    last_sync_time: std::time::Instant,
}

impl FederatedSGD {
    pub async fn new(config: DiLoCoConfig) -> anyhow::Result<Self> {
        let node_id = uuid::Uuid::new_v4().to_string();
        
        Ok(Self {
            config,
            node_id,
            peer_updates: Arc::new(RwLock::new(HashMap::new())),
            sync_state: Arc::new(Mutex::new(SyncState {
                current_round: 0,
                participants: vec![],
                sync_in_progress: false,
                last_sync_time: std::time::Instant::now(),
            })),
        })
    }

    /// Perform federated averaging of gradients
    pub async fn federated_average(
        &self,
        local_gradient: Gradient,
        peer_gradients: Vec<Gradient>,
    ) -> anyhow::Result<Gradient> {
        info!("Performing federated averaging with {} peers", peer_gradients.len());
        
        // Add local gradient to the mix
        let mut all_gradients = vec![local_gradient.clone()];
        all_gradients.extend(peer_gradients);
        
        // Validate gradients
        let valid_gradients = self.validate_gradients(all_gradients).await?;
        
        // Apply differential privacy if enabled
        let protected_gradients = if self.config.differential_privacy {
            self.apply_differential_privacy(valid_gradients).await?
        } else {
            valid_gradients
        };
        
        // Perform weighted averaging
        let averaged = self.weighted_average(protected_gradients).await?;
        
        Ok(averaged)
    }

    /// Validate gradients to detect anomalies or attacks
    async fn validate_gradients(&self, gradients: Vec<Gradient>) -> anyhow::Result<Vec<Gradient>> {
        let mut valid_gradients = Vec::new();
        
        // Calculate median norm for anomaly detection
        let norms: Vec<f32> = gradients.iter()
            .map(|g| g.values.iter().map(|v| v.powi(2)).sum::<f32>().sqrt())
            .collect();
        
        let median_norm = self.calculate_median(&norms);
        let threshold = median_norm * 3.0; // 3x median as outlier threshold
        
        for (grad, norm) in gradients.into_iter().zip(norms.iter()) {
            if *norm <= threshold {
                valid_gradients.push(grad);
            } else {
                warn!(
                    "Rejecting gradient from {} with norm {} (threshold: {})",
                    grad.node_id, norm, threshold
                );
            }
        }
        
        if valid_gradients.len() < 2 {
            return Err(anyhow::anyhow!("Too few valid gradients for averaging"));
        }
        
        Ok(valid_gradients)
    }

    /// Apply differential privacy noise to gradients
    async fn apply_differential_privacy(
        &self,
        gradients: Vec<Gradient>,
    ) -> anyhow::Result<Vec<Gradient>> {
        use rand::distributions::{Distribution, Normal};
        use rand::thread_rng;
        
        let noise_scale = 1.0 / self.config.dp_epsilon;
        let normal = Normal::new(0.0, noise_scale)?;
        let mut rng = thread_rng();
        
        let mut protected_gradients = Vec::new();
        
        for mut grad in gradients {
            // Add Gaussian noise to each gradient component
            for value in &mut grad.values {
                *value += normal.sample(&mut rng) as f32;
            }
            protected_gradients.push(grad);
        }
        
        debug!("Applied DP noise with epsilon={}", self.config.dp_epsilon);
        Ok(protected_gradients)
    }

    /// Perform weighted average of gradients
    async fn weighted_average(&self, gradients: Vec<Gradient>) -> anyhow::Result<Gradient> {
        if gradients.is_empty() {
            return Err(anyhow::anyhow!("No gradients to average"));
        }
        
        let grad_len = gradients[0].values.len();
        let mut averaged_values = vec![0.0f32; grad_len];
        let num_gradients = gradients.len() as f32;
        
        // Simple average (could be weighted by contribution or reliability)
        for grad in &gradients {
            for (i, value) in grad.values.iter().enumerate() {
                averaged_values[i] += value / num_gradients;
            }
        }
        
        Ok(Gradient {
            values: averaged_values,
            node_id: self.node_id.clone(),
            round: gradients[0].round,
            compressed: false,
        })
    }

    /// Initiate a synchronization round
    pub async fn initiate_sync_round(&self, round: u64) -> anyhow::Result<()> {
        let mut sync_state = self.sync_state.lock().await;
        
        if sync_state.sync_in_progress {
            return Err(anyhow::anyhow!("Sync already in progress"));
        }
        
        sync_state.sync_in_progress = true;
        sync_state.current_round = round;
        sync_state.last_sync_time = std::time::Instant::now();
        
        info!("Initiated sync round {}", round);
        Ok(())
    }

    /// Complete a synchronization round
    pub async fn complete_sync_round(&self) -> anyhow::Result<()> {
        let mut sync_state = self.sync_state.lock().await;
        sync_state.sync_in_progress = false;
        
        let sync_duration = sync_state.last_sync_time.elapsed();
        info!(
            "Completed sync round {} in {:?}",
            sync_state.current_round, sync_duration
        );
        
        // Clear old peer updates
        let mut peer_updates = self.peer_updates.write().await;
        peer_updates.clear();
        
        Ok(())
    }

    /// Handle asynchronous gradient updates from peers
    pub async fn receive_peer_update(&self, gradient: Gradient) -> anyhow::Result<()> {
        let mut peer_updates = self.peer_updates.write().await;
        
        peer_updates.insert(
            gradient.node_id.clone(),
            PeerUpdate {
                gradient,
                timestamp: std::time::Instant::now(),
                verified: false, // Will be verified during aggregation
            },
        );
        
        Ok(())
    }

    /// Get current peer updates for aggregation
    pub async fn get_peer_updates(&self, timeout_secs: u64) -> Vec<Gradient> {
        // Wait for updates with timeout
        let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
        
        while std::time::Instant::now() < deadline {
            let updates = self.peer_updates.read().await;
            if updates.len() >= 2 { // Minimum peers for aggregation
                break;
            }
            drop(updates);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        let updates = self.peer_updates.read().await;
        updates.values()
            .map(|update| update.gradient.clone())
            .collect()
    }

    fn calculate_median(&self, values: &[f32]) -> f32 {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Compress gradient for network transmission
    pub async fn compress_gradient(&self, gradient: &mut Gradient) -> anyhow::Result<u64> {
        if self.config.gradient_compression == 0 {
            return Ok(gradient.values.len() as u64 * 4); // f32 = 4 bytes
        }
        
        let original_size = gradient.values.len() * 4;
        
        // Quantization-based compression (simplified)
        let compression_factor = self.config.gradient_compression as f32 / 10.0;
        let quantization_levels = (256.0 * compression_factor) as i32;
        
        // Find min/max for quantization
        let min_val = gradient.values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = gradient.values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let range = max_val - min_val;
        
        if range > 0.0 {
            // Quantize values
            for value in &mut gradient.values {
                let normalized = (*value - min_val) / range;
                let quantized = (normalized * quantization_levels as f32).round() as i32;
                *value = min_val + (quantized as f32 / quantization_levels as f32) * range;
            }
        }
        
        gradient.compressed = true;
        
        // Estimate compressed size (simplified)
        let bits_per_value = (quantization_levels as f32).log2().ceil() as u64;
        let compressed_size = (gradient.values.len() as u64 * bits_per_value) / 8;
        
        debug!(
            "Compressed gradient from {} bytes to {} bytes ({}x reduction)",
            original_size,
            compressed_size,
            original_size / compressed_size.max(1)
        );
        
        Ok(compressed_size)
    }
}