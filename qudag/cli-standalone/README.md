# QuDAG CLI

Command-line interface for the QuDAG quantum-resistant distributed protocol.

## Installation

```bash
cargo install qudag
```

> **Note:** The CLI binary is named `qudag` after installation.

## Quick Start

```bash
# Start a QuDAG node
qudag start --port 8000

# Create your own darknet domain
qudag address register mynode.dark

# Connect to peers
qudag peer add /ip4/192.168.1.100/tcp/8000/p2p/12D3KooW...

# Check node status
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
qudag logs --follow
```

### Configuration

```bash
# Show current configuration
qudag config show

# Set configuration value
qudag config set key value

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

## Documentation

- [API Documentation](https://docs.rs/qudag)
- [QuDAG Project](https://github.com/ruvnet/QuDAG)
- [User Guide](https://github.com/ruvnet/QuDAG/blob/main/docs/cli/README.md)

## License

Licensed under either MIT or Apache-2.0 at your option.