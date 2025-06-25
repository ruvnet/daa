# 🧠 AI Integration API Reference

> **Intelligent decision-making for DAA agents** - Claude AI integration and Model Context Protocol (MCP) client for autonomous reasoning and decision support.

The `daa-ai` crate provides seamless integration with Claude AI and the MCP ecosystem, enabling agents to make intelligent decisions, learn from outcomes, and adapt their strategies over time.

---

## 📦 Installation

```toml
[dependencies]
daa-ai = "0.2.0"
```

## 🚀 Quick Start

```rust
use daa_ai::{AiIntegration, ClaudeClient, DecisionContext};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize AI integration
    let ai = AiIntegration::new()
        .with_claude_api_key(&std::env::var("ANTHROPIC_API_KEY")?)
        .with_model("claude-3-sonnet-20240229")
        .build().await?;
    
    // Create decision context
    let context = DecisionContext::new()
        .with_scenario("portfolio_rebalancing")
        .with_data("current_allocation", json!({
            "stocks": 0.6,
            "bonds": 0.3,
            "crypto": 0.1
        }))
        .with_constraint("max_risk_level", 0.7);
    
    // Get AI decision
    let decision = ai.make_decision(&context).await?;
    
    println!("AI Decision: {:?}", decision);
    println!("Confidence: {:.2}%", decision.confidence * 100.0);
    
    Ok(())
}
```

---

## 🏗️ Core Types

### `AiIntegration`

The main AI integration manager that coordinates all AI operations.

```rust
pub struct AiIntegration {
    // Internal fields...
}
```

#### Methods

##### `new() -> AiIntegrationBuilder`

Creates a new AI integration builder.

**Example:**
```rust
let ai = AiIntegration::new()
    .with_claude_api_key("your-api-key")
    .with_model("claude-3-sonnet-20240229")
    .with_max_tokens(4000)
    .with_temperature(0.7)
    .build().await?;
```

##### `make_decision(context: &DecisionContext) -> Result<Decision>`

Makes an AI-powered decision based on the provided context.

**Parameters:**
- `context`: Decision context with relevant data and constraints

**Returns:** `Result<Decision, AiError>`

**Example:**
```rust
let context = DecisionContext::new()
    .with_scenario("trading_opportunity")
    .with_data("market_data", market_snapshot)
    .with_data("portfolio", current_portfolio)
    .with_constraint("max_position_size", 0.1)
    .with_constraint("risk_tolerance", 0.5);

let decision = ai.make_decision(&context).await?;

match decision.action {
    DecisionAction::Execute(plan) => {
        println!("Execute trade plan: {:?}", plan);
    }
    DecisionAction::Wait(reason) => {
        println!("Wait for better opportunity: {}", reason);
    }
    DecisionAction::Investigate(questions) => {
        println!("Need more information: {:?}", questions);
    }
}
```

##### `analyze_situation(description: &str) -> Result<Analysis>`

Analyzes a given situation and provides insights.

**Parameters:**
- `description`: Description of the situation to analyze

**Returns:** `Result<Analysis, AiError>`

**Example:**
```rust
let analysis = ai.analyze_situation(
    "Market volatility has increased 300% over the last 24 hours, \
     with correlation between assets reaching 0.95. Our portfolio \
     is currently leveraged 2:1 with 60% in equities."
).await?;

println!("Risk Assessment: {:?}", analysis.risk_level);
println!("Recommendations: {:?}", analysis.recommendations);
```

##### `learn_from_outcome(decision: &Decision, outcome: &Outcome) -> Result<()>`

Learns from decision outcomes to improve future decisions.

**Parameters:**
- `decision`: The original decision made
- `outcome`: The actual outcome that occurred

**Returns:** `Result<(), AiError>`

**Example:**
```rust
let outcome = Outcome {
    success: true,
    actual_profit: 15000.0,
    predicted_profit: 12000.0,
    time_to_completion: Duration::from_hours(4),
    unexpected_events: vec!["market_surge".to_string()],
};

ai.learn_from_outcome(&decision, &outcome).await?;
```

---

## 🤖 Claude AI Client

### `ClaudeClient`

Direct interface to Claude AI API.

```rust
pub struct ClaudeClient {
    // Internal fields...
}
```

#### Methods

##### `new(api_key: &str) -> Self`

Creates a new Claude client.

**Example:**
```rust
let claude = ClaudeClient::new(&std::env::var("ANTHROPIC_API_KEY")?);
```

##### `complete(prompt: &str) -> Result<String>`

Gets a completion from Claude.

**Example:**
```rust
let prompt = "Given the following market conditions and portfolio allocation, \
              what would be the optimal rebalancing strategy?\n\n\
              Market Conditions:\n\
              - VIX: 28 (elevated)\n\
              - S&P 500: -5% (1 month)\n\
              - Bond yields: +0.5% (1 month)\n\n\
              Current Portfolio:\n\
              - Stocks: 70%\n\
              - Bonds: 25%\n\
              - Cash: 5%\n\n\
              Risk tolerance: Moderate\n\
              Investment horizon: 5 years";

let response = claude.complete(prompt).await?;
println!("Claude's recommendation: {}", response);
```

##### `structured_completion<T>(prompt: &str, schema: &Schema) -> Result<T>`

Gets a structured response from Claude.

**Example:**
```rust
#[derive(Serialize, Deserialize)]
struct TradingDecision {
    action: String,
    symbol: String,
    quantity: f64,
    confidence: f64,
    reasoning: String,
}

let schema = Schema::from::<TradingDecision>();
let decision: TradingDecision = claude.structured_completion(
    "Should I buy, sell, or hold AAPL given current market conditions?",
    &schema
).await?;

println!("Action: {}", decision.action);
println!("Confidence: {:.2}%", decision.confidence * 100.0);
```

##### `chat(conversation: &Conversation) -> Result<ChatResponse>`

Maintains conversation context across multiple exchanges.

**Example:**
```rust
let mut conversation = Conversation::new()
    .with_system_prompt("You are a financial advisor for an autonomous trading agent.");

conversation.add_user_message("What's your opinion on the current market volatility?");
let response1 = claude.chat(&conversation).await?;
conversation.add_assistant_message(&response1.content);

conversation.add_user_message("Given that analysis, should I increase my cash position?");
let response2 = claude.chat(&conversation).await?;

println!("Final recommendation: {}", response2.content);
```

---

## 🔌 MCP Integration

### `McpClient`

Model Context Protocol client for accessing external tools and data.

```rust
pub struct McpClient {
    // Internal fields...
}
```

#### Methods

##### `new() -> McpClientBuilder`

Creates a new MCP client builder.

**Example:**
```rust
let mcp = McpClient::new()
    .add_server("market_data", "http://localhost:3001")
    .add_server("portfolio_manager", "http://localhost:3002")
    .add_server("risk_analyzer", "http://localhost:3003")
    .build().await?;
```

##### `call_tool(server: &str, tool: &str, args: Value) -> Result<Value>`

Calls a tool on an MCP server.

**Parameters:**
- `server`: Server identifier
- `tool`: Tool name to call
- `args`: Arguments for the tool

**Returns:** `Result<Value, McpError>`

**Example:**
```rust
// Get real-time market data
let market_data = mcp.call_tool(
    "market_data",
    "get_price",
    json!({
        "symbol": "AAPL",
        "fields": ["price", "volume", "volatility"]
    })
).await?;

// Analyze portfolio risk
let risk_analysis = mcp.call_tool(
    "risk_analyzer",
    "calculate_var",
    json!({
        "portfolio": current_portfolio,
        "confidence_level": 0.95,
        "time_horizon": "1d"
    })
).await?;
```

##### `get_available_tools(server: &str) -> Result<Vec<Tool>>`

Lists available tools on an MCP server.

**Example:**
```rust
let tools = mcp.get_available_tools("market_data").await?;

for tool in tools {
    println!("Tool: {}", tool.name);
    println!("Description: {}", tool.description);
    println!("Parameters: {:?}", tool.parameters);
}
```

##### `stream_data(server: &str, stream: &str) -> Result<DataStream>`

Subscribes to a data stream from an MCP server.

**Example:**
```rust
let price_stream = mcp.stream_data("market_data", "price_updates").await?;

while let Some(update) = price_stream.next().await {
    let price_data: PriceUpdate = serde_json::from_value(update)?;
    println!("Price update: {} = ${}", price_data.symbol, price_data.price);
    
    // Trigger decision making if significant change
    if price_data.change_percent.abs() > 0.05 {
        let context = DecisionContext::new()
            .with_scenario("price_movement")
            .with_data("price_update", price_data);
        
        let decision = ai.make_decision(&context).await?;
        // Handle decision...
    }
}
```

---

## 🧩 Decision Framework

### `DecisionContext`

Provides context for AI decision making.

```rust
#[derive(Debug, Clone)]
pub struct DecisionContext {
    scenario: String,
    data: HashMap<String, Value>,
    constraints: HashMap<String, Value>,
    objectives: Vec<String>,
    timestamp: SystemTime,
}
```

#### Methods

##### `new() -> Self`

Creates a new decision context.

##### `with_scenario(mut self, scenario: &str) -> Self`

Sets the decision scenario.

**Example:**
```rust
let context = DecisionContext::new()
    .with_scenario("portfolio_optimization");
```

##### `with_data(mut self, key: &str, value: Value) -> Self`

Adds data to the decision context.

**Example:**
```rust
let context = DecisionContext::new()
    .with_data("market_cap", json!(50000000))
    .with_data("current_price", json!(150.25))
    .with_data("volume_24h", json!(1250000));
```

##### `with_constraint(mut self, key: &str, value: Value) -> Self`

Adds constraints to the decision.

**Example:**
```rust
let context = DecisionContext::new()
    .with_constraint("max_position_size", json!(0.1))
    .with_constraint("min_liquidity", json!(1000000))
    .with_constraint("risk_budget", json!(0.05));
```

##### `with_objective(mut self, objective: &str) -> Self`

Adds an objective to optimize for.

**Example:**
```rust
let context = DecisionContext::new()
    .with_objective("maximize_risk_adjusted_return")
    .with_objective("minimize_correlation")
    .with_objective("maintain_liquidity");
```

### `Decision`

Represents an AI-generated decision.

```rust
#[derive(Debug, Clone)]
pub struct Decision {
    pub action: DecisionAction,
    pub confidence: f64,
    pub reasoning: String,
    pub alternatives: Vec<Alternative>,
    pub risk_assessment: RiskAssessment,
    pub expected_outcome: ExpectedOutcome,
    pub metadata: HashMap<String, Value>,
}
```

### `DecisionAction`

Types of actions the AI can recommend.

```rust
#[derive(Debug, Clone)]
pub enum DecisionAction {
    Execute(ActionPlan),
    Wait(String),
    Investigate(Vec<String>),
    Escalate(EscalationReason),
    Abort(String),
}

#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub steps: Vec<ActionStep>,
    pub timeline: Duration,
    pub dependencies: Vec<String>,
    pub rollback_plan: Option<Vec<ActionStep>>,
}
```

**Example Usage:**
```rust
match decision.action {
    DecisionAction::Execute(plan) => {
        for step in plan.steps {
            println!("Step: {} (estimated time: {:?})", step.description, step.duration);
            // Execute step...
        }
    }
    DecisionAction::Wait(reason) => {
        println!("Waiting because: {}", reason);
        // Set up monitoring for trigger conditions
    }
    DecisionAction::Investigate(questions) => {
        for question in questions {
            println!("Need to investigate: {}", question);
            // Gather additional data
        }
    }
}
```

---

## 📚 Learning System

### `LearningEngine`

Enables the AI to learn from outcomes and improve over time.

```rust
pub struct LearningEngine {
    // Internal fields...
}
```

#### Methods

##### `record_decision(decision: &Decision, context: &DecisionContext) -> Result<()>`

Records a decision for future learning.

**Example:**
```rust
learning_engine.record_decision(&decision, &context).await?;
```

##### `update_from_outcome(decision_id: &str, outcome: &Outcome) -> Result<()>`

Updates learning models based on actual outcomes.

**Example:**
```rust
let outcome = Outcome {
    success: true,
    actual_profit: 12500.0,
    predicted_profit: 10000.0,
    execution_time: Duration::from_minutes(45),
    side_effects: vec!["increased_volatility".to_string()],
    lessons_learned: vec![
        "Market moved faster than expected".to_string(),
        "Liquidity was higher than estimated".to_string(),
    ],
};

learning_engine.update_from_outcome(&decision.id, &outcome).await?;
```

##### `get_performance_metrics() -> Result<PerformanceMetrics>`

Gets performance metrics for the learning system.

**Example:**
```rust
let metrics = learning_engine.get_performance_metrics().await?;

println!("Decision accuracy: {:.2}%", metrics.accuracy * 100.0);
println!("Average confidence calibration: {:.3}", metrics.calibration_score);
println!("Improvement rate: {:.2}%/month", metrics.monthly_improvement * 100.0);
```

### Adaptive Strategies

**Strategy Optimization:**
```rust
use daa_ai::strategies::{StrategyOptimizer, Strategy};

let optimizer = StrategyOptimizer::new();

// Define strategy parameters
let strategy = Strategy {
    name: "momentum_trading".to_string(),
    parameters: hashmap! {
        "lookback_period" => json!(14),
        "momentum_threshold" => json!(0.05),
        "position_size" => json!(0.1),
        "stop_loss" => json!(0.02),
    },
};

// Optimize based on historical performance
let optimized_strategy = optimizer.optimize(&strategy, &historical_data).await?;

println!("Optimized parameters: {:?}", optimized_strategy.parameters);
```

---

## 🎯 Specialized AI Modules

### `TradingAI`

Specialized AI for trading decisions.

```rust
use daa_ai::trading::TradingAI;

let trading_ai = TradingAI::new()
    .with_strategy("momentum")
    .with_risk_level(RiskLevel::Moderate)
    .build().await?;

// Analyze trading opportunity
let opportunity = TradingOpportunity {
    symbol: "BTC/USD".to_string(),
    current_price: 45000.0,
    volume_24h: 2500000000.0,
    technical_indicators: json!({
        "rsi": 65,
        "macd": 0.8,
        "bollinger_position": 0.75
    }),
};

let trading_decision = trading_ai.evaluate_opportunity(&opportunity).await?;

match trading_decision.action {
    TradingAction::Buy { quantity, price_limit } => {
        println!("Buy {} at max ${}", quantity, price_limit);
    }
    TradingAction::Sell { quantity, price_limit } => {
        println!("Sell {} at min ${}", quantity, price_limit);
    }
    TradingAction::Hold => {
        println!("Hold position");
    }
}
```

### `RiskAI`

AI specialized in risk assessment and management.

```rust
use daa_ai::risk::RiskAI;

let risk_ai = RiskAI::new()
    .with_risk_models(vec!["var", "cvar", "monte_carlo"])
    .with_confidence_level(0.95)
    .build().await?;

let risk_assessment = risk_ai.assess_portfolio_risk(&portfolio).await?;

println!("Portfolio VaR (95%): ${:.2}", risk_assessment.var_95);
println!("Expected shortfall: ${:.2}", risk_assessment.expected_shortfall);
println!("Risk factors: {:?}", risk_assessment.primary_risk_factors);

// Get risk mitigation recommendations
let mitigations = risk_ai.recommend_mitigations(&risk_assessment).await?;
for mitigation in mitigations {
    println!("Recommendation: {} (impact: {:.2}%)", 
             mitigation.action, mitigation.risk_reduction * 100.0);
}
```

---

## 🔧 Configuration

### AI Configuration

```toml
[ai]
# Claude AI settings
claude_api_key = "${ANTHROPIC_API_KEY}"
claude_model = "claude-3-sonnet-20240229"
max_tokens = 4000
temperature = 0.7
timeout = "30s"

# Decision making
decision_cache_enabled = true
decision_cache_ttl = "300s"
confidence_threshold = 0.7
max_retries = 3

# Learning settings
learning_enabled = true
learning_rate = 0.01
memory_retention = "90d"
performance_tracking = true

# MCP settings
mcp_enabled = true
mcp_timeout = "10s"
mcp_retry_attempts = 2
mcp_servers = [
    { name = "market_data", url = "http://localhost:3001" },
    { name = "portfolio_manager", url = "http://localhost:3002" }
]
```

### Environment Variables

```bash
# Claude AI configuration
export ANTHROPIC_API_KEY="your-claude-api-key"
export DAA_AI_MODEL="claude-3-sonnet-20240229"
export DAA_AI_MAX_TOKENS=4000
export DAA_AI_TEMPERATURE=0.7

# Decision making
export DAA_AI_DECISION_CACHE=true
export DAA_AI_CONFIDENCE_THRESHOLD=0.7

# Learning system
export DAA_AI_LEARNING_ENABLED=true
export DAA_AI_LEARNING_RATE=0.01

# MCP configuration
export DAA_AI_MCP_ENABLED=true
export DAA_AI_MCP_SERVERS="market_data:3001,portfolio:3002"
```

---

## 📊 Prompt Engineering

### System Prompts

**Financial Advisor Prompt:**
```rust
const FINANCIAL_ADVISOR_PROMPT: &str = r#"
You are an expert financial advisor and portfolio manager for an autonomous trading agent. 
Your role is to:

1. Analyze market conditions and portfolio performance
2. Recommend optimal trading strategies and asset allocations
3. Assess risks and suggest mitigation strategies
4. Provide clear, actionable investment decisions

Guidelines:
- Always consider risk-adjusted returns
- Factor in correlation between assets
- Account for market volatility and liquidity
- Provide confidence levels for your recommendations
- Explain your reasoning clearly and concisely

Current market context: {market_context}
Portfolio status: {portfolio_status}
Risk tolerance: {risk_tolerance}
Investment horizon: {investment_horizon}
"#;

let prompt = FINANCIAL_ADVISOR_PROMPT
    .replace("{market_context}", &market_context)
    .replace("{portfolio_status}", &portfolio_status)
    .replace("{risk_tolerance}", "Moderate")
    .replace("{investment_horizon}", "Medium-term (1-3 years)");
```

**Risk Assessment Prompt:**
```rust
const RISK_ASSESSMENT_PROMPT: &str = r#"
Analyze the following investment scenario for potential risks:

Scenario: {scenario}
Investment Details: {investment_details}
Market Conditions: {market_conditions}

Please provide:
1. Overall risk level (Low/Medium/High)
2. Primary risk factors
3. Potential impact of each risk
4. Recommended risk mitigation strategies
5. Confidence level in your assessment

Format your response as structured JSON.
"#;
```

### Dynamic Prompts

```rust
use daa_ai::prompts::{PromptTemplate, PromptBuilder};

let template = PromptTemplate::new()
    .with_system_role("financial_advisor")
    .with_context_variables(vec!["market_data", "portfolio", "objectives"])
    .with_output_format("structured_json");

let prompt = PromptBuilder::new()
    .from_template(&template)
    .set_variable("market_data", &current_market_data)
    .set_variable("portfolio", &current_portfolio)
    .set_variable("objectives", &investment_objectives)
    .build()?;

let response = claude.structured_completion::<InvestmentRecommendation>(&prompt, &schema).await?;
```

---

## 🚨 Error Handling

### `AiError`

Main error type for AI operations.

```rust
#[derive(Error, Debug)]
pub enum AiError {
    #[error("Claude API error: {0}")]
    ClaudeApi(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Confidence too low: {actual} < {threshold}")]
    LowConfidence { actual: f64, threshold: f64 },
    
    #[error("Context too large: {size} bytes")]
    ContextTooLarge { size: usize },
    
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),
    
    #[error("Rate limit exceeded")]
    RateLimit,
}
```

### Error Handling Strategies

```rust
use daa_ai::{AiIntegration, AiError};

async fn robust_ai_decision(
    ai: &AiIntegration, 
    context: &DecisionContext
) -> Result<Decision, AiError> {
    const MAX_RETRIES: u32 = 3;
    const MIN_CONFIDENCE: f64 = 0.6;
    
    for attempt in 1..=MAX_RETRIES {
        match ai.make_decision(context).await {
            Ok(decision) if decision.confidence >= MIN_CONFIDENCE => {
                return Ok(decision);
            }
            Ok(decision) => {
                log::warn!("Low confidence decision: {:.2}%", decision.confidence * 100.0);
                if attempt == MAX_RETRIES {
                    return Err(AiError::LowConfidence {
                        actual: decision.confidence,
                        threshold: MIN_CONFIDENCE,
                    });
                }
            }
            Err(AiError::RateLimit) => {
                let delay = Duration::from_secs(2_u64.pow(attempt));
                log::info!("Rate limited, waiting {:?}", delay);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(AiError::ClaudeApi("Max retries exceeded".to_string()))
}
```

---

## 📚 Examples

### Complete Trading Decision System

```rust
use daa_ai::prelude::*;

async fn autonomous_trading_system() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize AI system
    let ai = AiIntegration::new()
        .with_claude_api_key(&std::env::var("ANTHROPIC_API_KEY")?)
        .with_model("claude-3-sonnet-20240229")
        .with_learning_enabled(true)
        .build().await?;
    
    // Initialize MCP clients
    let mcp = McpClient::new()
        .add_server("market_data", "http://localhost:3001")
        .add_server("portfolio_manager", "http://localhost:3002")
        .add_server("risk_analyzer", "http://localhost:3003")
        .build().await?;
    
    // Main trading loop
    loop {
        // Gather market data
        let market_data = mcp.call_tool("market_data", "get_market_overview", json!({})).await?;
        
        // Get current portfolio
        let portfolio = mcp.call_tool("portfolio_manager", "get_portfolio", json!({})).await?;
        
        // Assess current risk
        let risk_analysis = mcp.call_tool("risk_analyzer", "analyze_portfolio", json!({
            "portfolio": portfolio
        })).await?;
        
        // Create decision context
        let context = DecisionContext::new()
            .with_scenario("trading_opportunity_scan")
            .with_data("market_data", market_data)
            .with_data("portfolio", portfolio)
            .with_data("risk_analysis", risk_analysis)
            .with_constraint("max_risk_increase", json!(0.1))
            .with_constraint("min_expected_return", json!(0.05))
            .with_objective("maximize_sharpe_ratio");
        
        // Make AI decision
        let decision = ai.make_decision(&context).await?;
        
        match decision.action {
            DecisionAction::Execute(plan) => {
                println!("Executing trading plan:");
                for step in plan.steps {
                    println!("  - {}", step.description);
                    
                    // Execute the step through MCP
                    match step.action_type.as_str() {
                        "buy_asset" => {
                            let result = mcp.call_tool("portfolio_manager", "buy_asset", step.parameters).await?;
                            println!("    Result: {:?}", result);
                        }
                        "sell_asset" => {
                            let result = mcp.call_tool("portfolio_manager", "sell_asset", step.parameters).await?;
                            println!("    Result: {:?}", result);
                        }
                        "rebalance" => {
                            let result = mcp.call_tool("portfolio_manager", "rebalance", step.parameters).await?;
                            println!("    Result: {:?}", result);
                        }
                        _ => {
                            println!("    Unknown action: {}", step.action_type);
                        }
                    }
                }
                
                // Record the decision for learning
                ai.learn_from_execution(&decision, &plan).await?;
            }
            DecisionAction::Wait(reason) => {
                println!("Waiting: {}", reason);
            }
            DecisionAction::Investigate(questions) => {
                println!("Need more information:");
                for question in questions {
                    println!("  - {}", question);
                }
            }
            DecisionAction::Escalate(reason) => {
                println!("Escalating to human operator: {:?}", reason);
                // Send alert to operators
            }
            DecisionAction::Abort(reason) => {
                println!("Aborting operations: {}", reason);
                // Emergency stop procedures
            }
        }
        
        // Wait before next iteration
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

---

## 🔗 Related Documentation

- [Orchestrator API](./orchestrator.md) - Core coordination engine
- [Rules Engine API](./rules.md) - Governance and decision making
- [Economy API](./economy.md) - Token management and economics
- [MCP Protocol Specification](https://modelcontextprotocol.io/) - External MCP documentation

---

## 📊 Performance Metrics

### Typical Performance

| Operation | Throughput | Latency |
|-----------|------------|---------|
| Simple decisions | 100/minute | 2-5s |
| Complex analysis | 20/minute | 5-15s |
| Learning updates | 1000/minute | <100ms |
| MCP tool calls | 500/minute | 100-500ms |

### Best Practices

1. **Caching**: Enable decision caching for repeated scenarios
2. **Batch Processing**: Group similar decisions when possible
3. **Async Operations**: Use async/await for all AI operations
4. **Confidence Thresholds**: Set appropriate confidence levels
5. **Fallback Strategies**: Implement fallbacks for low-confidence decisions

---

*For more detailed API documentation, see the [rustdoc documentation](https://docs.rs/daa-ai).*