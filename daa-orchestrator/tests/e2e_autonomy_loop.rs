//! End-to-end tests for DAA orchestrator basic functionality

use daa_orchestrator::{DaaOrchestrator, NodeConfig, OrchestratorConfig};

/// Test basic orchestrator creation and initialization
#[tokio::test]
async fn test_orchestrator_basic_lifecycle() {
    let node_config = NodeConfig::default();
    let config = OrchestratorConfig {
        node: node_config,
        ..OrchestratorConfig::default()
    };
    let orchestrator = DaaOrchestrator::new(config).await.unwrap();

    // Test that we can get statistics
    let stats = orchestrator.get_statistics().await;
    assert_eq!(stats.active_workflows, 0);
    assert_eq!(stats.registered_services, 0);
    assert_eq!(stats.coordinated_operations, 0);
    assert_eq!(stats.processed_events, 0);
    assert!(!stats.node_id.is_empty());
}

/// Test orchestrator configuration validation
#[tokio::test]
async fn test_orchestrator_configuration() {
    let node_config = NodeConfig::default();
    // Should be able to create orchestrator with default config
    let config = OrchestratorConfig {
        node: node_config,
        ..OrchestratorConfig::default()
    };
    let orchestrator = DaaOrchestrator::new(config).await;
    assert!(orchestrator.is_ok());
}
