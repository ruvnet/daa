//! DAA Compute - Decentralized AI Training with P2P Communication
//!
//! This crate provides a peer-to-peer communication layer for distributed
//! AI training, supporting browser nodes via WASM and efficient gradient
//! sharing across a global network.

pub mod coordinator;
pub mod distributed;
pub mod mesh;
pub mod p2p;
pub mod protocols;
pub mod training;

#[cfg(target_arch = "wasm32")]
pub mod wasm_training;

#[cfg(target_arch = "wasm32")]
pub mod wasm_inference;

pub use p2p::{
    gradient::{AllReduce, GradientMessage},
    transport::{create_transport, TransportConfig},
    NetworkBehavior, P2PNetwork, SwarmConfig,
};

// WASM exports
#[cfg(target_arch = "wasm32")]
pub use wasm_training::{BrowserTrainer, BrowserTrainingConfig};

#[cfg(target_arch = "wasm32")]
pub use wasm_inference::{BrowserInference, InferenceConfig};

pub use coordinator::round::RoundCoordinator;
pub use distributed::federated::FederatedSGD;
pub use mesh::elastic::ElasticDeviceMesh;
pub use protocols::aggregation::{AggregationStrategy, GradientAggregator};
pub use training::strategy::TrainingStrategy;

// Re-export common types
pub use libp2p::{Multiaddr, PeerId};

/// Configuration for DiLoCo-style distributed training: long local training
/// phases separated by infrequent global synchronisation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiLoCoConfig {
    /// Local optimisation steps between global synchronisations.
    pub local_epochs: usize,
    /// Target communication reduction factor relative to per-step sync.
    pub communication_reduction: usize,
    /// Upper bound on time spent training locally before a forced sync.
    pub max_local_time_minutes: u64,
    /// Gradient compression level (0 disables compression).
    pub gradient_compression: u8,
    /// Whether to add differential-privacy noise to shared gradients.
    pub differential_privacy: bool,
    /// Differential-privacy budget; smaller values add more noise.
    pub dp_epsilon: f32,
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}
