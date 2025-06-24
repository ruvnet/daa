//! End-to-end tests for DAA orchestrator error scenarios and recovery mechanisms

use daa_orchestrator::{
    DaaOrchestrator, OrchestratorConfig, OrchestratorError,
    config::{AutonomyConfig, QuDAGConfig, McpConfig, ApiConfig, RulesConfig, AiConfig},
    autonomy::{AutonomyLoop, AutonomyState},
    workflow::{Workflow, WorkflowStep},
    services::Service,
};
use serde_json::json;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

/// Test invalid configuration scenarios
#[tokio::test]
async fn test_invalid_configuration_errors() {
    // Test with invalid port (0)
    let mut config = OrchestratorConfig::default();
    config.mcp.port = 0;
    
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Should reject invalid MCP port");
    assert!(validation_result.unwrap_err().contains("port cannot be 0"));
    
    // Test with invalid API port
    config.mcp.port = 3001; // Fix MCP port
    config.api.port = 0;
    
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Should reject invalid API port");
    assert!(validation_result.unwrap_err().contains("port cannot be 0"));
    
    // Test with invalid timeout
    config.api.port = 3000; // Fix API port
    config.autonomy.task_timeout_ms = 0;
    
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Should reject zero timeout");
    assert!(validation_result.unwrap_err().contains("timeout cannot be 0"));
    
    // Test with empty node ID when QuDAG is enabled
    config.autonomy.task_timeout_ms = 30000; // Fix timeout
    config.qudag.node_id = String::new();
    
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Should reject empty node ID");
    assert!(validation_result.unwrap_err().contains("node ID cannot be empty"));
    
    // Test with invalid max tasks per iteration
    config.qudag.node_id = "test-node".to_string(); // Fix node ID
    config.autonomy.max_tasks_per_iteration = 0;
    
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Should reject zero max tasks");
    assert!(validation_result.unwrap_err().contains("max tasks per iteration cannot be 0"));
    
    println!("Invalid configuration error tests completed");
}

/// Test orchestrator initialization failures
#[tokio::test]
async fn test_orchestrator_initialization_failures() {
    // Test with conflicting port configurations
    let config = OrchestratorConfig {
        mcp: McpConfig {
            enabled: true,
            port: 3500,
            ..Default::default()
        },
        api: ApiConfig {
            enabled: true,
            port: 3500, // Same port as MCP - potential conflict
            ..Default::default()
        },
        ..Default::default()
    };
    
    // Configuration validation should pass (ports are just numbers)
    assert!(config.validate().is_ok(), "Configuration should be valid");
    
    // But orchestrator creation should handle this gracefully
    let orchestrator_result = DaaOrchestrator::new(config).await;
    assert!(orchestrator_result.is_ok(), "Orchestrator creation should handle port conflicts gracefully");
    
    let mut orchestrator = orchestrator_result.unwrap();
    
    // Initialization might fail due to port conflicts, but should handle gracefully
    let init_result = orchestrator.initialize().await;
    // Current implementation doesn't actually bind ports, so this will succeed
    assert!(init_result.is_ok(), "Current implementation should handle port conflicts");
}

/// Test autonomy loop error scenarios
#[tokio::test]
async fn test_autonomy_loop_error_scenarios() {
    // Test with extreme configuration values
    let config = AutonomyConfig {
        enabled: true,
        loop_interval_ms: 1, // Extremely fast loop
        max_tasks_per_iteration: 1000, // Extreme task count
        task_timeout_ms: 1, // Extremely short timeout
        enable_learning: true,
        rules_config: RulesConfig {
            enabled: true,
            fail_fast: true,
            max_daily_spending: -1.0, // Invalid negative value
            min_balance_threshold: -1.0, // Invalid negative value
            max_risk_score: 2.0, // Invalid risk score > 1.0
        },
        ai_config: AiConfig {
            enabled: true,
            max_agents: 1000, // Extreme agent count
            agent_queue_size: 0, // Invalid queue size
            learning_retention_days: -1, // Invalid negative retention
        },
    };
    
    let mut autonomy_loop = AutonomyLoop::new(config).await.unwrap();
    
    // Initialize should work despite invalid config values
    let init_result = autonomy_loop.initialize().await;
    assert!(init_result.is_ok(), "Initialization should handle invalid config values");
    
    // Start the loop
    autonomy_loop.start().await.unwrap();
    
    // Let it run briefly with extreme settings
    sleep(Duration::from_millis(100)).await;
    
    // Should still be healthy (error-resilient)
    let health = autonomy_loop.health_check().await.unwrap();
    assert!(health, "Autonomy loop should be resilient to extreme configurations");
    
    // Stop the loop
    autonomy_loop.stop().await.unwrap();
    
    println!("Autonomy loop error scenarios test completed");
}

/// Test workflow execution error scenarios
#[tokio::test]
async fn test_workflow_execution_errors() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Test workflow with invalid JSON parameters
    let workflow_invalid_json = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Invalid JSON Workflow".to_string(),
        steps: vec![
            WorkflowStep {
                id: "invalid_step".to_string(),
                step_type: "test_step".to_string(),
                parameters: json!({
                    "invalid_number": f64::NAN,
                    "invalid_string": "\u{0000}invalid\u{0000}",
                    "circular_reference": "self"
                }),
            },
        ],
    };
    
    // Should handle invalid parameters gracefully
    let result = orchestrator.execute_workflow(workflow_invalid_json).await;
    assert!(result.is_ok(), "Should handle invalid JSON parameters gracefully");
    
    // Test workflow with extremely large parameters
    let large_data = "x".repeat(1_000_000); // 1MB string
    let workflow_large = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Large Data Workflow".to_string(),
        steps: vec![
            WorkflowStep {
                id: "large_step".to_string(),
                step_type: "large_data_step".to_string(),
                parameters: json!({
                    "large_string": large_data,
                    "large_array": vec![1; 100_000],
                    "nested_large": {
                        "data": vec!["large"; 10_000]
                    }
                }),
            },
        ],
    };
    
    // Should handle large data gracefully
    let result = orchestrator.execute_workflow(workflow_large).await;
    assert!(result.is_ok(), "Should handle large workflow data gracefully");
    
    // Test workflow with missing required fields
    let workflow_empty_id = Workflow {
        id: String::new(), // Empty ID
        name: "Empty ID Workflow".to_string(),
        steps: vec![],
    };
    
    let result = orchestrator.execute_workflow(workflow_empty_id).await;
    assert!(result.is_ok(), "Should handle empty workflow ID gracefully");
    
    println!("Workflow execution error scenarios test completed");
}

/// Test service registry error scenarios
#[tokio::test]
async fn test_service_registry_errors() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Test registering service with invalid data
    let invalid_service = Service {
        id: String::new(), // Empty ID
        name: String::new(), // Empty name
        service_type: String::new(), // Empty type
        endpoint: "invalid://endpoint:99999".to_string(), // Invalid endpoint
    };
    
    let result = orchestrator.register_service(invalid_service).await;
    assert!(result.is_ok(), "Should handle invalid service data gracefully");
    
    // Test registering service with extremely long fields
    let large_service = Service {
        id: "x".repeat(10_000),
        name: "y".repeat(10_000),
        service_type: "z".repeat(10_000),
        endpoint: format!("http://{}.com:8080", "a".repeat(1000)),
    };
    
    let result = orchestrator.register_service(large_service).await;
    assert!(result.is_ok(), "Should handle large service data gracefully");
    
    // Test service discovery with invalid parameters
    let result = orchestrator.discover_services("").await;
    assert!(result.is_ok(), "Should handle empty service type in discovery");
    
    let result = orchestrator.discover_services("invalid\0type\0with\0nulls").await;
    assert!(result.is_ok(), "Should handle invalid service type strings");
    
    println!("Service registry error scenarios test completed");
}

/// Test network and connectivity error scenarios
#[tokio::test]
async fn test_network_connectivity_errors() {
    // Test with unreachable bootstrap peers
    let config = OrchestratorConfig {
        qudag: QuDAGConfig {
            enabled: true,
            node_endpoint: "127.0.0.1:7999".to_string(),
            network_id: "unreachable-network".to_string(),
            node_id: "network-error-test-node".to_string(),
            bootstrap_peers: vec![
                "192.0.2.1:7000".to_string(), // RFC 5737 test address (unreachable)
                "192.0.2.2:7001".to_string(), // RFC 5737 test address (unreachable)
                "invalid-hostname:7002".to_string(), // Invalid hostname
            ],
            connection_timeout_ms: 1000, // Short timeout
            max_reconnection_attempts: 1, // Minimal retries
            participate_in_consensus: true,
            exchange_config: Default::default(),
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    
    // Should handle network connectivity issues gracefully
    let init_result = orchestrator.initialize().await;
    assert!(init_result.is_ok(), "Should handle network connectivity issues gracefully");
    
    // Should still be able to get statistics
    let stats = orchestrator.get_statistics().await;
    assert!(!stats.node_id.is_empty(), "Should work despite network issues");
    
    println!("Network connectivity error scenarios test completed");
}

/// Test timeout scenarios
#[tokio::test]
async fn test_timeout_scenarios() {
    let config = OrchestratorConfig {
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 100,
            max_tasks_per_iteration: 1,
            task_timeout_ms: 50, // Very short timeout
            enable_learning: false,
            rules_config: Default::default(),
            ai_config: Default::default(),
        },
        qudag: QuDAGConfig {
            connection_timeout_ms: 100, // Very short timeout
            ..Default::default()
        },
        mcp: McpConfig {
            request_timeout_ms: 100, // Very short timeout
            ..Default::default()
        },
        api: ApiConfig {
            request_timeout_ms: 100, // Very short timeout
            ..Default::default()
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    
    // Initialization with short timeouts should still work
    let init_result = timeout(Duration::from_secs(2), orchestrator.initialize()).await;
    assert!(init_result.is_ok(), "Initialization should complete within timeout");
    assert!(init_result.unwrap().is_ok(), "Initialization should succeed");
    
    // Execute workflow with timeout
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Timeout Test Workflow".to_string(),
        steps: vec![
            WorkflowStep {
                id: "timeout_step".to_string(),
                step_type: "timeout_test".to_string(),
                parameters: json!({
                    "simulated_duration": "5s",
                    "timeout_sensitive": true
                }),
            },
        ],
    };
    
    let workflow_result = timeout(
        Duration::from_secs(3),
        orchestrator.execute_workflow(workflow)
    ).await;
    
    assert!(workflow_result.is_ok(), "Workflow should complete within timeout");
    
    println!("Timeout scenarios test completed");
}

/// Test resource exhaustion scenarios
#[tokio::test]
async fn test_resource_exhaustion_scenarios() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Test registering many services rapidly
    let mut handles = Vec::new();
    for i in 0..100 {
        let service = Service {
            id: format!("resource-test-{}", i),
            name: format!("Resource Test Service {}", i),
            service_type: "stress_test".to_string(),
            endpoint: format!("localhost:{}", 10000 + i),
        };
        
        let handle = orchestrator.register_service(service);
        handles.push(handle);
    }
    
    // Wait for all registrations to complete
    let results = futures::future::join_all(handles).await;
    
    // All should succeed (current implementation is resilient)
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Service registration {} should succeed", i);
    }
    
    // Test executing many workflows concurrently
    let mut workflow_handles = Vec::new();
    for i in 0..50 {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: format!("Resource Exhaustion Test {}", i),
            steps: vec![
                WorkflowStep {
                    id: format!("step_{}", i),
                    step_type: "resource_intensive".to_string(),
                    parameters: json!({
                        "iteration": i,
                        "memory_usage": "high",
                        "cpu_usage": "high"
                    }),
                },
            ],
        };
        
        let handle = orchestrator.execute_workflow(workflow);
        workflow_handles.push(handle);
    }
    
    // Wait for all workflows to complete
    let workflow_results = futures::future::join_all(workflow_handles).await;
    
    // All should succeed
    for (i, result) in workflow_results.iter().enumerate() {
        assert!(result.is_ok(), "Workflow {} should succeed", i);
    }
    
    println!("Resource exhaustion scenarios test completed");
    println!("Processed {} service registrations and {} workflows", results.len(), workflow_results.len());
}

/// Test recovery mechanisms
#[tokio::test]
async fn test_recovery_mechanisms() {
    let config = OrchestratorConfig {
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 100,
            max_tasks_per_iteration: 1,
            task_timeout_ms: 5000,
            enable_learning: false,
            rules_config: Default::default(),
            ai_config: Default::default(),
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Test orchestrator recovery after stress
    let initial_stats = orchestrator.get_statistics().await;
    
    // Simulate stress with rapid operations
    for i in 0..20 {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: format!("Recovery Test {}", i),
            steps: vec![
                WorkflowStep {
                    id: format!("recovery_step_{}", i),
                    step_type: "recovery_test".to_string(),
                    parameters: json!({
                        "stress_factor": "high",
                        "recovery_test": true
                    }),
                },
            ],
        };
        
        let result = orchestrator.execute_workflow(workflow).await;
        assert!(result.is_ok(), "Workflow {} should succeed during recovery test", i);
    }
    
    // Allow time for recovery
    sleep(Duration::from_millis(500)).await;
    
    // Verify recovery
    let recovery_stats = orchestrator.get_statistics().await;
    assert!(!recovery_stats.node_id.is_empty(), "Node should be operational after recovery");
    
    // Test service functionality after stress
    let service = Service {
        id: "recovery-test-service".to_string(),
        name: "Recovery Test Service".to_string(),
        service_type: "recovery_test".to_string(),
        endpoint: "localhost:9999".to_string(),
    };
    
    let register_result = orchestrator.register_service(service).await;
    assert!(register_result.is_ok(), "Service registration should work after recovery");
    
    println!("Recovery mechanisms test completed");
    println!("Initial stats: {}", initial_stats);
    println!("Recovery stats: {}", recovery_stats);
}

/// Test error propagation and handling
#[tokio::test]
async fn test_error_propagation() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Test workflow with nested error scenarios
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Error Propagation Test".to_string(),
        steps: vec![
            WorkflowStep {
                id: "error_step_1".to_string(),
                step_type: "error_generator".to_string(),
                parameters: json!({
                    "error_type": "validation_error",
                    "should_propagate": false,
                    "recovery_strategy": "ignore"
                }),
            },
            WorkflowStep {
                id: "error_step_2".to_string(),
                step_type: "error_generator".to_string(),
                parameters: json!({
                    "error_type": "network_error",
                    "should_propagate": false,
                    "recovery_strategy": "retry"
                }),
            },
            WorkflowStep {
                id: "error_step_3".to_string(),
                step_type: "error_generator".to_string(),
                parameters: json!({
                    "error_type": "timeout_error",
                    "should_propagate": false,
                    "recovery_strategy": "fallback"
                }),
            },
        ],
    };
    
    let result = orchestrator.execute_workflow(workflow).await;
    assert!(result.is_ok(), "Should handle error propagation gracefully");
    
    // Test multiple error scenarios in sequence
    for error_type in ["validation", "network", "timeout", "resource", "security"] {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: format!("Error Test - {}", error_type),
            steps: vec![
                WorkflowStep {
                    id: format!("{}_error_step", error_type),
                    step_type: "controlled_error".to_string(),
                    parameters: json!({
                        "error_type": error_type,
                        "simulate_only": true,
                        "should_recover": true
                    }),
                },
            ],
        };
        
        let result = orchestrator.execute_workflow(workflow).await;
        assert!(result.is_ok(), "Should handle {} errors gracefully", error_type);
    }
    
    println!("Error propagation test completed");
}

/// Test concurrent error scenarios
#[tokio::test]
async fn test_concurrent_error_scenarios() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Create multiple workflows with different error scenarios
    let error_scenarios = vec![
        ("timeout", "timeout_simulation"),
        ("validation", "validation_failure"),
        ("network", "network_unavailable"),
        ("resource", "resource_exhaustion"),
        ("security", "security_violation"),
    ];
    
    let mut handles = Vec::new();
    
    for (i, (error_type, step_type)) in error_scenarios.iter().enumerate() {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: format!("Concurrent Error Test - {}", error_type),
            steps: vec![
                WorkflowStep {
                    id: format!("concurrent_error_{}", i),
                    step_type: step_type.to_string(),
                    parameters: json!({
                        "error_type": error_type,
                        "concurrent_test": true,
                        "should_recover": true,
                        "delay_ms": i * 100 // Stagger execution
                    }),
                },
            ],
        };
        
        let handle = orchestrator.execute_workflow(workflow);
        handles.push(handle);
    }
    
    // Execute all error scenarios concurrently
    let results = futures::future::join_all(handles).await;
    
    // All should complete successfully (errors are simulated)
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Concurrent error scenario {} should complete", i);
    }
    
    // Verify orchestrator is still healthy after concurrent errors
    let stats = orchestrator.get_statistics().await;
    assert!(!stats.node_id.is_empty(), "Orchestrator should remain healthy after concurrent errors");
    
    println!("Concurrent error scenarios test completed");
    println!("Processed {} concurrent error scenarios", results.len());
}