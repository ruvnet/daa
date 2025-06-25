# 💰 Economy API Reference

> **Token economics and resource management for DAA agents** - Built-in economic engine for autonomous financial operations and resource allocation.

The `daa-economy` crate provides comprehensive economic management capabilities, including token operations, fee optimization, reward distribution, and risk management for autonomous agents.

---

## 📦 Installation

```toml
[dependencies]
daa-economy = "0.2.0"
```

## 🚀 Quick Start

```rust
use daa_economy::{EconomyManager, TokenManager, RuvToken};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize economy manager
    let mut economy = EconomyManager::new().await?;
    
    // Set initial allocation
    economy.allocate_budget("operations", 50_000).await?;
    economy.allocate_budget("emergency", 10_000).await?;
    
    // Enable auto-rebalancing
    economy.enable_auto_rebalancing(true).await?;
    
    // Execute a transaction
    let tx_result = economy.execute_transaction(
        "operations",
        5_000,
        "Portfolio rebalancing"
    ).await?;
    
    println!("Transaction executed: {:?}", tx_result);
    
    Ok(())
}
```

---

## 🏗️ Core Types

### `EconomyManager`

The main economy management system that coordinates all economic operations.

```rust
pub struct EconomyManager {
    // Internal fields...
}
```

#### Methods

##### `new() -> Result<Self>`

Creates a new economy manager instance.

**Example:**
```rust
let economy = EconomyManager::new().await?;
```

##### `allocate_budget(category: &str, amount: u64) -> Result<()>`

Allocates budget to a specific category.

**Parameters:**
- `category`: Budget category name
- `amount`: Amount to allocate (in base units)

**Returns:** `Result<(), EconomyError>`

**Example:**
```rust
// Allocate budget for different operations
economy.allocate_budget("trading", 100_000).await?;
economy.allocate_budget("gas_fees", 5_000).await?;
economy.allocate_budget("emergency_fund", 20_000).await?;
```

##### `get_balance(category: Option<&str>) -> Result<Balance>`

Gets the current balance for a category or total balance.

**Parameters:**
- `category`: Optional category name (None for total balance)

**Returns:** `Result<Balance, EconomyError>`

**Example:**
```rust
// Get total balance
let total_balance = economy.get_balance(None).await?;
println!("Total balance: {}", total_balance.total);

// Get category balance
let trading_balance = economy.get_balance(Some("trading")).await?;
println!("Trading balance: {}", trading_balance.available);
```

##### `execute_transaction(category: &str, amount: u64, description: &str) -> Result<TransactionResult>`

Executes a transaction from a specific budget category.

**Parameters:**
- `category`: Budget category to debit
- `amount`: Transaction amount
- `description`: Transaction description

**Returns:** `Result<TransactionResult, EconomyError>`

**Example:**
```rust
let result = economy.execute_transaction(
    "trading",
    15_000,
    "Buy ETH on Uniswap"
).await?;

println!("Transaction ID: {}", result.transaction_id);
println!("Final balance: {}", result.remaining_balance);
```

---

## 🪙 Token Management

### `TokenManager`

Manages rUv tokens and operations.

```rust
pub struct TokenManager {
    // Internal fields...
}
```

#### Methods

##### `new(symbol: &str) -> Result<Self>`

Creates a new token manager for a specific token.

**Example:**
```rust
let ruv_manager = TokenManager::new("rUv").await?;
```

##### `mint(recipient: &str, amount: u64) -> Result<MintResult>`

Mints new tokens to a recipient.

**Parameters:**
- `recipient`: Address or agent ID to receive tokens
- `amount`: Amount to mint

**Returns:** `Result<MintResult, TokenError>`

**Example:**
```rust
let mint_result = ruv_manager.mint("agent_001", 10_000).await?;
println!("Minted {} tokens, new supply: {}", 
         mint_result.amount, mint_result.total_supply);
```

##### `transfer(from: &str, to: &str, amount: u64) -> Result<TransferResult>`

Transfers tokens between accounts.

**Parameters:**
- `from`: Source address or agent ID
- `to`: Destination address or agent ID  
- `amount`: Amount to transfer

**Returns:** `Result<TransferResult, TokenError>`

**Example:**
```rust
let transfer_result = ruv_manager.transfer(
    "agent_001", 
    "agent_002", 
    5_000
).await?;

println!("Transfer completed: {}", transfer_result.transaction_hash);
```

##### `burn(holder: &str, amount: u64) -> Result<BurnResult>`

Burns tokens from an account.

**Parameters:**
- `holder`: Address or agent ID holding tokens
- `amount`: Amount to burn

**Returns:** `Result<BurnResult, TokenError>`

**Example:**
```rust
let burn_result = ruv_manager.burn("agent_001", 1_000).await?;
println!("Burned {} tokens, new supply: {}", 
         burn_result.amount, burn_result.total_supply);
```

---

## 📈 Fee Optimization

### `FeeOptimizer`

Dynamically optimizes transaction fees based on network conditions.

```rust
pub struct FeeOptimizer {
    // Internal fields...
}
```

#### Methods

##### `new(config: FeeOptimizerConfig) -> Self`

Creates a new fee optimizer with configuration.

**Example:**
```rust
let fee_config = FeeOptimizerConfig {
    target_confirmation_time: Duration::from_secs(60),
    max_fee_multiplier: 3.0,
    min_fee_threshold: 1_000,
    historical_window: Duration::from_hours(24),
};

let optimizer = FeeOptimizer::new(fee_config);
```

##### `estimate_optimal_fee(transaction_type: TransactionType) -> Result<FeeEstimate>`

Estimates optimal fee for a transaction type.

**Parameters:**
- `transaction_type`: Type of transaction (Transfer, Swap, etc.)

**Returns:** `Result<FeeEstimate, FeeError>`

**Example:**
```rust
let fee_estimate = optimizer.estimate_optimal_fee(TransactionType::TokenTransfer).await?;

println!("Recommended fee: {} gwei", fee_estimate.recommended_fee);
println!("Confirmation time: {:?}", fee_estimate.estimated_confirmation);
println!("Confidence: {}%", fee_estimate.confidence * 100.0);
```

##### `track_transaction_outcome(tx_hash: &str, actual_fee: u64, confirmation_time: Duration) -> Result<()>`

Tracks actual transaction outcomes for learning.

**Example:**
```rust
optimizer.track_transaction_outcome(
    "0xabc123...",
    25_000_000_000, // 25 gwei
    Duration::from_secs(45)
).await?;
```

### Fee Strategies

**Conservative Strategy:**
```rust
let conservative_config = FeeOptimizerConfig {
    target_confirmation_time: Duration::from_secs(300), // 5 minutes
    max_fee_multiplier: 1.5,
    min_fee_threshold: 1_000_000_000, // 1 gwei
    safety_margin: 0.2, // 20% safety margin
};
```

**Aggressive Strategy:**
```rust
let aggressive_config = FeeOptimizerConfig {
    target_confirmation_time: Duration::from_secs(30), // 30 seconds
    max_fee_multiplier: 5.0,
    min_fee_threshold: 5_000_000_000, // 5 gwei
    safety_margin: 0.5, // 50% safety margin
};
```

---

## 🎁 Reward System

### `RewardManager`

Manages incentive distribution and reward mechanisms.

```rust
pub struct RewardManager {
    // Internal fields...
}
```

#### Methods

##### `distribute_rewards(rewards: Vec<Reward>) -> Result<DistributionResult>`

Distributes rewards to multiple recipients.

**Example:**
```rust
let rewards = vec![
    Reward {
        recipient: "agent_001".to_string(),
        amount: 1_000,
        reason: RewardReason::SuccessfulTrade,
        multiplier: 1.0,
    },
    Reward {
        recipient: "agent_002".to_string(),
        amount: 500,
        reason: RewardReason::NetworkMaintenance,
        multiplier: 1.2, // 20% bonus
    },
];

let distribution = reward_manager.distribute_rewards(rewards).await?;
println!("Distributed {} rewards totaling {} tokens", 
         distribution.reward_count, distribution.total_amount);
```

##### `calculate_performance_bonus(agent_id: &str, period: Duration) -> Result<u64>`

Calculates performance-based bonus for an agent.

**Example:**
```rust
let bonus = reward_manager.calculate_performance_bonus(
    "agent_001", 
    Duration::from_secs(30 * 24 * 3600) // 30 days
).await?;

println!("Performance bonus: {} tokens", bonus);
```

### Reward Types

```rust
#[derive(Debug, Clone)]
pub enum RewardReason {
    SuccessfulTrade,
    NetworkMaintenance,
    RiskManagement,
    CommunityContribution,
    EarlyAdoption,
    Staking,
    LiquidityProvision,
    CustomMetric(String),
}

#[derive(Debug, Clone)]
pub struct Reward {
    pub recipient: String,
    pub amount: u64,
    pub reason: RewardReason,
    pub multiplier: f64,
    pub metadata: HashMap<String, String>,
}
```

**Performance Metrics:**
```rust
let metrics = PerformanceMetrics {
    trade_success_rate: 0.85, // 85% success rate
    profit_margin: 0.12, // 12% average profit
    risk_adjusted_return: 0.08, // 8% risk-adjusted return
    uptime_percentage: 0.99, // 99% uptime
};

let bonus = reward_manager.calculate_bonus_from_metrics(&metrics).await?;
```

---

## ⚖️ Risk Management

### `RiskManager`

Assesses and manages economic risks.

```rust
pub struct RiskManager {
    // Internal fields...
}
```

#### Methods

##### `assess_transaction_risk(transaction: &Transaction) -> Result<RiskAssessment>`

Assesses risk for a proposed transaction.

**Example:**
```rust
let transaction = Transaction {
    amount: 50_000,
    token: "ETH".to_string(),
    recipient: "0xabc123...".to_string(),
    transaction_type: TransactionType::TokenTransfer,
};

let risk = risk_manager.assess_transaction_risk(&transaction).await?;

match risk.level {
    RiskLevel::Low => println!("Low risk transaction"),
    RiskLevel::Medium => println!("Medium risk - require approval"),
    RiskLevel::High => println!("High risk - manual review required"),
}
```

##### `calculate_portfolio_var(confidence_level: f64) -> Result<VarResult>`

Calculates Value at Risk for the portfolio.

**Parameters:**
- `confidence_level`: Confidence level (e.g., 0.95 for 95%)

**Returns:** `Result<VarResult, RiskError>`

**Example:**
```rust
let var_result = risk_manager.calculate_portfolio_var(0.95).await?;

println!("95% VaR: {} tokens", var_result.var_amount);
println!("Expected shortfall: {} tokens", var_result.expected_shortfall);
```

### Risk Models

**Portfolio Risk Assessment:**
```rust
#[derive(Debug, Clone)]
pub struct PortfolioRisk {
    pub total_exposure: u64,
    pub concentration_risk: f64,
    pub correlation_risk: f64,
    pub liquidity_risk: f64,
    pub counterparty_risk: f64,
    pub overall_score: f64,
}

let portfolio_risk = risk_manager.assess_portfolio_risk().await?;

if portfolio_risk.overall_score > 0.7 {
    println!("High portfolio risk detected!");
    // Implement risk mitigation strategies
}
```

**Dynamic Risk Limits:**
```rust
let risk_limits = RiskLimits {
    max_single_transaction: 100_000,
    max_daily_volume: 1_000_000,
    max_position_size: 0.1, // 10% of portfolio
    max_drawdown: 0.05, // 5% maximum drawdown
    correlation_limit: 0.8, // Maximum correlation between assets
};

risk_manager.set_dynamic_limits(risk_limits).await?;
```

---

## 📊 Analytics & Reporting

### `EconomyAnalytics`

Provides economic analytics and insights.

```rust
pub struct EconomyAnalytics {
    // Internal fields...
}
```

#### Methods

##### `generate_financial_report(period: ReportPeriod) -> Result<FinancialReport>`

Generates comprehensive financial report.

**Example:**
```rust
let report = analytics.generate_financial_report(ReportPeriod::Monthly).await?;

println!("Total Revenue: {} tokens", report.total_revenue);
println!("Total Expenses: {} tokens", report.total_expenses);
println!("Net Profit: {} tokens", report.net_profit);
println!("ROI: {:.2}%", report.roi * 100.0);

// Detailed breakdowns
for category in &report.expense_categories {
    println!("{}: {} tokens", category.name, category.amount);
}
```

##### `calculate_sharpe_ratio(period: Duration) -> Result<f64>`

Calculates risk-adjusted returns.

**Example:**
```rust
let sharpe_ratio = analytics.calculate_sharpe_ratio(
    Duration::from_secs(90 * 24 * 3600) // 90 days
).await?;

println!("90-day Sharpe ratio: {:.3}", sharpe_ratio);
```

### Performance Metrics

```rust
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
}

let metrics = analytics.calculate_performance_metrics(
    Duration::from_secs(365 * 24 * 3600) // 1 year
).await?;

// Performance summary
println!("Annual Return: {:.2}%", metrics.annualized_return * 100.0);
println!("Volatility: {:.2}%", metrics.volatility * 100.0);
println!("Sharpe Ratio: {:.3}", metrics.sharpe_ratio);
println!("Max Drawdown: {:.2}%", metrics.max_drawdown * 100.0);
```

---

## 💱 Exchange Integration

### `ExchangeManager`

Manages interactions with external exchanges.

```rust
pub struct ExchangeManager {
    // Internal fields...
}
```

#### Methods

##### `execute_swap(swap_request: SwapRequest) -> Result<SwapResult>`

Executes token swaps across exchanges.

**Example:**
```rust
let swap_request = SwapRequest {
    from_token: "USDC".to_string(),
    to_token: "ETH".to_string(),
    amount: 10_000, // 10,000 USDC
    max_slippage: 0.005, // 0.5% slippage
    preferred_exchanges: vec!["Uniswap", "SushiSwap"],
};

let swap_result = exchange_manager.execute_swap(swap_request).await?;

println!("Swapped {} USDC for {} ETH", 
         swap_result.input_amount, swap_result.output_amount);
println!("Effective rate: {} USDC per ETH", swap_result.effective_rate);
```

##### `find_arbitrage_opportunities() -> Result<Vec<ArbitrageOpportunity>>`

Finds arbitrage opportunities across exchanges.

**Example:**
```rust
let opportunities = exchange_manager.find_arbitrage_opportunities().await?;

for opportunity in opportunities {
    if opportunity.profit_potential > 100.0 { // > $100 profit
        println!("Arbitrage opportunity: {} -> {}", 
                 opportunity.from_exchange, opportunity.to_exchange);
        println!("Profit potential: ${:.2}", opportunity.profit_potential);
        
        // Execute arbitrage
        let result = exchange_manager.execute_arbitrage(&opportunity).await?;
        println!("Arbitrage executed: profit = ${:.2}", result.actual_profit);
    }
}
```

---

## 🔧 Configuration

### Economy Configuration

```toml
[economy]
# Base currency settings
base_currency = "rUv"
precision = 18
max_supply = 1000000000000000000000000  # 1 million tokens

# Fee settings
default_fee_strategy = "dynamic"
max_fee_multiplier = 3.0
min_fee_threshold = 1000

# Risk management
max_single_transaction = 100000
max_daily_volume = 1000000
risk_assessment_enabled = true
auto_risk_mitigation = true

# Reward settings
performance_bonus_enabled = true
reward_distribution_frequency = "daily"
base_reward_rate = 0.1

# Analytics
metrics_collection_enabled = true
reporting_frequency = "weekly"
performance_tracking_window = "90d"
```

### Environment Variables

```bash
# Economy configuration
export DAA_ECONOMY_BASE_CURRENCY=rUv
export DAA_ECONOMY_PRECISION=18
export DAA_ECONOMY_MAX_SUPPLY=1000000000000000000000000

# Fee optimization
export DAA_ECONOMY_FEE_STRATEGY=dynamic
export DAA_ECONOMY_MAX_FEE_MULTIPLIER=3.0

# Risk management
export DAA_ECONOMY_RISK_ASSESSMENT=true
export DAA_ECONOMY_MAX_SINGLE_TX=100000

# Exchange integration
export DAA_ECONOMY_EXCHANGE_APIS=uniswap,sushiswap,curve
export DAA_ECONOMY_SLIPPAGE_TOLERANCE=0.005
```

---

## 📊 Advanced Features

### Yield Farming Integration

```rust
use daa_economy::yield_farming::{YieldFarmManager, FarmStrategy};

let yield_manager = YieldFarmManager::new();

// Define farming strategy
let strategy = FarmStrategy {
    min_apy: 0.08, // Minimum 8% APY
    max_risk_score: 0.6, // Maximum risk level
    preferred_protocols: vec!["Aave", "Compound", "Curve"],
    rebalance_threshold: 0.02, // 2% APY difference threshold
};

yield_manager.set_strategy(strategy).await?;

// Auto-compound rewards
yield_manager.enable_auto_compound(true).await?;

// Monitor positions
let positions = yield_manager.get_active_positions().await?;
for position in positions {
    println!("Protocol: {}, APY: {:.2}%, Amount: {}", 
             position.protocol, position.current_apy * 100.0, position.amount);
}
```

### Liquidity Management

```rust
use daa_economy::liquidity::{LiquidityManager, LiquidityPool};

let liquidity_manager = LiquidityManager::new();

// Provide liquidity to pools
let pool_allocation = vec![
    PoolAllocation {
        pool: "ETH/USDC".to_string(),
        percentage: 0.4, // 40% allocation
        min_fee_tier: 0.0005, // 0.05% minimum fee
    },
    PoolAllocation {
        pool: "BTC/ETH".to_string(),
        percentage: 0.3, // 30% allocation
        min_fee_tier: 0.003, // 0.3% minimum fee
    },
];

liquidity_manager.allocate_liquidity(pool_allocation).await?;

// Monitor impermanent loss
let il_analysis = liquidity_manager.analyze_impermanent_loss().await?;
if il_analysis.max_loss > 0.05 { // 5% loss threshold
    liquidity_manager.exit_high_risk_positions().await?;
}
```

---

## 🚨 Error Handling

### `EconomyError`

Main error type for economy operations.

```rust
#[derive(Error, Debug)]
pub enum EconomyError {
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Invalid amount: {0}")]
    InvalidAmount(u64),
    
    #[error("Risk threshold exceeded: {current} > {threshold}")]
    RiskThresholdExceeded { current: f64, threshold: f64 },
    
    #[error("Exchange error: {0}")]
    Exchange(#[from] ExchangeError),
    
    #[error("Token error: {0}")]
    Token(#[from] TokenError),
}
```

### Error Handling Best Practices

```rust
use daa_economy::{EconomyManager, EconomyError};

async fn safe_transaction_execution(
    economy: &EconomyManager,
    category: &str,
    amount: u64,
    description: &str
) -> Result<TransactionResult, EconomyError> {
    // Pre-flight checks
    let balance = economy.get_balance(Some(category)).await?;
    if balance.available < amount {
        return Err(EconomyError::InsufficientBalance {
            required: amount,
            available: balance.available,
        });
    }
    
    // Risk assessment
    let risk_score = economy.assess_transaction_risk(category, amount).await?;
    if risk_score > 0.8 {
        return Err(EconomyError::RiskThresholdExceeded {
            current: risk_score,
            threshold: 0.8,
        });
    }
    
    // Execute with retry logic
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 3;
    
    loop {
        match economy.execute_transaction(category, amount, description).await {
            Ok(result) => return Ok(result),
            Err(EconomyError::TransactionFailed(ref msg)) if attempts < MAX_ATTEMPTS => {
                attempts += 1;
                log::warn!("Transaction attempt {} failed: {}", attempts, msg);
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 📚 Examples

### Complete Treasury Management

```rust
use daa_economy::prelude::*;

async fn setup_treasury_management() -> Result<(), EconomyError> {
    let mut economy = EconomyManager::new().await?;
    
    // Initial setup
    economy.allocate_budget("operations", 500_000).await?;
    economy.allocate_budget("trading", 300_000).await?;
    economy.allocate_budget("emergency", 100_000).await?;
    economy.allocate_budget("rewards", 50_000).await?;
    
    // Configure risk management
    let risk_config = RiskConfig {
        max_single_transaction: 50_000,
        max_daily_volume: 200_000,
        risk_assessment_threshold: 0.7,
        auto_mitigation: true,
    };
    economy.configure_risk_management(risk_config).await?;
    
    // Setup fee optimization
    let fee_config = FeeOptimizerConfig {
        target_confirmation_time: Duration::from_secs(60),
        max_fee_multiplier: 2.0,
        strategy: FeeStrategy::Adaptive,
    };
    economy.configure_fee_optimization(fee_config).await?;
    
    // Enable auto-rebalancing
    economy.enable_auto_rebalancing(true).await?;
    economy.set_rebalance_threshold(0.1).await?; // 10% threshold
    
    // Setup monitoring
    economy.enable_real_time_monitoring(true).await?;
    economy.set_alert_thresholds(AlertThresholds {
        low_balance: 10_000,
        high_risk: 0.8,
        unusual_activity: 5.0,
    }).await?;
    
    println!("Treasury management system initialized successfully");
    Ok(())
}
```

---

## 🔗 Related Documentation

- [Orchestrator API](./orchestrator.md) - Core coordination engine
- [Rules Engine API](./rules.md) - Governance and decision making
- [AI Integration API](./ai.md) - Claude AI integration
- [Chain API](./chain.md) - Blockchain abstraction layer

---

## 📊 Performance Benchmarks

### Typical Performance

| Operation | Throughput | Latency |
|-----------|------------|---------|
| Balance queries | 100,000/sec | <0.1ms |
| Simple transactions | 10,000/sec | <1ms |
| Risk assessments | 5,000/sec | <2ms |
| Fee optimizations | 1,000/sec | <5ms |
| Analytics calculations | 100/sec | <50ms |

### Optimization Guidelines

1. **Batch Operations**: Group multiple transactions for better throughput
2. **Caching**: Enable balance and risk assessment caching
3. **Async Processing**: Use async operations for all I/O
4. **Connection Pooling**: Configure database connection pools appropriately
5. **Monitoring**: Enable performance monitoring for bottleneck identification

---

*For more detailed API documentation, see the [rustdoc documentation](https://docs.rs/daa-economy).*