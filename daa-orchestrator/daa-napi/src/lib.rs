//! # DAA Orchestrator NAPI Bindings
//!
//! Node.js bindings for the DAA (Decentralized Autonomous Agents) Orchestrator.
//! Provides access to the MRAP loop, workflow engine, rules engine, and token economy
//! from JavaScript/TypeScript applications.
//!
//! ## Features
//!
//! - **Orchestrator**: MRAP (Monitor, Reason, Act, Plan) autonomy loop
//! - **Workflow Engine**: Create and execute complex workflows
//! - **Rules Engine**: Define and evaluate business rules
//! - **Economy Manager**: Token management and account operations
//!
//! ## Example
//!
//! ```javascript
//! const { Orchestrator, WorkflowEngine, RulesEngine, EconomyManager } = require('@daa/orchestrator');
//!
//! // Start the orchestrator with MRAP loop
//! const orchestrator = new Orchestrator({ enabled: true, loopIntervalMs: 1000 });
//! await orchestrator.start();
//!
//! // Get system state
//! const state = await orchestrator.monitor();
//! console.log('System state:', state);
//! ```

#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

// Module declarations
mod economy;
mod orchestrator;
mod rules;
mod workflow;

// Re-export main types for easier access
pub use economy::*;
pub use orchestrator::*;
pub use rules::*;
pub use workflow::*;

/// Initialize the DAA orchestrator library.
/// This sets up logging and prepares the runtime environment.
///
/// # Arguments
///
/// * `log_level` - Log level (trace, debug, info, warn, error). Defaults to "info".
///
/// # Example
///
/// ```javascript
/// const { initialize } = require('@daa/orchestrator');
/// initialize('debug');
/// ```
#[napi]
pub fn initialize(log_level: Option<String>) -> Result<()> {
    let level = log_level.unwrap_or_else(|| "info".to_string());

    // Initialize tracing subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&level)),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| Error::from_reason(format!("Failed to initialize tracing: {}", e)))?;

    tracing::info!(
        "DAA Orchestrator NAPI initialized with log level: {}",
        level
    );

    Ok(())
}

/// Get the library version.
///
/// # Returns
///
/// The version string of the DAA orchestrator library.
///
/// # Example
///
/// ```javascript
/// const { version } = require('@daa/orchestrator');
/// console.log('Version:', version());
/// ```
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Health check for the library.
///
/// # Returns
///
/// A health status object indicating if the library is operational.
///
/// # Example
///
/// ```javascript
/// const { healthCheck } = require('@daa/orchestrator');
/// const health = await healthCheck();
/// console.log('Health:', health);
/// ```
#[napi]
pub async fn health_check() -> Result<HealthStatus> {
    Ok(HealthStatus {
        status: "healthy".to_string(),
        version: version(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Health status response
#[napi(object)]
pub struct HealthStatus {
    /// Overall health status
    pub status: String,
    /// Library version
    pub version: String,
    /// Current timestamp
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
        assert_eq!(v, env!("CARGO_PKG_VERSION"));
    }
}
