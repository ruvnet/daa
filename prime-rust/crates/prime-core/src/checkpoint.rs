//! Checkpoint management for DiLoCo training

use crate::error::{Error, Result};
use crate::model::{ModelState, Model};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tch::{nn, Tensor};

/// Training checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique checkpoint identifier
    pub checkpoint_id: String,
    
    /// Global training step
    pub global_step: u64,
    
    /// Local step within DiLoCo round
    pub local_step: u64,
    
    /// Model state
    pub model_state: ModelState,
    
    /// Optimizer state
    pub optimizer_state: OptimizerState,
    
    /// Training metrics
    pub metrics: TrainingMetrics,
    
    /// Checkpoint metadata
    pub metadata: CheckpointMetadata,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(
        checkpoint_id: String,
        global_step: u64,
        local_step: u64,
        model_state: ModelState,
    ) -> Self {
        Self {
            checkpoint_id,
            global_step,
            local_step,
            model_state,
            optimizer_state: OptimizerState::default(),
            metrics: TrainingMetrics::default(),
            metadata: CheckpointMetadata::default(),
        }
    }
    
    /// Save checkpoint to disk
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = bincode::serialize(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    
    /// Load checkpoint from disk
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path)?;
        let checkpoint = bincode::deserialize(&data)?;
        Ok(checkpoint)
    }
    
    /// Create a summary of this checkpoint
    pub fn summary(&self) -> CheckpointSummary {
        CheckpointSummary {
            checkpoint_id: self.checkpoint_id.clone(),
            global_step: self.global_step,
            training_loss: self.metrics.training_loss,
            size_bytes: self.model_state.metadata.size_bytes,
            created_at: self.metadata.created_at,
            storage_path: String::new(), // To be set by manager
        }
    }
}

/// Optimizer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerState {
    /// Optimizer type
    pub optimizer_type: String,
    
    /// Current learning rate
    pub learning_rate: f32,
    
    /// Momentum buffers (for SGD with momentum, Adam, etc.)
    pub momentum_buffers: HashMap<String, Vec<u8>>,
    
    /// Second moment buffers (for Adam, AdamW)
    pub second_moment_buffers: HashMap<String, Vec<u8>>,
    
    /// Step count (for Adam)
    pub step_count: u64,
    
    /// Additional configuration
    pub config: HashMap<String, String>,
}

impl Default for OptimizerState {
    fn default() -> Self {
        Self {
            optimizer_type: "AdamW".to_string(),
            learning_rate: 1e-4,
            momentum_buffers: HashMap::new(),
            second_moment_buffers: HashMap::new(),
            step_count: 0,
            config: HashMap::new(),
        }
    }
}

impl OptimizerState {
    /// Export from PyTorch optimizer
    pub fn from_optimizer(opt: &nn::Optimizer) -> Result<Self> {
        // This is a simplified version - real implementation would
        // need to extract actual optimizer state from PyTorch
        Ok(Self::default())
    }
    
    /// Apply to PyTorch optimizer
    pub fn apply_to_optimizer(&self, opt: &mut nn::Optimizer) -> Result<()> {
        // This is a simplified version - real implementation would
        // need to restore actual optimizer state to PyTorch
        opt.set_lr(self.learning_rate as f64);
        Ok(())
    }
}

/// Training metrics at checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    /// Training loss
    pub training_loss: f32,
    
    /// Validation loss (if available)
    pub validation_loss: Option<f32>,
    
    /// Gradient norm
    pub gradient_norm: f32,
    
    /// Learning rate at this step
    pub learning_rate: f32,
    
    /// Custom metrics
    pub custom_metrics: HashMap<String, f32>,
}

impl Default for TrainingMetrics {
    fn default() -> Self {
        Self {
            training_loss: 0.0,
            validation_loss: None,
            gradient_norm: 0.0,
            learning_rate: 0.0,
            custom_metrics: HashMap::new(),
        }
    }
}

/// Checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Worker that created this checkpoint
    pub worker_id: String,
    
    /// DiLoCo round number
    pub diloco_round: u64,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Training duration in seconds
    pub training_duration: f32,
    
    /// Hardware information
    pub hardware_info: HardwareInfo,
}

impl Default for CheckpointMetadata {
    fn default() -> Self {
        Self {
            worker_id: String::new(),
            diloco_round: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            training_duration: 0.0,
            hardware_info: HardwareInfo::default(),
        }
    }
}

/// Hardware information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// GPU model
    pub gpu_model: Option<String>,
    
    /// Number of GPUs
    pub num_gpus: u32,
    
    /// GPU memory in bytes
    pub gpu_memory: Option<u64>,
    
    /// CPU model
    pub cpu_model: String,
    
    /// RAM in bytes
    pub ram_bytes: u64,
}

impl Default for HardwareInfo {
    fn default() -> Self {
        Self {
            gpu_model: None,
            num_gpus: 0,
            gpu_memory: None,
            cpu_model: String::new(),
            ram_bytes: 0,
        }
    }
}

impl HardwareInfo {
    /// Detect current hardware
    pub fn detect() -> Self {
        let num_gpus = tch::Cuda::device_count() as u32;
        
        let (gpu_model, gpu_memory) = if num_gpus > 0 {
            // Get GPU info from first device
            if let Ok(props) = tch::Cuda::get_device_properties(0) {
                (
                    Some(props.name().to_string()),
                    Some(props.total_memory()),
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        
        Self {
            gpu_model,
            num_gpus,
            gpu_memory,
            cpu_model: "Unknown".to_string(), // Would need platform-specific code
            ram_bytes: 0, // Would need platform-specific code
        }
    }
}

/// Checkpoint summary for quick access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSummary {
    /// Checkpoint ID
    pub checkpoint_id: String,
    
    /// Global step
    pub global_step: u64,
    
    /// Training loss
    pub training_loss: f32,
    
    /// Size in bytes
    pub size_bytes: i64,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Storage path
    pub storage_path: String,
}

/// Checkpoint manager for organizing checkpoints
pub struct CheckpointManager {
    /// Base directory for checkpoints
    pub checkpoint_dir: PathBuf,
    
    /// Maximum checkpoints to keep
    pub max_checkpoints: usize,
    
    /// Checkpoint interval (in global steps)
    pub checkpoint_interval: u64,
    
    /// Checkpoint summaries
    summaries: Vec<CheckpointSummary>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new<P: AsRef<Path>>(checkpoint_dir: P, max_checkpoints: usize) -> Result<Self> {
        let checkpoint_dir = checkpoint_dir.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&checkpoint_dir)?;
        
        let mut manager = Self {
            checkpoint_dir,
            max_checkpoints,
            checkpoint_interval: crate::defaults::CHECKPOINT_INTERVAL,
            summaries: Vec::new(),
        };
        
        // Load existing summaries
        manager.refresh_summaries()?;
        
        Ok(manager)
    }
    
    /// Save a checkpoint
    pub fn save_checkpoint(&mut self, checkpoint: &Checkpoint) -> Result<PathBuf> {
        let filename = format!("checkpoint_{:08}.bin", checkpoint.global_step);
        let path = self.checkpoint_dir.join(&filename);
        
        // Save checkpoint
        checkpoint.save(&path)?;
        
        // Update summary
        let mut summary = checkpoint.summary();
        summary.storage_path = path.to_string_lossy().to_string();
        self.summaries.push(summary);
        
        // Clean up old checkpoints if needed
        self.cleanup_old_checkpoints()?;
        
        Ok(path)
    }
    
    /// Load a checkpoint by ID
    pub fn load_checkpoint(&self, checkpoint_id: &str) -> Result<Checkpoint> {
        let summary = self.summaries
            .iter()
            .find(|s| s.checkpoint_id == checkpoint_id)
            .ok_or_else(|| Error::Checkpoint(format!("Checkpoint not found: {}", checkpoint_id)))?;
        
        Checkpoint::load(&summary.storage_path)
    }
    
    /// Load the latest checkpoint
    pub fn load_latest(&self) -> Result<Option<Checkpoint>> {
        if let Some(summary) = self.summaries.last() {
            Ok(Some(Checkpoint::load(&summary.storage_path)?))
        } else {
            Ok(None)
        }
    }
    
    /// Get checkpoint summaries
    pub fn list_checkpoints(&self) -> &[CheckpointSummary] {
        &self.summaries
    }
    
    /// Check if we should save a checkpoint
    pub fn should_checkpoint(&self, global_step: u64) -> bool {
        global_step % self.checkpoint_interval == 0
    }
    
    /// Refresh summaries from disk
    fn refresh_summaries(&mut self) -> Result<()> {
        self.summaries.clear();
        
        // Read all checkpoint files
        for entry in std::fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                if let Ok(checkpoint) = Checkpoint::load(&path) {
                    let mut summary = checkpoint.summary();
                    summary.storage_path = path.to_string_lossy().to_string();
                    self.summaries.push(summary);
                }
            }
        }
        
        // Sort by global step
        self.summaries.sort_by_key(|s| s.global_step);
        
        Ok(())
    }
    
    /// Clean up old checkpoints
    fn cleanup_old_checkpoints(&mut self) -> Result<()> {
        while self.summaries.len() > self.max_checkpoints {
            if let Some(summary) = self.summaries.first() {
                // Delete the file
                std::fs::remove_file(&summary.storage_path)?;
                
                // Remove from summaries
                self.summaries.remove(0);
            }
        }
        
        Ok(())
    }
}

/// Create a checkpoint from current training state
pub fn create_checkpoint(
    model: &Model,
    optimizer: &nn::Optimizer,
    global_step: u64,
    local_step: u64,
    metrics: TrainingMetrics,
    worker_id: String,
    diloco_round: u64,
) -> Result<Checkpoint> {
    let checkpoint_id = format!("ckpt_{}_{}", worker_id, global_step);
    
    let model_state = model.export_state()?;
    let optimizer_state = OptimizerState::from_optimizer(optimizer)?;
    
    let mut checkpoint = Checkpoint::new(
        checkpoint_id,
        global_step,
        local_step,
        model_state,
    );
    
    checkpoint.optimizer_state = optimizer_state;
    checkpoint.metrics = metrics;
    checkpoint.metadata.worker_id = worker_id;
    checkpoint.metadata.diloco_round = diloco_round;
    checkpoint.metadata.hardware_info = HardwareInfo::detect();
    
    Ok(checkpoint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_checkpoint_save_load() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a checkpoint
        let model_state = ModelState::new("test_model".to_string());
        let checkpoint = Checkpoint::new(
            "test_checkpoint".to_string(),
            100,
            10,
            model_state,
        );
        
        // Save and load
        let path = temp_dir.path().join("checkpoint.bin");
        checkpoint.save(&path).unwrap();
        
        let loaded = Checkpoint::load(&path).unwrap();
        assert_eq!(loaded.checkpoint_id, "test_checkpoint");
        assert_eq!(loaded.global_step, 100);
    }
    
    #[test]
    fn test_checkpoint_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CheckpointManager::new(temp_dir.path(), 3).unwrap();
        
        // Save multiple checkpoints
        for i in 0..5 {
            let model_state = ModelState::new("test_model".to_string());
            let checkpoint = Checkpoint::new(
                format!("checkpoint_{}", i),
                (i * 100) as u64,
                0,
                model_state,
            );
            
            manager.save_checkpoint(&checkpoint).unwrap();
        }
        
        // Should only keep 3 most recent
        assert_eq!(manager.list_checkpoints().len(), 3);
        
        // Check that oldest were removed
        let summaries = manager.list_checkpoints();
        assert_eq!(summaries[0].global_step, 200);
        assert_eq!(summaries[2].global_step, 400);
    }
}