# QuDAG Node Management

The QuDAG CLI provides comprehensive node process management functionality for starting, stopping, and monitoring QuDAG nodes.

## Features

### Process Management
- **Background/Foreground Modes**: Run nodes as background daemons or in the foreground
- **Graceful Shutdown**: SIGTERM handling with configurable timeout
- **Process Monitoring**: PID file management and health checks
- **Automatic Restart**: Built-in restart functionality

### Configuration Management
- **TOML Configuration**: Node settings stored in `~/.qudag/config.toml`
- **Environment Overrides**: Support for environment variable configuration
- **Hot Reload**: Configuration can be updated without restart

### Logging
- **Structured Logging**: Comprehensive log output to `~/.qudag/qudag.log`
- **Log Rotation**: Automatic rotation based on size with configurable retention
- **Real-time Tailing**: View logs in real-time with `qudag logs -f`

### System Integration
- **Systemd Support**: Generate systemd service files for production deployments
- **Signal Handling**: Proper handling of SIGTERM, SIGINT, and other signals

## Commands

### Start Node

Start a QuDAG node with various options:

```bash
# Start in foreground (default)
qudag start

# Start in background/daemon mode
qudag start --background

# Start with custom port and data directory
qudag start --port 9000 --data-dir /opt/qudag/data

# Start with initial peers
qudag start --peer 192.168.1.100:8000 --peer node2.example.com:8000

# Start with custom log level
qudag start --log-level debug
```

### Stop Node

Stop a running node:

```bash
# Graceful shutdown (default)
qudag stop

# Force kill
qudag stop --force
```

### Restart Node

Restart a running node:

```bash
# Graceful restart
qudag restart

# Force restart
qudag restart --force
```

### Node Status

Check node status:

```bash
# Show node status
qudag status
```

Output includes:
- Running state and PID
- Port and data directory
- Uptime information
- Network statistics (if RPC is available)

### View Logs

View node logs:

```bash
# Show last 50 lines (default)
qudag logs

# Show last 100 lines
qudag logs -n 100

# Follow logs in real-time
qudag logs -f

# Follow last 200 lines
qudag logs -n 200 -f
```

### Systemd Integration

Generate systemd service file:

```bash
# Print to stdout
qudag systemd

# Save to file
qudag systemd --output /etc/systemd/system/qudag.service
```

Then install the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable qudag
sudo systemctl start qudag
```

## File Locations

Default file locations (can be customized):

- **Base Directory**: `~/.qudag/`
- **Configuration**: `~/.qudag/config.toml`
- **PID File**: `~/.qudag/qudag.pid`
- **Log File**: `~/.qudag/qudag.log`
- **Data Directory**: `~/.qudag/data/`
- **Peer Database**: `~/.qudag/peers.json`

## Configuration File

Example `config.toml`:

```toml
# Data directory for blockchain data
data_dir = "/home/user/.qudag/data"

# Network port
network_port = 8000

# Maximum number of peer connections
max_peers = 50

# Initial peers to connect to on startup
initial_peers = [
    "192.168.1.100:8000",
    "node2.example.com:8000"
]
```

## Environment Variables

Configuration can be overridden with environment variables:

- `QUDAG_PORT`: Network port
- `QUDAG_DATA_DIR`: Data directory path
- `QUDAG_LOG_LEVEL`: Log level (trace, debug, info, warn, error)
- `QUDAG_CONFIG`: Path to config file

## Process Management Details

### PID File Management

The node manager maintains a PID file at `~/.qudag/qudag.pid` containing:
- Process ID
- Start timestamp
- Port and configuration details

This file is automatically cleaned up on shutdown and checked on startup to prevent multiple instances.

### Health Monitoring

The node manager performs periodic health checks:
- Process liveness checks every 60 seconds
- Automatic cleanup of stale PID files
- Detection of crashed processes

### Log Rotation

Logs are automatically rotated when they reach 100MB (configurable):
- Up to 5 historical log files are kept
- Old logs are named `qudag.log.1`, `qudag.log.2`, etc.
- Oldest logs are automatically deleted

## Error Handling

Common errors and solutions:

### Node Already Running
```
Error: Node is already running
```
Solution: Stop the existing node with `qudag stop` or check the PID file

### Port Already in Use
```
Error: Address already in use
```
Solution: Use a different port with `--port` or stop the process using the port

### Permission Denied
```
Error: Permission denied
```
Solution: Ensure write permissions to the base directory or run with appropriate permissions

## Best Practices

1. **Production Deployment**: Use systemd for production deployments
2. **Monitoring**: Set up external monitoring for the node process
3. **Backups**: Regularly backup the data directory
4. **Updates**: Stop the node before updating the binary
5. **Resource Limits**: Configure appropriate system resource limits

## Troubleshooting

### Check if node is running:
```bash
qudag status
ps aux | grep qudag
cat ~/.qudag/qudag.pid
```

### View recent logs:
```bash
qudag logs -n 100
tail -f ~/.qudag/qudag.log
```

### Clean up stale PID file:
```bash
rm ~/.qudag/qudag.pid
```

### Debug startup issues:
```bash
qudag start --log-level debug
```