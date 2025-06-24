# QuDAG Peer Management

This document describes the peer management functionality implemented in the QuDAG CLI.

## Overview

The QuDAG CLI now includes comprehensive peer management capabilities through the `PeerManager` module. This allows for:

- Persistent peer storage
- Connection management with the NetworkManager
- Peer metadata and reputation tracking
- Import/export functionality
- Connectivity testing
- Blacklist/whitelist management

## Architecture

### Components

1. **PeerManager** (`tools/cli/src/peer_manager.rs`)
   - Manages peer persistence and metadata
   - Interfaces with NetworkManager for actual connections
   - Stores peer data in `~/.qudag/peers.json`

2. **CommandRouter** (`tools/cli/src/commands.rs`)
   - Routes CLI commands to appropriate handlers
   - Provides both PeerManager and RPC fallback methods
   - Handles user interaction and progress reporting

3. **NetworkManager** (`core/network/src/lib.rs`)
   - Handles actual P2P connections
   - Manages reputation and blacklisting
   - Provides network statistics

## CLI Commands

### Basic Peer Operations

```bash
# List all known peers
qudag peer list

# Add a new peer
qudag peer add 192.168.1.100:8000

# Add a peer with nickname
qudag peer add 192.168.1.100:8000 --nickname "Alice's Node"

# Remove a peer
qudag peer remove <peer-id>

# Remove with confirmation bypass
qudag peer remove <peer-id> --force
```

### Advanced Operations

```bash
# Get detailed peer information
qudag peer stats <peer-id>

# Ban a peer (blacklist)
qudag peer ban <peer-id>

# Unban a peer
qudag peer unban <address>

# Test connectivity to all peers
qudag peer test

# Import peers from file
qudag peer import peers.json
qudag peer import peers.json --merge  # Merge with existing

# Export peers to file
qudag peer export --output backup.json
```

## Data Storage

Peer data is stored in JSON format at `~/.qudag/peers.json`:

```json
[
  {
    "id": "12D3KooWExample123",
    "address": "192.168.1.100:8000",
    "nickname": "Alice's Node",
    "trust_level": 85,
    "first_seen": 1700000000,
    "last_seen": 1700001000,
    "total_messages": 1523,
    "success_rate": 0.98,
    "avg_latency_ms": 23.5,
    "tags": ["trusted", "fast"],
    "persistent": true
  }
]
```

## Features

### Persistence
- Peers are automatically saved at regular intervals (default: 5 minutes)
- Manual save on important operations (add, remove, ban)
- Survives node restarts

### Reputation System
- Trust levels from 0-100
- Automatic reputation updates based on behavior
- Auto-blacklisting for peers below -50 reputation

### Connectivity Testing
- Test all peers with progress indicator
- Measure latency
- Update success rates

### Import/Export
- JSON format for easy sharing
- Merge or replace modes
- Tag-based filtering for exports

## Implementation Details

### Connection Flow
1. User adds peer via CLI
2. CommandRouter validates input
3. PeerManager stores metadata
4. NetworkManager establishes connection
5. Peer data saved to disk

### Error Handling
- Graceful fallback to RPC methods
- Comprehensive error messages
- Connection timeout handling

### Performance
- Async operations throughout
- Connection pooling in NetworkManager
- Lazy initialization of PeerManager

## Usage Examples

See `examples/peer_management_example.rs` for a complete demonstration of the peer management API.

## Testing

Run the peer management tests:

```bash
cargo test -p qudag-cli peer_manager_tests
```

## Future Enhancements

1. **Peer Discovery**
   - DHT integration
   - Bootstrap node support
   - Automatic peer exchange

2. **Advanced Filtering**
   - Filter by trust level
   - Filter by latency
   - Geographic filtering

3. **Peer Groups**
   - Create named groups
   - Bulk operations on groups
   - Group-based policies

4. **Monitoring**
   - Real-time peer status
   - Connection quality metrics
   - Alert system for issues