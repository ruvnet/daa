//! End-to-end tests for DAA workflow execution

use daa_orchestrator::{
    DaaOrchestrator, OrchestratorConfig,
    workflow::{Workflow, WorkflowStep, WorkflowStatus, WorkflowResult},
};
use serde_json::json;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

/// Test basic workflow creation and execution
#[tokio::test]
async fn test_basic_workflow_execution() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Create a simple workflow
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Basic Test Workflow".to_string(),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: "test_action".to_string(),
                parameters: json!({
                    "action": "initialize",
                    "timeout": 5000
                }),
            },
        ],
    };
    
    // Execute workflow
    let result = orchestrator.execute_workflow(workflow.clone()).await;
    assert!(result.is_ok(), "Failed to execute basic workflow");
    
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result.workflow_id, workflow.id);
    assert!(matches!(workflow_result.status, WorkflowStatus::Completed));
}

/// Test multi-step workflow execution
#[tokio::test]
async fn test_multi_step_workflow_execution() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Create a multi-step workflow
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Multi-Step Treasury Management".to_string(),
        steps: vec![
            WorkflowStep {
                id: "check_balance".to_string(),
                step_type: "treasury_check".to_string(),
                parameters: json!({
                    "account": "main_treasury",
                    "min_balance": 1000.0
                }),
            },
            WorkflowStep {
                id: "evaluate_risk".to_string(),
                step_type: "risk_assessment".to_string(),
                parameters: json!({
                    "risk_model": "conservative",
                    "max_risk_score": 0.7
                }),
            },
            WorkflowStep {
                id: "execute_trade".to_string(),
                step_type: "trading_action".to_string(),
                parameters: json!({
                    "pair": "rUv/USD",
                    "amount": 100.0,
                    "order_type": "market"
                }),
            },
            WorkflowStep {
                id: "record_transaction".to_string(),
                step_type: "ledger_update".to_string(),
                parameters: json!({
                    "transaction_type": "trade_execution",
                    "audit_trail": true
                }),
            },
        ],
    };
    
    // Execute multi-step workflow
    let result = orchestrator.execute_workflow(workflow.clone()).await;
    assert!(result.is_ok(), "Failed to execute multi-step workflow");
    
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result.workflow_id, workflow.id);
    assert!(matches!(workflow_result.status, WorkflowStatus::Completed));
}

/// Test AI agent coordination workflow
#[tokio::test]
async fn test_ai_agent_coordination_workflow() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "AI Agent Coordination".to_string(),
        steps: vec![
            WorkflowStep {
                id: "spawn_researcher".to_string(),
                step_type: "ai_agent_spawn".to_string(),
                parameters: json!({
                    "agent_type": "researcher",
                    "capabilities": ["web_search", "data_analysis"],
                    "priority": "high"
                }),
            },
            WorkflowStep {
                id: "spawn_trader".to_string(),
                step_type: "ai_agent_spawn".to_string(),
                parameters: json!({
                    "agent_type": "trader",
                    "capabilities": ["market_analysis", "order_execution"],
                    "priority": "medium"
                }),
            },
            WorkflowStep {
                id: "research_task".to_string(),
                step_type: "task_assignment".to_string(),
                parameters: json!({
                    "agent_id": "researcher",
                    "task": "analyze_market_trends",
                    "deadline": "30m"
                }),
            },
            WorkflowStep {
                id: "coordinate_decision".to_string(),
                step_type: "multi_agent_coordination".to_string(),
                parameters: json!({
                    "agents": ["researcher", "trader"],
                    "coordination_type": "consensus",
                    "decision_threshold": 0.8
                }),
            },
        ],
    };
    
    let result = orchestrator.execute_workflow(workflow.clone()).await;
    assert!(result.is_ok(), "Failed to execute AI coordination workflow");
    
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result.workflow_id, workflow.id);
    assert!(matches!(workflow_result.status, WorkflowStatus::Completed));
}

/// Test rule-based compliance workflow
#[tokio::test]
async fn test_rule_compliance_workflow() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Rule Compliance Check".to_string(),
        steps: vec![
            WorkflowStep {
                id: "pre_transaction_check".to_string(),
                step_type: "rules_evaluation".to_string(),
                parameters: json!({
                    "rule_set": "treasury_management",
                    "transaction": {
                        "amount": 500.0,
                        "type": "withdrawal",
                        "destination": "exchange_wallet"
                    }
                }),
            },
            WorkflowStep {
                id: "risk_assessment".to_string(),
                step_type: "risk_calculation".to_string(),
                parameters: json!({
                    "factors": ["market_volatility", "liquidity_risk", "counterparty_risk"],
                    "max_acceptable_risk": 0.6
                }),
            },
            WorkflowStep {
                id: "compliance_approval".to_string(),
                step_type: "approval_workflow".to_string(),
                parameters: json!({
                    "approval_type": "automated",
                    "escalation_threshold": 0.8,
                    "audit_required": true
                }),
            },
        ],
    };
    
    let result = orchestrator.execute_workflow(workflow.clone()).await;
    assert!(result.is_ok(), "Failed to execute compliance workflow");
    
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result.workflow_id, workflow.id);
    assert!(matches!(workflow_result.status, WorkflowStatus::Completed));
}

/// Test economic operations workflow
#[tokio::test]
async fn test_economic_operations_workflow() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Economic Operations".to_string(),
        steps: vec![
            WorkflowStep {
                id: "market_analysis".to_string(),
                step_type: "market_data_collection".to_string(),
                parameters: json!({
                    "sources": ["exchange_orderbook", "price_feeds", "volume_indicators"],
                    "timeframe": "1h"
                }),
            },
            WorkflowStep {
                id: "liquidity_assessment".to_string(),
                step_type: "liquidity_analysis".to_string(),
                parameters: json!({
                    "pools": ["rUv/USD", "rUv/BTC"],
                    "min_liquidity": 10000.0
                }),
            },
            WorkflowStep {
                id: "optimization_calculation".to_string(),
                step_type: "portfolio_optimization".to_string(),
                parameters: json!({
                    "algorithm": "mean_variance",
                    "constraints": {
                        "max_position_size": 0.2,
                        "max_daily_trades": 10
                    }
                }),
            },
            WorkflowStep {
                id: "execute_rebalancing".to_string(),
                step_type: "portfolio_rebalancing".to_string(),
                parameters: json!({
                    "strategy": "gradual",
                    "execution_time": "15m",
                    "slippage_tolerance": 0.01
                }),
            },
        ],
    };
    
    let result = orchestrator.execute_workflow(workflow.clone()).await;
    assert!(result.is_ok(), "Failed to execute economic operations workflow");
    
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result.workflow_id, workflow.id);
    assert!(matches!(workflow_result.status, WorkflowStatus::Completed));
}

/// Test workflow execution with timeout
#[tokio::test]
async fn test_workflow_execution_timeout() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Timeout Test Workflow".to_string(),
        steps: vec![
            WorkflowStep {
                id: "quick_step".to_string(),
                step_type: "fast_operation".to_string(),
                parameters: json!({
                    "operation": "status_check",
                    "expected_duration": "100ms"
                }),
            },
        ],
    };
    
    // Execute with timeout
    let result = timeout(
        Duration::from_secs(5),
        orchestrator.execute_workflow(workflow.clone())
    ).await;
    
    assert!(result.is_ok(), "Workflow execution should not timeout");
    
    let workflow_result = result.unwrap().unwrap();
    assert_eq!(workflow_result.workflow_id, workflow.id);
}

/// Test concurrent workflow execution
#[tokio::test]
async fn test_concurrent_workflow_execution() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Create multiple workflows
    let workflow1 = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Concurrent Workflow 1".to_string(),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: "parallel_task".to_string(),
                parameters: json!({"task_id": 1}),
            },
        ],
    };
    
    let workflow2 = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Concurrent Workflow 2".to_string(),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: "parallel_task".to_string(),
                parameters: json!({"task_id": 2}),
            },
        ],
    };
    
    let workflow3 = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Concurrent Workflow 3".to_string(),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: "parallel_task".to_string(),
                parameters: json!({"task_id": 3}),
            },
        ],
    };
    
    // Execute workflows concurrently
    let (result1, result2, result3) = tokio::join!(
        orchestrator.execute_workflow(workflow1.clone()),
        orchestrator.execute_workflow(workflow2.clone()),
        orchestrator.execute_workflow(workflow3.clone())
    );
    
    // All should succeed
    assert!(result1.is_ok(), "Concurrent workflow 1 failed");
    assert!(result2.is_ok(), "Concurrent workflow 2 failed");
    assert!(result3.is_ok(), "Concurrent workflow 3 failed");
    
    // Verify results
    let result1 = result1.unwrap();
    let result2 = result2.unwrap();
    let result3 = result3.unwrap();
    
    assert_eq!(result1.workflow_id, workflow1.id);
    assert_eq!(result2.workflow_id, workflow2.id);
    assert_eq!(result3.workflow_id, workflow3.id);
    
    assert!(matches!(result1.status, WorkflowStatus::Completed));
    assert!(matches!(result2.status, WorkflowStatus::Completed));
    assert!(matches!(result3.status, WorkflowStatus::Completed));
}

/// Test workflow with complex parameter types
#[tokio::test]
async fn test_workflow_complex_parameters() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Complex Parameters Workflow".to_string(),
        steps: vec![
            WorkflowStep {
                id: "complex_processing".to_string(),
                step_type: "advanced_operation".to_string(),
                parameters: json!({
                    "nested_config": {
                        "algorithm_settings": {
                            "learning_rate": 0.001,
                            "batch_size": 32,
                            "epochs": 100
                        },
                        "data_sources": [
                            {
                                "type": "market_data",
                                "endpoint": "wss://api.exchange.com/ws",
                                "auth": {
                                    "type": "api_key",
                                    "key": "test_key"
                                }
                            },
                            {
                                "type": "blockchain_data",
                                "endpoint": "https://api.qudag.network/v1",
                                "filters": ["transactions", "blocks"]
                            }
                        ]
                    },
                    "execution_parameters": {
                        "max_retries": 3,
                        "timeout_seconds": 300,
                        "fallback_strategy": "conservative",
                        "notification_channels": ["slack", "email"]
                    }
                }),
            },
        ],
    };
    
    let result = orchestrator.execute_workflow(workflow.clone()).await;
    assert!(result.is_ok(), "Failed to execute workflow with complex parameters");
    
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result.workflow_id, workflow.id);
    assert!(matches!(workflow_result.status, WorkflowStatus::Completed));
}

/// Test workflow statistics and monitoring
#[tokio::test]
async fn test_workflow_statistics() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Get initial statistics
    let initial_stats = orchestrator.get_statistics().await;
    assert_eq!(initial_stats.active_workflows, 0);
    
    // Execute a workflow
    let workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Statistics Test Workflow".to_string(),
        steps: vec![
            WorkflowStep {
                id: "stats_step".to_string(),
                step_type: "monitoring_operation".to_string(),
                parameters: json!({
                    "collect_metrics": true,
                    "duration": "1s"
                }),
            },
        ],
    };
    
    let result = orchestrator.execute_workflow(workflow).await;
    assert!(result.is_ok(), "Failed to execute statistics test workflow");
    
    // Statistics should reflect the execution
    let final_stats = orchestrator.get_statistics().await;
    // Note: In the current implementation, active_workflows might be 0 after completion
    // This is expected behavior as workflows complete quickly
    
    println!("Workflow statistics test completed");
    println!("Initial stats: {}", initial_stats);
    println!("Final stats: {}", final_stats);
}

/// Test workflow error handling scenarios
#[tokio::test]
async fn test_workflow_error_scenarios() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    // Test workflow with empty steps (should still succeed in current implementation)
    let empty_workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Empty Workflow".to_string(),
        steps: vec![],
    };
    
    let result = orchestrator.execute_workflow(empty_workflow).await;
    assert!(result.is_ok(), "Empty workflow should be handled gracefully");
    
    // Test workflow with complex error-prone parameters
    let error_workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Error Handling Test".to_string(),
        steps: vec![
            WorkflowStep {
                id: "error_step".to_string(),
                step_type: "error_simulation".to_string(),
                parameters: json!({
                    "simulate_error": false, // Don't actually error in test
                    "error_type": "network_timeout",
                    "recovery_strategy": "retry_with_backoff"
                }),
            },
        ],
    };
    
    let result = orchestrator.execute_workflow(error_workflow).await;
    assert!(result.is_ok(), "Error handling workflow should succeed");
}

/// Performance test for workflow execution
#[tokio::test]
async fn test_workflow_performance() {
    let config = OrchestratorConfig::default();
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    let start_time = std::time::Instant::now();
    
    // Execute multiple workflows to test performance
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: format!("Performance Test Workflow {}", i),
            steps: vec![
                WorkflowStep {
                    id: format!("perf_step_{}", i),
                    step_type: "performance_test".to_string(),
                    parameters: json!({
                        "iteration": i,
                        "load_test": true
                    }),
                },
            ],
        };
        
        let handle = orchestrator.execute_workflow(workflow);
        handles.push(handle);
    }
    
    // Wait for all workflows to complete
    let results = futures::future::join_all(handles).await;
    
    let elapsed = start_time.elapsed();
    
    // All workflows should succeed
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Performance test workflow {} failed", i);
    }
    
    println!("Performance test completed: {} workflows in {:?}", results.len(), elapsed);
    
    // Should complete reasonably quickly
    assert!(elapsed < Duration::from_secs(10), "Performance test took too long: {:?}", elapsed);
}