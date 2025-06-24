//! Service registry

use serde::{Deserialize, Serialize};
use crate::{Result, ServiceConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub endpoint: String,
}

pub struct ServiceRegistry {
    config: ServiceConfig,
}

impl ServiceRegistry {
    pub fn new(config: ServiceConfig) -> Self {
        Self { config }
    }

    pub async fn start(&mut self) -> Result<()> { Ok(()) }
    
    pub async fn register(&mut self, _service: Service) -> Result<()> { Ok(()) }
    
    pub async fn discover(&self, _service_type: &str) -> Result<Vec<Service>> { Ok(vec![]) }
    
    pub async fn get_service_count(&self) -> u64 { 0 }
}