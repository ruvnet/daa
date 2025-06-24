//! Async swarm coordination with hierarchical structure

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use tokio::sync::{RwLock, mpsc, broadcast, Semaphore};
use tokio::time::{Duration, timeout};
use futures::future::join_all;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Agent identifier
pub type AgentId = String;

/// Message types for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Task assignment
    TaskAssignment { task_id: String, payload: Vec<u8> },
    /// Task result
    TaskResult { task_id: String, result: Vec<u8> },
    /// Heartbeat
    Heartbeat { agent_id: AgentId, timestamp: u64 },
    /// Broadcast message
    Broadcast { content: Vec<u8> },
    /// Point-to-point message
    P2P { from: AgentId, to: AgentId, content: Vec<u8> },
    /// Control message
    Control { command: String, params: HashMap<String, String> },
}

/// Agent trait for async operations
#[async_trait]
pub trait AsyncAgent: Send + Sync {
    /// Get agent ID
    fn id(&self) -> &AgentId;
    
    /// Process a message
    async fn process_message(&self, message: AgentMessage) -> Result<Option<AgentMessage>, AgentError>;
    
    /// Execute a task
    async fn execute_task(&self, task: Task) -> Result<TaskResult, AgentError>;
    
    /// Get agent status
    async fn status(&self) -> AgentStatus;
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub id: AgentId,
    pub state: AgentState,
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub last_heartbeat: u64,
}

/// Agent state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Working,
    Overloaded,
    Failed,
}

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub payload: Vec<u8>,
    pub priority: TaskPriority,
    pub timeout: Duration,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: AgentId,
    pub result: Vec<u8>,
    pub execution_time: Duration,
}

/// Agent error types
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Communication error: {0}")]
    Communication(String),
    #[error("Task execution error: {0}")]
    TaskExecution(String),
    #[error("Timeout error")]
    Timeout,
    #[error("Agent overloaded")]
    Overloaded,
}

/// Hierarchical swarm coordinator
pub struct HierarchicalSwarm {
    /// Root coordinator
    root: Arc<RwLock<CoordinatorNode>>,
    /// Agent registry
    agents: Arc<RwLock<HashMap<AgentId, Arc<dyn AsyncAgent>>>>,
    /// Communication channels
    channels: Arc<RwLock<HashMap<AgentId, mpsc::Sender<AgentMessage>>>>,
    /// Broadcast channel
    broadcast_tx: broadcast::Sender<AgentMessage>,
    /// Task queue
    task_queue: Arc<RwLock<TaskQueue>>,
    /// Configuration
    config: SwarmConfig,
    /// Statistics
    stats: Arc<SwarmStats>,
}

/// Swarm configuration
#[derive(Debug, Clone)]
pub struct SwarmConfig {
    /// Maximum agents per coordinator
    pub max_agents_per_coordinator: usize,
    /// Hierarchy depth
    pub max_hierarchy_depth: usize,
    /// Communication timeout
    pub communication_timeout: Duration,
    /// Task distribution strategy
    pub distribution_strategy: DistributionStrategy,
    /// Enable work stealing
    pub enable_work_stealing: bool,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            max_agents_per_coordinator: 10,
            max_hierarchy_depth: 3,
            communication_timeout: Duration::from_secs(5),
            distribution_strategy: DistributionStrategy::LoadBalanced,
            enable_work_stealing: true,
            heartbeat_interval: Duration::from_secs(10),
        }
    }
}

/// Task distribution strategies
#[derive(Debug, Clone, Copy)]
pub enum DistributionStrategy {
    RoundRobin,
    LoadBalanced,
    PriorityBased,
    Affinity,
}

/// Coordinator node in hierarchy
struct CoordinatorNode {
    /// Node ID
    id: String,
    /// Parent coordinator
    parent: Option<Arc<RwLock<CoordinatorNode>>>,
    /// Child coordinators
    children: Vec<Arc<RwLock<CoordinatorNode>>>,
    /// Managed agents
    agents: HashSet<AgentId>,
    /// Node statistics
    stats: NodeStats,
}

/// Node statistics
#[derive(Default)]
struct NodeStats {
    /// Tasks distributed
    tasks_distributed: AtomicU64,
    /// Messages routed
    messages_routed: AtomicU64,
}

/// Task queue with priority support
struct TaskQueue {
    /// Priority queues
    queues: HashMap<TaskPriority, Vec<Task>>,
    /// Task assignments
    assignments: HashMap<String, AgentId>,
    /// Pending tasks
    pending: HashSet<String>,
}

impl TaskQueue {
    fn new() -> Self {
        let mut queues = HashMap::new();
        queues.insert(TaskPriority::Critical, Vec::new());
        queues.insert(TaskPriority::High, Vec::new());
        queues.insert(TaskPriority::Normal, Vec::new());
        queues.insert(TaskPriority::Low, Vec::new());
        
        Self {
            queues,
            assignments: HashMap::new(),
            pending: HashSet::new(),
        }
    }
    
    fn push(&mut self, task: Task) {
        self.pending.insert(task.id.clone());
        self.queues.get_mut(&task.priority).unwrap().push(task);
    }
    
    fn pop(&mut self) -> Option<Task> {
        // Pop from highest priority queue first
        for priority in [TaskPriority::Critical, TaskPriority::High, TaskPriority::Normal, TaskPriority::Low] {
            if let Some(task) = self.queues.get_mut(&priority).and_then(|q| q.pop()) {
                self.pending.remove(&task.id);
                return Some(task);
            }
        }
        None
    }
}

/// Swarm statistics
#[derive(Default)]
struct SwarmStats {
    /// Total tasks executed
    total_tasks: AtomicU64,
    /// Failed tasks
    failed_tasks: AtomicU64,
    /// Average execution time (microseconds)
    avg_execution_time: AtomicU64,
    /// Active agents
    active_agents: AtomicU64,
}

impl HierarchicalSwarm {
    /// Create a new hierarchical swarm
    pub fn new(config: SwarmConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            root: Arc::new(RwLock::new(CoordinatorNode {
                id: "root".to_string(),
                parent: None,
                children: Vec::new(),
                agents: HashSet::new(),
                stats: NodeStats::default(),
            })),
            agents: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            config,
            stats: Arc::new(SwarmStats::default()),
        }
    }
    
    /// Add an agent to the swarm
    pub async fn add_agent(&self, agent: Arc<dyn AsyncAgent>) -> Result<(), AgentError> {
        let agent_id = agent.id().clone();
        
        // Create communication channel
        let (tx, mut rx) = mpsc::channel(100);
        
        // Register agent
        self.agents.write().await.insert(agent_id.clone(), agent.clone());
        self.channels.write().await.insert(agent_id.clone(), tx);
        
        // Assign to coordinator
        self.assign_to_coordinator(&agent_id).await?;
        
        // Start agent message handler
        let agent_clone = agent.clone();
        let broadcast_rx = self.broadcast_tx.subscribe();
        tokio::spawn(async move {
            Self::agent_message_handler(agent_clone, rx, broadcast_rx).await;
        });
        
        self.stats.active_agents.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Assign agent to appropriate coordinator in hierarchy
    async fn assign_to_coordinator(&self, agent_id: &AgentId) -> Result<(), AgentError> {
        let mut root = self.root.write().await;
        
        // Simple assignment - could be more sophisticated
        if root.agents.len() < self.config.max_agents_per_coordinator {
            root.agents.insert(agent_id.clone());
        } else {
            // Create or find child coordinator
            // This is simplified - real implementation would balance the tree
            root.agents.insert(agent_id.clone());
        }
        
        Ok(())
    }
    
    /// Agent message handler
    async fn agent_message_handler(
        agent: Arc<dyn AsyncAgent>,
        mut rx: mpsc::Receiver<AgentMessage>,
        mut broadcast_rx: broadcast::Receiver<AgentMessage>,
    ) {
        loop {
            tokio::select! {
                // Handle direct messages
                Some(msg) = rx.recv() => {
                    match agent.process_message(msg).await {
                        Ok(Some(response)) => {
                            // Handle response
                        }
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!("Agent {} error: {}", agent.id(), e);
                        }
                    }
                }
                
                // Handle broadcast messages
                Ok(msg) = broadcast_rx.recv() => {
                    let _ = agent.process_message(msg).await;
                }
                
                else => break,
            }
        }
    }
    
    /// Submit a task to the swarm
    pub async fn submit_task(&self, task: Task) -> Result<(), AgentError> {
        // Add to queue
        self.task_queue.write().await.push(task.clone());
        
        // Trigger distribution
        self.distribute_tasks().await?;
        
        Ok(())
    }
    
    /// Distribute tasks to agents
    async fn distribute_tasks(&self) -> Result<(), AgentError> {
        let mut queue = self.task_queue.write().await;
        let agents = self.agents.read().await;
        let channels = self.channels.read().await;
        
        while let Some(task) = queue.pop() {
            // Select agent based on strategy
            let agent_id = match self.config.distribution_strategy {
                DistributionStrategy::LoadBalanced => {
                    self.select_least_loaded_agent(&agents).await?
                }
                DistributionStrategy::RoundRobin => {
                    self.select_round_robin_agent(&agents).await?
                }
                _ => {
                    // Simplified - just pick first available
                    agents.keys().next().cloned()
                        .ok_or_else(|| AgentError::Communication("No agents available".into()))?
                }
            };
            
            // Send task to agent
            if let Some(tx) = channels.get(&agent_id) {
                let msg = AgentMessage::TaskAssignment {
                    task_id: task.id.clone(),
                    payload: task.payload,
                };
                
                queue.assignments.insert(task.id, agent_id.clone());
                
                if let Err(_) = tx.send(msg).await {
                    return Err(AgentError::Communication("Failed to send task".into()));
                }
            }
        }
        
        Ok(())
    }
    
    /// Select least loaded agent
    async fn select_least_loaded_agent(&self, agents: &HashMap<AgentId, Arc<dyn AsyncAgent>>) -> Result<AgentId, AgentError> {
        let mut best_agent = None;
        let mut min_tasks = usize::MAX;
        
        for (id, agent) in agents {
            let status = agent.status().await;
            if status.state != AgentState::Overloaded && status.active_tasks < min_tasks {
                min_tasks = status.active_tasks;
                best_agent = Some(id.clone());
            }
        }
        
        best_agent.ok_or_else(|| AgentError::Communication("No available agents".into()))
    }
    
    /// Select agent using round-robin
    async fn select_round_robin_agent(&self, agents: &HashMap<AgentId, Arc<dyn AsyncAgent>>) -> Result<AgentId, AgentError> {
        // Simplified - in production would maintain round-robin state
        agents.keys().next().cloned()
            .ok_or_else(|| AgentError::Communication("No agents available".into()))
    }
    
    /// Broadcast message to all agents
    pub async fn broadcast(&self, message: AgentMessage) -> Result<(), AgentError> {
        self.broadcast_tx.send(message)
            .map_err(|_| AgentError::Communication("Broadcast failed".into()))?;
        Ok(())
    }
    
    /// Send point-to-point message
    pub async fn send_p2p(&self, to: &AgentId, message: AgentMessage) -> Result<(), AgentError> {
        let channels = self.channels.read().await;
        
        if let Some(tx) = channels.get(to) {
            tx.send(message).await
                .map_err(|_| AgentError::Communication("P2P send failed".into()))?;
            Ok(())
        } else {
            Err(AgentError::Communication("Agent not found".into()))
        }
    }
    
    /// Execute parallel tasks across the swarm
    pub async fn parallel_execute<F, R>(&self, tasks: Vec<Task>, handler: F) -> Vec<Result<R, AgentError>>
    where
        F: Fn(Task) -> R + Send + Sync + Clone,
        R: Send + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(self.agents.read().await.len()));
        
        let futures = tasks.into_iter().map(|task| {
            let sem = semaphore.clone();
            let handler = handler.clone();
            
            async move {
                let _permit = sem.acquire().await.map_err(|_| AgentError::Communication("Semaphore error".into()))?;
                
                // Execute with timeout
                match timeout(task.timeout, tokio::task::spawn_blocking(move || handler(task))).await {
                    Ok(Ok(result)) => Ok(result),
                    Ok(Err(_)) => Err(AgentError::TaskExecution("Task panicked".into())),
                    Err(_) => Err(AgentError::Timeout),
                }
            }
        });
        
        join_all(futures).await
    }
    
    /// Get swarm statistics
    pub fn get_stats(&self) -> SwarmStatistics {
        SwarmStatistics {
            total_tasks: self.stats.total_tasks.load(Ordering::Relaxed),
            failed_tasks: self.stats.failed_tasks.load(Ordering::Relaxed),
            avg_execution_time_us: self.stats.avg_execution_time.load(Ordering::Relaxed),
            active_agents: self.stats.active_agents.load(Ordering::Relaxed),
        }
    }
}

/// Swarm statistics for external reporting
#[derive(Debug, Clone)]
pub struct SwarmStatistics {
    pub total_tasks: u64,
    pub failed_tasks: u64,
    pub avg_execution_time_us: u64,
    pub active_agents: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock agent for testing
    struct MockAgent {
        id: AgentId,
        active_tasks: Arc<AtomicU64>,
    }
    
    #[async_trait]
    impl AsyncAgent for MockAgent {
        fn id(&self) -> &AgentId {
            &self.id
        }
        
        async fn process_message(&self, _message: AgentMessage) -> Result<Option<AgentMessage>, AgentError> {
            Ok(None)
        }
        
        async fn execute_task(&self, task: Task) -> Result<TaskResult, AgentError> {
            self.active_tasks.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_millis(10)).await;
            self.active_tasks.fetch_sub(1, Ordering::Relaxed);
            
            Ok(TaskResult {
                task_id: task.id,
                agent_id: self.id.clone(),
                result: vec![42],
                execution_time: Duration::from_millis(10),
            })
        }
        
        async fn status(&self) -> AgentStatus {
            AgentStatus {
                id: self.id.clone(),
                state: AgentState::Idle,
                active_tasks: self.active_tasks.load(Ordering::Relaxed) as usize,
                completed_tasks: 0,
                last_heartbeat: 0,
            }
        }
    }
    
    #[tokio::test]
    async fn test_hierarchical_swarm() {
        let config = SwarmConfig::default();
        let swarm = HierarchicalSwarm::new(config);
        
        // Add agents
        for i in 0..5 {
            let agent = Arc::new(MockAgent {
                id: format!("agent_{}", i),
                active_tasks: Arc::new(AtomicU64::new(0)),
            });
            swarm.add_agent(agent).await.unwrap();
        }
        
        // Submit tasks
        for i in 0..10 {
            let task = Task {
                id: format!("task_{}", i),
                payload: vec![i as u8],
                priority: TaskPriority::Normal,
                timeout: Duration::from_secs(1),
            };
            swarm.submit_task(task).await.unwrap();
        }
        
        // Wait for completion
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check stats
        let stats = swarm.get_stats();
        assert_eq!(stats.active_agents, 5);
    }
}