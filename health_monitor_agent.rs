//! Health Monitor DAA Agent Implementation
//! Comprehensive health monitoring agent for system and agent health tracking

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc, broadcast};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{debug, info, warn, error};

/// Health monitor state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthMonitorState {
    Initializing,
    Monitoring,
    Alerting,
    Diagnosing,
    Recovering,
    Failed(String),
}

/// Health status levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
    Degraded,
}

/// Health check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    SystemResource,
    NetworkConnectivity,
    DatabaseConnection,
    ServiceEndpoint,
    AgentHealth,
    Custom(String),
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub id: String,
    pub name: String,
    pub check_type: HealthCheckType,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub enabled: bool,
    pub threshold_warning: f64,
    pub threshold_critical: f64,
    pub retry_count: usize,
    pub metadata: HashMap<String, String>,
}

/// Health metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: Instant,
    pub status: HealthStatus,
    pub tags: HashMap<String, String>,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_id: String,
    pub status: HealthStatus,
    pub value: f64,
    pub message: String,
    pub timestamp: Instant,
    pub duration: Duration,
    pub metadata: HashMap<String, String>,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub source: String,
    pub created_at: Instant,
    pub resolved_at: Option<Instant>,
    pub status: AlertStatus,
    pub tags: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
}

/// Health monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    pub check_interval_seconds: u64,
    pub alert_cooldown_seconds: u64,
    pub metric_retention_hours: u64,
    pub enable_auto_recovery: bool,
    pub max_concurrent_checks: usize,
    pub notification_channels: Vec<String>,
    pub system_resource_checks: bool,
    pub network_checks: bool,
    pub agent_health_checks: bool,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            alert_cooldown_seconds: 300,
            metric_retention_hours: 24,
            enable_auto_recovery: true,
            max_concurrent_checks: 20,
            notification_channels: vec!["console".to_string()],
            system_resource_checks: true,
            network_checks: true,
            agent_health_checks: true,
        }
    }
}

/// Health monitor messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthMonitorMessage {
    RegisterCheck { check: HealthCheck },
    UnregisterCheck { check_id: String },
    ReportMetric { metric: HealthMetric },
    TriggerCheck { check_id: String },
    GetHealthStatus { component_id: Option<String> },
    GetAlerts { active_only: bool },
    AcknowledgeAlert { alert_id: String },
    ResolveAlert { alert_id: String },
    GetMetrics { component_id: Option<String>, hours: Option<u64> },
    SetThreshold { check_id: String, warning: f64, critical: f64 },
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_io_mbps: f64,
    pub open_file_descriptors: u64,
    pub thread_count: u64,
}

/// Health Monitor DAA Agent
pub struct HealthMonitorAgent {
    id: String,
    config: HealthMonitorConfig,
    state: Arc<RwLock<HealthMonitorState>>,
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    check_results: Arc<RwLock<HashMap<String, Vec<HealthCheckResult>>>>,
    metrics: Arc<RwLock<HashMap<String, Vec<HealthMetric>>>>,
    alerts: Arc<RwLock<HashMap<String, HealthAlert>>>,
    system_metrics: Arc<RwLock<SystemResourceMetrics>>,
    message_channel: mpsc::Sender<HealthMonitorMessage>,
    alert_channel: broadcast::Sender<HealthAlert>,
    autonomy_handle: Option<tokio::task::JoinHandle<()>>,
    check_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl HealthMonitorAgent {
    /// Create a new health monitor agent
    pub async fn new(config: HealthMonitorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel(1000);
        let (alert_tx, _) = broadcast::channel(100);
        
        let agent = Self {
            id: Uuid::new_v4().to_string(),
            config,
            state: Arc::new(RwLock::new(HealthMonitorState::Initializing)),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            check_results: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemResourceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_percent: 0.0,
                disk_usage_percent: 0.0,
                network_io_mbps: 0.0,
                open_file_descriptors: 0,
                thread_count: 0,
            })),
            message_channel: tx,
            alert_channel: alert_tx,
            autonomy_handle: None,
            check_handle: None,
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        };

        // Start message handler
        agent.start_message_handler(rx).await;
        
        Ok(agent)
    }

    /// Initialize the health monitor
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Health Monitor Agent {}", self.id);
        self.set_state(HealthMonitorState::Initializing).await;
        
        // Register default health checks
        self.register_default_checks().await?;
        
        // Start autonomy loop
        self.start_autonomy_loop().await?;
        
        // Start health check loop
        self.start_health_check_loop().await?;
        
        self.set_state(HealthMonitorState::Monitoring).await;
        info!("Health Monitor Agent {} initialized", self.id);
        Ok(())
    }

    /// Register default health checks
    async fn register_default_checks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_checks = vec![
            HealthCheck {
                id: "cpu_usage".to_string(),
                name: "CPU Usage".to_string(),
                check_type: HealthCheckType::SystemResource,
                interval_seconds: 30,
                timeout_seconds: 5,
                enabled: self.config.system_resource_checks,
                threshold_warning: 80.0,
                threshold_critical: 95.0,
                retry_count: 3,
                metadata: HashMap::new(),
            },
            HealthCheck {
                id: "memory_usage".to_string(),
                name: "Memory Usage".to_string(),
                check_type: HealthCheckType::SystemResource,
                interval_seconds: 30,
                timeout_seconds: 5,
                enabled: self.config.system_resource_checks,
                threshold_warning: 85.0,
                threshold_critical: 95.0,
                retry_count: 3,
                metadata: HashMap::new(),
            },
            HealthCheck {
                id: "disk_usage".to_string(),
                name: "Disk Usage".to_string(),
                check_type: HealthCheckType::SystemResource,
                interval_seconds: 60,
                timeout_seconds: 10,
                enabled: self.config.system_resource_checks,
                threshold_warning: 80.0,
                threshold_critical: 90.0,
                retry_count: 2,
                metadata: HashMap::new(),
            },
            HealthCheck {
                id: "network_connectivity".to_string(),
                name: "Network Connectivity".to_string(),
                check_type: HealthCheckType::NetworkConnectivity,
                interval_seconds: 60,
                timeout_seconds: 10,
                enabled: self.config.network_checks,
                threshold_warning: 500.0,  // 500ms latency
                threshold_critical: 2000.0, // 2s latency
                retry_count: 3,
                metadata: HashMap::new(),
            },
        ];
        
        let mut checks = self.health_checks.write().await;
        for check in default_checks {
            checks.insert(check.id.clone(), check);
        }
        
        Ok(())
    }

    /// Start autonomy loop
    async fn start_autonomy_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.clone();
        let alerts = self.alerts.clone();
        let metrics = self.metrics.clone();
        let system_metrics = self.system_metrics.clone();
        let config = self.config.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let id = self.id.clone();

        let handle = tokio::spawn(async move {
            Self::run_autonomy_loop(
                id, state, alerts, metrics, system_metrics, config, shutdown_signal
            ).await;
        });

        self.autonomy_handle = Some(handle);
        Ok(())
    }

    /// Main autonomy loop
    async fn run_autonomy_loop(
        id: String,
        state: Arc<RwLock<HealthMonitorState>>,
        alerts: Arc<RwLock<HashMap<String, HealthAlert>>>,
        metrics: Arc<RwLock<HashMap<String, Vec<HealthMetric>>>>,
        system_metrics: Arc<RwLock<SystemResourceMetrics>>,
        config: HealthMonitorConfig,
        shutdown_signal: Arc<tokio::sync::Notify>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        info!("Health Monitor Agent {} autonomy loop started", id);

        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => {
                    info!("Health Monitor Agent {} received shutdown signal", id);
                    break;
                }
                
                _ = interval.tick() => {
                    // Collect system metrics
                    if let Err(e) = Self::collect_system_metrics(&system_metrics).await {
                        error!("Failed to collect system metrics: {}", e);
                    }
                    
                    // Process alerts
                    Self::process_alerts(&alerts, &config).await;
                    
                    // Cleanup old metrics
                    Self::cleanup_old_metrics(&metrics, &config).await;
                    
                    // Auto-recovery if enabled
                    if config.enable_auto_recovery {
                        Self::attempt_auto_recovery(&state, &alerts).await;
                    }
                }
            }
        }

        info!("Health Monitor Agent {} autonomy loop completed", id);
    }

    /// Collect system metrics
    async fn collect_system_metrics(
        system_metrics: &Arc<RwLock<SystemResourceMetrics>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Mock system metrics collection
        // In real implementation, would use system APIs
        let mut metrics = system_metrics.write().await;
        
        // Simulate varying system metrics
        let now = Instant::now();
        let variation = (now.elapsed().as_secs() % 60) as f64 / 60.0;
        
        metrics.cpu_usage_percent = 20.0 + 30.0 * variation;
        metrics.memory_usage_percent = 40.0 + 20.0 * variation;
        metrics.disk_usage_percent = 60.0 + 10.0 * variation;
        metrics.network_io_mbps = 10.0 + 5.0 * variation;
        metrics.open_file_descriptors = 100 + (50.0 * variation) as u64;
        metrics.thread_count = 20 + (10.0 * variation) as u64;
        
        Ok(())
    }

    /// Process alerts
    async fn process_alerts(
        alerts: &Arc<RwLock<HashMap<String, HealthAlert>>>,
        config: &HealthMonitorConfig,
    ) {
        let now = Instant::now();
        let mut alerts_map = alerts.write().await;
        
        // Auto-resolve old alerts
        for alert in alerts_map.values_mut() {
            if alert.status == AlertStatus::Active {
                let age = now.duration_since(alert.created_at);
                if age > Duration::from_secs(config.alert_cooldown_seconds * 2) {
                    alert.status = AlertStatus::Resolved;
                    alert.resolved_at = Some(now);
                    info!("Auto-resolved alert: {}", alert.title);
                }
            }
        }
    }

    /// Cleanup old metrics
    async fn cleanup_old_metrics(
        metrics: &Arc<RwLock<HashMap<String, Vec<HealthMetric>>>>,
        config: &HealthMonitorConfig,
    ) {
        let cutoff = Instant::now() - Duration::from_secs(config.metric_retention_hours * 3600);
        let mut metrics_map = metrics.write().await;
        
        for metric_list in metrics_map.values_mut() {
            metric_list.retain(|m| m.timestamp > cutoff);
        }
    }

    /// Attempt auto-recovery
    async fn attempt_auto_recovery(
        state: &Arc<RwLock<HealthMonitorState>>,
        alerts: &Arc<RwLock<HashMap<String, HealthAlert>>>,
    ) {
        let alerts_map = alerts.read().await;
        let critical_alerts: Vec<_> = alerts_map.values()
            .filter(|a| a.severity == AlertSeverity::Critical && a.status == AlertStatus::Active)
            .collect();
        
        if !critical_alerts.is_empty() {
            warn!("Auto-recovery: {} critical alerts detected", critical_alerts.len());
            *state.write().await = HealthMonitorState::Recovering;
            
            // In real implementation, would attempt recovery actions
            // For now, just log the attempt
            info!("Attempting auto-recovery for critical alerts");
            
            // Simulate recovery time
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            *state.write().await = HealthMonitorState::Monitoring;
        }
    }

    /// Start health check loop
    async fn start_health_check_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let health_checks = self.health_checks.clone();
        let check_results = self.check_results.clone();
        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();
        let system_metrics = self.system_metrics.clone();
        let alert_channel = self.alert_channel.clone();
        let config = self.config.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let id = self.id.clone();

        let handle = tokio::spawn(async move {
            Self::run_health_check_loop(
                id, health_checks, check_results, metrics, alerts, system_metrics, alert_channel, config, shutdown_signal
            ).await;
        });

        self.check_handle = Some(handle);
        Ok(())
    }

    /// Health check loop
    async fn run_health_check_loop(
        id: String,
        health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
        check_results: Arc<RwLock<HashMap<String, Vec<HealthCheckResult>>>>,
        metrics: Arc<RwLock<HashMap<String, Vec<HealthMetric>>>>,
        alerts: Arc<RwLock<HashMap<String, HealthAlert>>>,
        system_metrics: Arc<RwLock<SystemResourceMetrics>>,
        alert_channel: broadcast::Sender<HealthAlert>,
        config: HealthMonitorConfig,
        shutdown_signal: Arc<tokio::sync::Notify>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(config.check_interval_seconds));
        
        info!("Health Monitor Agent {} check loop started", id);

        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => {
                    info!("Health Monitor Agent {} check loop received shutdown signal", id);
                    break;
                }
                
                _ = interval.tick() => {
                    let checks = health_checks.read().await;
                    let enabled_checks: Vec<_> = checks.values().filter(|c| c.enabled).collect();
                    
                    for check in enabled_checks {
                        let result = Self::perform_health_check(check, &system_metrics).await;
                        
                        // Store result
                        let mut results = check_results.write().await;
                        results.entry(check.id.clone()).or_insert_with(Vec::new).push(result.clone());
                        
                        // Convert to metric
                        let metric = HealthMetric {
                            id: format!("{}_{}", check.id, result.timestamp.elapsed().as_millis()),
                            name: check.name.clone(),
                            value: result.value,
                            unit: "".to_string(),
                            timestamp: result.timestamp,
                            status: result.status.clone(),
                            tags: HashMap::new(),
                        };
                        
                        // Store metric
                        let mut metrics_map = metrics.write().await;
                        metrics_map.entry(check.id.clone()).or_insert_with(Vec::new).push(metric);
                        
                        // Check for alerts
                        if let Some(alert) = Self::check_for_alert(check, &result).await {
                            info!("Health alert triggered: {}", alert.title);
                            
                            // Store alert
                            alerts.write().await.insert(alert.id.clone(), alert.clone());
                            
                            // Broadcast alert
                            if let Err(e) = alert_channel.send(alert) {
                                debug!("Failed to broadcast alert: {}", e);
                            }
                        }
                    }
                }
            }
        }

        info!("Health Monitor Agent {} check loop completed", id);
    }

    /// Perform a health check
    async fn perform_health_check(
        check: &HealthCheck,
        system_metrics: &Arc<RwLock<SystemResourceMetrics>>,
    ) -> HealthCheckResult {
        let start_time = Instant::now();
        
        let (value, status, message) = match &check.check_type {
            HealthCheckType::SystemResource => {
                let metrics = system_metrics.read().await;
                match check.id.as_str() {
                    "cpu_usage" => {
                        let value = metrics.cpu_usage_percent;
                        let status = if value >= check.threshold_critical {
                            HealthStatus::Critical
                        } else if value >= check.threshold_warning {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Healthy
                        };
                        (value, status, format!("CPU usage: {:.1}%", value))
                    }
                    "memory_usage" => {
                        let value = metrics.memory_usage_percent;
                        let status = if value >= check.threshold_critical {
                            HealthStatus::Critical
                        } else if value >= check.threshold_warning {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Healthy
                        };
                        (value, status, format!("Memory usage: {:.1}%", value))
                    }
                    "disk_usage" => {
                        let value = metrics.disk_usage_percent;
                        let status = if value >= check.threshold_critical {
                            HealthStatus::Critical
                        } else if value >= check.threshold_warning {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Healthy
                        };
                        (value, status, format!("Disk usage: {:.1}%", value))
                    }
                    _ => (0.0, HealthStatus::Unknown, "Unknown check".to_string()),
                }
            }
            HealthCheckType::NetworkConnectivity => {
                // Mock network check
                let latency = 50.0 + (Instant::now().elapsed().as_millis() % 100) as f64;
                let status = if latency >= check.threshold_critical {
                    HealthStatus::Critical
                } else if latency >= check.threshold_warning {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Healthy
                };
                (latency, status, format!("Network latency: {:.1}ms", latency))
            }
            _ => (0.0, HealthStatus::Unknown, "Check not implemented".to_string()),
        };
        
        HealthCheckResult {
            check_id: check.id.clone(),
            status,
            value,
            message,
            timestamp: start_time,
            duration: start_time.elapsed(),
            metadata: HashMap::new(),
        }
    }

    /// Check if a health check result should trigger an alert
    async fn check_for_alert(
        check: &HealthCheck,
        result: &HealthCheckResult,
    ) -> Option<HealthAlert> {
        if result.status == HealthStatus::Critical || result.status == HealthStatus::Warning {
            let severity = if result.status == HealthStatus::Critical {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            Some(HealthAlert {
                id: Uuid::new_v4().to_string(),
                severity,
                title: format!("{} Alert", check.name),
                description: result.message.clone(),
                source: check.id.clone(),
                created_at: Instant::now(),
                resolved_at: None,
                status: AlertStatus::Active,
                tags: HashMap::new(),
            })
        } else {
            None
        }
    }

    /// Start message handler
    async fn start_message_handler(&self, mut rx: mpsc::Receiver<HealthMonitorMessage>) {
        let health_checks = self.health_checks.clone();
        let check_results = self.check_results.clone();
        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    HealthMonitorMessage::RegisterCheck { check } => {
                        info!("Registering health check: {}", check.name);
                        health_checks.write().await.insert(check.id.clone(), check);
                    }
                    
                    HealthMonitorMessage::UnregisterCheck { check_id } => {
                        info!("Unregistering health check: {}", check_id);
                        health_checks.write().await.remove(&check_id);
                    }
                    
                    HealthMonitorMessage::ReportMetric { metric } => {
                        debug!("Received metric: {} = {}", metric.name, metric.value);
                        let metric_key = format!("custom_{}", metric.name);
                        metrics.write().await.entry(metric_key).or_insert_with(Vec::new).push(metric);
                    }
                    
                    HealthMonitorMessage::TriggerCheck { check_id } => {
                        debug!("Triggering health check: {}", check_id);
                        // In real implementation, would trigger immediate check
                    }
                    
                    HealthMonitorMessage::GetHealthStatus { component_id } => {
                        if let Some(comp_id) = component_id {
                            debug!("Getting health status for component: {}", comp_id);
                        } else {
                            debug!("Getting overall health status");
                        }
                    }
                    
                    HealthMonitorMessage::GetAlerts { active_only } => {
                        let alerts_map = alerts.read().await;
                        let count = if active_only {
                            alerts_map.values().filter(|a| a.status == AlertStatus::Active).count()
                        } else {
                            alerts_map.len()
                        };
                        debug!("Found {} alerts", count);
                    }
                    
                    HealthMonitorMessage::AcknowledgeAlert { alert_id } => {
                        info!("Acknowledging alert: {}", alert_id);
                        if let Some(alert) = alerts.write().await.get_mut(&alert_id) {
                            alert.status = AlertStatus::Acknowledged;
                        }
                    }
                    
                    HealthMonitorMessage::ResolveAlert { alert_id } => {
                        info!("Resolving alert: {}", alert_id);
                        if let Some(alert) = alerts.write().await.get_mut(&alert_id) {
                            alert.status = AlertStatus::Resolved;
                            alert.resolved_at = Some(Instant::now());
                        }
                    }
                    
                    HealthMonitorMessage::GetMetrics { component_id, hours } => {
                        let metrics_map = metrics.read().await;
                        if let Some(comp_id) = component_id {
                            if let Some(comp_metrics) = metrics_map.get(&comp_id) {
                                debug!("Found {} metrics for component {}", comp_metrics.len(), comp_id);
                            }
                        } else {
                            let total_metrics: usize = metrics_map.values().map(|v| v.len()).sum();
                            debug!("Total metrics across all components: {}", total_metrics);
                        }
                    }
                    
                    HealthMonitorMessage::SetThreshold { check_id, warning, critical } => {
                        info!("Setting thresholds for {}: warning={}, critical={}", 
                              check_id, warning, critical);
                        if let Some(check) = health_checks.write().await.get_mut(&check_id) {
                            check.threshold_warning = warning;
                            check.threshold_critical = critical;
                        }
                    }
                }
            }
        });
    }

    /// Set state
    async fn set_state(&self, new_state: HealthMonitorState) {
        *self.state.write().await = new_state;
    }

    /// Get current state
    pub async fn get_state(&self) -> HealthMonitorState {
        self.state.read().await.clone()
    }

    /// Get overall health status
    pub async fn get_health_status(&self) -> HealthStatus {
        let alerts = self.alerts.read().await;
        let active_alerts: Vec<_> = alerts.values()
            .filter(|a| a.status == AlertStatus::Active)
            .collect();
        
        if active_alerts.iter().any(|a| a.severity == AlertSeverity::Critical) {
            HealthStatus::Critical
        } else if active_alerts.iter().any(|a| a.severity == AlertSeverity::Warning) {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<HealthAlert> {
        self.alerts.read().await.values()
            .filter(|a| a.status == AlertStatus::Active)
            .cloned()
            .collect()
    }

    /// Subscribe to alerts
    pub fn subscribe_alerts(&self) -> broadcast::Receiver<HealthAlert> {
        self.alert_channel.subscribe()
    }

    /// Report custom metric
    pub async fn report_metric(&self, metric: HealthMetric) -> Result<(), Box<dyn std::error::Error>> {
        self.message_channel.send(HealthMonitorMessage::ReportMetric { metric }).await?;
        Ok(())
    }

    /// Shutdown health monitor
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down Health Monitor Agent {}", self.id);
        
        self.shutdown_signal.notify_waiters();
        
        if let Some(handle) = self.autonomy_handle.take() {
            handle.await?;
        }
        
        if let Some(handle) = self.check_handle.take() {
            handle.await?;
        }
        
        info!("Health Monitor Agent {} shutdown complete", self.id);
        Ok(())
    }
}