# QuDAG Peer Management Implementation Plan

## Overview

This document outlines the implementation plan for peer management functionality in the QuDAG protocol. The peer management system handles peer discovery, connection management, reputation tracking, and provides CLI commands for operators to manage their node's peer relationships.

## 1. Peer Discovery and Management

### 1.1 Peer Discovery Mechanisms

#### Bootstrap Nodes
- Maintain a list of well-known bootstrap nodes
- Configuration in `~/.qudag/config.toml`
- Fallback to hardcoded bootstrap addresses
- Support for custom bootstrap lists

#### Peer Exchange Protocol (PEX)
- Exchange peer lists with connected nodes
- Filter peers based on reputation and availability
- Limit peer exchange frequency to prevent spam
- Validate peer addresses before adding

#### DNS-based Discovery
- Support DNS seeds for peer discovery
- TXT records containing peer addresses
- Regular DNS refresh intervals
- Fallback to other discovery methods

### 1.2 Peer Connection Management

#### Connection Pool
```rust
pub struct ConnectionPool {
    max_connections: usize,
    min_connections: usize,
    connections: HashMap<PeerId, Connection>,
    pending_connections: HashSet<SocketAddr>,
    connection_timeout: Duration,
}
```

#### Connection Lifecycle
1. **Discovery**: Find new peer addresses
2. **Validation**: Check address format and blacklist
3. **Connection**: Establish TCP/QUIC connection
4. **Handshake**: Exchange protocol version and capabilities
5. **Authentication**: Verify peer identity with quantum-resistant signatures
6. **Active**: Normal message exchange
7. **Disconnection**: Graceful shutdown or timeout

### 1.3 Peer State Machine

```
┌─────────────┐     ┌──────────────┐     ┌────────────┐
│ Discovered  │────▶│  Connecting  │────▶│ Handshaking│
└─────────────┘     └──────────────┘     └────────────┘
                           │                     │
                           ▼                     ▼
                    ┌──────────────┐     ┌────────────┐
                    │   Failed     │     │ Connected  │
                    └──────────────┘     └────────────┘
                                               │
                                               ▼
                                        ┌────────────┐
                                        │Disconnected│
                                        └────────────┘
```

## 2. Peer List Command Implementation

### 2.1 Command Structure
```bash
qudag peer list [--format <json|table>] [--filter <connected|all|banned>]
```

### 2.2 Peer Information Display

#### Table Format (Default)
```
Peer ID              Address              Status      Version  Latency  Messages  Reputation
──────────────────────────────────────────────────────────────────────────────────────────
a3f2b1c4...         192.168.1.100:8000   Connected   1.0.0    15ms     1,234     85/100
b4c5d6e7...         10.0.0.50:8000       Connected   1.0.0    25ms     567       92/100
c5d6e7f8...         172.16.0.10:8000     Connecting  -        -        0         50/100
```

#### JSON Format
```json
{
  "peers": [
    {
      "id": "a3f2b1c4...",
      "address": "192.168.1.100:8000",
      "status": "connected",
      "version": "1.0.0",
      "latency_ms": 15,
      "messages_exchanged": 1234,
      "reputation_score": 85,
      "connected_since": "2024-01-17T10:30:00Z",
      "last_activity": "2024-01-17T14:45:30Z",
      "capabilities": ["routing", "consensus", "storage"]
    }
  ],
  "summary": {
    "total_peers": 3,
    "connected": 2,
    "connecting": 1,
    "average_reputation": 75.67
  }
}
```

### 2.3 Extended Peer Information

```rust
pub struct PeerInfo {
    pub id: PeerId,
    pub address: SocketAddr,
    pub status: PeerStatus,
    pub version: Version,
    pub connected_since: Option<SystemTime>,
    pub last_activity: SystemTime,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latency: Option<Duration>,
    pub reputation: ReputationScore,
    pub capabilities: Vec<Capability>,
    pub user_agent: String,
}
```

## 3. Add Peer Command Implementation

### 3.1 Command Structure
```bash
qudag peer add <ADDRESS> [--priority <high|normal|low>]
```

### 3.2 Address Validation

#### Supported Formats
- IPv4: `192.168.1.100:8000`
- IPv6: `[2001:db8::1]:8000`
- Domain: `node1.qudag.network:8000`
- Onion: `3g2upl4pq6kufc4m.onion:8000`
- Dark Address: `quantum://a3f2b1c4...@node1.qudag.network`

#### Validation Steps
1. Parse address format
2. Resolve DNS if needed
3. Check against blacklist
4. Verify port is valid (1-65535)
5. Check if already connected

### 3.3 Connection and Handshake Process

```rust
pub async fn add_peer(address: String) -> Result<PeerId, PeerError> {
    // 1. Validate address
    let socket_addr = validate_and_resolve_address(&address)?;
    
    // 2. Check blacklist
    if is_blacklisted(&socket_addr) {
        return Err(PeerError::Blacklisted);
    }
    
    // 3. Check connection limit
    if connection_pool.is_full() {
        return Err(PeerError::ConnectionLimitReached);
    }
    
    // 4. Initiate connection
    let connection = connect_with_timeout(socket_addr, CONNECTION_TIMEOUT).await?;
    
    // 5. Perform handshake
    let peer_info = perform_handshake(&mut connection).await?;
    
    // 6. Verify protocol compatibility
    verify_protocol_version(&peer_info)?;
    
    // 7. Add to connection pool
    let peer_id = connection_pool.add_connection(connection, peer_info)?;
    
    // 8. Start message handler
    spawn_peer_handler(peer_id);
    
    Ok(peer_id)
}
```

### 3.4 Handshake Protocol

```rust
pub struct HandshakeMessage {
    pub version: Version,
    pub node_id: NodeId,
    pub capabilities: Vec<Capability>,
    pub timestamp: u64,
    pub nonce: [u8; 32],
    pub signature: Signature,
}

pub struct HandshakeResponse {
    pub accepted: bool,
    pub node_id: NodeId,
    pub capabilities: Vec<Capability>,
    pub timestamp: u64,
    pub signature: Signature,
}
```

## 4. Remove Peer Command Implementation

### 4.1 Command Structure
```bash
qudag peer remove <ADDRESS|PEER_ID> [--ban]
```

### 4.2 Graceful Disconnection Process

```rust
pub async fn remove_peer(identifier: String, ban: bool) -> Result<(), PeerError> {
    // 1. Find peer by address or ID
    let peer_id = resolve_peer_identifier(&identifier)?;
    
    // 2. Get connection
    let connection = connection_pool.get_connection(&peer_id)?;
    
    // 3. Send disconnect message
    send_disconnect_message(&connection, DisconnectReason::UserRequested).await?;
    
    // 4. Wait for acknowledgment (with timeout)
    wait_for_disconnect_ack(&connection, DISCONNECT_TIMEOUT).await.ok();
    
    // 5. Close connection
    connection.close().await?;
    
    // 6. Remove from connection pool
    connection_pool.remove_connection(&peer_id);
    
    // 7. Optionally ban the peer
    if ban {
        blacklist.add_peer(peer_id, BanReason::UserRequested);
    }
    
    // 8. Update peer database
    peer_db.mark_disconnected(&peer_id)?;
    
    Ok(())
}
```

### 4.3 Disconnect Reasons

```rust
pub enum DisconnectReason {
    UserRequested,
    ProtocolViolation,
    InactivityTimeout,
    ConnectionLimitReached,
    Shutdown,
    NetworkError,
    InvalidMessage,
    Banned,
}
```

## 5. Peer Reputation and Scoring

### 5.1 Reputation Components

```rust
pub struct ReputationScore {
    pub total_score: u32,          // 0-100
    pub uptime_score: u32,         // Connection stability
    pub latency_score: u32,        // Network performance
    pub validity_score: u32,       // Message validity
    pub consensus_score: u32,      // Consensus participation
    pub relay_score: u32,          // Message relay behavior
}
```

### 5.2 Scoring Algorithm

```rust
pub fn calculate_reputation(peer: &PeerInfo) -> ReputationScore {
    let uptime_score = calculate_uptime_score(
        peer.connected_since,
        peer.disconnect_count,
    );
    
    let latency_score = calculate_latency_score(
        peer.average_latency,
        peer.latency_variance,
    );
    
    let validity_score = calculate_validity_score(
        peer.valid_messages,
        peer.invalid_messages,
    );
    
    let consensus_score = calculate_consensus_score(
        peer.consensus_participations,
        peer.consensus_violations,
    );
    
    let relay_score = calculate_relay_score(
        peer.messages_relayed,
        peer.relay_failures,
    );
    
    let total_score = (
        uptime_score * 0.2 +
        latency_score * 0.2 +
        validity_score * 0.3 +
        consensus_score * 0.2 +
        relay_score * 0.1
    ).round() as u32;
    
    ReputationScore {
        total_score,
        uptime_score,
        latency_score,
        validity_score,
        consensus_score,
        relay_score,
    }
}
```

### 5.3 Reputation-based Actions

```rust
pub struct ReputationThresholds {
    pub min_relay: u32,           // Minimum score to relay messages
    pub min_consensus: u32,       // Minimum score for consensus participation
    pub auto_disconnect: u32,     // Disconnect if below this score
    pub auto_ban: u32,           // Ban if below this score
}

impl Default for ReputationThresholds {
    fn default() -> Self {
        Self {
            min_relay: 30,
            min_consensus: 50,
            auto_disconnect: 20,
            auto_ban: 10,
        }
    }
}
```

## 6. Test Scenarios

### 6.1 Unit Tests

#### Peer Discovery Tests
```rust
#[cfg(test)]
mod discovery_tests {
    #[test]
    fn test_bootstrap_node_loading() {
        // Test loading bootstrap nodes from config
    }
    
    #[test]
    fn test_dns_discovery() {
        // Test DNS-based peer discovery
    }
    
    #[test]
    fn test_peer_exchange_protocol() {
        // Test PEX message handling
    }
}
```

#### Connection Management Tests
```rust
#[cfg(test)]
mod connection_tests {
    #[tokio::test]
    async fn test_connection_lifecycle() {
        // Test full connection lifecycle
    }
    
    #[tokio::test]
    async fn test_connection_timeout() {
        // Test connection timeout handling
    }
    
    #[tokio::test]
    async fn test_max_connections() {
        // Test connection limit enforcement
    }
}
```

#### Reputation Tests
```rust
#[cfg(test)]
mod reputation_tests {
    #[test]
    fn test_reputation_calculation() {
        // Test reputation score calculation
    }
    
    #[test]
    fn test_reputation_decay() {
        // Test reputation decay over time
    }
    
    #[test]
    fn test_reputation_thresholds() {
        // Test reputation-based actions
    }
}
```

### 6.2 Integration Tests

#### CLI Command Tests
```rust
#[tokio::test]
async fn test_peer_list_command() {
    let node = start_test_node().await;
    
    // Add some test peers
    node.add_peer("127.0.0.1:8001").await.unwrap();
    node.add_peer("127.0.0.1:8002").await.unwrap();
    
    // Test list command
    let output = run_cli_command(&["peer", "list"]).await;
    assert!(output.contains("127.0.0.1:8001"));
    assert!(output.contains("127.0.0.1:8002"));
    
    // Test JSON format
    let json_output = run_cli_command(&["peer", "list", "--format", "json"]).await;
    let parsed: Value = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed["summary"]["total_peers"], 2);
}

#[tokio::test]
async fn test_peer_add_remove() {
    let node = start_test_node().await;
    
    // Test add peer
    let output = run_cli_command(&["peer", "add", "127.0.0.1:8003"]).await;
    assert!(output.contains("successfully"));
    
    // Verify peer was added
    let peers = node.get_peers().await;
    assert_eq!(peers.len(), 1);
    
    // Test remove peer
    let output = run_cli_command(&["peer", "remove", "127.0.0.1:8003"]).await;
    assert!(output.contains("removed"));
    
    // Verify peer was removed
    let peers = node.get_peers().await;
    assert_eq!(peers.len(), 0);
}
```

### 6.3 Stress Tests

```rust
#[tokio::test]
async fn test_concurrent_peer_operations() {
    let node = start_test_node().await;
    let mut handles = vec![];
    
    // Spawn 100 concurrent peer additions
    for i in 0..100 {
        let addr = format!("127.0.0.1:{}", 9000 + i);
        let handle = tokio::spawn(async move {
            run_cli_command(&["peer", "add", &addr]).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify connection limit is respected
    let peers = node.get_peers().await;
    assert!(peers.len() <= MAX_CONNECTIONS);
}
```

### 6.4 Security Tests

```rust
#[tokio::test]
async fn test_malicious_peer_handling() {
    let node = start_test_node().await;
    
    // Test invalid handshake
    let malicious_peer = start_malicious_peer().await;
    let result = node.add_peer(&malicious_peer.address).await;
    assert!(matches!(result, Err(PeerError::HandshakeFailed)));
    
    // Test banned peer reconnection
    node.ban_peer(&malicious_peer.id).await;
    let result = node.add_peer(&malicious_peer.address).await;
    assert!(matches!(result, Err(PeerError::Blacklisted)));
}

#[tokio::test]
async fn test_reputation_based_disconnect() {
    let node = start_test_node().await;
    let peer = node.add_peer("127.0.0.1:8004").await.unwrap();
    
    // Simulate bad behavior
    for _ in 0..10 {
        node.report_invalid_message(&peer).await;
    }
    
    // Wait for reputation update
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Verify peer was disconnected
    let peers = node.get_peers().await;
    assert!(!peers.iter().any(|p| p.id == peer));
}
```

## 7. Implementation Timeline

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Implement `PeerManager` trait and basic structures
- [ ] Create connection pool management
- [ ] Implement peer state machine
- [ ] Add peer database storage

### Phase 2: Discovery Mechanisms (Week 3)
- [ ] Implement bootstrap node loading
- [ ] Add DNS-based discovery
- [ ] Implement peer exchange protocol
- [ ] Create peer validation logic

### Phase 3: CLI Commands (Week 4)
- [ ] Implement `peer list` command with formatting
- [ ] Implement `peer add` with validation
- [ ] Implement `peer remove` with graceful disconnect
- [ ] Add command error handling

### Phase 4: Reputation System (Week 5)
- [ ] Implement reputation scoring algorithm
- [ ] Add reputation-based actions
- [ ] Create reputation persistence
- [ ] Implement reputation decay

### Phase 5: Testing and Integration (Week 6)
- [ ] Write comprehensive unit tests
- [ ] Create integration test suite
- [ ] Perform security testing
- [ ] Conduct stress testing

## 8. Configuration Options

```toml
[network.peers]
# Maximum number of peer connections
max_connections = 50

# Minimum number of peer connections to maintain
min_connections = 5

# Connection timeout in seconds
connection_timeout = 10

# Handshake timeout in seconds
handshake_timeout = 5

# Peer discovery interval in seconds
discovery_interval = 300

# Bootstrap nodes
bootstrap_nodes = [
    "bootstrap1.qudag.network:8000",
    "bootstrap2.qudag.network:8000",
    "bootstrap3.qudag.network:8000"
]

# DNS seeds for peer discovery
dns_seeds = [
    "seed.qudag.network",
    "nodes.qudag.network"
]

[network.reputation]
# Minimum reputation for message relay
min_relay_reputation = 30

# Minimum reputation for consensus participation
min_consensus_reputation = 50

# Auto-disconnect threshold
auto_disconnect_threshold = 20

# Auto-ban threshold
auto_ban_threshold = 10

# Reputation decay rate (per day)
decay_rate = 0.1
```

## 9. Monitoring and Metrics

### Peer Metrics
```rust
pub struct PeerMetrics {
    pub total_peers: Gauge,
    pub connected_peers: Gauge,
    pub peer_connections: Counter,
    pub peer_disconnections: Counter,
    pub handshake_failures: Counter,
    pub average_peer_latency: Histogram,
    pub peer_reputation_distribution: Histogram,
    pub messages_per_peer: Histogram,
}
```

### Monitoring Dashboard
- Real-time peer count and status
- Geographic distribution of peers
- Reputation score distribution
- Connection success/failure rates
- Average peer latency trends
- Bandwidth usage per peer

## 10. Future Enhancements

### Advanced Features
1. **Peer Clustering**: Group peers by geographic location or network topology
2. **Smart Peer Selection**: ML-based peer selection for optimal network performance
3. **Peer Analytics**: Detailed analytics on peer behavior and performance
4. **Automated Peer Management**: AI-driven peer connection optimization
5. **Cross-chain Peer Discovery**: Integration with other blockchain networks

### Security Enhancements
1. **Sybil Attack Detection**: Advanced algorithms to detect and prevent Sybil attacks
2. **Peer Fingerprinting**: Identify peers across reconnections
3. **Traffic Analysis Protection**: Enhanced privacy for peer communications
4. **Zero-Knowledge Peer Proofs**: Prove peer properties without revealing identity

### Performance Optimizations
1. **Connection Multiplexing**: Multiple logical connections over single transport
2. **Peer Load Balancing**: Distribute load across peers based on capacity
3. **Predictive Connection Management**: Anticipate and pre-connect to peers
4. **Bandwidth Optimization**: Adaptive message compression and batching