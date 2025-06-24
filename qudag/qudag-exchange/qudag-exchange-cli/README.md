# QuDAG Exchange CLI

[![Crates.io](https://img.shields.io/crates/v/qudag-exchange-standalone-cli.svg)](https://crates.io/crates/qudag-exchange-standalone-cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ruvnet/QuDAG/blob/main/LICENSE)

Standalone command-line interface for QuDAG Exchange with comprehensive rUv token management and business plan payout stream operations.

## Features

### 🏦 Exchange Operations
- **Account Management**: Create and manage rUv token accounts
- **Token Transfers**: Secure quantum-resistant token transfers
- **Balance Checking**: Real-time account balance queries
- **Node Operations**: Start, stop, and monitor exchange nodes

### 💰 Business Plan Management
- **Payout Configuration**: Enable/disable automatic fee distribution
- **Contributor Management**: Register and manage contributors across all roles
- **Split Configuration**: Customize payout percentages and thresholds
- **History Tracking**: View complete payout history and audit trails

### 🌐 Network Operations
- **Peer Management**: Connect to and manage network peers
- **Network Status**: Monitor network health and connectivity
- **Dark Addressing**: Support for .dark domain operations

## Installation

Install from crates.io:

```bash
cargo install qudag-exchange-standalone-cli
```

Or build from source:

```bash
git clone https://github.com/ruvnet/QuDAG
cd QuDAG/qudag-exchange/qudag-exchange-cli
cargo build --release
```

## Quick Start

### Basic Exchange Operations

```bash
# Create a new account
qudag-exchange-cli create-account --name "my-account"

# Check account balance
qudag-exchange-cli balance --account "my-account"

# Transfer tokens
qudag-exchange-cli transfer --from "sender" --to "receiver" --amount 1000
```

### Business Plan Operations

```bash
# Enable business plan features
qudag-exchange-cli business-plan enable \
    --auto-distribution \
    --role-earnings \
    --vault-management

# Check business plan status
qudag-exchange-cli business-plan status

# Register contributors
qudag-exchange-cli business-plan contributors register \
    agent-123 agent-provider vault-abc \
    --custom-percentage 0.90

# Configure payout thresholds
qudag-exchange-cli business-plan configure threshold 50
qudag-exchange-cli business-plan configure system-fee 0.002

# View payout history
qudag-exchange-cli business-plan payouts --limit 10
```

### Node Operations

```bash
# Start an exchange node
qudag-exchange-cli node start --port 8080

# Check node status
qudag-exchange-cli node status

# Stop the node
qudag-exchange-cli node stop
```

### Network Operations

```bash
# Check network status
qudag-exchange-cli network status

# List connected peers
qudag-exchange-cli network peers

# Connect to a specific peer
qudag-exchange-cli network connect "/ip4/192.168.1.100/tcp/8080"
```

## Business Plan Commands

### Enable/Disable Features

```bash
# Enable all business plan features
qudag-exchange-cli business-plan enable \
    --auto-distribution \
    --vault-management \
    --role-earnings \
    --bounty-rewards

# Disable business plan features
qudag-exchange-cli business-plan disable

# Check current status
qudag-exchange-cli business-plan status
```

### Configure Payout Settings

```bash
# Set minimum payout threshold (in rUv)
qudag-exchange-cli business-plan configure threshold 100

# Set system fee percentage (0.0 to 0.1)
qudag-exchange-cli business-plan configure system-fee 0.001

# Configure single-agent payout split
qudag-exchange-cli business-plan configure single-agent \
    --agent 0.95 \
    --infrastructure 0.05

# Configure plugin-enhanced payout split  
qudag-exchange-cli business-plan configure plugin-enhanced \
    --agent 0.85 \
    --plugin 0.10 \
    --infrastructure 0.05
```

### Contributor Management

```bash
# Register different types of contributors
qudag-exchange-cli business-plan contributors register \
    agent-123 agent-provider vault-abc

qudag-exchange-cli business-plan contributors register \
    plugin-456 plugin-creator vault-def \
    --custom-percentage 0.12

qudag-exchange-cli business-plan contributors register \
    node-789 node-operator vault-ghi

qudag-exchange-cli business-plan contributors register \
    bounty-001 bounty-agent vault-jkl

# List all contributors
qudag-exchange-cli business-plan contributors list

# Show specific contributor details
qudag-exchange-cli business-plan contributors show agent-123

# Update contributor settings
qudag-exchange-cli business-plan contributors update agent-123 \
    --custom-percentage 0.88
```

### Payout History

```bash
# View recent payouts
qudag-exchange-cli business-plan payouts

# View more entries
qudag-exchange-cli business-plan payouts --limit 50

# Filter by contributor
qudag-exchange-cli business-plan payouts --contributor agent-123
```

## Contributor Roles

The CLI supports four types of contributors:

### Agent Providers
Earn per compute/storage/bandwidth consumed:
```bash
qudag-exchange-cli business-plan contributors register \
    agent-123 agent-provider vault-abc
```

### Plugin Creators
Earn micro-payouts when modules are used:
```bash
qudag-exchange-cli business-plan contributors register \
    plugin-456 plugin-creator vault-def
```

### Node Operators
Earn via routing/consensus participation:
```bash
qudag-exchange-cli business-plan contributors register \
    node-789 node-operator vault-ghi
```

### Bounty Agents
Claim rewards for task completion:
```bash
qudag-exchange-cli business-plan contributors register \
    bounty-001 bounty-agent vault-jkl
```

## Configuration

The CLI uses the core QuDAG Exchange configuration system. Settings are stored locally and can be managed through the command interface.

### Default Payout Splits

- **Single-Agent Jobs**: 95% agent, 5% infrastructure
- **Plugin-Enhanced Jobs**: 85% agent, 10% plugin, 5% infrastructure
- **Node Operations**: 80% node operator, 15% network, 5% system
- **Bounty Completion**: 90% agent, 5% bounty poster, 5% system

### Custom Percentages

Contributors can override default percentages (subject to governance limits):

```bash
qudag-exchange-cli business-plan contributors register \
    agent-123 agent-provider vault-abc \
    --custom-percentage 0.90  # 90% instead of default 95%
```

## Vault Integration

All payouts are securely distributed to contributor vaults:

- **Zero-Custody**: Contributors control their own vault keys
- **Quantum-Resistant**: ML-DSA-87 signatures for vault access
- **Rate Limiting**: Configurable withdrawal limits and time-locks
- **Audit Trails**: Complete history of all vault operations

## Security Features

- **Quantum-Resistant Signatures**: ML-DSA-87 for all operations
- **Secure Key Storage**: Integration with QuDAG Vault system
- **Transaction Validation**: Comprehensive input validation
- **Replay Protection**: Prevents transaction replay attacks

## Examples

### Complete Workflow

```bash
# 1. Enable business plan features
qudag-exchange-cli business-plan enable --auto-distribution --role-earnings

# 2. Register contributors
qudag-exchange-cli business-plan contributors register agent-001 agent-provider vault-001
qudag-exchange-cli business-plan contributors register plugin-002 plugin-creator vault-002

# 3. Configure settings
qudag-exchange-cli business-plan configure threshold 100
qudag-exchange-cli business-plan configure system-fee 0.001

# 4. Start node
qudag-exchange-cli node start --port 8080

# 5. Monitor operations
qudag-exchange-cli business-plan status
qudag-exchange-cli business-plan payouts --limit 20
```

## Help

Get help for any command:

```bash
qudag-exchange-cli --help
qudag-exchange-cli business-plan --help
qudag-exchange-cli business-plan contributors --help
```

## Documentation

- [QuDAG Exchange Core](https://docs.rs/qudag-exchange-core)
- [Business Plan Specification](https://github.com/ruvnet/QuDAG/blob/main/qudag-exchange/docs/business-plan.md)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)

## License

Licensed under the MIT License. See [LICENSE](https://github.com/ruvnet/QuDAG/blob/main/LICENSE) for details.

---

Part of the [QuDAG](https://github.com/ruvnet/QuDAG) quantum-resistant distributed ledger ecosystem.