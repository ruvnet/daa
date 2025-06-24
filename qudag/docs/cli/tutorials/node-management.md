# Node Management Tutorial

This tutorial covers common operations for managing QuDAG nodes using the CLI.

## Basic Node Operations

### Starting a Node

```bash
# Start a node with default configuration
qudag node start

# Start a node with custom config file
qudag node start --config /path/to/config.toml

# Start a node with specific identity key
qudag node start --identity-key /path/to/key.pem
```

### Stopping a Node

```bash
# Stop a running node gracefully
qudag node stop

# Force stop a node that's not responding
qudag node stop --force
```

### Node Status

```bash
# Check node status and health
qudag node status

# Get detailed node metrics
qudag node status --verbose
```

## Configuration Management

### Managing Node Identity

```bash
# Generate new node identity
qudag identity new

# Import existing identity
qudag identity import /path/to/key.pem

# Export current identity
qudag identity export /path/to/backup.pem
```

### Updating Node Configuration

```bash
# Edit node configuration
qudag config edit

# Validate configuration file
qudag config validate /path/to/config.toml

# Apply configuration changes without restart
qudag config reload
```

## Node Maintenance

### Memory Management

```bash
# Clear node memory cache
qudag node clear-cache

# View memory usage statistics
qudag node stats memory
```

### Database Operations

```bash
# Compact the node database
qudag db compact

# Verify database integrity
qudag db verify

# Create database backup
qudag db backup /path/to/backup
```

### Diagnostic Commands

```bash
# Run node diagnostics
qudag node diagnose

# Check network connectivity
qudag node connectivity-test

# View node logs
qudag node logs
```

## Advanced Operations

### Resource Limits

```bash
# Set maximum memory limit
qudag node set-limit memory 8G

# Set maximum connections
qudag node set-limit connections 1000

# View current resource limits
qudag node get-limits
```

### Performance Tuning

```bash
# Enable performance mode
qudag node tune performance

# Optimize for low latency
qudag node tune latency

# Reset tuning to defaults
qudag node tune reset
```

### Hot Updates

```bash
# Update node version without downtime
qudag node update

# Rollback to previous version
qudag node rollback

# View update history
qudag node updates list
```

## Tips and Best Practices

1. Always use the `--dry-run` flag first when making configuration changes
2. Keep regular backups of node identity and configuration
3. Monitor resource usage and adjust limits accordingly
4. Use diagnostic commands for troubleshooting
5. Keep the node software updated to the latest stable version