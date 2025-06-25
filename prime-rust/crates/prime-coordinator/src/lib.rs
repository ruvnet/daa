// Prime Coordinator with DAA Orchestration
use anyhow::Result;
use async_trait::async_trait;
use daa_orchestrator::{DaaOrchestrator, OrchestratorConfig};
use prime_core::{
    DaaContext, DhtInterface, EconomyInterface, GovernanceInterface,
    ModelParams, NodeCapacity, NodeType, PrimeMessage, TrainingTask, TaskType,
    DataShard, GovernanceRule, RuleType,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Coordinator state tracking active nodes and tasks
#[derive(Default)]
pub struct CoordinatorState {
    active_nodes: HashMap<String, NodeInfo>,
    pending_tasks: Vec<TrainingTask>,
    completed_tasks: HashMap<String, CompletedTask>,
    current_round: u64,
    model_version: u64,
}

#[derive(Clone)]
pub struct NodeInfo {
    pub node_id: String,
    pub node_type: NodeType,
    pub capacity: NodeCapacity,
    pub last_heartbeat: u64,
    pub reliability_score: f32,
}

#[derive(Clone)]
pub struct CompletedTask {
    pub task_id: String,
    pub completed_by: String,
    pub accuracy: f32,
    pub completion_time: u64,
}

/// Coordinator node with DAA orchestration
pub struct CoordinatorNode {
    orchestrator: DaaOrchestrator,
    state: Arc<RwLock<CoordinatorState>>,
    config: CoordinatorConfig,
}

#[derive(Clone)]
pub struct CoordinatorConfig {
    pub min_nodes_for_round: usize,
    pub heartbeat_timeout_ms: u64,
    pub task_timeout_ms: u64,
    pub consensus_threshold: f32,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            min_nodes_for_round: 3,
            heartbeat_timeout_ms: 5000,
            task_timeout_ms: 60000,
            consensus_threshold: 0.66,
        }
    }
}

impl CoordinatorNode {
    /// Create a new coordinator with DAA integration
    pub async fn new(
        node_id: String,
        config: CoordinatorConfig,
        dht: Arc<dyn DhtInterface>,
        economy: Arc<dyn EconomyInterface>,
        governance: Arc<dyn GovernanceInterface>,
    ) -> Result<Self> {
        let orch_config = OrchestratorConfig::default()
            .with_name(format!("coordinator-{}", node_id))
            .with_autonomy_interval(std::time::Duration::from_secs(5));
        
        let orchestrator = DaaOrchestrator::new(orch_config).await?;
        let state = Arc::new(RwLock::new(CoordinatorState::default()));
        
        // Initialize governance rules
        governance.add_rule(GovernanceRule {
            id: "min_accuracy".to_string(),
            name: "Minimum Accuracy Rule".to_string(),
            description: "Ensure model maintains minimum accuracy".to_string(),
            rule_type: RuleType::RequiredAccuracy(0.80),
            parameters: HashMap::new(),
        }).await?;
        
        governance.add_rule(GovernanceRule {
            id: "max_spending".to_string(),
            name: "Daily Spending Limit".to_string(),
            description: "Limit daily token spending".to_string(),
            rule_type: RuleType::MaxDailySpending(10000),
            parameters: HashMap::new(),
        }).await?;
        
        Ok(Self {
            orchestrator,
            state,
            config,
        })
    }
    
    /// Setup autonomy loops for the coordinator
    pub async fn setup_autonomy_loops(&mut self, daa_context: DaaContext) -> Result<()> {
        let state = self.state.clone();
        let config = self.config.clone();
        
        // Monitor - track node health and collect heartbeats
        self.orchestrator.add_task("monitor_nodes", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let state = state.clone();
                let config = config.clone();
                async move {
                    let mut state_guard = state.write().await;
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    // Remove inactive nodes
                    state_guard.active_nodes.retain(|_, node| {
                        let is_active = current_time - node.last_heartbeat < config.heartbeat_timeout_ms;
                        if !is_active {
                            tracing::warn!("Node {} timed out", node.node_id);
                        }
                        is_active
                    });
                    
                    tracing::info!("Active nodes: {}", state_guard.active_nodes.len());
                    
                    // Store node list in DHT
                    let node_list: Vec<String> = state_guard.active_nodes.keys().cloned().collect();
                    let serialized = serde_json::to_vec(&node_list)?;
                    daa_ctx.dht_handle.put("nodes:active", serialized).await?;
                    
                    Ok(())
                }
            }
        });
        
        // Reason - decide on task allocation strategy
        self.orchestrator.add_task("reason_allocation", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let state = state.clone();
                let config = config.clone();
                async move {
                    let state_guard = state.read().await;
                    
                    // Check if we have enough nodes for a training round
                    if state_guard.active_nodes.len() >= config.min_nodes_for_round {
                        tracing::info!("Ready to start training round {}", state_guard.current_round + 1);
                        
                        // Validate with governance
                        let mut params = HashMap::new();
                        params.insert("num_nodes".to_string(), state_guard.active_nodes.len().to_string());
                        params.insert("round".to_string(), state_guard.current_round.to_string());
                        
                        let is_valid = daa_ctx.governance_handle
                            .validate_action("start_round", params)
                            .await?;
                        
                        if !is_valid {
                            tracing::warn!("Governance blocked training round");
                        }
                    } else {
                        tracing::debug!("Waiting for more nodes: {}/{}", 
                            state_guard.active_nodes.len(), 
                            config.min_nodes_for_round
                        );
                    }
                    
                    Ok(())
                }
            }
        });
        
        // Act - allocate tasks to nodes
        self.orchestrator.add_task("act_allocate", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let state = state.clone();
                let config = config.clone();
                async move {
                    let mut state_guard = state.write().await;
                    
                    if state_guard.active_nodes.len() < config.min_nodes_for_round {
                        return Ok(());
                    }
                    
                    // Create tasks for active nodes
                    let nodes: Vec<_> = state_guard.active_nodes.values().cloned().collect();
                    let num_nodes = nodes.len();
                    
                    for (idx, node) in nodes.iter().enumerate() {
                        let task = TrainingTask {
                            task_id: format!("task-{}-{}", state_guard.current_round, idx),
                            task_type: TaskType::ForwardBackward,
                            data_shard: DataShard {
                                shard_id: format!("shard-{}", idx),
                                start_idx: idx * 1000,
                                end_idx: (idx + 1) * 1000,
                                data_uri: format!("dht://data/shard-{}", idx),
                            },
                            deadline: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64 + config.task_timeout_ms,
                            reward_amount: 100, // Base reward
                        };
                        
                        // Store task assignment in DHT
                        let assignment = serde_json::to_vec(&task)?;
                        daa_ctx.dht_handle.put(&format!("task:{}", task.task_id), assignment).await?;
                        
                        // Charge coordination fee
                        daa_ctx.economy_handle.charge_usage(&node.node_id, 5).await?;
                        
                        state_guard.pending_tasks.push(task);
                    }
                    
                    state_guard.current_round += 1;
                    tracing::info!("Started training round {} with {} tasks", 
                        state_guard.current_round, 
                        state_guard.pending_tasks.len()
                    );
                    
                    Ok(())
                }
            }
        });
        
        // Reflect - analyze completed tasks
        self.orchestrator.add_task("reflect_results", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let state = state.clone();
                async move {
                    let state_guard = state.read().await;
                    
                    // Calculate average accuracy
                    if !state_guard.completed_tasks.is_empty() {
                        let avg_accuracy: f32 = state_guard.completed_tasks.values()
                            .map(|t| t.accuracy)
                            .sum::<f32>() / state_guard.completed_tasks.len() as f32;
                        
                        tracing::info!("Average task accuracy: {:.2}%", avg_accuracy * 100.0);
                        
                        // Update model metadata in DHT
                        let metadata = serde_json::json!({
                            "version": state_guard.model_version,
                            "accuracy": avg_accuracy,
                            "round": state_guard.current_round,
                        });
                        
                        daa_ctx.dht_handle.put("model:metadata", serde_json::to_vec(&metadata)?).await?;
                    }
                    
                    Ok(())
                }
            }
        });
        
        // Adapt - adjust strategy based on performance
        self.orchestrator.add_task("adapt_strategy", {
            let daa_ctx = daa_context.clone();
            move |_| {
                let daa_ctx = daa_ctx.clone();
                let state = state.clone();
                async move {
                    let mut state_guard = state.write().await;
                    
                    // Adjust node reliability scores
                    for (node_id, completed_task) in &state_guard.completed_tasks {
                        if let Some(node_info) = state_guard.active_nodes.get_mut(&completed_task.completed_by) {
                            // Update reliability based on accuracy
                            node_info.reliability_score = 
                                0.9 * node_info.reliability_score + 0.1 * completed_task.accuracy;
                        }
                    }
                    
                    // Clear old completed tasks periodically
                    if state_guard.completed_tasks.len() > 1000 {
                        state_guard.completed_tasks.clear();
                    }
                    
                    Ok(())
                }
            }
        });
        
        Ok(())
    }
    
    /// Handle incoming messages
    pub async fn handle_message(&self, message: PrimeMessage) -> Result<()> {
        match message {
            PrimeMessage::Heartbeat { node_id, node_type, capacity } => {
                let mut state = self.state.write().await;
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                
                state.active_nodes.insert(node_id.clone(), NodeInfo {
                    node_id,
                    node_type,
                    capacity,
                    last_heartbeat: current_time,
                    reliability_score: 0.9,
                });
            }
            PrimeMessage::ValidationResult { task_id, accuracy, loss } => {
                let mut state = self.state.write().await;
                state.completed_tasks.insert(task_id.clone(), CompletedTask {
                    task_id: task_id.clone(),
                    completed_by: "validator".to_string(),
                    accuracy,
                    completion_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                });
                
                // Remove from pending
                state.pending_tasks.retain(|t| t.task_id != task_id);
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Start the coordinator
    pub async fn start(mut self) -> Result<()> {
        tracing::info!("Starting coordinator node");
        self.orchestrator.run_autonomy_loop().await
    }
}