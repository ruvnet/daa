use super::{Gradient, ModelInterface, TrainingMetrics};
use crate::DiLoCoConfig;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration, Instant};
use tracing::{debug, info, warn};

/// Local trainer that runs epochs without communication
pub struct LocalTrainer {
    config: DiLoCoConfig,
    node_id: String,
    optimizer_state: Arc<RwLock<OptimizerState>>,
    metrics_aggregator: MetricsAggregator,
}

#[derive(Clone)]
struct OptimizerState {
    // Adam optimizer state
    momentum: Vec<f32>,
    variance: Vec<f32>,
    beta1: f32,
    beta2: f32,
    epsilon: f32,
    learning_rate: f32,
    step: u64,
}

impl Default for OptimizerState {
    fn default() -> Self {
        Self {
            momentum: Vec::new(),
            variance: Vec::new(),
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            learning_rate: 0.001,
            step: 0,
        }
    }
}

struct MetricsAggregator {
    window_size: usize,
    loss_history: Vec<f32>,
    accuracy_history: Vec<f32>,
}

impl LocalTrainer {
    pub fn new(config: DiLoCoConfig, node_id: String) -> Self {
        Self {
            config,
            node_id,
            optimizer_state: Arc::new(RwLock::new(OptimizerState::default())),
            metrics_aggregator: MetricsAggregator {
                window_size: 100,
                loss_history: Vec::new(),
                accuracy_history: Vec::new(),
            },
        }
    }

    /// Run local training epochs
    pub async fn train_local_epochs(
        &self,
        model: Arc<RwLock<dyn ModelInterface>>,
        data_loader: Arc<dyn DataLoader>,
        num_epochs: usize,
        metrics_tx: mpsc::Sender<TrainingMetrics>,
    ) -> anyhow::Result<LocalTrainingResult> {
        info!("Starting {} local epochs on node {}", num_epochs, self.node_id);
        
        let start_time = Instant::now();
        let mut total_batches = 0;
        let mut accumulated_gradient = None;
        
        for epoch in 0..num_epochs {
            let epoch_start = Instant::now();
            let mut epoch_loss = 0.0;
            let mut epoch_accuracy = 0.0;
            let mut batches_in_epoch = 0;
            
            // Train for one epoch
            loop {
                match data_loader.next_batch().await {
                    Ok(batch) => {
                        // Process batch
                        let metrics = self.process_batch(
                            model.clone(),
                            batch,
                            &mut accumulated_gradient,
                        ).await?;
                        
                        epoch_loss += metrics.loss;
                        epoch_accuracy += metrics.accuracy;
                        batches_in_epoch += 1;
                        total_batches += 1;
                        
                        // Send metrics periodically
                        if total_batches % 10 == 0 {
                            let _ = metrics_tx.send(metrics).await;
                        }
                    }
                    Err(e) if e.to_string().contains("epoch complete") => {
                        // End of epoch
                        break;
                    }
                    Err(e) => {
                        warn!("Error getting batch: {}", e);
                        break;
                    }
                }
                
                // Yield periodically to prevent blocking
                if batches_in_epoch % 50 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            
            // Log epoch statistics
            if batches_in_epoch > 0 {
                let avg_loss = epoch_loss / batches_in_epoch as f32;
                let avg_accuracy = epoch_accuracy / batches_in_epoch as f32;
                
                info!(
                    "Epoch {}/{} completed in {:?} - Loss: {:.4}, Accuracy: {:.2}%",
                    epoch + 1,
                    num_epochs,
                    epoch_start.elapsed(),
                    avg_loss,
                    avg_accuracy * 100.0
                );
            }
            
            // Reset data loader for next epoch
            data_loader.reset().await?;
        }
        
        let training_duration = start_time.elapsed();
        
        Ok(LocalTrainingResult {
            total_batches,
            training_duration,
            final_gradient: accumulated_gradient,
            average_loss: self.metrics_aggregator.average_loss(),
            average_accuracy: self.metrics_aggregator.average_accuracy(),
        })
    }

    /// Process a single batch
    async fn process_batch(
        &self,
        model: Arc<RwLock<dyn ModelInterface>>,
        batch: DataBatch,
        accumulated_gradient: &mut Option<Gradient>,
    ) -> anyhow::Result<TrainingMetrics> {
        let mut model_guard = model.write().await;
        
        // Forward pass
        let output = model_guard.forward(&batch.data);
        
        // Calculate loss
        let loss = self.calculate_loss(&output, &batch.labels);
        
        // Backward pass
        let mut gradient = model_guard.backward(loss);
        
        // Apply optimizer (Adam)
        self.apply_optimizer(&mut gradient).await?;
        
        // Apply gradient to model
        model_guard.apply_gradient(&gradient);
        
        // Accumulate gradient for later sync
        match accumulated_gradient {
            Some(ref mut acc) => {
                // Add to accumulated gradient
                for (i, val) in gradient.values.iter().enumerate() {
                    if i < acc.values.len() {
                        acc.values[i] += val;
                    }
                }
            }
            None => {
                *accumulated_gradient = Some(gradient.clone());
            }
        }
        
        // Calculate metrics
        let accuracy = self.calculate_accuracy(&output, &batch.labels);
        let grad_norm = self.calculate_gradient_norm(&gradient);
        
        // Update metrics history
        self.metrics_aggregator.update(loss, accuracy);
        
        Ok(TrainingMetrics {
            loss,
            accuracy,
            gradients_norm: grad_norm,
            communication_bytes: 0, // No communication during local training
        })
    }

    /// Apply Adam optimizer to gradient
    async fn apply_optimizer(&self, gradient: &mut Gradient) -> anyhow::Result<()> {
        let mut opt_state = self.optimizer_state.write().await;
        
        // Initialize optimizer state if needed
        if opt_state.momentum.len() != gradient.values.len() {
            opt_state.momentum = vec![0.0; gradient.values.len()];
            opt_state.variance = vec![0.0; gradient.values.len()];
        }
        
        opt_state.step += 1;
        let lr = opt_state.learning_rate;
        let beta1 = opt_state.beta1;
        let beta2 = opt_state.beta2;
        let epsilon = opt_state.epsilon;
        let step = opt_state.step as f32;
        
        // Bias correction
        let bias_correction1 = 1.0 - beta1.powf(step);
        let bias_correction2 = 1.0 - beta2.powf(step);
        
        // Apply Adam update
        for i in 0..gradient.values.len() {
            // Update biased first moment estimate
            opt_state.momentum[i] = beta1 * opt_state.momentum[i] + (1.0 - beta1) * gradient.values[i];
            
            // Update biased second raw moment estimate
            opt_state.variance[i] = beta2 * opt_state.variance[i] + (1.0 - beta2) * gradient.values[i].powi(2);
            
            // Compute bias-corrected first moment estimate
            let m_hat = opt_state.momentum[i] / bias_correction1;
            
            // Compute bias-corrected second raw moment estimate
            let v_hat = opt_state.variance[i] / bias_correction2;
            
            // Update gradient with Adam rule
            gradient.values[i] = lr * m_hat / (v_hat.sqrt() + epsilon);
        }
        
        if self.config.differential_privacy {
            self.add_dp_noise(gradient).await?;
        }
        
        Ok(())
    }

    /// Add differential privacy noise
    async fn add_dp_noise(&self, gradient: &mut Gradient) -> anyhow::Result<()> {
        use rand::distributions::{Distribution, Normal};
        use rand::thread_rng;
        
        let noise_scale = 1.0 / self.config.dp_epsilon;
        let normal = Normal::new(0.0, noise_scale)?;
        let mut rng = thread_rng();
        
        for value in &mut gradient.values {
            *value += normal.sample(&mut rng) as f32;
        }
        
        Ok(())
    }

    fn calculate_loss(&self, output: &[f32], labels: &[f32]) -> f32 {
        // Cross-entropy loss for classification
        let mut loss = 0.0;
        for (i, (out, label)) in output.iter().zip(labels.iter()).enumerate() {
            let epsilon = 1e-7; // Prevent log(0)
            loss -= label * (out.max(epsilon).ln());
        }
        loss / output.len() as f32
    }

    fn calculate_accuracy(&self, output: &[f32], labels: &[f32]) -> f32 {
        let correct = output.iter()
            .zip(labels.iter())
            .filter(|(out, label)| {
                // For classification: check if argmax matches
                (out.round() - label.round()).abs() < f32::EPSILON
            })
            .count();
        correct as f32 / output.len() as f32
    }

    fn calculate_gradient_norm(&self, gradient: &Gradient) -> f32 {
        gradient.values.iter()
            .map(|v| v.powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

impl MetricsAggregator {
    fn update(&mut self, loss: f32, accuracy: f32) {
        self.loss_history.push(loss);
        self.accuracy_history.push(accuracy);
        
        // Keep only recent history
        if self.loss_history.len() > self.window_size {
            self.loss_history.remove(0);
            self.accuracy_history.remove(0);
        }
    }

    fn average_loss(&self) -> f32 {
        if self.loss_history.is_empty() {
            0.0
        } else {
            self.loss_history.iter().sum::<f32>() / self.loss_history.len() as f32
        }
    }

    fn average_accuracy(&self) -> f32 {
        if self.accuracy_history.is_empty() {
            0.0
        } else {
            self.accuracy_history.iter().sum::<f32>() / self.accuracy_history.len() as f32
        }
    }
}

#[async_trait::async_trait]
pub trait DataLoader: Send + Sync {
    async fn next_batch(&self) -> anyhow::Result<DataBatch>;
    async fn reset(&self) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct DataBatch {
    pub data: Vec<f32>,
    pub labels: Vec<f32>,
}

#[derive(Debug)]
pub struct LocalTrainingResult {
    pub total_batches: usize,
    pub training_duration: Duration,
    pub final_gradient: Option<Gradient>,
    pub average_loss: f32,
    pub average_accuracy: f32,
}