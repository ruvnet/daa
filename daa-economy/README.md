# DAA Economy

**🚀 FULL IMPLEMENTATION - This is the complete, production-ready implementation of the DAA Economy module, not a placeholder.**

Economic layer for the Decentralized Autonomous Agents (DAA) system, providing rUv token management, exchange operations, and economic incentives through QuDAG exchange integration.

## Overview

DAA Economy manages the complete economic ecosystem for autonomous agents, including:

- **rUv Token Management**: Native currency for resource valuation and exchange
- **Exchange Operations**: QuDAG-integrated trading and market making
- **Reward Systems**: Performance-based incentive mechanisms
- **Account Management**: Agent financial account lifecycle
- **Market Operations**: Liquidity provision and price discovery

## Features

### Core Economic Functions
- **Token Operations**: Mint, burn, transfer, lock/unlock rUv tokens
- **Account Management**: Create and manage agent financial accounts
- **Transfer System**: Secure token transfers with fee calculation
- **Balance Tracking**: Real-time balance and transaction history

### Exchange Integration
- **QuDAG Exchange**: Native integration with QuDAG trading infrastructure
- **Order Management**: Place, track, and manage trading orders
- **Market Making**: Automated liquidity provision
- **Price Discovery**: Real-time market data and pricing

### Reward Systems
- **Task Rewards**: Automatic rewards for task completion
- **Quality Bonuses**: Performance-based reward multipliers
- **Staking Rewards**: Incentives for network participation
- **Referral Programs**: Growth incentive mechanisms

### Advanced Features
- **Fee Management**: Configurable transaction and exchange fees
- **Supply Control**: Token inflation and maximum supply management
- **Database Integration**: Optional persistent storage
- **Chain Integration**: Optional blockchain transaction recording

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  EconomySystem  │    │  TokenManager   │    │ ExchangeManager │
│                 │    │                 │    │                 │
│ - Configuration │◄──►│ - rUv Tokens    │◄──►│ - QuDAG Exchange│
│ - Coordination  │    │ - Balances      │    │ - Order Book    │
│ - Statistics    │    │ - Transactions  │    │ - Trade History │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ AccountManager  │    │  RewardSystem   │    │  MarketManager  │
│                 │    │                 │    │                 │
│ - Agent Accounts│    │ - Reward Calc   │    │ - Market Data   │
│ - KYC/Status    │    │ - Performance   │    │ - Liquidity     │
│ - Metadata      │    │ - Distribution  │    │ - Price Oracle  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Usage

### Basic Setup

```rust
use daa_economy::{EconomySystem, EconomyConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create economy configuration
    let config = EconomyConfig {
        base_currency: CurrencyConfig {
            symbol: "rUv".to_string(),
            name: "Resource Units of value".to_string(),
            initial_supply: Decimal::from(1_000_000_000),
            max_supply: Some(Decimal::from(10_000_000_000)),
            inflation_rate: Decimal::from(5) / Decimal::from(100),
            ..Default::default()
        },
        ..Default::default()
    };

    // Initialize economy system
    let mut economy = EconomySystem::new(config).await?;
    economy.initialize().await?;
    
    println!("DAA Economy System initialized");
    Ok(())
}
```

### Account Operations

```rust
// Create agent account
let account = economy.create_account("agent-123".to_string()).await?;
println!("Created account: {}", account.id);

// Check balance
let balance = economy.get_balance(&account.id).await?;
println!("Account balance: {} rUv", balance);

// Transfer tokens
let tx_id = economy.transfer(
    &from_account,
    &to_account,
    Decimal::from(100), // 100 rUv
).await?;
println!("Transfer completed: {}", tx_id);
```

### Reward Distribution

```rust
use daa_economy::rewards::RewardType;

// Distribute task completion reward
let reward_amount = economy.distribute_reward(
    &account.id,
    RewardType::TaskCompletion,
    None, // No performance score
).await?;

// Distribute performance-based reward
let performance_reward = economy.distribute_reward(
    &account.id,
    RewardType::HighQualityWork,
    Some(Decimal::from(95) / Decimal::from(100)), // 95% performance
).await?;

println!("Rewards distributed: {} rUv", reward_amount + performance_reward);
```

### Exchange Operations

```rust
use qudag_exchange::OrderType;

// Place buy order
let order_id = economy.place_order(
    &account.id,
    OrderType::Buy,
    "rUv".to_string(),    // Base token
    "ETH".to_string(),    // Quote token
    Decimal::from(1000),  // Quantity
    Decimal::from(100),   // Price
).await?;

// Get market data
let market_data = economy.get_market_data("rUv", "ETH").await?;
println!("rUv/ETH Price: {}", market_data.last_price);
```

## Configuration

### Economy Configuration

```rust
EconomyConfig {
    base_currency: CurrencyConfig {
        symbol: "rUv".to_string(),
        name: "Resource Units of value".to_string(),
        decimals: 18,
        initial_supply: Decimal::from(1_000_000_000),
        max_supply: Some(Decimal::from(10_000_000_000)),
        inflation_rate: Decimal::from(5) / Decimal::from(100), // 5% per year
    },
    
    rewards: RewardConfig {
        base_task_reward: Decimal::from(10),      // 10 rUv per task
        quality_multiplier: Decimal::from(2),     // 2x for excellent work
        failure_penalty: Decimal::from(5),        // -5 rUv for failures
        staking_rewards: Decimal::from(100),      // 100 rUv per epoch
        minimum_stake: Decimal::from(1000),       // 1000 rUv minimum
    },
    
    fees: FeeConfig {
        transaction_fee: Decimal::from(1) / Decimal::from(1000), // 0.1%
        exchange_fee: Decimal::from(5) / Decimal::from(1000),    // 0.5%
        withdrawal_fee: Decimal::from(1),                        // 1 rUv
        minimum_fee: Decimal::from(1) / Decimal::from(100),      // 0.01 rUv
    },
    
    market_maker: MarketMakerConfig {
        initial_liquidity: Decimal::from(100_000), // 100k rUv
        spread: Decimal::from(1) / Decimal::from(100), // 1% spread
        min_order_size: Decimal::from(1),
        max_order_size: Decimal::from(10_000),
    },
}
```

## Token Economics

### rUv Token Properties
- **Symbol**: rUv (Resource Units of value)
- **Decimals**: 18
- **Initial Supply**: 1 billion rUv
- **Maximum Supply**: 10 billion rUv
- **Inflation**: 5% annually (configurable)

### Reward Structure
- **Task Completion**: 10 rUv base reward
- **High Quality Work**: Up to 2x multiplier based on performance
- **Staking**: 100 rUv per epoch for validators
- **Bug Reports**: 30 rUv for verified bugs
- **Referrals**: 5 rUv for successful agent onboarding

### Fee Structure
- **Transaction Fee**: 0.1% of transfer amount
- **Exchange Fee**: 0.5% of trade value
- **Withdrawal Fee**: 1 rUv flat fee
- **Minimum Fee**: 0.01 rUv floor

## Features

The crate supports several feature flags:

- `default`: Includes exchange and tokens features
- `exchange`: Enables QuDAG exchange integration
- `tokens`: Basic token management (always enabled)
- `chain-integration`: Enables blockchain transaction recording
- `database`: Adds persistent database storage
- `full`: Includes all features

```toml
[dependencies]
daa-economy = { version = "0.1.0", features = ["full"] }
```

## Integration Examples

### With DAA Chain
```rust
#[cfg(feature = "chain-integration")]
use daa_economy::chain_bridge::ChainBridge;

// Record transactions on blockchain
let bridge = ChainBridge::new(chain_client).await?;
economy.set_chain_bridge(Some(bridge)).await?;

// Transfers will now be recorded on-chain
let tx_id = economy.transfer(&from, &to, amount).await?;
```

### With Database
```rust
#[cfg(feature = "database")]
let config = EconomyConfig {
    database_url: Some("sqlite://economy.db".to_string()),
    ..Default::default()
};

// All operations will be persisted to database
let mut economy = EconomySystem::new(config).await?;
economy.initialize().await?;
```

## API Reference

### EconomySystem
Main system coordinator managing all economic operations.

**Key Methods:**
- `new(config)` - Create new economy system
- `initialize()` - Initialize all subsystems
- `create_account(agent_id)` - Create agent account
- `transfer(from, to, amount)` - Transfer tokens
- `distribute_reward(account, type, score)` - Distribute rewards
- `place_order(account, type, tokens, qty, price)` - Place exchange order
- `get_statistics()` - Get system statistics

### TokenManager
Manages rUv tokens and balances.

**Key Methods:**
- `mint(account, amount)` - Create new tokens
- `burn(account, amount)` - Destroy tokens
- `transfer_token(from, to, token, amount, fee)` - Transfer specific token
- `lock_tokens(account, token, amount)` - Lock tokens for staking/orders
- `get_balance(account)` - Get account balance

### ExchangeManager
Integrates with QuDAG exchange for trading.

**Key Methods:**
- `place_order(account, type, base, quote, qty, price)` - Place trading order
- `cancel_order(order_id)` - Cancel existing order
- `get_order_book(base, quote)` - Get current order book

## Testing

Run the test suite:

```bash
# Basic tests
cargo test --package daa-economy

# All features
cargo test --package daa-economy --all-features

# Specific feature
cargo test --package daa-economy --features database
```

## Dependencies

### QuDAG Dependencies
- `qudag-core`: Core blockchain primitives
- `qudag-exchange`: Exchange and trading functionality

### Economic Dependencies
- `rust_decimal`: Precise decimal arithmetic for financial calculations
- `chrono`: Date and time handling
- `bigdecimal`: Extended precision decimal numbers

### Optional Dependencies
- `sqlx`: Database integration (with `database` feature)
- `daa-chain`: Blockchain integration (with `chain-integration` feature)

## License

MIT OR Apache-2.0