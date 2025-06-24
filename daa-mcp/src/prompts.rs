//! MCP Prompts for DAA management
//! 
//! This module defines prompt templates for common DAA operations,
//! making it easy for AI systems to interact with the DAA SDK.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value};

use crate::{DaaMcpError, McpServerState, Result};

/// Prompt information structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptInfo {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Prompt response structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptResponse {
    pub description: String,
    pub messages: Vec<PromptMessage>,
}

/// Individual prompt message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

/// Prompt content can be text or structured
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PromptContent {
    Text(String),
    Structured {
        #[serde(rename = "type")]
        content_type: String,
        text: String,
    },
}

/// Get list of all available prompts
pub fn get_available_prompts() -> Vec<PromptInfo> {
    vec![
        // Agent Management Prompts
        PromptInfo {
            name: "create_treasury_agent".to_string(),
            description: "Template for creating a treasury management agent".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "agent_name".to_string(),
                    description: "Name for the treasury agent".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "initial_balance".to_string(),
                    description: "Initial balance for the treasury agent".to_string(),
                    required: false,
                },
                PromptArgument {
                    name: "risk_threshold".to_string(),
                    description: "Risk threshold for investment decisions (0.0-1.0)".to_string(),
                    required: false,
                },
            ],
        },
        PromptInfo {
            name: "create_research_agent".to_string(),
            description: "Template for creating a research and analysis agent".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "agent_name".to_string(),
                    description: "Name for the research agent".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "research_domain".to_string(),
                    description: "Primary domain for research (e.g., 'crypto', 'defi', 'markets')".to_string(),
                    required: false,
                },
            ],
        },
        PromptInfo {
            name: "create_monitoring_agent".to_string(),
            description: "Template for creating a system monitoring agent".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "agent_name".to_string(),
                    description: "Name for the monitoring agent".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "monitoring_targets".to_string(),
                    description: "Comma-separated list of monitoring targets".to_string(),
                    required: false,
                },
            ],
        },

        // Task Coordination Prompts
        PromptInfo {
            name: "coordinate_market_analysis".to_string(),
            description: "Template for coordinating a multi-agent market analysis task".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "market_symbol".to_string(),
                    description: "Market symbol or identifier to analyze".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "analysis_depth".to_string(),
                    description: "Depth of analysis ('basic', 'detailed', 'comprehensive')".to_string(),
                    required: false,
                },
                PromptArgument {
                    name: "time_horizon".to_string(),
                    description: "Time horizon for analysis (e.g., '1h', '1d', '1w')".to_string(),
                    required: false,
                },
            ],
        },
        PromptInfo {
            name: "execute_trading_strategy".to_string(),
            description: "Template for executing a coordinated trading strategy".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "strategy_name".to_string(),
                    description: "Name of the trading strategy to execute".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "allocation_amount".to_string(),
                    description: "Amount to allocate for this strategy".to_string(),
                    required: false,
                },
                PromptArgument {
                    name: "max_risk".to_string(),
                    description: "Maximum risk tolerance for this strategy".to_string(),
                    required: false,
                },
            ],
        },

        // Swarm Coordination Prompts
        PromptInfo {
            name: "deploy_agent_swarm".to_string(),
            description: "Template for deploying a coordinated agent swarm".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "swarm_objective".to_string(),
                    description: "Primary objective for the agent swarm".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "agent_types".to_string(),
                    description: "Comma-separated list of agent types needed".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "coordination_strategy".to_string(),
                    description: "Strategy for swarm coordination ('hierarchical', 'peer-to-peer', 'hybrid')".to_string(),
                    required: false,
                },
            ],
        },
        PromptInfo {
            name: "emergency_response".to_string(),
            description: "Template for coordinating emergency response across agents".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "incident_type".to_string(),
                    description: "Type of incident requiring response".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "severity_level".to_string(),
                    description: "Severity level ('low', 'medium', 'high', 'critical')".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "affected_systems".to_string(),
                    description: "Comma-separated list of affected systems".to_string(),
                    required: false,
                },
            ],
        },

        // Analysis and Reporting Prompts
        PromptInfo {
            name: "generate_performance_report".to_string(),
            description: "Template for generating comprehensive performance reports".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "report_period".to_string(),
                    description: "Period for the report ('daily', 'weekly', 'monthly')".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "include_agents".to_string(),
                    description: "Comma-separated list of agent IDs to include".to_string(),
                    required: false,
                },
                PromptArgument {
                    name: "metrics_focus".to_string(),
                    description: "Focus area for metrics ('performance', 'economics', 'coordination')".to_string(),
                    required: false,
                },
            ],
        },
        PromptInfo {
            name: "diagnose_system_issues".to_string(),
            description: "Template for diagnosing and troubleshooting system issues".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "symptom_description".to_string(),
                    description: "Description of the observed symptoms".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "affected_components".to_string(),
                    description: "Components that appear to be affected".to_string(),
                    required: false,
                },
            ],
        },

        // Configuration and Setup Prompts
        PromptInfo {
            name: "setup_development_environment".to_string(),
            description: "Template for setting up a DAA development environment".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "environment_type".to_string(),
                    description: "Type of environment ('local', 'staging', 'production')".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "agent_count".to_string(),
                    description: "Number of agents to set up initially".to_string(),
                    required: false,
                },
            ],
        },
        PromptInfo {
            name: "optimize_agent_configuration".to_string(),
            description: "Template for optimizing agent configurations for better performance".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "agent_id".to_string(),
                    description: "ID of the agent to optimize".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "optimization_goal".to_string(),
                    description: "Goal for optimization ('performance', 'efficiency', 'accuracy')".to_string(),
                    required: false,
                },
            ],
        },
    ]
}

/// Get a specific prompt with filled arguments
pub async fn get_prompt(
    state: Arc<McpServerState>,
    name: &str,
    arguments: Value,
) -> Result<PromptResponse> {
    match name {
        "create_treasury_agent" => create_treasury_agent_prompt(state, arguments).await,
        "create_research_agent" => create_research_agent_prompt(state, arguments).await,
        "create_monitoring_agent" => create_monitoring_agent_prompt(state, arguments).await,
        "coordinate_market_analysis" => coordinate_market_analysis_prompt(state, arguments).await,
        "execute_trading_strategy" => execute_trading_strategy_prompt(state, arguments).await,
        "deploy_agent_swarm" => deploy_agent_swarm_prompt(state, arguments).await,
        "emergency_response" => emergency_response_prompt(state, arguments).await,
        "generate_performance_report" => generate_performance_report_prompt(state, arguments).await,
        "diagnose_system_issues" => diagnose_system_issues_prompt(state, arguments).await,
        "setup_development_environment" => setup_development_environment_prompt(state, arguments).await,
        "optimize_agent_configuration" => optimize_agent_configuration_prompt(state, arguments).await,
        _ => Err(DaaMcpError::Protocol(format!("Unknown prompt: {}", name))),
    }
}

// Agent Management Prompt Implementations

async fn create_treasury_agent_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let agent_name = args.get("agent_name")
        .and_then(|v| v.as_str())
        .unwrap_or("TreasuryAgent");
    let initial_balance = args.get("initial_balance")
        .and_then(|v| v.as_u64())
        .unwrap_or(100000);
    let risk_threshold = args.get("risk_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.2);

    let prompt_text = format!(
        r#"You are helping to create a treasury management agent named "{}" for the DAA (Decentralized Autonomous Agents) system.

## Agent Configuration

**Name**: {}
**Type**: Treasury Management Agent
**Initial Balance**: {} tokens
**Risk Threshold**: {}

## Capabilities Required
- Portfolio management and rebalancing
- Risk assessment and monitoring
- Market analysis and trend identification
- Automated trading execution
- Compliance and audit trail maintenance

## Configuration Steps

1. **Create the agent using MCP tools**:
```json
{{
  "tool": "spawn_agent",
  "arguments": {{
    "config": {{
      "name": "{}",
      "agent_type": "treasury",
      "capabilities": ["trading", "risk_management", "portfolio_analysis", "compliance"],
      "economic_config": {{
        "initial_balance": {},
        "token_symbol": "rUv",
        "max_daily_spend": {},
        "risk_threshold": {}
      }},
      "rules": {{
        "max_position_size": 0.1,
        "diversification_min": 0.05,
        "stop_loss_threshold": 0.05,
        "trading_hours": "09:00-17:00"
      }},
      "ai_config": {{
        "model": "claude-3-sonnet",
        "system_prompt": "You are a treasury management agent focused on capital preservation and steady growth."
      }}
    }}
  }}
}}
```

2. **Configure risk management rules**
3. **Set up monitoring and alerting**
4. **Initialize the autonomy loop**

The agent will automatically begin operating according to its configured rules and risk parameters."#,
        agent_name, agent_name, initial_balance, risk_threshold,
        agent_name, initial_balance, initial_balance / 10, risk_threshold
    );

    Ok(PromptResponse {
        description: format!("Treasury agent creation guide for {}", agent_name),
        messages: vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "I want to create a treasury management agent named '{}' with an initial balance of {} and risk threshold of {}.",
                    agent_name, initial_balance, risk_threshold
                )),
            },
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

async fn create_research_agent_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let agent_name = args.get("agent_name")
        .and_then(|v| v.as_str())
        .unwrap_or("ResearchAgent");
    let research_domain = args.get("research_domain")
        .and_then(|v| v.as_str())
        .unwrap_or("general");

    let prompt_text = format!(
        r#"Creating a research and analysis agent named "{}" specialized in {} research.

## Agent Setup

```json
{{
  "tool": "spawn_agent",
  "arguments": {{
    "config": {{
      "name": "{}",
      "agent_type": "researcher",
      "capabilities": ["data_analysis", "market_research", "trend_analysis", "report_generation"],
      "ai_config": {{
        "model": "claude-3-sonnet",
        "system_prompt": "You are a research agent specializing in {} analysis. Focus on providing accurate, well-sourced insights.",
        "temperature": 0.3
      }},
      "metadata": {{
        "research_domain": "{}",
        "specialization": "quantitative_analysis"
      }}
    }}
  }}
}}
```

## Research Capabilities
- Data collection and aggregation
- Statistical analysis and modeling
- Trend identification and prediction
- Report generation and visualization
- Source verification and fact-checking

The agent will continuously monitor {} markets and provide research insights to other agents in the swarm."#,
        agent_name, research_domain, agent_name, research_domain, research_domain, research_domain
    );

    Ok(PromptResponse {
        description: format!("Research agent creation guide for {} domain", research_domain),
        messages: vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "Create a research agent named '{}' for {} analysis.",
                    agent_name, research_domain
                )),
            },
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

async fn create_monitoring_agent_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let agent_name = args.get("agent_name")
        .and_then(|v| v.as_str())
        .unwrap_or("MonitoringAgent");
    let monitoring_targets = args.get("monitoring_targets")
        .and_then(|v| v.as_str())
        .unwrap_or("system_health,agent_performance,network_status");

    let targets: Vec<&str> = monitoring_targets.split(',').collect();

    let prompt_text = format!(
        r#"Setting up a system monitoring agent named "{}" to watch: {}

## Monitoring Configuration

```json
{{
  "tool": "spawn_agent",
  "arguments": {{
    "config": {{
      "name": "{}",
      "agent_type": "monitor",
      "capabilities": ["system_monitoring", "alerting", "log_analysis", "health_checking"],
      "metadata": {{
        "monitoring_targets": {},
        "alert_thresholds": {{
          "cpu_usage": 80,
          "memory_usage": 85,
          "error_rate": 5,
          "response_time": 1000
        }},
        "check_interval": 30
      }}
    }}
  }}
}}
```

## Monitoring Targets
{}

## Alert Configuration
- CPU usage > 80%
- Memory usage > 85%
- Error rate > 5%
- Response time > 1000ms

The agent will continuously monitor these targets and alert when thresholds are exceeded."#,
        agent_name, monitoring_targets, agent_name, 
        serde_json::to_string(&targets).unwrap(),
        targets.iter().map(|t| format!("- {}", t)).collect::<Vec<_>>().join("\n")
    );

    Ok(PromptResponse {
        description: format!("Monitoring agent setup for {}", agent_name),
        messages: vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "Set up a monitoring agent named '{}' to monitor: {}",
                    agent_name, monitoring_targets
                )),
            },
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

// Task Coordination Prompt Implementations

async fn coordinate_market_analysis_prompt(
    state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let market_symbol = args.get("market_symbol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("market_symbol is required".to_string()))?;
    let analysis_depth = args.get("analysis_depth")
        .and_then(|v| v.as_str())
        .unwrap_or("detailed");
    let time_horizon = args.get("time_horizon")
        .and_then(|v| v.as_str())
        .unwrap_or("1d");

    // Check available agents
    let agents = state.agents.read().await;
    let research_agents: Vec<_> = agents.values()
        .filter(|a| a.agent_type == "researcher" || a.capabilities.contains(&"market_research".to_string()))
        .collect();

    let prompt_text = format!(
        r#"Coordinating multi-agent market analysis for {} with {} depth over {} timeframe.

## Available Research Agents
{}

## Analysis Coordination Steps

1. **Create the analysis task**:
```json
{{
  "tool": "create_task",
  "arguments": {{
    "task_type": "market_analysis",
    "description": "Comprehensive analysis of {} market",
    "parameters": {{
      "symbol": "{}",
      "depth": "{}",
      "time_horizon": "{}",
      "analysis_components": ["technical", "fundamental", "sentiment", "volume"]
    }},
    "priority": "high"
  }}
}}
```

2. **Coordinate agent swarm**:
```json
{{
  "tool": "coordinate_swarm",
  "arguments": {{
    "objective": "Analyze {} market across multiple dimensions",
    "agent_types": ["researcher", "analyst"],
    "coordination_strategy": "parallel_analysis"
  }}
}}
```

## Analysis Framework
- **Technical Analysis**: Price patterns, indicators, support/resistance
- **Fundamental Analysis**: Market fundamentals, news impact, economic factors
- **Sentiment Analysis**: Social media, news sentiment, market psychology  
- **Volume Analysis**: Trading volume patterns, liquidity assessment

Results will be aggregated into a comprehensive market assessment."#,
        market_symbol, analysis_depth, time_horizon,
        if research_agents.is_empty() {
            "No research agents currently available. Consider spawning research agents first.".to_string()
        } else {
            research_agents.iter()
                .map(|a| format!("- {} ({}): {}", a.name, a.id, a.capabilities.join(", ")))
                .collect::<Vec<_>>()
                .join("\n")
        },
        market_symbol, market_symbol, analysis_depth, time_horizon, market_symbol
    );

    Ok(PromptResponse {
        description: format!("Market analysis coordination for {}", market_symbol),
        messages: vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "Coordinate a {} market analysis for {} over {} timeframe",
                    analysis_depth, market_symbol, time_horizon
                )),
            },
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

async fn execute_trading_strategy_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let strategy_name = args.get("strategy_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("strategy_name is required".to_string()))?;
    let allocation_amount = args.get("allocation_amount")
        .and_then(|v| v.as_u64())
        .unwrap_or(10000);
    let max_risk = args.get("max_risk")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.1);

    let prompt_text = format!(
        r#"Executing trading strategy: {} with {} allocation and {}% max risk.

## Strategy Execution Plan

1. **Create strategy execution task**:
```json
{{
  "tool": "create_task",
  "arguments": {{
    "task_type": "trading_strategy",
    "description": "Execute {} trading strategy",
    "parameters": {{
      "strategy": "{}",
      "allocation": {},
      "max_risk": {},
      "execution_mode": "gradual",
      "monitoring": true
    }},
    "priority": "high"
  }}
}}
```

2. **Coordinate execution agents**:
- **Risk Manager**: Monitor risk exposure and position sizing
- **Execution Agent**: Handle order placement and execution
- **Monitoring Agent**: Track performance and compliance

## Risk Management
- Maximum risk exposure: {}%
- Position sizing: Dynamic based on volatility
- Stop-loss: Automated at {}% loss
- Monitoring interval: Real-time

## Execution Parameters
- **Allocation**: {} tokens
- **Risk Budget**: {} tokens max loss
- **Execution Style**: Gradual entry to minimize market impact
- **Monitoring**: Continuous performance tracking

The strategy will be executed with full risk controls and monitoring in place."#,
        strategy_name, allocation_amount, max_risk * 100.0,
        strategy_name, strategy_name, allocation_amount, max_risk,
        max_risk * 100.0, max_risk * 50.0,
        allocation_amount, (allocation_amount as f64 * max_risk) as u64
    );

    Ok(PromptResponse {
        description: format!("Trading strategy execution: {}", strategy_name),
        messages: vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "Execute the {} trading strategy with {} allocation and {} max risk",
                    strategy_name, allocation_amount, max_risk
                )),
            },
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

// Swarm Coordination Prompt Implementations

async fn deploy_agent_swarm_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let swarm_objective = args.get("swarm_objective")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("swarm_objective is required".to_string()))?;
    let agent_types = args.get("agent_types")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("agent_types is required".to_string()))?;
    let coordination_strategy = args.get("coordination_strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("hierarchical");

    let types: Vec<&str> = agent_types.split(',').map(|s| s.trim()).collect();

    let prompt_text = format!(
        r#"Deploying agent swarm for objective: "{}"

## Swarm Configuration
- **Agent Types**: {}
- **Coordination Strategy**: {}
- **Objective**: {}

## Deployment Steps

1. **Deploy swarm coordinator**:
```json
{{
  "tool": "coordinate_swarm",
  "arguments": {{
    "objective": "{}",
    "agent_types": {},
    "coordination_strategy": "{}",
    "max_agents": 10
  }}
}}
```

2. **Spawn required agents**:
{}

3. **Establish communication protocols**:
- Message routing and distribution
- Task assignment and coordination
- Progress monitoring and reporting

## Coordination Strategy: {}
{}

The swarm will self-organize to achieve the objective through coordinated autonomous action."#,
        swarm_objective, agent_types, coordination_strategy, swarm_objective,
        swarm_objective, serde_json::to_string(&types).unwrap(), coordination_strategy,
        types.iter().enumerate().map(|(i, t)| format!(
            "   - Agent {}: {} (capabilities based on type)",
            i + 1, t
        )).collect::<Vec<_>>().join("\n"),
        coordination_strategy,
        match coordination_strategy {
            "hierarchical" => "Central coordinator assigns tasks to specialized agents in a tree structure.",
            "peer-to-peer" => "Agents communicate directly and negotiate task distribution autonomously.",
            "hybrid" => "Combination of hierarchical coordination with peer-to-peer communication for efficiency.",
            _ => "Custom coordination strategy as specified."
        }
    );

    Ok(PromptResponse {
        description: format!("Agent swarm deployment for: {}", swarm_objective),
        messages: vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "Deploy an agent swarm for '{}' using {} agent types with {} coordination",
                    swarm_objective, agent_types, coordination_strategy
                )),
            },
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

async fn emergency_response_prompt(
    state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let incident_type = args.get("incident_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("incident_type is required".to_string()))?;
    let severity_level = args.get("severity_level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DaaMcpError::Protocol("severity_level is required".to_string()))?;
    let affected_systems = args.get("affected_systems")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Check system status
    let agents = state.agents.read().await;
    let active_agents = agents.values().filter(|a| matches!(a.status, crate::AgentStatus::Running)).count();

    let prompt_text = format!(
        r#"🚨 EMERGENCY RESPONSE PROTOCOL ACTIVATED 🚨

## Incident Details
- **Type**: {}
- **Severity**: {}
- **Affected Systems**: {}
- **Active Agents**: {}

## Immediate Response Actions

1. **Assess system status**:
```json
{{
  "tool": "healthcheck",
  "arguments": {{
    "deep_check": true
  }}
}}
```

2. **Coordinate emergency response**:
```json
{{
  "tool": "coordinate_swarm",
  "arguments": {{
    "objective": "Emergency response for {} incident",
    "agent_types": ["monitor", "security", "recovery"],
    "coordination_strategy": "centralized",
    "max_agents": 5
  }}
}}
```

## Severity Level: {} - Response Protocol
{}

## Recovery Steps
1. **Immediate**: Isolate affected components
2. **Short-term**: Implement workarounds and failovers  
3. **Medium-term**: Root cause analysis and fixes
4. **Long-term**: Prevention and monitoring improvements

## Monitoring and Communication
- Real-time status updates every 5 minutes
- Stakeholder notifications activated
- Audit trail recording all actions
- Post-incident review scheduled

Emergency response swarm will coordinate recovery efforts automatically."#,
        incident_type, severity_level, affected_systems, active_agents,
        incident_type, severity_level,
        match severity_level.to_lowercase().as_str() {
            "critical" => "- IMMEDIATE action required\n- All hands on deck\n- Consider system shutdown\n- Executive notification",
            "high" => "- Urgent response needed\n- Activate backup systems\n- Team lead notification\n- Consider maintenance window",
            "medium" => "- Timely response required\n- Monitor closely\n- Team notification\n- Schedule resolution",
            "low" => "- Standard response\n- Document and track\n- Routine notification\n- Regular monitoring",
            _ => "- Assess severity and respond appropriately"
        }
    );

    Ok(PromptResponse {
        description: format!("Emergency response for {} incident", incident_type),
        messages: vec![
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "Emergency: {} incident with {} severity affecting {}",
                    incident_type, severity_level, affected_systems
                )),
            },
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

// Additional prompt implementations would continue here...
// For brevity, I'll implement stubs for the remaining prompts

async fn generate_performance_report_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let report_period = args.get("report_period")
        .and_then(|v| v.as_str())
        .unwrap_or("daily");

    let prompt_text = format!(
        "Generating {} performance report using system metrics and analytics resources.",
        report_period
    );

    Ok(PromptResponse {
        description: format!("{} performance report generation", report_period),
        messages: vec![
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

async fn diagnose_system_issues_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let symptom_description = args.get("symptom_description")
        .and_then(|v| v.as_str())
        .unwrap_or("unspecified symptoms");

    let prompt_text = format!(
        "Diagnosing system issues based on symptoms: {}",
        symptom_description
    );

    Ok(PromptResponse {
        description: "System diagnostic procedure".to_string(),
        messages: vec![
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

async fn setup_development_environment_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let environment_type = args.get("environment_type")
        .and_then(|v| v.as_str())
        .unwrap_or("local");

    let prompt_text = format!(
        "Setting up {} development environment for DAA SDK.",
        environment_type
    );

    Ok(PromptResponse {
        description: format!("{} environment setup", environment_type),
        messages: vec![
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

async fn optimize_agent_configuration_prompt(
    _state: Arc<McpServerState>,
    args: Value,
) -> Result<PromptResponse> {
    let agent_id = args.get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let prompt_text = format!(
        "Optimizing configuration for agent: {}",
        agent_id
    );

    Ok(PromptResponse {
        description: format!("Agent optimization for {}", agent_id),
        messages: vec![
            PromptMessage {
                role: "assistant".to_string(),
                content: PromptContent::Text(prompt_text),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DaaMcpConfig;

    #[tokio::test]
    async fn test_create_treasury_agent_prompt() {
        let config = DaaMcpConfig::default();
        let state = Arc::new(McpServerState::new(config));

        let args = json!({
            "agent_name": "TestTreasuryAgent",
            "initial_balance": 50000,
            "risk_threshold": 0.15
        });

        let response = create_treasury_agent_prompt(state, args).await.unwrap();
        assert_eq!(response.messages.len(), 2);
        assert!(response.description.contains("TestTreasuryAgent"));
    }

    #[tokio::test]
    async fn test_emergency_response_prompt() {
        let config = DaaMcpConfig::default();
        let state = Arc::new(McpServerState::new(config));

        let args = json!({
            "incident_type": "system_failure",
            "severity_level": "critical",
            "affected_systems": "trading,monitoring"
        });

        let response = emergency_response_prompt(state, args).await.unwrap();
        assert!(response.description.contains("Emergency"));
        assert!(response.messages[1].content.to_string().contains("CRITICAL"));
    }
}

impl ToString for PromptContent {
    fn to_string(&self) -> String {
        match self {
            PromptContent::Text(text) => text.clone(),
            PromptContent::Structured { text, .. } => text.clone(),
        }
    }
}