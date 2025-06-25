//! Condition evaluation for rules

use regex::Regex;
use crate::{RuleCondition, Result, RulesError};
use crate::context::ExecutionContext;

/// Condition evaluator
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    /// Create a new condition evaluator
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a condition against the context
    pub async fn evaluate_condition(&self, condition: &RuleCondition, context: &ExecutionContext) -> Result<bool> {
        match condition {
            RuleCondition::Equals { field, value } => {
                if let Some(field_value) = context.get_variable(field) {
                    Ok(field_value == value)
                } else {
                    Ok(false)
                }
            }
            
            RuleCondition::NotEquals { field, value } => {
                if let Some(field_value) = context.get_variable(field) {
                    Ok(field_value != value)
                } else {
                    Ok(true)
                }
            }
            
            RuleCondition::GreaterThan { field, value } => {
                if let Some(field_value) = context.get_variable(field) {
                    if let Ok(num) = field_value.parse::<f64>() {
                        Ok(num > *value)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            
            RuleCondition::LessThan { field, value } => {
                if let Some(field_value) = context.get_variable(field) {
                    if let Ok(num) = field_value.parse::<f64>() {
                        Ok(num < *value)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            
            RuleCondition::Matches { field, pattern } => {
                if let Some(field_value) = context.get_variable(field) {
                    let regex = Regex::new(pattern)
                        .map_err(|e| RulesError::ConditionEvaluation(format!("Invalid regex: {}", e)))?;
                    Ok(regex.is_match(field_value))
                } else {
                    Ok(false)
                }
            }
            
            RuleCondition::Exists { field } => {
                Ok(context.get_variable(field).is_some())
            }
            
            RuleCondition::In { field, values } => {
                if let Some(field_value) = context.get_variable(field) {
                    Ok(values.contains(field_value))
                } else {
                    Ok(false)
                }
            }
            
            RuleCondition::And { conditions } => {
                for condition in conditions {
                    if !Box::pin(self.evaluate_condition(condition, context)).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            
            RuleCondition::Or { conditions } => {
                for condition in conditions {
                    if Box::pin(self.evaluate_condition(condition, context)).await? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            
            RuleCondition::Not { condition } => {
                let result = Box::pin(self.evaluate_condition(condition, context)).await?;
                Ok(!result)
            }
            
            _ => {
                // For other condition types, default to false
                tracing::warn!("Unhandled condition type: {:?}", condition);
                Ok(false)
            }
        }
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}