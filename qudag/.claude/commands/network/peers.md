# QuDAG Peer Management Commands

Commands for managing peers in the QuDAG distributed network with libp2p integration.

## Add Peers

### Add peer by multiaddr
```bash
./claude-flow qudag peer add "/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWBmwkafWE2fqfzS96VoTYLKJV2s2xw7Q3vPCVHp3sHqPa"
```

### Add multiple peers from bootstrap list
```bash
./claude-flow qudag peer bootstrap --file ./config/bootstrap_peers.json
```

### Add peer with dark addressing
```bash
./claude-flow qudag peer add-dark "node1.dark:9001" --verify
```

## Remove Peers

### Remove specific peer
```bash
./claude-flow qudag peer remove "12D3KooWBmwkafWE2fqfzS96VoTYLKJV2s2xw7Q3vPCVHp3sHqPa"
```

### Remove offline peers
```bash
./claude-flow qudag peer cleanup --offline-timeout 300s
```

### Remove all peers (full reset)
```bash
./claude-flow qudag peer reset --confirm
```

## Peer Discovery

### Start active peer discovery
```bash
./claude-flow qudag peer discover --mdns --bootstrap --duration 60s
```

### Discovery via DHT
```bash
./claude-flow qudag peer discover-dht --target-peers 20 --timeout 30s
```

### Discovery via rendezvous point
```bash
./claude-flow qudag peer discover-rendezvous --namespace "qudag-mainnet" --interval 15s
```

### Passive discovery (listen only)
```bash
./claude-flow qudag peer discover --passive --listen-ports 9000-9010
```

## Connection Health Monitoring

### Monitor all peer connections
```bash
./claude-flow qudag peer monitor --refresh 5s --show-latency
```

### Health check specific peer
```bash
./claude-flow qudag peer health "12D3KooWBmwkafWE2fqfzS96VoTYLKJV2s2xw7Q3vPCVHp3sHqPa"
```

### Connection quality metrics
```bash
./claude-flow qudag peer metrics --detailed --export metrics.json
```

### Auto-heal connections
```bash
./claude-flow qudag peer auto-heal --max-retries 3 --backoff exponential
```

## NAT Traversal Setup

### Configure STUN servers
```bash
./claude-flow qudag peer nat-config --stun "stun.l.google.com:19302" --stun "stun1.l.google.com:19302"
```

### Test NAT traversal
```bash
./claude-flow qudag peer nat-test --timeout 30s --report
```

### Enable hole punching
```bash
./claude-flow qudag peer enable-holepunch --relay-limit 5
```

### Configure relay nodes
```bash
./claude-flow qudag peer relay add "/ip4/relay1.qudag.org/tcp/443/wss/p2p/12D3RelayNode1"
```

## Peer Information

### List all connected peers
```bash
./claude-flow qudag peer list --format table --show-connections
```

### Show peer details
```bash
./claude-flow qudag peer info "12D3KooWBmwkafWE2fqfzS96VoTYLKJV2s2xw7Q3vPCVHp3sHqPa" --verbose
```

### Export peer database
```bash
./claude-flow qudag peer export --format json --file peers-backup.json
```

### Import peer database
```bash
./claude-flow qudag peer import --file peers-backup.json --merge
```

## Network Topology

### View network topology
```bash
./claude-flow qudag peer topology --depth 3 --visualize
```

### Export topology for analysis
```bash
./claude-flow qudag peer topology --export graph.dot --format graphviz
```

### Measure network diameter
```bash
./claude-flow qudag peer diameter --samples 100 --timeout 60s
```

## Security Controls

### Whitelist trusted peers
```bash
./claude-flow qudag peer whitelist add "12D3KooWTrustedPeer" --auto-connect
```

### Blacklist malicious peers
```bash
./claude-flow qudag peer blacklist add "12D3KooWMaliciousPeer" --permanent
```

### Rate limiting per peer
```bash
./claude-flow qudag peer rate-limit --max-conn 5 --max-msg 100/s
```

### Enable peer authentication
```bash
./claude-flow qudag peer auth enable --require-signatures --challenge-timeout 10s
```

## Testing Scenarios

### Simulate peer churn
```bash
./claude-flow qudag peer test-churn --add-rate 5/min --remove-rate 3/min --duration 10m
```

### Test with high latency peers
```bash
./claude-flow qudag peer test-latency --inject-delay 500ms --peer-count 10
```

### Connection stress test
```bash
./claude-flow qudag peer stress-test --concurrent 100 --duration 60s
```

### Byzantine peer simulation
```bash
./claude-flow qudag peer test-byzantine --malicious-count 5 --behavior random-drop
```