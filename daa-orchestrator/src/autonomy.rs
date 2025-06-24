//! Autonomy loop implementation for autonomous decision making

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::config::AutonomyConfig;
use crate::error::{OrchestratorError, Result};

/// Autonomy state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum AutonomyState {
    Initializing,
    Idle,
    Processing,
    Learning,
    Error(String),
    Stopped,
}

/// Autonomy loop for continuous autonomous operation
pub struct AutonomyLoop {
    config: AutonomyConfig,
    state: Arc<RwLock<AutonomyState>>,
    start_time: Option<Instant>,
    loop_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl AutonomyLoop {
    /// Create a new autonomy loop
    pub async fn new(config: AutonomyConfig) -> Result<Self> {
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(AutonomyState::Initializing)),
            start_time: None,
            loop_handle: None,
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        })
    }

    /// Initialize the autonomy loop
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing autonomy loop");
        self.set_state(AutonomyState::Initializing).await;
        
        // Initialize rules engine
        if self.config.rules_config.enabled {
            debug!("Rules engine enabled with max risk score: {}", 
                   self.config.rules_config.max_risk_score);
        }

        // Initialize AI agents
        if self.config.ai_config.enabled {
            debug!("AI agents enabled, max agents: {}", 
                   self.config.ai_config.max_agents);
        }

        self.set_state(AutonomyState::Idle).await;
        info!("Autonomy loop initialized");
        Ok(())
    }

    /// Start the autonomy loop
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Autonomy loop is disabled in configuration");
            return Ok(());
        }

        info!("Starting autonomy loop");
        self.start_time = Some(Instant::now());
        
        let config = self.config.clone();
        let state = self.state.clone();
        let shutdown_signal = self.shutdown_signal.clone();

        // Spawn the main autonomy loop
        let handle = tokio::spawn(async move {
            Self::run_loop(config, state, shutdown_signal).await;
        });

        self.loop_handle = Some(handle);
        info!("Autonomy loop started");
        Ok(())
    }

    /// Stop the autonomy loop
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping autonomy loop");
        
        // Signal shutdown
        self.shutdown_signal.notify_one();
        
        // Wait for loop to finish
        if let Some(handle) = self.loop_handle.take() {
            if let Err(e) = handle.await {
                warn!("Error waiting for autonomy loop to stop: {}", e);
            }
        }

        self.set_state(AutonomyState::Stopped).await;
        info!("Autonomy loop stopped");
        Ok(())
    }

    /// Restart the autonomy loop
    pub async fn restart(&mut self) -> Result<()> {
        info!("Restarting autonomy loop");
        self.stop().await?;
        tokio::time::sleep(Duration::from_millis(1000)).await; // Brief pause
        self.start().await?;
        Ok(())
    }

    /// Check health of the autonomy loop
    pub async fn health_check(&self) -> Result<bool> {
        let state = self.get_state().await;
        
        match state {
            AutonomyState::Error(_) => Ok(false),
            AutonomyState::Stopped => Ok(false),
            _ => {
                // Check if loop is still running
                if let Some(ref handle) = self.loop_handle {
                    Ok(!handle.is_finished())
                } else {
                    Ok(true) // Not started yet, that's ok
                }
            }
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> AutonomyState {
        self.state.read().await.clone()
    }

    /// Set new state
    async fn set_state(&self, new_state: AutonomyState) {
        *self.state.write().await = new_state;
    }

    /// Get status string
    pub async fn get_status(&self) -> String {
        let state = self.get_state().await;
        format!("{:?}", state)
    }

    /// Get uptime
    pub async fn get_uptime(&self) -> Duration {
        if let Some(start_time) = self.start_time {
            start_time.elapsed()
        } else {
            Duration::from_secs(0)
        }
    }

    /// Main autonomy loop implementation
    async fn run_loop(
        config: AutonomyConfig,
        state: Arc<RwLock<AutonomyState>>,
        shutdown_signal: Arc<tokio::sync::Notify>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(config.loop_interval_ms));
        let mut iteration_count = 0u64;

        info!("Autonomy loop started with interval: {}ms", config.loop_interval_ms);

        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => {
                    info!("Autonomy loop received shutdown signal");
                    break;
                }
                
                _ = interval.tick() => {
                    iteration_count += 1;
                    
                    // Set processing state
                    *state.write().await = AutonomyState::Processing;
                    
                    debug!("Autonomy loop iteration {}", iteration_count);
                    
                    // Perform autonomous tasks
                    if let Err(e) = Self::process_iteration(&config).await {
                        error!("Error in autonomy loop iteration {}: {}", iteration_count, e);
                        *state.write().await = AutonomyState::Error(e.to_string());
                        
                        // Sleep before retrying
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        *state.write().await = AutonomyState::Idle;
                    } else {
                        // Return to idle state
                        *state.write().await = AutonomyState::Idle;
                    }
                }
            }
        }

        info!("Autonomy loop completed {} iterations", iteration_count);
    }

    /// Process a single autonomy loop iteration
    async fn process_iteration(config: &AutonomyConfig) -> Result<()> {
        // Mock autonomous processing
        debug!("Processing autonomous tasks...");
        
        // Simulate task processing time
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Mock rule evaluation
        if config.rules_config.enabled {
            debug!("Evaluating rules...");
            // Mock rule evaluation
        }

        // Mock AI agent task processing
        if config.ai_config.enabled {
            debug!("Processing AI agent tasks...");
            // Mock AI processing
        }

        // Mock learning if enabled
        if config.enable_learning {
            debug!("Learning from experiences...");
            // Mock learning
        }

        debug!("Autonomous iteration completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AutonomyConfig;

    #[tokio::test]
    async fn test_autonomy_loop_creation() {
        let config = AutonomyConfig::default();
        let autonomy_loop = AutonomyLoop::new(config).await;
        assert!(autonomy_loop.is_ok());
    }

    #[tokio::test]
    async fn test_autonomy_loop_initialization() {
        let config = AutonomyConfig::default();
        let mut autonomy_loop = AutonomyLoop::new(config).await.unwrap();
        
        autonomy_loop.initialize().await.unwrap();
        let state = autonomy_loop.get_state().await;
        assert_eq!(state, AutonomyState::Idle);
    }

    #[tokio::test]
    async fn test_autonomy_loop_start_stop() {
        let config = AutonomyConfig::default();
        let mut autonomy_loop = AutonomyLoop::new(config).await.unwrap();
        
        autonomy_loop.initialize().await.unwrap();
        autonomy_loop.start().await.unwrap();
        
        // Let it run for a brief moment
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        autonomy_loop.stop().await.unwrap();
        let state = autonomy_loop.get_state().await;
        assert_eq!(state, AutonomyState::Stopped);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AutonomyConfig::default();
        let autonomy_loop = AutonomyLoop::new(config).await.unwrap();
        
        let health = autonomy_loop.health_check().await.unwrap();
        assert!(health); // Should be healthy when just created
    }

    #[tokio::test]
    async fn test_uptime() {
        let config = AutonomyConfig::default();
        let mut autonomy_loop = AutonomyLoop::new(config).await.unwrap();
        
        let uptime_before = autonomy_loop.get_uptime().await;
        assert_eq!(uptime_before, Duration::from_secs(0));
        
        autonomy_loop.start().await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let uptime_after = autonomy_loop.get_uptime().await;
        assert!(uptime_after > Duration::from_secs(0));
        
        autonomy_loop.stop().await.unwrap();
    }
}