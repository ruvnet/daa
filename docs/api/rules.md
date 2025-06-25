# ⚖️ Rules Engine API Reference

> **Governance and decision-making system for DAA agents** - Define, evaluate, and audit business rules that guide autonomous agent behavior.

The `daa-rules` crate provides a flexible, auditable rule engine that allows agents to make consistent decisions based on predefined governance rules and real-time context.

---

## 📦 Installation

```toml
[dependencies]
daa-rules = "0.2.0"
```

## 🚀 Quick Start

```rust
use daa_rules::{RulesEngine, Rule, Context, Action};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create rules engine
    let mut engine = RulesEngine::new();
    
    // Add a simple rule
    engine.add_rule(Rule::new("max_daily_spend")
        .condition("context.daily_spent < 10000")
        .action(Action::Allow)
        .priority(100)
    ).await?;
    
    // Evaluate rules
    let context = Context::new()
        .set("daily_spent", 5000.0);
    
    let result = engine.evaluate("transaction_request", &context).await?;
    println!("Rule decision: {:?}", result);
    
    Ok(())
}
```

---

## 🏗️ Core Types

### `RulesEngine`

The main engine that manages and evaluates rules.

```rust
pub struct RulesEngine {
    // Internal fields...
}
```

#### Methods

##### `new() -> Self`

Creates a new rules engine instance.

**Example:**
```rust
let engine = RulesEngine::new();
```

##### `add_rule(rule: Rule) -> Result<()>`

Adds a new rule to the engine.

**Parameters:**
- `rule`: The rule to add

**Returns:** `Result<(), RulesError>`

**Example:**
```rust
let rule = Rule::new("treasury_limit")
    .condition("context.transaction_amount <= context.daily_limit")
    .action(Action::Allow)
    .priority(100);

engine.add_rule(rule).await?;
```

##### `remove_rule(rule_id: &str) -> Result<()>`

Removes a rule from the engine.

**Parameters:**
- `rule_id`: ID of the rule to remove

**Returns:** `Result<(), RulesError>`

**Example:**
```rust
engine.remove_rule("treasury_limit").await?;
```

##### `evaluate(trigger: &str, context: &Context) -> Result<RuleResult>`

Evaluates all rules for a given trigger and context.

**Parameters:**
- `trigger`: The event trigger name
- `context`: Current execution context

**Returns:** `Result<RuleResult, RulesError>`

**Example:**
```rust
let context = Context::new()
    .set("transaction_amount", 5000.0)
    .set("daily_limit", 10000.0)
    .set("current_balance", 50000.0);

let result = engine.evaluate("transaction_request", &context).await?;

match result.decision {
    Decision::Allow => println!("Transaction approved"),
    Decision::Deny => println!("Transaction denied: {}", result.reason),
    Decision::Require(actions) => println!("Additional actions required: {:?}", actions),
}
```

---

## 📜 Rule Definition

### `Rule`

Represents a single business rule.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: String,
    pub actions: Vec<Action>,
    pub priority: u32,
    pub enabled: bool,
    pub triggers: Vec<String>,
    pub metadata: HashMap<String, Value>,
}
```

#### Builder Methods

##### `new(id: &str) -> RuleBuilder`

Creates a new rule builder.

**Example:**
```rust
let rule = Rule::new("risk_assessment")
    .description("Assess transaction risk before execution")
    .condition("context.risk_score < 0.7")
    .action(Action::Allow)
    .priority(200)
    .trigger("transaction_request")
    .build();
```

##### `condition(condition: &str) -> Self`

Sets the rule condition using expression syntax.

**Condition Syntax:**
```rust
// Comparison operators
"context.amount > 1000"
"context.risk_level == 'high'"
"context.balance >= context.amount * 1.1"

// Logical operators
"context.amount > 1000 && context.risk_score < 0.5"
"context.account_type == 'premium' || context.amount < 500"

// Mathematical expressions
"context.portfolio_value * 0.02 >= context.transaction_amount"

// Function calls
"contains(context.allowed_tokens, context.token_address)"
"time_since(context.last_transaction) > duration('1h')"
```

##### `action(action: Action) -> Self`

Adds an action to be taken when the rule matches.

**Example:**
```rust
Rule::new("compliance_check")
    .condition("context.kyc_verified == true")
    .action(Action::Allow)
    .action(Action::Log { 
        level: LogLevel::Info,
        message: "KYC verified transaction approved".to_string(),
    });
```

### `Action`

Actions that can be taken when rules match.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Allow,
    Deny { reason: String },
    Require { actions: Vec<String> },
    Log { level: LogLevel, message: String },
    Notify { recipient: String, message: String },
    SetContext { key: String, value: Value },
    CallFunction { name: String, args: Vec<Value> },
    Delay { duration: Duration },
    RateLimit { max_requests: u32, window: Duration },
}
```

**Examples:**
```rust
// Simple actions
Action::Allow
Action::Deny { reason: "Insufficient funds".to_string() }

// Complex actions
Action::Require { 
    actions: vec!["multi_sig_approval".to_string(), "manager_review".to_string()]
}

Action::RateLimit { 
    max_requests: 10, 
    window: Duration::from_secs(3600) 
}

Action::CallFunction { 
    name: "send_alert".to_string(),
    args: vec![Value::String("High value transaction detected".to_string())]
}
```

---

## 📊 Context System

### `Context`

Provides data for rule evaluation.

```rust
#[derive(Debug, Clone)]
pub struct Context {
    data: HashMap<String, Value>,
    timestamp: SystemTime,
    metadata: HashMap<String, String>,
}
```

#### Methods

##### `new() -> Self`

Creates a new empty context.

##### `set(mut self, key: &str, value: impl Into<Value>) -> Self`

Sets a context value.

**Example:**
```rust
let context = Context::new()
    .set("user_id", "user123")
    .set("transaction_amount", 15000.0)
    .set("account_balance", 50000.0)
    .set("risk_score", 0.3)
    .set("timestamp", SystemTime::now())
    .set("metadata", json!({
        "source": "mobile_app",
        "session_id": "sess_abc123"
    }));
```

##### `get<T>(&self, key: &str) -> Option<T>`

Gets a value from the context.

**Example:**
```rust
let amount: Option<f64> = context.get("transaction_amount");
let user_id: Option<&str> = context.get("user_id");
```

##### `merge(mut self, other: Context) -> Self`

Merges another context into this one.

**Example:**
```rust
let base_context = Context::new()
    .set("user_id", "user123")
    .set("account_type", "premium");

let transaction_context = Context::new()
    .set("amount", 5000.0)
    .set("currency", "USD");

let merged = base_context.merge(transaction_context);
```

### Context Providers

Custom context providers can be implemented:

```rust
#[async_trait]
pub trait ContextProvider: Send + Sync {
    async fn provide_context(&self, trigger: &str) -> Result<Context, ContextError>;
}

pub struct DatabaseContextProvider {
    db: Arc<Database>,
}

#[async_trait]
impl ContextProvider for DatabaseContextProvider {
    async fn provide_context(&self, trigger: &str) -> Result<Context, ContextError> {
        match trigger {
            "transaction_request" => {
                let user_data = self.db.get_user_context().await?;
                let market_data = self.db.get_market_context().await?;
                
                Ok(Context::new()
                    .set("user", user_data)
                    .set("market", market_data))
            }
            _ => Ok(Context::new())
        }
    }
}
```

---

## 📝 Rule Evaluation

### `RuleResult`

Result of rule evaluation.

```rust
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub decision: Decision,
    pub reason: String,
    pub matched_rules: Vec<String>,
    pub actions_taken: Vec<Action>,
    pub context_updates: HashMap<String, Value>,
    pub evaluation_time: Duration,
}
```

### `Decision`

The final decision from rule evaluation.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Allow,
    Deny,
    Require(Vec<String>),
    Conditional(Vec<Condition>),
}
```

### Advanced Evaluation

**Batch Evaluation:**
```rust
// Evaluate multiple contexts at once
let contexts = vec![context1, context2, context3];
let results = engine.evaluate_batch("transaction_request", &contexts).await?;

for (i, result) in results.iter().enumerate() {
    println!("Context {}: {:?}", i, result.decision);
}
```

**Parallel Evaluation:**
```rust
// Evaluate with multiple triggers
let triggers = vec!["pre_transaction", "transaction_request", "post_transaction"];
let results = engine.evaluate_parallel(&triggers, &context).await?;

for (trigger, result) in triggers.iter().zip(results.iter()) {
    println!("{}: {:?}", trigger, result.decision);
}
```

---

## 🔍 Audit System

### `AuditLogger`

Tracks all rule evaluations for compliance and debugging.

```rust
pub struct AuditLogger {
    // Internal fields...
}
```

#### Methods

##### `log_evaluation(evaluation: &RuleEvaluation) -> Result<()>`

Logs a rule evaluation event.

**Example:**
```rust
let audit_logger = AuditLogger::new();

let evaluation = RuleEvaluation {
    rule_id: "treasury_limit".to_string(),
    trigger: "transaction_request".to_string(),
    context: context.clone(),
    result: result.clone(),
    timestamp: SystemTime::now(),
};

audit_logger.log_evaluation(&evaluation).await?;
```

##### `query_evaluations(query: AuditQuery) -> Result<Vec<RuleEvaluation>>`

Queries audit logs.

**Example:**
```rust
// Find all denied transactions in the last 24 hours
let query = AuditQuery::new()
    .decision(Decision::Deny)
    .time_range(SystemTime::now() - Duration::from_secs(86400), SystemTime::now())
    .rule_id("treasury_limit");

let evaluations = audit_logger.query_evaluations(query).await?;

for eval in evaluations {
    println!("Denied transaction: {} at {:?}", 
             eval.context.get::<String>("transaction_id").unwrap_or_default(),
             eval.timestamp);
}
```

### Audit Reports

```rust
// Generate compliance report
let report = audit_logger.generate_report(ReportConfig {
    start_time: SystemTime::now() - Duration::from_secs(30 * 86400), // 30 days
    end_time: SystemTime::now(),
    include_successful: false,
    group_by: GroupBy::Rule,
    format: ReportFormat::Json,
}).await?;

println!("Compliance Report: {}", serde_json::to_string_pretty(&report)?);
```

---

## 🔧 Advanced Features

### Custom Functions

Register custom functions for use in rule conditions:

```rust
use daa_rules::functions::FunctionRegistry;

// Register custom function
let mut registry = FunctionRegistry::new();

registry.register("calculate_risk_score", |args: &[Value]| -> Result<Value, FunctionError> {
    let amount = args[0].as_f64().ok_or(FunctionError::InvalidArgument)?;
    let account_age = args[1].as_i64().ok_or(FunctionError::InvalidArgument)?;
    
    let risk_score = if account_age < 30 {
        0.8 // High risk for new accounts
    } else if amount > 10000.0 {
        0.6 // Medium risk for large amounts
    } else {
        0.2 // Low risk
    };
    
    Ok(Value::Number(serde_json::Number::from_f64(risk_score).unwrap()))
});

// Use in rules
let rule = Rule::new("dynamic_risk")
    .condition("calculate_risk_score(context.amount, context.account_age) < 0.5")
    .action(Action::Allow);
```

### Rule Templates

Create reusable rule templates:

```rust
pub struct RuleTemplate {
    pub template: String,
    pub parameters: Vec<String>,
}

impl RuleTemplate {
    pub fn instantiate(&self, params: &HashMap<String, Value>) -> Result<Rule, TemplateError> {
        let mut condition = self.template.clone();
        
        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => format!("\"{}\"", s),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => return Err(TemplateError::UnsupportedType),
            };
            condition = condition.replace(&placeholder, &replacement);
        }
        
        Ok(Rule::new(&format!("template_{}", uuid::Uuid::new_v4()))
            .condition(&condition)
            .action(Action::Allow))
    }
}

// Usage
let template = RuleTemplate {
    template: "context.amount <= {max_amount} && context.account_type == {account_type}".to_string(),
    parameters: vec!["max_amount".to_string(), "account_type".to_string()],
};

let params = HashMap::from([
    ("max_amount".to_string(), Value::Number(serde_json::Number::from(5000))),
    ("account_type".to_string(), Value::String("premium".to_string())),
]);

let rule = template.instantiate(&params)?;
```

### Rule Composition

Combine multiple rules into complex decision trees:

```rust
use daa_rules::composition::{RuleGroup, GroupOperator};

let risk_rules = RuleGroup::new("risk_assessment")
    .operator(GroupOperator::All) // All rules must pass
    .add_rule(Rule::new("amount_check")
        .condition("context.amount <= context.daily_limit"))
    .add_rule(Rule::new("velocity_check")
        .condition("context.transaction_velocity < 10"))
    .add_rule(Rule::new("location_check")
        .condition("context.location_risk_score < 0.5"));

let compliance_rules = RuleGroup::new("compliance")
    .operator(GroupOperator::Any) // Any rule can approve
    .add_rule(Rule::new("kyc_verified")
        .condition("context.kyc_status == 'verified'"))
    .add_rule(Rule::new("manual_approval")
        .condition("context.manual_approval == true"));

// Combine groups
let master_group = RuleGroup::new("transaction_approval")
    .operator(GroupOperator::All)
    .add_group(risk_rules)
    .add_group(compliance_rules);

// Evaluate composed rules
let result = engine.evaluate_group(&master_group, &context).await?;
```

---

## 📊 Performance Optimization

### Rule Caching

```rust
use daa_rules::cache::{RuleCache, CacheConfig};

let cache_config = CacheConfig {
    max_size: 10000,
    ttl: Duration::from_secs(300), // 5 minutes
    enable_statistics: true,
};

let mut engine = RulesEngine::new()
    .with_cache(RuleCache::new(cache_config));

// Rules and contexts will be cached automatically
let result = engine.evaluate("transaction_request", &context).await?;
```

### Parallel Evaluation

```rust
// Enable parallel rule evaluation
let engine = RulesEngine::new()
    .with_parallel_evaluation(true)
    .with_max_parallelism(8);

// Rules will be evaluated in parallel when safe to do so
let result = engine.evaluate("complex_scenario", &context).await?;
```

### Rule Indexing

```rust
// Create indexes for faster rule lookup
engine.create_index("trigger", IndexType::Hash).await?;
engine.create_index("priority", IndexType::BTree).await?;

// Rules will be retrieved more efficiently
let result = engine.evaluate("indexed_trigger", &context).await?;
```

---

## 🚨 Error Handling

### `RulesError`

Main error type for rules operations.

```rust
#[derive(Error, Debug)]
pub enum RulesError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    
    #[error("Invalid condition syntax: {0}")]
    InvalidCondition(String),
    
    #[error("Context error: {0}")]
    Context(#[from] ContextError),
    
    #[error("Evaluation timeout")]
    Timeout,
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

### Error Handling Best Practices

```rust
use daa_rules::{RulesEngine, RulesError};

async fn safe_rule_evaluation(engine: &RulesEngine, trigger: &str, context: &Context) -> Result<Decision, RulesError> {
    match engine.evaluate(trigger, context).await {
        Ok(result) => Ok(result.decision),
        Err(RulesError::RuleNotFound(rule_id)) => {
            // Log missing rule and use default policy
            log::warn!("Rule not found: {}, using default policy", rule_id);
            Ok(Decision::Allow) // or your default policy
        },
        Err(RulesError::InvalidCondition(condition)) => {
            // Log syntax error and disable rule
            log::error!("Invalid rule condition: {}", condition);
            // engine.disable_rule_by_condition(&condition).await?;
            Ok(Decision::Allow)
        },
        Err(RulesError::Timeout) => {
            // Log timeout and use fail-safe policy
            log::error!("Rule evaluation timeout for trigger: {}", trigger);
            Ok(Decision::Deny) // Fail-safe approach
        },
        Err(e) => {
            // Log unexpected error and fail safe
            log::error!("Unexpected rules error: {}", e);
            Err(e)
        }
    }
}
```

---

## 🔧 Configuration

### Engine Configuration

```toml
[rules]
# Evaluation settings
evaluation_timeout = "30s"
max_context_size = "1MB"
enable_parallel_evaluation = true
max_parallel_rules = 16

# Caching
cache_enabled = true
cache_max_size = 10000
cache_ttl = "300s"

# Audit settings
audit_enabled = true
audit_retention = "90d"
audit_compression = true

# Performance
rule_compilation = true
expression_optimization = true
context_pooling = true
```

### Environment Variables

```bash
# Rules engine configuration
export DAA_RULES_EVALUATION_TIMEOUT=30s
export DAA_RULES_CACHE_ENABLED=true
export DAA_RULES_AUDIT_ENABLED=true
export DAA_RULES_PARALLEL_EVALUATION=true

# Database configuration (if using persistent storage)
export DAA_RULES_DB_URL="postgresql://user:pass@localhost/daa_rules"
export DAA_RULES_DB_POOL_SIZE=20
```

---

## 📚 Examples

### Treasury Management Rules

```rust
use daa_rules::prelude::*;

async fn setup_treasury_rules(engine: &mut RulesEngine) -> Result<(), RulesError> {
    // Daily spending limit
    engine.add_rule(Rule::new("daily_limit")
        .description("Enforce daily spending limits")
        .condition("context.daily_spent + context.amount <= context.daily_limit")
        .action(Action::Allow)
        .action(Action::Log { 
            level: LogLevel::Info,
            message: "Transaction within daily limit".to_string()
        })
        .priority(100)
        .trigger("transaction_request")
    ).await?;
    
    // Risk assessment
    engine.add_rule(Rule::new("risk_threshold")
        .description("Block high-risk transactions")
        .condition("context.risk_score <= 0.7")
        .action(Action::Allow)
        .else_action(Action::Deny { 
            reason: "Transaction risk score too high".to_string() 
        })
        .priority(200)
        .trigger("transaction_request")
    ).await?;
    
    // Large transaction approval
    engine.add_rule(Rule::new("large_transaction")
        .description("Require approval for large transactions")
        .condition("context.amount > 50000")
        .action(Action::Require { 
            actions: vec!["manager_approval".to_string(), "board_approval".to_string()]
        })
        .priority(300)
        .trigger("transaction_request")
    ).await?;
    
    // Emergency stop
    engine.add_rule(Rule::new("emergency_stop")
        .description("Stop all transactions during emergency")
        .condition("context.emergency_mode == false")
        .action(Action::Allow)
        .else_action(Action::Deny { 
            reason: "Emergency mode activated".to_string() 
        })
        .priority(1000) // Highest priority
        .trigger("transaction_request")
    ).await?;
    
    Ok(())
}
```

### DeFi Strategy Rules

```rust
async fn setup_defi_rules(engine: &mut RulesEngine) -> Result<(), RulesError> {
    // Yield farming opportunity
    engine.add_rule(Rule::new("yield_opportunity")
        .description("Enter yield farming when APY is attractive")
        .condition("context.apy > 0.15 && context.pool_liquidity > 1000000")
        .action(Action::CallFunction { 
            name: "enter_yield_farm".to_string(),
            args: vec![Value::String("optimal_allocation".to_string())]
        })
        .trigger("yield_scan")
    ).await?;
    
    // Impermanent loss protection
    engine.add_rule(Rule::new("impermanent_loss_check")
        .description("Exit position if impermanent loss exceeds threshold")
        .condition("context.impermanent_loss > 0.05") // 5% loss
        .action(Action::CallFunction { 
            name: "exit_position".to_string(),
            args: vec![Value::String("immediate".to_string())]
        })
        .trigger("position_monitor")
    ).await?;
    
    // Rebalancing trigger
    engine.add_rule(Rule::new("portfolio_rebalance")
        .description("Rebalance when allocation drifts")
        .condition("abs(context.current_allocation - context.target_allocation) > 0.1")
        .action(Action::CallFunction { 
            name: "rebalance_portfolio".to_string(),
            args: vec![]
        })
        .trigger("allocation_check")
    ).await?;
    
    Ok(())
}
```

---

## 🔗 Related Documentation

- [Orchestrator API](./orchestrator.md) - Core orchestration engine
- [Economy API](./economy.md) - Token management and economics
- [AI Integration API](./ai.md) - Claude AI and decision support
- [Architecture Guide](../architecture/README.md) - System design overview

---

## 📊 Performance Metrics

### Typical Performance

| Operation | Throughput | Latency |
|-----------|------------|---------|
| Simple rule evaluation | 50,000/sec | <0.1ms |
| Complex rule evaluation | 10,000/sec | <1ms |
| Batch evaluation (100 rules) | 1,000/sec | <10ms |
| Audit log write | 100,000/sec | <0.05ms |

### Optimization Tips

1. **Rule Ordering**: Place high-priority, frequently-triggered rules first
2. **Context Optimization**: Keep context data minimal and relevant
3. **Caching**: Enable rule caching for frequently-evaluated scenarios
4. **Indexing**: Create indexes on commonly-queried rule attributes
5. **Batch Operations**: Use batch evaluation for multiple contexts

---

*For more detailed API documentation, see the [rustdoc documentation](https://docs.rs/daa-rules).*