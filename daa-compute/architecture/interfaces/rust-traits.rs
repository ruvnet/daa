// DAA-Compute Core Interfaces and Traits
// This file defines the core interfaces for all components in the distributed compute system

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Core Types
// ============================================================================

pub type NodeId = Uuid;
pub type TaskId = Uuid;
pub type RoundId = u64;
pub type ShardId = Uuid;
pub type CheckpointId = Uuid;
pub type VertexId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blake3Hash(pub [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp(pub SystemTime);

// ============================================================================
// Core Node Interface
// ============================================================================

#[async_trait]
pub trait ComputeNode: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Get node information and capabilities
    async fn get_info(&self) -> Result<NodeInfo, Self::Error>;
    
    /// Join the training network
    async fn join_network(&mut self) -> Result<(), Self::Error>;
    
    /// Leave the training network gracefully
    async fn leave_network(&mut self) -> Result<(), Self::Error>;
    
    /// Execute a training task
    async fn execute_task(&mut self, task: Task) -> Result<TaskResult, Self::Error>;
    
    /// Participate in model synchronization
    async fn synchronize(&mut self, sync_event: SyncEvent) -> Result<SyncResult, Self::Error>;
    
    /// Validate computation from other nodes
    async fn validate(&self, validation_request: ValidationRequest) -> Result<ValidationResult, Self::Error>;
    
    /// Handle incoming messages
    async fn handle_message(&mut self, message: NetworkMessage) -> Result<(), Self::Error>;
    
    /// Get current resource utilization
    async fn get_resource_usage(&self) -> Result<ResourceUsage, Self::Error>;
    
    /// Update node configuration
    async fn update_config(&mut self, config: NodeConfig) -> Result<(), Self::Error>;
}

// ============================================================================
// Node Types and Capabilities
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub capabilities: NodeCapabilities,
    pub network_address: NetworkAddress,
    pub reputation: f32,
    pub uptime: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Cloud,
    Edge,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub compute_power: ComputePower,
    pub memory_capacity: MemoryCapacity,
    pub storage_capacity: StorageCapacity,
    pub network_bandwidth: NetworkBandwidth,
    pub supported_features: Vec<Feature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePower {
    pub cpu_cores: u32,
    pub gpu_count: u32,
    pub gpu_memory_gb: u32,
    pub tflops: f32,
    pub supports_precision: Vec<Precision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Precision {
    FP32,
    FP16,
    BF16,
    INT8,
    INT4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCapacity {
    pub ram_gb: u32,
    pub vram_gb: u32,
    pub available_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapacity {
    pub total_gb: u64,
    pub available_gb: u64,
    pub io_bandwidth_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBandwidth {
    pub download_mbps: f32,
    pub upload_mbps: f32,
    pub latency_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Feature {
    ModelHosting,
    Coordination,
    Validation,
    WebAssembly,
    WebGPU,
    QuantumResistantCrypto,
}

// ============================================================================
// Training Coordination Interface
// ============================================================================

#[async_trait]
pub trait TrainingCoordinator: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Initialize a new training round
    async fn initialize_round(&mut self) -> Result<TrainingRound, Self::Error>;
    
    /// Distribute tasks to nodes
    async fn distribute_tasks(
        &mut self,
        round: &TrainingRound,
        available_nodes: Vec<NodeId>,
    ) -> Result<TaskDistribution, Self::Error>;
    
    /// Collect results from training tasks
    async fn collect_results(
        &mut self,
        task_assignments: TaskDistribution,
    ) -> Result<Vec<TaskResult>, Self::Error>;
    
    /// Aggregate gradients or model updates
    async fn aggregate_updates(
        &mut self,
        updates: Vec<ModelUpdate>,
    ) -> Result<AggregatedUpdate, Self::Error>;
    
    /// Apply aggregated update to global model
    async fn apply_global_update(
        &mut self,
        update: AggregatedUpdate,
    ) -> Result<ModelState, Self::Error>;
    
    /// Create checkpoint
    async fn create_checkpoint(
        &mut self,
        model_state: &ModelState,
        round_info: &TrainingRound,
    ) -> Result<CheckpointVertex, Self::Error>;
    
    /// Handle node failures
    async fn handle_node_failure(&mut self, node_id: NodeId) -> Result<(), Self::Error>;
    
    /// Get training metrics
    async fn get_metrics(&self) -> Result<TrainingMetrics, Self::Error>;
}

// ============================================================================
// Training Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRound {
    pub round_id: RoundId,
    pub round_number: u64,
    pub start_time: Timestamp,
    pub timeout: Duration,
    pub participating_nodes: Vec<NodeId>,
    pub model_version: u64,
    pub expected_tasks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDistribution {
    pub assignments: HashMap<NodeId, Vec<TaskAssignment>>,
    pub total_tasks: u32,
    pub estimated_completion_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: TaskId,
    pub task_type: TaskType,
    pub node_id: NodeId,
    pub deadline: Timestamp,
    pub priority: Priority,
    pub dependencies: Vec<TaskId>,
    pub parameters: TaskParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Training {
        batch_ids: Vec<u64>,
        model_shard: ShardId,
    },
    Validation {
        validation_set: String,
        metrics: Vec<MetricType>,
    },
    Aggregation {
        gradient_ids: Vec<Uuid>,
    },
    Checkpoint {
        model_state: ModelState,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// Model and Sharding Interfaces
// ============================================================================

#[async_trait]
pub trait ModelShard: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Load shard from storage
    async fn load(&mut self, shard_id: ShardId) -> Result<(), Self::Error>;
    
    /// Save shard to storage
    async fn save(&self, location: StorageLocation) -> Result<(), Self::Error>;
    
    /// Get shard metadata
    fn get_metadata(&self) -> &ShardMetadata;
    
    /// Forward pass through shard
    async fn forward(&self, input: Tensor) -> Result<Tensor, Self::Error>;
    
    /// Backward pass through shard
    async fn backward(&self, gradient: Tensor) -> Result<Tensor, Self::Error>;
    
    /// Get current parameters
    fn get_parameters(&self) -> &ParameterMap;
    
    /// Update parameters
    fn update_parameters(&mut self, updates: ParameterUpdate) -> Result<(), Self::Error>;
    
    /// Calculate memory footprint
    fn memory_footprint(&self) -> u64;
    
    /// Estimate compute requirements
    fn compute_requirements(&self) -> ComputeRequirements;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMetadata {
    pub shard_id: ShardId,
    pub layer_range: std::ops::Range<usize>,
    pub parameter_count: u64,
    pub size_bytes: u64,
    pub dependencies: Vec<ShardId>,
    pub checksum: Blake3Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequirements {
    pub flops_per_sample: u64,
    pub memory_mb: u32,
    pub preferred_precision: Precision,
}

// ============================================================================
// Network and Communication Interfaces
// ============================================================================

#[async_trait]
pub trait NetworkManager: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Connect to the network
    async fn connect(&mut self) -> Result<(), Self::Error>;
    
    /// Disconnect from the network
    async fn disconnect(&mut self) -> Result<(), Self::Error>;
    
    /// Send message to specific peer
    async fn send_to_peer(
        &mut self,
        peer_id: NodeId,
        message: NetworkMessage,
    ) -> Result<(), Self::Error>;
    
    /// Broadcast message to all peers
    async fn broadcast(&mut self, message: NetworkMessage) -> Result<(), Self::Error>;
    
    /// Subscribe to message type
    async fn subscribe(&mut self, message_type: MessageType) -> Result<(), Self::Error>;
    
    /// Discover peers
    async fn discover_peers(&mut self) -> Result<Vec<NodeInfo>, Self::Error>;
    
    /// Get network status
    async fn get_status(&self) -> Result<NetworkStatus, Self::Error>;
    
    /// Handle incoming messages
    async fn handle_incoming(&mut self) -> Option<(NodeId, NetworkMessage)>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_id: Uuid,
    pub sender: NodeId,
    pub recipient: Option<NodeId>, // None for broadcast
    pub message_type: MessageType,
    pub payload: MessagePayload,
    pub timestamp: Timestamp,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    TaskAssignment,
    TaskResult,
    ModelUpdate,
    GradientShare,
    CheckpointAnnouncement,
    ConsensusVote,
    Heartbeat,
    NodeJoin,
    NodeLeave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    TaskAssignment(TaskAssignment),
    TaskResult(TaskResult),
    ModelUpdate(ModelUpdate),
    GradientUpdate(GradientUpdate),
    CheckpointVertex(CheckpointVertex),
    ConsensusVote(ConsensusVote),
    Heartbeat(HeartbeatData),
    NodeInfo(NodeInfo),
}

// ============================================================================
// Storage and Checkpoint Interfaces
// ============================================================================

#[async_trait]
pub trait CheckpointStorage: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Store checkpoint
    async fn store_checkpoint(
        &mut self,
        checkpoint: &CheckpointVertex,
        model_data: &[u8],
    ) -> Result<StorageLocation, Self::Error>;
    
    /// Retrieve checkpoint
    async fn retrieve_checkpoint(
        &self,
        checkpoint_id: CheckpointId,
    ) -> Result<CheckpointVertex, Self::Error>;
    
    /// List available checkpoints
    async fn list_checkpoints(&self) -> Result<Vec<CheckpointId>, Self::Error>;
    
    /// Delete checkpoint
    async fn delete_checkpoint(&mut self, checkpoint_id: CheckpointId) -> Result<(), Self::Error>;
    
    /// Get storage statistics
    async fn get_stats(&self) -> Result<StorageStats, Self::Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointVertex {
    pub vertex_id: VertexId,
    pub checkpoint_id: CheckpointId,
    pub parent_checkpoints: Vec<VertexId>,
    pub model_state_hash: Blake3Hash,
    pub round_number: u64,
    pub timestamp: Timestamp,
    pub metadata: CheckpointMetadata,
    pub consensus_proof: ConsensusProof,
    pub storage_locations: Vec<StorageLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub model_version: String,
    pub total_parameters: u64,
    pub training_steps: u64,
    pub loss_value: f32,
    pub accuracy: Option<f32>,
    pub contributors: Vec<NodeId>,
}

// ============================================================================
// Consensus and Validation Interfaces
// ============================================================================

#[async_trait]
pub trait ConsensusProtocol: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Submit proposal for consensus
    async fn submit_proposal(&mut self, proposal: Proposal) -> Result<ProposalId, Self::Error>;
    
    /// Vote on proposal
    async fn vote(&mut self, proposal_id: ProposalId, vote: Vote) -> Result<(), Self::Error>;
    
    /// Get consensus result
    async fn get_result(&self, proposal_id: ProposalId) -> Result<ConsensusResult, Self::Error>;
    
    /// Check if proposal is finalized
    async fn is_finalized(&self, proposal_id: ProposalId) -> Result<bool, Self::Error>;
    
    /// Get current consensus state
    async fn get_state(&self) -> Result<ConsensusState, Self::Error>;
}

pub type ProposalId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub proposal_id: ProposalId,
    pub proposer: NodeId,
    pub proposal_type: ProposalType,
    pub data: Vec<u8>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    ModelUpdate,
    CheckpointCreation,
    NodeAdmission,
    ParameterChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: NodeId,
    pub proposal_id: ProposalId,
    pub decision: VoteDecision,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteDecision {
    Accept,
    Reject,
    Abstain,
}

// ============================================================================
// DAA Autonomy Loop Interface
// ============================================================================

#[async_trait]
pub trait AutonomyLoop: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    type MonitoringData;
    type Decision;
    type ActionOutcome;
    type Insights;
    
    /// Monitor system state and environment
    async fn monitor(&mut self) -> Result<Self::MonitoringData, Self::Error>;
    
    /// Reason about the monitored data and make decisions
    async fn reason(&mut self, data: Self::MonitoringData) -> Result<Self::Decision, Self::Error>;
    
    /// Act on the decision
    async fn act(&mut self, decision: Self::Decision) -> Result<Self::ActionOutcome, Self::Error>;
    
    /// Reflect on the action outcome
    async fn reflect(&mut self, outcome: Self::ActionOutcome) -> Result<Self::Insights, Self::Error>;
    
    /// Adapt based on insights
    async fn adapt(&mut self, insights: Self::Insights) -> Result<(), Self::Error>;
    
    /// Run the complete autonomy loop
    async fn run_loop(&mut self) -> Result<(), Self::Error> {
        let data = self.monitor().await?;
        let decision = self.reason(data).await?;
        let outcome = self.act(decision).await?;
        let insights = self.reflect(outcome).await?;
        self.adapt(insights).await?;
        Ok(())
    }
}

// ============================================================================
// Task and Result Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: TaskId,
    pub task_type: TaskType,
    pub parameters: TaskParameters,
    pub deadline: Timestamp,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskParameters {
    pub batch_size: usize,
    pub learning_rate: f32,
    pub data_indices: Vec<u64>,
    pub model_shard_ids: Vec<ShardId>,
    pub custom_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub node_id: NodeId,
    pub result_type: ResultType,
    pub execution_time: Duration,
    pub resource_usage: ResourceUsage,
    pub metrics: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultType {
    GradientUpdate(GradientUpdate),
    ModelUpdate(ModelUpdate),
    ValidationResult(ValidationResult),
    CheckpointCreated(CheckpointId),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientUpdate {
    pub gradient_id: Uuid,
    pub layer_gradients: HashMap<String, Tensor>,
    pub batch_size: usize,
    pub loss_value: f32,
    pub gradient_norm: f32,
    pub compression_ratio: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdate {
    pub update_id: Uuid,
    pub parameter_updates: ParameterUpdate,
    pub optimizer_state: Option<OptimizerState>,
    pub metadata: UpdateMetadata,
}

// ============================================================================
// Resource Monitoring
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_used_mb: u32,
    pub gpu_utilization: f32,
    pub network_in_mbps: f32,
    pub network_out_mbps: f32,
    pub storage_read_mbps: f32,
    pub storage_write_mbps: f32,
}

// ============================================================================
// Utility Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub dtype: DataType,
    pub data: Vec<u8>, // Serialized tensor data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    F32,
    F16,
    I32,
    I8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterMap {
    pub parameters: HashMap<String, Tensor>,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterUpdate {
    pub updates: HashMap<String, Tensor>,
    pub update_type: UpdateType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    Gradient,
    Parameter,
    Momentum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerState {
    pub state_dict: HashMap<String, Tensor>,
    pub step_count: u64,
    pub learning_rate: f32,
}

// ============================================================================
// Validation and Metrics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub request_id: Uuid,
    pub validation_type: ValidationType,
    pub data: Vec<u8>,
    pub expected_result: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    GradientVerification,
    ModelIntegrity,
    ComputationCorrectness,
    ConsensusParticipation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub request_id: Uuid,
    pub is_valid: bool,
    pub confidence: f32,
    pub details: String,
    pub validator_id: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    pub round_number: u64,
    pub loss: f32,
    pub accuracy: Option<f32>,
    pub convergence_rate: f32,
    pub node_participation: f32,
    pub communication_overhead: Duration,
    pub computation_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Loss,
    Accuracy,
    F1Score,
    Perplexity,
    Bleu,
    Custom(String),
}

// ============================================================================
// Network and Storage Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAddress {
    pub ip: String,
    pub port: u16,
    pub protocol: NetworkProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkProtocol {
    Tcp,
    Quic,
    WebSocket,
    WebRtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub connected_peers: u32,
    pub total_bandwidth_mbps: f32,
    pub average_latency_ms: f32,
    pub message_queue_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocation {
    pub url: String,
    pub storage_type: StorageType,
    pub access_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    Local,
    S3Compatible,
    Ipfs,
    Distributed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_capacity_gb: u64,
    pub used_capacity_gb: u64,
    pub checkpoints_stored: u32,
    pub average_checkpoint_size_mb: f32,
}

// ============================================================================
// Configuration Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub max_concurrent_tasks: u32,
    pub resource_limits: ResourceLimits,
    pub network_config: NetworkConfig,
    pub storage_config: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_gb: u32,
    pub max_cpu_percent: f32,
    pub max_gpu_percent: f32,
    pub max_bandwidth_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_address: String,
    pub bootstrap_peers: Vec<String>,
    pub max_connections: u32,
    pub heartbeat_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub local_path: String,
    pub remote_backends: Vec<StorageLocation>,
    pub replication_factor: u32,
    pub compression_enabled: bool,
}

// ============================================================================
// Synchronization Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub sync_id: Uuid,
    pub sync_type: SyncType,
    pub participants: Vec<NodeId>,
    pub deadline: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    ModelSync,
    GradientAllReduce,
    CheckpointSync,
    ConsensusSync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub sync_id: Uuid,
    pub success: bool,
    pub participants: Vec<NodeId>,
    pub duration: Duration,
    pub data_transferred_mb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedUpdate {
    pub aggregation_id: Uuid,
    pub aggregated_gradients: HashMap<String, Tensor>,
    pub contributor_count: u32,
    pub aggregation_method: AggregationMethod,
    pub quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationMethod {
    Average,
    WeightedAverage,
    Median,
    TrimmedMean,
    FederatedAverage,
}

// ============================================================================
// Consensus Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProof {
    pub round: u64,
    pub decision: ConsensusDecision,
    pub signatures: Vec<(NodeId, Vec<u8>)>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusDecision {
    Accept,
    Reject,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    pub proposal_id: ProposalId,
    pub voter: NodeId,
    pub decision: VoteDecision,
    pub timestamp: Timestamp,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub proposal_id: ProposalId,
    pub final_decision: ConsensusDecision,
    pub vote_count: HashMap<VoteDecision, u32>,
    pub finalization_time: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub current_round: u64,
    pub active_proposals: Vec<ProposalId>,
    pub validator_set: Vec<NodeId>,
    pub byzantine_threshold: u32,
}

// ============================================================================
// Model State Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelState {
    pub model_id: Uuid,
    pub version: u64,
    pub parameters: ParameterMap,
    pub optimizer_state: Option<OptimizerState>,
    pub training_metadata: TrainingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetadata {
    pub total_samples: u64,
    pub total_steps: u64,
    pub current_epoch: u32,
    pub best_loss: f32,
    pub best_accuracy: Option<f32>,
    pub training_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadata {
    pub update_id: Uuid,
    pub source_nodes: Vec<NodeId>,
    pub aggregation_method: AggregationMethod,
    pub quality_metrics: QualityMetrics,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub gradient_norm: f32,
    pub parameter_change_norm: f32,
    pub convergence_indicator: f32,
    pub noise_estimate: f32,
}

// ============================================================================
// Heartbeat Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    pub node_id: NodeId,
    pub timestamp: Timestamp,
    pub status: NodeStatus,
    pub current_tasks: Vec<TaskId>,
    pub resource_usage: ResourceUsage,
    pub last_checkpoint: Option<CheckpointId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Healthy,
    Busy,
    Degraded,
    Joining,
    Leaving,
    Failed,
}