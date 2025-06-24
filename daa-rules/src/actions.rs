//! Action execution for rules

use crate::{RuleAction, Result, RulesError, LogLevel};
use crate::context::ExecutionContext;

/// Action executor
pub struct ActionExecutor;

impl ActionExecutor {
    /// Create a new action executor
    pub fn new() -> Self {
        Self
    }

    /// Execute an action
    pub async fn execute_action(&self, action: &RuleAction, context: &mut ExecutionContext) -> Result<()> {
        match action {
            RuleAction::SetField { field, value } => {
                context.set_variable(field.clone(), value.clone());
                Ok(())
            }
            
            RuleAction::Log { level, message } => {
                match level {
                    LogLevel::Trace => tracing::trace!("{}", message),
                    LogLevel::Debug => tracing::debug!("{}", message),
                    LogLevel::Info => tracing::info!("{}", message),
                    LogLevel::Warn => tracing::warn!("{}", message),
                    LogLevel::Error => tracing::error!("{}", message),
                }
                Ok(())
            }
            
            RuleAction::ModifyContext { modifications } => {
                for (key, value) in modifications {
                    context.set_variable(key.clone(), value.clone());
                }
                Ok(())
            }
            
            RuleAction::Abort { reason } => {
                Err(RulesError::ActionExecution(format!("Execution aborted: {}", reason)))
            }
            
            _ => {
                // For other action types, log a warning
                tracing::warn!("Unhandled action type: {:?}", action);
                Ok(())
            }
        }
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}