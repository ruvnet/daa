//! Parameter Server DAA Agent Implementation
//! Distributed parameter management agent for machine learning and system parameters

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc, broadcast};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{debug, info, warn, error};

/// Parameter server state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterServerState {
    Initializing,
    Ready,
    Synchronizing,
    Aggregating,
    Persisting,
    Recovering,
    Failed(String),
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    ModelWeight,
    Hyperparameter,
    SystemConfig,
    UserPreference,
    Runtime,
    Custom(String),
}

/// Parameter entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub id: String,
    pub name: String,
    pub parameter_type: ParameterType,
    pub value: ParameterValue,
    pub version: u64,
    pub owner_id: String,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub metadata: HashMap<String, String>,
    pub constraints: Option<ParameterConstraints>,
}

/// Parameter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Float(f64),
    Integer(i64),
    String(String),
    Boolean(bool),
    Array(Vec<f64>),
    Matrix(Vec<Vec<f64>>),
    Tensor(Vec<usize>, Vec<f64>), // (dimensions, flat_data)
    Json(serde_json::Value),
}

/// Parameter constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraints {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<String>>,
    pub regex_pattern: Option<String>,
    pub required: bool,
}

/// Parameter update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterUpdate {
    pub parameter_id: String,
    pub new_value: ParameterValue,
    pub gradient: Option<Vec<f64>>,
    pub client_id: String,
    pub timestamp: Instant,
    pub metadata: HashMap<String, String>,
}

/// Aggregation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    Average,
    WeightedAverage,
    Median,
    Max,
    Min,
    FederatedAverage,
    Custom(String),
}

/// Parameter server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterServerConfig {
    pub max_parameters: usize,
    pub sync_interval_ms: u64,
    pub aggregation_strategy: AggregationStrategy,
    pub persistence_enabled: bool,
    pub backup_interval_ms: u64,
    pub version_history_limit: usize,
    pub consensus_required: bool,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

impl Default for ParameterServerConfig {
    fn default() -> Self {
        Self {
            max_parameters: 100000,
            sync_interval_ms: 5000,
            aggregation_strategy: AggregationStrategy::Average,
            persistence_enabled: true,
            backup_interval_ms: 30000,
            version_history_limit: 10,
            consensus_required: false,
            compression_enabled: true,
            encryption_enabled: false,
        }
    }
}

/// Parameter server messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterServerMessage {
    RegisterClient { client_id: String, capabilities: Vec<String> },
    UnregisterClient { client_id: String },
    SetParameter { parameter: Parameter },
    GetParameter { parameter_id: String, client_id: String },
    UpdateParameter { update: ParameterUpdate },
    BatchUpdate { updates: Vec<ParameterUpdate> },
    Synchronize { client_id: String },
    Aggregate { parameter_ids: Vec<String> },
    Checkpoint { path: String },
    Restore { path: String },
    GetMetrics,
}

/// Client registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub capabilities: Vec<String>,
    pub last_sync: Instant,
    pub active: bool,
}

/// Parameter server metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterServerMetrics {
    pub total_parameters: usize,
    pub active_clients: usize,
    pub sync_rate: f64,
    pub update_rate: f64,
    pub average_latency: Duration,
    pub storage_usage: f64,
    pub compression_ratio: f64,
}

/// Parameter Server DAA Agent
pub struct ParameterServerAgent {
    id: String,
    config: ParameterServerConfig,
    state: Arc<RwLock<ParameterServerState>>,
    parameters: Arc<RwLock<HashMap<String, Parameter>>>,
    parameter_history: Arc<RwLock<HashMap<String, Vec<Parameter>>>>,
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    pending_updates: Arc<RwLock<HashMap<String, Vec<ParameterUpdate>>>>,
    metrics: Arc<RwLock<ParameterServerMetrics>>,
    message_channel: mpsc::Sender<ParameterServerMessage>,
    broadcast_channel: broadcast::Sender<Parameter>,
    autonomy_handle: Option<tokio::task::JoinHandle<()>>,
    sync_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl ParameterServerAgent {
    /// Create a new parameter server agent
    pub async fn new(config: ParameterServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel(10000);
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        let agent = Self {
            id: Uuid::new_v4().to_string(),
            config,
            state: Arc::new(RwLock::new(ParameterServerState::Initializing)),
            parameters: Arc::new(RwLock::new(HashMap::new())),
            parameter_history: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            pending_updates: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ParameterServerMetrics {
                total_parameters: 0,
                active_clients: 0,
                sync_rate: 0.0,
                update_rate: 0.0,
                average_latency: Duration::from_millis(0),
                storage_usage: 0.0,
                compression_ratio: 1.0,
            })),
            message_channel: tx,
            broadcast_channel: broadcast_tx,
            autonomy_handle: None,
            sync_handle: None,
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        };

        // Start message handler
        agent.start_message_handler(rx).await;
        
        Ok(agent)
    }

    /// Initialize the parameter server
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Parameter Server Agent {}", self.id);
        self.set_state(ParameterServerState::Initializing).await;
        
        // Initialize default parameters
        self.initialize_default_parameters().await?;
        
        // Start autonomy loop
        self.start_autonomy_loop().await?;
        
        // Start synchronization loop
        self.start_sync_loop().await?;
        
        self.set_state(ParameterServerState::Ready).await;
        info!("Parameter Server Agent {} initialized", self.id);
        Ok(())
    }

    /// Initialize default parameters
    async fn initialize_default_parameters(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_params = vec![
            Parameter {
                id: "learning_rate".to_string(),
                name: "Learning Rate".to_string(),
                parameter_type: ParameterType::Hyperparameter,
                value: ParameterValue::Float(0.001),
                version: 1,
                owner_id: "system".to_string(),
                created_at: Instant::now(),
                updated_at: Instant::now(),
                metadata: HashMap::new(),
                constraints: Some(ParameterConstraints {
                    min_value: Some(0.0),
                    max_value: Some(1.0),
                    allowed_values: None,
                    regex_pattern: None,
                    required: true,
                }),
            },
            Parameter {
                id: "batch_size".to_string(),
                name: "Batch Size".to_string(),
                parameter_type: ParameterType::Hyperparameter,
                value: ParameterValue::Integer(32),
                version: 1,
                owner_id: "system".to_string(),
                created_at: Instant::now(),
                updated_at: Instant::now(),
                metadata: HashMap::new(),
                constraints: Some(ParameterConstraints {
                    min_value: Some(1.0),
                    max_value: Some(1024.0),
                    allowed_values: None,
                    regex_pattern: None,
                    required: true,
                }),
            },
        ];
        
        let mut params = self.parameters.write().await;
        for param in default_params {
            params.insert(param.id.clone(), param);
        }
        
        Ok(())
    }

    /// Start autonomy loop
    async fn start_autonomy_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.clone();
        let parameters = self.parameters.clone();
        let clients = self.clients.clone();
        let pending_updates = self.pending_updates.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let id = self.id.clone();

        let handle = tokio::spawn(async move {
            Self::run_autonomy_loop(
                id, state, parameters, clients, pending_updates, metrics, config, shutdown_signal
            ).await;
        });

        self.autonomy_handle = Some(handle);
        Ok(())
    }

    /// Main autonomy loop
    async fn run_autonomy_loop(
        id: String,
        state: Arc<RwLock<ParameterServerState>>,
        parameters: Arc<RwLock<HashMap<String, Parameter>>>,
        clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
        pending_updates: Arc<RwLock<HashMap<String, Vec<ParameterUpdate>>>>,
        metrics: Arc<RwLock<ParameterServerMetrics>>,
        config: ParameterServerConfig,
        shutdown_signal: Arc<tokio::sync::Notify>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000));
        
        info!("Parameter Server Agent {} autonomy loop started", id);

        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => {
                    info!("Parameter Server Agent {} received shutdown signal", id);
                    break;
                }
                
                _ = interval.tick() => {
                    // Process pending updates
                    if let Err(e) = Self::process_pending_updates(
                        &state, &parameters, &pending_updates, &config
                    ).await {
                        error!("Error processing pending updates: {}", e);
                    }
                    
                    // Monitor client health
                    Self::monitor_client_health(&clients).await;
                    
                    // Update metrics
                    Self::update_metrics(&metrics, &parameters, &clients).await;
                    
                    // Garbage collection
                    Self::garbage_collection(&parameters, &config).await;
                    
                    // Auto-backup if enabled
                    if config.persistence_enabled {
                        if let Err(e) = Self::auto_backup(&parameters, &config).await {
                            warn!("Auto-backup failed: {}", e);
                        }
                    }
                }
            }
        }

        info!("Parameter Server Agent {} autonomy loop completed", id);
    }

    /// Process pending parameter updates
    async fn process_pending_updates(
        state: &Arc<RwLock<ParameterServerState>>,
        parameters: &Arc<RwLock<HashMap<String, Parameter>>>,
        pending_updates: &Arc<RwLock<HashMap<String, Vec<ParameterUpdate>>>>,
        config: &ParameterServerConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut updates = pending_updates.write().await;
        
        if updates.is_empty() {
            return Ok(());
        }
        
        *state.write().await = ParameterServerState::Aggregating;
        
        for (param_id, param_updates) in updates.iter() {
            if param_updates.is_empty() {
                continue;
            }
            
            // Aggregate updates
            let aggregated_value = Self::aggregate_updates(param_updates, &config.aggregation_strategy)?;
            
            // Update parameter
            let mut params = parameters.write().await;
            if let Some(param) = params.get_mut(param_id) {
                param.value = aggregated_value;
                param.version += 1;
                param.updated_at = Instant::now();
                
                info!("Updated parameter {} to version {}", param_id, param.version);
            }
        }
        
        // Clear processed updates
        updates.clear();
        
        *state.write().await = ParameterServerState::Ready;
        Ok(())
    }

    /// Aggregate parameter updates
    fn aggregate_updates(
        updates: &[ParameterUpdate],
        strategy: &AggregationStrategy,
    ) -> Result<ParameterValue, Box<dyn std::error::Error>> {
        if updates.is_empty() {
            return Err("No updates to aggregate".into());
        }
        
        match strategy {
            AggregationStrategy::Average => {
                // Simple average of values
                match &updates[0].new_value {
                    ParameterValue::Float(_) => {
                        let sum: f64 = updates.iter()
                            .filter_map(|u| match &u.new_value {
                                ParameterValue::Float(v) => Some(*v),
                                _ => None,
                            })
                            .sum();
                        let count = updates.len() as f64;
                        Ok(ParameterValue::Float(sum / count))
                    }
                    ParameterValue::Array(ref arr) => {
                        let len = arr.len();
                        let mut aggregated = vec![0.0; len];
                        
                        for update in updates {
                            if let ParameterValue::Array(ref values) = &update.new_value {
                                for (i, value) in values.iter().enumerate() {
                                    if i < len {
                                        aggregated[i] += value;
                                    }
                                }
                            }
                        }
                        
                        let count = updates.len() as f64;
                        for value in &mut aggregated {
                            *value /= count;
                        }
                        
                        Ok(ParameterValue::Array(aggregated))
                    }
                    _ => {
                        // For other types, just return the latest value
                        Ok(updates.last().unwrap().new_value.clone())
                    }
                }
            }
            
            AggregationStrategy::WeightedAverage => {
                // Implement weighted average based on client weights
                // For now, just use regular average
                Self::aggregate_updates(updates, &AggregationStrategy::Average)
            }
            
            AggregationStrategy::Median => {
                match &updates[0].new_value {
                    ParameterValue::Float(_) => {
                        let mut values: Vec<f64> = updates.iter()
                            .filter_map(|u| match &u.new_value {
                                ParameterValue::Float(v) => Some(*v),
                                _ => None,
                            })
                            .collect();
                        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        
                        let median = if values.len() % 2 == 0 {
                            (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
                        } else {
                            values[values.len() / 2]
                        };
                        
                        Ok(ParameterValue::Float(median))
                    }
                    _ => {
                        // For non-numeric types, return the middle value
                        let mid = updates.len() / 2;
                        Ok(updates[mid].new_value.clone())
                    }
                }
            }
            
            _ => {
                // Default to latest value for other strategies
                Ok(updates.last().unwrap().new_value.clone())
            }
        }
    }

    /// Monitor client health
    async fn monitor_client_health(clients: &Arc<RwLock<HashMap<String, ClientInfo>>>) {
        let now = Instant::now();
        let mut clients_map = clients.write().await;
        
        for client in clients_map.values_mut() {
            let time_since_sync = now.duration_since(client.last_sync);
            
            if time_since_sync > Duration::from_secs(60) && client.active {
                warn!("Client {} has been inactive for {:?}", client.id, time_since_sync);
                client.active = false;
            }
        }
    }

    /// Update metrics
    async fn update_metrics(
        metrics: &Arc<RwLock<ParameterServerMetrics>>,
        parameters: &Arc<RwLock<HashMap<String, Parameter>>>,
        clients: &Arc<RwLock<HashMap<String, ClientInfo>>>,
    ) {
        let mut metrics_guard = metrics.write().await;
        let params = parameters.read().await;
        let clients_map = clients.read().await;
        
        metrics_guard.total_parameters = params.len();
        metrics_guard.active_clients = clients_map.values().filter(|c| c.active).count();
        
        // Calculate storage usage (rough estimate)
        let storage_mb = params.len() as f64 * 0.1; // Rough estimate
        metrics_guard.storage_usage = storage_mb;
    }

    /// Garbage collection
    async fn garbage_collection(
        parameters: &Arc<RwLock<HashMap<String, Parameter>>>,
        config: &ParameterServerConfig,
    ) {
        let params = parameters.read().await;
        
        if params.len() > config.max_parameters {
            warn!("Parameter count ({}) exceeds limit ({})", 
                  params.len(), config.max_parameters);
            // In real implementation, would remove old/unused parameters
        }
    }

    /// Auto-backup parameters
    async fn auto_backup(
        parameters: &Arc<RwLock<HashMap<String, Parameter>>>,
        config: &ParameterServerConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let params = parameters.read().await;
        
        // Mock backup - in real implementation would serialize to persistent storage
        debug!("Auto-backup: {} parameters", params.len());
        
        Ok(())
    }

    /// Start synchronization loop
    async fn start_sync_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.clone();
        let parameters = self.parameters.clone();
        let clients = self.clients.clone();
        let broadcast_tx = self.broadcast_channel.clone();
        let config = self.config.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let id = self.id.clone();

        let handle = tokio::spawn(async move {
            Self::run_sync_loop(
                id, state, parameters, clients, broadcast_tx, config, shutdown_signal
            ).await;
        });

        self.sync_handle = Some(handle);
        Ok(())
    }

    /// Synchronization loop
    async fn run_sync_loop(
        id: String,
        state: Arc<RwLock<ParameterServerState>>,
        parameters: Arc<RwLock<HashMap<String, Parameter>>>,
        clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
        broadcast_tx: broadcast::Sender<Parameter>,
        config: ParameterServerConfig,
        shutdown_signal: Arc<tokio::sync::Notify>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(config.sync_interval_ms));
        
        info!("Parameter Server Agent {} sync loop started", id);

        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => {
                    info!("Parameter Server Agent {} sync loop received shutdown signal", id);
                    break;
                }
                
                _ = interval.tick() => {
                    *state.write().await = ParameterServerState::Synchronizing;
                    
                    // Broadcast updated parameters to clients
                    let params = parameters.read().await;
                    let clients_map = clients.read().await;
                    let active_clients = clients_map.values().filter(|c| c.active).count();
                    
                    if active_clients > 0 {
                        for param in params.values() {
                            if let Err(e) = broadcast_tx.send(param.clone()) {
                                debug!("Failed to broadcast parameter {}: {}", param.id, e);
                            }
                        }
                        
                        debug!("Synchronized {} parameters to {} clients", 
                               params.len(), active_clients);
                    }
                    
                    *state.write().await = ParameterServerState::Ready;
                }
            }
        }

        info!("Parameter Server Agent {} sync loop completed", id);
    }

    /// Start message handler
    async fn start_message_handler(&self, mut rx: mpsc::Receiver<ParameterServerMessage>) {
        let state = self.state.clone();
        let parameters = self.parameters.clone();
        let parameter_history = self.parameter_history.clone();
        let clients = self.clients.clone();
        let pending_updates = self.pending_updates.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    ParameterServerMessage::RegisterClient { client_id, capabilities } => {
                        info!("Registering client: {}", client_id);
                        let client_info = ClientInfo {
                            id: client_id.clone(),
                            capabilities,
                            last_sync: Instant::now(),
                            active: true,
                        };
                        clients.write().await.insert(client_id, client_info);
                    }
                    
                    ParameterServerMessage::UnregisterClient { client_id } => {
                        info!("Unregistering client: {}", client_id);
                        clients.write().await.remove(&client_id);
                    }
                    
                    ParameterServerMessage::SetParameter { parameter } => {
                        info!("Setting parameter: {}", parameter.id);
                        
                        // Store in history if enabled
                        if config.version_history_limit > 0 {
                            let mut history = parameter_history.write().await;
                            let param_history = history.entry(parameter.id.clone()).or_insert_with(Vec::new);
                            param_history.push(parameter.clone());
                            
                            // Limit history size
                            if param_history.len() > config.version_history_limit {
                                param_history.remove(0);
                            }
                        }
                        
                        parameters.write().await.insert(parameter.id.clone(), parameter);
                    }
                    
                    ParameterServerMessage::GetParameter { parameter_id, client_id } => {
                        debug!("Client {} requesting parameter {}", client_id, parameter_id);
                        
                        // Update client last sync
                        if let Some(client) = clients.write().await.get_mut(&client_id) {
                            client.last_sync = Instant::now();
                        }
                        
                        // In real implementation, would send parameter back to client
                    }
                    
                    ParameterServerMessage::UpdateParameter { update } => {
                        debug!("Parameter update for {}", update.parameter_id);
                        
                        let mut updates = pending_updates.write().await;
                        updates.entry(update.parameter_id.clone())
                            .or_insert_with(Vec::new)
                            .push(update);
                    }
                    
                    ParameterServerMessage::BatchUpdate { updates } => {
                        info!("Batch update: {} parameters", updates.len());
                        
                        let mut pending = pending_updates.write().await;
                        for update in updates {
                            pending.entry(update.parameter_id.clone())
                                .or_insert_with(Vec::new)
                                .push(update);
                        }
                    }
                    
                    ParameterServerMessage::Synchronize { client_id } => {
                        debug!("Synchronizing client: {}", client_id);
                        
                        if let Some(client) = clients.write().await.get_mut(&client_id) {
                            client.last_sync = Instant::now();
                            client.active = true;
                        }
                    }
                    
                    ParameterServerMessage::Aggregate { parameter_ids } => {
                        info!("Aggregating parameters: {:?}", parameter_ids);
                        *state.write().await = ParameterServerState::Aggregating;
                        // Trigger immediate aggregation
                    }
                    
                    ParameterServerMessage::Checkpoint { path } => {
                        info!("Creating checkpoint at: {}", path);
                        *state.write().await = ParameterServerState::Persisting;
                        // In real implementation, would save to disk
                    }
                    
                    ParameterServerMessage::Restore { path } => {
                        info!("Restoring from checkpoint: {}", path);
                        *state.write().await = ParameterServerState::Recovering;
                        // In real implementation, would load from disk
                    }
                    
                    ParameterServerMessage::GetMetrics => {
                        let metrics_guard = metrics.read().await;
                        debug!("Current metrics: {:?}", *metrics_guard);
                    }
                }
            }
        });
    }

    /// Set state
    async fn set_state(&self, new_state: ParameterServerState) {
        *self.state.write().await = new_state;
    }

    /// Get current state
    pub async fn get_state(&self) -> ParameterServerState {
        self.state.read().await.clone()
    }

    /// Set parameter
    pub async fn set_parameter(&self, parameter: Parameter) -> Result<(), Box<dyn std::error::Error>> {
        self.message_channel.send(ParameterServerMessage::SetParameter { parameter }).await?;
        Ok(())
    }

    /// Get parameter
    pub async fn get_parameter(&self, parameter_id: &str) -> Option<Parameter> {
        self.parameters.read().await.get(parameter_id).cloned()
    }

    /// Update parameter
    pub async fn update_parameter(&self, update: ParameterUpdate) -> Result<(), Box<dyn std::error::Error>> {
        self.message_channel.send(ParameterServerMessage::UpdateParameter { update }).await?;
        Ok(())
    }

    /// Subscribe to parameter updates
    pub fn subscribe(&self) -> broadcast::Receiver<Parameter> {
        self.broadcast_channel.subscribe()
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> ParameterServerMetrics {
        self.metrics.read().await.clone()
    }

    /// Shutdown parameter server
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down Parameter Server Agent {}", self.id);
        
        self.shutdown_signal.notify_waiters();
        
        if let Some(handle) = self.autonomy_handle.take() {
            handle.await?;
        }
        
        if let Some(handle) = self.sync_handle.take() {
            handle.await?;
        }
        
        info!("Parameter Server Agent {} shutdown complete", self.id);
        Ok(())
    }
}