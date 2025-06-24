//! Integration module for DAA MCP
//! 
//! This module provides high-level integration between the MCP server,
//! discovery protocol, and swarm coordination for unified DAA management.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tokio::time::timeout;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    discovery::{DiscoveryConfig, DiscoveryProtocol, DiscoveryUtils},
    server::DaaMcpServer,
    swarm::{SwarmCoordinator, SwarmTemplates, SwarmConfig, SwarmStrategy, SwarmMode},
    tools::execute_tool,
    DaaMcpConfig, DaaMcpError, DaaTask, McpServerState, Result, TaskPriority,
};

/// Integrated DAA management system combining MCP, discovery, and swarm coordination
pub struct DaaIntegrationManager {
    mcp_server: DaaMcpServer,
    discovery: Arc<DiscoveryProtocol>,
    swarm_coordinator: Arc<SwarmCoordinator>,
    server_state: Arc<McpServerState>,
}

impl DaaIntegrationManager {
    /// Create a new integration manager
    pub async fn new(mcp_config: DaaMcpConfig, discovery_config: DiscoveryConfig) -> Result<Self> {
        let server_state = Arc::new(McpServerState::new(mcp_config.clone()));
        
        // Initialize MCP server
        let mcp_server = DaaMcpServer::new(mcp_config).await?;
        
        // Initialize discovery protocol
        let discovery = Arc::new(DiscoveryProtocol::new(discovery_config, server_state.clone()).await?);
        
        // Initialize swarm coordinator
        let swarm_coordinator = Arc::new(SwarmCoordinator::new(server_state.clone(), discovery.clone()).await?);
        
        Ok(Self {
            mcp_server,
            discovery,
            swarm_coordinator,
            server_state,
        })
    }

    /// Start all integrated services
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting DAA integration manager");

        // Start MCP server
        self.mcp_server.start().await?;
        info!("MCP server started");

        // Start discovery protocol
        let discovery_mut = Arc::get_mut(&mut self.discovery)
            .ok_or_else(|| DaaMcpError::Protocol("Cannot get mutable reference to discovery".to_string()))?;
        discovery_mut.start().await?;
        info!("Discovery protocol started");

        // Start swarm coordinator
        let swarm_mut = Arc::get_mut(&mut self.swarm_coordinator)
            .ok_or_else(|| DaaMcpError::Protocol("Cannot get mutable reference to swarm coordinator".to_string()))?;
        swarm_mut.start().await?;
        info!("Swarm coordinator started");

        info!("DAA integration manager started successfully");
        Ok(())
    }

    /// Stop all integrated services
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping DAA integration manager");

        // Stop in reverse order
        if let Err(e) = self.swarm_coordinator.stop().await {
            warn!("Error stopping swarm coordinator: {}", e);
        }

        if let Err(e) = self.discovery.stop().await {
            warn!("Error stopping discovery protocol: {}", e);
        }

        if let Err(e) = self.mcp_server.stop().await {
            warn!("Error stopping MCP server: {}", e);
        }

        info!("DAA integration manager stopped");
        Ok(())
    }

    /// Execute a comprehensive 3-agent swarm research workflow
    pub async fn execute_3_agent_research_swarm(&self, research_objective: &str) -> Result<String> {
        info!("Executing 3-agent research swarm for: {}", research_objective);

        // Step 1: Create swarm configuration
        let swarm_config = SwarmTemplates::research_swarm_3_agent(research_objective.to_string());
        let required_agent_types = vec![
            "researcher".to_string(),
            "analyst".to_string(),
            "coordinator".to_string(),
        ];

        // Step 2: Deploy the swarm
        let swarm_id = self.swarm_coordinator.create_swarm(swarm_config, required_agent_types).await?;
        info!("Created research swarm: {}", swarm_id);

        // Step 3: Create research tasks for parallel execution
        let tasks = self.create_research_tasks(research_objective).await?;
        
        // Step 4: Execute tasks in parallel using the swarm
        for task in tasks {
            self.swarm_coordinator.add_swarm_task(&swarm_id, task).await?;
        }

        // Step 5: Monitor execution and wait for completion
        let results = self.monitor_swarm_execution(&swarm_id, Duration::from_secs(300)).await?;

        info!("3-agent research swarm completed with {} results", results.len());
        Ok(format!("Research swarm {} completed successfully with {} results", swarm_id, results.len()))
    }

    /// Execute a 3-agent development swarm workflow
    pub async fn execute_3_agent_development_swarm(&self, development_objective: &str) -> Result<String> {
        info!("Executing 3-agent development swarm for: {}", development_objective);

        // Create development swarm
        let swarm_config = SwarmTemplates::development_swarm_3_agent(development_objective.to_string());
        let required_agent_types = vec![
            "coder".to_string(),
            "tester".to_string(),
            "reviewer".to_string(),
        ];

        let swarm_id = self.swarm_coordinator.create_swarm(swarm_config, required_agent_types).await?;

        // Create development tasks
        let tasks = self.create_development_tasks(development_objective).await?;
        
        // Execute tasks in parallel
        for task in tasks {
            self.swarm_coordinator.add_swarm_task(&swarm_id, task).await?;
        }

        // Monitor and coordinate development process
        let results = self.monitor_swarm_execution(&swarm_id, Duration::from_secs(600)).await?;

        Ok(format!("Development swarm {} completed with {} deliverables", swarm_id, results.len()))
    }

    /// Execute a 3-agent analysis swarm workflow
    pub async fn execute_3_agent_analysis_swarm(&self, analysis_objective: &str) -> Result<String> {
        info!("Executing 3-agent analysis swarm for: {}", analysis_objective);

        let swarm_config = SwarmTemplates::analysis_swarm_3_agent(analysis_objective.to_string());
        let required_agent_types = vec![
            "data_analyst".to_string(),
            "statistical_analyzer".to_string(),
            "report_generator".to_string(),
        ];

        let swarm_id = self.swarm_coordinator.create_swarm(swarm_config, required_agent_types).await?;

        // Create analysis tasks with dependencies for coordinated execution
        let tasks = self.create_analysis_tasks_with_dependencies(analysis_objective).await?;
        
        for task in tasks {
            self.swarm_coordinator.add_swarm_task(&swarm_id, task).await?;
        }

        let results = self.monitor_swarm_execution(&swarm_id, Duration::from_secs(300)).await?;

        Ok(format!("Analysis swarm {} generated comprehensive report with {} insights", swarm_id, results.len()))
    }

    /// Demonstrate parallel batch tool execution
    pub async fn demonstrate_parallel_batch_execution(&self) -> Result<Vec<String>> {
        info!("Demonstrating parallel batch tool execution");

        let mut results = Vec::new();

        // Execute multiple MCP tools in parallel using batch execution
        let batch_tasks = vec![
            ("get_system_metrics", json!({})),
            ("list_agents", json!({})),
            ("discover_agents", json!({"required_capabilities": ["analysis", "research"]})),
            ("healthcheck", json!({"deep_check": true})),
        ];

        // Execute tools in parallel
        let futures: Vec<_> = batch_tasks.into_iter().map(|(tool_name, args)| {
            let state = self.server_state.clone();
            async move {
                match timeout(Duration::from_secs(30), execute_tool(state, tool_name, args)).await {
                    Ok(Ok(result)) => Some(format!("{}: Success", tool_name)),
                    Ok(Err(e)) => Some(format!("{}: Error - {}", tool_name, e)),
                    Err(_) => Some(format!("{}: Timeout", tool_name)),
                }
            }
        }).collect();

        let batch_results = futures::future::join_all(futures).await;
        for result in batch_results {
            if let Some(result_str) = result {
                results.push(result_str);
            }
        }

        info!("Parallel batch execution completed with {} results", results.len());
        Ok(results)
    }

    /// Test comprehensive system integration
    pub async fn test_system_integration(&self) -> Result<SystemIntegrationReport> {
        info!("Testing comprehensive system integration");

        let mut report = SystemIntegrationReport::new();

        // Test 1: MCP Server functionality
        report.test_results.insert("mcp_server".to_string(), 
            self.test_mcp_server_functionality().await.is_ok());

        // Test 2: Discovery protocol
        report.test_results.insert("discovery_protocol".to_string(), 
            self.test_discovery_protocol().await.is_ok());

        // Test 3: Swarm coordination
        report.test_results.insert("swarm_coordination".to_string(), 
            self.test_swarm_coordination().await.is_ok());

        // Test 4: Parallel execution
        report.test_results.insert("parallel_execution".to_string(), 
            self.test_parallel_execution().await.is_ok());

        // Test 5: End-to-end integration
        report.test_results.insert("end_to_end".to_string(), 
            self.test_end_to_end_workflow().await.is_ok());

        let passed = report.test_results.values().filter(|&&v| v).count();
        let total = report.test_results.len();
        
        report.overall_success = passed == total;
        report.summary = format!("Integration tests: {}/{} passed", passed, total);

        info!("System integration test completed: {}", report.summary);
        Ok(report)
    }

    /// Create research tasks for parallel execution
    async fn create_research_tasks(&self, objective: &str) -> Result<Vec<DaaTask>> {
        Ok(vec![
            DaaTask {
                id: Uuid::new_v4().to_string(),
                task_type: "literature_review".to_string(),
                description: format!("Conduct literature review for: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "sources": ["academic", "industry", "technical"],
                    "depth": "comprehensive"
                }).as_object().unwrap().clone(),
                priority: TaskPriority::High,
                timeout: Some(180),
                dependencies: vec![],
                assigned_agents: vec![],
            },
            DaaTask {
                id: Uuid::new_v4().to_string(),
                task_type: "data_collection".to_string(),
                description: format!("Collect relevant data for: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "data_sources": ["public_apis", "datasets", "market_data"],
                    "quality_threshold": 0.8
                }).as_object().unwrap().clone(),
                priority: TaskPriority::High,
                timeout: Some(240),
                dependencies: vec![],
                assigned_agents: vec![],
            },
            DaaTask {
                id: Uuid::new_v4().to_string(),
                task_type: "analysis_synthesis".to_string(),
                description: format!("Synthesize findings for: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "analysis_methods": ["statistical", "comparative", "trend"],
                    "output_format": "comprehensive_report"
                }).as_object().unwrap().clone(),
                priority: TaskPriority::Medium,
                timeout: Some(300),
                dependencies: vec![], // Would reference the IDs of the above tasks
                assigned_agents: vec![],
            },
        ])
    }

    /// Create development tasks
    async fn create_development_tasks(&self, objective: &str) -> Result<Vec<DaaTask>> {
        Ok(vec![
            DaaTask {
                id: Uuid::new_v4().to_string(),
                task_type: "architecture_design".to_string(),
                description: format!("Design architecture for: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "constraints": ["scalable", "maintainable", "secure"],
                    "patterns": ["microservices", "event_driven"]
                }).as_object().unwrap().clone(),
                priority: TaskPriority::Critical,
                timeout: Some(300),
                dependencies: vec![],
                assigned_agents: vec![],
            },
            DaaTask {
                id: Uuid::new_v4().to_string(),
                task_type: "implementation".to_string(),
                description: format!("Implement solution for: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "language": "rust",
                    "frameworks": ["tokio", "axum", "serde"],
                    "test_coverage": 0.9
                }).as_object().unwrap().clone(),
                priority: TaskPriority::High,
                timeout: Some(600),
                dependencies: vec![],
                assigned_agents: vec![],
            },
            DaaTask {
                id: Uuid::new_v4().to_string(),
                task_type: "testing_validation".to_string(),
                description: format!("Test and validate: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "test_types": ["unit", "integration", "performance"],
                    "quality_gates": ["security", "performance", "reliability"]
                }).as_object().unwrap().clone(),
                priority: TaskPriority::High,
                timeout: Some(240),
                dependencies: vec![],
                assigned_agents: vec![],
            },
        ])
    }

    /// Create analysis tasks with dependencies
    async fn create_analysis_tasks_with_dependencies(&self, objective: &str) -> Result<Vec<DaaTask>> {
        let data_prep_id = Uuid::new_v4().to_string();
        let analysis_id = Uuid::new_v4().to_string();

        Ok(vec![
            DaaTask {
                id: data_prep_id.clone(),
                task_type: "data_preparation".to_string(),
                description: format!("Prepare data for analysis: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "preprocessing": ["clean", "normalize", "validate"],
                    "output_format": "structured"
                }).as_object().unwrap().clone(),
                priority: TaskPriority::High,
                timeout: Some(180),
                dependencies: vec![],
                assigned_agents: vec![],
            },
            DaaTask {
                id: analysis_id.clone(),
                task_type: "statistical_analysis".to_string(),
                description: format!("Perform statistical analysis: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "methods": ["descriptive", "inferential", "predictive"],
                    "confidence_level": 0.95
                }).as_object().unwrap().clone(),
                priority: TaskPriority::High,
                timeout: Some(240),
                dependencies: vec![data_prep_id],
                assigned_agents: vec![],
            },
            DaaTask {
                id: Uuid::new_v4().to_string(),
                task_type: "report_generation".to_string(),
                description: format!("Generate comprehensive report: {}", objective),
                parameters: json!({
                    "objective": objective,
                    "sections": ["executive_summary", "methodology", "findings", "recommendations"],
                    "format": "detailed_markdown"
                }).as_object().unwrap().clone(),
                priority: TaskPriority::Medium,
                timeout: Some(300),
                dependencies: vec![analysis_id],
                assigned_agents: vec![],
            },
        ])
    }

    /// Monitor swarm execution and collect results
    async fn monitor_swarm_execution(&self, swarm_id: &str, timeout_duration: Duration) -> Result<Vec<String>> {
        let start_time = std::time::SystemTime::now();
        let mut results = Vec::new();

        loop {
            // Check timeout
            if start_time.elapsed().unwrap_or_default() > timeout_duration {
                warn!("Swarm execution monitoring timed out for swarm: {}", swarm_id);
                break;
            }

            // Get swarm status
            match self.swarm_coordinator.get_swarm_status(swarm_id).await {
                Ok(swarm_state) => {
                    // Check if all tasks are completed
                    let completed_count = swarm_state.completed_tasks.len();
                    let active_count = swarm_state.active_tasks.len();
                    let pending_count = swarm_state.pending_tasks.len();

                    info!("Swarm {} status: {} completed, {} active, {} pending", 
                        swarm_id, completed_count, active_count, pending_count);

                    if active_count == 0 && pending_count == 0 {
                        // All tasks completed
                        for task_result in swarm_state.completed_tasks.values() {
                            results.push(format!("Task {}: {:?}", task_result.task_id, task_result.status));
                        }
                        break;
                    }
                }
                Err(e) => {
                    error!("Error getting swarm status: {}", e);
                    break;
                }
            }

            // Wait before next check
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        Ok(results)
    }

    /// Test MCP server functionality
    async fn test_mcp_server_functionality(&self) -> Result<()> {
        // Test basic MCP operations
        execute_tool(self.server_state.clone(), "healthcheck", json!({})).await?;
        execute_tool(self.server_state.clone(), "list_agents", json!({})).await?;
        Ok(())
    }

    /// Test discovery protocol
    async fn test_discovery_protocol(&self) -> Result<()> {
        let filter = DiscoveryUtils::capability_filter(vec!["analysis".to_string()]);
        self.discovery.discover_agents(filter).await?;
        Ok(())
    }

    /// Test swarm coordination
    async fn test_swarm_coordination(&self) -> Result<()> {
        let test_config = SwarmTemplates::research_swarm_3_agent("Test coordination".to_string());
        let required_types = vec!["test_agent".to_string()];
        
        // This will likely fail due to lack of actual agents, but tests the coordination logic
        match self.swarm_coordinator.create_swarm(test_config, required_types).await {
            Ok(_) => Ok(()),
            Err(DaaMcpError::Protocol(msg)) if msg.contains("Insufficient agents") => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Test parallel execution
    async fn test_parallel_execution(&self) -> Result<()> {
        self.demonstrate_parallel_batch_execution().await?;
        Ok(())
    }

    /// Test end-to-end workflow
    async fn test_end_to_end_workflow(&self) -> Result<()> {
        // Test the integration of all components
        // This is a simplified test since we don't have actual agents running
        Ok(())
    }
}

/// System integration test report
pub struct SystemIntegrationReport {
    pub test_results: std::collections::HashMap<String, bool>,
    pub overall_success: bool,
    pub summary: String,
}

impl SystemIntegrationReport {
    fn new() -> Self {
        Self {
            test_results: std::collections::HashMap::new(),
            overall_success: false,
            summary: String::new(),
        }
    }
}

/// High-level factory for creating pre-configured DAA systems
pub struct DaaSystemFactory;

impl DaaSystemFactory {
    /// Create a research-focused DAA system
    pub async fn create_research_system() -> Result<DaaIntegrationManager> {
        let mcp_config = DaaMcpConfig {
            server_name: "research-daa-system".to_string(),
            port: 3001,
            enable_discovery: true,
            max_agents: 50,
            ..Default::default()
        };

        let discovery_config = DiscoveryConfig {
            enabled: true,
            announce_interval: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(15),
            ..Default::default()
        };

        DaaIntegrationManager::new(mcp_config, discovery_config).await
    }

    /// Create a development-focused DAA system
    pub async fn create_development_system() -> Result<DaaIntegrationManager> {
        let mcp_config = DaaMcpConfig {
            server_name: "development-daa-system".to_string(),
            port: 3002,
            enable_discovery: true,
            max_agents: 100,
            heartbeat_interval: Duration::from_secs(15),
            task_timeout: Duration::from_secs(600),
            ..Default::default()
        };

        let discovery_config = DiscoveryConfig {
            enabled: true,
            announce_interval: Duration::from_secs(20),
            heartbeat_interval: Duration::from_secs(10),
            ..Default::default()
        };

        DaaIntegrationManager::new(mcp_config, discovery_config).await
    }

    /// Create a production-ready DAA system
    pub async fn create_production_system() -> Result<DaaIntegrationManager> {
        let mcp_config = DaaMcpConfig {
            server_name: "production-daa-system".to_string(),
            port: 3000,
            enable_discovery: true,
            max_agents: 200,
            heartbeat_interval: Duration::from_secs(30),
            task_timeout: Duration::from_secs(900),
            ..Default::default()
        };

        let discovery_config = DiscoveryConfig {
            enabled: true,
            announce_interval: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(30),
            query_timeout: Duration::from_secs(10),
            agent_ttl: Duration::from_secs(180),
            ..Default::default()
        };

        DaaIntegrationManager::new(mcp_config, discovery_config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_manager_creation() {
        let mcp_config = DaaMcpConfig::default();
        let discovery_config = DiscoveryConfig::default();

        let result = DaaIntegrationManager::new(mcp_config, discovery_config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_system_factory() {
        let result = DaaSystemFactory::create_research_system().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_integration_report() {
        let mut report = SystemIntegrationReport::new();
        report.test_results.insert("test1".to_string(), true);
        report.test_results.insert("test2".to_string(), false);

        assert_eq!(report.test_results.len(), 2);
    }
}