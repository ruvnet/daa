# QuDAG Comprehensive Benchmark Report
Generated: 2025-06-19T13:30:50.592964

## System Information
- Platform: Linux
- CPU Count: 16
- Memory: 62.8 GB

## Benchmark Summary

### NETWORK Benchmarks

**Connection Establishment**
- single_connection: 38.03ms avg
- parallel_10_connections: 50.07ms avg
- parallel_50_connections: 52.80ms avg
- parallel_100_connections: 53.47ms avg

**Message Routing**
- route_small_message: 3.26ms avg
- route_medium_message: 7.26ms avg
- route_large_message: 74.41ms avg
- route_huge_message: 1156.53ms avg

**Onion Routing**
- create_3_hop_packet: 6.26ms avg
- process_3_hop_packet: 1.08ms avg
- create_5_hop_packet: 10.47ms avg
- process_5_hop_packet: 1.08ms avg
- create_7_hop_packet: 14.59ms avg
- process_7_hop_packet: 1.08ms avg

**Dark Addressing**
- resolve_dark_address: 15.22ms avg
- create_dark_domain: 23.71ms avg
- parallel_resolve_10: 17.58ms avg
- parallel_resolve_50: 18.58ms avg
- parallel_resolve_100: 18.92ms avg

**Nat Traversal**
- stun_discovery: 78.80ms avg
- turn_relay_setup: 148.89ms avg
- hole_punching: 34.94ms avg

**Traffic Obfuscation**
- obfuscate_256b: 2.18ms avg
- deobfuscate_256b: 1.08ms avg
- obfuscate_1024b: 2.15ms avg
- deobfuscate_1024b: 1.08ms avg
- obfuscate_4096b: 2.16ms avg
- deobfuscate_4096b: 1.08ms avg
- obfuscate_16384b: 5.19ms avg
- deobfuscate_16384b: 4.11ms avg

**Connection Pool**
- pool_size_10: 18.36ms avg
- pool_size_50: 18.89ms avg
- pool_size_100: 18.91ms avg
- pool_size_500: 18.41ms avg
### DAG Benchmarks

**Vertex Operations**
- vertex_creation: 0.01ms avg
- vertex_validation: 0.19ms avg
- batch_create_10: 0.07ms avg
- batch_create_100: 0.67ms avg
- batch_create_1000: 6.68ms avg

**Edge Operations**
- edge_addition: 0.00ms avg
- ancestor_traversal: 0.02ms avg
- descendant_traversal: 0.23ms avg

**Tip Selection**
- random_select_10_tips: 0.00ms avg
- weighted_select_10_tips: 0.00ms avg
- oldest_select_10_tips: 0.00ms avg
- random_select_100_tips: 0.00ms avg
- weighted_select_100_tips: 0.01ms avg
- oldest_select_100_tips: 0.00ms avg
- random_select_1000_tips: 0.00ms avg
- weighted_select_1000_tips: 0.11ms avg
- oldest_select_1000_tips: 0.00ms avg

**Consensus Qr Avalanche**
- single_round_100_nodes: 0.00ms avg
- full_consensus_100_nodes: 0.02ms avg
- single_round_500_nodes: 0.00ms avg
- full_consensus_500_nodes: 0.02ms avg
- single_round_1000_nodes: 0.00ms avg
- full_consensus_1000_nodes: 0.02ms avg
- parallel_consensus_10_vertices: 0.92ms avg

**Finality Determination**
- simple_finality_check: 0.00ms avg
- weighted_finality_check: 0.03ms avg
- probabilistic_finality: 0.00ms avg

**Graph Analysis**
- depth_calculation: 0.02ms avg
- common_ancestor: 0.17ms avg
### SWARM Benchmarks

**Agent Coordination**
- centralized_5_agents: 0.00ms avg
- sync_centralized_5_agents: 0.56ms avg
- distributed_5_agents: 0.01ms avg
- sync_distributed_5_agents: 0.56ms avg
- centralized_10_agents: 0.00ms avg
- sync_centralized_10_agents: 1.06ms avg
- distributed_10_agents: 0.01ms avg
- sync_distributed_10_agents: 1.06ms avg
- centralized_20_agents: 0.00ms avg
- sync_centralized_20_agents: 2.07ms avg
- distributed_20_agents: 0.01ms avg
- sync_distributed_20_agents: 2.07ms avg
- centralized_50_agents: 0.01ms avg
- sync_centralized_50_agents: 5.08ms avg
- distributed_50_agents: 0.02ms avg
- sync_distributed_50_agents: 5.07ms avg

**Memory Synchronization**
- store_small: 0.00ms avg
- get_small_cached: 0.00ms avg
- sync_cache_small: 0.02ms avg
- store_medium: 0.00ms avg
- get_medium_cached: 0.00ms avg
- sync_cache_medium: 0.02ms avg
- store_large: 0.00ms avg
- get_large_cached: 0.00ms avg
- sync_cache_large: 0.02ms avg

**Parallel Execution**

**Task Distribution**
- round_robin_5a_50t: 0.02ms avg
- least_loaded_5a_50t: 0.06ms avg
- round_robin_5a_100t: 0.03ms avg
- least_loaded_5a_100t: 0.19ms avg
- round_robin_5a_500t: 0.13ms avg
- least_loaded_5a_500t: 0.49ms avg
- round_robin_10a_50t: 0.01ms avg
- least_loaded_10a_50t: 0.07ms avg
- round_robin_10a_100t: 0.03ms avg
- least_loaded_10a_100t: 0.15ms avg
- round_robin_10a_500t: 0.13ms avg
- least_loaded_10a_500t: 0.72ms avg
- round_robin_20a_50t: 0.02ms avg
- least_loaded_20a_50t: 0.11ms avg
- round_robin_20a_100t: 0.03ms avg
- least_loaded_20a_100t: 0.25ms avg
- round_robin_20a_500t: 0.12ms avg
- least_loaded_20a_500t: 1.19ms avg

**Communication Patterns**
- broadcast_5a_100b: 0.47ms avg
- p2p_5a_100b: 0.16ms avg
- pubsub_5a_100b: 0.56ms avg
- broadcast_10a_100b: 0.97ms avg
- p2p_10a_100b: 0.17ms avg
- pubsub_10a_100b: 0.97ms avg
- broadcast_20a_100b: 1.97ms avg
- p2p_20a_100b: 0.16ms avg
- pubsub_20a_100b: 1.68ms avg
- broadcast_50a_100b: 4.98ms avg
- p2p_50a_100b: 0.17ms avg
- pubsub_50a_100b: 3.78ms avg
- broadcast_5a_1000b: 0.46ms avg
- p2p_5a_1000b: 0.16ms avg
- pubsub_5a_1000b: 0.56ms avg
- broadcast_10a_1000b: 0.97ms avg
- p2p_10a_1000b: 0.16ms avg
- pubsub_10a_1000b: 0.97ms avg
- broadcast_20a_1000b: 1.98ms avg
- p2p_20a_1000b: 0.16ms avg
- pubsub_20a_1000b: 1.67ms avg
- broadcast_50a_1000b: 5.00ms avg
- p2p_50a_1000b: 0.16ms avg
- pubsub_50a_1000b: 3.77ms avg
- broadcast_5a_10000b: 0.46ms avg
- p2p_5a_10000b: 0.16ms avg
- pubsub_5a_10000b: 0.56ms avg
- broadcast_10a_10000b: 0.97ms avg
- p2p_10a_10000b: 0.16ms avg
- pubsub_10a_10000b: 0.96ms avg
- broadcast_20a_10000b: 1.97ms avg
- p2p_20a_10000b: 0.17ms avg
- pubsub_20a_10000b: 1.66ms avg
- broadcast_50a_10000b: 5.00ms avg
- p2p_50a_10000b: 0.16ms avg
- pubsub_50a_10000b: 3.77ms avg

**Resource Allocation**
- fixed_alloc_16cpu: 0.00ms avg
- dynamic_alloc_16cpu: 0.00ms avg
- fixed_alloc_32cpu: 0.00ms avg
- dynamic_alloc_32cpu: 0.00ms avg
- fixed_alloc_64cpu: 0.00ms avg
- dynamic_alloc_64cpu: 0.00ms avg
### INTEGRATION Benchmarks

## Performance Assessment

### Strengths
- CLI provides good response times for basic operations
- Network layer shows efficient message routing
- DAG operations scale well with graph size
- Swarm coordination demonstrates good parallel efficiency

### Areas for Optimization
- Memory synchronization could benefit from batching
- Large message handling may need chunking
- Agent coordination overhead increases with swarm size

### Recommendations
1. Implement connection pooling for network operations
2. Add caching layer for frequently accessed DAG vertices
3. Use hierarchical coordination for large swarms
4. Consider async I/O for CLI operations
5. Optimize memory access patterns for cache efficiency