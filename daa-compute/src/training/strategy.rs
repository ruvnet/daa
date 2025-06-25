use super::{ModelInterface, TrainingMetrics};
use crate::{DiLoCoConfig, ElasticDeviceMesh, FederatedSGD, GradientAggregator, RoundCoordinator};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

pub struct TrainingStrategy {
    config: DiLoCoConfig,
    mesh: Arc<RwLock<ElasticDeviceMesh>>,
    federated_sgd: Arc<FederatedSGD>,
    aggregator: Arc<GradientAggregator>,
    coordinator: Arc<RoundCoordinator>,
}

impl TrainingStrategy {
    pub async fn new(config: DiLoCoConfig) -> anyhow::Result<Self> {
        let mesh = Arc::new(RwLock::new(ElasticDeviceMesh::new().await?));
        let federated_sgd = Arc::new(FederatedSGD::new(config.clone()).await?);
        let aggregator = Arc::new(GradientAggregator::new(config.gradient_compression).await?);
        let coordinator = Arc::new(RoundCoordinator::new(mesh.clone()).await?);

        Ok(Self {
            config,
            mesh,
            federated_sgd,
            aggregator,
            coordinator,
        })
    }

    /// Main training loop implementing DiLoCo-style distributed training
    pub async fn train(
        &self,
        model: Arc<RwLock<dyn ModelInterface>>,
        data_loader: Arc<dyn DataLoader>,
    ) -> anyhow::Result<()> {
        info!("Starting DiLoCo distributed training with {} local epochs", self.config.local_epochs);
        
        let (tx_metrics, mut rx_metrics) = mpsc::channel::<TrainingMetrics>(100);
        
        // Spawn metrics aggregation task
        let metrics_handle = tokio::spawn(async move {
            while let Some(metrics) = rx_metrics.recv().await {
                info!(
                    "Training metrics - Loss: {:.4}, Accuracy: {:.2}%, Gradients norm: {:.4}",
                    metrics.loss, metrics.accuracy * 100.0, metrics.gradients_norm
                );
            }
        });

        let mut round = 0u64;
        let mut sync_timer = interval(Duration::from_secs(self.config.max_local_time_minutes * 60));
        
        loop {
            // Check if we should sync based on time or epochs
            let should_sync = tokio::select! {
                _ = sync_timer.tick() => {
                    info!("Time-based sync triggered after {} minutes", self.config.max_local_time_minutes);
                    true
                }
                result = self.run_local_epochs(model.clone(), data_loader.clone(), &tx_metrics) => {
                    match result {
                        Ok(_) => {
                            info!("Completed {} local epochs", self.config.local_epochs);
                            true
                        }
                        Err(e) => {
                            error!("Local training error: {}", e);
                            false
                        }
                    }
                }
            };

            if should_sync {
                // Perform global synchronization
                match self.perform_global_sync(model.clone(), round).await {
                    Ok(comm_bytes) => {
                        let reduction_factor = self.calculate_communication_reduction(comm_bytes);
                        info!(
                            "Global sync completed for round {}. Communication reduction: {}x",
                            round, reduction_factor
                        );
                        round += 1;
                    }
                    Err(e) => {
                        error!("Global sync failed: {}", e);
                        // Continue training with local model
                    }
                }
            }

            // Check for dynamic node changes
            self.handle_elastic_membership().await?;
        }
    }

    /// Run local training epochs without communication
    async fn run_local_epochs(
        &self,
        model: Arc<RwLock<dyn ModelInterface>>,
        data_loader: Arc<dyn DataLoader>,
        metrics_tx: &mpsc::Sender<TrainingMetrics>,
    ) -> anyhow::Result<()> {
        let mut local_steps = 0;
        
        while local_steps < self.config.local_epochs {
            // Get next batch
            let batch = data_loader.next_batch().await?;
            
            // Forward pass
            let mut model_guard = model.write().await;
            let output = model_guard.forward(&batch.data);
            
            // Calculate loss (simplified)
            let loss = self.calculate_loss(&output, &batch.labels);
            
            // Backward pass
            let gradient = model_guard.backward(loss);
            
            // Apply gradient locally
            model_guard.apply_gradient(&gradient);
            
            // Send metrics
            let metrics = TrainingMetrics {
                loss,
                accuracy: self.calculate_accuracy(&output, &batch.labels),
                gradients_norm: self.calculate_gradient_norm(&gradient),
                communication_bytes: 0, // No communication during local training
            };
            
            let _ = metrics_tx.send(metrics).await;
            
            local_steps += 1;
            
            // Yield to allow other tasks to run
            if local_steps % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        Ok(())
    }

    /// Perform global synchronization using federated SGD
    async fn perform_global_sync(
        &self,
        model: Arc<RwLock<dyn ModelInterface>>,
        round: u64,
    ) -> anyhow::Result<u64> {
        info!("Starting global synchronization for round {}", round);
        
        // Get current model parameters
        let current_params = {
            let model_guard = model.read().await;
            model_guard.get_parameters()
        };
        
        // Perform federated averaging through coordinator
        let (aggregated_params, comm_bytes) = self.coordinator
            .coordinate_round(round, current_params, &self.aggregator)
            .await?;
        
        // Update local model with aggregated parameters
        {
            let mut model_guard = model.write().await;
            model_guard.set_parameters(aggregated_params);
        }
        
        Ok(comm_bytes)
    }

    /// Handle elastic membership changes
    async fn handle_elastic_membership(&self) -> anyhow::Result<()> {
        let mut mesh_guard = self.mesh.write().await;
        
        // Check for new nodes
        let new_nodes = mesh_guard.check_new_nodes().await?;
        for node in new_nodes {
            info!("New node joined: {}", node.id);
            mesh_guard.add_node(node).await?;
        }
        
        // Check for failed nodes
        let failed_nodes = mesh_guard.check_failed_nodes().await?;
        for node_id in failed_nodes {
            warn!("Node failed: {}", node_id);
            mesh_guard.remove_node(&node_id).await?;
        }
        
        Ok(())
    }

    /// Calculate communication reduction factor
    fn calculate_communication_reduction(&self, actual_bytes: u64) -> usize {
        // Estimate what full synchronous training would use
        let full_sync_bytes = self.estimate_full_sync_bytes();
        (full_sync_bytes / actual_bytes.max(1)) as usize
    }

    fn estimate_full_sync_bytes(&self) -> u64 {
        // Assume 1GB model, sync every step = local_epochs * 1GB
        let model_size_bytes = 1_000_000_000u64; // 1GB
        model_size_bytes * self.config.local_epochs as u64
    }

    fn calculate_loss(&self, output: &[f32], labels: &[f32]) -> f32 {
        // Simplified MSE loss
        output.iter()
            .zip(labels.iter())
            .map(|(o, l)| (o - l).powi(2))
            .sum::<f32>() / output.len() as f32
    }

    fn calculate_accuracy(&self, output: &[f32], labels: &[f32]) -> f32 {
        // Simplified accuracy calculation
        let correct = output.iter()
            .zip(labels.iter())
            .filter(|(o, l)| (o.round() - l.round()).abs() < f32::EPSILON)
            .count();
        correct as f32 / output.len() as f32
    }

    fn calculate_gradient_norm(&self, gradient: &super::Gradient) -> f32 {
        gradient.values.iter()
            .map(|v| v.powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

#[async_trait::async_trait]
pub trait DataLoader: Send + Sync {
    async fn next_batch(&self) -> anyhow::Result<DataBatch>;
}

#[derive(Debug)]
pub struct DataBatch {
    pub data: Vec<f32>,
    pub labels: Vec<f32>,
}