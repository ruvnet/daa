//! QuDAG Consensus Implementation for Model Updates
//! 
//! This module provides consensus mechanisms using QuDAG's QR-Avalanche algorithm
//! for distributed model updates, Byzantine fault-tolerant aggregation, and
//! checkpoint management in the DAA ecosystem.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{info, warn};

use crate::qudag_stubs::qudag_core::Hash;
use crate::{ChainError, Result};

/// Model update types that can be consensused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelUpdate {
    /// Weight update for neural network models
    WeightUpdate {
        model_id: String,
        layer_id: String,
        weights: Vec<f32>,
        gradient_norm: f32,
    },
    /// Architecture change
    ArchitectureUpdate {
        model_id: String,
        change_type: ArchitectureChangeType,
        parameters: HashMap<String, String>,
    },
    /// Hyperparameter adjustment
    HyperparameterUpdate {
        model_id: String,
        hyperparameters: HashMap<String, f64>,
    },
    /// Checkpoint creation
    Checkpoint {
        model_id: String,
        checkpoint_id: String,
        state_hash: Hash,
        metrics: ModelMetrics,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitectureChangeType {
    AddLayer,
    RemoveLayer,
    ModifyLayer,
    ResizeLayer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub accuracy: f64,
    pub loss: f64,
    pub validation_score: f64,
    pub timestamp: u64,
}

/// Byzantine fault-tolerant aggregation for model updates
#[derive(Debug)]
pub struct ByzantineAggregator {
    /// Validator threshold for accepting updates
    validator_threshold: f64,
    /// Maximum allowed deviation from median
    max_deviation: f64,
    /// History of updates for anomaly detection
    update_history: VecDeque<ModelUpdate>,
    /// Known Byzantine nodes
    byzantine_nodes: HashSet<String>,
}

impl ByzantineAggregator {
    pub fn new(validator_threshold: f64, max_deviation: f64) -> Self {
        Self {
            validator_threshold,
            max_deviation,
            update_history: VecDeque::with_capacity(1000),
            byzantine_nodes: HashSet::new(),
        }
    }

    /// Aggregate weight updates with Byzantine fault tolerance
    pub async fn aggregate_weight_updates(
        &mut self,
        updates: Vec<(String, Vec<f32>)>, // (node_id, weights)
    ) -> Result<Vec<f32>> {
        if updates.is_empty() {
            return Err(ChainError::Consensus("No updates to aggregate".to_string()));
        }

        let num_features = updates[0].1.len();
        let mut aggregated_weights = vec![0.0; num_features];
        let mut valid_updates = Vec::new();

        // Filter out Byzantine nodes
        for (node_id, weights) in &updates {
            if self.byzantine_nodes.contains(node_id) {
                continue;
            }

            // Validate weight dimensions
            if weights.len() != num_features {
                warn!("Invalid weight dimensions from node {}", node_id);
                continue;
            }

            // Check for anomalous values
            if self.is_anomalous(&weights) {
                warn!("Anomalous weights detected from node {}", node_id);
                self.byzantine_nodes.insert(node_id.clone());
                continue;
            }

            valid_updates.push((node_id, weights));
        }

        // Check if we have enough valid updates
        let valid_ratio = valid_updates.len() as f64 / updates.len() as f64;
        if valid_ratio < self.validator_threshold {
            return Err(ChainError::Consensus(
                "Insufficient valid updates for consensus".to_string(),
            ));
        }

        // Compute median-based aggregation (more robust than mean)
        for i in 0..num_features {
            let mut feature_values: Vec<f32> = valid_updates
                .iter()
                .map(|(_, weights)| weights[i])
                .collect();
            
            feature_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            // Use trimmed mean (exclude top and bottom 10%)
            let trim_count = valid_updates.len() / 10;
            let trimmed_values = &feature_values[trim_count..feature_values.len() - trim_count];
            
            if !trimmed_values.is_empty() {
                aggregated_weights[i] = trimmed_values.iter().sum::<f32>() / trimmed_values.len() as f32;
            }
        }

        Ok(aggregated_weights)
    }

    /// Check if weights contain anomalous values
    fn is_anomalous(&self, weights: &[f32]) -> bool {
        // Check for NaN or infinite values
        if weights.iter().any(|w| !w.is_finite()) {
            return true;
        }

        // Check for extreme values (beyond reasonable bounds)
        let max_abs_value = 100.0; // Configurable threshold
        if weights.iter().any(|w| w.abs() > max_abs_value) {
            return true;
        }

        // Check variance from historical patterns
        if self.update_history.len() > 10 {
            // Implement statistical anomaly detection here
            // For now, use simple threshold
            let mean_magnitude: f32 = weights.iter().map(|w| w.abs()).sum::<f32>() / weights.len() as f32;
            if mean_magnitude > 10.0 * self.max_deviation as f32 {
                return true;
            }
        }

        false
    }

    /// Mark a node as Byzantine
    pub fn mark_byzantine(&mut self, node_id: String) {
        self.byzantine_nodes.insert(node_id);
    }

    /// Get list of Byzantine nodes
    pub fn get_byzantine_nodes(&self) -> Vec<String> {
        self.byzantine_nodes.iter().cloned().collect()
    }
}

/// Validator node in the consensus network
#[derive(Debug, Clone)]
pub struct ValidatorNode {
    pub id: String,
    pub public_key: Vec<u8>,
    pub stake: u64,
    pub reputation: f64,
    pub last_active: Instant,
    pub validation_count: u64,
    pub correct_validations: u64,
}

impl ValidatorNode {
    pub fn new(id: String, public_key: Vec<u8>, stake: u64) -> Self {
        Self {
            id,
            public_key,
            stake,
            reputation: 1.0,
            last_active: Instant::now(),
            validation_count: 0,
            correct_validations: 0,
        }
    }

    /// Update validator reputation based on validation accuracy
    pub fn update_reputation(&mut self, was_correct: bool) {
        self.validation_count += 1;
        if was_correct {
            self.correct_validations += 1;
        }

        // Calculate reputation as ratio of correct validations with decay
        let accuracy = self.correct_validations as f64 / self.validation_count as f64;
        let time_decay = 0.95; // Small decay factor
        self.reputation = self.reputation * time_decay + accuracy * (1.0 - time_decay);
        self.last_active = Instant::now();
    }

    /// Check if validator is active
    pub fn is_active(&self) -> bool {
        self.last_active.elapsed() < Duration::from_secs(300) // 5 minute timeout
    }
}

/// Validator network for model update verification
#[derive(Debug)]
pub struct ValidatorNetwork {
    validators: Arc<RwLock<HashMap<String, ValidatorNode>>>,
    minimum_validators: usize,
    stake_threshold: u64,
    event_sender: broadcast::Sender<ValidationEvent>,
}

#[derive(Debug, Clone)]
pub enum ValidationEvent {
    ValidatorJoined(String),
    ValidatorLeft(String),
    ValidationCompleted {
        update_id: String,
        validator_id: String,
        result: bool,
    },
}

impl ValidatorNetwork {
    pub fn new(minimum_validators: usize, stake_threshold: u64) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            validators: Arc::new(RwLock::new(HashMap::new())),
            minimum_validators,
            stake_threshold,
            event_sender,
        }
    }

    /// Register a new validator
    pub async fn register_validator(&self, validator: ValidatorNode) -> Result<()> {
        if validator.stake < self.stake_threshold {
            return Err(ChainError::Consensus(
                "Insufficient stake for validator registration".to_string(),
            ));
        }

        let validator_id = validator.id.clone();
        self.validators.write().await.insert(validator_id.clone(), validator);
        
        let _ = self.event_sender.send(ValidationEvent::ValidatorJoined(validator_id));
        Ok(())
    }

    /// Select validators for a round based on stake and reputation
    pub async fn select_validators(&self, count: usize) -> Result<Vec<ValidatorNode>> {
        let validators = self.validators.read().await;
        
        let active_validators: Vec<_> = validators
            .values()
            .filter(|v| v.is_active())
            .cloned()
            .collect();

        if active_validators.len() < self.minimum_validators {
            return Err(ChainError::Consensus(
                "Insufficient active validators".to_string(),
            ));
        }

        // Sort by stake * reputation for weighted selection
        let mut weighted_validators = active_validators;
        weighted_validators.sort_by(|a, b| {
            let a_weight = a.stake as f64 * a.reputation;
            let b_weight = b.stake as f64 * b.reputation;
            b_weight.partial_cmp(&a_weight).unwrap()
        });

        Ok(weighted_validators.into_iter().take(count).collect())
    }

    /// Validate a model update
    pub async fn validate_update(&self, update: &ModelUpdate) -> Result<bool> {
        let validators = self.select_validators(self.minimum_validators).await?;
        let mut positive_votes = 0;
        let mut total_stake = 0;

        for validator in validators {
            // Simulate validation (in real implementation, this would involve
            // cryptographic verification and model testing)
            let is_valid = self.perform_validation(&validator, update).await?;
            
            if is_valid {
                positive_votes += validator.stake;
            }
            total_stake += validator.stake;

            let _ = self.event_sender.send(ValidationEvent::ValidationCompleted {
                update_id: format!("{:?}", update),
                validator_id: validator.id.clone(),
                result: is_valid,
            });
        }

        // Require 2/3 stake majority
        Ok(positive_votes * 3 > total_stake * 2)
    }

    /// Perform actual validation (placeholder for real implementation)
    async fn perform_validation(&self, _validator: &ValidatorNode, update: &ModelUpdate) -> Result<bool> {
        // In a real implementation, this would:
        // 1. Verify cryptographic signatures
        // 2. Test model update impact
        // 3. Check for malicious patterns
        // 4. Validate against consensus rules

        match update {
            ModelUpdate::WeightUpdate { gradient_norm, .. } => {
                // Check gradient norm is reasonable
                Ok(*gradient_norm < 10.0 && *gradient_norm > 0.0)
            }
            ModelUpdate::Checkpoint { metrics, .. } => {
                // Validate metrics are reasonable
                Ok(metrics.accuracy >= 0.0 && metrics.accuracy <= 1.0 &&
                   metrics.loss >= 0.0)
            }
            _ => Ok(true), // Simplified for other types
        }
    }
}

/// Checkpoint consensus manager
#[derive(Debug)]
pub struct CheckpointConsensus {
    checkpoints: Arc<RwLock<HashMap<String, ModelCheckpoint>>>,
    checkpoint_interval: Duration,
    last_checkpoint: Arc<RwLock<Option<Instant>>>,
    finality_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCheckpoint {
    pub id: String,
    pub model_id: String,
    pub block_height: u64,
    pub state_hash: Hash,
    pub metrics: ModelMetrics,
    pub validators: Vec<String>,
    pub timestamp: u64,
    pub parent_checkpoint: Option<String>,
}

impl CheckpointConsensus {
    pub fn new(checkpoint_interval: Duration, finality_threshold: f64) -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
            checkpoint_interval,
            last_checkpoint: Arc::new(RwLock::new(None)),
            finality_threshold,
        }
    }

    /// Create a new checkpoint if interval has passed
    pub async fn maybe_create_checkpoint(
        &self,
        model_id: &str,
        block_height: u64,
        state_hash: Hash,
        metrics: ModelMetrics,
        validators: Vec<String>,
    ) -> Result<Option<ModelCheckpoint>> {
        let mut last_checkpoint = self.last_checkpoint.write().await;
        
        let should_checkpoint = match *last_checkpoint {
            None => true,
            Some(last) => last.elapsed() >= self.checkpoint_interval,
        };

        if !should_checkpoint {
            return Ok(None);
        }

        let checkpoint = ModelCheckpoint {
            id: format!("ckpt_{}_{}", model_id, block_height),
            model_id: model_id.to_string(),
            block_height,
            state_hash,
            metrics,
            validators,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            parent_checkpoint: self.get_latest_checkpoint_id(model_id).await,
        };

        self.checkpoints.write().await.insert(checkpoint.id.clone(), checkpoint.clone());
        *last_checkpoint = Some(Instant::now());

        info!("Created checkpoint {} at height {}", checkpoint.id, block_height);
        Ok(Some(checkpoint))
    }

    /// Get the latest checkpoint for a model
    async fn get_latest_checkpoint_id(&self, model_id: &str) -> Option<String> {
        let checkpoints = self.checkpoints.read().await;
        checkpoints
            .values()
            .filter(|c| c.model_id == model_id)
            .max_by_key(|c| c.block_height)
            .map(|c| c.id.clone())
    }

    /// Verify checkpoint consensus
    pub async fn verify_checkpoint_consensus(
        &self,
        checkpoint_id: &str,
        validator_votes: HashMap<String, bool>,
    ) -> Result<bool> {
        let checkpoints = self.checkpoints.read().await;
        let _checkpoint = checkpoints.get(checkpoint_id)
            .ok_or_else(|| ChainError::Consensus("Checkpoint not found".to_string()))?;

        let positive_votes = validator_votes.values().filter(|&&v| v).count();
        let total_votes = validator_votes.len();

        if total_votes == 0 {
            return Ok(false);
        }

        let consensus_ratio = positive_votes as f64 / total_votes as f64;
        Ok(consensus_ratio >= self.finality_threshold)
    }

    /// Get checkpoint chain for verification
    pub async fn get_checkpoint_chain(&self, checkpoint_id: &str) -> Result<Vec<ModelCheckpoint>> {
        let checkpoints = self.checkpoints.read().await;
        let mut chain = Vec::new();
        let mut current_id = Some(checkpoint_id.to_string());

        while let Some(id) = current_id {
            if let Some(checkpoint) = checkpoints.get(&id) {
                current_id = checkpoint.parent_checkpoint.clone();
                chain.push(checkpoint.clone());
            } else {
                break;
            }
        }

        chain.reverse();
        Ok(chain)
    }
}

/// Rollback and recovery mechanism
#[derive(Debug)]
pub struct RollbackRecovery {
    /// State snapshots for rollback
    snapshots: Arc<RwLock<HashMap<String, StateSnapshot>>>,
    /// Maximum snapshots to keep
    max_snapshots: usize,
    /// Recovery strategies
    recovery_strategies: Vec<Box<dyn RecoveryStrategy>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub id: String,
    pub model_id: String,
    pub block_height: u64,
    pub state_data: Vec<u8>,
    pub timestamp: u64,
    pub validated: bool,
}

#[async_trait::async_trait]
pub trait RecoveryStrategy: Send + Sync + std::fmt::Debug {
    async fn can_recover(&self, error: &ChainError) -> bool;
    async fn recover(&self, snapshot: &StateSnapshot) -> Result<()>;
    fn name(&self) -> &str;
}

/// Simple checkpoint-based recovery strategy
#[derive(Debug)]
pub struct CheckpointRecovery {
    checkpoint_consensus: Arc<CheckpointConsensus>,
}

#[async_trait::async_trait]
impl RecoveryStrategy for CheckpointRecovery {
    async fn can_recover(&self, error: &ChainError) -> bool {
        matches!(error, ChainError::Consensus(_))
    }

    async fn recover(&self, snapshot: &StateSnapshot) -> Result<()> {
        info!("Recovering from checkpoint snapshot {}", snapshot.id);
        // Implement actual state restoration logic
        Ok(())
    }

    fn name(&self) -> &str {
        "CheckpointRecovery"
    }
}

impl RollbackRecovery {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots,
            recovery_strategies: Vec::new(),
        }
    }

    /// Add a recovery strategy
    pub fn add_strategy(&mut self, strategy: Box<dyn RecoveryStrategy>) {
        self.recovery_strategies.push(strategy);
    }

    /// Create a new snapshot
    pub async fn create_snapshot(
        &self,
        model_id: &str,
        block_height: u64,
        state_data: Vec<u8>,
    ) -> Result<String> {
        let snapshot = StateSnapshot {
            id: format!("snap_{}_{}", model_id, block_height),
            model_id: model_id.to_string(),
            block_height,
            state_data,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            validated: false,
        };

        let snapshot_id = snapshot.id.clone();
        let mut snapshots = self.snapshots.write().await;
        
        snapshots.insert(snapshot_id.clone(), snapshot);

        // Cleanup old snapshots if needed
        if snapshots.len() > self.max_snapshots {
            let mut sorted_ids: Vec<_> = snapshots.keys().cloned().collect();
            sorted_ids.sort();
            
            for id in sorted_ids.into_iter().take(snapshots.len() - self.max_snapshots) {
                snapshots.remove(&id);
            }
        }

        Ok(snapshot_id)
    }

    /// Rollback to a specific snapshot
    pub async fn rollback_to_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots.get(snapshot_id)
            .ok_or_else(|| ChainError::Consensus("Snapshot not found".to_string()))?;

        info!("Rolling back to snapshot {} at height {}", snapshot_id, snapshot.block_height);

        // Find appropriate recovery strategy
        for strategy in &self.recovery_strategies {
            if strategy.can_recover(&ChainError::Consensus("Rollback required".to_string())).await {
                strategy.recover(snapshot).await?;
                break;
            }
        }

        Ok(())
    }

    /// Get available snapshots for a model
    pub async fn get_snapshots(&self, model_id: &str) -> Vec<StateSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots
            .values()
            .filter(|s| s.model_id == model_id)
            .cloned()
            .collect()
    }

    /// Validate a snapshot
    pub async fn validate_snapshot(&self, snapshot_id: &str) -> Result<bool> {
        let mut snapshots = self.snapshots.write().await;
        if let Some(snapshot) = snapshots.get_mut(snapshot_id) {
            // Perform validation (placeholder logic)
            snapshot.validated = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Main QuDAG consensus implementation for model updates
#[derive(Debug)]
pub struct QuDAGModelConsensus {
    /// Byzantine aggregator for fault tolerance
    pub aggregator: Arc<Mutex<ByzantineAggregator>>,
    /// Validator network
    pub validator_network: Arc<ValidatorNetwork>,
    /// Checkpoint consensus
    pub checkpoint_consensus: Arc<CheckpointConsensus>,
    /// Rollback recovery
    pub rollback_recovery: Arc<Mutex<RollbackRecovery>>,
    /// QR-Avalanche consensus engine (would use actual QuDAG implementation)
    pub avalanche_config: QRAvalancheConfig,
}

/// QR-Avalanche configuration for model consensus
#[derive(Debug, Clone)]
pub struct QRAvalancheConfig {
    pub beta: f64,
    pub alpha: f64,
    pub query_sample_size: usize,
    pub max_rounds: usize,
    pub finality_threshold: f64,
    pub round_timeout: Duration,
}

impl Default for QRAvalancheConfig {
    fn default() -> Self {
        Self {
            beta: 0.8,
            alpha: 0.6,
            query_sample_size: 20,
            max_rounds: 50,
            finality_threshold: 0.9,
            round_timeout: Duration::from_millis(100),
        }
    }
}

impl QuDAGModelConsensus {
    /// Create a new QuDAG model consensus instance
    pub fn new() -> Self {
        let aggregator = Arc::new(Mutex::new(ByzantineAggregator::new(0.66, 2.0)));
        let validator_network = Arc::new(ValidatorNetwork::new(5, 1000));
        let checkpoint_consensus = Arc::new(CheckpointConsensus::new(
            Duration::from_secs(300), // 5 minute checkpoints
            0.8, // 80% finality threshold
        ));
        let rollback_recovery = Arc::new(Mutex::new(RollbackRecovery::new(10)));

        Self {
            aggregator,
            validator_network,
            checkpoint_consensus,
            rollback_recovery,
            avalanche_config: QRAvalancheConfig::default(),
        }
    }

    /// Process a model update through consensus
    pub async fn process_model_update(&self, update: ModelUpdate) -> Result<bool> {
        // Step 1: Validate update through validator network
        let is_valid = self.validator_network.validate_update(&update).await?;
        
        if !is_valid {
            warn!("Model update failed validation");
            return Ok(false);
        }

        // Step 2: Apply Byzantine fault-tolerant aggregation if needed
        match &update {
            ModelUpdate::WeightUpdate { weights, .. } => {
                // In a real scenario, we'd aggregate multiple weight updates
                // For now, just validate the single update
                let mut aggregator = self.aggregator.lock().await;
                let updates = vec![("node1".to_string(), weights.clone())];
                let _ = aggregator.aggregate_weight_updates(updates).await?;
            }
            _ => {}
        }

        // Step 3: Create checkpoint if needed
        if let ModelUpdate::Checkpoint { model_id, state_hash, metrics, .. } = &update {
            let validators = self.validator_network.select_validators(5).await?;
            let validator_ids: Vec<String> = validators.iter().map(|v| v.id.clone()).collect();
            
            let _ = self.checkpoint_consensus
                .maybe_create_checkpoint(
                    model_id,
                    0, // Block height would come from chain
                    *state_hash,
                    metrics.clone(),
                    validator_ids,
                )
                .await?;
        }

        Ok(true)
    }

    /// Recover from a consensus failure
    pub async fn recover_from_failure(&self, model_id: &str) -> Result<()> {
        // Get available snapshots
        let recovery = self.rollback_recovery.lock().await;
        let snapshots = recovery.get_snapshots(model_id).await;
        
        if let Some(latest_snapshot) = snapshots.last() {
            info!("Recovering from snapshot {}", latest_snapshot.id);
            recovery.rollback_to_snapshot(&latest_snapshot.id).await?;
        } else {
            return Err(ChainError::Consensus("No snapshots available for recovery".to_string()));
        }

        Ok(())
    }

    /// Get consensus metrics
    pub async fn get_metrics(&self) -> ConsensusMetrics {
        ConsensusMetrics {
            total_validators: self.validator_network.validators.read().await.len(),
            byzantine_nodes: self.aggregator.lock().await.get_byzantine_nodes().len(),
            checkpoints_created: self.checkpoint_consensus.checkpoints.read().await.len(),
            snapshots_available: self.rollback_recovery.lock().await.snapshots.read().await.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    pub total_validators: usize,
    pub byzantine_nodes: usize,
    pub checkpoints_created: usize,
    pub snapshots_available: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_byzantine_aggregation() {
        let mut aggregator = ByzantineAggregator::new(0.66, 2.0);
        
        let updates = vec![
            ("node1".to_string(), vec![1.0, 2.0, 3.0]),
            ("node2".to_string(), vec![1.1, 2.1, 3.1]),
            ("node3".to_string(), vec![100.0, 200.0, 300.0]), // Byzantine
            ("node4".to_string(), vec![1.2, 1.9, 3.2]),
        ];

        let result = aggregator.aggregate_weight_updates(updates).await;
        assert!(result.is_ok());
        
        let weights = result.unwrap();
        assert!(weights[0] > 0.9 && weights[0] < 1.3); // Should be around 1.1
    }

    #[tokio::test]
    async fn test_validator_network() {
        let network = ValidatorNetwork::new(3, 100);
        
        let validator1 = ValidatorNode::new("val1".to_string(), vec![1, 2, 3], 1000);
        let validator2 = ValidatorNode::new("val2".to_string(), vec![4, 5, 6], 2000);
        let validator3 = ValidatorNode::new("val3".to_string(), vec![7, 8, 9], 1500);
        
        network.register_validator(validator1).await.unwrap();
        network.register_validator(validator2).await.unwrap();
        network.register_validator(validator3).await.unwrap();
        
        let selected = network.select_validators(2).await.unwrap();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, "val2"); // Highest stake
    }

    #[tokio::test]
    async fn test_checkpoint_creation() {
        let consensus = CheckpointConsensus::new(Duration::from_secs(0), 0.8);
        
        let metrics = ModelMetrics {
            accuracy: 0.95,
            loss: 0.05,
            validation_score: 0.93,
            timestamp: 0,
        };
        
        let checkpoint = consensus
            .maybe_create_checkpoint(
                "model1",
                100,
                Hash::new([0; 32]),
                metrics,
                vec!["val1".to_string()],
            )
            .await
            .unwrap();
        
        assert!(checkpoint.is_some());
        assert_eq!(checkpoint.unwrap().model_id, "model1");
    }

    #[tokio::test]
    async fn test_rollback_recovery() {
        let recovery = RollbackRecovery::new(5);
        
        let snapshot_id = recovery
            .create_snapshot("model1", 100, vec![1, 2, 3, 4])
            .await
            .unwrap();
        
        assert!(snapshot_id.contains("snap_model1_100"));
        
        let snapshots = recovery.get_snapshots("model1").await;
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].model_id, "model1");
    }
}