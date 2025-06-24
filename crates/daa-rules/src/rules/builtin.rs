//! Built-in rule implementations

use super::{Rule, RuleEvaluationResult, RuleViolation, Severity, ViolationType};
use crate::context::{StateContext, OperationalMode, TransactionType};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rule to enforce maximum daily spending limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxDailySpendingRule {
    pub id: String,
    pub name: String,
    pub max_daily_amount: u128,
    pub enabled: bool,
}

#[async_trait]
impl Rule for MaxDailySpendingRule {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Enforces maximum daily spending limits per account"
    }
    
    async fn evaluate(&self, context: &StateContext) -> Result<RuleEvaluationResult> {
        let start_time = std::time::Instant::now();
        
        if let Some(tx) = &context.current_transaction {
            // Check if transaction amount exceeds daily limit
            if tx.amount > self.max_daily_amount {
                let violation = RuleViolation::new(
                    self.id.clone(),
                    ViolationType::ExceedsMaximum,
                    format!("Transaction amount {} exceeds daily limit {}", tx.amount, self.max_daily_amount),
                    Severity::Error,
                )
                .with_field("amount".to_string())
                .with_values(
                    serde_json::json!(self.max_daily_amount),
                    serde_json::json!(tx.amount),
                );
                
                return Ok(RuleEvaluationResult::failure(
                    self.id.clone(),
                    vec![violation],
                    start_time.elapsed().as_millis() as u64,
                ));
            }
        }
        
        Ok(RuleEvaluationResult::success(
            self.id.clone(),
            start_time.elapsed().as_millis() as u64,
        ))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn config(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Rule to enforce risk thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholdRule {
    pub id: String,
    pub name: String,
    pub max_risk_score: f64,
    pub enabled: bool,
}

#[async_trait]
impl Rule for RiskThresholdRule {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Enforces maximum risk score thresholds for transactions"
    }
    
    async fn evaluate(&self, context: &StateContext) -> Result<RuleEvaluationResult> {
        let start_time = std::time::Instant::now();
        
        // Calculate risk score based on transaction and context
        let risk_score = self.calculate_risk_score(context);
        
        if risk_score > self.max_risk_score {
            let violation = RuleViolation::new(
                self.id.clone(),
                ViolationType::ExceedsMaximum,
                format!("Risk score {} exceeds maximum threshold {}", risk_score, self.max_risk_score),
                Severity::Warning,
            )
            .with_field("risk_score".to_string())
            .with_values(
                serde_json::json!(self.max_risk_score),
                serde_json::json!(risk_score),
            );
            
            return Ok(RuleEvaluationResult::failure(
                self.id.clone(),
                vec![violation],
                start_time.elapsed().as_millis() as u64,
            ));
        }
        
        Ok(RuleEvaluationResult::success(
            self.id.clone(),
            start_time.elapsed().as_millis() as u64,
        ))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn config(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl RiskThresholdRule {
    fn calculate_risk_score(&self, context: &StateContext) -> f64 {
        let mut score = 0.0;
        
        if let Some(tx) = &context.current_transaction {
            // Higher amounts = higher risk
            score += (tx.amount as f64) / 1_000_000.0;
            
            // Different transaction types have different risk levels
            score += match tx.transaction_type {
                TransactionType::Transfer => 0.1,
                TransactionType::Mint => 0.5,
                TransactionType::Burn => 0.3,
                TransactionType::Governance => 0.8,
                _ => 0.2,
            };
        }
        
        // System state affects risk
        match context.system_state.operational_mode {
            OperationalMode::Emergency => score += 2.0,
            OperationalMode::Maintenance => score += 1.0,
            OperationalMode::Paused => score += 5.0,
            OperationalMode::Normal => {},
        }
        
        score
    }
}

/// Rule to enforce minimum balance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimumBalanceRule {
    pub id: String,
    pub name: String,
    pub minimum_balance: u128,
    pub enabled: bool,
}

#[async_trait]
impl Rule for MinimumBalanceRule {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Enforces minimum balance requirements for accounts"
    }
    
    async fn evaluate(&self, context: &StateContext) -> Result<RuleEvaluationResult> {
        let start_time = std::time::Instant::now();
        
        if let Some(tx) = &context.current_transaction {
            let from_balance = context.get_balance(&tx.from);
            let remaining_balance = from_balance.saturating_sub(tx.amount);
            
            if remaining_balance < self.minimum_balance {
                let violation = RuleViolation::new(
                    self.id.clone(),
                    ViolationType::BelowMinimum,
                    format!("Remaining balance {} below minimum required {}", remaining_balance, self.minimum_balance),
                    Severity::Error,
                )
                .with_field("balance".to_string())
                .with_values(
                    serde_json::json!(self.minimum_balance),
                    serde_json::json!(remaining_balance),
                );
                
                return Ok(RuleEvaluationResult::failure(
                    self.id.clone(),
                    vec![violation],
                    start_time.elapsed().as_millis() as u64,
                ));
            }
        }
        
        Ok(RuleEvaluationResult::success(
            self.id.clone(),
            start_time.elapsed().as_millis() as u64,
        ))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn config(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Rule to enforce maximum transaction amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxTransactionAmountRule {
    pub id: String,
    pub name: String,
    pub max_amount: u128,
    pub enabled: bool,
}

#[async_trait]
impl Rule for MaxTransactionAmountRule {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Enforces maximum transaction amount limits"
    }
    
    async fn evaluate(&self, context: &StateContext) -> Result<RuleEvaluationResult> {
        let start_time = std::time::Instant::now();
        
        if let Some(tx) = &context.current_transaction {
            if tx.amount > self.max_amount {
                let violation = RuleViolation::new(
                    self.id.clone(),
                    ViolationType::ExceedsMaximum,
                    format!("Transaction amount {} exceeds maximum allowed {}", tx.amount, self.max_amount),
                    Severity::Error,
                )
                .with_field("amount".to_string())
                .with_values(
                    serde_json::json!(self.max_amount),
                    serde_json::json!(tx.amount),
                );
                
                return Ok(RuleEvaluationResult::failure(
                    self.id.clone(),
                    vec![violation],
                    start_time.elapsed().as_millis() as u64,
                ));
            }
        }
        
        Ok(RuleEvaluationResult::success(
            self.id.clone(),
            start_time.elapsed().as_millis() as u64,
        ))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn config(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Rule to enforce operational hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalHoursRule {
    pub id: String,
    pub name: String,
    pub start_hour: u32,
    pub end_hour: u32,
    pub enabled: bool,
}

#[async_trait]
impl Rule for OperationalHoursRule {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Enforces operational hours for transactions"
    }
    
    async fn evaluate(&self, context: &StateContext) -> Result<RuleEvaluationResult> {
        let start_time = std::time::Instant::now();
        
        let current_hour = context.system_state.current_time.hour();
        
        if current_hour < self.start_hour || current_hour >= self.end_hour {
            let violation = RuleViolation::new(
                self.id.clone(),
                ViolationType::TimeConstraintViolated,
                format!("Transaction attempted outside operational hours ({}-{})", self.start_hour, self.end_hour),
                Severity::Warning,
            )
            .with_field("hour".to_string())
            .with_values(
                serde_json::json!(format!("{}-{}", self.start_hour, self.end_hour)),
                serde_json::json!(current_hour),
            );
            
            return Ok(RuleEvaluationResult::failure(
                self.id.clone(),
                vec![violation],
                start_time.elapsed().as_millis() as u64,
            ));
        }
        
        Ok(RuleEvaluationResult::success(
            self.id.clone(),
            start_time.elapsed().as_millis() as u64,
        ))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn config(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Rule to enforce rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    pub id: String,
    pub name: String,
    pub max_requests_per_minute: u32,
    pub enabled: bool,
    #[serde(skip)]
    pub request_counts: HashMap<String, (chrono::DateTime<chrono::Utc>, u32)>,
}

#[async_trait]
impl Rule for RateLimitRule {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Enforces rate limits for transactions per account"
    }
    
    async fn evaluate(&self, context: &StateContext) -> Result<RuleEvaluationResult> {
        let start_time = std::time::Instant::now();
        
        if let Some(tx) = &context.current_transaction {
            // In a real implementation, this would need to be thread-safe
            // and persistent across rule engine instances
            let current_count = self.get_request_count(&tx.from);
            
            if current_count >= self.max_requests_per_minute {
                let violation = RuleViolation::new(
                    self.id.clone(),
                    ViolationType::RateLimitExceeded,
                    format!("Rate limit exceeded: {} requests per minute", current_count),
                    Severity::Warning,
                )
                .with_field("request_count".to_string())
                .with_values(
                    serde_json::json!(self.max_requests_per_minute),
                    serde_json::json!(current_count),
                );
                
                return Ok(RuleEvaluationResult::failure(
                    self.id.clone(),
                    vec![violation],
                    start_time.elapsed().as_millis() as u64,
                ));
            }
        }
        
        Ok(RuleEvaluationResult::success(
            self.id.clone(),
            start_time.elapsed().as_millis() as u64,
        ))
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn config(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl RateLimitRule {
    fn get_request_count(&self, account: &str) -> u32 {
        // This is a simplified implementation
        // In practice, this would be stored in a shared state
        let now = chrono::Utc::now();
        let minute_ago = now - chrono::Duration::minutes(1);
        
        // Mock implementation - always return 0 for now
        // Real implementation would track requests per account
        0
    }
}