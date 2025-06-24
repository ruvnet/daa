//! MCP resources implementation for QuDAG data access

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub mod dag;
pub mod exchange;
pub mod network;
pub mod system;
pub mod vault;

pub use dag::DagStateResource;
pub use exchange::ExchangeResource;
pub use network::NetworkPeersResource;
pub use system::SystemStatusResource;
pub use vault::VaultEntriesResource;

use crate::{
    error::{Error, Result},
    types::{Resource, ResourceContent, ResourceURI},
};
use std::sync::Arc;

/// Resource registry for managing available resources
#[derive(Clone)]
pub struct ResourceRegistry {
    resources: HashMap<String, Arc<dyn McpResource>>,
}

impl ResourceRegistry {
    /// Create new resource registry
    pub fn new() -> Self {
        let mut registry = Self {
            resources: HashMap::new(),
        };

        // Register built-in resources
        registry.register("vault", Arc::new(VaultEntriesResource::new()));
        registry.register("dag", Arc::new(DagStateResource::new()));
        registry.register("network", Arc::new(NetworkPeersResource::new()));
        registry.register("system", Arc::new(SystemStatusResource::new()));
        registry.register("exchange", Arc::new(ExchangeResource::new()));

        registry
    }

    /// Register a resource
    pub fn register(&mut self, name: &str, resource: Arc<dyn McpResource>) {
        self.resources.insert(name.to_string(), resource);
    }

    /// Get all available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>> {
        let mut resources = Vec::new();
        for (name, resource) in &self.resources {
            let definition = resource.definition();
            resources.push(Resource {
                uri: definition.uri,
                name: definition.name,
                description: definition.description,
                mime_type: definition.mime_type,
            });
        }
        Ok(resources)
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: &ResourceURI) -> Result<Vec<ResourceContent>> {
        let uri_str = uri.as_str();

        // Parse resource type from URI
        let resource_type = if uri_str.starts_with("vault://") {
            "vault"
        } else if uri_str.starts_with("dag://") {
            "dag"
        } else if uri_str.starts_with("network://") {
            "network"
        } else if uri_str.starts_with("exchange://") {
            "exchange"
        } else if uri_str.starts_with("crypto://") {
            "crypto"
        } else {
            return Err(Error::resource("unknown", "Unknown resource type"));
        };

        if let Some(resource) = self.resources.get(resource_type) {
            resource.read(uri).await
        } else {
            Err(Error::resource(
                resource_type,
                "Resource provider not found",
            ))
        }
    }

    /// Subscribe to resource changes
    pub async fn subscribe_to_resource(&self, uri: &ResourceURI) -> Result<()> {
        let uri_str = uri.as_str();

        // Parse resource type from URI
        let resource_type = if uri_str.starts_with("vault://") {
            "vault"
        } else if uri_str.starts_with("dag://") {
            "dag"
        } else if uri_str.starts_with("network://") {
            "network"
        } else if uri_str.starts_with("exchange://") {
            "exchange"
        } else if uri_str.starts_with("crypto://") {
            "crypto"
        } else {
            return Err(Error::resource("unknown", "Unknown resource type"));
        };

        if let Some(resource) = self.resources.get(resource_type) {
            if resource.supports_subscriptions() {
                // TODO: Implement actual subscription mechanism
                Ok(())
            } else {
                Err(Error::resource(
                    resource_type,
                    "Resource does not support subscriptions",
                ))
            }
        } else {
            Err(Error::resource(
                resource_type,
                "Resource provider not found",
            ))
        }
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for MCP resources
#[async_trait]
pub trait McpResource {
    /// Get the resource URI
    fn uri(&self) -> &str;

    /// Get the resource name
    fn name(&self) -> &str;

    /// Get the resource description
    fn description(&self) -> Option<&str>;

    /// Get the resource MIME type
    fn mime_type(&self) -> Option<&str>;

    /// Get resource definition
    fn definition(&self) -> Resource;

    /// Read the resource contents
    async fn read(&self, uri: &ResourceURI) -> Result<Vec<ResourceContent>>;

    /// Check if resource supports subscriptions
    fn supports_subscriptions(&self) -> bool {
        false
    }

    /// Get resource metadata
    fn metadata(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}
