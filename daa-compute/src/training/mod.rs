pub mod local_trainer;
pub mod optimizer;
pub mod strategy;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    pub weights: Vec<f32>,
    pub version: u64,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub values: Vec<f32>,
    pub node_id: String,
    pub round: u64,
    pub compressed: bool,
}

#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    pub loss: f32,
    pub accuracy: f32,
    pub gradients_norm: f32,
    pub communication_bytes: u64,
}

pub trait ModelInterface: Send + Sync {
    fn forward(&self, input: &[f32]) -> Vec<f32>;
    fn backward(&mut self, loss: f32) -> Gradient;
    fn apply_gradient(&mut self, gradient: &Gradient);
    fn get_parameters(&self) -> ModelParameters;
    fn set_parameters(&mut self, params: ModelParameters);
}