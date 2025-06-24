//! Application state management

use anyhow::Result;
use std::sync::Arc;
use qudag_exchange_core::{Exchange, ExchangeConfig};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    // TODO: Add actual exchange instance when core is implemented
    // pub exchange: Arc<tokio::sync::RwLock<Exchange>>,
    pub config: Arc<ExchangeConfig>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let config = ExchangeConfig::default();
        
        // TODO: Initialize exchange instance
        // let exchange = Exchange::new(config.clone())?;
        
        Ok(Self {
            // exchange: Arc::new(tokio::sync::RwLock::new(exchange)),
            config: Arc::new(config),
        })
    }
}