//! Rule definitions and built-in rules

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::context::StateContext;
use crate::error::{Result, RuleError};

/// A rule violation with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_id: String,
    pub message: String,
    pub severity: ViolationSeverity,
    pub timestamp: DateTime<Utc>,
    pub context: serde_json::Value,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A rule that can be evaluated against a state context
pub trait Rule: Send + Sync {
    /// Unique identifier for the rule
    fn id(&self) -> &str;
    
    /// Human-readable description of the rule
    fn description(&self) -> &str;
    
    /// Evaluate the rule against the given context
    fn evaluate(&self, context: &StateContext) -> Result<Option<RuleViolation>>;
    
    /// Check if the rule is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Built-in rules module
pub mod builtin {
    use super::*;

    /// Rule that limits maximum daily spending
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MaxDailySpendingRule {
        pub max_amount: u128,
        pub enabled: bool,
    }

    impl MaxDailySpendingRule {
        pub fn new(max_amount: u128) -> Self {
            Self {
                max_amount,
                enabled: true,
            }
        }
    }

    impl Rule for MaxDailySpendingRule {
        fn id(&self) -> &str {
            "max_daily_spending"
        }

        fn description(&self) -> &str {
            "Limits maximum daily spending amount"
        }

        fn evaluate(&self, context: &StateContext) -> Result<Option<RuleViolation>> {
            if !self.enabled {
                return Ok(None);
            }

            let today = context.timestamp.format("%Y-%m-%d").to_string();
            let daily_total = context.get_daily_spending(&today);

            if daily_total > self.max_amount {
                Ok(Some(RuleViolation {
                    rule_id: self.id().to_string(),
                    message: format!(
                        "Daily spending limit exceeded: {} > {}",
                        daily_total, self.max_amount
                    ),
                    severity: ViolationSeverity::Error,
                    timestamp: Utc::now(),
                    context: serde_json::json!({
                        "daily_total": daily_total,
                        "max_amount": self.max_amount,
                        "date": today
                    }),
                }))
            } else {
                Ok(None)
            }
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    /// Rule that enforces minimum balance requirements
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MinimumBalanceRule {
        pub address: String,
        pub min_balance: u128,
        pub enabled: bool,
    }

    impl MinimumBalanceRule {
        pub fn new(address: String, min_balance: u128) -> Self {
            Self {
                address,
                min_balance,
                enabled: true,
            }
        }
    }

    impl Rule for MinimumBalanceRule {
        fn id(&self) -> &str {
            "minimum_balance"
        }

        fn description(&self) -> &str {
            "Ensures minimum balance is maintained"
        }

        fn evaluate(&self, context: &StateContext) -> Result<Option<RuleViolation>> {
            if !self.enabled {
                return Ok(None);
            }

            let balance = context.get_balance(&self.address);

            if balance < self.min_balance {
                Ok(Some(RuleViolation {
                    rule_id: self.id().to_string(),
                    message: format!(
                        "Minimum balance requirement not met for {}: {} < {}",
                        self.address, balance, self.min_balance
                    ),
                    severity: ViolationSeverity::Warning,
                    timestamp: Utc::now(),
                    context: serde_json::json!({
                        "address": self.address,
                        "current_balance": balance,
                        "min_balance": self.min_balance
                    }),
                }))
            } else {
                Ok(None)
            }
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    /// Rule that limits maximum transaction amount
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MaxTransactionAmountRule {
        pub max_amount: u128,
        pub enabled: bool,
    }

    impl MaxTransactionAmountRule {
        pub fn new(max_amount: u128) -> Self {
            Self {
                max_amount,
                enabled: true,
            }
        }
    }

    impl Rule for MaxTransactionAmountRule {
        fn id(&self) -> &str {
            "max_transaction_amount"
        }

        fn description(&self) -> &str {
            "Limits maximum single transaction amount"
        }

        fn evaluate(&self, context: &StateContext) -> Result<Option<RuleViolation>> {
            if !self.enabled {
                return Ok(None);
            }

            for &amount in &context.transaction_amounts {
                if amount > self.max_amount {
                    return Ok(Some(RuleViolation {
                        rule_id: self.id().to_string(),
                        message: format!(
                            "Transaction amount exceeds limit: {} > {}",
                            amount, self.max_amount
                        ),
                        severity: ViolationSeverity::Error,
                        timestamp: Utc::now(),
                        context: serde_json::json!({
                            "transaction_amount": amount,
                            "max_amount": self.max_amount
                        }),
                    }));
                }
            }

            Ok(None)
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    /// Rule that enforces operational hours
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OperationalHoursRule {
        pub start_hour: u32,
        pub end_hour: u32,
        pub enabled: bool,
    }

    impl OperationalHoursRule {
        pub fn new(start_hour: u32, end_hour: u32) -> Self {
            Self {
                start_hour,
                end_hour,
                enabled: true,
            }
        }

        pub fn business_hours() -> Self {
            Self::new(9, 17) // 9 AM to 5 PM
        }
    }

    impl Rule for OperationalHoursRule {
        fn id(&self) -> &str {
            "operational_hours"
        }

        fn description(&self) -> &str {
            "Restricts operations to specific hours"
        }

        fn evaluate(&self, context: &StateContext) -> Result<Option<RuleViolation>> {
            if !self.enabled {
                return Ok(None);
            }

            let current_hour = context.timestamp.hour();

            if current_hour < self.start_hour || current_hour >= self.end_hour {
                Ok(Some(RuleViolation {
                    rule_id: self.id().to_string(),
                    message: format!(
                        "Operation outside allowed hours: {} (allowed: {}-{})",
                        current_hour, self.start_hour, self.end_hour
                    ),
                    severity: ViolationSeverity::Warning,
                    timestamp: Utc::now(),
                    context: serde_json::json!({
                        "current_hour": current_hour,
                        "start_hour": self.start_hour,
                        "end_hour": self.end_hour
                    }),
                }))
            } else {
                Ok(None)
            }
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    /// Rule that implements rate limiting
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RateLimitRule {
        pub key: String,
        pub max_requests: u32,
        pub window_seconds: u64,
        pub enabled: bool,
    }

    impl RateLimitRule {
        pub fn new(key: String, max_requests: u32, window_seconds: u64) -> Self {
            Self {
                key,
                max_requests,
                window_seconds,
                enabled: true,
            }
        }
    }

    impl Rule for RateLimitRule {
        fn id(&self) -> &str {
            "rate_limit"
        }

        fn description(&self) -> &str {
            "Implements rate limiting for operations"
        }

        fn evaluate(&self, context: &StateContext) -> Result<Option<RuleViolation>> {
            if !self.enabled {
                return Ok(None);
            }

            let current_count = context.get_rate_limit(&self.key);

            if current_count > self.max_requests {
                Ok(Some(RuleViolation {
                    rule_id: self.id().to_string(),
                    message: format!(
                        "Rate limit exceeded for {}: {} > {}",
                        self.key, current_count, self.max_requests
                    ),
                    severity: ViolationSeverity::Error,
                    timestamp: Utc::now(),
                    context: serde_json::json!({
                        "key": self.key,
                        "current_count": current_count,
                        "max_requests": self.max_requests,
                        "window_seconds": self.window_seconds
                    }),
                }))
            } else {
                Ok(None)
            }
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    /// Rule that enforces risk thresholds
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RiskThresholdRule {
        pub max_risk_score: f64,
        pub enabled: bool,
    }

    impl RiskThresholdRule {
        pub fn new(max_risk_score: f64) -> Self {
            Self {
                max_risk_score,
                enabled: true,
            }
        }

        fn calculate_risk_score(&self, context: &StateContext) -> f64 {
            // Simple risk calculation based on transaction amounts
            let total_amount: u128 = context.transaction_amounts.iter().sum();
            let avg_amount = if context.transaction_amounts.is_empty() {
                0.0
            } else {
                total_amount as f64 / context.transaction_amounts.len() as f64
            };

            // Risk increases with higher amounts and more transactions
            let amount_risk = (avg_amount / 10000.0).min(1.0);
            let frequency_risk = (context.transaction_amounts.len() as f64 / 100.0).min(1.0);
            
            (amount_risk + frequency_risk) / 2.0
        }
    }

    impl Rule for RiskThresholdRule {
        fn id(&self) -> &str {
            "risk_threshold"
        }

        fn description(&self) -> &str {
            "Monitors risk score and alerts on threshold breach"
        }

        fn evaluate(&self, context: &StateContext) -> Result<Option<RuleViolation>> {
            if !self.enabled {
                return Ok(None);
            }

            let risk_score = self.calculate_risk_score(context);

            if risk_score > self.max_risk_score {
                Ok(Some(RuleViolation {
                    rule_id: self.id().to_string(),
                    message: format!(
                        "Risk threshold exceeded: {:.2} > {:.2}",
                        risk_score, self.max_risk_score
                    ),
                    severity: ViolationSeverity::Critical,
                    timestamp: Utc::now(),
                    context: serde_json::json!({
                        "risk_score": risk_score,
                        "max_risk_score": self.max_risk_score,
                        "transaction_count": context.transaction_amounts.len()
                    }),
                }))
            } else {
                Ok(None)
            }
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }
}

impl fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViolationSeverity::Info => write!(f, "INFO"),
            ViolationSeverity::Warning => write!(f, "WARNING"),
            ViolationSeverity::Error => write!(f, "ERROR"),
            ViolationSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.severity, self.rule_id, self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::builtin::*;

    #[test]
    fn test_max_daily_spending_rule() {
        let rule = MaxDailySpendingRule::new(1000);
        let mut context = StateContext::new();
        
        // No violation initially
        assert!(rule.evaluate(&context).unwrap().is_none());
        
        // Add spending that exceeds limit
        let today = context.timestamp.format("%Y-%m-%d").to_string();
        context.add_daily_spending(today, 1500);
        
        let violation = rule.evaluate(&context).unwrap();
        assert!(violation.is_some());
        assert_eq!(violation.unwrap().severity, ViolationSeverity::Error);
    }

    #[test]
    fn test_minimum_balance_rule() {
        let rule = MinimumBalanceRule::new("addr1".to_string(), 100);
        let mut context = StateContext::new();
        
        // Violation when balance is below minimum
        let violation = rule.evaluate(&context).unwrap();
        assert!(violation.is_some());
        
        // No violation when balance meets minimum
        context.set_balance("addr1".to_string(), 150);
        assert!(rule.evaluate(&context).unwrap().is_none());
    }
}