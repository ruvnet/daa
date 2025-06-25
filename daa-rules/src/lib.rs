//! # DAA Rules
//!
//! A comprehensive rules engine for the Decentralized Autonomous Agents (DAA) system.
//! Provides policy enforcement, decision automation, and governance capabilities.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regex::Regex;
use async_trait::async_trait;

pub mod engine;
pub mod conditions;
pub mod actions;
pub mod context;
pub mod storage;

#[cfg(feature = "scripting")]
pub mod scripting;

#[cfg(feature = "database")]
pub mod database;

/// Rules engine error types
#[derive(Error, Debug)]
pub enum RulesError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    
    #[error("Invalid rule definition: {0}")]
    InvalidRule(String),
    
    #[error("Condition evaluation failed: {0}")]
    ConditionEvaluation(String),
    
    #[error("Action execution failed: {0}")]
    ActionExecution(String),
    
    #[error("Context error: {0}")]
    Context(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Scripting error: {0}")]
    Scripting(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Parsing error: {0}")]
    Parsing(String),
}

pub type Result<T> = std::result::Result<T, RulesError>;

/// Rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique rule identifier
    pub id: String,
    
    /// Human-readable rule name
    pub name: String,
    
    /// Rule description
    pub description: String,
    
    /// Rule conditions that must be met
    pub conditions: Vec<RuleCondition>,
    
    /// Actions to execute when conditions are met
    pub actions: Vec<RuleAction>,
    
    /// Rule priority (higher number = higher priority)
    pub priority: u32,
    
    /// Whether the rule is enabled
    pub enabled: bool,
    
    /// Rule creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Rule last modified timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Rule metadata
    pub metadata: HashMap<String, String>,
}

impl Rule {
    /// Create a new rule
    pub fn new(
        id: String,
        name: String,
        conditions: Vec<RuleCondition>,
        actions: Vec<RuleAction>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id,
            name,
            description: String::new(),
            conditions,
            actions,
            priority: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create a new rule with generated ID
    pub fn new_with_generated_id(
        name: String,
        conditions: Vec<RuleCondition>,
        actions: Vec<RuleAction>,
    ) -> Self {
        Self::new(Uuid::new_v4().to_string(), name, conditions, actions)
    }

    /// Check if rule is valid
    pub fn is_valid(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(RulesError::InvalidRule("Rule ID cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(RulesError::InvalidRule("Rule name cannot be empty".to_string()));
        }

        if self.conditions.is_empty() {
            return Err(RulesError::InvalidRule("Rule must have at least one condition".to_string()));
        }

        if self.actions.is_empty() {
            return Err(RulesError::InvalidRule("Rule must have at least one action".to_string()));
        }

        // Validate conditions
        for condition in &self.conditions {
            condition.validate()?;
        }

        // Validate actions
        for action in &self.actions {
            action.validate()?;
        }

        Ok(())
    }
}

/// Rule condition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Simple equality check
    Equals {
        field: String,
        value: String,
    },
    
    /// Inequality check
    NotEquals {
        field: String,
        value: String,
    },
    
    /// Greater than comparison
    GreaterThan {
        field: String,
        value: f64,
    },
    
    /// Less than comparison
    LessThan {
        field: String,
        value: f64,
    },
    
    /// Pattern matching with regex
    Matches {
        field: String,
        pattern: String,
    },
    
    /// Field existence check
    Exists {
        field: String,
    },
    
    /// Value in list check
    In {
        field: String,
        values: Vec<String>,
    },
    
    /// Time-based condition
    TimeCondition {
        field: String,
        operator: TimeOperator,
        value: DateTime<Utc>,
    },
    
    /// Complex logical condition
    And {
        conditions: Vec<RuleCondition>,
    },
    
    /// Complex logical condition
    Or {
        conditions: Vec<RuleCondition>,
    },
    
    /// Negation condition
    Not {
        condition: Box<RuleCondition>,
    },
    
    /// Custom condition with parameters
    Custom {
        condition_type: String,
        parameters: HashMap<String, String>,
    },
}

impl RuleCondition {
    /// Validate the condition
    pub fn validate(&self) -> Result<()> {
        match self {
            RuleCondition::Matches { pattern, .. } => {
                Regex::new(pattern)
                    .map_err(|e| RulesError::InvalidRule(format!("Invalid regex pattern: {}", e)))?;
            }
            RuleCondition::And { conditions } | RuleCondition::Or { conditions } => {
                if conditions.is_empty() {
                    return Err(RulesError::InvalidRule("Logical conditions must have at least one sub-condition".to_string()));
                }
                for condition in conditions {
                    condition.validate()?;
                }
            }
            RuleCondition::Not { condition } => {
                condition.validate()?;
            }
            _ => {} // Other conditions are always valid
        }
        Ok(())
    }
}

/// Time comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeOperator {
    Before,
    After,
    Between { end: DateTime<Utc> },
}

/// Rule action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Set a field value
    SetField {
        field: String,
        value: String,
    },
    
    /// Log a message
    Log {
        level: LogLevel,
        message: String,
    },
    
    /// Send notification
    Notify {
        recipient: String,
        message: String,
        channel: NotificationChannel,
    },
    
    /// Execute script
    #[cfg(feature = "scripting")]
    Script {
        script_type: String,
        script: String,
    },
    
    /// Trigger external webhook
    Webhook {
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body: String,
    },
    
    /// Modify context
    ModifyContext {
        modifications: HashMap<String, String>,
    },
    
    /// Abort execution
    Abort {
        reason: String,
    },
    
    /// Custom action with parameters
    Custom {
        action_type: String,
        parameters: HashMap<String, String>,
    },
}

impl RuleAction {
    /// Validate the action
    pub fn validate(&self) -> Result<()> {
        match self {
            RuleAction::Webhook { url, .. } => {
                if url.is_empty() {
                    return Err(RulesError::InvalidRule("Webhook URL cannot be empty".to_string()));
                }
            }
            _ => {} // Other actions are always valid
        }
        Ok(())
    }
}

/// Log levels for logging actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Slack,
    Discord,
    Webhook,
    Internal,
}

/// Result of rule execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleResult {
    /// Rule conditions were met and actions executed successfully
    Allow,
    
    /// Rule conditions were met but execution was denied
    Deny(String),
    
    /// Rule execution resulted in modifications
    Modified(HashMap<String, String>),
    
    /// Rule execution was skipped (conditions not met)
    Skipped,
    
    /// Rule execution failed
    Failed(String),
}

impl fmt::Display for RuleResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleResult::Allow => write!(f, "Allow"),
            RuleResult::Deny(reason) => write!(f, "Deny: {}", reason),
            RuleResult::Modified(changes) => write!(f, "Modified: {:?}", changes),
            RuleResult::Skipped => write!(f, "Skipped"),
            RuleResult::Failed(error) => write!(f, "Failed: {}", error),
        }
    }
}

/// Execution context for rules
pub use context::ExecutionContext;

/// Rules engine
pub use engine::RuleEngine;

/// Re-export key types for convenience
pub use conditions::ConditionEvaluator;
pub use actions::ActionExecutor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = Rule::new_with_generated_id(
            "Test Rule".to_string(),
            vec![RuleCondition::Equals {
                field: "status".to_string(),
                value: "active".to_string(),
            }],
            vec![RuleAction::Log {
                level: LogLevel::Info,
                message: "Rule triggered".to_string(),
            }],
        );

        assert!(!rule.id.is_empty());
        assert_eq!(rule.name, "Test Rule");
        assert!(rule.enabled);
        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(rule.actions.len(), 1);
    }

    #[test]
    fn test_rule_validation() {
        let rule = Rule::new_with_generated_id(
            "Valid Rule".to_string(),
            vec![RuleCondition::Equals {
                field: "test".to_string(),
                value: "value".to_string(),
            }],
            vec![RuleAction::Log {
                level: LogLevel::Info,
                message: "test".to_string(),
            }],
        );

        assert!(rule.is_valid().is_ok());
    }

    #[test]
    fn test_invalid_rule_validation() {
        let rule = Rule::new(
            String::new(), // Empty ID should be invalid
            "Invalid Rule".to_string(),
            vec![RuleCondition::Equals {
                field: "test".to_string(),
                value: "value".to_string(),
            }],
            vec![RuleAction::Log {
                level: LogLevel::Info,
                message: "test".to_string(),
            }],
        );

        assert!(rule.is_valid().is_err());
    }

    #[test]
    fn test_condition_validation() {
        let valid_condition = RuleCondition::Matches {
            field: "email".to_string(),
            pattern: r"^[^@]+@[^@]+\.[^@]+$".to_string(),
        };
        assert!(valid_condition.validate().is_ok());

        let invalid_condition = RuleCondition::Matches {
            field: "email".to_string(),
            pattern: "[".to_string(), // Invalid regex
        };
        assert!(invalid_condition.validate().is_err());
    }

    #[test]
    fn test_rule_result_display() {
        assert_eq!(RuleResult::Allow.to_string(), "Allow");
        assert_eq!(RuleResult::Deny("test".to_string()).to_string(), "Deny: test");
        assert_eq!(RuleResult::Skipped.to_string(), "Skipped");
    }
}