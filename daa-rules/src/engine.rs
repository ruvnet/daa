//! Rules engine implementation

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{Rule, RuleResult, RulesError, Result};
use crate::context::ExecutionContext;
use crate::conditions::ConditionEvaluator;
use crate::actions::ActionExecutor;

/// Main rules engine
pub struct RuleEngine {
    /// Stored rules
    rules: Arc<RwLock<HashMap<String, Rule>>>,
    
    /// Condition evaluator
    condition_evaluator: ConditionEvaluator,
    
    /// Action executor
    action_executor: ActionExecutor,
}

impl RuleEngine {
    /// Create a new rules engine
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            condition_evaluator: ConditionEvaluator::new(),
            action_executor: ActionExecutor::new(),
        }
    }

    /// Add a rule to the engine
    pub async fn add_rule(&mut self, rule: Rule) -> Result<()> {
        rule.is_valid()?;
        self.rules.write().await.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Execute a rule
    pub async fn execute_rule(&self, rule: &Rule, context: &mut ExecutionContext) -> Result<RuleResult> {
        if !rule.enabled {
            return Ok(RuleResult::Skipped);
        }

        // Evaluate conditions
        for condition in &rule.conditions {
            if !self.condition_evaluator.evaluate_condition(condition, context).await? {
                return Ok(RuleResult::Skipped);
            }
        }

        // Execute actions
        for action in &rule.actions {
            self.action_executor.execute_action(action, context).await?;
        }

        Ok(RuleResult::Allow)
    }

    /// Evaluate a condition
    pub async fn evaluate_condition(&self, condition: &crate::RuleCondition, context: &ExecutionContext) -> Result<bool> {
        self.condition_evaluator.evaluate_condition(condition, context).await
    }

    /// Execute an action
    pub async fn execute_action(&self, action: &crate::RuleAction, context: &mut ExecutionContext) -> Result<()> {
        self.action_executor.execute_action(action, context).await
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}