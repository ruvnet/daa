//! End-to-end tests for DAA orchestrator initialization

use daa_orchestrator::{
    DaaOrchestrator, OrchestratorConfig, AutonomyConfig, QuDAGConfig, McpConfig, 
    ApiConfig, LoggingConfig, HealthCheckConfig, RulesConfig, AiConfig, ExchangeConfig
};
use std::time::Duration;
use tokio::time::timeout;

/// Test basic orchestrator creation and initialization
#[tokio::test]
async fn test_orchestrator_basic_initialization() {
    let config = OrchestratorConfig::default();
    
    // Test orchestrator creation
    let orchestrator = DaaOrchestrator::new(config).await;
    assert!(orchestrator.is_ok(), "Failed to create orchestrator: {:?}", orchestrator.err());
    
    let mut orchestrator = orchestrator.unwrap();
    
    // Test initialization
    let result = orchestrator.initialize().await;
    assert!(result.is_ok(), "Failed to initialize orchestrator: {:?}", result.err());
}

/// Test orchestrator initialization with custom configuration
#[tokio::test]
async fn test_orchestrator_custom_config_initialization() {
    let config = OrchestratorConfig {
        name: "test-orchestrator".to_string(),
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 500,
            max_tasks_per_iteration: 5,
            task_timeout_ms: 15000,
            enable_learning: true,
            rules_config: RulesConfig {
                enabled: true,
                fail_fast: true,
                max_daily_spending: 5000.0,
                min_balance_threshold: 50.0,
                max_risk_score: 0.7,
            },
            ai_config: AiConfig {
                enabled: true,
                max_agents: 3,
                agent_queue_size: 50,
                learning_retention_days: 15,
            },
        },
        qudag: QuDAGConfig {
            enabled: true,
            node_endpoint: "localhost:7500".to_string(),
            network_id: "test-network".to_string(),
            node_id: "test-node-001".to_string(),
            bootstrap_peers: vec!["localhost:7501".to_string()],
            connection_timeout_ms: 5000,
            max_reconnection_attempts: 3,
            participate_in_consensus: true,
            exchange_config: ExchangeConfig {
                enabled: true,
                endpoint: "localhost:8081".to_string(),
                trading_pairs: vec!["rUv/USD".to_string(), "rUv/BTC".to_string()],
                order_book_depth: 10,
            },
        },
        mcp: McpConfig {
            enabled: true,
            bind_address: "127.0.0.1".to_string(),
            port: 3002,
            max_connections: 50,
            request_timeout_ms: 15000,
            enable_auth: true,
            api_key: Some("test-api-key".to_string()),
        },
        api: ApiConfig {
            enabled: true,
            bind_address: "127.0.0.1".to_string(),
            port: 3001,
            max_connections: 50,
            request_timeout_ms: 15000,
            enable_cors: false,
            cors_origins: vec!["http://localhost:3000".to_string()],
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
            stdout: true,
            file_path: Some("/tmp/test-orchestrator.log".to_string()),
            structured: true,
        },
        health_check: HealthCheckConfig {
            interval_seconds: 15,
            component_timeout_ms: 3000,
            auto_restart: false,
            max_restart_attempts: 2,
        },
    };
    
    // Validate configuration
    assert!(config.validate().is_ok(), "Custom configuration is invalid");
    
    // Test orchestrator creation with custom config
    let orchestrator = DaaOrchestrator::new(config).await;
    assert!(orchestrator.is_ok(), "Failed to create orchestrator with custom config");
    
    let mut orchestrator = orchestrator.unwrap();
    
    // Test initialization
    let result = orchestrator.initialize().await;
    assert!(result.is_ok(), "Failed to initialize orchestrator with custom config");
}

/// Test orchestrator initialization with timeout
#[tokio::test]
async fn test_orchestrator_initialization_timeout() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    
    // Test initialization with timeout
    let result = timeout(Duration::from_secs(10), orchestrator.initialize()).await;
    assert!(result.is_ok(), "Initialization timed out");
    assert!(result.unwrap().is_ok(), "Initialization failed");
}

/// Test orchestrator statistics collection
#[tokio::test]
async fn test_orchestrator_statistics() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Get statistics
    let stats = orchestrator.get_statistics().await;
    
    // Verify statistics structure
    assert_eq!(stats.active_workflows, 0);
    assert_eq!(stats.registered_services, 0);
    assert_eq!(stats.coordinated_operations, 0);
    assert_eq!(stats.processed_events, 0);
    assert!(!stats.node_id.is_empty());
    
    // Verify statistics display
    let stats_string = stats.to_string();
    assert!(stats_string.contains("Workflows=0"));
    assert!(stats_string.contains("Services=0"));
    assert!(stats_string.contains("Operations=0"));
    assert!(stats_string.contains("Events=0"));
}

/// Test orchestrator component initialization order
#[tokio::test]
async fn test_component_initialization_order() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    
    // Initialize and verify no errors
    let result = orchestrator.initialize().await;
    assert!(result.is_ok(), "Component initialization failed");
    
    // Verify statistics after initialization
    let stats = orchestrator.get_statistics().await;
    assert!(!stats.node_id.is_empty(), "Node ID should be set after initialization");
}

/// Test orchestrator with disabled integrations
#[tokio::test]
async fn test_orchestrator_disabled_integrations() {
    let mut config = OrchestratorConfig::default();
    
    // Disable all integrations
    config.autonomy.enabled = false;
    config.qudag.enabled = false;
    config.mcp.enabled = false;
    config.api.enabled = false;
    config.autonomy.rules_config.enabled = false;
    config.autonomy.ai_config.enabled = false;
    config.qudag.exchange_config.enabled = false;
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    
    // Should still initialize successfully
    let result = orchestrator.initialize().await;
    assert!(result.is_ok(), "Failed to initialize with disabled integrations");
    
    // Statistics should still work
    let stats = orchestrator.get_statistics().await;
    assert!(!stats.node_id.is_empty());
}

/// Test multiple orchestrator instances
#[tokio::test]
async fn test_multiple_orchestrator_instances() {
    let config1 = OrchestratorConfig {
        name: "orchestrator-1".to_string(),
        mcp: McpConfig {
            port: 3010,
            ..Default::default()
        },
        api: ApiConfig {
            port: 3011,
            ..Default::default()
        },
        qudag: QuDAGConfig {
            node_endpoint: "localhost:7010".to_string(),
            node_id: "node-1".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let config2 = OrchestratorConfig {
        name: "orchestrator-2".to_string(),
        mcp: McpConfig {
            port: 3020,
            ..Default::default()
        },
        api: ApiConfig {
            port: 3021,
            ..Default::default()
        },
        qudag: QuDAGConfig {
            node_endpoint: "localhost:7020".to_string(),
            node_id: "node-2".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    
    // Create both orchestrators
    let mut orchestrator1 = DaaOrchestrator::new(config1).await.unwrap();
    let mut orchestrator2 = DaaOrchestrator::new(config2).await.unwrap();
    
    // Initialize both
    let result1 = orchestrator1.initialize().await;
    let result2 = orchestrator2.initialize().await;
    
    assert!(result1.is_ok(), "Failed to initialize first orchestrator");
    assert!(result2.is_ok(), "Failed to initialize second orchestrator");
    
    // Verify different node IDs
    let stats1 = orchestrator1.get_statistics().await;
    let stats2 = orchestrator2.get_statistics().await;
    
    assert_ne!(stats1.node_id, stats2.node_id, "Orchestrators should have different node IDs");
}

/// Integration test for orchestrator lifecycle
#[tokio::test]
async fn test_orchestrator_full_lifecycle() {
    let config = OrchestratorConfig::default();
    
    // Create orchestrator
    let orchestrator_result = DaaOrchestrator::new(config).await;
    assert!(orchestrator_result.is_ok(), "Failed to create orchestrator");
    
    let mut orchestrator = orchestrator_result.unwrap();
    
    // Initialize orchestrator
    let init_result = orchestrator.initialize().await;
    assert!(init_result.is_ok(), "Failed to initialize orchestrator");
    
    // Get initial statistics
    let initial_stats = orchestrator.get_statistics().await;
    assert_eq!(initial_stats.active_workflows, 0);
    assert_eq!(initial_stats.registered_services, 0);
    
    // Test service registration
    let service = daa_orchestrator::services::Service {
        id: "test-service-1".to_string(),
        name: "Test Service".to_string(),
        service_type: "ai_agent".to_string(),
        endpoint: "localhost:9000".to_string(),
    };
    
    let register_result = orchestrator.register_service(service).await;
    assert!(register_result.is_ok(), "Failed to register service");
    
    // Test service discovery
    let discovery_result = orchestrator.discover_services("ai_agent").await;
    assert!(discovery_result.is_ok(), "Failed to discover services");
    
    // Final statistics check
    let final_stats = orchestrator.get_statistics().await;
    assert!(!final_stats.node_id.is_empty(), "Node ID should be present");
    
    println!("Orchestrator lifecycle test completed successfully");
    println!("Final statistics: {}", final_stats);
}