# DAA Rules

**🚀 FULL IMPLEMENTATION - This is the complete, production-ready implementation of the DAA Rules Engine, not a placeholder.**

A comprehensive rules engine for the Decentralized Autonomous Agents (DAA) system, providing policy enforcement, decision automation, and governance capabilities.

## Overview

DAA Rules provides a flexible and powerful rules engine that enables autonomous agents to make decisions based on predefined policies and conditions. The engine supports complex logical operations, pattern matching, time-based conditions, and various action types.

## Features

### Core Rule Engine
- **Rule Definition**: Flexible rule creation with conditions and actions
- **Condition Evaluation**: Support for equality, comparison, pattern matching, and logical operations
- **Action Execution**: Set fields, log messages, send notifications, and custom actions
- **Context Management**: Execution context with variables and metadata

### Advanced Features
- **Complex Logic**: AND, OR, NOT operations for sophisticated rule conditions
- **Pattern Matching**: Regular expression support for text matching
- **Time-based Rules**: Temporal conditions and scheduling
- **Priority System**: Rule execution order based on priority levels
- **Validation**: Comprehensive rule and condition validation

### Optional Features
- **Scripting Support**: Execute custom scripts within rules (with `scripting` feature)
- **Database Storage**: Persistent rule storage (with `database` feature)
- **Custom Extensions**: Pluggable condition and action types

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   RuleEngine    │    │ConditionEvaluator│    │ ActionExecutor  │
│                 │    │                 │    │                 │
│ - Rule Storage  │◄──►│ - Equals/Compare│◄──►│ - SetField      │
│ - Execution     │    │ - Pattern Match │    │ - Log Message   │
│ - Validation    │    │ - Logic Ops     │    │ - Notifications │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ExecutionContext │    │   RuleStorage   │    │   Scripting     │
│                 │    │                 │    │   (Optional)    │
│ - Variables     │    │ - InMemory      │    │ - Rhai Engine   │
│ - Metadata      │    │ - Database      │    │ - Custom Scripts│
│ - Timestamp     │    │ - File System   │    │ - Extensions    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Usage

### Basic Rule Creation

```rust
use daa_rules::{Rule, RuleCondition, RuleAction, LogLevel};

// Create a simple rule
let rule = Rule::new_with_generated_id(
    "Agent Status Check".to_string(),
    vec![
        RuleCondition::Equals {
            field: "agent_status".to_string(),
            value: "active".to_string(),
        },
        RuleCondition::GreaterThan {
            field: "performance_score".to_string(),
            value: 0.8,
        },
    ],
    vec![
        RuleAction::Log {
            level: LogLevel::Info,
            message: "High-performing active agent detected".to_string(),
        },
        RuleAction::SetField {
            field: "eligibility".to_string(),
            value: "premium".to_string(),
        },
    ],
);

println!("Created rule: {}", rule.name);
```

### Rules Engine Setup

```rust
use daa_rules::{RuleEngine, ExecutionContext};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create rules engine
    let mut engine = RuleEngine::new();
    
    // Add rule to engine
    engine.add_rule(rule).await?;
    
    // Create execution context
    let mut context = ExecutionContext::new();
    context.set_variable("agent_status".to_string(), "active".to_string());
    context.set_variable("performance_score".to_string(), "0.85".to_string());
    
    // Execute rule
    let result = engine.execute_rule(&rule, &mut context).await?;
    println!("Rule execution result: {}", result);
    
    Ok(())
}
```

### Complex Conditions

```rust
use daa_rules::{RuleCondition, TimeOperator};
use chrono::{Utc, Duration};

// Complex logical condition
let complex_condition = RuleCondition::And {
    conditions: vec![
        RuleCondition::Or {
            conditions: vec![
                RuleCondition::Equals {
                    field: "task_type".to_string(),
                    value: "compute".to_string(),
                },
                RuleCondition::Equals {
                    field: "task_type".to_string(),
                    value: "storage".to_string(),
                },
            ],
        },
        RuleCondition::Not {
            condition: Box::new(RuleCondition::Equals {
                field: "agent_status".to_string(),
                value: "suspended".to_string(),
            }),
        },
        RuleCondition::TimeCondition {
            field: "last_active".to_string(),
            operator: TimeOperator::After,
            value: Utc::now() - Duration::hours(24),
        },
    ],
};
```

### Pattern Matching

```rust
// Email validation rule
let email_rule = Rule::new_with_generated_id(
    "Email Validation".to_string(),
    vec![
        RuleCondition::Matches {
            field: "email".to_string(),
            pattern: r"^[^@]+@[^@]+\.[^@]+$".to_string(),
        },
    ],
    vec![
        RuleAction::SetField {
            field: "email_valid".to_string(),
            value: "true".to_string(),
        },
    ],
);

// IP address validation
let ip_rule = Rule::new_with_generated_id(
    "IP Address Check".to_string(),
    vec![
        RuleCondition::Matches {
            field: "client_ip".to_string(),
            pattern: r"^192\.168\.1\.\d{1,3}$".to_string(),
        },
    ],
    vec![
        RuleAction::SetField {
            field: "network_zone".to_string(),
            value: "internal".to_string(),
        },
    ],
);
```

### Action Types

```rust
use daa_rules::{RuleAction, LogLevel, NotificationChannel};
use std::collections::HashMap;

// Logging action
let log_action = RuleAction::Log {
    level: LogLevel::Warn,
    message: "Unusual activity detected".to_string(),
};

// Field modification
let set_field_action = RuleAction::SetField {
    field: "alert_level".to_string(),
    value: "high".to_string(),
};

// Notification
let notify_action = RuleAction::Notify {
    recipient: "admin@example.com".to_string(),
    message: "Security alert triggered".to_string(),
    channel: NotificationChannel::Email,
};

// Context modification
let mut modifications = HashMap::new();
modifications.insert("processed".to_string(), "true".to_string());
modifications.insert("processor".to_string(), "security_agent".to_string());

let modify_context_action = RuleAction::ModifyContext {
    modifications,
};

// Webhook trigger
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer token".to_string());
headers.insert("Content-Type".to_string(), "application/json".to_string());

let webhook_action = RuleAction::Webhook {
    url: "https://api.example.com/alerts".to_string(),
    method: "POST".to_string(),
    headers,
    body: r#"{"alert": "security_event", "level": "high"}"#.to_string(),
};
```

## Rule Definition Examples

### Agent Performance Monitoring

```rust
let performance_rule = Rule::new_with_generated_id(
    "Performance Monitoring".to_string(),
    vec![
        RuleCondition::LessThan {
            field: "success_rate".to_string(),
            value: 0.7, // Below 70% success rate
        },
        RuleCondition::GreaterThan {
            field: "task_count".to_string(),
            value: 10.0, // More than 10 tasks
        },
    ],
    vec![
        RuleAction::Log {
            level: LogLevel::Warn,
            message: "Agent performance below threshold".to_string(),
        },
        RuleAction::SetField {
            field: "performance_status".to_string(),
            value: "review_required".to_string(),
        },
        RuleAction::Notify {
            recipient: "performance_monitor".to_string(),
            message: "Agent requires performance review".to_string(),
            channel: NotificationChannel::Internal,
        },
    ],
);
```

### Resource Allocation

```rust
let resource_rule = Rule::new_with_generated_id(
    "Resource Allocation".to_string(),
    vec![
        RuleCondition::In {
            field: "agent_tier".to_string(),
            values: vec!["premium".to_string(), "enterprise".to_string()],
        },
        RuleCondition::LessThan {
            field: "current_load".to_string(),
            value: 0.8, // Below 80% load
        },
    ],
    vec![
        RuleAction::SetField {
            field: "resource_allocation".to_string(),
            value: "high".to_string(),
        },
        RuleAction::SetField {
            field: "priority_queue".to_string(),
            value: "fast_track".to_string(),
        },
    ],
);
```

### Security Policy

```rust
let security_rule = Rule::new_with_generated_id(
    "Security Policy".to_string(),
    vec![
        RuleCondition::And {
            conditions: vec![
                RuleCondition::Matches {
                    field: "request_path".to_string(),
                    pattern: r"/admin/.*".to_string(),
                },
                RuleCondition::NotEquals {
                    field: "user_role".to_string(),
                    value: "administrator".to_string(),
                },
            ],
        },
    ],
    vec![
        RuleAction::Log {
            level: LogLevel::Error,
            message: "Unauthorized admin access attempt".to_string(),
        },
        RuleAction::Abort {
            reason: "Insufficient privileges for admin access".to_string(),
        },
    ],
);
```

## Configuration

### Basic Configuration

```rust
use daa_rules::RuleEngine;

// Create engine with default settings
let engine = RuleEngine::new();
```

### With Database Storage

```toml
[dependencies]
daa-rules = { version = "0.2.0", features = ["database"] }
```

```rust
#[cfg(feature = "database")]
use daa_rules::database::DatabaseStorage;

// Create engine with database storage
let storage = DatabaseStorage::new("sqlite://rules.db").await?;
let mut engine = RuleEngine::with_storage(Box::new(storage));
```

### With Scripting Support

```toml
[dependencies]
daa-rules = { version = "0.2.0", features = ["scripting"] }
```

```rust
#[cfg(feature = "scripting")]
use daa_rules::RuleAction;

let script_action = RuleAction::Script {
    script_type: "rhai".to_string(),
    script: r#"
        let result = performance_score * quality_multiplier;
        if result > 0.9 {
            set_field("bonus_eligible", "true");
        }
    "#.to_string(),
};
```

## Features

The crate supports several feature flags:

- `default`: Basic rules engine functionality
- `basic`: Core features only (same as default)
- `scripting`: Enables Rhai scripting support for custom actions
- `database`: Adds SQLite database storage for rules
- `full`: Includes all features

```toml
[dependencies]
daa-rules = { version = "0.2.0", features = ["full"] }
```

## Integration with DAA System

### With DAA Chain

```rust
// Rules can be integrated with blockchain operations
let chain_rule = Rule::new_with_generated_id(
    "Blockchain Validation".to_string(),
    vec![
        RuleCondition::GreaterThan {
            field: "transaction_amount".to_string(),
            value: 1000.0,
        },
    ],
    vec![
        RuleAction::Custom {
            action_type: "blockchain_verify".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("verification_level".to_string(), "enhanced".to_string());
                params
            },
        },
    ],
);
```

### With DAA Economy

```rust
// Economic policy enforcement
let economic_rule = Rule::new_with_generated_id(
    "Economic Policy".to_string(),
    vec![
        RuleCondition::And {
            conditions: vec![
                RuleCondition::Equals {
                    field: "transaction_type".to_string(),
                    value: "reward".to_string(),
                },
                RuleCondition::GreaterThan {
                    field: "performance_score".to_string(),
                    value: 0.95,
                },
            ],
        },
    ],
    vec![
        RuleAction::Custom {
            action_type: "apply_bonus".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("bonus_multiplier".to_string(), "1.5".to_string());
                params
            },
        },
    ],
);
```

## API Reference

### RuleEngine
Main rules engine for executing policies.

**Key Methods:**
- `new()` - Create new rules engine
- `add_rule(rule)` - Add rule to engine
- `execute_rule(rule, context)` - Execute specific rule
- `evaluate_condition(condition, context)` - Evaluate condition
- `execute_action(action, context)` - Execute action

### Rule
Individual rule definition with conditions and actions.

**Key Methods:**
- `new(id, name, conditions, actions)` - Create new rule
- `new_with_generated_id(name, conditions, actions)` - Create with auto-generated ID
- `is_valid()` - Validate rule definition

### ExecutionContext
Context for rule execution containing variables and metadata.

**Key Methods:**
- `new()` - Create new context
- `set_variable(key, value)` - Set context variable
- `get_variable(key)` - Get context variable
- `set_metadata(key, value)` - Set metadata

## Testing

Run the test suite:

```bash
# Basic tests
cargo test --package daa-rules

# All features
cargo test --package daa-rules --all-features

# Specific feature
cargo test --package daa-rules --features scripting
```

## Examples

See the `/examples` directory for comprehensive usage examples:

- `basic_rules.rs` - Simple rule creation and execution
- `complex_conditions.rs` - Advanced logical conditions
- `pattern_matching.rs` - Regular expression usage
- `time_based_rules.rs` - Temporal conditions
- `custom_actions.rs` - Extending with custom actions

## Performance

The rules engine is designed for high performance:

- **Fast Evaluation**: Optimized condition evaluation with short-circuiting
- **Memory Efficient**: Minimal memory overhead for rule storage
- **Concurrent Safe**: Thread-safe execution with async support
- **Scalable**: Handles thousands of rules efficiently

## License

MIT OR Apache-2.0