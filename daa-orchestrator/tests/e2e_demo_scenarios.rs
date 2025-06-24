//! Demo scenarios showcasing DAA orchestrator capabilities

use daa_orchestrator::{
    DaaOrchestrator, OrchestratorConfig,
    workflow::{Workflow, WorkflowStep, WorkflowStatus},
    services::Service,
    config::{AutonomyConfig, QuDAGConfig, McpConfig, ApiConfig, ExchangeConfig, RulesConfig, AiConfig},
};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Demo: Autonomous Treasury Management
/// Demonstrates the complete lifecycle of autonomous treasury operations
#[tokio::test]
async fn demo_autonomous_treasury_management() {
    println!("\n=== DEMO: Autonomous Treasury Management ===");
    
    // Configure orchestrator for treasury management
    let config = OrchestratorConfig {
        name: "treasury-manager".to_string(),
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 1000,
            max_tasks_per_iteration: 5,
            task_timeout_ms: 30000,
            enable_learning: true,
            rules_config: RulesConfig {
                enabled: true,
                fail_fast: false,
                max_daily_spending: 50000.0,
                min_balance_threshold: 10000.0,
                max_risk_score: 0.7,
            },
            ai_config: AiConfig {
                enabled: true,
                max_agents: 3,
                agent_queue_size: 50,
                learning_retention_days: 30,
            },
        },
        qudag: QuDAGConfig {
            enabled: true,
            node_endpoint: "localhost:7500".to_string(),
            network_id: "treasury-network".to_string(),
            node_id: "treasury-node-001".to_string(),
            bootstrap_peers: vec!["localhost:7501".to_string()],
            connection_timeout_ms: 10000,
            max_reconnection_attempts: 5,
            participate_in_consensus: true,
            exchange_config: ExchangeConfig {
                enabled: true,
                endpoint: "localhost:8500".to_string(),
                trading_pairs: vec!["rUv/USD".to_string(), "rUv/BTC".to_string(), "rUv/ETH".to_string()],
                order_book_depth: 50,
            },
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    println!("✓ Treasury management orchestrator initialized");
    
    // Register treasury-specific services
    let treasury_services = vec![
        Service {
            id: "treasury-ai-advisor".to_string(),
            name: "Treasury AI Advisor".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9500".to_string(),
        },
        Service {
            id: "risk-assessment-engine".to_string(),
            name: "Risk Assessment Engine".to_string(),
            service_type: "rules_engine".to_string(),
            endpoint: "localhost:9501".to_string(),
        },
        Service {
            id: "market-data-provider".to_string(),
            name: "Market Data Provider".to_string(),
            service_type: "data_provider".to_string(),
            endpoint: "localhost:9502".to_string(),
        },
        Service {
            id: "blockchain-interface".to_string(),
            name: "QuDAG Blockchain Interface".to_string(),
            service_type: "blockchain_bridge".to_string(),
            endpoint: "localhost:9503".to_string(),
        },
    ];
    
    for service in &treasury_services {
        orchestrator.register_service(service.clone()).await.unwrap();
        println!("✓ Registered service: {}", service.name);
    }
    
    // Execute comprehensive treasury management workflow
    let treasury_workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Autonomous Treasury Management Cycle".to_string(),
        steps: vec![
            WorkflowStep {
                id: "portfolio_assessment".to_string(),
                step_type: "portfolio_analysis".to_string(),
                parameters: json!({
                    "assessment_type": "comprehensive",
                    "include_holdings": true,
                    "include_performance": true,
                    "timeframe": "24h",
                    "benchmark": "market_index"
                }),
            },
            WorkflowStep {
                id: "market_analysis".to_string(),
                step_type: "market_intelligence".to_string(),
                parameters: json!({
                    "data_sources": ["exchange_orderbook", "price_feeds", "volume_indicators", "sentiment_analysis"],
                    "analysis_depth": "deep",
                    "prediction_horizon": "4h",
                    "confidence_threshold": 0.8
                }),
            },
            WorkflowStep {
                id: "risk_evaluation".to_string(),
                step_type: "risk_assessment".to_string(),
                parameters: json!({
                    "risk_factors": [
                        "market_volatility",
                        "liquidity_risk",
                        "counterparty_risk",
                        "regulatory_risk",
                        "operational_risk"
                    ],
                    "risk_model": "monte_carlo",
                    "confidence_level": 0.95,
                    "stress_test": true
                }),
            },
            WorkflowStep {
                id: "compliance_check".to_string(),
                step_type: "regulatory_compliance".to_string(),
                parameters: json!({
                    "compliance_frameworks": ["treasury_policy", "risk_limits", "audit_requirements"],
                    "auto_approve_threshold": 0.9,
                    "escalation_required": false
                }),
            },
            WorkflowStep {
                id: "optimization_calculation".to_string(),
                step_type: "portfolio_optimization".to_string(),
                parameters: json!({
                    "optimization_objective": "risk_adjusted_return",
                    "constraints": {
                        "max_position_size": 0.25,
                        "max_daily_turnover": 0.1,
                        "min_liquidity_ratio": 0.2,
                        "diversification_threshold": 0.6
                    },
                    "algorithm": "black_litterman"
                }),
            },
            WorkflowStep {
                id: "execution_planning".to_string(),
                step_type: "trade_execution_planning".to_string(),
                parameters: json!({
                    "execution_strategy": "twap",
                    "execution_window": "2h",
                    "slippage_tolerance": 0.005,
                    "market_impact_limit": 0.01,
                    "fragmentation_allowed": true
                }),
            },
            WorkflowStep {
                id: "trade_execution".to_string(),
                step_type: "automated_trading".to_string(),
                parameters: json!({
                    "execution_mode": "cautious",
                    "monitoring_frequency": "30s",
                    "stop_loss_enabled": true,
                    "profit_taking_enabled": true,
                    "partial_fill_handling": "accumulate"
                }),
            },
            WorkflowStep {
                id: "settlement_management".to_string(),
                step_type: "settlement_processing".to_string(),
                parameters: json!({
                    "settlement_network": "qudag",
                    "confirmation_requirements": 3,
                    "timeout_handling": "retry_with_escalation",
                    "audit_trail": true
                }),
            },
            WorkflowStep {
                id: "performance_reporting".to_string(),
                step_type: "performance_analysis".to_string(),
                parameters: json!({
                    "report_types": ["execution_report", "risk_report", "performance_attribution"],
                    "distribution_list": ["treasury_team", "risk_committee", "audit"],
                    "real_time_updates": true
                }),
            },
            WorkflowStep {
                id: "learning_integration".to_string(),
                step_type: "machine_learning_update".to_string(),
                parameters: json!({
                    "learning_sources": ["execution_outcomes", "market_movements", "risk_realizations"],
                    "model_updates": ["prediction_models", "risk_models", "execution_models"],
                    "validation_required": true
                }),
            },
        ],
    };
    
    println!("🚀 Executing autonomous treasury management workflow...");
    let result = orchestrator.execute_workflow(treasury_workflow.clone()).await.unwrap();
    
    assert!(matches!(result.status, WorkflowStatus::Completed));
    println!("✅ Treasury management workflow completed successfully");
    println!("   Workflow ID: {}", result.workflow_id);
    
    // Display final statistics
    let stats = orchestrator.get_statistics().await;
    println!("📊 Treasury Management Statistics:");
    println!("   Active Workflows: {}", stats.active_workflows);
    println!("   Registered Services: {}", stats.registered_services);
    println!("   Operations Coordinated: {}", stats.coordinated_operations);
    println!("   Events Processed: {}", stats.processed_events);
    println!("   Node ID: {}", stats.node_id);
    
    println!("✅ Demo: Autonomous Treasury Management completed successfully\n");
}

/// Demo: Multi-Agent Coordination for DeFi Operations
/// Shows how multiple AI agents coordinate complex DeFi strategies
#[tokio::test]
async fn demo_multi_agent_defi_coordination() {
    println!("\n=== DEMO: Multi-Agent DeFi Coordination ===");
    
    let config = OrchestratorConfig {
        name: "defi-coordinator".to_string(),
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 2000,
            max_tasks_per_iteration: 8,
            task_timeout_ms: 45000,
            enable_learning: true,
            rules_config: RulesConfig {
                enabled: true,
                fail_fast: false,
                max_daily_spending: 100000.0,
                min_balance_threshold: 5000.0,
                max_risk_score: 0.8,
            },
            ai_config: AiConfig {
                enabled: true,
                max_agents: 10,
                agent_queue_size: 100,
                learning_retention_days: 60,
            },
        },
        qudag: QuDAGConfig {
            enabled: true,
            node_endpoint: "localhost:7600".to_string(),
            network_id: "defi-coordination-network".to_string(),
            node_id: "defi-coordinator-001".to_string(),
            bootstrap_peers: vec![
                "localhost:7601".to_string(),
                "localhost:7602".to_string(),
                "localhost:7603".to_string(),
            ],
            connection_timeout_ms: 15000,
            max_reconnection_attempts: 5,
            participate_in_consensus: true,
            exchange_config: ExchangeConfig {
                enabled: true,
                endpoint: "localhost:8600".to_string(),
                trading_pairs: vec![
                    "rUv/USD".to_string(),
                    "rUv/BTC".to_string(),
                    "rUv/ETH".to_string(),
                    "rUv/USDC".to_string(),
                ],
                order_book_depth: 100,
            },
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    println!("✓ DeFi coordination orchestrator initialized");
    
    // Register specialized DeFi agents
    let defi_agents = vec![
        Service {
            id: "yield-farming-agent".to_string(),
            name: "Yield Farming Specialist".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9600".to_string(),
        },
        Service {
            id: "arbitrage-agent".to_string(),
            name: "Arbitrage Opportunity Hunter".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9601".to_string(),
        },
        Service {
            id: "liquidity-agent".to_string(),
            name: "Liquidity Pool Manager".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9602".to_string(),
        },
        Service {
            id: "risk-monitor-agent".to_string(),
            name: "DeFi Risk Monitor".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9603".to_string(),
        },
        Service {
            id: "governance-agent".to_string(),
            name: "Governance Participation Agent".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9604".to_string(),
        },
        Service {
            id: "defi-rules-engine".to_string(),
            name: "DeFi Compliance Engine".to_string(),
            service_type: "rules_engine".to_string(),
            endpoint: "localhost:9605".to_string(),
        },
        Service {
            id: "protocol-bridge".to_string(),
            name: "Multi-Protocol Bridge".to_string(),
            service_type: "blockchain_bridge".to_string(),
            endpoint: "localhost:9606".to_string(),
        },
    ];
    
    for agent in &defi_agents {
        orchestrator.register_service(agent.clone()).await.unwrap();
        println!("✓ Registered agent: {}", agent.name);
    }
    
    // Execute coordinated DeFi strategy workflow
    let defi_workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Multi-Agent DeFi Strategy Coordination".to_string(),
        steps: vec![
            WorkflowStep {
                id: "agent_coordination_setup".to_string(),
                step_type: "multi_agent_initialization".to_string(),
                parameters: json!({
                    "coordination_mode": "collaborative",
                    "communication_protocol": "consensus_based",
                    "conflict_resolution": "weighted_voting",
                    "performance_tracking": true,
                    "agents": [
                        {"id": "yield-farming-agent", "weight": 0.25, "specialty": "yield_optimization"},
                        {"id": "arbitrage-agent", "weight": 0.20, "specialty": "price_inefficiencies"},
                        {"id": "liquidity-agent", "weight": 0.20, "specialty": "liquidity_provision"},
                        {"id": "risk-monitor-agent", "weight": 0.25, "specialty": "risk_assessment"},
                        {"id": "governance-agent", "weight": 0.10, "specialty": "governance_voting"}
                    ]
                }),
            },
            WorkflowStep {
                id: "market_opportunity_discovery".to_string(),
                step_type: "collaborative_market_analysis".to_string(),
                parameters: json!({
                    "analysis_scope": "cross_protocol",
                    "opportunity_types": [
                        "yield_farming",
                        "arbitrage",
                        "liquidity_mining",
                        "governance_rewards",
                        "flash_loan_opportunities"
                    ],
                    "time_horizon": "4h",
                    "minimum_apy": 0.05,
                    "agent_collaboration": {
                        "data_sharing": true,
                        "joint_analysis": true,
                        "consensus_threshold": 0.7
                    }
                }),
            },
            WorkflowStep {
                id: "risk_coordination".to_string(),
                step_type: "multi_agent_risk_assessment".to_string(),
                parameters: json!({
                    "risk_assessment_agents": ["risk-monitor-agent", "yield-farming-agent", "liquidity-agent"],
                    "risk_factors": [
                        "smart_contract_risk",
                        "impermanent_loss",
                        "protocol_risk",
                        "liquidity_risk",
                        "governance_risk"
                    ],
                    "coordination_method": "holistic_evaluation",
                    "risk_tolerance": "moderate_aggressive"
                }),
            },
            WorkflowStep {
                id: "strategy_consensus".to_string(),
                step_type: "agent_strategy_consensus".to_string(),
                parameters: json!({
                    "consensus_mechanism": "weighted_approval_voting",
                    "minimum_agreement": 0.75,
                    "strategy_categories": [
                        "capital_allocation",
                        "timing_coordination",
                        "risk_mitigation",
                        "profit_optimization"
                    ],
                    "fallback_strategy": "conservative_default"
                }),
            },
            WorkflowStep {
                id: "coordinated_execution".to_string(),
                step_type: "multi_agent_execution".to_string(),
                parameters: json!({
                    "execution_coordination": "synchronized",
                    "monitoring_frequency": "real_time",
                    "adaptive_execution": true,
                    "cross_agent_communication": true,
                    "performance_optimization": {
                        "gas_optimization": true,
                        "mev_protection": true,
                        "slippage_minimization": true
                    }
                }),
            },
            WorkflowStep {
                id: "governance_participation".to_string(),
                step_type: "coordinated_governance".to_string(),
                parameters: json!({
                    "governance_agent": "governance-agent",
                    "participation_strategy": "value_maximizing",
                    "coordination_with_other_agents": true,
                    "voting_power_optimization": true,
                    "proposal_analysis": "automated"
                }),
            },
            WorkflowStep {
                id: "performance_synchronization".to_string(),
                step_type: "multi_agent_performance_sync".to_string(),
                parameters: json!({
                    "performance_metrics": [
                        "individual_agent_performance",
                        "collaborative_efficiency",
                        "strategy_effectiveness",
                        "risk_adjusted_returns"
                    ],
                    "learning_sharing": true,
                    "model_synchronization": true,
                    "collective_intelligence": true
                }),
            },
            WorkflowStep {
                id: "adaptive_learning".to_string(),
                step_type: "collaborative_learning".to_string(),
                parameters: json!({
                    "learning_scope": "collective",
                    "knowledge_sharing": "bidirectional",
                    "experience_synthesis": true,
                    "strategy_evolution": "continuous",
                    "performance_feedback_loop": true
                }),
            },
        ],
    };
    
    println!("🤖 Executing multi-agent DeFi coordination workflow...");
    let result = orchestrator.execute_workflow(defi_workflow.clone()).await.unwrap();
    
    assert!(matches!(result.status, WorkflowStatus::Completed));
    println!("✅ Multi-agent DeFi coordination completed successfully");
    
    // Simulate some agent coordination time
    sleep(Duration::from_millis(500)).await;
    
    let stats = orchestrator.get_statistics().await;
    println!("🤖 Multi-Agent Coordination Statistics:");
    println!("   Coordinated Agents: {}", defi_agents.len());
    println!("   Workflow Executions: 1");
    println!("   Active Services: {}", stats.registered_services);
    println!("   Coordination Operations: {}", stats.coordinated_operations);
    
    println!("✅ Demo: Multi-Agent DeFi Coordination completed successfully\n");
}

/// Demo: Rule Violation Handling and Compliance
/// Shows how the system handles rule violations and maintains compliance
#[tokio::test]
async fn demo_rule_violation_handling() {
    println!("\n=== DEMO: Rule Violation Handling and Compliance ===");
    
    let config = OrchestratorConfig {
        name: "compliance-manager".to_string(),
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 1500,
            max_tasks_per_iteration: 3,
            task_timeout_ms: 20000,
            enable_learning: true,
            rules_config: RulesConfig {
                enabled: true,
                fail_fast: true, // Strict compliance mode
                max_daily_spending: 25000.0, // Conservative limit
                min_balance_threshold: 15000.0, // High safety threshold
                max_risk_score: 0.5, // Low risk tolerance
            },
            ai_config: AiConfig {
                enabled: true,
                max_agents: 2,
                agent_queue_size: 20,
                learning_retention_days: 90, // Long learning retention for compliance
            },
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    println!("✓ Compliance management orchestrator initialized");
    
    // Register compliance-focused services
    let compliance_services = vec![
        Service {
            id: "compliance-monitor".to_string(),
            name: "Real-time Compliance Monitor".to_string(),
            service_type: "rules_engine".to_string(),
            endpoint: "localhost:9700".to_string(),
        },
        Service {
            id: "audit-agent".to_string(),
            name: "Audit and Investigation Agent".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9701".to_string(),
        },
        Service {
            id: "notification-service".to_string(),
            name: "Compliance Notification Service".to_string(),
            service_type: "notification".to_string(),
            endpoint: "localhost:9702".to_string(),
        },
        Service {
            id: "remediation-engine".to_string(),
            name: "Automated Remediation Engine".to_string(),
            service_type: "remediation".to_string(),
            endpoint: "localhost:9703".to_string(),
        },
    ];
    
    for service in &compliance_services {
        orchestrator.register_service(service.clone()).await.unwrap();
        println!("✓ Registered service: {}", service.name);
    }
    
    // Test various compliance scenarios
    let compliance_scenarios = vec![
        ("spending_limit_violation", "Spending Limit Violation Test"),
        ("balance_threshold_violation", "Balance Threshold Violation Test"),
        ("risk_score_violation", "Risk Score Violation Test"),
        ("unauthorized_operation", "Unauthorized Operation Test"),
        ("compliance_recovery", "Compliance Recovery Test"),
    ];
    
    for (scenario_type, scenario_name) in &compliance_scenarios {
        println!("🔍 Testing scenario: {}", scenario_name);
        
        let compliance_workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name: scenario_name.to_string(),
            steps: vec![
                WorkflowStep {
                    id: "compliance_pre_check".to_string(),
                    step_type: "compliance_validation".to_string(),
                    parameters: json!({
                        "validation_type": "pre_execution",
                        "rules_to_check": ["spending_limits", "balance_thresholds", "risk_limits"],
                        "strict_mode": true,
                        "auto_remediation": true
                    }),
                },
                WorkflowStep {
                    id: "simulate_violation".to_string(),
                    step_type: "compliance_simulation".to_string(),
                    parameters: json!({
                        "scenario_type": scenario_type,
                        "simulation_mode": true,
                        "violation_severity": if scenario_type.contains("violation") { "medium" } else { "low" },
                        "test_remediation": true
                    }),
                },
                WorkflowStep {
                    id: "violation_detection".to_string(),
                    step_type: "real_time_monitoring".to_string(),
                    parameters: json!({
                        "monitoring_scope": "all_rules",
                        "detection_sensitivity": "high",
                        "immediate_alerts": true,
                        "automated_response": true
                    }),
                },
                WorkflowStep {
                    id: "investigation_analysis".to_string(),
                    step_type: "automated_investigation".to_string(),
                    parameters: json!({
                        "investigation_depth": "thorough",
                        "root_cause_analysis": true,
                        "impact_assessment": true,
                        "recommendation_generation": true
                    }),
                },
                WorkflowStep {
                    id: "remediation_execution".to_string(),
                    step_type: "automated_remediation".to_string(),
                    parameters: json!({
                        "remediation_strategy": "immediate_containment",
                        "escalation_threshold": "medium",
                        "user_notification": true,
                        "audit_trail_creation": true
                    }),
                },
                WorkflowStep {
                    id: "compliance_restoration".to_string(),
                    step_type: "compliance_restoration".to_string(),
                    parameters: json!({
                        "restoration_method": "systematic",
                        "verification_required": true,
                        "learning_integration": true,
                        "policy_updates": "as_needed"
                    }),
                },
                WorkflowStep {
                    id: "post_incident_review".to_string(),
                    step_type: "post_incident_analysis".to_string(),
                    parameters: json!({
                        "review_scope": "comprehensive",
                        "lessons_learned": true,
                        "policy_recommendations": true,
                        "preventive_measures": true
                    }),
                },
            ],
        };
        
        let result = orchestrator.execute_workflow(compliance_workflow).await.unwrap();
        assert!(matches!(result.status, WorkflowStatus::Completed));
        println!("  ✅ {} completed successfully", scenario_name);
    }
    
    // Execute comprehensive compliance workflow
    let comprehensive_compliance = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Comprehensive Compliance Management".to_string(),
        steps: vec![
            WorkflowStep {
                id: "system_compliance_audit".to_string(),
                step_type: "full_compliance_audit".to_string(),
                parameters: json!({
                    "audit_scope": "complete_system",
                    "compliance_frameworks": [
                        "internal_policies",
                        "regulatory_requirements",
                        "industry_standards",
                        "risk_management_policies"
                    ],
                    "audit_depth": "comprehensive",
                    "automated_testing": true
                }),
            },
            WorkflowStep {
                id: "policy_optimization".to_string(),
                step_type: "compliance_policy_optimization".to_string(),
                parameters: json!({
                    "optimization_objectives": [
                        "risk_reduction",
                        "operational_efficiency",
                        "regulatory_alignment",
                        "cost_effectiveness"
                    ],
                    "machine_learning_insights": true,
                    "stakeholder_requirements": true
                }),
            },
            WorkflowStep {
                id: "continuous_monitoring_setup".to_string(),
                step_type: "compliance_monitoring_enhancement".to_string(),
                parameters: json!({
                    "monitoring_frequency": "real_time",
                    "alert_thresholds": "dynamic",
                    "predictive_compliance": true,
                    "adaptive_rules": true
                }),
            },
        ],
    };
    
    println!("📋 Executing comprehensive compliance management...");
    let result = orchestrator.execute_workflow(comprehensive_compliance).await.unwrap();
    assert!(matches!(result.status, WorkflowStatus::Completed));
    
    let stats = orchestrator.get_statistics().await;
    println!("📋 Compliance Management Statistics:");
    println!("   Compliance Scenarios Tested: {}", compliance_scenarios.len());
    println!("   Total Workflows Executed: {}", compliance_scenarios.len() + 1);
    println!("   Compliance Services: {}", compliance_services.len());
    println!("   Operations Coordinated: {}", stats.coordinated_operations);
    
    println!("✅ Demo: Rule Violation Handling and Compliance completed successfully\n");
}

/// Demo: Economic Operations and Optimization
/// Demonstrates sophisticated economic operations and portfolio optimization
#[tokio::test]
async fn demo_economic_operations() {
    println!("\n=== DEMO: Economic Operations and Optimization ===");
    
    let config = OrchestratorConfig {
        name: "economic-optimizer".to_string(),
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 3000,
            max_tasks_per_iteration: 10,
            task_timeout_ms: 60000,
            enable_learning: true,
            rules_config: RulesConfig {
                enabled: true,
                fail_fast: false,
                max_daily_spending: 200000.0,
                min_balance_threshold: 25000.0,
                max_risk_score: 0.85,
            },
            ai_config: AiConfig {
                enabled: true,
                max_agents: 8,
                agent_queue_size: 150,
                learning_retention_days: 45,
            },
        },
        qudag: QuDAGConfig {
            enabled: true,
            node_endpoint: "localhost:7700".to_string(),
            network_id: "economic-ops-network".to_string(),
            node_id: "economic-optimizer-001".to_string(),
            bootstrap_peers: vec![
                "localhost:7701".to_string(),
                "localhost:7702".to_string(),
            ],
            connection_timeout_ms: 20000,
            max_reconnection_attempts: 3,
            participate_in_consensus: true,
            exchange_config: ExchangeConfig {
                enabled: true,
                endpoint: "localhost:8700".to_string(),
                trading_pairs: vec![
                    "rUv/USD".to_string(),
                    "rUv/BTC".to_string(),
                    "rUv/ETH".to_string(),
                    "rUv/USDC".to_string(),
                    "rUv/DAI".to_string(),
                ],
                order_book_depth: 200,
            },
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    println!("✓ Economic operations orchestrator initialized");
    
    // Register economic optimization services
    let economic_services = vec![
        Service {
            id: "portfolio-optimizer".to_string(),
            name: "Advanced Portfolio Optimizer".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9800".to_string(),
        },
        Service {
            id: "market-maker".to_string(),
            name: "Automated Market Maker".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9801".to_string(),
        },
        Service {
            id: "yield-maximizer".to_string(),
            name: "Yield Maximization Engine".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9802".to_string(),
        },
        Service {
            id: "risk-optimizer".to_string(),
            name: "Risk-Return Optimizer".to_string(),
            service_type: "ai_agent".to_string(),
            endpoint: "localhost:9803".to_string(),
        },
        Service {
            id: "economic-analytics".to_string(),
            name: "Economic Analytics Engine".to_string(),
            service_type: "analytics".to_string(),
            endpoint: "localhost:9804".to_string(),
        },
        Service {
            id: "execution-optimizer".to_string(),
            name: "Trade Execution Optimizer".to_string(),
            service_type: "execution".to_string(),
            endpoint: "localhost:9805".to_string(),
        },
    ];
    
    for service in &economic_services {
        orchestrator.register_service(service.clone()).await.unwrap();
        println!("✓ Registered service: {}", service.name);
    }
    
    // Execute comprehensive economic operations workflow
    let economic_workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Advanced Economic Operations and Optimization".to_string(),
        steps: vec![
            WorkflowStep {
                id: "economic_environment_analysis".to_string(),
                step_type: "macro_economic_analysis".to_string(),
                parameters: json!({
                    "analysis_scope": "global_macro",
                    "data_sources": [
                        "central_bank_policies",
                        "economic_indicators",
                        "market_sentiment",
                        "geopolitical_events",
                        "crypto_market_dynamics"
                    ],
                    "prediction_models": ["lstm", "transformer", "ensemble"],
                    "time_horizons": ["1h", "4h", "24h", "7d", "30d"]
                }),
            },
            WorkflowStep {
                id: "portfolio_universe_analysis".to_string(),
                step_type: "investment_universe_screening".to_string(),
                parameters: json!({
                    "screening_criteria": {
                        "liquidity_minimum": 1000000,
                        "volatility_maximum": 0.8,
                        "correlation_threshold": 0.7,
                        "sharpe_ratio_minimum": 0.5
                    },
                    "asset_classes": ["cryptocurrencies", "defi_tokens", "stablecoins"],
                    "dynamic_screening": true,
                    "esg_filtering": true
                }),
            },
            WorkflowStep {
                id: "multi_objective_optimization".to_string(),
                step_type: "advanced_portfolio_optimization".to_string(),
                parameters: json!({
                    "optimization_objectives": [
                        "maximize_sharpe_ratio",
                        "minimize_maximum_drawdown",
                        "maximize_diversification",
                        "minimize_transaction_costs"
                    ],
                    "optimization_method": "pareto_frontier",
                    "constraints": {
                        "budget_constraint": 1.0,
                        "position_limits": {"min": 0.01, "max": 0.3},
                        "sector_limits": {"max": 0.4},
                        "liquidity_constraints": true
                    },
                    "robust_optimization": true
                }),
            },
            WorkflowStep {
                id: "risk_budgeting".to_string(),
                step_type: "dynamic_risk_budgeting".to_string(),
                parameters: json!({
                    "risk_budgeting_method": "equal_risk_contribution",
                    "risk_factors": [
                        "market_risk",
                        "credit_risk",
                        "liquidity_risk",
                        "operational_risk"
                    ],
                    "dynamic_adjustment": true,
                    "stress_testing": {
                        "scenarios": ["market_crash", "liquidity_crisis", "regulatory_shock"],
                        "monte_carlo_simulations": 10000
                    }
                }),
            },
            WorkflowStep {
                id: "yield_optimization".to_string(),
                step_type: "multi_strategy_yield_optimization".to_string(),
                parameters: json!({
                    "yield_strategies": [
                        "liquidity_provision",
                        "staking_rewards",
                        "lending_protocols",
                        "yield_farming",
                        "governance_rewards"
                    ],
                    "optimization_horizon": "dynamic",
                    "compound_frequency": "continuous",
                    "tax_optimization": true,
                    "gas_cost_optimization": true
                }),
            },
            WorkflowStep {
                id: "smart_execution".to_string(),
                step_type: "intelligent_execution_optimization".to_string(),
                parameters: json!({
                    "execution_algorithms": [
                        "adaptive_twap",
                        "implementation_shortfall",
                        "participation_rate",
                        "market_on_close"
                    ],
                    "market_impact_modeling": "advanced",
                    "timing_optimization": true,
                    "venue_selection": "optimal",
                    "slippage_prediction": true
                }),
            },
            WorkflowStep {
                id: "dynamic_rebalancing".to_string(),
                step_type: "adaptive_rebalancing".to_string(),
                parameters: json!({
                    "rebalancing_triggers": [
                        "drift_threshold",
                        "volatility_regime_change",
                        "market_momentum_shift",
                        "correlation_breakdown"
                    ],
                    "rebalancing_frequency": "adaptive",
                    "cost_benefit_analysis": true,
                    "tax_loss_harvesting": true,
                    "market_timing": "moderate"
                }),
            },
            WorkflowStep {
                id: "performance_attribution".to_string(),
                step_type: "advanced_performance_attribution".to_string(),
                parameters: json!({
                    "attribution_models": ["brinson", "factor_based", "holdings_based"],
                    "risk_attribution": true,
                    "transaction_cost_analysis": true,
                    "benchmark_comparison": "multiple_benchmarks",
                    "peer_analysis": true
                }),
            },
            WorkflowStep {
                id: "economic_learning".to_string(),
                step_type: "economic_intelligence_learning".to_string(),
                parameters: json!({
                    "learning_sources": [
                        "market_microstructure",
                        "behavioral_patterns",
                        "regime_changes",
                        "alpha_decay",
                        "execution_quality"
                    ],
                    "model_adaptation": "online_learning",
                    "ensemble_updating": true,
                    "performance_feedback": "continuous"
                }),
            },
        ],
    };
    
    println!("💰 Executing advanced economic operations workflow...");
    let result = orchestrator.execute_workflow(economic_workflow.clone()).await.unwrap();
    
    assert!(matches!(result.status, WorkflowStatus::Completed));
    println!("✅ Economic operations workflow completed successfully");
    
    // Simulate economic operations time
    sleep(Duration::from_millis(1000)).await;
    
    let stats = orchestrator.get_statistics().await;
    println!("💰 Economic Operations Statistics:");
    println!("   Portfolio Optimization Steps: 9");
    println!("   Economic Services: {}", economic_services.len());
    println!("   Total Operations: {}", stats.coordinated_operations);
    println!("   Processing Node: {}", stats.node_id);
    
    println!("✅ Demo: Economic Operations and Optimization completed successfully\n");
}

/// Integration demo showing all systems working together
#[tokio::test]
async fn demo_full_system_integration() {
    println!("\n=== DEMO: Full System Integration ===");
    
    let config = OrchestratorConfig {
        name: "full-system-demo".to_string(),
        autonomy: AutonomyConfig {
            enabled: true,
            loop_interval_ms: 2000,
            max_tasks_per_iteration: 15,
            task_timeout_ms: 90000,
            enable_learning: true,
            rules_config: RulesConfig {
                enabled: true,
                fail_fast: false,
                max_daily_spending: 500000.0,
                min_balance_threshold: 50000.0,
                max_risk_score: 0.8,
            },
            ai_config: AiConfig {
                enabled: true,
                max_agents: 15,
                agent_queue_size: 200,
                learning_retention_days: 90,
            },
        },
        qudag: QuDAGConfig {
            enabled: true,
            node_endpoint: "localhost:7800".to_string(),
            network_id: "full-integration-network".to_string(),
            node_id: "full-system-demo-001".to_string(),
            bootstrap_peers: vec![
                "localhost:7801".to_string(),
                "localhost:7802".to_string(),
                "localhost:7803".to_string(),
            ],
            connection_timeout_ms: 30000,
            max_reconnection_attempts: 5,
            participate_in_consensus: true,
            exchange_config: ExchangeConfig {
                enabled: true,
                endpoint: "localhost:8800".to_string(),
                trading_pairs: vec![
                    "rUv/USD".to_string(),
                    "rUv/BTC".to_string(),
                    "rUv/ETH".to_string(),
                    "rUv/USDC".to_string(),
                    "rUv/DAI".to_string(),
                ],
                order_book_depth: 500,
            },
        },
        mcp: McpConfig {
            enabled: true,
            port: 3800,
            max_connections: 100,
            request_timeout_ms: 60000,
            enable_auth: false,
            api_key: None,
            ..Default::default()
        },
        api: ApiConfig {
            enabled: true,
            port: 3801,
            max_connections: 100,
            request_timeout_ms: 60000,
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };
    
    let mut orchestrator = DaaOrchestrator::new(config).await.unwrap();
    orchestrator.initialize().await.unwrap();
    
    println!("✓ Full system integration orchestrator initialized");
    
    // Register comprehensive service ecosystem
    let all_services = vec![
        // AI Agents
        Service { id: "chief-ai-strategist".to_string(), name: "Chief AI Strategist".to_string(), service_type: "ai_agent".to_string(), endpoint: "localhost:9900".to_string() },
        Service { id: "portfolio-manager".to_string(), name: "Portfolio Manager AI".to_string(), service_type: "ai_agent".to_string(), endpoint: "localhost:9901".to_string() },
        Service { id: "risk-manager".to_string(), name: "Risk Manager AI".to_string(), service_type: "ai_agent".to_string(), endpoint: "localhost:9902".to_string() },
        Service { id: "compliance-officer".to_string(), name: "Compliance Officer AI".to_string(), service_type: "ai_agent".to_string(), endpoint: "localhost:9903".to_string() },
        Service { id: "market-analyst".to_string(), name: "Market Analyst AI".to_string(), service_type: "ai_agent".to_string(), endpoint: "localhost:9904".to_string() },
        // Rules Engines
        Service { id: "master-rules-engine".to_string(), name: "Master Rules Engine".to_string(), service_type: "rules_engine".to_string(), endpoint: "localhost:9905".to_string() },
        Service { id: "compliance-rules".to_string(), name: "Compliance Rules Engine".to_string(), service_type: "rules_engine".to_string(), endpoint: "localhost:9906".to_string() },
        // Infrastructure
        Service { id: "blockchain-gateway".to_string(), name: "Blockchain Gateway".to_string(), service_type: "blockchain_bridge".to_string(), endpoint: "localhost:9907".to_string() },
        Service { id: "data-aggregator".to_string(), name: "Data Aggregation Service".to_string(), service_type: "data_provider".to_string(), endpoint: "localhost:9908".to_string() },
        Service { id: "execution-engine".to_string(), name: "Trade Execution Engine".to_string(), service_type: "execution".to_string(), endpoint: "localhost:9909".to_string() },
    ];
    
    for service in &all_services {
        orchestrator.register_service(service.clone()).await.unwrap();
        println!("✓ Registered: {}", service.name);
    }
    
    // Execute comprehensive integration workflow
    let integration_workflow = Workflow {
        id: Uuid::new_v4().to_string(),
        name: "Full System Integration Demonstration".to_string(),
        steps: vec![
            WorkflowStep {
                id: "system_health_check".to_string(),
                step_type: "comprehensive_health_check".to_string(),
                parameters: json!({
                    "check_components": ["orchestrator", "autonomy_loop", "services", "integrations"],
                    "health_metrics": ["performance", "availability", "compliance", "security"],
                    "automated_remediation": true
                }),
            },
            WorkflowStep {
                id: "strategic_planning".to_string(),
                step_type: "ai_strategic_planning".to_string(),
                parameters: json!({
                    "planning_horizon": "24h",
                    "strategic_objectives": ["risk_adjusted_returns", "compliance", "efficiency"],
                    "ai_collaboration": true,
                    "human_oversight": "minimal"
                }),
            },
            WorkflowStep {
                id: "market_intelligence".to_string(),
                step_type: "comprehensive_market_analysis".to_string(),
                parameters: json!({
                    "intelligence_sources": ["onchain", "offchain", "sentiment", "technical"],
                    "ai_processing": "advanced",
                    "real_time_updates": true,
                    "predictive_modeling": true
                }),
            },
            WorkflowStep {
                id: "portfolio_optimization".to_string(),
                step_type: "holistic_portfolio_optimization".to_string(),
                parameters: json!({
                    "optimization_scope": "complete_portfolio",
                    "ai_coordination": true,
                    "rules_compliance": "strict",
                    "performance_targets": "dynamic"
                }),
            },
            WorkflowStep {
                id: "execution_coordination".to_string(),
                step_type: "coordinated_execution".to_string(),
                parameters: json!({
                    "execution_strategy": "intelligent",
                    "market_impact_minimization": true,
                    "cost_optimization": true,
                    "timing_optimization": true
                }),
            },
            WorkflowStep {
                id: "continuous_monitoring".to_string(),
                step_type: "real_time_monitoring".to_string(),
                parameters: json!({
                    "monitoring_scope": "full_system",
                    "alert_sensitivity": "adaptive",
                    "automated_responses": true,
                    "learning_integration": true
                }),
            },
            WorkflowStep {
                id: "performance_evaluation".to_string(),
                step_type: "comprehensive_performance_evaluation".to_string(),
                parameters: json!({
                    "evaluation_scope": "holistic",
                    "benchmarking": "multi_dimensional",
                    "attribution_analysis": "complete",
                    "improvement_recommendations": true
                }),
            },
            WorkflowStep {
                id: "system_learning".to_string(),
                step_type: "integrated_system_learning".to_string(),
                parameters: json!({
                    "learning_scope": "system_wide",
                    "knowledge_integration": "cross_component",
                    "model_updates": "coordinated",
                    "performance_optimization": "continuous"
                }),
            },
        ],
    };
    
    println!("🌟 Executing full system integration workflow...");
    let result = orchestrator.execute_workflow(integration_workflow.clone()).await.unwrap();
    
    assert!(matches!(result.status, WorkflowStatus::Completed));
    println!("✅ Full system integration workflow completed successfully");
    
    let stats = orchestrator.get_statistics().await;
    println!("🌟 Full System Integration Statistics:");
    println!("   Total Services Registered: {}", all_services.len());
    println!("   Integration Components: 8");
    println!("   Workflow Complexity: High");
    println!("   System Health: Operational");
    println!("   Node ID: {}", stats.node_id);
    
    println!("✅ Demo: Full System Integration completed successfully\n");
    
    println!("🎉 ALL DEMO SCENARIOS COMPLETED SUCCESSFULLY! 🎉");
    println!("The DAA SDK has demonstrated comprehensive capabilities across:");
    println!("  ✓ Autonomous Treasury Management");
    println!("  ✓ Multi-Agent DeFi Coordination");
    println!("  ✓ Rule Violation Handling and Compliance");
    println!("  ✓ Economic Operations and Optimization");
    println!("  ✓ Full System Integration");
}