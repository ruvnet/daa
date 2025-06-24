//! # DAA Rules Engine
//! 
//! This crate provides a symbolic rule engine for DAA (Decentralized Autonomous Application)
//! that encodes governance rules, safety constraints, and high-level decision policies
//! in an explicit and auditable form.

pub mod context;
pub mod engine;
pub mod rules;
pub mod audit;
pub mod error;

pub use context::StateContext;
pub use engine::{RuleEngine, RuleEvaluationResult};
pub use rules::{Rule, RuleViolation};
pub use audit::{AuditLog, AuditEntry, AuditLogger};
pub use error::{RuleError, Result};

// Re-export commonly used types
pub use rules::builtin::{
    MaxDailySpendingRule,
    RiskThresholdRule,
    MinimumBalanceRule,
    MaxTransactionAmountRule,
    OperationalHoursRule,
    RateLimitRule,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_engine_creation() {
        let engine = RuleEngine::new();
        assert_eq!(engine.rule_count(), 0);
    }
}