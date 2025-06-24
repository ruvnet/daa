//! QuDAG WASM bindings
//!
//! This module provides WebAssembly bindings for the QuDAG protocol,
//! enabling browser and Node.js applications to interact with the
//! quantum-resistant DAG network.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Core modules
pub mod error;
pub mod utils;

// WASM crypto implementation
#[cfg(target_arch = "wasm32")]
pub mod wasm_crypto;

// Export crypto module for WASM
#[cfg(target_arch = "wasm32")]
pub mod crypto {
    pub use crate::wasm_crypto::*;
}

// Crypto abstraction layer (disabled for initial build)
// pub mod crypto_abstraction;
// pub mod crypto_traits;
// pub mod crypto_unified;

// Conditional modules based on features and target
#[cfg(all(feature = "dag", not(target_arch = "wasm32")))]
pub mod dag;

#[cfg(all(feature = "dag", target_arch = "wasm32"))]
pub mod dag_stubs;

#[cfg(all(feature = "dag", target_arch = "wasm32"))]
pub use dag_stubs as dag;

// Network modules (only available for non-WASM targets with full feature)
#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
pub mod network;

#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
pub mod address;

// WASM stubs for network functionality
#[cfg(all(any(feature = "full", feature = "dag"), target_arch = "wasm32"))]
pub mod network_stubs;

#[cfg(all(any(feature = "full", feature = "dag"), target_arch = "wasm32"))]
pub use network_stubs as network;

#[cfg(all(any(feature = "full", feature = "dag"), target_arch = "wasm32"))]
pub mod address_stubs;

#[cfg(all(any(feature = "full", feature = "dag"), target_arch = "wasm32"))]
pub use address_stubs as address;

// Optional vault module
#[cfg(feature = "vault")]
pub mod vault;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages in the browser
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Log initialization
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"QuDAG WASM initialized".into());
}

/// QuDAG WASM client for interacting with the protocol
#[wasm_bindgen]
pub struct QuDAGClient {
    config: ClientConfig,
}

#[derive(Serialize, Deserialize, Clone)]
struct ClientConfig {
    network_port: u16,
    max_peers: usize,
    enable_vault: bool,
    features: Vec<String>,
}

#[wasm_bindgen]
impl QuDAGClient {
    /// Create a new QuDAG client
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let features = vec!["crypto".to_string()];

        #[cfg(feature = "dag")]
        let features = {
            let mut f = features;
            f.push("dag".to_string());
            f
        };

        #[cfg(feature = "full")]
        let features = {
            let mut f = features;
            f.push("full".to_string());
            f
        };

        #[cfg(feature = "vault")]
        let features = {
            let mut f = features;
            f.push("vault".to_string());
            f
        };

        Self {
            config: ClientConfig {
                network_port: 8000,
                max_peers: 50,
                enable_vault: cfg!(feature = "vault"),
                features,
            },
        }
    }

    /// Create a new QuDAG client with custom configuration
    #[wasm_bindgen(js_name = "newWithConfig")]
    pub fn new_with_config(config: JsValue) -> Result<QuDAGClient, JsError> {
        let mut config: ClientConfig = serde_wasm_bindgen::from_value(config)?;

        // Update features based on compile-time flags
        let features = vec!["crypto".to_string()];
        #[cfg(feature = "dag")]
        let features = {
            let mut f = features;
            f.push("dag".to_string());
            f
        };
        #[cfg(feature = "full")]
        let features = {
            let mut f = features;
            f.push("full".to_string());
            f
        };
        #[cfg(feature = "vault")]
        let features = {
            let mut f = features;
            f.push("vault".to_string());
            f
        };

        config.features = features;
        Ok(Self { config })
    }

    /// Get the current configuration
    #[wasm_bindgen(js_name = "getConfig")]
    pub fn get_config(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&self.config)?)
    }

    /// Get library version
    #[wasm_bindgen(js_name = "getVersion")]
    pub fn get_version() -> String {
        String::from(env!("CARGO_PKG_VERSION"))
    }

    /// Check if a feature is enabled
    #[wasm_bindgen(js_name = "hasFeature")]
    pub fn has_feature(feature: &str) -> bool {
        match feature {
            "crypto" => true, // Always available
            "dag" => cfg!(feature = "dag"),
            "full" => cfg!(feature = "full"),
            "vault" => cfg!(feature = "vault"),
            "wee_alloc" => cfg!(feature = "wee_alloc"),
            "network" => cfg!(all(feature = "full", not(target_arch = "wasm32"))),
            _ => false,
        }
    }

    /// Get list of enabled features
    #[wasm_bindgen(js_name = "getFeatures")]
    pub fn get_features(&self) -> Vec<String> {
        self.config.features.clone()
    }
}

/// Error type for WASM operations
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct QuDAGError {
    message: String,
    code: String,
}

#[wasm_bindgen]
impl QuDAGError {
    /// Create a new error
    pub fn new(message: String, code: String) -> Self {
        Self { message, code }
    }

    /// Get error message
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get error code
    pub fn code(&self) -> String {
        self.code.clone()
    }
}

impl From<anyhow::Error> for QuDAGError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
            code: "QUDAG_ERROR".to_string(),
        }
    }
}

/// Convert QuDAGError to JsError
impl From<QuDAGError> for JsError {
    fn from(err: QuDAGError) -> Self {
        JsError::new(&format!("{}: {}", err.code, err.message))
    }
}

/// Module initialization status
#[wasm_bindgen]
pub struct InitStatus {
    initialized: bool,
    features: Vec<String>,
    version: String,
}

#[wasm_bindgen]
impl InitStatus {
    /// Check if module is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get enabled features
    pub fn features(&self) -> Vec<String> {
        self.features.clone()
    }

    /// Get version
    pub fn version(&self) -> String {
        self.version.clone()
    }
}

/// Get module initialization status
#[wasm_bindgen(js_name = "getInitStatus")]
pub fn get_init_status() -> InitStatus {
    let features = vec!["crypto".to_string()];

    #[cfg(feature = "dag")]
    let features = {
        let mut f = features;
        f.push("dag".to_string());
        f
    };

    #[cfg(feature = "full")]
    let features = {
        let mut f = features;
        f.push("full".to_string());
        f
    };

    #[cfg(feature = "vault")]
    let features = {
        let mut f = features;
        f.push("vault".to_string());
        f
    };

    #[cfg(target_arch = "wasm32")]
    let features = {
        let mut f = features;
        f.push("wasm".to_string());
        f
    };

    InitStatus {
        initialized: true,
        features,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
