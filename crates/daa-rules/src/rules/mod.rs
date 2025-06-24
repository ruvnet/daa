//! Rule definitions and implementations

use crate::context::StateContext;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod builtin;
pub mod qudag;

pub use builtin::*;
pub use qudag::*;

/// Rule trait that all rules must implement
#[async_trait]
pub trait Rule: Debug + Send + Sync {
    /// Unique identifier for this rule
    fn id(&self) -> &str;
    
    /// Human-readable name for this rule
    fn name(&self) -> &str;
    
    /// Description of what this rule does
    fn description(&self) -> &str;
    
    /// Evaluate the rule against the given context
    async fn evaluate(&self, context: &StateContext) -> Result<RuleEvaluationResult>;
    
    /// Check if this rule should be evaluated for the given context
    fn should_evaluate(&self, context: &StateContext) -> bool {
        let _ = context;
        true // Default: always evaluate
    }
    
    /// Get rule configuration as JSON
    fn config(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
    
    /// Priority of this rule (higher number = higher priority)
    fn priority(&self) -> u32 {
        100 // Default priority
    }
    
    /// Whether this rule is enabled
    fn is_enabled(&self) -> bool {
        true // Default: enabled
    }
    
    /// Rule version for compatibility tracking
    fn version(&self) -> &str {
        "1.0.0"
    }
}

/// Result of rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluationResult {
    pub rule_id: String,
    pub passed: bool,
    pub violations: Vec<RuleViolation>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub execution_time_ms: u64,
}

/// Rule violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_id: String,
    pub violation_type: ViolationType,
    pub message: String,
    pub severity: Severity,
    pub field: Option<String>,
    pub expected: Option<serde_json::Value>,
    pub actual: Option<serde_json::Value>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of rule violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationType {
    /// Value exceeds maximum allowed
    ExceedsMaximum,
    /// Value is below minimum required
    BelowMinimum,
    /// Required value is missing
    MissingRequired,
    /// Value format is invalid
    InvalidFormat,
    /// Operation is not allowed
    NotAllowed,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Balance insufficient
    InsufficientBalance,
    /// Time constraint violated
    TimeConstraintViolated,
    /// Custom violation type
    Custom(String),
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl RuleEvaluationResult {
    /// Create a new successful evaluation result
    pub fn success(rule_id: String, execution_time_ms: u64) -> Self {
        Self {
            rule_id,
            passed: true,
            violations: Vec::new(),
            metadata: std::collections::HashMap::new(),
            execution_time_ms,
        }
    }
    
    /// Create a new failed evaluation result
    pub fn failure(rule_id: String, violations: Vec<RuleViolation>, execution_time_ms: u64) -> Self {
        Self {
            rule_id,
            passed: false,
            violations,
            metadata: std::collections::HashMap::new(),
            execution_time_ms,
        }
    }
    
    /// Add metadata to the result
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Check if the result has any critical violations
    pub fn has_critical_violations(&self) -> bool {
        self.violations.iter().any(|v| v.severity == Severity::Critical)
    }
    
    /// Get violations by severity
    pub fn get_violations_by_severity(&self, severity: Severity) -> Vec<&RuleViolation> {
        self.violations.iter().filter(|v| v.severity == severity).collect()
    }
}

impl RuleViolation {
    /// Create a new rule violation
    pub fn new(
        rule_id: String,
        violation_type: ViolationType,
        message: String,
        severity: Severity,
    ) -> Self {
        Self {
            rule_id,
            violation_type,
            message,
            severity,
            field: None,
            expected: None,
            actual: None,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Set the field that caused the violation
    pub fn with_field(mut self, field: String) -> Self {
        self.field = Some(field);
        self
    }
    
    /// Set expected and actual values
    pub fn with_values(mut self, expected: serde_json::Value, actual: serde_json::Value) -> Self {
        self.expected = Some(expected);
        self.actual = Some(actual);
        self
    }
    
    /// Add metadata to the violation
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}