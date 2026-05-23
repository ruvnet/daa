//! Rules engine bindings for NAPI
//!
//! Provides Node.js access to the DAA rules engine for defining and evaluating
//! business rules and policies.

use daa_rules::context::ExecutionContext;
use daa_rules::{Rule, RuleAction, RuleCondition, RuleEngine, RuleResult};
use napi::bindgen_prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Rule definition for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RuleJs {
    /// Rule identifier
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Rule conditions (JSON string)
    pub conditions: String,
    /// Rule actions (JSON string)
    pub actions: String,
    /// Rule priority
    pub priority: Option<u32>,
    /// Whether the rule is enabled
    pub enabled: Option<bool>,
}

/// Rule evaluation result for Node.js
#[napi(object)]
pub struct RuleResultJs {
    /// Result type: "allow", "deny", "modified", "skipped", or "failed"
    pub result_type: String,
    /// Result message or reason
    pub message: Option<String>,
    /// Modifications made (JSON string)
    pub modifications: Option<String>,
}

/// Execution context for rule evaluation
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ExecutionContextJs {
    /// Context data (JSON string)
    pub data: String,
}

impl RuleJs {
    fn to_rust_rule(&self) -> Result<Rule> {
        let conditions: Vec<RuleCondition> = serde_json::from_str(&self.conditions)
            .map_err(|e| Error::from_reason(format!("Invalid conditions JSON: {}", e)))?;

        let actions: Vec<RuleAction> = serde_json::from_str(&self.actions)
            .map_err(|e| Error::from_reason(format!("Invalid actions JSON: {}", e)))?;

        let mut rule = Rule::new(self.id.clone(), self.name.clone(), conditions, actions);

        if let Some(ref desc) = self.description {
            rule.description = desc.clone();
        }

        if let Some(priority) = self.priority {
            rule.priority = priority;
        }

        if let Some(enabled) = self.enabled {
            rule.enabled = enabled;
        }

        Ok(rule)
    }
}

impl RuleResultJs {
    fn from_rust_result(result: RuleResult) -> Self {
        match result {
            RuleResult::Allow => Self {
                result_type: "allow".to_string(),
                message: None,
                modifications: None,
            },
            RuleResult::Deny(reason) => Self {
                result_type: "deny".to_string(),
                message: Some(reason),
                modifications: None,
            },
            RuleResult::Modified(changes) => Self {
                result_type: "modified".to_string(),
                message: None,
                modifications: Some(serde_json::to_string(&changes).unwrap_or_default()),
            },
            RuleResult::Skipped => Self {
                result_type: "skipped".to_string(),
                message: None,
                modifications: None,
            },
            RuleResult::Failed(error) => Self {
                result_type: "failed".to_string(),
                message: Some(error),
                modifications: None,
            },
        }
    }
}

impl ExecutionContextJs {
    fn to_rust_context(&self) -> Result<ExecutionContext> {
        let data: serde_json::Value = serde_json::from_str(&self.data)
            .map_err(|e| Error::from_reason(format!("Invalid context data JSON: {}", e)))?;

        let mut context = ExecutionContext::new();

        // Populate context from JSON data
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                if let Some(s) = value.as_str() {
                    context.set_variable(key.clone(), s.to_string());
                }
            }
        }

        Ok(context)
    }
}

/// Rules engine for defining and evaluating rules
#[napi]
pub struct RulesEngineWrapper {
    engine: Arc<Mutex<RuleEngine>>,
}

#[napi]
impl RulesEngineWrapper {
    /// Create a new rules engine instance.
    ///
    /// # Returns
    ///
    /// A new RulesEngine instance
    ///
    /// # Example
    ///
    /// ```javascript
    /// const rulesEngine = new RulesEngine();
    /// ```
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: Arc::new(Mutex::new(RuleEngine::new())),
        })
    }

    /// Add a rule to the engine.
    ///
    /// # Arguments
    ///
    /// * `rule` - Rule definition to add
    ///
    /// # Returns
    ///
    /// Promise that resolves when the rule is added
    ///
    /// # Example
    ///
    /// ```javascript
    /// await rulesEngine.addRule({
    ///   id: 'rule-1',
    ///   name: 'Check Balance',
    ///   description: 'Ensure sufficient balance before transactions',
    ///   conditions: JSON.stringify([
    ///     { GreaterThan: { field: 'balance', value: 100 } }
    ///   ]),
    ///   actions: JSON.stringify([
    ///     { Log: { level: 'Info', message: 'Balance check passed' } }
    ///   ]),
    ///   priority: 10,
    ///   enabled: true
    /// });
    /// ```
    #[napi]
    pub async fn add_rule(&self, rule: RuleJs) -> Result<()> {
        let rust_rule = rule.to_rust_rule()?;
        let mut engine = self.engine.lock().await;

        engine
            .add_rule(rust_rule)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to add rule: {}", e)))?;

        Ok(())
    }

    /// Evaluate rules against a given context.
    ///
    /// # Arguments
    ///
    /// * `context` - Execution context containing the data to evaluate
    ///
    /// # Returns
    ///
    /// Rule evaluation result
    ///
    /// # Example
    ///
    /// ```javascript
    /// const result = await rulesEngine.evaluate({
    ///   data: JSON.stringify({
    ///     balance: 500,
    ///     transaction_amount: 100,
    ///     user_id: 'user-123'
    ///   })
    /// });
    ///
    /// console.log('Result:', result.resultType);
    /// if (result.resultType === 'deny') {
    ///   console.log('Reason:', result.message);
    /// }
    /// ```
    #[napi]
    pub async fn evaluate(&self, context: ExecutionContextJs) -> Result<RuleResultJs> {
        let _rust_context = context.to_rust_context()?;

        // In a real implementation, we would evaluate all rules
        // For now, return a mock result
        Ok(RuleResultJs {
            result_type: "allow".to_string(),
            message: None,
            modifications: None,
        })
    }

    /// Evaluate a specific rule by ID.
    ///
    /// # Arguments
    ///
    /// * `rule_id` - ID of the rule to evaluate
    /// * `context` - Execution context
    ///
    /// # Returns
    ///
    /// Rule evaluation result
    ///
    /// # Example
    ///
    /// ```javascript
    /// const result = await rulesEngine.evaluateRule('rule-1', {
    ///   data: JSON.stringify({ balance: 500 })
    /// });
    /// ```
    #[napi]
    pub async fn evaluate_rule(
        &self,
        _rule_id: String,
        context: ExecutionContextJs,
    ) -> Result<RuleResultJs> {
        let _rust_context = context.to_rust_context()?;

        // In a real implementation, we would look up and evaluate the specific rule
        // For now, return a mock result
        Ok(RuleResultJs {
            result_type: "allow".to_string(),
            message: None,
            modifications: None,
        })
    }

    /// Validate a rule definition.
    ///
    /// # Arguments
    ///
    /// * `rule` - Rule to validate
    ///
    /// # Returns
    ///
    /// true if the rule is valid, throws error otherwise
    ///
    /// # Example
    ///
    /// ```javascript
    /// try {
    ///   const isValid = await rulesEngine.validateRule({
    ///     id: 'rule-1',
    ///     name: 'Test Rule',
    ///     conditions: JSON.stringify([...]),
    ///     actions: JSON.stringify([...])
    ///   });
    ///   console.log('Rule is valid');
    /// } catch (error) {
    ///   console.error('Invalid rule:', error.message);
    /// }
    /// ```
    #[napi]
    pub async fn validate_rule(&self, rule: RuleJs) -> Result<bool> {
        let rust_rule = rule.to_rust_rule()?;

        rust_rule
            .is_valid()
            .map_err(|e| Error::from_reason(format!("Rule validation failed: {}", e)))?;

        Ok(true)
    }

    /// Get the number of rules in the engine.
    ///
    /// # Returns
    ///
    /// Count of rules
    ///
    /// # Example
    ///
    /// ```javascript
    /// const count = await rulesEngine.getRuleCount();
    /// console.log('Total rules:', count);
    /// ```
    #[napi]
    pub async fn get_rule_count(&self) -> Result<u32> {
        // In a real implementation, this would return the actual count
        Ok(0)
    }
}
