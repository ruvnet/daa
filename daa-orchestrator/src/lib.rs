//! # DAA Orchestrator
//!
//! Orchestration layer for the Decentralized Autonomous Agents (DAA) system.
//! Coordinates all DAA components using QuDAG protocol Node for distributed operations.

mod qudag_stubs;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use anyhow;
use hex;

// Re-export QuDAG protocol types
pub use crate::qudag_stubs::qudag_protocol::{Node, NodeConfig, Message};

pub mod coordinator;
pub mod workflow;
pub mod services;
pub mod events;

#[cfg(feature = "chain-integration")]
pub mod chain_integration;

#[cfg(feature = "economy-integration")]
pub mod economy_integration;

#[cfg(feature = "rules-integration")]
pub mod rules_integration;

#[cfg(feature = "ai-integration")]
pub mod ai_integration;

/// Orchestrator error types
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] crate::qudag_stubs::ProtocolError),
    
    #[error("Message error: {0}")]
    Message(#[from] crate::qudag_stubs::MessageError),
    
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
    
    #[error("Service error: {0}")]
    Service(String),
    
    #[error("Workflow error: {0}")]
    Workflow(String),
    
    #[error("Coordination error: {0}")]
    Coordination(String),
    
    #[error("Integration error: {0}")]
    Integration(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Configuration for the DAA orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Node configuration for QuDAG protocol
    pub node: NodeConfig,
    
    /// Coordination settings
    pub coordination: CoordinationConfig,
    
    /// Service registry configuration
    pub services: ServiceConfig,
    
    /// Workflow engine configuration
    pub workflows: WorkflowConfig,
    
    /// Integration configurations
    pub integrations: IntegrationConfig,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            node: NodeConfig::default(),
            coordination: CoordinationConfig::default(),
            services: ServiceConfig::default(),
            workflows: WorkflowConfig::default(),
            integrations: IntegrationConfig::default(),
        }
    }
}

/// Coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    
    /// Operation timeout in seconds
    pub operation_timeout: u64,
    
    /// Retry configuration
    pub retry_attempts: u32,
    
    /// Leader election timeout
    pub leader_election_timeout: u64,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 100,
            operation_timeout: 300, // 5 minutes
            retry_attempts: 3,
            leader_election_timeout: 30,
        }
    }
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Auto-discovery enabled
    pub auto_discovery: bool,
    
    /// Service health check interval
    pub health_check_interval: u64,
    
    /// Service registration TTL
    pub registration_ttl: u64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            health_check_interval: 30, // 30 seconds
            registration_ttl: 300, // 5 minutes
        }
    }
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Maximum workflow execution time
    pub max_execution_time: u64,
    
    /// Maximum steps per workflow
    pub max_steps: usize,
    
    /// Parallel execution enabled
    pub parallel_execution: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_execution_time: 3600, // 1 hour
            max_steps: 100,
            parallel_execution: true,
        }
    }
}

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Enable chain integration
    pub enable_chain: bool,
    
    /// Enable economy integration
    pub enable_economy: bool,
    
    /// Enable rules integration
    pub enable_rules: bool,
    
    /// Enable AI integration
    pub enable_ai: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enable_chain: true,
            enable_economy: true,
            enable_rules: true,
            enable_ai: true,
        }
    }
}

/// Main orchestrator coordinating all DAA operations
pub struct DaaOrchestrator {
    /// System configuration
    config: OrchestratorConfig,
    
    /// QuDAG protocol node
    node: Node,
    
    /// Coordination manager
    coordinator: coordinator::Coordinator,
    
    /// Workflow engine
    workflow_engine: workflow::WorkflowEngine,
    
    /// Service registry
    service_registry: services::ServiceRegistry,
    
    /// Event manager
    event_manager: events::EventManager,
    
    /// Integration managers
    #[cfg(feature = "chain-integration")]
    chain_integration: Option<chain_integration::ChainIntegration>,
    
    #[cfg(feature = "economy-integration")]
    economy_integration: Option<economy_integration::EconomyIntegration>,
    
    #[cfg(feature = "rules-integration")]
    rules_integration: Option<rules_integration::RulesIntegration>,
    
    #[cfg(feature = "ai-integration")]
    ai_integration: Option<ai_integration::AIIntegration>,
}

impl DaaOrchestrator {
    /// Create a new orchestrator
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        // Initialize QuDAG node
        let node = Node::new(config.node.clone()).await?;
        
        // Initialize managers
        let coordinator = coordinator::Coordinator::new(config.coordination.clone());
        let workflow_engine = workflow::WorkflowEngine::new(config.workflows.clone());
        let service_registry = services::ServiceRegistry::new(config.services.clone());
        let event_manager = events::EventManager::new();
        
        // Initialize integrations
        #[cfg(feature = "chain-integration")]
        let chain_integration = if config.integrations.enable_chain {
            Some(chain_integration::ChainIntegration::new().await?)
        } else {
            None
        };
        
        #[cfg(feature = "economy-integration")]
        let economy_integration = if config.integrations.enable_economy {
            Some(economy_integration::EconomyIntegration::new().await?)
        } else {
            None
        };
        
        #[cfg(feature = "rules-integration")]
        let rules_integration = if config.integrations.enable_rules {
            Some(rules_integration::RulesIntegration::new().await?)
        } else {
            None
        };
        
        #[cfg(feature = "ai-integration")]
        let ai_integration = if config.integrations.enable_ai {
            Some(ai_integration::AIIntegration::new().await?)
        } else {
            None
        };

        Ok(Self {
            config,
            node,
            coordinator,
            workflow_engine,
            service_registry,
            event_manager,
            #[cfg(feature = "chain-integration")]
            chain_integration,
            #[cfg(feature = "economy-integration")]
            economy_integration,
            #[cfg(feature = "rules-integration")]
            rules_integration,
            #[cfg(feature = "ai-integration")]
            ai_integration,
        })
    }

    /// Initialize the orchestrator
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing DAA Orchestrator");
        
        // Start QuDAG node
        self.node.start().await?;
        
        // Initialize coordinator
        self.coordinator.initialize().await?;
        
        // Start workflow engine
        self.workflow_engine.start().await?;
        
        // Start service registry
        self.service_registry.start().await?;
        
        // Initialize event manager
        self.event_manager.initialize().await?;
        
        // Initialize integrations
        #[cfg(feature = "chain-integration")]
        if let Some(ref mut integration) = self.chain_integration {
            integration.initialize().await?;
        }
        
        #[cfg(feature = "economy-integration")]
        if let Some(ref mut integration) = self.economy_integration {
            integration.initialize().await?;
        }
        
        #[cfg(feature = "rules-integration")]
        if let Some(ref mut integration) = self.rules_integration {
            integration.initialize().await?;
        }
        
        #[cfg(feature = "ai-integration")]
        if let Some(ref mut integration) = self.ai_integration {
            integration.initialize().await?;
        }
        
        tracing::info!("DAA Orchestrator initialized successfully");
        Ok(())
    }

    /// Execute a workflow
    pub async fn execute_workflow(
        &mut self,
        workflow: workflow::Workflow,
    ) -> Result<workflow::WorkflowResult> {
        tracing::info!("Executing workflow: {}", workflow.id);
        
        // Coordinate workflow execution
        let execution_id = self.coordinator.coordinate_workflow(&workflow).await?;
        
        // Execute through workflow engine
        let result = self.workflow_engine.execute(workflow).await?;
        
        // Publish completion event
        self.event_manager.publish_event(events::Event::WorkflowCompleted {
            execution_id,
            result: result.clone(),
        }).await?;
        
        Ok(result)
    }

    /// Register a service
    pub async fn register_service(&mut self, service: services::Service) -> Result<()> {
        self.service_registry.register(service).await
    }

    /// Discover services
    pub async fn discover_services(&self, service_type: &str) -> Result<Vec<services::Service>> {
        self.service_registry.discover(service_type).await
    }

    /// Send protocol message
    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        self.node.handle_message(message).await?;
        Ok(())
    }

    /// Get orchestrator statistics
    pub async fn get_statistics(&self) -> OrchestratorStatistics {
        OrchestratorStatistics {
            active_workflows: self.workflow_engine.get_active_count().await,
            registered_services: self.service_registry.get_service_count().await,
            coordinated_operations: self.coordinator.get_operation_count().await,
            processed_events: self.event_manager.get_event_count().await,
            node_id: hex::encode(&self.node.node_id),
        }
    }
}

/// Orchestrator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatistics {
    /// Number of active workflows
    pub active_workflows: u64,
    
    /// Number of registered services
    pub registered_services: u64,
    
    /// Number of coordinated operations
    pub coordinated_operations: u64,
    
    /// Number of processed events
    pub processed_events: u64,
    
    /// Node identifier
    pub node_id: String,
}

impl std::fmt::Display for OrchestratorStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Orchestrator Stats: Workflows={}, Services={}, Operations={}, Events={}, Node={}",
            self.active_workflows,
            self.registered_services,
            self.coordinated_operations,
            self.processed_events,
            self.node_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.coordination.max_concurrent_operations, 100);
        assert_eq!(config.coordination.operation_timeout, 300);
        assert!(config.services.auto_discovery);
        assert!(config.workflows.parallel_execution);
    }

    #[test]
    fn test_statistics_display() {
        let stats = OrchestratorStatistics {
            active_workflows: 5,
            registered_services: 10,
            coordinated_operations: 100,
            processed_events: 500,
            node_id: "test-node".to_string(),
        };
        
        let display = stats.to_string();
        assert!(display.contains("Workflows=5"));
        assert!(display.contains("Services=10"));
        assert!(display.contains("Node=test-node"));
    }
}