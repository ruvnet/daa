use proptest::prelude::*;
use qudag_dag::{Confidence, ConsensusConfig, ConsensusError, DAGConsensus, Vertex};
use std::time::Duration;

fn create_test_vertex(id: &str, parents: Vec<&str>) -> Vertex {
    Vertex {
        id: id.to_string(),
        parents: parents.into_iter().map(String::from).collect(),
        timestamp: 0,
        signature: vec![],
        payload: vec![],
    }
}

// Test Total Order Property
#[test]
fn test_total_order() {
    let mut dag = DAGConsensus::new();

    // Create a simple chain: A -> B -> C
    let vertex_a = create_test_vertex("A", vec![]);
    let vertex_b = create_test_vertex("B", vec!["A"]);
    let vertex_c = create_test_vertex("C", vec!["B"]);

    dag.add_vertex(vertex_a).unwrap();
    dag.add_vertex(vertex_b).unwrap();
    dag.add_vertex(vertex_c).unwrap();

    // Verify that vertices are ordered correctly
    let order = dag.get_total_order().unwrap();
    assert_eq!(order, vec!["A", "B", "C"]);
}

// Test Agreement Property
#[test]
fn test_agreement() {
    let config = ConsensusConfig {
        query_sample_size: 5,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(2),
        confirmation_depth: 3,
    };

    let mut dag1 = DAGConsensus::with_config(config.clone());
    let mut dag2 = DAGConsensus::with_config(config);

    // Create identical vertices in both DAGs
    let vertex_a = create_test_vertex("A", vec![]);
    let vertex_b = create_test_vertex("B", vec!["A"]);

    // Add to both DAGs
    dag1.add_vertex(vertex_a.clone()).unwrap();
    dag1.add_vertex(vertex_b.clone()).unwrap();

    dag2.add_vertex(vertex_a).unwrap();
    dag2.add_vertex(vertex_b).unwrap();

    // Both DAGs should reach the same final state
    assert_eq!(dag1.get_confidence("B"), dag2.get_confidence("B"));
}

// Test Validity Property
#[test]
fn test_validity() {
    let mut dag = DAGConsensus::new();

    // Create valid vertex
    let vertex_a = create_test_vertex("A", vec![]);
    assert!(dag.add_vertex(vertex_a).is_ok());

    // Try to add vertex with non-existent parent
    let invalid_vertex = create_test_vertex("invalid", vec!["nonexistent"]);
    assert!(matches!(
        dag.add_vertex(invalid_vertex),
        Err(ConsensusError::InvalidVertex(_))
    ));

    // Try to add vertex that creates a cycle
    let cycle_vertex = create_test_vertex("A", vec!["A"]);
    assert!(matches!(
        dag.add_vertex(cycle_vertex),
        Err(ConsensusError::ValidationError(_))
    ));
}

// Property-based test for safety properties
proptest! {
    #[test]
    fn prop_total_order_consistency(
        vertex_count in 2..10usize,
        parent_probability in 0.1..0.5f64
    ) {
        let mut dag = DAGConsensus::new();
        let mut vertices = Vec::new();

        // Add vertices with random parent relationships
        for i in 0..vertex_count {
            let id = format!("V{}", i);
            let mut parents = Vec::new();

            // Randomly select parents from existing vertices
            for j in 0..i {
                if rand::random::<f64>() < parent_probability {
                    parents.push(format!("V{}", j));
                }
            }

            let vertex = Vertex {
                id: id.clone(),
                parents,
                timestamp: i as u64,
                signature: vec![],
                payload: vec![],
            };

            vertices.push(vertex.clone());
            dag.add_vertex(vertex).unwrap();
        }

        // Verify total order properties
        let order = dag.get_total_order().unwrap();
        prop_assert!(order.len() == vertex_count);

        // Verify that parents come before children
        for vertex in &vertices {
            let vertex_idx = order.iter().position(|id| id == &vertex.id).unwrap();
            for parent in &vertex.parents {
                let parent_idx = order.iter().position(|id| id == parent).unwrap();
                prop_assert!(parent_idx < vertex_idx);
            }
        }
    }
}
