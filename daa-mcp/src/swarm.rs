//! Swarm Coordination System for DAA MCP
//! 
//! This module implements a distributed swarm coordination system that manages
//! multiple agents working together on complex tasks through parallel execution.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock, Mutex};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    discovery::{AgentDiscoveryInfo, DiscoveryFilter, DiscoveryProtocol, DiscoveryUtils},
    DaaMcpError, DaaTask, McpServerState, Result, SwarmMessage, SwarmMessageType, TaskPriority, TaskResult, TaskStatus,
};

/// Swarm coordination strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmStrategy {
    /// Centralized coordination with a single coordinator
    Centralized,
    /// Distributed peer-to-peer coordination
    Distributed,
    /// Hierarchical coordination with multiple levels
    Hierarchical,
    /// Mesh network coordination
    Mesh,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

/// Swarm coordination modes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmMode {
    /// Research-focused swarm
    Research,
    /// Development-focused swarm
    Development,
    /// Analysis and monitoring swarm
    Analysis,
    /// Testing and validation swarm
    Testing,
    /// Optimization and performance swarm
    Optimization,
    /// Maintenance and operations swarm
    Maintenance,
}

/// Swarm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub id: String,
    pub name: String,
    pub objective: String,
    pub strategy: SwarmStrategy,
    pub mode: SwarmMode,
    pub max_agents: usize,
    pub min_agents: usize,
    pub coordination_interval: Duration,
    pub task_distribution_method: TaskDistributionMethod,
    pub enable_parallel_execution: bool,
    pub enable_load_balancing: bool,
    pub failure_tolerance: f32,
}

/// Task distribution methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskDistributionMethod {
    /// Round-robin distribution
    RoundRobin,
    /// Load-based distribution
    LoadBased,
    /// Capability-based distribution
    CapabilityBased,
    /// Priority-based distribution
    PriorityBased,
    /// Random distribution
    Random,
}

/// Swarm agent role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmRole {
    /// Coordinator agent that manages the swarm
    Coordinator,
    /// Worker agent that executes tasks
    Worker,
    /// Monitor agent that tracks performance
    Monitor,
    /// Resource agent that manages shared resources
    Resource,
}

/// Swarm agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAgent {
    pub id: String,
    pub discovery_info: AgentDiscoveryInfo,
    pub role: SwarmRole,
    pub assigned_tasks: Vec<String>,
    pub current_load: f32,
    pub performance_score: f32,
    pub joined_at: SystemTime,
    pub last_heartbeat: SystemTime,
}

/// Swarm state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmState {
    pub id: String,
    pub config: SwarmConfig,
    pub agents: HashMap<String, SwarmAgent>,
    pub pending_tasks: VecDeque<DaaTask>,
    pub active_tasks: HashMap<String, DaaTask>,
    pub completed_tasks: HashMap<String, TaskResult>,
    pub message_queue: VecDeque<SwarmMessage>,
    pub created_at: SystemTime,
    pub last_coordination: SystemTime,
    pub status: SwarmStatus,
}

/// Swarm status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmStatus {
    Initializing,
    Active,
    Paused,
    Scaling,
    Degraded,
    Terminating,
    Terminated,
}

/// Swarm coordinator that manages multiple agent swarms
pub struct SwarmCoordinator {
    server_state: Arc<McpServerState>,
    discovery: Arc<DiscoveryProtocol>,
    swarms: Arc<RwLock<HashMap<String, SwarmState>>>,
    message_bus: Arc<RwLock<broadcast::Sender<SwarmMessage>>>,
    task_scheduler: Arc<Mutex<TaskScheduler>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl SwarmCoordinator {
    /// Create a new swarm coordinator
    pub async fn new(
        server_state: Arc<McpServerState>,
        discovery: Arc<DiscoveryProtocol>,
    ) -> Result<Self> {
        let (message_tx, _) = broadcast::channel(1000);
        let task_scheduler = TaskScheduler::new();

        Ok(Self {
            server_state,
            discovery,
            swarms: Arc::new(RwLock::new(HashMap::new())),
            message_bus: Arc::new(RwLock::new(message_tx)),
            task_scheduler: Arc::new(Mutex::new(task_scheduler)),
            shutdown_tx: None,
        })
    }

    /// Start the swarm coordination system
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        // Start coordination loop
        self.start_coordination_loop(shutdown_tx.subscribe()).await?;

        // Start message processing
        self.start_message_processor(shutdown_tx.subscribe()).await?;

        // Start task scheduler
        self.start_task_scheduler(shutdown_tx.subscribe()).await?;

        info!("Swarm coordinator started successfully");
        Ok(())
    }

    /// Stop the swarm coordination system
    pub async fn stop(&self) -> Result<()> {
        if let Some(ref shutdown_tx) = self.shutdown_tx {
            // Terminate all swarms gracefully
            let swarms = self.swarms.read().await;
            for swarm_id in swarms.keys() {
                if let Err(e) = self.terminate_swarm_internal(swarm_id).await {
                    warn!("Error terminating swarm {}: {}", swarm_id, e);
                }
            }

            // Signal shutdown
            let _ = shutdown_tx.send(());
        }

        info!("Swarm coordinator stopped");
        Ok(())
    }

    /// Create and deploy a new swarm
    pub async fn create_swarm(
        &self,
        config: SwarmConfig,
        required_agent_types: Vec<String>,
    ) -> Result<String> {
        let swarm_id = config.id.clone();
        
        info!("Creating swarm {} with {} required agent types", swarm_id, required_agent_types.len());

        // Discover suitable agents
        let agents = self.discover_swarm_agents(&required_agent_types, config.max_agents).await?;
        
        if agents.len() < config.min_agents {
            return Err(DaaMcpError::Protocol(format!(
                "Insufficient agents found. Required: {}, Found: {}",
                config.min_agents, agents.len()
            )));
        }

        // Create swarm state
        let mut swarm_state = SwarmState {
            id: swarm_id.clone(),
            config: config.clone(),
            agents: HashMap::new(),
            pending_tasks: VecDeque::new(),
            active_tasks: HashMap::new(),
            completed_tasks: HashMap::new(),
            message_queue: VecDeque::new(),
            created_at: SystemTime::now(),
            last_coordination: SystemTime::now(),
            status: SwarmStatus::Initializing,
        };

        // Assign roles and add agents to swarm
        self.assign_agent_roles(&mut swarm_state, agents).await?;

        // Store swarm state
        {
            let mut swarms = self.swarms.write().await;
            swarms.insert(swarm_id.clone(), swarm_state);
        }

        // Send initialization messages to agents
        self.initialize_swarm_agents(&swarm_id).await?;

        // Mark swarm as active
        self.update_swarm_status(&swarm_id, SwarmStatus::Active).await?;

        info!("Swarm {} created successfully with {} agents", swarm_id, config.max_agents);
        Ok(swarm_id)
    }

    /// Terminate a swarm
    pub async fn terminate_swarm(&self, swarm_id: &str) -> Result<()> {
        self.terminate_swarm_internal(swarm_id).await
    }

    /// Add a task to a swarm for execution
    pub async fn add_swarm_task(&self, swarm_id: &str, task: DaaTask) -> Result<()> {
        let mut swarms = self.swarms.write().await;
        
        if let Some(swarm) = swarms.get_mut(swarm_id) {
            swarm.pending_tasks.push_back(task.clone());
            
            // Notify task scheduler
            let mut scheduler = self.task_scheduler.lock().await;
            scheduler.schedule_task(swarm_id, task).await?;
            
            info!("Task {} added to swarm {}", task.id, swarm_id);
            Ok(())
        } else {
            Err(DaaMcpError::Protocol(format!("Swarm not found: {}", swarm_id)))
        }
    }

    /// Get swarm status
    pub async fn get_swarm_status(&self, swarm_id: &str) -> Result<SwarmState> {
        let swarms = self.swarms.read().await;
        
        swarms.get(swarm_id)
            .cloned()
            .ok_or_else(|| DaaMcpError::Protocol(format!("Swarm not found: {}", swarm_id)))
    }

    /// List all active swarms
    pub async fn list_swarms(&self) -> Vec<SwarmState> {
        let swarms = self.swarms.read().await;
        swarms.values().cloned().collect()
    }

    /// Send a message to swarm agents
    pub async fn send_swarm_message(
        &self,
        swarm_id: &str,
        message_type: SwarmMessageType,
        payload: serde_json::Value,
        target_agents: Option<Vec<String>>,
    ) -> Result<()> {
        let message = SwarmMessage {
            id: Uuid::new_v4().to_string(),
            from_agent: "swarm_coordinator".to_string(),
            to_agents: target_agents.unwrap_or_default(),
            message_type,
            payload,
            timestamp: chrono::Utc::now(),
            ttl: Some(300), // 5 minutes TTL
        };

        // Add to swarm message queue
        {
            let mut swarms = self.swarms.write().await;
            if let Some(swarm) = swarms.get_mut(swarm_id) {
                swarm.message_queue.push_back(message.clone());
            }
        }

        // Broadcast to message bus
        let message_bus = self.message_bus.read().await;
        let _ = message_bus.send(message);

        Ok(())
    }

    /// Discover suitable agents for swarm formation
    async fn discover_swarm_agents(
        &self,
        required_types: &[String],
        max_agents: usize,
    ) -> Result<Vec<AgentDiscoveryInfo>> {
        let mut all_agents = Vec::new();

        // Discover agents for each required type
        for agent_type in required_types {
            let filter = DiscoveryFilter {
                agent_type: Some(agent_type.clone()),
                capabilities: None,
                max_agents: Some(max_agents / required_types.len().max(1)),
                exclude_self: true,
            };

            let agents = self.discovery.discover_agents(filter).await?;
            all_agents.extend(agents);
        }

        // Remove duplicates and sort by suitability
        let mut unique_agents: Vec<_> = all_agents.into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Sort by suitability (availability, load, response time)
        unique_agents.sort_by(|a, b| {
            let score_a = Self::calculate_agent_score(a);
            let score_b = Self::calculate_agent_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        unique_agents.truncate(max_agents);
        Ok(unique_agents)
    }

    /// Calculate agent suitability score
    fn calculate_agent_score(agent: &AgentDiscoveryInfo) -> f32 {
        let availability_score = match agent.availability {
            crate::discovery::AgentAvailability::Available => 1.0,
            crate::discovery::AgentAvailability::Busy => 0.7,
            crate::discovery::AgentAvailability::Overloaded => 0.3,
            crate::discovery::AgentAvailability::Maintenance => 0.0,
        };

        let load_score = 1.0 - agent.load_factor;
        let response_score = 1.0 - (agent.response_time_avg / 1000.0).min(1.0);

        (availability_score * 0.4) + (load_score * 0.3) + (response_score * 0.3)
    }

    /// Assign roles to agents in the swarm
    async fn assign_agent_roles(
        &self,
        swarm_state: &mut SwarmState,
        agents: Vec<AgentDiscoveryInfo>,
    ) -> Result<()> {
        let total_agents = agents.len();
        
        for (index, agent_info) in agents.into_iter().enumerate() {
            let role = match swarm_state.config.strategy {
                SwarmStrategy::Centralized => {
                    if index == 0 {
                        SwarmRole::Coordinator
                    } else if index == 1 && total_agents > 2 {
                        SwarmRole::Monitor
                    } else {
                        SwarmRole::Worker
                    }
                }
                SwarmStrategy::Hierarchical => {
                    if index == 0 {
                        SwarmRole::Coordinator
                    } else if index < total_agents / 4 {
                        SwarmRole::Monitor
                    } else {
                        SwarmRole::Worker
                    }
                }
                _ => {
                    // For distributed/mesh strategies, assign roles based on capabilities
                    if agent_info.capabilities.contains(&"coordination".to_string()) {
                        SwarmRole::Coordinator
                    } else if agent_info.capabilities.contains(&"monitoring".to_string()) {
                        SwarmRole::Monitor
                    } else {
                        SwarmRole::Worker
                    }
                }
            };

            let swarm_agent = SwarmAgent {
                id: agent_info.id.clone(),
                discovery_info: agent_info,
                role,
                assigned_tasks: Vec::new(),
                current_load: 0.0,
                performance_score: 1.0,
                joined_at: SystemTime::now(),
                last_heartbeat: SystemTime::now(),
            };

            swarm_state.agents.insert(swarm_agent.id.clone(), swarm_agent);
        }

        Ok(())
    }

    /// Initialize agents in the swarm
    async fn initialize_swarm_agents(&self, swarm_id: &str) -> Result<()> {
        let swarms = self.swarms.read().await;
        
        if let Some(swarm) = swarms.get(swarm_id) {
            for agent in swarm.agents.values() {
                let init_message = serde_json::json!({
                    "swarm_id": swarm_id,
                    "agent_role": agent.role,
                    "swarm_config": swarm.config,
                    "coordination_strategy": swarm.config.strategy
                });

                // Send initialization message to agent
                // In a real implementation, this would use the agent's MCP endpoint
                debug!("Initializing agent {} for swarm {}", agent.id, swarm_id);
            }
        }

        Ok(())
    }

    /// Update swarm status
    async fn update_swarm_status(&self, swarm_id: &str, status: SwarmStatus) -> Result<()> {
        let mut swarms = self.swarms.write().await;
        
        if let Some(swarm) = swarms.get_mut(swarm_id) {
            swarm.status = status;
            swarm.last_coordination = SystemTime::now();
        }

        Ok(())
    }

    /// Internal method to terminate a swarm
    async fn terminate_swarm_internal(&self, swarm_id: &str) -> Result<()> {
        // Update status to terminating
        self.update_swarm_status(swarm_id, SwarmStatus::Terminating).await?;

        // Send termination messages to agents
        self.send_swarm_message(
            swarm_id,
            SwarmMessageType::Coordination,
            serde_json::json!({"action": "terminate"}),
            None,
        ).await?;

        // Remove swarm from state
        {
            let mut swarms = self.swarms.write().await;
            swarms.remove(swarm_id);
        }

        info!("Swarm {} terminated", swarm_id);
        Ok(())
    }

    /// Start the coordination loop
    async fn start_coordination_loop(&self, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
        let swarms = self.swarms.clone();
        let discovery = self.discovery.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Swarm coordination loop shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        let swarms_guard = swarms.read().await;
                        
                        for (swarm_id, swarm) in swarms_guard.iter() {
                            // Check agent health and availability
                            Self::check_swarm_health(swarm_id, swarm, &discovery).await;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Check swarm health and agent availability
    async fn check_swarm_health(
        swarm_id: &str,
        swarm: &SwarmState,
        discovery: &Arc<DiscoveryProtocol>,
    ) {
        let now = SystemTime::now();
        let mut unhealthy_agents = 0;

        for agent in swarm.agents.values() {
            // Check if agent has sent recent heartbeat
            if let Ok(duration) = now.duration_since(agent.last_heartbeat) {
                if duration > Duration::from_secs(120) { // 2 minutes timeout
                    unhealthy_agents += 1;
                    warn!("Agent {} in swarm {} appears unhealthy", agent.id, swarm_id);
                }
            }
        }

        // Check if swarm needs scaling or replacement agents
        let healthy_ratio = 1.0 - (unhealthy_agents as f32 / swarm.agents.len() as f32);
        if healthy_ratio < swarm.config.failure_tolerance {
            warn!("Swarm {} is degraded: {:.1}% healthy agents", swarm_id, healthy_ratio * 100.0);
            // Could trigger agent replacement logic here
        }
    }

    /// Start message processor
    async fn start_message_processor(&self, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
        let message_bus = self.message_bus.clone();

        tokio::spawn(async move {
            let message_rx = {
                let bus = message_bus.read().await;
                bus.subscribe()
            };
            let mut rx = message_rx;

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Swarm message processor shutting down");
                        break;
                    }
                    result = rx.recv() => {
                        match result {
                            Ok(message) => {
                                debug!("Processing swarm message: {:?}", message);
                                // Process message based on type
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Start task scheduler
    async fn start_task_scheduler(&self, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
        let scheduler = self.task_scheduler.clone();
        let swarms = self.swarms.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Swarm task scheduler shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        let mut sched = scheduler.lock().await;
                        let swarms_guard = swarms.read().await;
                        
                        for (swarm_id, swarm_state) in swarms_guard.iter() {
                            if let Err(e) = sched.process_pending_tasks(swarm_id, swarm_state).await {
                                warn!("Error processing tasks for swarm {}: {}", swarm_id, e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

/// Task scheduler for swarm coordination
pub struct TaskScheduler {
    pending_assignments: HashMap<String, Vec<DaaTask>>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new() -> Self {
        Self {
            pending_assignments: HashMap::new(),
        }
    }

    /// Schedule a task for execution
    pub async fn schedule_task(&mut self, swarm_id: &str, task: DaaTask) -> Result<()> {
        self.pending_assignments
            .entry(swarm_id.to_string())
            .or_insert_with(Vec::new)
            .push(task);
        
        Ok(())
    }

    /// Process pending tasks for a swarm
    pub async fn process_pending_tasks(
        &mut self,
        swarm_id: &str,
        swarm_state: &SwarmState,
    ) -> Result<()> {
        if let Some(tasks) = self.pending_assignments.get_mut(swarm_id) {
            if !tasks.is_empty() && matches!(swarm_state.status, SwarmStatus::Active) {
                // Distribute tasks based on the configured method
                self.distribute_tasks(swarm_state, tasks).await?;
                tasks.clear();
            }
        }

        Ok(())
    }

    /// Distribute tasks to swarm agents
    async fn distribute_tasks(
        &self,
        swarm_state: &SwarmState,
        tasks: &[DaaTask],
    ) -> Result<()> {
        let worker_agents: Vec<_> = swarm_state.agents.values()
            .filter(|agent| matches!(agent.role, SwarmRole::Worker))
            .collect();

        if worker_agents.is_empty() {
            warn!("No worker agents available in swarm {}", swarm_state.id);
            return Ok(());
        }

        for task in tasks {
            let selected_agent = match swarm_state.config.task_distribution_method {
                TaskDistributionMethod::LoadBased => {
                    // Select agent with lowest current load
                    worker_agents.iter()
                        .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap()
                }
                TaskDistributionMethod::CapabilityBased => {
                    // Select agent with best capability match
                    self.select_best_capability_match(&worker_agents, task)
                }
                TaskDistributionMethod::PriorityBased => {
                    // Select based on task priority and agent performance
                    self.select_by_priority(&worker_agents, task)
                }
                _ => {
                    // Default to round-robin
                    &worker_agents[0] // Simplified for this implementation
                }
            };

            info!("Assigned task {} to agent {} in swarm {}", task.id, selected_agent.id, swarm_state.id);
        }

        Ok(())
    }

    /// Select agent with best capability match
    fn select_best_capability_match<'a>(
        &self,
        agents: &[&'a SwarmAgent],
        task: &DaaTask,
    ) -> &'a SwarmAgent {
        // Extract required capabilities from task parameters
        let required_capabilities: Vec<String> = task.parameters.get("required_capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        agents.iter()
            .max_by(|a, b| {
                let score_a = DiscoveryUtils::compatibility_score(&a.discovery_info, &required_capabilities);
                let score_b = DiscoveryUtils::compatibility_score(&b.discovery_info, &required_capabilities);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&agents[0])
    }

    /// Select agent based on priority and performance
    fn select_by_priority<'a>(
        &self,
        agents: &[&'a SwarmAgent],
        task: &DaaTask,
    ) -> &'a SwarmAgent {
        let priority_weight = match task.priority {
            TaskPriority::Critical => 1.0,
            TaskPriority::High => 0.8,
            TaskPriority::Medium => 0.6,
            TaskPriority::Low => 0.4,
        };

        agents.iter()
            .max_by(|a, b| {
                let score_a = a.performance_score * priority_weight - a.current_load * 0.3;
                let score_b = b.performance_score * priority_weight - b.current_load * 0.3;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&agents[0])
    }
}

/// Pre-configured swarm templates for common use cases
pub struct SwarmTemplates;

impl SwarmTemplates {
    /// Create a 3-agent research swarm
    pub fn research_swarm_3_agent(objective: String) -> SwarmConfig {
        SwarmConfig {
            id: Uuid::new_v4().to_string(),
            name: "Research Swarm".to_string(),
            objective,
            strategy: SwarmStrategy::Hierarchical,
            mode: SwarmMode::Research,
            max_agents: 3,
            min_agents: 3,
            coordination_interval: Duration::from_secs(60),
            task_distribution_method: TaskDistributionMethod::CapabilityBased,
            enable_parallel_execution: true,
            enable_load_balancing: true,
            failure_tolerance: 0.33, // Can tolerate losing 1 out of 3 agents
        }
    }

    /// Create a 3-agent development swarm
    pub fn development_swarm_3_agent(objective: String) -> SwarmConfig {
        SwarmConfig {
            id: Uuid::new_v4().to_string(),
            name: "Development Swarm".to_string(),
            objective,
            strategy: SwarmStrategy::Distributed,
            mode: SwarmMode::Development,
            max_agents: 3,
            min_agents: 3,
            coordination_interval: Duration::from_secs(30),
            task_distribution_method: TaskDistributionMethod::LoadBased,
            enable_parallel_execution: true,
            enable_load_balancing: true,
            failure_tolerance: 0.33,
        }
    }

    /// Create a 3-agent analysis swarm
    pub fn analysis_swarm_3_agent(objective: String) -> SwarmConfig {
        SwarmConfig {
            id: Uuid::new_v4().to_string(),
            name: "Analysis Swarm".to_string(),
            objective,
            strategy: SwarmStrategy::Mesh,
            mode: SwarmMode::Analysis,
            max_agents: 3,
            min_agents: 3,
            coordination_interval: Duration::from_secs(45),
            task_distribution_method: TaskDistributionMethod::PriorityBased,
            enable_parallel_execution: true,
            enable_load_balancing: true,
            failure_tolerance: 0.33,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DaaMcpConfig, discovery::DiscoveryConfig};

    #[test]
    fn test_swarm_templates() {
        let research_swarm = SwarmTemplates::research_swarm_3_agent("Research crypto markets".to_string());
        assert_eq!(research_swarm.max_agents, 3);
        assert_eq!(research_swarm.min_agents, 3);
        assert!(matches!(research_swarm.mode, SwarmMode::Research));

        let dev_swarm = SwarmTemplates::development_swarm_3_agent("Build trading bot".to_string());
        assert_eq!(dev_swarm.max_agents, 3);
        assert!(matches!(dev_swarm.mode, SwarmMode::Development));
    }

    #[test]
    fn test_agent_score_calculation() {
        use crate::discovery::{AgentDiscoveryInfo, AgentAvailability};

        let agent = AgentDiscoveryInfo {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            agent_type: "worker".to_string(),
            capabilities: vec!["analysis".to_string()],
            endpoint: "http://localhost:3002".to_string(),
            mcp_endpoint: None,
            last_seen: 0,
            availability: AgentAvailability::Available,
            load_factor: 0.3,
            response_time_avg: 150.0,
        };

        let score = SwarmCoordinator::calculate_agent_score(&agent);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[tokio::test]
    async fn test_task_scheduler() {
        let mut scheduler = TaskScheduler::new();
        
        let task = DaaTask {
            id: "test-task".to_string(),
            task_type: "analysis".to_string(),
            description: "Test task".to_string(),
            parameters: HashMap::new(),
            priority: TaskPriority::Medium,
            timeout: Some(300),
            dependencies: Vec::new(),
            assigned_agents: Vec::new(),
        };

        let result = scheduler.schedule_task("test-swarm", task).await;
        assert!(result.is_ok());
    }
}