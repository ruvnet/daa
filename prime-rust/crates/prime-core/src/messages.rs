//! Message handling for DiLoCo protocol

use crate::error::{Error, Result};
use crate::gradient::GradientBatch;
use crate::model::{ModelState, ModelDelta};
use crate::checkpoint::{Checkpoint, CheckpointSummary};
use crate::training::TrainingConfig;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// DiLoCo message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Worker registration
    WorkerRegister,
    /// Worker heartbeat
    WorkerHeartbeat,
    /// Gradient update from worker
    GradientUpdate,
    /// Model synchronization
    ModelSync,
    /// Checkpoint operation
    CheckpointOp,
    /// Training control
    TrainingControl,
    /// Round completion
    RoundComplete,
    /// Error message
    Error,
}

/// Base message wrapper for DiLoCo protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiLoCoMessage {
    /// Message type
    pub message_type: MessageType,
    
    /// Unique message ID
    pub message_id: String,
    
    /// Sender ID
    pub sender_id: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Message payload
    pub payload: MessagePayload,
}

impl DiLoCoMessage {
    /// Create a new message
    pub fn new(
        message_type: MessageType,
        sender_id: String,
        payload: MessagePayload,
    ) -> Self {
        let message_id = format!("{}_{}", sender_id, uuid::Uuid::new_v4());
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            message_type,
            message_id,
            sender_id,
            timestamp,
            payload,
        }
    }
    
    /// Create a worker registration message
    pub fn worker_register(
        sender_id: String,
        capabilities: WorkerCapabilities,
        model_version: String,
    ) -> Self {
        Self::new(
            MessageType::WorkerRegister,
            sender_id,
            MessagePayload::WorkerRegister(WorkerRegister {
                capabilities,
                model_version,
                config: Default::default(),
            }),
        )
    }
    
    /// Create a gradient update message
    pub fn gradient_update(
        sender_id: String,
        round_number: u64,
        batch: GradientBatch,
        metrics: crate::checkpoint::TrainingMetrics,
    ) -> Self {
        Self::new(
            MessageType::GradientUpdate,
            sender_id,
            MessagePayload::GradientUpdate(GradientUpdate {
                round_number,
                gradient_batch: batch,
                local_steps: 0, // To be set by caller
                metrics,
            }),
        )
    }
}

/// Message payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    WorkerRegister(WorkerRegister),
    WorkerHeartbeat(WorkerHeartbeat),
    GradientUpdate(GradientUpdate),
    ModelSync(ModelSync),
    CheckpointRequest(CheckpointRequest),
    TrainingControl(TrainingControl),
    RoundComplete(RoundComplete),
    Error(ErrorMessage),
}

/// Worker registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegister {
    /// Worker capabilities
    pub capabilities: WorkerCapabilities,
    
    /// Initial model version
    pub model_version: String,
    
    /// Worker configuration
    pub config: std::collections::HashMap<String, String>,
}

/// Worker capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    /// Computing power in FLOPS
    pub compute_flops: i64,
    
    /// Available memory in bytes
    pub memory_bytes: i64,
    
    /// Network bandwidth in bytes/sec
    pub bandwidth_bps: i64,
    
    /// Hardware information
    pub hardware: crate::checkpoint::HardwareInfo,
}

impl Default for WorkerCapabilities {
    fn default() -> Self {
        Self {
            compute_flops: 0,
            memory_bytes: 0,
            bandwidth_bps: 0,
            hardware: Default::default(),
        }
    }
}

/// Worker heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    /// Current training step
    pub current_step: u64,
    
    /// Worker status
    pub status: WorkerStatus,
    
    /// Resource utilization
    pub utilization: ResourceUtilization,
}

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Training,
    Syncing,
    Checkpointing,
    Error,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU usage percentage
    pub cpu_percent: f32,
    
    /// Memory usage percentage
    pub memory_percent: f32,
    
    /// GPU usage percentage
    pub gpu_percent: f32,
    
    /// Network usage in bytes/sec
    pub network_bps: i64,
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_percent: 0.0,
            gpu_percent: 0.0,
            network_bps: 0,
        }
    }
}

/// Gradient update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientUpdate {
    /// DiLoCo round number
    pub round_number: u64,
    
    /// Compressed gradients
    pub gradient_batch: GradientBatch,
    
    /// Local training steps completed
    pub local_steps: u64,
    
    /// Local training metrics
    pub metrics: crate::checkpoint::TrainingMetrics,
}

/// Model synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSync {
    /// Sync type
    pub sync_type: SyncType,
    
    /// Model data
    pub model_data: ModelData,
    
    /// Target workers (empty for broadcast)
    pub target_workers: Vec<String>,
}

/// Sync types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncType {
    /// Full model sync
    FullSync,
    /// Delta sync
    DeltaSync,
    /// Specific parameters only
    ParameterSync,
}

/// Model data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelData {
    /// Full model state
    FullModel(ModelState),
    /// Model delta
    ModelDelta(ModelDelta),
    /// Specific parameters
    Parameters(Vec<crate::model::ModelParameter>),
}

/// Checkpoint operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRequest {
    /// Operation type
    pub operation: CheckpointOp,
    
    /// Checkpoint data
    pub data: CheckpointData,
}

/// Checkpoint operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointOp {
    Save,
    Load,
    Delete,
    List,
}

/// Checkpoint data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointData {
    /// Full checkpoint
    Checkpoint(Box<Checkpoint>),
    /// Checkpoint summary
    Summary(CheckpointSummary),
    /// Checkpoint ID
    CheckpointId(String),
    /// List of summaries
    Summaries(Vec<CheckpointSummary>),
}

/// Training control messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingControl {
    /// Control command
    pub command: TrainingCommand,
    
    /// Training configuration
    pub config: Option<TrainingConfig>,
}

/// Training commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingCommand {
    Start,
    Stop,
    Pause,
    Resume,
    Reset,
}

/// Round completion notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundComplete {
    /// Round number
    pub round_number: u64,
    
    /// Participating workers
    pub worker_ids: Vec<String>,
    
    /// Aggregated metrics
    pub aggregated_metrics: crate::checkpoint::TrainingMetrics,
    
    /// Next round configuration
    pub next_round_config: Option<TrainingConfig>,
}

/// Error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// Error code
    pub code: i32,
    
    /// Error message
    pub message: String,
    
    /// Error details
    pub details: std::collections::HashMap<String, String>,
    
    /// Whether error is recoverable
    pub recoverable: bool,
}

/// Message handler trait
pub trait MessageHandler: Send + Sync {
    /// Handle a DiLoCo message
    fn handle_message(&mut self, message: DiLoCoMessage) -> Result<Option<DiLoCoMessage>>;
}

/// Message router for dispatching messages
pub struct MessageRouter {
    handlers: std::collections::HashMap<MessageType, Box<dyn MessageHandler>>,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }
    
    /// Register a handler for a message type
    pub fn register_handler(
        &mut self,
        message_type: MessageType,
        handler: Box<dyn MessageHandler>,
    ) {
        self.handlers.insert(message_type, handler);
    }
    
    /// Route a message to the appropriate handler
    pub fn route_message(&mut self, message: DiLoCoMessage) -> Result<Option<DiLoCoMessage>> {
        if let Some(handler) = self.handlers.get_mut(&message.message_type) {
            handler.handle_message(message)
        } else {
            Err(Error::Other(format!(
                "No handler registered for message type: {:?}",
                message.message_type
            )))
        }
    }
}

/// UUID module placeholder (would use actual uuid crate)
mod uuid {
    pub struct Uuid;
    
    impl Uuid {
        pub fn new_v4() -> String {
            // Simplified UUID generation
            format!("{:x}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let capabilities = WorkerCapabilities::default();
        let message = DiLoCoMessage::worker_register(
            "worker_1".to_string(),
            capabilities,
            "v1.0".to_string(),
        );
        
        assert_eq!(message.message_type, MessageType::WorkerRegister);
        assert_eq!(message.sender_id, "worker_1");
        assert!(!message.message_id.is_empty());
    }
    
    #[test]
    fn test_message_serialization() {
        let message = DiLoCoMessage::new(
            MessageType::Error,
            "test".to_string(),
            MessagePayload::Error(ErrorMessage {
                code: 500,
                message: "Test error".to_string(),
                details: Default::default(),
                recoverable: true,
            }),
        );
        
        // Test serialization
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: DiLoCoMessage = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.message_type, MessageType::Error);
        assert_eq!(deserialized.sender_id, "test");
    }
}