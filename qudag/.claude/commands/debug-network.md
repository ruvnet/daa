# debug-network

Diagnose networking issues including P2P connectivity, routing, and dark addressing

## Usage

```
/debug-network [node] [options]
```

## Parameters

### node (optional)
- **Type**: string
- **Description**: Optional node ID or dark address to debug

### component (optional)
- **Type**: string
- **Description**: Specific network component to debug
- **Options**: p2p, routing, dark_addressing, onion, transport, discovery, all
- **Default**: all

### verbose (optional)
- **Type**: boolean
- **Description**: Enable verbose logging
- **Default**: false

### trace (optional)
- **Type**: boolean
- **Description**: Enable message tracing
- **Default**: false

## Examples

```
/debug-network
/debug-network --node node-123
/debug-network --component routing --verbose
/debug-network --node dark:7f3a9b2c --trace
/debug-network --component dark_addressing
/debug-network --component onion --verbose
```

## Diagnostic Checks

### P2P
- peer_connections
- handshake_status
- connection_health
- bandwidth_usage

### Routing
- routing_table
- path_diversity
- route_convergence
- dead_routes

### Dark Addressing
- address_resolution
- dark_pool_status
- anonymity_metrics
- lookup_performance

### Onion
- layer_construction
- circuit_health
- relay_performance
- anonymity_preservation

### Transport
- protocol_stats
- encryption_status
- compression_ratio
- error_rates

## Output Format

```
1. Network Overview
   - Node ID: {node_id}
   - Dark Address: {dark_address}
   - Uptime: {uptime}
   - Network Status: {status}
2. P2P Connectivity
   - Active Peers: {peer_count}
   - Connection Health: {connection_metrics}
   - Bandwidth: In: {bandwidth_in}, Out: {bandwidth_out}
   - Failed Connections: {failed_connections}
3. Routing Analysis
   - Routing Table Size: {route_count}
   - Path Diversity: {path_diversity_score}
   - Average Hop Count: {avg_hops}
   - Routing Failures: {routing_failures}
4. Dark Addressing System
   - Active Dark Addresses: {dark_address_count}
   - Resolution Success Rate: {resolution_rate}%
   - Anonymity Set Size: {anonymity_set}
   - Lookup Latency: {lookup_latency}ms
5. Onion Routing
   - Active Circuits: {circuit_count}
   - Layer Count: {onion_layers}
   - Circuit Build Time: {circuit_build_time}ms
   - Relay Performance: {relay_metrics}
6. Performance Metrics
   - Message Throughput: {messages_per_second}
   - Average Latency: {avg_latency}ms
   - Memory Usage: {memory_usage}MB
   - CPU Usage: {cpu_usage}%
7. Error Analysis
   - Recent Errors: {error_summary}
   - Error Rate: {error_rate}
   - Critical Issues: {critical_issues}
8. Security Status
   - Encryption: {encryption_status}
   - Authentication: {auth_status}
   - Anomalies Detected: {anomaly_count}
9. Recommendations
   - Performance: {performance_suggestions}
   - Security: {security_recommendations}
   - Configuration: {config_improvements}
```

## Performance Thresholds

- **Latency Warning**: 100ms
- **Latency Critical**: 1000ms
- **Minimum Peer Count**: 3
- **Minimum Bandwidth**: 1MB/s
- **Maximum Error Rate**: 0.01

## Error Handling

- **node_offline**: Node is offline or unreachable
- **invalid_node**: Invalid node ID or dark address format
- **network_error**: Network diagnostic failure: {error}
- **component_error**: Failed to diagnose {component}: {error}
- **permission_denied**: Insufficient permissions to access network diagnostics

## Related Commands

- `/network-simulate`
- `/performance-benchmark network`
- `/security-audit network`
- `/tdd-cycle network`

## Agent Context

- **Primary Agent**: `agents/network_agent.md`
- **Contexts**: `contexts/performance_context.md`, `contexts/test_status.md`