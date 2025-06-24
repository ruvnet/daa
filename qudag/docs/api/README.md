# QuDAG API Documentation

Complete reference for the QuDAG JSON-RPC API and P2P protocol interfaces.

## JSON-RPC API

### Connection
- **Default Endpoint:** `http://localhost:9090`
- **Protocol:** JSON-RPC 2.0
- **Transport:** HTTP, TCP, or Unix sockets

### Node Management Methods

#### `get_status`
Get comprehensive node status.

**Request:**
```json
{
  "id": 1,
  "method": "get_status",
  "params": {}
}
```

**Response:**
```json
{
  "id": 1,
  "result": {
    "node_id": "12D3KooW...",
    "status": "running",
    "uptime": 3600,
    "network": {
      "peer_count": 5,
      "listening_addrs": ["/ip4/0.0.0.0/tcp/8000"]
    },
    "dag": {
      "vertex_count": 1234,
      "consensus_state": "active"
    }
  }
}
```

#### `list_peers`
List all connected peers.

**Request:**
```json
{
  "id": 2,
  "method": "list_peers",
  "params": {}
}
```

#### `add_peer`
Connect to a new peer.

**Request:**
```json
{
  "id": 3,
  "method": "add_peer",
  "params": {
    "multiaddr": "/ip4/192.168.1.100/tcp/8000"
  }
}
```

### Network Methods

#### `get_network_stats`
Get network performance statistics.

**Request:**
```json
{
  "id": 4,
  "method": "get_network_stats",
  "params": {}
}
```

## P2P Protocol API

### Protocol Identifiers
- `/qudag/req/1.0.0` - Request/response messaging
- `/kad/1.0.0` - Kademlia DHT
- `/gossipsub/1.1.0` - Pub/sub messaging
- `/dark-resolve/1.0.0` - Address resolution

### Security
All connections use ML-KEM-768 encryption and ML-DSA signatures.

For complete API specification, see the full documentation.