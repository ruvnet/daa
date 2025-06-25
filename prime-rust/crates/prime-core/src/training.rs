//! Core training abstractions for DiLoCo algorithm

use crate::error::{Error, Result};
use crate::gradient::{Gradient, GradientBatch, CompressionAlgorithm};
use crate::model::{Model, ModelState, ModelDelta};
use crate::checkpoint::{Checkpoint, CheckpointManager, TrainingMetrics};
use crate::compression::{GradientCompressor, Int8Compressor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tch::{nn, Device, Tensor};
use tokio::sync::mpsc;

/// DiLoCo training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Batch size for local training
    pub batch_size: usize,
    
    /// Learning rate
    pub learning_rate: f32,
    
    /// Number of local steps per DiLoCo round
    pub local_steps: u32,
    
    /// Gradient accumulation steps
    pub gradient_accumulation_steps: u32,
    
    /// Maximum gradient norm for clipping
    pub max_grad_norm: f32,
    
    /// Whether to use mixed precision training
    pub mixed_precision: bool,
    
    /// Compression algorithm
    pub compression_algorithm: CompressionAlgorithm,
    
    /// Device for training
    pub device: Device,
    
    /// Additional configuration
    pub extra_config: HashMap<String, String>,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            learning_rate: 1e-4,
            local_steps: crate::defaults::LOCAL_STEPS_PER_ROUND,
            gradient_accumulation_steps: crate::defaults::GRADIENT_ACCUMULATION_STEPS,
            max_grad_norm: 1.0,
            mixed_precision: false,
            compression_algorithm: CompressionAlgorithm::Int8Quantization,
            device: Device::cuda_if_available(),
            extra_config: HashMap::new(),
        }
    }
}

/// DiLoCo trainer state
pub struct DiLoCoTrainer {
    /// Model being trained
    pub model: Model,
    
    /// Optimizer
    pub optimizer: nn::Optimizer,
    
    /// Training configuration
    pub config: TrainingConfig,
    
    /// Current global step
    pub global_step: u64,
    
    /// Current local step within round
    pub local_step: u64,
    
    /// Current DiLoCo round
    pub diloco_round: u64,
    
    /// Worker ID
    pub worker_id: String,
    
    /// Gradient compressor
    compressor: Box<dyn GradientCompressor + Send + Sync>,
    
    /// Checkpoint manager
    checkpoint_manager: Option<CheckpointManager>,
    
    /// Accumulated gradients
    accumulated_gradients: HashMap<String, Tensor>,
    
    /// Training metrics
    metrics: TrainingMetrics,
}

impl DiLoCoTrainer {
    /// Create a new DiLoCo trainer
    pub fn new(
        model: Model,
        config: TrainingConfig,
        worker_id: String,
    ) -> Result<Self> {
        // Create optimizer
        let optimizer = nn::Adam::default()
            .build(&model.vs, config.learning_rate as f64)?;
        
        // Create compressor
        let compressor: Box<dyn GradientCompressor + Send + Sync> = match config.compression_algorithm {
            CompressionAlgorithm::Int8Quantization => Box::new(Int8Compressor::default()),
            CompressionAlgorithm::None => Box::new(NoOpCompressor),
        };
        
        Ok(Self {
            model,
            optimizer,
            config,
            global_step: 0,
            local_step: 0,
            diloco_round: 0,
            worker_id,
            compressor,
            checkpoint_manager: None,
            accumulated_gradients: HashMap::new(),
            metrics: TrainingMetrics::default(),
        })
    }
    
    /// Set checkpoint manager
    pub fn set_checkpoint_manager(&mut self, manager: CheckpointManager) {
        self.checkpoint_manager = Some(manager);
    }
    
    /// Perform a local training step
    pub fn local_step(&mut self, batch: &TrainingBatch) -> Result<StepMetrics> {
        let _guard = tch::no_grad_guard();
        
        // Forward pass
        let logits = self.forward(batch)?;
        
        // Compute loss
        let loss = self.compute_loss(&logits, batch)?;
        
        // Backward pass
        self.optimizer.zero_grad();
        loss.backward();
        
        // Accumulate gradients if needed
        if self.config.gradient_accumulation_steps > 1 {
            self.accumulate_gradients()?;
        }
        
        // Apply gradients every N steps
        if (self.local_step + 1) % self.config.gradient_accumulation_steps as u64 == 0 {
            self.apply_accumulated_gradients()?;
            
            // Gradient clipping
            self.clip_gradients()?;
            
            // Optimizer step
            self.optimizer.step();
        }
        
        // Update metrics
        let step_metrics = StepMetrics {
            loss: loss.double_value(&[]) as f32,
            learning_rate: self.config.learning_rate,
            gradient_norm: self.calculate_gradient_norm()?,
        };
        
        self.metrics.training_loss = step_metrics.loss;
        self.metrics.gradient_norm = step_metrics.gradient_norm;
        self.metrics.learning_rate = step_metrics.learning_rate;
        
        // Update counters
        self.local_step += 1;
        self.global_step += 1;
        
        Ok(step_metrics)
    }
    
    /// Complete a DiLoCo round and prepare gradients for communication
    pub fn complete_round(&mut self) -> Result<GradientBatch> {
        // Create gradient batch
        let mut batch = GradientBatch::new(
            format!("round_{}_worker_{}", self.diloco_round, self.worker_id),
            self.global_step,
            self.worker_id.clone(),
        );
        
        // Compress and add all gradients
        for (name, tensor) in self.model.vs.variables() {
            if tensor.requires_grad() {
                let gradient = Gradient::new(name.clone(), tensor.grad().shallow_clone());
                let compressed = self.compressor.compress(&gradient)?;
                batch.add_gradient(compressed);
            }
        }
        
        // Reset local step counter
        self.local_step = 0;
        self.diloco_round += 1;
        
        // Save checkpoint if needed
        if let Some(ref mut manager) = self.checkpoint_manager {
            if manager.should_checkpoint(self.global_step) {
                self.save_checkpoint(manager)?;
            }
        }
        
        Ok(batch)
    }
    
    /// Apply gradient updates from other workers
    pub fn apply_gradient_updates(&mut self, updates: Vec<GradientBatch>) -> Result<()> {
        // Average gradients from all workers
        let mut gradient_sums: HashMap<String, Tensor> = HashMap::new();
        let num_workers = updates.len() as f64;
        
        for batch in updates {
            for compressed in batch.gradients {
                let gradient = self.compressor.decompress(&compressed, self.config.device)?;
                
                gradient_sums.entry(gradient.layer_id.clone())
                    .and_modify(|sum| *sum = sum + &gradient.tensor)
                    .or_insert(gradient.tensor);
            }
        }
        
        // Apply averaged gradients to model
        for (name, avg_gradient) in gradient_sums {
            if let Some((_, var)) = self.model.vs.variables().find(|(n, _)| n == &name) {
                let scaled_gradient = avg_gradient / num_workers;
                var.set_grad(&scaled_gradient);
            }
        }
        
        // Optimizer step with aggregated gradients
        self.optimizer.step();
        
        Ok(())
    }
    
    /// Load from checkpoint
    pub fn load_checkpoint(&mut self, checkpoint: &Checkpoint) -> Result<()> {
        // Load model state
        self.model.import_state(&checkpoint.model_state)?;
        
        // Load optimizer state
        checkpoint.optimizer_state.apply_to_optimizer(&mut self.optimizer)?;
        
        // Update training state
        self.global_step = checkpoint.global_step;
        self.local_step = checkpoint.local_step;
        self.diloco_round = checkpoint.metadata.diloco_round;
        self.metrics = checkpoint.metrics.clone();
        
        Ok(())
    }
    
    /// Forward pass (to be implemented by specific models)
    fn forward(&self, batch: &TrainingBatch) -> Result<Tensor> {
        // This would be implemented by specific model architectures
        Err(Error::NotImplemented("Forward pass not implemented".to_string()))
    }
    
    /// Compute loss (to be implemented by specific tasks)
    fn compute_loss(&self, logits: &Tensor, batch: &TrainingBatch) -> Result<Tensor> {
        // This would be implemented by specific training tasks
        Err(Error::NotImplemented("Loss computation not implemented".to_string()))
    }
    
    /// Accumulate gradients for gradient accumulation
    fn accumulate_gradients(&mut self) -> Result<()> {
        for (name, var) in self.model.vs.variables() {
            if let Some(grad) = var.grad_opt() {
                self.accumulated_gradients
                    .entry(name.clone())
                    .and_modify(|acc| *acc = acc + grad)
                    .or_insert_with(|| grad.shallow_clone());
            }
        }
        Ok(())
    }
    
    /// Apply accumulated gradients
    fn apply_accumulated_gradients(&mut self) -> Result<()> {
        let accumulation_steps = self.config.gradient_accumulation_steps as f64;
        
        for (name, acc_grad) in &self.accumulated_gradients {
            if let Some((_, var)) = self.model.vs.variables().find(|(n, _)| n == name) {
                let scaled_grad = acc_grad / accumulation_steps;
                var.set_grad(&scaled_grad);
            }
        }
        
        self.accumulated_gradients.clear();
        Ok(())
    }
    
    /// Clip gradients by norm
    fn clip_gradients(&mut self) -> Result<()> {
        let total_norm = self.calculate_gradient_norm()?;
        
        if total_norm > self.config.max_grad_norm {
            let scale = self.config.max_grad_norm / total_norm;
            
            for (_, var) in self.model.vs.variables() {
                if let Some(grad) = var.grad_opt() {
                    let clipped = grad * scale as f64;
                    var.set_grad(&clipped);
                }
            }
        }
        
        Ok(())
    }
    
    /// Calculate gradient norm
    fn calculate_gradient_norm(&self) -> Result<f32> {
        let mut total_norm = 0.0;
        
        for (_, var) in self.model.vs.variables() {
            if let Some(grad) = var.grad_opt() {
                let norm = grad.norm().double_value(&[]) as f32;
                total_norm += norm * norm;
            }
        }
        
        Ok(total_norm.sqrt())
    }
    
    /// Save checkpoint
    fn save_checkpoint(&self, manager: &mut CheckpointManager) -> Result<()> {
        let checkpoint = crate::checkpoint::create_checkpoint(
            &self.model,
            &self.optimizer,
            self.global_step,
            self.local_step,
            self.metrics.clone(),
            self.worker_id.clone(),
            self.diloco_round,
        )?;
        
        manager.save_checkpoint(&checkpoint)?;
        Ok(())
    }
}

/// Training batch data
#[derive(Debug, Clone)]
pub struct TrainingBatch {
    /// Input tensors
    pub inputs: HashMap<String, Tensor>,
    
    /// Target tensors
    pub targets: HashMap<String, Tensor>,
    
    /// Batch size
    pub batch_size: usize,
}

/// Metrics for a single training step
#[derive(Debug, Clone)]
pub struct StepMetrics {
    /// Training loss
    pub loss: f32,
    
    /// Current learning rate
    pub learning_rate: f32,
    
    /// Gradient norm
    pub gradient_norm: f32,
}

/// No-op compressor for uncompressed gradients
struct NoOpCompressor;

impl GradientCompressor for NoOpCompressor {
    fn compress(&self, gradient: &Gradient) -> Result<CompressedGradient> {
        gradient.compress(CompressionAlgorithm::None)
    }
    
    fn decompress(&self, compressed: &CompressedGradient, device: Device) -> Result<Gradient> {
        compressed.decompress(device)
    }
    
    fn algorithm(&self) -> CompressionAlgorithm {
        CompressionAlgorithm::None
    }
}

/// DiLoCo round coordinator
pub struct RoundCoordinator {
    /// Number of workers
    num_workers: usize,
    
    /// Received gradient batches
    received_batches: Vec<GradientBatch>,
    
    /// Channel for receiving batches
    batch_receiver: mpsc::Receiver<GradientBatch>,
    
    /// Channel for sending aggregated updates
    update_sender: mpsc::Sender<Vec<GradientBatch>>,
}

impl RoundCoordinator {
    /// Create a new round coordinator
    pub fn new(
        num_workers: usize,
        batch_receiver: mpsc::Receiver<GradientBatch>,
        update_sender: mpsc::Sender<Vec<GradientBatch>>,
    ) -> Self {
        Self {
            num_workers,
            received_batches: Vec::new(),
            batch_receiver,
            update_sender,
        }
    }
    
    /// Coordinate a DiLoCo round
    pub async fn coordinate_round(&mut self) -> Result<()> {
        // Collect gradient batches from all workers
        self.received_batches.clear();
        
        for _ in 0..self.num_workers {
            match self.batch_receiver.recv().await {
                Some(batch) => self.received_batches.push(batch),
                None => return Err(Error::Network("Channel closed".to_string())),
            }
        }
        
        // Send aggregated batches to all workers
        for _ in 0..self.num_workers {
            self.update_sender.send(self.received_batches.clone()).await
                .map_err(|_| Error::Network("Failed to send updates".to_string()))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_training_config() {
        let config = TrainingConfig::default();
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.local_steps, 100);
        assert_eq!(config.compression_algorithm, CompressionAlgorithm::Int8Quantization);
    }
    
    #[test]
    fn test_step_metrics() {
        let metrics = StepMetrics {
            loss: 0.5,
            learning_rate: 1e-4,
            gradient_norm: 1.2,
        };
        
        assert_eq!(metrics.loss, 0.5);
        assert_eq!(metrics.learning_rate, 1e-4);
        assert_eq!(metrics.gradient_norm, 1.2);
    }
}