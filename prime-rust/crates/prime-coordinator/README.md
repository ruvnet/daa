# DAA Prime Coordinator

[![Crates.io](https://img.shields.io/crates/v/daa-prime-coordinator.svg)](https://crates.io/crates/daa-prime-coordinator)
[![Documentation](https://docs.rs/daa-prime-coordinator/badge.svg)](https://docs.rs/daa-prime-coordinator)
[![License](https://img.shields.io/crates/l/daa-prime-coordinator.svg)](https://github.com/yourusername/daa/blob/main/LICENSE)

Governance and coordination layer for the Prime distributed machine learning framework. Provides decentralized coordination, task distribution, consensus mechanisms, and economic incentives through deep integration with the DAA ecosystem.

## Overview

DAA Prime Coordinator serves as the governance and orchestration layer for distributed ML training, providing:

- **Decentralized Coordination**: Consensus-based task distribution without central authority
- **Economic Incentives**: Token-based rewards for quality contributions via DAA Economy
- **Governance Integration**: Rule-based policies through DAA Rules system
- **Fault Tolerance**: Byzantine fault-tolerant coordination protocols
- **Performance Monitoring**: Real-time training metrics and node reliability scoring

## Features

- 🎯 **Smart Coordination**: Intelligent task assignment based on node capabilities
- 💰 **Economic Incentives**: Automated reward distribution for quality work
- 🏛️ **Governance**: Decentralized decision making with stakeholder voting
- 🛡️ **Byzantine Fault Tolerance**: Robust against malicious and faulty nodes
- 📊 **Analytics**: Comprehensive training progress and node performance tracking
- 🔄 **Self-Healing**: Automatic recovery from coordinator failures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
daa-prime-coordinator = "0.2.1"
daa-prime-core = "0.2.1"
daa-ai = "0.2.1"
daa-rules = "0.2.1"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

### Basic Coordinator

```rust
use daa_prime_coordinator::{CoordinatorNode, CoordinatorConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create coordinator with default configuration
    let config = CoordinatorConfig::default();
    let coordinator = CoordinatorNode::new("coordinator-001".to_string(), config).await?;
    
    // Start coordination services
    coordinator.start().await?;
    
    Ok(())
}
```

### Custom Configuration

```rust
use daa_prime_coordinator::{CoordinatorNode, CoordinatorConfig};
use std::time::Duration;

let config = CoordinatorConfig {
    min_nodes_for_round: 5,           // Minimum participants per round
    heartbeat_timeout_ms: 10000,      // 10 second heartbeat timeout
    task_timeout_ms: 300000,          // 5 minute task timeout
    consensus_threshold: 0.75,        // 75% consensus required
};

let coordinator = CoordinatorNode::new("coordinator-001".to_string(), config).await?;
```

### Node Management

```rust
use daa_prime_coordinator::{CoordinatorNode, NodeInfo};

let coordinator = CoordinatorNode::new("coordinator-001".to_string(), config).await?;

// Add a trainer node
let node_info = NodeInfo {
    node_id: "trainer-001".to_string(),
    node_type: "trainer".to_string(),
    last_heartbeat: current_timestamp(),
    reliability_score: 0.95,
};

coordinator.add_node(node_info).await?;

// Get coordinator status
let status = coordinator.get_status().await?;
println!("Active nodes: {}, Pending tasks: {}", 
    status.active_nodes, status.pending_tasks);
```

## Core Concepts

### Training Round Coordination

```rust
use daa_prime_coordinator::{TrainingRound, RoundConfig};

// Configure training round
let round_config = RoundConfig {
    round_id: 42,
    min_participants: 5,
    max_participants: 20,
    deadline: Duration::from_secs(600), // 10 minutes
    model_version: 15,
    aggregation_strategy: AggregationStrategy::FederatedAveraging,
};

// Start new training round
coordinator.start_training_round(round_config).await?;

// Monitor round progress
let progress = coordinator.get_round_progress(42).await?;
println!("Round 42: {}/{} participants completed", 
    progress.completed, progress.total);
```

### Task Distribution

```rust
use daa_prime_coordinator::{TrainingTask, TaskPriority};

// Create training task
let task = TrainingTask {
    task_id: "gradient_computation_1".to_string(),
    task_type: "gradient_computation".to_string(),
    deadline: current_timestamp() + 300, // 5 minutes from now
    reward_amount: 100, // Token reward
    priority: TaskPriority::High,
    required_capabilities: vec!["gpu".to_string(), "pytorch".to_string()],
};

// Distribute task to eligible nodes
coordinator.distribute_task(task).await?;
```

### Consensus Protocol

```rust
use daa_prime_coordinator::{ConsensusManager, ConsensusConfig};

let consensus_config = ConsensusConfig {
    algorithm: ConsensusAlgorithm::PBFT, // Practical Byzantine Fault Tolerance
    byzantine_tolerance: 0.33,           // Tolerate up to 1/3 byzantine nodes
    timeout: Duration::from_secs(30),
    max_rounds: 10,
};

let consensus = ConsensusManager::new(consensus_config);

// Propose a value for consensus
let proposal = ConsensusProposal {
    round: 1,
    value: serde_json::to_vec(&aggregated_gradients)?,
    proposer: coordinator.node_id(),
};

consensus.propose(proposal).await?;
```

## Advanced Usage

### Economic Incentives Integration

```rust
use daa_prime_coordinator::{CoordinatorNode, EconomicConfig};
use daa_economy::{TokenManager, RewardCalculator};

// Configure economic incentives
let economic_config = EconomicConfig {
    base_reward: 100,              // Base tokens per task
    quality_multiplier: 2.0,       // Bonus for high-quality work
    reliability_bonus: 1.5,        // Bonus for reliable nodes
    slashing_rate: 0.1,           // Penalty for bad behavior
};

let coordinator = CoordinatorNode::with_economic_config(
    "coordinator-001".to_string(),
    config,
    economic_config
).await?;

// Automatic reward distribution after task completion
coordinator.enable_automatic_rewards().await?;
```

### Governance Integration

```rust
use daa_prime_coordinator::{GovernanceManager, ProposalType};
use daa_rules::{Rule, PolicyEngine};

// Create governance proposal
let proposal = GovernanceProposal {
    proposal_id: "increase_min_nodes".to_string(),
    proposal_type: ProposalType::ParameterChange,
    description: "Increase minimum nodes per round to 10".to_string(),
    changes: vec![
        ParameterChange {
            key: "min_nodes_for_round".to_string(),
            old_value: "5".to_string(),
            new_value: "10".to_string(),
        }
    ],
    voting_deadline: current_timestamp() + 86400, // 24 hours
};

// Submit for stakeholder voting
governance.submit_proposal(proposal).await?;

// Vote on proposal (as a stakeholder)
governance.vote("increase_min_nodes", VoteChoice::Approve, 1000).await?; // 1000 token weight
```

### Custom Node Selection

```rust
use daa_prime_coordinator::{NodeSelector, SelectionCriteria};

// Implement custom node selection strategy
struct PerformanceBasedSelector;

impl NodeSelector for PerformanceBasedSelector {
    async fn select_nodes(
        &self,
        available_nodes: &[NodeInfo],
        criteria: &SelectionCriteria,
    ) -> Result<Vec<NodeInfo>> {
        let mut nodes = available_nodes.to_vec();
        
        // Sort by reliability score and recent performance
        nodes.sort_by(|a, b| {
            let score_a = a.reliability_score * a.recent_performance_score;
            let score_b = b.reliability_score * b.recent_performance_score;
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        // Select top N nodes
        Ok(nodes.into_iter().take(criteria.max_nodes).collect())
    }
}

// Use custom selector
coordinator.set_node_selector(Box::new(PerformanceBasedSelector)).await?;
```

### Multi-Coordinator Setup

```rust
use daa_prime_coordinator::{CoordinatorCluster, ClusterConfig};

// Set up coordinator cluster for high availability
let cluster_config = ClusterConfig {
    cluster_id: "main_cluster".to_string(),
    coordinators: vec![
        "coordinator-001".to_string(),
        "coordinator-002".to_string(),
        "coordinator-003".to_string(),
    ],
    leader_election: LeaderElection::Raft,
    failover_timeout: Duration::from_secs(30),
};

let cluster = CoordinatorCluster::new(cluster_config).await?;

// Coordinators automatically handle:
// - Leader election
// - Load balancing
// - Failover
// - State synchronization
```

### Performance Monitoring

```rust
use daa_prime_coordinator::{MetricsCollector, PerformanceMetrics};

// Set up comprehensive monitoring
let metrics = MetricsCollector::new();
coordinator.set_metrics_collector(metrics).await?;

// Real-time monitoring loop
loop {
    let performance = coordinator.get_performance_metrics().await?;
    
    println!("Coordinator Performance:");
    println!("  Active rounds: {}", performance.active_rounds);
    println!("  Tasks/second: {:.2}", performance.task_throughput);
    println!("  Average task completion: {:.1}s", performance.avg_task_completion);
    println!("  Node reliability: {:.1}%", performance.avg_node_reliability * 100.0);
    
    // Alert on performance degradation
    if performance.task_throughput < 1.0 {
        eprintln!("WARNING: Low task throughput detected!");
    }
    
    tokio::time::sleep(Duration::from_secs(30)).await;
}
```

## Configuration

### Coordinator Configuration

```rust
use daa_prime_coordinator::CoordinatorConfig;
use std::time::Duration;

let config = CoordinatorConfig {
    // Basic coordination parameters
    min_nodes_for_round: 3,
    max_nodes_per_round: 100,
    heartbeat_timeout_ms: 5000,
    task_timeout_ms: 60000,
    
    // Consensus parameters
    consensus_threshold: 0.66,      // 2/3 majority
    byzantine_tolerance: 0.33,      // Tolerate 1/3 byzantine nodes
    max_consensus_rounds: 5,
    
    // Economic parameters
    enable_rewards: true,
    base_reward_amount: 100,
    quality_threshold: 0.8,
    
    // Performance parameters
    max_concurrent_rounds: 10,
    task_queue_size: 1000,
    node_cache_ttl: Duration::from_secs(300),
};
```

### Security Configuration

```rust
use daa_prime_coordinator::SecurityConfig;

let security_config = SecurityConfig {
    // Authentication
    require_node_signatures: true,
    verify_task_results: true,
    
    // Anti-sybil measures
    min_stake_requirement: 1000,    // Minimum tokens to participate
    reputation_threshold: 0.7,      // Minimum reputation score
    
    // Rate limiting
    max_tasks_per_node_per_hour: 100,
    max_heartbeats_per_minute: 2,
    
    // Byzantine fault tolerance
    enable_bft_consensus: true,
    slashing_enabled: true,
    evidence_preservation_period: Duration::from_secs(86400 * 7), // 1 week
};
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use daa_prime_coordinator::test_utils::*;
    
    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = CoordinatorConfig::default();
        let coordinator = CoordinatorNode::new("test-coord".to_string(), config).await.unwrap();
        
        let status = coordinator.get_status().await.unwrap();
        assert_eq!(status.active_nodes, 0);
        assert_eq!(status.current_round, 0);
    }
    
    #[tokio::test]
    async fn test_node_management() {
        let coordinator = create_test_coordinator().await;
        
        let node_info = NodeInfo {
            node_id: "test-node".to_string(),
            node_type: "trainer".to_string(),
            last_heartbeat: current_timestamp(),
            reliability_score: 0.9,
        };
        
        coordinator.add_node(node_info.clone()).await.unwrap();
        
        let status = coordinator.get_status().await.unwrap();
        assert_eq!(status.active_nodes, 1);
        
        let nodes = coordinator.list_nodes().await.unwrap();
        assert_eq!(nodes[0].node_id, "test-node");
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use daa_prime_trainer::TrainerNode;
    
    #[tokio::test]
    async fn test_full_training_round() {
        // Set up coordinator
        let coordinator = CoordinatorNode::new(
            "test-coordinator".to_string(),
            CoordinatorConfig::default()
        ).await.unwrap();
        
        // Set up multiple trainers
        let mut trainers = Vec::new();
        for i in 0..5 {
            let trainer = TrainerNode::new(format!("trainer-{}", i)).await.unwrap();
            trainers.push(trainer);
        }
        
        // Register trainers with coordinator
        for trainer in &trainers {
            let node_info = NodeInfo {
                node_id: trainer.node_id().clone(),
                node_type: "trainer".to_string(),
                last_heartbeat: current_timestamp(),
                reliability_score: 0.95,
            };
            coordinator.add_node(node_info).await.unwrap();
        }
        
        // Start training round
        let round_config = RoundConfig::default();
        coordinator.start_training_round(round_config).await.unwrap();
        
        // Wait for round completion
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // Verify round completed successfully
        let status = coordinator.get_status().await.unwrap();
        assert!(status.completed_rounds > 0);
    }
}
```

### Load Testing

```rust
#[cfg(test)]
mod load_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    
    #[tokio::test]
    async fn test_high_load_coordination() {
        let coordinator = create_test_coordinator().await;
        
        // Simulate 1000 concurrent node registrations
        let semaphore = Arc::new(Semaphore::new(100)); // Limit to 100 concurrent
        let mut handles = Vec::new();
        
        for i in 0..1000 {
            let coordinator = coordinator.clone();
            let semaphore = semaphore.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                let node_info = NodeInfo {
                    node_id: format!("load-test-node-{}", i),
                    node_type: "trainer".to_string(),
                    last_heartbeat: current_timestamp(),
                    reliability_score: 0.9,
                };
                
                coordinator.add_node(node_info).await.unwrap();
            });
            
            handles.push(handle);
        }
        
        // Wait for all registrations to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let status = coordinator.get_status().await.unwrap();
        assert_eq!(status.active_nodes, 1000);
    }
}
```

## Deployment

### Single Coordinator

```bash
# Start coordinator
cargo run --bin prime-coordinator -- \
    --node-id coordinator-001 \
    --config config.json \
    --port 8080 \
    --enable-governance \
    --enable-economics
```

### Coordinator Cluster

```bash
# Start first coordinator (will become leader)
cargo run --bin prime-coordinator -- \
    --node-id coordinator-001 \
    --cluster-config cluster.json \
    --bootstrap

# Start additional coordinators
cargo run --bin prime-coordinator -- \
    --node-id coordinator-002 \
    --cluster-config cluster.json \
    --join coordinator-001:8080

cargo run --bin prime-coordinator -- \
    --node-id coordinator-003 \
    --cluster-config cluster.json \
    --join coordinator-001:8080
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin prime-coordinator

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/prime-coordinator /usr/local/bin/

EXPOSE 8080
CMD ["prime-coordinator", "--config", "/etc/coordinator/config.json"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  coordinator-1:
    image: daa-prime-coordinator:latest
    ports:
      - "8080:8080"
    environment:
      - NODE_ID=coordinator-001
      - CLUSTER_MODE=true
    volumes:
      - ./config:/etc/coordinator
    
  coordinator-2:
    image: daa-prime-coordinator:latest
    ports:
      - "8081:8080"
    environment:
      - NODE_ID=coordinator-002
      - CLUSTER_MODE=true
      - BOOTSTRAP_PEER=coordinator-1:8080
    volumes:
      - ./config:/etc/coordinator
```

## Monitoring and Observability

### Prometheus Metrics

```rust
use daa_prime_coordinator::metrics::PrometheusExporter;

// Export metrics for Prometheus
let exporter = PrometheusExporter::new();
coordinator.set_metrics_exporter(exporter).await?;

// Metrics exported:
// - daa_coordinator_active_nodes
// - daa_coordinator_active_rounds  
// - daa_coordinator_task_completion_time
// - daa_coordinator_consensus_duration
// - daa_coordinator_byzantine_faults_detected
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "DAA Prime Coordinator",
    "panels": [
      {
        "title": "Active Nodes",
        "type": "stat",
        "targets": [
          {
            "expr": "daa_coordinator_active_nodes"
          }
        ]
      },
      {
        "title": "Task Throughput",
        "type": "graph", 
        "targets": [
          {
            "expr": "rate(daa_coordinator_tasks_completed_total[5m])"
          }
        ]
      }
    ]
  }
}
```

## Troubleshooting

### Common Issues

1. **Consensus Timeouts**
   ```rust
   // Increase consensus timeout for slow networks
   let config = CoordinatorConfig {
       consensus_timeout: Duration::from_secs(60),
       max_consensus_rounds: 10,
       ..Default::default()
   };
   ```

2. **Byzantine Behavior Detection**
   ```rust
   // Enable enhanced byzantine detection
   coordinator.enable_byzantine_detection(ByzantineDetectionConfig {
       gradient_similarity_threshold: 0.8,
       performance_deviation_threshold: 2.0,
       voting_pattern_analysis: true,
   }).await?;
   ```

3. **High Memory Usage**
   ```rust
   // Configure memory limits
   coordinator.set_memory_limits(MemoryLimits {
       max_cached_rounds: 100,
       max_node_history: 1000,
       cleanup_interval: Duration::from_secs(3600),
   }).await?;
   ```

## Roadmap

- [ ] Advanced consensus algorithms (HotStuff, Tendermint)
- [ ] Cross-chain governance integration
- [ ] ML model governance and versioning
- [ ] Automated economic parameter tuning
- [ ] Advanced byzantine fault detection
- [ ] Integration with external monitoring systems

## Contributing

Contributions are welcome! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Related Crates

- [`daa-prime-core`](../prime-core): Core types and protocol definitions
- [`daa-prime-trainer`](../prime-trainer): Training node implementation
- [`daa-prime-dht`](../prime-dht): Distributed hash table for coordination
- [`daa-prime-cli`](../prime-cli): Command-line interface

## References

- [Practical Byzantine Fault Tolerance](https://pmg.csail.mit.edu/papers/osdi99.pdf) - Castro & Liskov
- [Federated Learning Systems](https://arxiv.org/abs/1902.01046) - Bonawitz et al.
- [Blockchain Governance](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=3096175) - Miscione et al.

## Support

- 📖 [Documentation](https://docs.rs/daa-prime-coordinator)
- 🐛 [Issue Tracker](https://github.com/yourusername/daa/issues)
- 💬 [Discussions](https://github.com/yourusername/daa/discussions)