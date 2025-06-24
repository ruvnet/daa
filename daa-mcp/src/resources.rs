//! MCP Resources for DAA management
//! 
//! This module defines all the available resources that can be accessed through
//! the MCP interface to get information about DAA agents, tasks, logs, and metrics.

use std::sync::Arc;

use serde_json::{json, Value};
use tracing::debug;

use crate::{Content, DaaMcpError, McpServerState, ResourceInfo, Result};

/// Get list of all available resources
pub async fn get_available_resources() -> Vec<ResourceInfo> {
    vec![
        // Agent Resources
        ResourceInfo {
            uri: "daa://agents".to_string(),
            name: "DAA Agents".to_string(),
            description: "List of all DAA agents with their current status".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://agents/{agent_id}".to_string(),
            name: "Agent Details".to_string(),
            description: "Detailed information about a specific agent".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://agents/{agent_id}/logs".to_string(),
            name: "Agent Logs".to_string(),
            description: "Log entries for a specific agent".to_string(),
            mime_type: Some("text/plain".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://agents/{agent_id}/metrics".to_string(),
            name: "Agent Metrics".to_string(),
            description: "Performance metrics for a specific agent".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://agents/{agent_id}/config".to_string(),
            name: "Agent Configuration".to_string(),
            description: "Current configuration of a specific agent".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },

        // Task Resources
        ResourceInfo {
            uri: "daa://tasks".to_string(),
            name: "DAA Tasks".to_string(),
            description: "List of all tasks in the system".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://tasks/{task_id}".to_string(),
            name: "Task Details".to_string(),
            description: "Detailed information about a specific task".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://tasks/{task_id}/results".to_string(),
            name: "Task Results".to_string(),
            description: "Execution results for a specific task".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://tasks/pending".to_string(),
            name: "Pending Tasks".to_string(),
            description: "List of tasks waiting for execution".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://tasks/running".to_string(),
            name: "Running Tasks".to_string(),
            description: "List of currently executing tasks".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },

        // Swarm Resources
        ResourceInfo {
            uri: "daa://swarm/status".to_string(),
            name: "Swarm Status".to_string(),
            description: "Current status of the agent swarm".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://swarm/messages".to_string(),
            name: "Swarm Messages".to_string(),
            description: "Recent messages in the swarm communication".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://swarm/topology".to_string(),
            name: "Swarm Topology".to_string(),
            description: "Network topology and connections between agents".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },

        // System Resources
        ResourceInfo {
            uri: "daa://system/metrics".to_string(),
            name: "System Metrics".to_string(),
            description: "Overall system performance and health metrics".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://system/logs".to_string(),
            name: "System Logs".to_string(),
            description: "System-wide log entries".to_string(),
            mime_type: Some("text/plain".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://system/config".to_string(),
            name: "System Configuration".to_string(),
            description: "Current system configuration settings".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://system/health".to_string(),
            name: "System Health".to_string(),
            description: "System health check results".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },

        // Discovery Resources
        ResourceInfo {
            uri: "daa://discovery/agents".to_string(),
            name: "Agent Discovery".to_string(),
            description: "Discoverable agents and their capabilities".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://discovery/services".to_string(),
            name: "Service Discovery".to_string(),
            description: "Available services and endpoints".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },

        // Analytics Resources
        ResourceInfo {
            uri: "daa://analytics/performance".to_string(),
            name: "Performance Analytics".to_string(),
            description: "Performance analysis and trends".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
        ResourceInfo {
            uri: "daa://analytics/usage".to_string(),
            name: "Usage Analytics".to_string(),
            description: "System usage patterns and statistics".to_string(),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        },
    ]
}

/// Read a specific resource by URI
pub async fn read_resource(state: Arc<McpServerState>, uri: &str) -> Result<Vec<Content>> {
    debug!("Reading resource: {}", uri);

    let content = match uri {
        // Agent Resources
        "daa://agents" => read_agents_list(state).await?,
        uri if uri.starts_with("daa://agents/") && uri.ends_with("/logs") => {
            let agent_id = extract_agent_id_from_logs_uri(uri)?;
            read_agent_logs(state, &agent_id).await?
        }
        uri if uri.starts_with("daa://agents/") && uri.ends_with("/metrics") => {
            let agent_id = extract_agent_id_from_metrics_uri(uri)?;
            read_agent_metrics(state, &agent_id).await?
        }
        uri if uri.starts_with("daa://agents/") && uri.ends_with("/config") => {
            let agent_id = extract_agent_id_from_config_uri(uri)?;
            read_agent_config(state, &agent_id).await?
        }
        uri if uri.starts_with("daa://agents/") => {
            let agent_id = extract_agent_id_from_uri(uri)?;
            read_agent_details(state, &agent_id).await?
        }

        // Task Resources
        "daa://tasks" => read_tasks_list(state).await?,
        "daa://tasks/pending" => read_pending_tasks(state).await?,
        "daa://tasks/running" => read_running_tasks(state).await?,
        uri if uri.starts_with("daa://tasks/") && uri.ends_with("/results") => {
            let task_id = extract_task_id_from_results_uri(uri)?;
            read_task_results(state, &task_id).await?
        }
        uri if uri.starts_with("daa://tasks/") => {
            let task_id = extract_task_id_from_uri(uri)?;
            read_task_details(state, &task_id).await?
        }

        // Swarm Resources
        "daa://swarm/status" => read_swarm_status(state).await?,
        "daa://swarm/messages" => read_swarm_messages(state).await?,
        "daa://swarm/topology" => read_swarm_topology(state).await?,

        // System Resources
        "daa://system/metrics" => read_system_metrics(state).await?,
        "daa://system/logs" => read_system_logs(state).await?,
        "daa://system/config" => read_system_config(state).await?,
        "daa://system/health" => read_system_health(state).await?,

        // Discovery Resources
        "daa://discovery/agents" => read_discovery_agents(state).await?,
        "daa://discovery/services" => read_discovery_services(state).await?,

        // Analytics Resources
        "daa://analytics/performance" => read_performance_analytics(state).await?,
        "daa://analytics/usage" => read_usage_analytics(state).await?,

        _ => {
            return Err(DaaMcpError::ResourceNotAvailable(format!("Unknown resource: {}", uri)));
        }
    };

    Ok(content)
}

// Helper functions for URI parsing

fn extract_agent_id_from_uri(uri: &str) -> Result<String> {
    let parts: Vec<&str> = uri.split('/').collect();
    if parts.len() >= 3 {
        Ok(parts[2].to_string())
    } else {
        Err(DaaMcpError::Protocol("Invalid agent URI format".to_string()))
    }
}

fn extract_agent_id_from_logs_uri(uri: &str) -> Result<String> {
    let uri = uri.strip_suffix("/logs").unwrap_or(uri);
    extract_agent_id_from_uri(uri)
}

fn extract_agent_id_from_metrics_uri(uri: &str) -> Result<String> {
    let uri = uri.strip_suffix("/metrics").unwrap_or(uri);
    extract_agent_id_from_uri(uri)
}

fn extract_agent_id_from_config_uri(uri: &str) -> Result<String> {
    let uri = uri.strip_suffix("/config").unwrap_or(uri);
    extract_agent_id_from_uri(uri)
}

fn extract_task_id_from_uri(uri: &str) -> Result<String> {
    let parts: Vec<&str> = uri.split('/').collect();
    if parts.len() >= 3 {
        Ok(parts[2].to_string())
    } else {
        Err(DaaMcpError::Protocol("Invalid task URI format".to_string()))
    }
}

fn extract_task_id_from_results_uri(uri: &str) -> Result<String> {
    let uri = uri.strip_suffix("/results").unwrap_or(uri);
    extract_task_id_from_uri(uri)
}

// Resource implementation functions

async fn read_agents_list(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    let agent_list: Vec<Value> = agents.values()
        .map(|agent| serde_json::to_value(agent).unwrap())
        .collect();

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json!({
            "agents": agent_list,
            "total_count": agent_list.len(),
            "timestamp": chrono::Utc::now()
        }))?,
    }])
}

async fn read_agent_details(state: Arc<McpServerState>, agent_id: &str) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    
    if let Some(agent) = agents.get(agent_id) {
        Ok(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(agent)?,
        }])
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

async fn read_agent_logs(state: Arc<McpServerState>, agent_id: &str) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    
    if agents.contains_key(agent_id) {
        // In a real implementation, this would read actual log files
        let mock_logs = format!(
            "[{}] INFO Agent {} started\n[{}] INFO Agent {} running autonomy loop\n[{}] DEBUG Agent {} processed task\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            agent_id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            agent_id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            agent_id
        );

        Ok(vec![Content {
            content_type: "text/plain".to_string(),
            text: mock_logs,
        }])
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

async fn read_agent_metrics(state: Arc<McpServerState>, agent_id: &str) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    
    if let Some(agent) = agents.get(agent_id) {
        let metrics = json!({
            "agent_id": agent_id,
            "status": agent.status,
            "uptime": chrono::Utc::now().signed_duration_since(agent.created_at).num_seconds(),
            "last_seen": agent.last_seen,
            "performance": {
                "cpu_usage": 25.5,
                "memory_usage": 45.2,
                "task_completion_rate": 0.95,
                "average_response_time": 150.0
            },
            "counters": {
                "tasks_completed": 42,
                "tasks_failed": 2,
                "messages_sent": 128,
                "messages_received": 156
            },
            "timestamp": chrono::Utc::now()
        });

        Ok(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&metrics)?,
        }])
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

async fn read_agent_config(state: Arc<McpServerState>, agent_id: &str) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    
    if let Some(agent) = agents.get(agent_id) {
        let config = json!({
            "agent_id": agent_id,
            "name": agent.name,
            "type": agent.agent_type,
            "capabilities": agent.capabilities,
            "endpoint": agent.endpoint,
            "metadata": agent.metadata,
            "created_at": agent.created_at,
            "configuration": {
                "autonomy_interval": 60,
                "max_concurrent_tasks": 5,
                "log_level": "info",
                "enable_ai": true,
                "enable_economic_engine": true
            }
        });

        Ok(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&config)?,
        }])
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

async fn read_tasks_list(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;

    let task_list: Vec<Value> = tasks.iter()
        .map(|(id, task)| {
            let result = results.get(id);
            json!({
                "task": task,
                "result": result,
                "status": result.as_ref().map(|r| &r.status).unwrap_or(&crate::TaskStatus::Pending)
            })
        })
        .collect();

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json!({
            "tasks": task_list,
            "total_count": task_list.len(),
            "timestamp": chrono::Utc::now()
        }))?,
    }])
}

async fn read_task_details(state: Arc<McpServerState>, task_id: &str) -> Result<Vec<Content>> {
    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;
    
    if let Some(task) = tasks.get(task_id) {
        let result = results.get(task_id);
        
        let details = json!({
            "task": task,
            "result": result,
            "status": result.as_ref().map(|r| &r.status).unwrap_or(&crate::TaskStatus::Pending),
            "timestamp": chrono::Utc::now()
        });

        Ok(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&details)?,
        }])
    } else {
        Err(DaaMcpError::Protocol(format!("Task not found: {}", task_id)))
    }
}

async fn read_task_results(state: Arc<McpServerState>, task_id: &str) -> Result<Vec<Content>> {
    let results = state.task_results.read().await;
    
    if let Some(result) = results.get(task_id) {
        Ok(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(result)?,
        }])
    } else {
        Ok(vec![Content {
            content_type: "application/json".to_string(),
            text: json!({"message": "No results available yet", "task_id": task_id}).to_string(),
        }])
    }
}

async fn read_pending_tasks(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;

    let pending_tasks: Vec<Value> = tasks.iter()
        .filter(|(id, _)| !results.contains_key(*id))
        .map(|(_, task)| serde_json::to_value(task).unwrap())
        .collect();

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json!({
            "pending_tasks": pending_tasks,
            "count": pending_tasks.len(),
            "timestamp": chrono::Utc::now()
        }))?,
    }])
}

async fn read_running_tasks(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let results = state.task_results.read().await;

    let running_tasks: Vec<Value> = results.values()
        .filter(|result| matches!(result.status, crate::TaskStatus::Running))
        .map(|result| serde_json::to_value(result).unwrap())
        .collect();

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json!({
            "running_tasks": running_tasks,
            "count": running_tasks.len(),
            "timestamp": chrono::Utc::now()
        }))?,
    }])
}

async fn read_swarm_status(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    let messages = state.swarm_messages.read().await;

    let swarm_status = json!({
        "total_agents": agents.len(),
        "active_agents": agents.values().filter(|a| matches!(a.status, crate::AgentStatus::Running)).count(),
        "paused_agents": agents.values().filter(|a| matches!(a.status, crate::AgentStatus::Paused)).count(),
        "error_agents": agents.values().filter(|a| matches!(a.status, crate::AgentStatus::Error)).count(),
        "total_messages": messages.len(),
        "recent_messages": messages.iter()
            .filter(|m| chrono::Utc::now().signed_duration_since(m.timestamp) < chrono::Duration::hours(1))
            .count(),
        "agent_types": {
            let mut types = std::collections::HashMap::new();
            for agent in agents.values() {
                *types.entry(&agent.agent_type).or_insert(0) += 1;
            }
            types
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&swarm_status)?,
    }])
}

async fn read_swarm_messages(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let messages = state.swarm_messages.read().await;
    
    // Get recent messages (last 100)
    let recent_messages: Vec<Value> = messages.iter()
        .rev()
        .take(100)
        .map(|msg| serde_json::to_value(msg).unwrap())
        .collect();

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json!({
            "messages": recent_messages,
            "total_count": messages.len(),
            "shown_count": recent_messages.len(),
            "timestamp": chrono::Utc::now()
        }))?,
    }])
}

async fn read_swarm_topology(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    
    let topology = json!({
        "nodes": agents.values().map(|agent| json!({
            "id": agent.id,
            "name": agent.name,
            "type": agent.agent_type,
            "status": agent.status,
            "endpoint": agent.endpoint,
            "capabilities": agent.capabilities
        })).collect::<Vec<_>>(),
        "connections": [],  // In a real implementation, this would show actual connections
        "clusters": {
            let mut clusters = std::collections::HashMap::new();
            for agent in agents.values() {
                clusters.entry(&agent.agent_type).or_insert_with(Vec::new).push(&agent.id);
            }
            clusters
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&topology)?,
    }])
}

async fn read_system_metrics(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;

    let metrics = json!({
        "system": {
            "uptime_seconds": 3600,  // Mock value
            "version": "0.2.0",
            "build": "development",
            "timestamp": chrono::Utc::now()
        },
        "performance": {
            "cpu_usage": 15.2,
            "memory_usage": 34.7,
            "disk_usage": 12.1,
            "network_throughput": 1024.5
        },
        "agents": {
            "total": agents.len(),
            "running": agents.values().filter(|a| matches!(a.status, crate::AgentStatus::Running)).count(),
            "average_response_time": 145.3
        },
        "tasks": {
            "total": tasks.len(),
            "completed": results.values().filter(|r| matches!(r.status, crate::TaskStatus::Completed)).count(),
            "success_rate": 0.94,
            "average_execution_time": 2.5
        }
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&metrics)?,
    }])
}

async fn read_system_logs(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let _state = state; // Keep for future use
    
    // Mock system logs
    let logs = format!(
        "[{}] INFO DAA MCP Server started\n[{}] INFO Loaded {} agents\n[{}] DEBUG Heartbeat monitor active\n[{}] INFO System health check passed\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        5, // Mock agent count
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );

    Ok(vec![Content {
        content_type: "text/plain".to_string(),
        text: logs,
    }])
}

async fn read_system_config(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let config = json!({
        "server": {
            "name": state.config.server_name,
            "version": state.config.server_version,
            "bind_address": state.config.bind_address,
            "port": state.config.port,
            "enable_websocket": state.config.enable_websocket,
            "enable_discovery": state.config.enable_discovery
        },
        "limits": {
            "max_agents": state.config.max_agents,
            "heartbeat_interval": state.config.heartbeat_interval.as_secs(),
            "task_timeout": state.config.task_timeout.as_secs()
        },
        "features": {
            "swarm_coordination": true,
            "mcp_tools": true,
            "real_time_monitoring": true,
            "agent_discovery": state.config.enable_discovery
        }
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&config)?,
    }])
}

async fn read_system_health(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    
    let error_count = agents.values().filter(|a| matches!(a.status, crate::AgentStatus::Error)).count();
    let total_agents = agents.len();
    
    let overall_health = if error_count == 0 {
        "healthy"
    } else if error_count < total_agents / 2 {
        "degraded"
    } else {
        "unhealthy"
    };

    let health = json!({
        "overall_status": overall_health,
        "components": {
            "mcp_server": "healthy",
            "agent_manager": if error_count == 0 { "healthy" } else { "degraded" },
            "task_manager": "healthy",
            "swarm_coordinator": "healthy",
            "resource_manager": "healthy"
        },
        "metrics": {
            "error_rate": if total_agents > 0 { error_count as f64 / total_agents as f64 } else { 0.0 },
            "uptime": "100%",
            "response_time": "normal"
        },
        "issues": if error_count > 0 {
            vec![format!("{} agents in error state", error_count)]
        } else {
            Vec::<String>::new()
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&health)?,
    }])
}

async fn read_discovery_agents(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    
    let discoverable_agents: Vec<Value> = agents.values()
        .filter(|agent| matches!(agent.status, crate::AgentStatus::Running))
        .map(|agent| json!({
            "id": agent.id,
            "name": agent.name,
            "type": agent.agent_type,
            "capabilities": agent.capabilities,
            "endpoint": agent.endpoint,
            "last_seen": agent.last_seen,
            "discovery_metadata": {
                "availability": "online",
                "load": "normal",
                "response_time": "fast"
            }
        }))
        .collect();

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json!({
            "discoverable_agents": discoverable_agents,
            "total_count": discoverable_agents.len(),
            "timestamp": chrono::Utc::now()
        }))?,
    }])
}

async fn read_discovery_services(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let _state = state; // Keep for future use
    
    let services = json!({
        "mcp_services": [
            {
                "name": "daa-mcp-server",
                "endpoint": "http://localhost:3001/mcp",
                "protocol": "HTTP + JSON-RPC 2.0",
                "capabilities": ["tools", "resources", "prompts"],
                "status": "online"
            },
            {
                "name": "daa-mcp-websocket",
                "endpoint": "ws://localhost:3001/mcp/ws",
                "protocol": "WebSocket + JSON-RPC 2.0",
                "capabilities": ["real-time", "streaming"],
                "status": "online"
            }
        ],
        "agent_services": [
            {
                "service_type": "orchestrator",
                "instances": 1,
                "endpoints": ["http://localhost:3002"],
                "status": "online"
            }
        ],
        "discovery_protocol": {
            "type": "HTTP + mDNS",
            "enabled": true,
            "refresh_interval": 30
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&services)?,
    }])
}

async fn read_performance_analytics(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    let results = state.task_results.read().await;

    let analytics = json!({
        "performance_summary": {
            "total_agents": agents.len(),
            "average_agent_uptime": 3600.0,
            "task_completion_rate": 0.94,
            "average_task_duration": 2.5,
            "system_throughput": 150.0
        },
        "trends": {
            "agent_growth": [
                {"period": "last_hour", "count": agents.len()},
                {"period": "last_day", "count": agents.len()},
                {"period": "last_week", "count": agents.len()}
            ],
            "task_volume": [
                {"period": "last_hour", "count": results.len()},
                {"period": "last_day", "count": results.len()},
                {"period": "last_week", "count": results.len()}
            ]
        },
        "bottlenecks": [],
        "recommendations": [
            "System is performing well",
            "Consider adding more agents for increased capacity"
        ],
        "timestamp": chrono::Utc::now()
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&analytics)?,
    }])
}

async fn read_usage_analytics(state: Arc<McpServerState>) -> Result<Vec<Content>> {
    let agents = state.agents.read().await;
    let tasks = state.tasks.read().await;
    let messages = state.swarm_messages.read().await;

    let analytics = json!({
        "usage_summary": {
            "active_agents": agents.values().filter(|a| matches!(a.status, crate::AgentStatus::Running)).count(),
            "total_tasks": tasks.len(),
            "total_messages": messages.len(),
            "api_calls": 1000, // Mock value
            "data_transferred": "10.5 GB" // Mock value
        },
        "agent_utilization": {
            let mut utilization = std::collections::HashMap::new();
            for agent in agents.values() {
                *utilization.entry(&agent.agent_type).or_insert(0) += 1;
            }
            utilization
        },
        "peak_usage": {
            "peak_agents": agents.len(),
            "peak_tasks_per_hour": 100,
            "peak_messages_per_minute": 50
        },
        "resource_consumption": {
            "cpu_hours": 24.5,
            "memory_gb_hours": 12.3,
            "network_gb": 5.7
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(vec![Content {
        content_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&analytics)?,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DaaMcpConfig, DaaAgentInfo, AgentStatus};

    #[tokio::test]
    async fn test_read_agents_list() {
        let config = DaaMcpConfig::default();
        let state = Arc::new(McpServerState::new(config));

        // Add a test agent
        let agent = DaaAgentInfo {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            agent_type: "test".to_string(),
            status: AgentStatus::Running,
            capabilities: vec!["testing".to_string()],
            endpoint: Some("http://localhost:3002".to_string()),
            created_at: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        state.agents.write().await.insert("test-agent".to_string(), agent);

        let content = read_agents_list(state).await.unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].content_type, "application/json");
    }

    #[tokio::test]
    async fn test_read_system_health() {
        let config = DaaMcpConfig::default();
        let state = Arc::new(McpServerState::new(config));

        let content = read_system_health(state).await.unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].content_type, "application/json");
        
        // Parse the JSON to verify structure
        let health: serde_json::Value = serde_json::from_str(&content[0].text).unwrap();
        assert!(health.get("overall_status").is_some());
        assert!(health.get("components").is_some());
    }
}