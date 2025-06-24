//! Decision making engine and context

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AiError, Result};

/// Decision context containing relevant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub context_id: String,
    pub decision_type: String,
    pub data: HashMap<String, serde_json::Value>,
    pub constraints: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

impl DecisionContext {
    pub fn new(context_id: String, decision_type: String) -> Self {
        Self {
            context_id,
            decision_type,
            data: HashMap::new(),
            constraints: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn add_data(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    pub fn add_constraint(&mut self, constraint: String) {
        self.constraints.push(constraint);
    }
}

/// Decision made by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub decision_id: String,
    pub context_id: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub reasoning: String,
    pub timestamp: DateTime<Utc>,
}

impl Decision {
    pub fn new(decision_id: String, context_id: String, action: String) -> Self {
        Self {
            decision_id,
            context_id,
            action,
            parameters: HashMap::new(),
            confidence: 0.5,
            reasoning: String::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_reasoning(mut self, reasoning: String) -> Self {
        self.reasoning = reasoning;
        self
    }
}

/// Decision making engine
pub struct DecisionEngine {
    engine_id: String,
    initialized: bool,
}

impl DecisionEngine {
    pub fn new() -> Self {
        Self {
            engine_id: "default".to_string(),
            initialized: true,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub async fn make_decision(&self, context: &DecisionContext) -> Result<Decision> {
        // Mock decision making
        let decision = Decision::new(
            format!("decision_{}", chrono::Utc::now().timestamp()),
            context.context_id.clone(),
            "mock_action".to_string(),
        ).with_confidence(0.8)
         .with_reasoning("Mock decision based on context".to_string());

        Ok(decision)
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_context() {
        let mut context = DecisionContext::new(
            "ctx_1".to_string(),
            "trading".to_string(),
        );

        context.add_data("price".to_string(), serde_json::json!(100.0));
        context.add_constraint("max_risk".to_string());

        assert_eq!(context.data.len(), 1);
        assert_eq!(context.constraints.len(), 1);
    }

    #[tokio::test]
    async fn test_decision_engine() {
        let engine = DecisionEngine::new();
        assert!(engine.is_initialized());

        let context = DecisionContext::new(
            "ctx_1".to_string(),
            "test".to_string(),
        );

        let decision = engine.make_decision(&context).await.unwrap();
        assert_eq!(decision.context_id, "ctx_1");
        assert!(decision.confidence > 0.0);
    }
}