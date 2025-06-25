//! Coordinator DAA Agent Implementation
//! Central coordination agent for orchestrating multi-agent operations

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc, broadcast};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{debug, info, warn, error};

/// Coordination state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoordinationState {
    Initializing,
    Ready,
    Coordinating,
    Delegating,
    Monitoring,
    Aggregating,
    Completing,
    Failed(String),
}

/// Agent registration info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub last_heartbeat: Instant,
    pub metrics: HashMap<String, f64>,
}

/// Agent status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Online,
    Busy,
    Idle,
    Offline,
    Failed,
}

/// Coordination task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationTask {
    pub id: String,
    pub task_type: String,
    pub priority: TaskPriority,
    pub requirements: Vec<String>,
    pub assigned_agents: Vec<String>,
    pub status: TaskStatus,
    pub created_at: Instant,
    pub deadline: Option<Duration>,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Task status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

/// Coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorConfig {
    pub max_agents: usize,
    pub heartbeat_interval_ms: u64,
    pub task_timeout_ms: u64,
    pub load_balancing_enabled: bool,
    pub auto_scaling_enabled: bool,
    pub consensus_required: bool,
    pub consensus_threshold: f64,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_agents: 100,
            heartbeat_interval_ms: 5000,
            task_timeout_ms: 60000,
            load_balancing_enabled: true,
            auto_scaling_enabled: true,
            consensus_required: false,
            consensus_threshold: 0.66,
        }
    }
}

/// Coordinator messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinatorMessage {
    RegisterAgent { agent_info: AgentInfo },
    UnregisterAgent { agent_id: String },
    SubmitTask { task: CoordinationTask },
    AgentHeartbeat { agent_id: String, metrics: HashMap<String, f64> },
    TaskCompleted { task_id: String, agent_id: String, result: TaskResult },
    RequestConsensus { proposal: String, timeout: Duration },
    EmergencyStop,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time: Duration,
}

/// Coordinator DAA Agent
pub struct CoordinatorAgent {
    id: String,
    config: CoordinatorConfig,
    state: Arc<RwLock<CoordinationState>>,
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    tasks: Arc<RwLock<HashMap<String, CoordinationTask>>>,
    task_queue: Arc<RwLock<Vec<CoordinationTask>>>,
    message_channel: mpsc::Sender<CoordinatorMessage>,
    broadcast_channel: broadcast::Sender<CoordinatorMessage>,
    autonomy_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl CoordinatorAgent {
    /// Create a new coordinator agent
    pub async fn new(config: CoordinatorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel(1000);
        let (broadcast_tx, _) = broadcast::channel(100);
        
        let agent = Self {
            id: Uuid::new_v4().to_string(),
            config,
            state: Arc::new(RwLock::new(CoordinationState::Initializing)),
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            message_channel: tx,
            broadcast_channel: broadcast_tx,
            autonomy_handle: None,
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        };

        // Start message handler
        agent.start_message_handler(rx).await;
        
        Ok(agent)
    }

    /// Initialize the coordinator
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Coordinator Agent {}", self.id);
        self.set_state(CoordinationState::Initializing).await;
        
        // Start autonomy loop
        self.start_autonomy_loop().await?;
        
        // Start heartbeat monitor
        self.start_heartbeat_monitor().await?;
        
        self.set_state(CoordinationState::Ready).await;
        info!("Coordinator Agent {} initialized", self.id);
        Ok(())
    }

    /// Start the autonomy loop
    async fn start_autonomy_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.clone();
        let agents = self.agents.clone();
        let tasks = self.tasks.clone();
        let task_queue = self.task_queue.clone();
        let config = self.config.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let id = self.id.clone();

        let handle = tokio::spawn(async move {
            Self::run_autonomy_loop(id, state, agents, tasks, task_queue, config, shutdown_signal).await;
        });

        self.autonomy_handle = Some(handle);
        Ok(())
    }

    /// Main autonomy loop
    async fn run_autonomy_loop(
        id: String,
        state: Arc<RwLock<CoordinationState>>,
        agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
        tasks: Arc<RwLock<HashMap<String, CoordinationTask>>>,
        task_queue: Arc<RwLock<Vec<CoordinationTask>>>,
        config: CoordinatorConfig,
        shutdown_signal: Arc<tokio::sync::Notify>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        
        info!("Coordinator Agent {} autonomy loop started", id);

        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => {
                    info!("Coordinator Agent {} received shutdown signal", id);
                    break;
                }
                
                _ = interval.tick() => {
                    let current_state = state.read().await.clone();
                    
                    match current_state {
                        CoordinationState::Ready | CoordinationState::Monitoring => {
                            // Process task queue
                            if let Err(e) = Self::process_task_queue(
                                &state,
                                &agents,
                                &tasks,
                                &task_queue,
                                &config
                            ).await {
                                error!("Error processing task queue: {}", e);
                            }
                            
                            // Monitor agent health
                            Self::monitor_agent_health(&agents).await;
                            
                            // Auto-scale if needed
                            if config.auto_scaling_enabled {
                                Self::auto_scale_agents(&agents, &tasks, &config).await;
                            }
                            
                            // Load balance tasks
                            if config.load_balancing_enabled {
                                Self::load_balance_tasks(&agents, &tasks).await;
                            }
                        }
                        
                        CoordinationState::Failed(ref error) => {
                            warn!("Coordinator in failed state: {}", error);
                            // Attempt recovery
                            if let Ok(_) = Self::attempt_recovery(&state).await {
                                *state.write().await = CoordinationState::Ready;
                            }
                        }
                        
                        _ => {
                            debug!("Current state: {:?}", current_state);
                        }
                    }
                }
            }
        }

        info!("Coordinator Agent {} autonomy loop completed", id);
    }

    /// Process task queue
    async fn process_task_queue(
        state: &Arc<RwLock<CoordinationState>>,
        agents: &Arc<RwLock<HashMap<String, AgentInfo>>>,
        tasks: &Arc<RwLock<HashMap<String, CoordinationTask>>>,
        task_queue: &Arc<RwLock<Vec<CoordinationTask>>>,
        config: &CoordinatorConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = task_queue.write().await;
        
        if queue.is_empty() {
            return Ok(());
        }
        
        *state.write().await = CoordinationState::Delegating;
        
        // Sort by priority
        queue.sort_by(|a, b| {
            match (&a.priority, &b.priority) {
                (TaskPriority::Critical, TaskPriority::Critical) => a.created_at.cmp(&b.created_at),
                (TaskPriority::Critical, _) => std::cmp::Ordering::Less,
                (_, TaskPriority::Critical) => std::cmp::Ordering::Greater,
                (TaskPriority::High, TaskPriority::High) => a.created_at.cmp(&b.created_at),
                (TaskPriority::High, _) => std::cmp::Ordering::Less,
                (_, TaskPriority::High) => std::cmp::Ordering::Greater,
                _ => a.created_at.cmp(&b.created_at),
            }
        });
        
        // Assign tasks to available agents
        let agents_map = agents.read().await;
        let mut tasks_map = tasks.write().await;
        
        let mut assigned_count = 0;
        queue.retain(|task| {
            // Find suitable agents
            let suitable_agents: Vec<_> = agents_map.values()
                .filter(|agent| {
                    agent.status == AgentStatus::Idle &&
                    task.requirements.iter().all(|req| agent.capabilities.contains(req))
                })
                .take(1)  // For now, assign one agent per task
                .collect();
            
            if !suitable_agents.is_empty() {
                let mut task_clone = task.clone();
                task_clone.status = TaskStatus::Assigned;
                task_clone.assigned_agents = suitable_agents.iter().map(|a| a.id.clone()).collect();
                
                tasks_map.insert(task.id.clone(), task_clone);
                assigned_count += 1;
                
                info!("Assigned task {} to agents: {:?}", task.id, task.assigned_agents);
                false  // Remove from queue
            } else {
                debug!("No suitable agents for task {}", task.id);
                true   // Keep in queue
            }
        });
        
        if assigned_count > 0 {
            info!("Assigned {} tasks to agents", assigned_count);
        }
        
        *state.write().await = CoordinationState::Monitoring;
        Ok(())
    }

    /// Monitor agent health
    async fn monitor_agent_health(agents: &Arc<RwLock<HashMap<String, AgentInfo>>>) {
        let mut agents_map = agents.write().await;
        let now = Instant::now();
        
        for agent in agents_map.values_mut() {
            let time_since_heartbeat = now.duration_since(agent.last_heartbeat);
            
            if time_since_heartbeat > Duration::from_secs(30) {
                if agent.status != AgentStatus::Offline {
                    warn!("Agent {} missed heartbeat, marking as offline", agent.id);
                    agent.status = AgentStatus::Offline;
                }
            }
        }
    }

    /// Auto-scale agents based on load
    async fn auto_scale_agents(
        agents: &Arc<RwLock<HashMap<String, AgentInfo>>>,
        tasks: &Arc<RwLock<HashMap<String, CoordinationTask>>>,
        config: &CoordinatorConfig,
    ) {
        let agents_count = agents.read().await.len();
        let pending_tasks = tasks.read().await.values()
            .filter(|t| t.status == TaskStatus::Pending)
            .count();
        
        let load_ratio = pending_tasks as f64 / (agents_count.max(1) as f64);
        
        if load_ratio > 2.0 && agents_count < config.max_agents {
            info!("High load detected (ratio: {:.2}), need to scale up", load_ratio);
            // In real implementation, would spawn new agents
        } else if load_ratio < 0.5 && agents_count > 1 {
            debug!("Low load detected (ratio: {:.2}), could scale down", load_ratio);
            // In real implementation, would gracefully shutdown idle agents
        }
    }

    /// Load balance tasks across agents
    async fn load_balance_tasks(
        agents: &Arc<RwLock<HashMap<String, AgentInfo>>>,
        tasks: &Arc<RwLock<HashMap<String, CoordinationTask>>>,
    ) {
        let agents_map = agents.read().await;
        let tasks_map = tasks.read().await;
        
        // Calculate load per agent
        let mut agent_loads: HashMap<String, usize> = HashMap::new();
        
        for task in tasks_map.values() {
            if task.status == TaskStatus::InProgress {
                for agent_id in &task.assigned_agents {
                    *agent_loads.entry(agent_id.clone()).or_insert(0) += 1;
                }
            }
        }
        
        // Check for imbalanced load
        if !agent_loads.is_empty() {
            let max_load = agent_loads.values().max().unwrap_or(&0);
            let min_load = agent_loads.values().min().unwrap_or(&0);
            
            if max_load - min_load > 2 {
                debug!("Load imbalance detected: max={}, min={}", max_load, min_load);
                // In real implementation, would redistribute tasks
            }
        }
    }

    /// Attempt recovery from failed state
    async fn attempt_recovery(
        state: &Arc<RwLock<CoordinationState>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Attempting to recover coordinator from failed state");
        
        // In real implementation, would:
        // 1. Check agent connectivity
        // 2. Verify task states
        // 3. Restart failed components
        // 4. Rebalance workload
        
        Ok(())
    }

    /// Start heartbeat monitor
    async fn start_heartbeat_monitor(&self) -> Result<(), Box<dyn std::error::Error>> {
        let agents = self.agents.clone();
        let heartbeat_interval = self.config.heartbeat_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(heartbeat_interval));
            
            loop {
                interval.tick().await;
                
                // Check for stale agents
                let now = Instant::now();
                let mut agents_map = agents.write().await;
                
                agents_map.retain(|id, agent| {
                    let age = now.duration_since(agent.last_heartbeat);
                    if age > Duration::from_secs(60) {
                        warn!("Removing stale agent {}", id);
                        false
                    } else {
                        true
                    }
                });
            }
        });
        
        Ok(())
    }

    /// Start message handler
    async fn start_message_handler(&self, mut rx: mpsc::Receiver<CoordinatorMessage>) {
        let agents = self.agents.clone();
        let tasks = self.tasks.clone();
        let task_queue = self.task_queue.clone();
        let broadcast_tx = self.broadcast_channel.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    CoordinatorMessage::RegisterAgent { agent_info } => {
                        info!("Registering agent: {} ({})", agent_info.id, agent_info.agent_type);
                        agents.write().await.insert(agent_info.id.clone(), agent_info);
                    }
                    
                    CoordinatorMessage::UnregisterAgent { agent_id } => {
                        info!("Unregistering agent: {}", agent_id);
                        agents.write().await.remove(&agent_id);
                    }
                    
                    CoordinatorMessage::SubmitTask { task } => {
                        info!("New task submitted: {} ({})", task.id, task.task_type);
                        task_queue.write().await.push(task);
                    }
                    
                    CoordinatorMessage::AgentHeartbeat { agent_id, metrics } => {
                        if let Some(agent) = agents.write().await.get_mut(&agent_id) {
                            agent.last_heartbeat = Instant::now();
                            agent.metrics = metrics;
                        }
                    }
                    
                    CoordinatorMessage::TaskCompleted { task_id, agent_id, result } => {
                        info!("Task {} completed by agent {} (success: {})", 
                              task_id, agent_id, result.success);
                        
                        if let Some(task) = tasks.write().await.get_mut(&task_id) {
                            task.status = if result.success {
                                TaskStatus::Completed
                            } else {
                                TaskStatus::Failed
                            };
                        }
                        
                        // Update agent status
                        if let Some(agent) = agents.write().await.get_mut(&agent_id) {
                            agent.status = AgentStatus::Idle;
                        }
                    }
                    
                    CoordinatorMessage::RequestConsensus { proposal, timeout } => {
                        info!("Consensus requested for proposal: {}", proposal);
                        // Broadcast to all agents
                        let _ = broadcast_tx.send(msg);
                    }
                    
                    CoordinatorMessage::EmergencyStop => {
                        error!("Emergency stop requested!");
                        // Broadcast to all agents
                        let _ = broadcast_tx.send(msg);
                    }
                }
            }
        });
    }

    /// Set coordinator state
    async fn set_state(&self, new_state: CoordinationState) {
        *self.state.write().await = new_state;
    }

    /// Get current state
    pub async fn get_state(&self) -> CoordinationState {
        self.state.read().await.clone()
    }

    /// Get registered agents
    pub async fn get_agents(&self) -> Vec<AgentInfo> {
        self.agents.read().await.values().cloned().collect()
    }

    /// Get active tasks
    pub async fn get_tasks(&self) -> Vec<CoordinationTask> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// Submit a new task
    pub async fn submit_task(&self, task: CoordinationTask) -> Result<(), Box<dyn std::error::Error>> {
        self.message_channel.send(CoordinatorMessage::SubmitTask { task }).await?;
        Ok(())
    }

    /// Register an agent
    pub async fn register_agent(&self, agent_info: AgentInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.message_channel.send(CoordinatorMessage::RegisterAgent { agent_info }).await?;
        Ok(())
    }

    /// Get broadcast receiver
    pub fn subscribe(&self) -> broadcast::Receiver<CoordinatorMessage> {
        self.broadcast_channel.subscribe()
    }

    /// Shutdown the coordinator
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down Coordinator Agent {}", self.id);
        
        // Broadcast emergency stop
        let _ = self.broadcast_channel.send(CoordinatorMessage::EmergencyStop);
        
        // Signal shutdown
        self.shutdown_signal.notify_one();
        
        // Wait for autonomy loop
        if let Some(handle) = self.autonomy_handle.take() {
            handle.await?;
        }
        
        info!("Coordinator Agent {} shutdown complete", self.id);
        Ok(())
    }
}