//! Custom assertions and test helpers

use std::time::Duration;
use std::collections::HashMap;

/// Assert that a future completes within a timeout
#[macro_export]
macro_rules! assert_completes_within {
    ($timeout:expr, $future:expr) => {
        match tokio::time::timeout($timeout, $future).await {
            Ok(result) => result,
            Err(_) => panic!("Operation timed out after {:?}", $timeout),
        }
    };
}

/// Assert that a future eventually returns true
#[macro_export]
macro_rules! assert_eventually {
    ($condition:expr, $timeout:expr) => {
        assert_eventually!($condition, $timeout, Duration::from_millis(100))
    };
    ($condition:expr, $timeout:expr, $interval:expr) => {{
        let start = std::time::Instant::now();
        loop {
            if $condition {
                break;
            }
            if start.elapsed() > $timeout {
                panic!("Condition not met within {:?}", $timeout);
            }
            tokio::time::sleep($interval).await;
        }
    }};
}

/// Assert that values converge within tolerance
#[macro_export]
macro_rules! assert_converges {
    ($values:expr, $tolerance:expr) => {{
        let values: Vec<f32> = $values;
        if values.is_empty() {
            panic!("No values to check convergence");
        }
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        for value in &values {
            let diff = (value - mean).abs();
            if diff > $tolerance {
                panic!(
                    "Value {} differs from mean {} by {}, exceeding tolerance {}",
                    value, mean, diff, $tolerance
                );
            }
        }
    }};
}

/// Assert network properties
pub struct NetworkAssertions;

impl NetworkAssertions {
    /// Assert that all nodes are connected
    pub fn assert_fully_connected(connections: &HashMap<String, Vec<String>>) {
        let nodes: Vec<&String> = connections.keys().collect();
        
        for node in &nodes {
            let connected_to = connections.get(*node).unwrap();
            for other in &nodes {
                if node != other && !connected_to.contains(other) {
                    panic!("Node {} is not connected to {}", node, other);
                }
            }
        }
    }

    /// Assert that network has no partitions
    pub fn assert_no_partitions(connections: &HashMap<String, Vec<String>>) {
        if connections.is_empty() {
            return;
        }

        let mut visited = std::collections::HashSet::new();
        let start_node = connections.keys().next().unwrap();
        
        Self::dfs(start_node, connections, &mut visited);
        
        if visited.len() != connections.len() {
            panic!(
                "Network is partitioned: only {} of {} nodes reachable",
                visited.len(),
                connections.len()
            );
        }
    }

    fn dfs(
        node: &String,
        connections: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(node) {
            return;
        }
        
        visited.insert(node.clone());
        
        if let Some(neighbors) = connections.get(node) {
            for neighbor in neighbors {
                Self::dfs(neighbor, connections, visited);
            }
        }
    }

    /// Assert minimum connectivity
    pub fn assert_min_connectivity(
        connections: &HashMap<String, Vec<String>>,
        min_connections: usize,
    ) {
        for (node, connected) in connections {
            if connected.len() < min_connections {
                panic!(
                    "Node {} has only {} connections, minimum required: {}",
                    node,
                    connected.len(),
                    min_connections
                );
            }
        }
    }
}

/// Assert consensus properties
pub struct ConsensusAssertions;

impl ConsensusAssertions {
    /// Assert safety: no two different values committed in same round
    pub fn assert_safety(commits: &[(String, u64, Vec<u8>)]) {
        let mut round_values: HashMap<u64, Vec<u8>> = HashMap::new();
        
        for (_, round, value) in commits {
            if let Some(existing) = round_values.get(round) {
                if existing != value {
                    panic!(
                        "Safety violation: different values committed in round {}",
                        round
                    );
                }
            } else {
                round_values.insert(*round, value.clone());
            }
        }
    }

    /// Assert liveness: progress is being made
    pub fn assert_liveness(rounds: &[u64], max_gap: u64) {
        if rounds.len() < 2 {
            return;
        }

        let mut sorted_rounds = rounds.to_vec();
        sorted_rounds.sort();
        
        for window in sorted_rounds.windows(2) {
            let gap = window[1] - window[0];
            if gap > max_gap {
                panic!(
                    "Liveness violation: gap of {} rounds between {} and {}",
                    gap, window[0], window[1]
                );
            }
        }
    }

    /// Assert agreement: all nodes agree on committed values
    pub fn assert_agreement(node_commits: &HashMap<String, Vec<(u64, Vec<u8>)>>) {
        let mut round_values: HashMap<u64, Vec<(String, Vec<u8>)>> = HashMap::new();
        
        for (node, commits) in node_commits {
            for (round, value) in commits {
                round_values
                    .entry(*round)
                    .or_default()
                    .push((node.clone(), value.clone()));
            }
        }

        for (round, node_values) in round_values {
            let first_value = &node_values[0].1;
            for (node, value) in &node_values[1..] {
                if value != first_value {
                    panic!(
                        "Agreement violation in round {}: nodes have different values",
                        round
                    );
                }
            }
        }
    }
}

/// Assert training properties
pub struct TrainingAssertions;

impl TrainingAssertions {
    /// Assert model convergence
    pub fn assert_convergence(losses: &[f32], window_size: usize, threshold: f32) {
        if losses.len() < window_size * 2 {
            return;
        }

        let early_window: Vec<f32> = losses[..window_size].to_vec();
        let late_window: Vec<f32> = losses[losses.len() - window_size..].to_vec();
        
        let early_avg = early_window.iter().sum::<f32>() / window_size as f32;
        let late_avg = late_window.iter().sum::<f32>() / window_size as f32;
        
        let improvement = (early_avg - late_avg) / early_avg;
        
        if improvement < threshold {
            panic!(
                "Model not converging: improvement {} is less than threshold {}",
                improvement, threshold
            );
        }
    }

    /// Assert gradient consistency
    pub fn assert_gradient_consistency(
        gradients: &HashMap<String, Vec<f32>>,
        tolerance: f32,
    ) {
        for (param, grads) in gradients {
            let mean = grads.iter().sum::<f32>() / grads.len() as f32;
            let variance = grads
                .iter()
                .map(|g| (g - mean).powi(2))
                .sum::<f32>() / grads.len() as f32;
            
            if variance.sqrt() > tolerance {
                panic!(
                    "Gradient inconsistency for {}: std deviation {} exceeds tolerance {}",
                    param,
                    variance.sqrt(),
                    tolerance
                );
            }
        }
    }

    /// Assert parameter synchronization
    pub fn assert_parameters_synchronized(
        node_params: &HashMap<String, HashMap<String, Vec<f32>>>,
        tolerance: f32,
    ) {
        if node_params.is_empty() {
            return;
        }

        let param_names: Vec<String> = node_params
            .values()
            .next()
            .unwrap()
            .keys()
            .cloned()
            .collect();

        for param_name in &param_names {
            let mut all_values = Vec::new();
            
            for (_, params) in node_params {
                if let Some(values) = params.get(param_name) {
                    all_values.extend(values);
                }
            }

            assert_converges!(all_values, tolerance);
        }
    }
}

/// Performance assertions
pub struct PerformanceAssertions;

impl PerformanceAssertions {
    /// Assert operation completes within time limit
    pub fn assert_latency(actual: Duration, max_allowed: Duration, operation: &str) {
        if actual > max_allowed {
            panic!(
                "{} took {:?}, exceeding limit of {:?}",
                operation, actual, max_allowed
            );
        }
    }

    /// Assert throughput meets requirements
    pub fn assert_throughput(
        operations: usize,
        duration: Duration,
        min_ops_per_sec: f64,
        operation: &str,
    ) {
        let actual_ops_per_sec = operations as f64 / duration.as_secs_f64();
        
        if actual_ops_per_sec < min_ops_per_sec {
            panic!(
                "{} throughput {:.2} ops/sec is below minimum {:.2} ops/sec",
                operation, actual_ops_per_sec, min_ops_per_sec
            );
        }
    }

    /// Assert memory usage is within bounds
    pub fn assert_memory_usage(used_bytes: usize, max_bytes: usize, component: &str) {
        if used_bytes > max_bytes {
            panic!(
                "{} memory usage {} bytes exceeds limit of {} bytes",
                component, used_bytes, max_bytes
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_assert_completes_within() {
        let result = assert_completes_within!(
            Duration::from_secs(1),
            async { 42 }
        );
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_assert_eventually() {
        let mut counter = 0;
        assert_eventually!(
            {
                counter += 1;
                counter >= 3
            },
            Duration::from_secs(1),
            Duration::from_millis(100)
        );
    }

    #[test]
    fn test_assert_converges() {
        let values = vec![1.0, 1.1, 0.9, 1.05, 0.95];
        assert_converges!(values, 0.2);
    }

    #[test]
    fn test_network_assertions() {
        let mut connections = HashMap::new();
        connections.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        connections.insert("B".to_string(), vec!["A".to_string(), "C".to_string()]);
        connections.insert("C".to_string(), vec!["A".to_string(), "B".to_string()]);
        
        NetworkAssertions::assert_fully_connected(&connections);
        NetworkAssertions::assert_no_partitions(&connections);
        NetworkAssertions::assert_min_connectivity(&connections, 2);
    }

    #[test]
    fn test_consensus_assertions() {
        let commits = vec![
            ("node1".to_string(), 1, vec![1, 2, 3]),
            ("node2".to_string(), 1, vec![1, 2, 3]),
            ("node3".to_string(), 2, vec![4, 5, 6]),
        ];
        
        ConsensusAssertions::assert_safety(&commits);
        ConsensusAssertions::assert_liveness(&[1, 2, 3, 4], 2);
    }

    #[test]
    fn test_training_assertions() {
        let losses = vec![10.0, 8.0, 6.0, 4.0, 3.0, 2.5, 2.0, 1.8, 1.5, 1.3];
        TrainingAssertions::assert_convergence(&losses, 3, 0.3);
    }
}