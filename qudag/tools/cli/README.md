# QuDAG CLI

Comprehensive command-line interface for QuDAG - the darknet for agent swarms with quantum-resistant distributed communication, rUv token exchange, and business plan payouts.

## Features

- **Node Management**: Start, stop, and manage QuDAG nodes
- **Exchange Operations**: rUv token transfers with dynamic fee models
- **Business Plan**: Automated payout distribution for contributors
- **Dark Addressing**: Anonymous .dark domain registration and routing
- **MCP Server**: Model Context Protocol integration for AI agents
- **Quantum Crypto**: ML-DSA signatures and ML-KEM encryption
- **Password Vault**: Quantum-resistant credential management
- **P2P Networking**: Onion routing and NAT traversal

## Installation

```bash
cargo install qudag-cli
```

> **Note:** The CLI binary is named `qudag` after installation.

## Quick Start

```bash
# Start a QuDAG node with exchange enabled
qudag start --port 8000 --enable-exchange

# Create an rUv token account
qudag exchange create-account --name "alice" --email "alice@example.com"

# Register your darknet domain
qudag address register mynode.dark

# Start MCP server for AI agent integration
qudag mcp start --port 3000

# Check comprehensive status
qudag status
```

## Commands

### Node Management

```bash
# Start a node
qudag start [--port 8000] [--rpc-port 9090]

# Stop the node
qudag stop

# Get node status
qudag status

# Restart node
qudag restart
```

### Peer Operations

```bash
# List connected peers
qudag peer list

# Add a peer
qudag peer add <multiaddr>

# Remove a peer
qudag peer remove <peer_id>

# Ban a peer
qudag peer ban <peer_id>

# Test connectivity
qudag network test
```

### Dark Addressing

```bash
# Register a .dark domain
qudag address register mydomain.dark

# Resolve a domain
qudag address resolve somedomain.dark

# Create temporary shadow address
qudag address shadow --ttl 3600

# Generate quantum fingerprint
qudag address fingerprint --data "Hello World"

# List your registered domains
qudag address list
```

### Exchange Operations

Manage rUv tokens and business plan payouts:

```bash
# Account Management
qudag exchange create-account --name "alice" --email alice@example.com
qudag exchange balance --account "alice"
qudag exchange list-accounts --format table

# Token Transfers
qudag exchange transfer --from "alice" --to "bob" --amount 1000
qudag exchange transfer --from "alice" --to "bob" --amount 5000 --memo "Payment for services"

# Fee Management
qudag exchange fee-info --examples
qudag exchange calculate-fee --account "alice" --amount 10000
qudag exchange verify-agent "alice" --proof-path ./proofs/alice_kyc.proof

# Immutable Deployment
qudag exchange deploy-immutable --key-path ./keys/quantum_master.key
qudag exchange immutable-status --format json
```

### Business Plan Management

Configure and manage automated payout distribution:

```bash
# Enable Business Plan Features
qudag exchange business-plan enable \
    --auto-distribution \
    --vault-management \
    --role-earnings \
    --bounty-rewards

# Check Status
qudag exchange business-plan status

# Configure Payouts
qudag exchange business-plan configure threshold 100
qudag exchange business-plan configure system-fee 0.001
qudag exchange business-plan configure single-agent --agent 0.95 --infrastructure 0.05
qudag exchange business-plan configure plugin-enhanced --agent 0.85 --plugin 0.10 --infrastructure 0.05

# Contributor Management
qudag exchange business-plan contributors register agent-001 agent-provider vault-001
qudag exchange business-plan contributors register plugin-002 plugin-creator vault-002 --custom-percentage 0.12
qudag exchange business-plan contributors list
qudag exchange business-plan contributors show agent-001
qudag exchange business-plan contributors update agent-001 --custom-percentage 0.90

# View Payout History
qudag exchange business-plan payouts --limit 50
qudag exchange business-plan payouts --contributor agent-001
```

### MCP Server Operations

Model Context Protocol server for AI agent integration:

```bash
# Start MCP Server
qudag mcp start --port 3000 --host 0.0.0.0
qudag mcp start --stdio  # For stdio transport

# Server Management
qudag mcp status
qudag mcp stop
qudag mcp logs --follow

# List Available Tools
qudag mcp tools list
qudag mcp resources list

# Configuration
qudag mcp config show
qudag mcp config set max_connections 100
```

### Quantum Cryptography

Generate and manage quantum-resistant keys:

```bash
# Key Generation
qudag key generate --algorithm ml-dsa    # ML-DSA-87 signatures
qudag key generate --algorithm ml-kem    # ML-KEM-768 encryption
qudag key generate --algorithm hqc       # HQC hybrid encryption
qudag key list
qudag key export <key-id> --format pem

# Signing Operations
qudag sign "message to sign" --key <key-id>
qudag sign --file document.pdf --key <key-id> --output document.sig
qudag verify <signature> "message" --key <public-key>

# Encryption Operations
qudag encrypt "secret message" --recipient <public-key>
qudag encrypt --file secret.doc --recipient <public-key> --output secret.enc
qudag decrypt <ciphertext> --key <private-key>
qudag hybrid-encrypt <data> --recipients key1,key2,key3
```

### DAG Visualization

Generate and analyze DAG structure:

```bash
# Generate DAG visualization
qudag dag --output dag_graph.dot
qudag dag --format png --output dag_graph.png
qudag dag --depth 10 --highlight-tips

# DAG Analysis
qudag dag analyze --metrics
qudag dag tips --format json
qudag dag ancestors <vertex-id>
qudag dag descendants <vertex-id>
```

### Password Vault

The QuDAG CLI includes a quantum-resistant password vault for secure credential management:

```bash
# Initialize a new vault
qudag vault init

# Generate secure passwords
qudag vault generate                    # Default 16-character password
qudag vault generate --length 32       # 32-character password
qudag vault generate --count 5         # Generate 5 passwords
qudag vault generate --no-symbols      # Alphanumeric only

# Add password entries
qudag vault add github --username myuser    # Prompts for password
qudag vault add email --generate           # Auto-generates password

# Retrieve passwords
qudag vault get github                 # Shows password details
qudag vault get github --clipboard     # Copies to clipboard

# List vault entries
qudag vault list                       # Simple list
qudag vault list --format json        # JSON output
qudag vault list --format tree        # Tree structure

# Update existing entries
qudag vault update github --password   # Change password
qudag vault update github --username   # Change username

# Remove entries
qudag vault remove github             # Remove entry
qudag vault remove github --force     # Skip confirmation

# Vault management
qudag vault export backup.qdag        # Export encrypted backup
qudag vault import backup.qdag        # Import from backup
qudag vault passwd                     # Change master password
qudag vault stats                      # Show vault statistics

# Configuration
qudag vault config show               # Show vault settings
qudag vault config set auto_lock 600  # Set auto-lock timeout
```

### Network Operations

```bash
# Show network statistics
qudag network stats

# Test peer connectivity
qudag network test

# Monitor network events
qudag network monitor --interval 5

# NAT Traversal
qudag nat status
qudag nat configure --upnp
qudag nat test

# Routing Configuration
qudag route onion --hops 3
qudag route direct --peer <peer-id>
qudag tunnel create --to <destination>
```

### Monitoring and Debugging

```bash
# Real-time monitoring
qudag monitor --metrics --log --interval 1

# Performance metrics
qudag metrics --format prometheus
qudag metrics export --output metrics.json

# Debug operations
qudag debug network --verbose
qudag debug consensus --verbose
qudag debug profile --duration 60

# Benchmarking
qudag benchmark --test crypto --duration 30
qudag benchmark --test network --parallel

# Health checks
qudag health --detailed
qudag health export --format json
```

### Configuration

```bash
# Show current configuration
qudag config show

# Set configuration value
qudag config set key value

# Import/Export configuration
qudag config import config.toml
qudag config export --output backup-config.toml

# Generate systemd service
qudag systemd --output /etc/systemd/system/
```

## Examples

### Setting Up Your First Node

```bash
# 1. Start your node
qudag start --port 8000

# 2. Register your identity
qudag address register mynode.dark

# 3. Connect to the network (use bootstrap peers)
qudag peer add /ip4/bootstrap.qudag.io/tcp/8000/p2p/12D3KooW...

# 4. Check status
qudag status
```

### Creating a Private Network

```bash
# Node 1
qudag start --port 8001
qudag address register node1.dark

# Node 2
qudag start --port 8002
qudag address register node2.dark
qudag peer add /ip4/127.0.0.1/tcp/8001/p2p/...

# Node 3
qudag start --port 8003
qudag address register node3.dark
qudag peer add /ip4/127.0.0.1/tcp/8001/p2p/...
qudag peer add /ip4/127.0.0.1/tcp/8002/p2p/...
```

### Dark Domain System

```bash
# Register domains for different services
qudag address register chat.dark
qudag address register files.dark
qudag address register api.dark

# Create temporary addresses for ephemeral communication
qudag address shadow --ttl 3600  # 1 hour
qudag address shadow --ttl 86400 # 24 hours

# Resolve any .dark domain to find peers
qudag address resolve chat.dark
qudag address resolve files.dark
```

### Exchange and Business Plan Workflow

```bash
# Initialize exchange with business plan
qudag exchange business-plan enable --auto-distribution --role-earnings

# Create accounts for participants
qudag exchange create-account --name "agent-provider" --email agent@example.com
qudag exchange create-account --name "plugin-creator" --email plugin@example.com
qudag exchange create-account --name "node-operator" --email node@example.com

# Register contributors
qudag exchange business-plan contributors register agent-001 agent-provider vault-001
qudag exchange business-plan contributors register plugin-002 plugin-creator vault-002 --custom-percentage 0.12
qudag exchange business-plan contributors register node-003 node-operator vault-003

# Configure payout thresholds
qudag exchange business-plan configure threshold 100
qudag exchange business-plan configure system-fee 0.001

# Process transactions with automatic fee distribution
qudag exchange transfer --from "user" --to "service" --amount 10000  # Fees auto-distributed

# Check payout history
qudag exchange business-plan payouts --limit 20
```

### MCP Server for AI Agents

```bash
# Start MCP server with full QuDAG integration
qudag mcp start --port 3000 --enable-vault --enable-exchange

# In another terminal, list available tools for agents
qudag mcp tools list

# Example output:
# - vault_create: Create secure vaults
# - vault_unlock: Access vault contents
# - exchange_transfer: Send rUv tokens
# - dag_query: Query DAG structure
# - network_peer_info: Get peer information

# Check server status
qudag mcp status
```

### Password Vault Setup

```bash
# Initialize your vault
qudag vault init

# Generate and store service passwords
qudag vault add github --username myuser --generate
qudag vault add docker --username dockerhub_user --generate
qudag vault add email --username user@example.com

# Create organized structure
qudag vault add work/aws --username admin --generate
qudag vault add work/kubernetes --generate
qudag vault add personal/social/twitter --username @myhandle

# Backup your vault
qudag vault export ~/secure-backup.qdag

# Daily usage
qudag vault get github --clipboard  # Copy to clipboard
qudag vault list work/             # List work passwords
qudag vault generate --length 32   # Generate new password
```

## Configuration File

QuDAG CLI uses a configuration file at `~/.qudag/config.toml`:

```toml
[node]
port = 8000
rpc_port = 9090
data_dir = "~/.qudag/data"
log_level = "info"

[network]
max_peers = 50
bootstrap_peers = [
    "/ip4/bootstrap1.qudag.io/tcp/8000/p2p/12D3KooW...",
    "/ip4/bootstrap2.qudag.io/tcp/8000/p2p/12D3KooW..."
]

[dark_addressing]
enable = true
ttl_default = 3600

[security]
enable_encryption = true
quantum_resistant = true

[vault]
path = "~/.qudag/vault.qdag"
auto_lock = 300  # seconds
clipboard_timeout = 30  # seconds
kdf_iterations = 3
kdf_memory = 65536  # KB

[exchange]
enable = true
network_name = "qudag-mainnet"
chain_id = 1
immutable_deployment = false

[exchange.fee_model]
f_min = 0.001  # 0.1%
f_max = 0.01   # 1.0%
f_min_verified = 0.0025  # 0.25%
f_max_verified = 0.005   # 0.5%
time_constant_days = 30
usage_threshold = 10000

[exchange.business_plan]
enabled = true
auto_distribution = true
vault_management = true
role_earnings = true
bounty_rewards = true
min_payout_threshold = 100
system_fee_percentage = 0.001

[mcp]
enable = true
port = 3000
host = "127.0.0.1"
transport = "http"  # http, stdio, or websocket
enable_vault_tools = true
enable_exchange_tools = true
enable_dag_tools = true
enable_network_tools = true

[monitoring]
enable_metrics = true
metrics_port = 9091
enable_tracing = true
trace_export = "jaeger"  # jaeger, otlp, or none
```

## Output Formats

Many commands support different output formats:

```bash
# JSON output
qudag status --output json

# Table output (default)
qudag peer list --output table

# Raw output for scripting
qudag peer list --output raw
```

## Logging

```bash
# View logs
qudag logs

# Follow logs in real-time
qudag logs --follow

# Filter by level
qudag logs --level error

# Save logs to file
qudag logs --output /var/log/qudag.log
```

## Systemd Integration

Generate systemd service files:

```bash
# Generate service file
qudag systemd --output /etc/systemd/system/

# Enable and start
sudo systemctl enable qudag
sudo systemctl start qudag
sudo systemctl status qudag
```

## Environment Variables

- `QUDAG_CONFIG` - Path to configuration file
- `QUDAG_DATA_DIR` - Data directory override
- `QUDAG_LOG_LEVEL` - Log level (trace, debug, info, warn, error)
- `QUDAG_PORT` - Default port override
- `QUDAG_RPC_PORT` - RPC port override
- `QUDAG_EXCHANGE_ENABLED` - Enable/disable exchange (true/false)
- `QUDAG_MCP_PORT` - MCP server port override
- `QUDAG_MCP_TRANSPORT` - MCP transport (http, stdio, websocket)
- `QUDAG_VAULT_PATH` - Vault file path override
- `QUDAG_NETWORK_NAME` - Network name for exchange
- `QUDAG_CHAIN_ID` - Chain ID for exchange

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Network error
- `4` - Permission error
- `5` - Not found error

## Shell Completion

Generate shell completion scripts:

```bash
# Bash
qudag completions bash > /etc/bash_completion.d/qudag

# Zsh
qudag completions zsh > ~/.zsh/completions/_qudag

# Fish
qudag completions fish > ~/.config/fish/completions/qudag.fish
```

## Security Considerations

- All communication is quantum-resistant encrypted
- Private keys are stored securely in `~/.qudag/keys/`
- Configuration supports file permissions verification
- Network traffic uses onion routing for anonymity

## Troubleshooting

### Common Issues

**Node won't start**
```bash
# Check if port is in use
netstat -ln | grep :8000

# Check logs
qudag logs --level error
```

**Can't connect to peers**
```bash
# Test network connectivity
qudag network test

# Check firewall settings
sudo ufw status
```

**Permission errors**
```bash
# Check data directory permissions
ls -la ~/.qudag/

# Fix permissions
chmod 700 ~/.qudag/
```

**Exchange not working**
```bash
# Check if exchange is enabled
qudag exchange status

# Verify configuration
qudag config show | grep exchange

# Check vault connectivity
qudag vault status
```

**MCP server issues**
```bash
# Check if port is available
lsof -i :3000

# Test MCP server
qudag mcp status

# View MCP logs
qudag mcp logs --level debug
```

**Business plan payout issues**
```bash
# Check business plan status
qudag exchange business-plan status

# Verify contributor registration
qudag exchange business-plan contributors list

# Check minimum thresholds
qudag exchange business-plan configure show
```

## Documentation

- [QuDAG CLI Documentation](https://docs.rs/qudag-cli)
- [Exchange Core Documentation](https://docs.rs/qudag-exchange-core)
- [MCP Server Documentation](https://docs.rs/qudag-mcp)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)
- [User Guide](https://github.com/ruvnet/QuDAG/blob/main/docs/cli/README.md)
- [Exchange Business Plan](https://github.com/ruvnet/QuDAG/blob/main/qudag-exchange/docs/business-plan.md)

## License

Licensed under either MIT or Apache-2.0 at your option.