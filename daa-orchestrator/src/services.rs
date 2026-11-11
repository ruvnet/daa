//! Service registry

use crate::{Result, ServiceConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub endpoint: String,
}

/// In-memory registry of services, keyed by service id.
pub struct ServiceRegistry {
    config: ServiceConfig,
    services: RwLock<HashMap<String, Service>>,
}

impl ServiceRegistry {
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Registry configuration
    pub fn config(&self) -> &ServiceConfig {
        &self.config
    }

    pub async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Register a service. Re-registering an existing id replaces the entry.
    pub async fn register(&self, service: Service) -> Result<()> {
        self.services
            .write()
            .await
            .insert(service.id.clone(), service);
        Ok(())
    }

    /// Return all registered services of the given type.
    pub async fn discover(&self, service_type: &str) -> Result<Vec<Service>> {
        Ok(self
            .services
            .read()
            .await
            .values()
            .filter(|service| service.service_type == service_type)
            .cloned()
            .collect())
    }

    pub async fn get_service_count(&self) -> u64 {
        self.services.read().await.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(id: &str, service_type: &str) -> Service {
        Service {
            id: id.to_string(),
            name: format!("{id} service"),
            service_type: service_type.to_string(),
            endpoint: "localhost:0".to_string(),
        }
    }

    #[tokio::test]
    async fn register_and_discover_by_type() {
        let registry = ServiceRegistry::new(ServiceConfig::default());
        registry.register(service("a", "ai_agent")).await.unwrap();
        registry.register(service("b", "ai_agent")).await.unwrap();
        registry
            .register(service("c", "rules_engine"))
            .await
            .unwrap();
        // Same id replaces rather than duplicates.
        registry
            .register(service("c", "rules_engine"))
            .await
            .unwrap();

        assert_eq!(registry.get_service_count().await, 3);
        assert_eq!(registry.discover("ai_agent").await.unwrap().len(), 2);
        assert_eq!(registry.discover("rules_engine").await.unwrap().len(), 1);
        assert!(registry.discover("unknown").await.unwrap().is_empty());
    }
}
