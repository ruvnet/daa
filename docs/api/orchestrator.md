# 🎛️ Orchestrator API Reference

> **Core orchestration engine for DAA agents** - Coordinates all components and manages the autonomy loop.

The `daa-orchestrator` crate provides the central coordination layer for Decentralized Autonomous Agents, managing workflows, services, and integrations through the QuDAG protocol.

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
daa-orchestrator = "0.2.0"
```

## 🚀 Quick Start

```rust
use daa_orchestrator::{DaaOrchestrator, OrchestratorConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = OrchestratorConfig::default();
    let orchestrator = DaaOrchestrator::new(config).await?;
    
    // Start the orchestrator
    orchestrator.start().await?;
    
    Ok(())
}
```

---

## 🏗️ Core Types

### `DaaOrchestrator`

The main orchestrator struct that coordinates all DAA operations.

```rust
pub struct DaaOrchestrator {
    // Internal fields...
}
```

#### Methods

##### `new(config: OrchestratorConfig) -> Result<Self>`

Creates a new orchestrator instance with the given configuration.

**Parameters:**
- `config`: Configuration for the orchestrator

**Returns:** `Result<DaaOrchestrator, OrchestratorError>`

**Example:**
```rust
let config = OrchestratorConfig {
    node: NodeConfig::default(),
    coordination: CoordinationConfig::default(),
    services: ServiceConfig::default(),
    workflows: WorkflowConfig::default(),
    integrations: IntegrationConfig::default(),
};

let orchestrator = DaaOrchestrator::new(config).await?;
```

##### `start() -> Result<()>`

Starts the orchestrator and begins processing.

**Returns:** `Result<(), OrchestratorError>`

**Example:**
```rust
orchestrator.start().await?;
```

##### `stop() -> Result<()>`

Gracefully stops the orchestrator.

**Returns:** `Result<(), OrchestratorError>`

**Example:**
```rust
orchestrator.stop().await?;
```

##### `submit_workflow(workflow: Workflow) -> Result<WorkflowId>`

Submits a workflow for execution.

**Parameters:**
- `workflow`: The workflow to execute

**Returns:** `Result<WorkflowId, OrchestratorError>`

**Example:**
```rust
let workflow = Workflow::new("treasury_management")
    .add_step(Step::Monitor { interval: Duration::from_secs(60) })
    .add_step(Step::Analyze { criteria: "risk_threshold < 0.2" })
    .add_step(Step::Execute { action: "rebalance_portfolio" });

let workflow_id = orchestrator.submit_workflow(workflow).await?;
```

---

## ⚙️ Configuration

### `OrchestratorConfig`

Main configuration struct for the orchestrator.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub node: NodeConfig,
    pub coordination: CoordinationConfig,
    pub services: ServiceConfig,
    pub workflows: WorkflowConfig,
    pub integrations: IntegrationConfig,
}
```

#### Fields

- **`node`**: QuDAG protocol node configuration
- **`coordination`**: Coordination settings for multi-agent scenarios
- **`services`**: Service registry configuration
- **`workflows`**: Workflow engine configuration
- **`integrations`**: External integration settings

### `CoordinationConfig`

Configuration for multi-agent coordination.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    pub max_agents: usize,
    pub consensus_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub leader_election: bool,
}
```

#### Default Values

```rust
CoordinationConfig {
    max_agents: 10,
    consensus_timeout: Duration::from_secs(30),
    heartbeat_interval: Duration::from_secs(5),
    leader_election: true,
}
```

### `ServiceConfig`

Configuration for the service registry.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub discovery_interval: Duration,
    pub health_check_timeout: Duration,
    pub retry_attempts: usize,
}
```

### `WorkflowConfig`

Configuration for workflow execution.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub max_concurrent_workflows: usize,
    pub step_timeout: Duration,
    pub retry_policy: RetryPolicy,
}
```

### `IntegrationConfig`

Configuration for external integrations.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub chain_enabled: bool,
    pub economy_enabled: bool,
    pub rules_enabled: bool,
    pub ai_enabled: bool,
}
```

---

## 🔄 Workflow System

### `Workflow`

Represents a workflow to be executed by the orchestrator.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: WorkflowId,
    pub name: String,
    pub steps: Vec<Step>,
    pub metadata: HashMap<String, String>,
}
```

#### Methods

##### `new(name: &str) -> Self`

Creates a new workflow with the given name.

**Example:**
```rust
let workflow = Workflow::new("portfolio_rebalancing");
```

##### `add_step(mut self, step: Step) -> Self`

Adds a step to the workflow.

**Example:**
```rust
let workflow = Workflow::new("trading_strategy")
    .add_step(Step::Monitor { 
        interval: Duration::from_secs(30),
        condition: "market_volatility > 0.1".to_string(),
    })
    .add_step(Step::Analyze { 
        criteria: "profit_potential > 0.05".to_string(),
    })
    .add_step(Step::Execute { 
        action: "place_order".to_string(),
        parameters: vec!["amount=1000", "type=market"],
    });
```

### `Step`

Individual workflow steps.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Step {
    Monitor {
        interval: Duration,
        condition: String,
    },
    Analyze {
        criteria: String,
    },
    Execute {
        action: String,
        parameters: Vec<String>,
    },
    Conditional {
        condition: String,
        then_step: Box<Step>,
        else_step: Option<Box<Step>>,
    },
    Parallel {
        steps: Vec<Step>,
    },
    Sequential {
        steps: Vec<Step>,
    },
}
```

---

## 🔌 Services

### Service Registry

The orchestrator maintains a registry of available services.

#### `register_service(service: Service) -> Result<()>`

Registers a new service with the orchestrator.

**Example:**
```rust
let price_service = Service::new("price_oracle")
    .with_endpoint("https://api.example.com/prices")
    .with_health_check("/health");

orchestrator.register_service(price_service).await?;
```

#### `discover_services() -> Result<Vec<Service>>`

Discovers available services in the network.

**Example:**
```rust
let services = orchestrator.discover_services().await?;
for service in services {
    println!("Found service: {}", service.name);
}
```

---

## 🌐 Network Operations

### Node Management

#### `get_node_status() -> NodeStatus`

Gets the current status of the node.

**Example:**
```rust
let status = orchestrator.get_node_status().await;
println!("Node ID: {}", status.node_id);
println!("Peers: {}", status.peer_count);
```

#### `connect_to_peer(peer_id: PeerId) -> Result<()>`

Connects to a specific peer.

**Example:**
```rust
let peer_id = PeerId::from_str("12D3KooW...")?;
orchestrator.connect_to_peer(peer_id).await?;
```

---

## 📊 Metrics and Monitoring

### Health Checks

#### `health_check() -> HealthStatus`

Performs a comprehensive health check.

**Example:**
```rust
let health = orchestrator.health_check().await;
if health.is_healthy() {
    println!("System is healthy");
} else {
    println!("Issues detected: {:?}", health.issues);
}
```

### Metrics Collection

#### `get_metrics() -> Metrics`

Collects system metrics.

**Example:**
```rust
let metrics = orchestrator.get_metrics().await;
println!("Workflows executed: {}", metrics.workflows_completed);
println!("Average response time: {}ms", metrics.avg_response_time);
```

---

## 🚨 Error Handling

### `OrchestratorError`

Main error type for orchestrator operations.

```rust
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] qudag_protocol::ProtocolError),
    
    #[error("Service error: {0}")]
    Service(String),
    
    #[error("Workflow error: {0}")]
    Workflow(String),
    
    #[error("Coordination error: {0}")]
    Coordination(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}
```

### Error Handling Best Practices

```rust
use daa_orchestrator::{DaaOrchestrator, OrchestratorError};

async fn handle_orchestrator_operations() {
    match orchestrator.start().await {
        Ok(()) => println!("Orchestrator started successfully"),
        Err(OrchestratorError::Configuration(msg)) => {
            eprintln!("Configuration error: {}", msg);
            // Handle configuration issues
        },
        Err(OrchestratorError::Protocol(err)) => {
            eprintln!("Protocol error: {}", err);
            // Handle protocol issues
        },
        Err(err) => {
            eprintln!("Unexpected error: {}", err);
            // Handle other errors
        }
    }
}
```

---

## 🔧 Advanced Usage

### Custom Workflows

```rust
use daa_orchestrator::workflow::{Workflow, Step, Condition};

let advanced_workflow = Workflow::new("advanced_trading")
    .add_step(Step::Conditional {
        condition: "market_trend == 'bullish'".to_string(),
        then_step: Box::new(Step::Execute {
            action: "increase_position".to_string(),
            parameters: vec!["multiplier=1.5"],
        }),
        else_step: Some(Box::new(Step::Execute {
            action: "reduce_position".to_string(),
            parameters: vec!["multiplier=0.8"],
        })),
    })
    .add_step(Step::Parallel {
        steps: vec![
            Step::Monitor { 
                interval: Duration::from_secs(10),
                condition: "risk_level".to_string(),
            },
            Step::Analyze { 
                criteria: "technical_indicators".to_string(),
            },
        ],
    });
```

### Multi-Agent Coordination

```rust
use daa_orchestrator::coordination::{CoordinationProtocol, AgentRole};

let coordinator = orchestrator.coordination_manager();

// Register as a coordinator
coordinator.register_role(AgentRole::Coordinator).await?;

// Coordinate with other agents
let consensus = coordinator.reach_consensus(
    "portfolio_allocation",
    vec!["conservative", "moderate", "aggressive"]
).await?;

println!("Consensus reached: {}", consensus);
```

---

## 📚 Examples

### Complete Treasury Management Agent

```rust
use daa_orchestrator::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the orchestrator
    let config = OrchestratorConfig {
        node: NodeConfig::new().with_port(8080),
        coordination: CoordinationConfig {
            max_agents: 5,
            consensus_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
            leader_election: true,
        },
        services: ServiceConfig::default(),
        workflows: WorkflowConfig {
            max_concurrent_workflows: 10,
            step_timeout: Duration::from_secs(60),
            retry_policy: RetryPolicy::exponential(3),
        },
        integrations: IntegrationConfig {
            chain_enabled: true,
            economy_enabled: true,
            rules_enabled: true,
            ai_enabled: true,
        },
    };

    // Create and start orchestrator
    let orchestrator = DaaOrchestrator::new(config).await?;
    orchestrator.start().await?;

    // Define treasury management workflow
    let treasury_workflow = Workflow::new("treasury_management")
        .add_step(Step::Monitor {
            interval: Duration::from_secs(300), // 5 minutes
            condition: "treasury_balance_changed".to_string(),
        })
        .add_step(Step::Analyze {
            criteria: "risk_assessment AND yield_opportunities".to_string(),
        })
        .add_step(Step::Conditional {
            condition: "rebalancing_required".to_string(),
            then_step: Box::new(Step::Sequential {
                steps: vec![
                    Step::Execute {
                        action: "calculate_optimal_allocation".to_string(),
                        parameters: vec!["risk_tolerance=0.2"],
                    },
                    Step::Execute {
                        action: "execute_rebalancing".to_string(),
                        parameters: vec!["slippage_tolerance=0.005"],
                    },
                ],
            }),
            else_step: Some(Box::new(Step::Execute {
                action: "log_no_action_needed".to_string(),
                parameters: vec![],
            })),
        });

    // Submit the workflow
    let workflow_id = orchestrator.submit_workflow(treasury_workflow).await?;
    println!("Treasury workflow submitted with ID: {}", workflow_id);

    // Keep running
    orchestrator.run_forever().await
}
```

---

## 🔗 Related Documentation

- [Rules Engine API](./rules.md) - Governance and decision making
- [Economy API](./economy.md) - Token management and economics
- [AI Integration API](./ai.md) - Claude AI and MCP integration
- [Chain API](./chain.md) - Blockchain abstraction layer
- [Architecture Overview](../architecture/README.md) - System design

---

## 🐛 Troubleshooting

### Common Issues

**Orchestrator fails to start:**
```
Error: Configuration error: Invalid node configuration
```

**Solution:** Check your `NodeConfig` settings, ensure ports are available.

**Workflow execution timeout:**
```
Error: Workflow error: Step timeout exceeded
```

**Solution:** Increase `step_timeout` in `WorkflowConfig` or optimize your workflow steps.

**Service discovery fails:**
```
Error: Service error: No services found
```

**Solution:** Ensure services are properly registered and network connectivity is available.

---

## 📊 Performance Considerations

### Optimization Tips

1. **Batch Operations**: Group related operations together
2. **Async Processing**: Use async/await for I/O operations
3. **Resource Limits**: Set appropriate limits in configuration
4. **Monitoring**: Enable metrics collection for performance insights

### Benchmarks

| Operation | Throughput | Latency |
|-----------|------------|---------|
| Workflow submission | 1000/sec | <1ms |
| Service discovery | 100/sec | <10ms |
| Node coordination | 50/sec | <50ms |

---

## 🔄 Migration Guide

### From v0.1.x to v0.2.x

**Breaking Changes:**
- `OrchestratorConfig` structure changed
- `Workflow` API updated
- Service registration requires health checks

**Migration Steps:**
1. Update configuration structure
2. Update workflow definitions
3. Add health check endpoints to services

---

*For more detailed API documentation, see the [rustdoc documentation](https://docs.rs/daa-orchestrator).*