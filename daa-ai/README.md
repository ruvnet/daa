# DAA AI

**🚀 FULL IMPLEMENTATION - This is the complete, production-ready implementation of the DAA AI module, not a placeholder.**

AI integration layer for the Decentralized Autonomous Agents (DAA) system, providing Claude AI integration via QuDAG MCP (Model Context Protocol) for intelligent decision making and task automation.

## Overview

DAA AI enables autonomous agents to leverage Claude's advanced AI capabilities through QuDAG's MCP integration, providing:

- **Claude Integration**: Direct integration with Anthropic's Claude models
- **Agent Management**: Spawn and manage specialized AI agents
- **Task Execution**: Automated task processing with AI reasoning
- **Tool Integration**: MCP-based tool usage for extended capabilities
- **Memory Management**: Persistent memory for agent learning and context

## Features

### Core AI Capabilities
- **Claude API Integration**: Native support for Claude 3 models
- **MCP Protocol**: QuDAG MCP integration for tool usage
- **Agent Spawning**: Create specialized agents (Researcher, Coder, Analyst)
- **Task Automation**: AI-powered task execution and reasoning
- **Memory System**: Persistent memory for agent context and learning

### Agent Types
- **Researcher**: Information gathering and analysis
- **Coder**: Code generation and software development
- **Analyst**: Data analysis and pattern recognition
- **Coordinator**: Multi-agent task coordination
- **Specialist**: Domain-specific expertise

### Tool Integration
- **Code Execution**: Execute code via MCP tools
- **File Operations**: File system operations
- **Web Search**: Internet research capabilities
- **Data Analysis**: Statistical and analytical tools

## Usage

### Basic Setup

```rust
use daa_ai::{AISystem, AIConfig, agents::AgentType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure AI system
    let config = AIConfig {
        claude: claude::ClaudeConfig {
            api_key: "your-anthropic-api-key".to_string(),
            model: "claude-3-opus-20240229".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Initialize AI system
    let mut ai_system = AISystem::new(config).await?;
    ai_system.initialize().await?;

    // Spawn an AI agent
    let agent_id = ai_system.spawn_agent(
        AgentType::Researcher,
        Some(vec!["web_search".to_string(), "analysis".to_string()]),
        None,
    ).await?;

    println!("Spawned agent: {}", agent_id);
    Ok(())
}
```

### Task Execution

```rust
use daa_ai::tasks::Task;

// Create a task
let task = Task {
    id: uuid::Uuid::new_v4().to_string(),
    task_type: "research".to_string(),
    description: "Research recent developments in quantum computing".to_string(),
    parameters: serde_json::json!({
        "topic": "quantum computing",
        "time_range": "last_6_months",
        "sources": ["arxiv", "nature", "science"]
    }),
};

// Execute task with agent
let result = ai_system.execute_task(&agent_id, task).await?;
println!("Task result: {:?}", result);
```

### Tool Usage

```rust
use std::collections::HashMap;

// Use a tool via MCP
let mut parameters = HashMap::new();
parameters.insert("query".to_string(), serde_json::json!("latest AI research"));
parameters.insert("limit".to_string(), serde_json::json!(10));

let tool_result = ai_system.use_tool(
    &agent_id,
    "web_search",
    parameters,
).await?;

println!("Tool result: {:?}", tool_result);
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│    AISystem     │    │  ClaudeClient   │    │   MCPClient     │
│                 │    │                 │    │                 │
│ - Agent Mgmt    │◄──►│ - API Calls     │◄──►│ - Tool Calls    │
│ - Task Coord    │    │ - Model Config  │    │ - Protocol Mgmt │
│ - Memory Mgmt   │    │ - Response Parse│    │ - Connection    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  AgentManager   │    │  TaskManager    │    │  ToolRegistry   │
│                 │    │                 │    │                 │
│ - Agent Spawn   │    │ - Task Queue    │    │ - Tool Catalog  │
│ - Capabilities  │    │ - Execution     │    │ - MCP Tools     │
│ - Lifecycle     │    │ - Results       │    │ - Custom Tools  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Features

The crate supports several feature flags:

- `default`: Includes MCP and Claude features
- `mcp`: Enables QuDAG MCP integration
- `claude`: Claude API support (always enabled)
- `rules-integration`: Integration with DAA Rules engine
- `database`: Persistent storage for agents and tasks
- `full`: All features enabled

```toml
[dependencies]
daa-ai = { version = "0.1.0", features = ["full"] }
```

## Configuration

### Claude Configuration

```rust
use daa_ai::claude::ClaudeConfig;

let claude_config = ClaudeConfig {
    api_key: std::env::var("ANTHROPIC_API_KEY")?,
    model: "claude-3-opus-20240229".to_string(),
    endpoint: "https://api.anthropic.com".to_string(),
    timeout: 60,
};
```

### MCP Configuration

```rust
use daa_ai::MCPClientConfig;

let mcp_config = MCPClientConfig {
    server_url: "http://localhost:3000".to_string(),
    timeout: 30,
    max_connections: 10,
    retry_attempts: 3,
    available_tools: vec![
        "code_execution".to_string(),
        "file_operations".to_string(),
        "web_search".to_string(),
    ],
};
```

## Integration with DAA System

### With DAA Rules
```rust
#[cfg(feature = "rules-integration")]
use daa_ai::rules_integration::RulesIntegration;

// AI agents can leverage rules for decision making
let rules_integration = RulesIntegration::new(rules_engine);
ai_system.add_rules_integration(rules_integration).await?;
```

### With DAA Economy
```rust
// AI agents can interact with the economic system
let economic_task = Task {
    id: uuid::Uuid::new_v4().to_string(),
    task_type: "economic_analysis".to_string(),
    description: "Analyze token distribution patterns".to_string(),
    parameters: serde_json::json!({
        "token": "rUv",
        "time_period": "30_days",
        "metrics": ["distribution", "velocity", "concentration"]
    }),
};
```

## License

MIT OR Apache-2.0