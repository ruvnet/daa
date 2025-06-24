//! AI model abstractions and implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AiError, Result};

/// AI model types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    LanguageModel,
    ClassificationModel,
    RegressionModel,
    ReinforcementLearning,
    TimeSeriesForecasting,
    AnomalyDetection,
    RecommendationEngine,
}

/// Model response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub model_id: String,
    pub output: serde_json::Value,
    pub confidence: f64,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ModelResponse {
    pub fn new(model_id: String, output: serde_json::Value, confidence: f64) -> Self {
        Self {
            model_id,
            output,
            confidence,
            processing_time_ms: 0,
            metadata: HashMap::new(),
        }
    }
}

/// AI model trait
#[async_trait]
pub trait AiModel: Send + Sync {
    /// Get model ID
    fn id(&self) -> &str;
    
    /// Get model type
    fn model_type(&self) -> ModelType;
    
    /// Process input and return prediction/response
    async fn process(&self, input: &serde_json::Value) -> Result<ModelResponse>;
    
    /// Check if model is ready
    async fn is_ready(&self) -> bool;
    
    /// Get model metadata
    fn get_metadata(&self) -> HashMap<String, serde_json::Value>;
}

/// Mock AI model for testing
pub struct MockAiModel {
    id: String,
    model_type: ModelType,
    metadata: HashMap<String, serde_json::Value>,
}

impl MockAiModel {
    pub fn new(id: String, model_type: ModelType) -> Self {
        Self {
            id,
            model_type,
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
impl AiModel for MockAiModel {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn model_type(&self) -> ModelType {
        self.model_type.clone()
    }
    
    async fn process(&self, input: &serde_json::Value) -> Result<ModelResponse> {
        // Mock response
        let output = serde_json::json!({
            "prediction": "mock_result",
            "input": input
        });
        
        Ok(ModelResponse::new(
            self.id.clone(),
            output,
            0.85, // Mock confidence
        ))
    }
    
    async fn is_ready(&self) -> bool {
        true
    }
    
    fn get_metadata(&self) -> HashMap<String, serde_json::Value> {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_ai_model() {
        let model = MockAiModel::new(
            "test_model".to_string(),
            ModelType::LanguageModel,
        );

        assert_eq!(model.id(), "test_model");
        assert_eq!(model.model_type(), ModelType::LanguageModel);
        assert!(model.is_ready().await);

        let input = serde_json::json!({"text": "test input"});
        let response = model.process(&input).await.unwrap();
        assert_eq!(response.model_id, "test_model");
        assert!(response.confidence > 0.0);
    }
}