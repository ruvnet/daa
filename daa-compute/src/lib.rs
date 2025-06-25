//! DAA Compute - Decentralized AI Training with P2P Communication
//!
//! This crate provides a peer-to-peer communication layer for distributed
//! AI training, supporting browser nodes via WASM and efficient gradient
//! sharing across a global network.

pub mod p2p;

#[cfg(target_arch = "wasm32")]
pub mod wasm_training;

#[cfg(target_arch = "wasm32")] 
pub mod wasm_inference;

pub use p2p::{
    NetworkBehavior, P2PNetwork, SwarmConfig,
    gradient::{AllReduce, GradientMessage},
    transport::{TransportConfig, create_transport},
};

// WASM exports
#[cfg(target_arch = "wasm32")]
pub use wasm_training::{BrowserTrainer, BrowserTrainingConfig};

#[cfg(target_arch = "wasm32")]
pub use wasm_inference::{BrowserInference, InferenceConfig};

// Re-export common types
pub use libp2p::{PeerId, Multiaddr};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}