//! Learning engine for continuous improvement

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AiError, Result};

/// Learning data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningData {
    pub data_id: String,
    pub input: serde_json::Value,
    pub expected_output: Option<serde_json::Value>,
    pub actual_output: Option<serde_json::Value>,
    pub feedback: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LearningData {
    pub fn new(data_id: String, input: serde_json::Value) -> Self {
        Self {
            data_id,
            input,
            expected_output: None,
            actual_output: None,
            feedback: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_expected_output(mut self, output: serde_json::Value) -> Self {
        self.expected_output = Some(output);
        self
    }

    pub fn with_actual_output(mut self, output: serde_json::Value) -> Self {
        self.actual_output = Some(output);
        self
    }

    pub fn with_feedback(mut self, feedback: f64) -> Self {
        self.feedback = Some(feedback);
        self
    }
}

/// Learning result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResult {
    pub result_id: String,
    pub improvement_score: f64,
    pub confidence_change: f64,
    pub parameters_updated: bool,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

impl LearningResult {
    pub fn new(result_id: String, improvement_score: f64) -> Self {
        Self {
            result_id,
            improvement_score,
            confidence_change: 0.0,
            parameters_updated: false,
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }
}

/// Learning engine for continuous improvement
pub struct LearningEngine {
    engine_id: String,
    learning_data: Vec<LearningData>,
    ready: bool,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            engine_id: "default_learning".to_string(),
            learning_data: Vec::new(),
            ready: true,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub async fn add_learning_data(&mut self, data: LearningData) -> Result<()> {
        self.learning_data.push(data);
        Ok(())
    }

    pub async fn learn(&mut self) -> Result<LearningResult> {
        // Mock learning process
        let improvement_score = if self.learning_data.is_empty() {
            0.0
        } else {
            0.1 // Mock improvement
        };

        let result = LearningResult::new(
            format!("learn_{}", chrono::Utc::now().timestamp()),
            improvement_score,
        );

        Ok(result)
    }

    pub fn get_learning_data_count(&self) -> usize {
        self.learning_data.len()
    }

    pub async fn clear_old_data(&mut self, keep_days: i64) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(keep_days);
        let initial_count = self.learning_data.len();
        
        self.learning_data.retain(|data| data.timestamp >= cutoff);
        
        Ok(initial_count - self.learning_data.len())
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_learning_engine() {
        let mut engine = LearningEngine::new();
        assert!(engine.is_ready());

        let data = LearningData::new(
            "data_1".to_string(),
            serde_json::json!({"input": "test"}),
        ).with_feedback(0.8);

        engine.add_learning_data(data).await.unwrap();
        assert_eq!(engine.get_learning_data_count(), 1);

        let result = engine.learn().await.unwrap();
        assert!(result.improvement_score >= 0.0);
    }

    #[tokio::test]
    async fn test_learning_data_cleanup() {
        let mut engine = LearningEngine::new();
        
        let data = LearningData::new(
            "old_data".to_string(),
            serde_json::json!({"input": "old"}),
        );

        engine.add_learning_data(data).await.unwrap();
        
        // Clean data older than 0 days (should remove all)
        let removed = engine.clear_old_data(0).await.unwrap();
        assert_eq!(removed, 1);
        assert_eq!(engine.get_learning_data_count(), 0);
    }
}