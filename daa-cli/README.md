# DAA CLI

**🚀 FULL IMPLEMENTATION - This is the complete, production-ready implementation of the DAA CLI, not a placeholder.**

Command-line interface for the Decentralized Autonomous Agents (DAA) system, providing comprehensive tooling for managing DAA components with QuDAG configuration options.

## Overview

DAA CLI is the primary command-line tool for interacting with and managing the DAA ecosystem. It provides unified access to all DAA components including blockchain operations, economic management, rules governance, AI coordination, and workflow orchestration.

## Installation

### From Source
```bash
# Clone the repository
git clone https://github.com/daa-hq/daa-sdk
cd daa-sdk

# Build the CLI
cargo build --release --package daa-cli

# Install globally
cargo install --path daa-cli
```

### Binary Releases
Download pre-built binaries from the [releases page](https://github.com/daa-hq/daa-sdk/releases).

## Quick Start

### Initialize DAA System
```bash
# Initialize with default configuration
daa init

# Initialize with custom template
daa init --template advanced

# Force overwrite existing configuration
daa init --force
```

### Check System Status
```bash
# Basic status check
daa status

# Detailed component status
daa status --detailed
```

### Configuration Management
```bash
# Show current configuration
daa config show

# Get specific configuration value
daa config get chain.network

# Set configuration value
daa config set ai.model claude-3-opus

# Validate configuration
daa config validate

# Reset to defaults
daa config reset
```

## Available Commands

### Core Commands
- `daa init` - Initialize DAA system configuration
- `daa status` - Display system status and health
- `daa config` - Manage configuration settings

### Component Commands (Feature-dependent)

#### Chain Operations (with `chain` feature)
```bash
# Start blockchain node
daa chain start

# Check blockchain status
daa chain status

# Submit transaction
daa chain tx send --to <address> --amount <amount>

# Query block information
daa chain block get <hash>
```

#### Economy Operations (with `economy` feature)
```bash
# Check account balance
daa economy balance <account>

# Transfer rUv tokens
daa economy transfer --from <account> --to <account> --amount <amount>

# View market data
daa economy market --pair rUv/ETH

# Distribute rewards
daa economy reward --account <account> --type task_completion
```

#### Rules Management (with `rules` feature)
```bash
# List active rules
daa rules list

# Add new rule
daa rules add --file rule.json

# Test rule execution
daa rules test --rule <rule_id> --context context.json

# Enable/disable rules
daa rules enable <rule_id>
daa rules disable <rule_id>
```

#### AI Operations (with `ai` feature)
```bash
# Spawn AI agent
daa ai spawn --type researcher --capabilities web_search,analysis

# List active agents
daa ai agents list

# Execute task
daa ai task execute --agent <agent_id> --task "research quantum computing"

# View agent memory
daa ai memory get --agent <agent_id>
```

#### Orchestration (with `orchestrator` feature)
```bash
# Start orchestrator
daa orchestrator start

# Execute workflow
daa orchestrator workflow run --file workflow.json

# List services
daa orchestrator services list

# Register service
daa orchestrator services register --type ai_agent --endpoint http://localhost:8080
```

## Configuration

### Configuration File Locations
- **Global**: `~/.config/daa/config.toml`
- **Project**: `./daa.toml`
- **Custom**: Specified with `--config` flag

### Configuration Structure
```toml
[chain]
enabled = true
network = "mainnet"
rpc_url = "http://localhost:8545"

[economy]
enabled = true
base_currency = "rUv"
exchange_url = "http://localhost:3000"

[rules]
enabled = true
rules_file = "rules.json"
auto_reload = true

[ai]
enabled = true
model = "claude-3-opus"
api_key = "${ANTHROPIC_API_KEY}"
mcp_server = "http://localhost:3000"

[orchestrator]
enabled = true
port = 8080
max_workflows = 100
```

### Environment Variables
- `DAA_CONFIG` - Configuration file path
- `DAA_LOG_LEVEL` - Logging level (trace, debug, info, warn, error)
- `ANTHROPIC_API_KEY` - Claude API key
- `DAA_CHAIN_RPC` - Blockchain RPC endpoint

## Output Formats

DAA CLI supports multiple output formats:

### JSON Output
```bash
daa status --output json
```

### Table Output (default)
```bash
daa status --output table
```

### YAML Output
```bash
daa status --output yaml
```

## Global Options

All commands support these global options:

- `--verbose, -v` - Enable verbose output
- `--config, -c <file>` - Specify configuration file
- `--output, -o <format>` - Output format (json, table, yaml)
- `--no-color` - Disable colored output

## Examples

### Complete Workflow Example
```bash
# 1. Initialize system
daa init --template production

# 2. Start core services
daa chain start &
daa orchestrator start &

# 3. Spawn AI agent
AGENT_ID=$(daa ai spawn --type researcher --output json | jq -r '.agent_id')

# 4. Create and execute workflow
cat > workflow.json << EOF
{
  "name": "Research and Report",
  "steps": [
    {
      "type": "ai_task",
      "agent_id": "$AGENT_ID",
      "task": "research recent developments in quantum computing"
    },
    {
      "type": "economy_reward",
      "account": "$AGENT_ID",
      "amount": 100,
      "reason": "research completion"
    }
  ]
}
EOF

daa orchestrator workflow run --file workflow.json

# 5. Check results
daa economy balance $AGENT_ID
daa ai memory get --agent $AGENT_ID
```

### Rule-based Task Automation
```bash
# Create a rule for automatic rewards
cat > auto_reward_rule.json << EOF
{
  "name": "Auto Reward High Performance",
  "conditions": [
    {
      "type": "performance_score",
      "operator": "greater_than",
      "value": 0.9
    }
  ],
  "actions": [
    {
      "type": "economy_reward",
      "amount": 50,
      "reason": "high_performance_bonus"
    }
  ]
}
EOF

# Add the rule
daa rules add --file auto_reward_rule.json

# Test the rule
daa rules test --rule auto_reward --context '{"performance_score": 0.95}'
```

## Development

### Building from Source
```bash
# Build with all features
cargo build --release --features full

# Build with specific features
cargo build --release --features "chain,economy,ai"

# Run tests
cargo test --package daa-cli
```

### Adding Custom Commands
The CLI is extensible through the plugin system:

```rust
// In your plugin crate
use daa_cli::{Plugin, CommandResult};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn commands(&self) -> Vec<Command> {
        vec![
            Command::new("my-command")
                .about("My custom command")
                .handler(|args| {
                    // Command implementation
                    Ok(())
                })
        ]
    }
}
```

## Troubleshooting

### Common Issues

#### Configuration Not Found
```bash
# Check configuration location
daa config show

# Create default configuration
daa init
```

#### Service Connection Errors
```bash
# Check service status
daa status --detailed

# Verify network connectivity
ping localhost
```

#### Permission Errors
```bash
# Check file permissions
ls -la ~/.config/daa/

# Fix permissions
chmod 600 ~/.config/daa/config.toml
```

### Debug Mode
Enable debug logging for troubleshooting:

```bash
export DAA_LOG_LEVEL=debug
daa --verbose status
```

## Features

The CLI supports feature flags for optional components:

- `default`: Basic CLI functionality
- `chain`: Blockchain operations
- `economy`: Economic management
- `rules`: Rules engine integration
- `ai`: AI agent operations
- `orchestrator`: Workflow orchestration
- `full`: All features enabled

## License

MIT OR Apache-2.0