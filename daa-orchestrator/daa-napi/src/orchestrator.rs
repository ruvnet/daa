//! Orchestrator bindings for NAPI
//!
//! Provides Node.js access to the DAA orchestrator's MRAP (Monitor, Reason, Act, Plan) loop
//! and autonomy management capabilities.

use daa_orchestrator::autonomy::{AutonomyLoop, AutonomyState};
use daa_orchestrator::config::{AiConfig, AutonomyConfig, RulesConfig};
use napi::bindgen_prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Orchestrator configuration for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Whether the autonomy loop is enabled
    pub enabled: bool,
    /// Loop iteration interval in milliseconds
    pub loop_interval_ms: Option<u32>,
    /// Maximum number of tasks to process per iteration
    pub max_tasks_per_iteration: Option<u32>,
    /// Task timeout in milliseconds
    pub task_timeout_ms: Option<u32>,
    /// Whether to enable learning from decisions
    pub enable_learning: Option<bool>,
    /// Rules engine configuration
    pub rules: Option<RulesConfigJs>,
    /// AI agents configuration
    pub ai: Option<AiConfigJs>,
}

/// Rules configuration for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RulesConfigJs {
    pub enabled: bool,
    pub fail_fast: Option<bool>,
    pub max_daily_spending: Option<f64>,
    pub min_balance_threshold: Option<f64>,
    pub max_risk_score: Option<f64>,
}

/// AI configuration for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct AiConfigJs {
    pub enabled: bool,
    pub max_agents: Option<u32>,
    pub agent_queue_size: Option<u32>,
    pub learning_retention_days: Option<i32>,
}

/// System state information
#[napi(object)]
pub struct SystemState {
    /// Current autonomy state
    pub state: String,
    /// System uptime in seconds
    pub uptime_seconds: f64,
    /// Whether the system is healthy
    pub is_healthy: bool,
    /// Status description
    pub status: String,
}

/// System statistics
#[napi(object)]
pub struct SystemStatistics {
    /// Total iterations completed
    pub total_iterations: f64,
    /// Average iteration time in milliseconds
    pub avg_iteration_ms: f64,
    /// Active tasks count
    pub active_tasks: f64,
    /// Completed tasks count
    pub completed_tasks: f64,
    /// Failed tasks count
    pub failed_tasks: f64,
}

impl OrchestratorConfig {
    /// Convert JavaScript config to Rust config
    fn to_rust_config(&self) -> AutonomyConfig {
        let mut config = AutonomyConfig {
            enabled: self.enabled,
            loop_interval_ms: self.loop_interval_ms.unwrap_or(1000) as u64,
            max_tasks_per_iteration: self.max_tasks_per_iteration.unwrap_or(10) as usize,
            task_timeout_ms: self.task_timeout_ms.unwrap_or(30000) as u64,
            enable_learning: self.enable_learning.unwrap_or(true),
            rules_config: RulesConfig::default(),
            ai_config: AiConfig::default(),
        };

        if let Some(ref rules) = self.rules {
            config.rules_config = RulesConfig {
                enabled: rules.enabled,
                fail_fast: rules.fail_fast.unwrap_or(false),
                max_daily_spending: rules.max_daily_spending.unwrap_or(10000.0),
                min_balance_threshold: rules.min_balance_threshold.unwrap_or(100.0),
                max_risk_score: rules.max_risk_score.unwrap_or(0.8),
            };
        }

        if let Some(ref ai) = self.ai {
            config.ai_config = AiConfig {
                enabled: ai.enabled,
                max_agents: ai.max_agents.unwrap_or(5) as usize,
                agent_queue_size: ai.agent_queue_size.unwrap_or(100) as usize,
                learning_retention_days: ai.learning_retention_days.unwrap_or(30) as i64,
            };
        }

        config
    }
}

/// Main orchestrator class exposing the MRAP loop
#[napi]
pub struct Orchestrator {
    autonomy_loop: Arc<Mutex<AutonomyLoop>>,
    config: AutonomyConfig,
}

#[napi]
impl Orchestrator {
    /// Create a new orchestrator instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Orchestrator configuration
    ///
    /// # Returns
    ///
    /// A new Orchestrator instance
    ///
    /// # Example
    ///
    /// ```javascript
    /// const orchestrator = new Orchestrator({
    ///   enabled: true,
    ///   loopIntervalMs: 1000,
    ///   maxTasksPerIteration: 10,
    ///   taskTimeoutMs: 30000,
    ///   enableLearning: true,
    ///   rules: {
    ///     enabled: true,
    ///     maxDailySpending: 10000,
    ///     minBalanceThreshold: 100
    ///   },
    ///   ai: {
    ///     enabled: true,
    ///     maxAgents: 5
    ///   }
    /// });
    /// ```
    #[napi(constructor)]
    pub fn new(config: OrchestratorConfig) -> Result<Self> {
        let rust_config = config.to_rust_config();

        // Create autonomy loop in a blocking context
        let autonomy_loop = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                AutonomyLoop::new(rust_config.clone()).await.map_err(|e| {
                    Error::from_reason(format!("Failed to create autonomy loop: {}", e))
                })
            })
        })?;

        Ok(Self {
            autonomy_loop: Arc::new(Mutex::new(autonomy_loop)),
            config: rust_config,
        })
    }

    /// Start the MRAP (Monitor, Reason, Act, Plan) autonomy loop.
    ///
    /// # Returns
    ///
    /// Promise that resolves when the loop is started
    ///
    /// # Example
    ///
    /// ```javascript
    /// await orchestrator.start();
    /// console.log('MRAP loop started');
    /// ```
    #[napi]
    pub async fn start(&self) -> Result<()> {
        let mut loop_guard = self.autonomy_loop.lock().await;

        loop_guard
            .initialize()
            .await
            .map_err(|e| Error::from_reason(format!("Failed to initialize: {}", e)))?;

        loop_guard
            .start()
            .await
            .map_err(|e| Error::from_reason(format!("Failed to start: {}", e)))?;

        Ok(())
    }

    /// Stop the MRAP autonomy loop.
    ///
    /// # Returns
    ///
    /// Promise that resolves when the loop is stopped
    ///
    /// # Example
    ///
    /// ```javascript
    /// await orchestrator.stop();
    /// console.log('MRAP loop stopped');
    /// ```
    #[napi]
    pub async fn stop(&self) -> Result<()> {
        let mut loop_guard = self.autonomy_loop.lock().await;

        loop_guard
            .stop()
            .await
            .map_err(|e| Error::from_reason(format!("Failed to stop: {}", e)))?;

        Ok(())
    }

    /// Restart the MRAP autonomy loop.
    ///
    /// # Returns
    ///
    /// Promise that resolves when the loop is restarted
    ///
    /// # Example
    ///
    /// ```javascript
    /// await orchestrator.restart();
    /// console.log('MRAP loop restarted');
    /// ```
    #[napi]
    pub async fn restart(&self) -> Result<()> {
        let mut loop_guard = self.autonomy_loop.lock().await;

        loop_guard
            .restart()
            .await
            .map_err(|e| Error::from_reason(format!("Failed to restart: {}", e)))?;

        Ok(())
    }

    /// Get the current system state (Monitor phase of MRAP).
    ///
    /// # Returns
    ///
    /// Current system state including health status and uptime
    ///
    /// # Example
    ///
    /// ```javascript
    /// const state = await orchestrator.monitor();
    /// console.log('State:', state.state);
    /// console.log('Uptime:', state.uptimeSeconds);
    /// console.log('Healthy:', state.isHealthy);
    /// ```
    #[napi]
    pub async fn monitor(&self) -> Result<SystemState> {
        let loop_guard = self.autonomy_loop.lock().await;

        let state = loop_guard.get_state().await;
        let status = loop_guard.get_status().await;
        let uptime = loop_guard.get_uptime().await;
        let is_healthy = loop_guard
            .health_check()
            .await
            .map_err(|e| Error::from_reason(format!("Health check failed: {}", e)))?;

        let state_str = match state {
            AutonomyState::Initializing => "initializing",
            AutonomyState::Idle => "idle",
            AutonomyState::Processing => "processing",
            AutonomyState::Learning => "learning",
            AutonomyState::Error(_) => "error",
            AutonomyState::Stopped => "stopped",
        };

        Ok(SystemState {
            state: state_str.to_string(),
            uptime_seconds: uptime.as_secs_f64(),
            is_healthy,
            status,
        })
    }

    /// Perform a health check on the orchestrator.
    ///
    /// # Returns
    ///
    /// true if the orchestrator is healthy, false otherwise
    ///
    /// # Example
    ///
    /// ```javascript
    /// const isHealthy = await orchestrator.healthCheck();
    /// if (!isHealthy) {
    ///   console.error('Orchestrator is unhealthy!');
    /// }
    /// ```
    #[napi]
    pub async fn health_check(&self) -> Result<bool> {
        let loop_guard = self.autonomy_loop.lock().await;

        loop_guard
            .health_check()
            .await
            .map_err(|e| Error::from_reason(format!("Health check failed: {}", e)))
    }

    /// Get the current configuration.
    ///
    /// # Returns
    ///
    /// The current orchestrator configuration
    ///
    /// # Example
    ///
    /// ```javascript
    /// const config = orchestrator.getConfig();
    /// console.log('Loop interval:', config.loopIntervalMs);
    /// ```
    #[napi]
    pub fn get_config(&self) -> Result<OrchestratorConfig> {
        Ok(OrchestratorConfig {
            enabled: self.config.enabled,
            loop_interval_ms: Some(self.config.loop_interval_ms as u32),
            max_tasks_per_iteration: Some(self.config.max_tasks_per_iteration as u32),
            task_timeout_ms: Some(self.config.task_timeout_ms as u32),
            enable_learning: Some(self.config.enable_learning),
            rules: Some(RulesConfigJs {
                enabled: self.config.rules_config.enabled,
                fail_fast: Some(self.config.rules_config.fail_fast),
                max_daily_spending: Some(self.config.rules_config.max_daily_spending),
                min_balance_threshold: Some(self.config.rules_config.min_balance_threshold),
                max_risk_score: Some(self.config.rules_config.max_risk_score),
            }),
            ai: Some(AiConfigJs {
                enabled: self.config.ai_config.enabled,
                max_agents: Some(self.config.ai_config.max_agents as u32),
                agent_queue_size: Some(self.config.ai_config.agent_queue_size as u32),
                learning_retention_days: Some(self.config.ai_config.learning_retention_days as i32),
            }),
        })
    }

    /// Get system statistics.
    ///
    /// # Returns
    ///
    /// System statistics including iteration counts and task metrics
    ///
    /// # Example
    ///
    /// ```javascript
    /// const stats = await orchestrator.getStatistics();
    /// console.log('Total iterations:', stats.totalIterations);
    /// console.log('Active tasks:', stats.activeTasks);
    /// ```
    #[napi]
    pub async fn get_statistics(&self) -> Result<SystemStatistics> {
        // Mock statistics for now - in a real implementation, these would
        // come from the actual orchestrator state
        Ok(SystemStatistics {
            total_iterations: 0.0,
            avg_iteration_ms: 0.0,
            active_tasks: 0.0,
            completed_tasks: 0.0,
            failed_tasks: 0.0,
        })
    }
}
