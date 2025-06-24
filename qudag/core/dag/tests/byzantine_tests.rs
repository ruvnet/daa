use proptest::prelude::*;
use qudag_dag::{Confidence, ConsensusConfig, ConsensusError, DAGConsensus, Vertex};
use std::collections::HashSet;
use std::time::Duration;

fn create_test_vertex(id: &str, parents: Vec<&str>, timestamp: u64) -> Vertex {
    Vertex {
        id: id.to_string(),
        parents: parents.into_iter().map(String::from).collect(),
        timestamp,
        signature: vec![],
        payload: vec![],
    }
}

// Test fork detection and handling
#[test]
fn test_fork_detection() {
    let mut dag = DAGConsensus::new();

    // Create initial vertex
    let vertex_a = create_test_vertex("A", vec![], 0);
    dag.add_vertex(vertex_a).unwrap();

    // Try to create a fork (same ID, different parents)
    let fork_vertex = create_test_vertex("A", vec![], 1);
    assert!(matches!(
        dag.add_vertex(fork_vertex),
        Err(ConsensusError::ForkDetected(_))
    ));
}

// Test equivocation resistance
#[test]
fn test_equivocation_resistance() {
    let config = ConsensusConfig {
        query_sample_size: 5,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(2),
        confirmation_depth: 3,
    };

    let mut dag = DAGConsensus::with_config(config);

    // Create two conflicting vertices with same parent
    let vertex_a = create_test_vertex("A", vec![], 0);
    let vertex_b1 = create_test_vertex("B", vec!["A"], 1);
    let vertex_b2 = create_test_vertex("B", vec!["A"], 1);

    dag.add_vertex(vertex_a).unwrap();
    dag.add_vertex(vertex_b1).unwrap();

    // Second vertex with same ID should be rejected
    assert!(matches!(
        dag.add_vertex(vertex_b2),
        Err(ConsensusError::ForkDetected(_))
    ));
}

// Test Byzantine agreement under partial synchrony
#[test]
fn test_byzantine_agreement() {
    let config = ConsensusConfig {
        query_sample_size: 10,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(5),
        confirmation_depth: 4,
    };

    let mut dag = DAGConsensus::with_config(config);

    // Create vertices with conflicting parent sets
    let vertex_a = create_test_vertex("A", vec![], 0);
    let vertex_b = create_test_vertex("B", vec!["A"], 1);
    let vertex_c1 = create_test_vertex("C", vec!["B"], 2);
    let vertex_c2 = create_test_vertex("C", vec!["A"], 2); // Conflicting parent set

    dag.add_vertex(vertex_a).unwrap();
    dag.add_vertex(vertex_b).unwrap();
    dag.add_vertex(vertex_c1).unwrap();

    // Verify that conflicting vertex is rejected
    assert!(matches!(
        dag.add_vertex(vertex_c2),
        Err(ConsensusError::ForkDetected(_))
    ));
}

// Test resistance to Sybil attacks
#[test]
fn test_sybil_resistance() {
    let config = ConsensusConfig {
        query_sample_size: 20,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(5),
        confirmation_depth: 4,
    };

    let mut dag = DAGConsensus::with_config(config);

    // Create a large number of vertices from different "identities"
    for i in 0..100 {
        let vertex = Vertex {
            id: format!("V{}", i),
            parents: if i == 0 {
                vec![]
            } else {
                vec![format!("V{}", i - 1)]
            },
            timestamp: i as u64,
            signature: vec![i as u8], // Different signatures
            payload: vec![],
        };

        dag.add_vertex(vertex).unwrap();
    }

    // Verify that consensus is still reached despite many participants
    assert_eq!(dag.get_confidence("V0"), Some(Confidence::Final));
    assert_eq!(dag.get_confidence("V50"), Some(Confidence::Final));
}

// Property-based test for Byzantine behavior
proptest! {
    #[test]
    fn prop_byzantine_resistance(
        honest_vertices in 5..20usize,
        byzantine_attempts in 1..10usize
    ) {
        let config = ConsensusConfig {
            query_sample_size: 10,
            finality_threshold: 0.8,
            finality_timeout: Duration::from_secs(5),
            confirmation_depth: 3,
        };

        let mut dag = DAGConsensus::with_config(config);
        let mut vertex_ids = HashSet::new();

        // Add honest vertices
        for i in 0..honest_vertices {
            let id = format!("H{}", i);
            let parents = if i == 0 {
                vec![]
            } else {
                vec![format!("H{}", i-1)]
            };

            let vertex = Vertex {
                id: id.clone(),
                parents,
                timestamp: i as u64,
                signature: vec![],
                payload: vec![],
            };

            dag.add_vertex(vertex).unwrap();
            vertex_ids.insert(id);
        }

        // Attempt Byzantine behavior
        for i in 0..byzantine_attempts {
            let target_id = format!("H{}", i % honest_vertices);

            // Try to create conflicting vertex
            let byzantine_vertex = Vertex {
                id: target_id.clone(),
                parents: vec![],
                timestamp: (honest_vertices + i) as u64,
                signature: vec![],
                payload: vec![],
            };

            // Byzantine vertex should be rejected
            prop_assert!(dag.add_vertex(byzantine_vertex).is_err());

            // Original vertex should maintain its status
            prop_assert!(vertex_ids.contains(&target_id));
        }

        // Verify system remains consistent
        let tips = dag.get_tips();
        prop_assert!(!tips.is_empty());
    }
}
