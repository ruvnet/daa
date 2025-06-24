# Command Reference

Complete reference for all QuDAG CLI commands.

## Global Options

```bash
--config    Path to config file (optional)
--help      Show help information
--version   Show version information
```

## Node Commands

### start
Start a new QuDAG node instance
```bash
qudag start [OPTIONS]

Options:
--data-dir <PATH>     Directory for node data storage (default: "./data")
--port <PORT>         Network port to listen on (default: 8000)
--peers <ADDRESSES>   List of initial peer addresses to connect to
```

### stop
Gracefully stop the running node
```bash
qudag stop
```

### status
Show current network and node status
```bash
qudag status

# Shows:
# - Active peer count
# - Messages in DAG
# - Number of tips
# - Last consensus round
```

### node status
Show node status and information
```bash
qudag node status [--json]
```

### node metrics
Display node performance metrics
```bash
qudag node metrics [--format <text|json|prometheus>]
```

## Network Commands

### peer
Manage peer connections
```bash
# List all connected peers
qudag peer list

# Connect to a new peer
qudag peer add <address>

# Disconnect from a peer
qudag peer remove <address>
```

### network stats
Show network statistics
```bash
qudag network stats [--interval <seconds>]
```

### network test
Test network connectivity
```bash
qudag network test [--timeout <seconds>]
```

## DAG Commands

### dag
Generate DAG visualization
```bash
qudag dag [OPTIONS]

Options:
--output <PATH>    Output file path (default: "dag_visualization.dot")
--format <FORMAT>  Output format (default: "dot")
```

## Configuration Commands

### config show
Show current configuration
```bash
qudag config show [--format <text|json|toml>]
```

### config set
Set configuration values
```bash
qudag config set <key> <value>
```

### config import
Import configuration from file
```bash
qudag config import <file>
```

### config export
Export configuration to file
```bash
qudag config export <file>
```

## Monitoring Commands

### monitor
Monitor node status and metrics
```bash
qudag monitor [--metrics] [--log] [--interval <seconds>]
```

### logs
View and manage node logs
```bash
qudag logs show [--follow] [--lines <n>]
qudag logs export <file>
```

## Debugging Commands

### debug network
Debug network issues
```bash
qudag debug network [--verbose]
```

### debug consensus
Debug consensus issues
```bash
qudag debug consensus [--verbose]
```

### debug profile
Profile node performance
```bash
qudag debug profile [--duration <seconds>]
```

## Exchange Commands

### exchange create-account
Create a new rUv token account
```bash
qudag exchange create-account --name <account-name> [--email <email>]
```

### exchange balance
Check account balance
```bash
qudag exchange balance --account <account-name> [--format <text|json>]
```

### exchange transfer
Transfer rUv tokens between accounts
```bash
qudag exchange transfer --from <sender> --to <receiver> --amount <amount>
```

### exchange list-accounts
List all accounts
```bash
qudag exchange list-accounts [--format <text|json|table>]
```

### exchange fee-info
Show fee model information
```bash
qudag exchange fee-info [--examples] [--format <text|json>]
```

### exchange verify-agent
Verify agent status for reduced fees
```bash
qudag exchange verify-agent <account> --proof-path <path>
```

### exchange calculate-fee
Calculate transaction fee
```bash
qudag exchange calculate-fee --account <account> --amount <amount>
```

### exchange immutable-status
Check immutable deployment status
```bash
qudag exchange immutable-status [--format <text|json>]
```

### exchange deploy-immutable
Deploy exchange in immutable mode
```bash
qudag exchange deploy-immutable --key-path <quantum-key-path>
```

## Business Plan Commands

### business-plan enable
Enable business plan features
```bash
qudag exchange business-plan enable [OPTIONS]

Options:
--auto-distribution     Enable automatic fee distribution
--vault-management      Enable vault management features
--role-earnings        Enable role-based earnings
--bounty-rewards       Enable bounty reward system
```

### business-plan disable
Disable business plan features
```bash
qudag exchange business-plan disable
```

### business-plan status
Show business plan status
```bash
qudag exchange business-plan status
```

### business-plan configure
Configure business plan settings
```bash
# Set minimum payout threshold
qudag exchange business-plan configure threshold <amount>

# Set system fee percentage
qudag exchange business-plan configure system-fee <percentage>

# Configure single-agent split
qudag exchange business-plan configure single-agent --agent <pct> --infrastructure <pct>

# Configure plugin-enhanced split
qudag exchange business-plan configure plugin-enhanced --agent <pct> --plugin <pct> --infrastructure <pct>
```

### business-plan contributors
Manage business plan contributors
```bash
# Register contributor
qudag exchange business-plan contributors register <id> <role> <vault-id> [--custom-percentage <pct>]

# List all contributors
qudag exchange business-plan contributors list

# Show contributor details
qudag exchange business-plan contributors show <id>

# Update contributor
qudag exchange business-plan contributors update <id> --custom-percentage <pct>
```

### business-plan payouts
View payout history
```bash
qudag exchange business-plan payouts [--limit <n>] [--contributor <id>]
```

## Advanced Commands

### crypto verify
Verify cryptographic operations
```bash
qudag crypto verify [--algorithm <name>]
```

### benchmark
Run performance benchmarks
```bash
qudag benchmark [--test <name>] [--duration <seconds>]
```

### maintenance
Perform maintenance tasks
```bash
qudag maintenance [--task <name>]
```