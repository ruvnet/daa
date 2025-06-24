# QuDAG Exchange Core

[![Crates.io](https://img.shields.io/crates/v/qudag-exchange-core.svg)](https://crates.io/crates/qudag-exchange-core)
[![Documentation](https://docs.rs/qudag-exchange-core/badge.svg)](https://docs.rs/qudag-exchange-core)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ruvnet/QuDAG/blob/main/LICENSE)

Core library for the QuDAG Exchange system with quantum-resistant rUv (Resource Utilization Voucher) token functionality and business plan payout streams.

## Features

### 🏦 Core Exchange Functionality
- **rUv Token System**: Resource utilization vouchers for quantum-secure transactions
- **Dynamic Fee Model**: Tiered fee structure based on agent verification and usage
- **Immutable Deployment**: Lock system configuration with quantum-resistant signatures
- **Quantum-Resistant Security**: ML-DSA-87 signatures and post-quantum cryptography

### 💰 Business Plan & Payout Streams
- **Vault-Based Distribution**: Automatic fee distribution to contributor vaults
- **Role-Based Earnings**: Support for agent providers, plugin creators, node operators, and bounty agents
- **Configurable Splits**: Default templates with custom percentage overrides
- **Audit Trails**: Complete transaction history and payout tracking
- **Governance Controls**: Optional approval thresholds and voting mechanisms

### 🔧 Configuration & Management
- **Optional Features**: All business plan features are opt-in
- **Flexible Configuration**: Granular control over all system parameters
- **WASM Compatibility**: no_std support for WebAssembly deployment
- **Integration Ready**: Works with QuDAG Vault and DAG consensus

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
qudag-exchange-core = "0.3.0"
```

### Basic Usage

```rust
use qudag_exchange_core::{
    ExchangeConfig, ExchangeConfigBuilder, BusinessPlanConfig,
    rUv, AccountId, types::Timestamp
};

// Create basic exchange configuration
let config = ExchangeConfig::new()?;

// Enable business plan features
let config = ExchangeConfigBuilder::new()
    .with_basic_business_plan()
    .build()?;

// Create and transfer rUv tokens
let sender = AccountId::new("sender_vault");
let receiver = AccountId::new("receiver_vault"); 
let amount = rUv::new(1000);
```

### Business Plan Integration

```rust
use qudag_exchange_core::{
    PayoutConfig, FeeRouter, ContributorRole, ContributorInfo
};

// Configure automatic payouts
let payout_config = PayoutConfig {
    enabled: true,
    min_payout_threshold: rUv::new(50),
    system_fee_percentage: 0.002,
    ..Default::default()
};

let mut fee_router = FeeRouter::new(payout_config);

// Register a contributor
let contributor = ContributorInfo {
    vault_id: AccountId::new("contributor_vault"),
    role: ContributorRole::AgentProvider {
        agent_id: "agent_123".to_string(),
        resource_consumed: 100,
    },
    custom_percentage: Some(0.90), // 90% instead of default 95%
    registered_at: Timestamp::now(),
    total_earnings: rUv::new(0),
    last_payout: None,
};

fee_router.register_contributor("agent_123".to_string(), contributor)?;

// Distribute fees automatically
let roles = vec![ContributorRole::AgentProvider {
    agent_id: "agent_123".to_string(),
    resource_consumed: 100,
}];

let payout_tx = fee_router.distribute_fees(
    "tx_001".to_string(),
    rUv::new(1000), // Total fee collected
    roles,
    Timestamp::now(),
)?;
```

## Architecture

### Core Components

- **ExchangeConfig**: Main configuration management with optional business plan features
- **FeeRouter**: Automatic fee distribution engine with vault-based payouts
- **PayoutConfig**: Configurable payout parameters and split templates
- **ContributorRole**: Type-safe representation of different contributor types
- **rUv**: Quantum-resistant resource utilization voucher token

### Payout System

The business plan implements a vault-based payout stream system:

1. **Fee Collection**: Transaction fees are automatically captured
2. **Role Recognition**: Contributors are identified by their roles (agent, plugin, node, bounty)
3. **Split Calculation**: Fees are split according to configurable templates
4. **Vault Distribution**: Payouts are deposited to contributor vaults
5. **Audit Trail**: Complete history is maintained for transparency

### Default Payout Splits

- **Single-Agent Jobs**: 95% agent, 5% infrastructure
- **Plugin-Enhanced**: 85% agent, 10% plugin, 5% infrastructure  
- **Node Operations**: 80% node operator, 15% network, 5% system
- **Bounty Completion**: 90% agent, 5% bounty poster, 5% system

## Configuration

### Basic Configuration

```rust
let config = ExchangeConfigBuilder::new()
    .with_chain_id(1)
    .with_network_name("qudag-mainnet")
    .build()?;
```

### Business Plan Configuration

```rust
let bp_config = BusinessPlanConfig {
    enabled: true,
    enable_auto_distribution: true,
    enable_vault_management: true,
    enable_role_earnings: true,
    enable_bounty_rewards: true,
    payout_config: PayoutConfig {
        enabled: true,
        min_payout_threshold: rUv::new(100),
        system_fee_percentage: 0.001,
        ..Default::default()
    },
    ..Default::default()
};

let config = ExchangeConfigBuilder::new()
    .with_business_plan(bp_config)
    .build()?;
```

## CLI Tool

For command-line interaction, use the standalone CLI:

```bash
cargo install qudag-exchange-standalone-cli

# Enable business plan features
qudag-exchange-cli business-plan enable --auto-distribution --role-earnings

# Register contributors
qudag-exchange-cli business-plan contributors register agent-123 agent-provider vault-abc

# View status and history
qudag-exchange-cli business-plan status
qudag-exchange-cli business-plan payouts --limit 10
```

## Security

- **Quantum-Resistant**: Uses ML-DSA-87 signatures and post-quantum cryptography
- **Vault Integration**: Secure payout storage through QuDAG Vault
- **Audit Trails**: Complete transaction and payout history
- **Validation**: Comprehensive parameter validation and overflow protection
- **Zero-Custody**: Contributors control their own vault keys

## WASM Support

The library supports WebAssembly deployment with `no_std`:

```toml
[dependencies]
qudag-exchange-core = { version = "0.3.0", default-features = false }
```

## Examples

See the [examples directory](https://github.com/ruvnet/QuDAG/tree/main/qudag-exchange/core/examples) for complete usage examples.

## Testing

Run the comprehensive test suite:

```bash
cargo test
cargo test business_plan_integration
```

## Documentation

- [API Documentation](https://docs.rs/qudag-exchange-core)
- [Business Plan Specification](https://github.com/ruvnet/QuDAG/blob/main/qudag-exchange/docs/business-plan.md)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)

## License

Licensed under the MIT License. See [LICENSE](https://github.com/ruvnet/QuDAG/blob/main/LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](https://github.com/ruvnet/QuDAG/blob/main/CONTRIBUTING.md).

---

Part of the [QuDAG](https://github.com/ruvnet/QuDAG) quantum-resistant distributed ledger ecosystem.