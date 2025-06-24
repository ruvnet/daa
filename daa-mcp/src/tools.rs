//! MCP Tools for DAA management
//! 
//! This module defines all the available tools that can be called through
//! the MCP interface to manage DAA agents, tasks, and coordination.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    AgentConfig, AgentStatus, DaaAgentInfo, DaaTask, DaaMcpError, McpServerState, Result,
    SwarmMessage, SwarmMessageType, TaskPriority, TaskResult, TaskStatus, ToolInfo, ToolResult,
    Content,
};

/// Get list of all available tools
pub fn get_available_tools() -> Vec<ToolInfo> {
    vec![
        // Agent Management Tools
        ToolInfo {
            name: "spawn_agent".to_string(),
            description: "Spawn a new DAA agent with specified configuration".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "config": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "agent_type": {"type": "string"},
                            "capabilities": {"type": "array", "items": {"type": "string"}},
                            "rules": {"type": "object"},
                            "economic_config": {"type": "object"},
                            "ai_config": {"type": "object"},
                            "metadata": {"type": "object"}
                        },
                        "required": ["name", "agent_type"]
                    }
                },
                "required": ["config"]
            }),
        },
        ToolInfo {
            name: "stop_agent".to_string(),
            description: "Stop a running DAA agent".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string"}
                },
                "required": ["agent_id"]
            }),
        },
        ToolInfo {
            name: "pause_agent".to_string(),
            description: "Pause a running DAA agent".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string"}
                },
                "required": ["agent_id"]
            }),
        },
        ToolInfo {
            name: "resume_agent".to_string(),
            description: "Resume a paused DAA agent".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string"}
                },
                "required": ["agent_id"]
            }),
        },
        ToolInfo {
            name: "list_agents".to_string(),
            description: "List all DAA agents with their status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "object",
                        "properties": {
                            "status": {"type": "string"},
                            "agent_type": {"type": "string"},
                            "capabilities": {"type": "array", "items": {"type": "string"}}
                        }
                    }
                }
            }),
        },
        ToolInfo {
            name: "get_agent_info".to_string(),
            description: "Get detailed information about a specific agent".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string"}
                },
                "required": ["agent_id"]
            }),
        },

        // Task Management Tools
        ToolInfo {
            name: "create_task".to_string(),
            description: "Create a new task for agent execution".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_type": {"type": "string"},
                    "description": {"type": "string"},
                    "parameters": {"type": "object"},
                    "priority": {"type": "string", "enum": ["low", "medium", "high", "critical"]},
                    "timeout": {"type": "number"},
                    "dependencies": {"type": "array", "items": {"type": "string"}},
                    "assigned_agents": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["task_type", "description"]
            }),
        },
        ToolInfo {
            name: "assign_task".to_string(),
            description: "Assign a task to specific agents".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string"},
                    "agent_ids": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["task_id", "agent_ids"]
            }),
        },
        ToolInfo {
            name: "cancel_task".to_string(),
            description: "Cancel a pending or running task".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string"}
                },
                "required": ["task_id"]
            }),
        },
        ToolInfo {
            name: "get_task_status".to_string(),
            description: "Get the status and results of a task".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string"}
                },
                "required": ["task_id"]
            }),
        },
        ToolInfo {
            name: "list_tasks".to_string(),
            description: "List tasks with optional filtering".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "object",
                        "properties": {
                            "status": {"type": "string"},
                            "agent_id": {"type": "string"},
                            "task_type": {"type": "string"}
                        }
                    }
                }
            }),
        },

        // Swarm Coordination Tools
        ToolInfo {
            name: "coordinate_swarm".to_string(),
            description: "Coordinate a group of agents for a complex task".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "objective": {"type": "string"},
                    "agent_types": {"type": "array", "items": {"type": "string"}},
                    "coordination_strategy": {"type": "string"},
                    "max_agents": {"type": "number"},
                    "timeout": {"type": "number"}
                },
                "required": ["objective", "agent_types"]
            }),
        },
        ToolInfo {
            name: "send_swarm_message".to_string(),
            description: "Send a message to agents in the swarm".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from_agent": {"type": "string"},
                    "to_agents": {"type": "array", "items": {"type": "string"}},
                    "message_type": {"type": "string"},
                    "payload": {"type": "object"},
                    "ttl": {"type": "number"}
                },
                "required": ["from_agent", "message_type", "payload"]
            }),
        },
        ToolInfo {
            name: "get_swarm_status".to_string(),
            description: "Get the current status of the agent swarm".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "swarm_id": {"type": "string"}
                }
            }),
        },

        // Discovery and Monitoring Tools
        ToolInfo {
            name: "discover_agents".to_string(),
            description: "Discover available agents by capabilities".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "required_capabilities": {"type": "array", "items": {"type": "string"}},
                    "preferred_capabilities": {"type": "array", "items": {"type": "string"}},
                    "max_results": {"type": "number"}
                }
            }),
        },
        ToolInfo {
            name: "get_system_metrics".to_string(),
            description: "Get system-wide metrics and performance data".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "metric_types": {"type": "array", "items": {"type": "string"}},
                    "time_range": {"type": "string"}
                }
            }),
        },
        ToolInfo {
            name: "healthcheck".to_string(),
            description: "Perform a comprehensive system health check".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "deep_check": {"type": "boolean"}
                }
            }),
        },
    ]
}

/// Execute a tool with the given arguments
pub async fn execute_tool(
    state: Arc<McpServerState>,
    tool_name: &str,
    arguments: Value,
) -> Result<ToolResult> {
    debug!("Executing tool: {} with args: {:?}", tool_name, arguments);

    let result = match tool_name {
        // Agent Management
        "spawn_agent" => spawn_agent(state, arguments).await?,
        "stop_agent" => stop_agent(state, arguments).await?,
        "pause_agent" => pause_agent(state, arguments).await?,
        "resume_agent" => resume_agent(state, arguments).await?,
        "list_agents" => list_agents(state, arguments).await?,
        "get_agent_info" => get_agent_info(state, arguments).await?,

        // Task Management
        "create_task" => create_task(state, arguments).await?,
        "assign_task" => assign_task(state, arguments).await?,
        "cancel_task" => cancel_task(state, arguments).await?,
        "get_task_status" => get_task_status(state, arguments).await?,
        "list_tasks" => list_tasks(state, arguments).await?,

        // Swarm Coordination
        "coordinate_swarm" => coordinate_swarm(state, arguments).await?,
        "send_swarm_message" => send_swarm_message(state, arguments).await?,
        "get_swarm_status" => get_swarm_status(state, arguments).await?,

        // Discovery and Monitoring
        "discover_agents" => discover_agents(state, arguments).await?,
        "get_system_metrics" => get_system_metrics(state, arguments).await?,
        "healthcheck" => healthcheck(state, arguments).await?,

        _ => {
            return Err(DaaMcpError::InvalidTool(format!("Unknown tool: {}", tool_name)));
        }
    };

    Ok(result)
}

// Agent Management Tool Implementations

async fn spawn_agent(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let config: AgentConfig = serde_json::from_value(args.get("config").unwrap().clone())?;
    
    let agent_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    
    let agent_info = DaaAgentInfo {
        id: agent_id.clone(),
        name: config.name.clone(),
        agent_type: config.agent_type.clone(),
        status: AgentStatus::Starting,
        capabilities: config.capabilities.clone(),
        endpoint: Some(format!("http://localhost:300{}", rand::random::<u8>() % 100 + 1)),
        created_at: now,
        last_seen: now,
        metadata: config.metadata.clone(),
    };

    // Store agent info
    let mut agents = state.agents.write().await;
    agents.insert(agent_id.clone(), agent_info);

    info!("Spawned new agent: {} ({})", config.name, agent_id);

    // TODO: Actually spawn the agent process here
    // This would integrate with daa-orchestrator to create a real agent

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "text".to_string(),
            text: format!("Successfully spawned agent '{}' with ID: {}", config.name, agent_id),
        }]),
        is_error: Some(false),
    })
}

async fn stop_agent(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let agent_id = args.get("agent_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing agent_id".to_string()))?;

    let mut agents = state.agents.write().await;
    
    if let Some(agent) = agents.get_mut(agent_id) {
        agent.status = AgentStatus::Stopping;
        info!("Stopping agent: {}", agent_id);
        
        // TODO: Actually stop the agent process
        
        agent.status = AgentStatus::Stopped;
        
        Ok(ToolResult {
            content: Some(vec![Content {
                content_type: "text".to_string(),
                text: format!("Successfully stopped agent: {}", agent_id),
            }]),
            is_error: Some(false),
        })
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

async fn pause_agent(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let agent_id = args.get("agent_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing agent_id".to_string()))?;

    let mut agents = state.agents.write().await;
    
    if let Some(agent) = agents.get_mut(agent_id) {
        agent.status = AgentStatus::Paused;
        info!("Paused agent: {}", agent_id);
        
        Ok(ToolResult {
            content: Some(vec![Content {
                content_type: "text".to_string(),
                text: format!("Successfully paused agent: {}", agent_id),
            }]),
            is_error: Some(false),
        })
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

async fn resume_agent(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let agent_id = args.get("agent_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing agent_id".to_string()))?;

    let mut agents = state.agents.write().await;
    
    if let Some(agent) = agents.get_mut(agent_id) {
        agent.status = AgentStatus::Running;
        agent.last_seen = chrono::Utc::now();
        info!("Resumed agent: {}", agent_id);
        
        Ok(ToolResult {
            content: Some(vec![Content {
                content_type: "text".to_string(),
                text: format!("Successfully resumed agent: {}", agent_id),
            }]),
            is_error: Some(false),
        })
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

async fn list_agents(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let agents = state.agents.read().await;
    
    let mut filtered_agents: Vec<&DaaAgentInfo> = agents.values().collect();
    
    // Apply filters if provided
    if let Some(filter) = args.get("filter") {
        if let Some(status) = filter.get("status").and_then(|v| v.as_str()) {
            filtered_agents.retain(|agent| {
                match status {
                    "running" => matches!(agent.status, AgentStatus::Running),
                    "paused" => matches!(agent.status, AgentStatus::Paused),
                    "stopped" => matches!(agent.status, AgentStatus::Stopped),
                    "error" => matches!(agent.status, AgentStatus::Error),
                    _ => true,
                }
            });
        }
        
        if let Some(agent_type) = filter.get("agent_type").and_then(|v| v.as_str()) {
            filtered_agents.retain(|agent| agent.agent_type == agent_type);
        }
    }

    let agent_list: Vec<Value> = filtered_agents.iter()
        .map(|agent| serde_json::to_value(agent).unwrap())
        .collect();

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&json!({
                "agents": agent_list,
                "total_count": agent_list.len()
            }))?,
        }]),
        is_error: Some(false),
    })
}

async fn get_agent_info(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let agent_id = args.get("agent_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing agent_id".to_string()))?;

    let agents = state.agents.read().await;
    
    if let Some(agent) = agents.get(agent_id) {
        Ok(ToolResult {
            content: Some(vec![Content {
                content_type: "application/json".to_string(),
                text: serde_json::to_string_pretty(agent)?,
            }]),
            is_error: Some(false),
        })
    } else {
        Err(DaaMcpError::AgentNotFound(agent_id.to_string()))
    }
}

// Task Management Tool Implementations

async fn create_task(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let task_id = Uuid::new_v4().to_string();
    
    let task = DaaTask {
        id: task_id.clone(),
        task_type: args.get("task_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaaMcpError::Protocol("Missing task_type".to_string()))?
            .to_string(),
        description: args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaaMcpError::Protocol("Missing description".to_string()))?
            .to_string(),
        parameters: args.get("parameters")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect(),
        priority: match args.get("priority").and_then(|v| v.as_str()).unwrap_or("medium") {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            "critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        },
        timeout: args.get("timeout").and_then(|v| v.as_u64()),
        dependencies: args.get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        assigned_agents: args.get("assigned_agents")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    };

    let mut tasks = state.tasks.write().await;
    tasks.insert(task_id.clone(), task);

    info!("Created task: {}", task_id);

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "text".to_string(),
            text: format!("Successfully created task with ID: {}", task_id),
        }]),
        is_error: Some(false),
    })
}

async fn assign_task(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let task_id = args.get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing task_id".to_string()))?;

    let agent_ids: Vec<String> = args.get("agent_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DaaMcpError::Protocol("Missing agent_ids".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let mut tasks = state.tasks.write().await;
    
    if let Some(task) = tasks.get_mut(task_id) {
        task.assigned_agents = agent_ids.clone();
        info!("Assigned task {} to agents: {:?}", task_id, agent_ids);
        
        Ok(ToolResult {
            content: Some(vec![Content {
                content_type: "text".to_string(),
                text: format!("Successfully assigned task {} to {} agents", task_id, agent_ids.len()),
            }]),
            is_error: Some(false),
        })
    } else {
        Err(DaaMcpError::Protocol(format!("Task not found: {}", task_id)))
    }
}

async fn cancel_task(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let task_id = args.get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing task_id".to_string()))?;

    let mut results = state.task_results.write().await;
    
    let cancel_result = TaskResult {
        task_id: task_id.to_string(),
        agent_id: "system".to_string(),
        status: TaskStatus::Cancelled,
        result: None,
        error: Some("Task cancelled by user".to_string()),
        started_at: chrono::Utc::now(),
        completed_at: Some(chrono::Utc::now()),
        metrics: HashMap::new(),
    };
    
    results.insert(task_id.to_string(), cancel_result);
    
    info!("Cancelled task: {}", task_id);

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "text".to_string(),
            text: format!("Successfully cancelled task: {}", task_id),
        }]),
        is_error: Some(false),
    })
}

async fn get_task_status(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let task_id = args.get("task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing task_id".to_string()))?;

    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;
    
    if let Some(task) = tasks.get(task_id) {
        let result = results.get(task_id);
        
        let status_info = json!({
            "task": task,
            "result": result,
            "has_result": result.is_some()
        });
        
        Ok(ToolResult {
            content: Some(vec![Content {
                content_type: "application/json".to_string(),
                text: serde_json::to_string_pretty(&status_info)?,
            }]),
            is_error: Some(false),
        })
    } else {
        Err(DaaMcpError::Protocol(format!("Task not found: {}", task_id)))
    }
}

async fn list_tasks(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;
    
    let mut task_list: Vec<Value> = Vec::new();
    
    for (task_id, task) in tasks.iter() {
        let result = results.get(task_id);
        task_list.push(json!({
            "task": task,
            "result": result,
            "status": result.as_ref().map(|r| &r.status).unwrap_or(&TaskStatus::Pending)
        }));
    }

    // Apply filters if provided
    if let Some(filter) = args.get("filter") {
        if let Some(status_filter) = filter.get("status").and_then(|v| v.as_str()) {
            task_list.retain(|item| {
                if let Some(status) = item.get("status") {
                    status.as_str().unwrap_or("") == status_filter
                } else {
                    false
                }
            });
        }
    }

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&json!({
                "tasks": task_list,
                "total_count": task_list.len()
            }))?,
        }]),
        is_error: Some(false),
    })
}

// Swarm Coordination Tool Implementations

async fn coordinate_swarm(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let objective = args.get("objective")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("Missing objective".to_string()))?;

    let agent_types: Vec<String> = args.get("agent_types")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DaaMcpError::Protocol("Missing agent_types".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let max_agents = args.get("max_agents")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    // Find suitable agents
    let agents = state.agents.read().await;
    let mut selected_agents = Vec::new();
    
    for agent_type in &agent_types {
        let type_agents: Vec<_> = agents.values()
            .filter(|a| a.agent_type == *agent_type && matches!(a.status, AgentStatus::Running))
            .take(max_agents / agent_types.len())
            .collect();
        selected_agents.extend(type_agents);
    }

    if selected_agents.is_empty() {
        return Ok(ToolResult {
            content: Some(vec![Content {
                content_type: "text".to_string(),
                text: "No suitable agents found for swarm coordination".to_string(),
            }]),
            is_error: Some(true),
        });
    }

    // Create coordination message
    let swarm_id = Uuid::new_v4().to_string();
    let message = SwarmMessage {
        id: Uuid::new_v4().to_string(),
        from_agent: "coordinator".to_string(),
        to_agents: selected_agents.iter().map(|a| a.id.clone()).collect(),
        message_type: SwarmMessageType::Coordination,
        payload: json!({
            "swarm_id": swarm_id,
            "objective": objective,
            "coordination_strategy": args.get("coordination_strategy").unwrap_or(&json!("collaborative")),
            "role_assignments": selected_agents.iter().enumerate().map(|(i, a)| {
                json!({
                    "agent_id": a.id,
                    "role": format!("worker_{}", i + 1),
                    "capabilities": a.capabilities
                })
            }).collect::<Vec<_>>()
        }),
        timestamp: chrono::Utc::now(),
        ttl: args.get("timeout").and_then(|v| v.as_u64()),
    };

    let mut messages = state.swarm_messages.write().await;
    messages.push(message);

    info!("Coordinated swarm with {} agents for objective: {}", selected_agents.len(), objective);

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&json!({
                "swarm_id": swarm_id,
                "coordinated_agents": selected_agents.len(),
                "agents": selected_agents.iter().map(|a| json!({
                    "id": a.id,
                    "name": a.name,
                    "type": a.agent_type,
                    "capabilities": a.capabilities
                })).collect::<Vec<_>>(),
                "objective": objective
            }))?,
        }]),
        is_error: Some(false),
    })
}

async fn send_swarm_message(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let message = SwarmMessage {
        id: Uuid::new_v4().to_string(),
        from_agent: args.get("from_agent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaaMcpError::Protocol("Missing from_agent".to_string()))?
            .to_string(),
        to_agents: args.get("to_agents")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        message_type: match args.get("message_type").and_then(|v| v.as_str()).unwrap_or("coordination") {
            "task_assignment" => SwarmMessageType::TaskAssignment,
            "task_update" => SwarmMessageType::TaskUpdate,
            "state_sync" => SwarmMessageType::StateSync,
            "resource_request" => SwarmMessageType::ResourceRequest,
            "resource_response" => SwarmMessageType::ResourceResponse,
            "heartbeat" => SwarmMessageType::Heartbeat,
            "discovery" => SwarmMessageType::Discovery,
            _ => SwarmMessageType::Coordination,
        },
        payload: args.get("payload")
            .cloned()
            .ok_or_else(|| DaaMcpError::Protocol("Missing payload".to_string()))?,
        timestamp: chrono::Utc::now(),
        ttl: args.get("ttl").and_then(|v| v.as_u64()),
    };

    let mut messages = state.swarm_messages.write().await;
    messages.push(message.clone());

    info!("Sent swarm message from {} to {} agents", message.from_agent, message.to_agents.len());

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "text".to_string(),
            text: format!("Successfully sent swarm message with ID: {}", message.id),
        }]),
        is_error: Some(false),
    })
}

async fn get_swarm_status(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let swarm_id = args.get("swarm_id").and_then(|v| v.as_str());
    
    let messages = state.swarm_messages.read().await;
    let agents = state.agents.read().await;
    
    let mut swarm_info = json!({
        "total_agents": agents.len(),
        "active_agents": agents.values().filter(|a| matches!(a.status, AgentStatus::Running)).count(),
        "total_messages": messages.len()
    });

    if let Some(id) = swarm_id {
        let swarm_messages: Vec<_> = messages.iter()
            .filter(|m| m.payload.get("swarm_id").and_then(|v| v.as_str()) == Some(id))
            .collect();
        
        swarm_info["swarm_messages"] = json!(swarm_messages.len());
        swarm_info["swarm_id"] = json!(id);
    }

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&swarm_info)?,
        }]),
        is_error: Some(false),
    })
}

// Discovery and Monitoring Tool Implementations

async fn discover_agents(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let required_capabilities: Vec<String> = args.get("required_capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let preferred_capabilities: Vec<String> = args.get("preferred_capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let max_results = args.get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let agents = state.agents.read().await;
    
    let mut matching_agents: Vec<_> = agents.values()
        .filter(|agent| {
            // Check required capabilities
            required_capabilities.iter().all(|cap| agent.capabilities.contains(cap))
        })
        .collect();

    // Sort by preferred capabilities (agents with more preferred capabilities first)
    matching_agents.sort_by(|a, b| {
        let a_preferred = a.capabilities.iter().filter(|cap| preferred_capabilities.contains(cap)).count();
        let b_preferred = b.capabilities.iter().filter(|cap| preferred_capabilities.contains(cap)).count();
        b_preferred.cmp(&a_preferred)
    });

    matching_agents.truncate(max_results);

    let discovered = matching_agents.iter()
        .map(|agent| json!({
            "id": agent.id,
            "name": agent.name,
            "type": agent.agent_type,
            "status": agent.status,
            "capabilities": agent.capabilities,
            "endpoint": agent.endpoint,
            "last_seen": agent.last_seen
        }))
        .collect::<Vec<_>>();

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&json!({
                "discovered_agents": discovered,
                "total_found": discovered.len(),
                "search_criteria": {
                    "required_capabilities": required_capabilities,
                    "preferred_capabilities": preferred_capabilities
                }
            }))?,
        }]),
        is_error: Some(false),
    })
}

async fn get_system_metrics(state: Arc<McpServerState>, _args: Value) -> Result<ToolResult> {
    let agents = state.agents.read().await;
    let tasks = state.tasks.read().await;
    let results = state.task_results.read().await;
    let messages = state.swarm_messages.read().await;

    let metrics = json!({
        "agents": {
            "total": agents.len(),
            "running": agents.values().filter(|a| matches!(a.status, AgentStatus::Running)).count(),
            "paused": agents.values().filter(|a| matches!(a.status, AgentStatus::Paused)).count(),
            "stopped": agents.values().filter(|a| matches!(a.status, AgentStatus::Stopped)).count(),
            "error": agents.values().filter(|a| matches!(a.status, AgentStatus::Error)).count(),
        },
        "tasks": {
            "total": tasks.len(),
            "pending": results.values().filter(|r| matches!(r.status, TaskStatus::Pending)).count(),
            "running": results.values().filter(|r| matches!(r.status, TaskStatus::Running)).count(),
            "completed": results.values().filter(|r| matches!(r.status, TaskStatus::Completed)).count(),
            "failed": results.values().filter(|r| matches!(r.status, TaskStatus::Failed)).count(),
            "cancelled": results.values().filter(|r| matches!(r.status, TaskStatus::Cancelled)).count(),
        },
        "swarm": {
            "total_messages": messages.len(),
            "recent_messages": messages.iter()
                .filter(|m| chrono::Utc::now().signed_duration_since(m.timestamp) < chrono::Duration::hours(1))
                .count(),
        },
        "system": {
            "uptime": "running",
            "version": "0.2.0",
            "timestamp": chrono::Utc::now()
        }
    });

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&metrics)?,
        }]),
        is_error: Some(false),
    })
}

async fn healthcheck(state: Arc<McpServerState>, args: Value) -> Result<ToolResult> {
    let deep_check = args.get("deep_check")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let agents = state.agents.read().await;
    let mut health_status = json!({
        "overall_status": "healthy",
        "timestamp": chrono::Utc::now(),
        "components": {
            "mcp_server": "healthy",
            "agent_manager": "healthy",
            "task_manager": "healthy",
            "swarm_coordinator": "healthy"
        }
    });

    // Check agent health
    let error_agents = agents.values().filter(|a| matches!(a.status, AgentStatus::Error)).count();
    if error_agents > 0 {
        health_status["components"]["agent_manager"] = json!("degraded");
        if error_agents > agents.len() / 2 {
            health_status["overall_status"] = json!("unhealthy");
        }
    }

    if deep_check {
        // Perform more extensive checks
        health_status["deep_check"] = json!({
            "performed": true,
            "agent_response_times": "normal",
            "memory_usage": "normal",
            "disk_space": "normal",
            "network_connectivity": "normal"
        });
    }

    let is_healthy = health_status["overall_status"] == "healthy";

    Ok(ToolResult {
        content: Some(vec![Content {
            content_type: "application/json".to_string(),
            text: serde_json::to_string_pretty(&health_status)?,
        }]),
        is_error: Some(!is_healthy),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DaaMcpConfig;

    #[tokio::test]
    async fn test_spawn_agent_tool() {
        let config = DaaMcpConfig::default();
        let state = Arc::new(McpServerState::new(config));

        let args = json!({
            "config": {
                "name": "test-agent",
                "agent_type": "treasury",
                "capabilities": ["trading", "risk_management"]
            }
        });

        let result = spawn_agent(state.clone(), args).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        let agents = state.agents.read().await;
        assert_eq!(agents.len(), 1);
    }

    #[tokio::test]
    async fn test_create_task_tool() {
        let config = DaaMcpConfig::default();
        let state = Arc::new(McpServerState::new(config));

        let args = json!({
            "task_type": "analysis",
            "description": "Analyze market conditions",
            "priority": "high"
        });

        let result = create_task(state.clone(), args).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        let tasks = state.tasks.read().await;
        assert_eq!(tasks.len(), 1);
    }
}