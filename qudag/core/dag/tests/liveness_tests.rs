use qudag_dag::{Confidence, ConsensusConfig, DAGConsensus, Vertex};
use std::thread;
use std::time::Duration;
use tokio_test::block_on;

fn create_test_vertex(id: &str, parents: Vec<&str>, timestamp: u64) -> Vertex {
    Vertex {
        id: id.to_string(),
        parents: parents.into_iter().map(String::from).collect(),
        timestamp,
        signature: vec![],
        payload: vec![],
    }
}

// Test termination property - vertices eventually reach finality
#[test]
fn test_termination() {
    let config = ConsensusConfig {
        query_sample_size: 5,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(5),
        confirmation_depth: 3,
    };

    let mut dag = DAGConsensus::with_config(config);

    // Add sequence of vertices
    let vertex_a = create_test_vertex("A", vec![], 0);
    let vertex_b = create_test_vertex("B", vec!["A"], 1);
    let vertex_c = create_test_vertex("C", vec!["B"], 2);

    dag.add_vertex(vertex_a).unwrap();
    dag.add_vertex(vertex_b).unwrap();
    dag.add_vertex(vertex_c).unwrap();

    // Wait for consensus to complete
    thread::sleep(Duration::from_millis(100));

    // Verify all vertices reached finality
    assert_eq!(dag.get_confidence("A"), Some(Confidence::Final));
    assert_eq!(dag.get_confidence("B"), Some(Confidence::Final));
    assert_eq!(dag.get_confidence("C"), Some(Confidence::Final));
}

// Test progress property - system continues to make progress
#[test]
fn test_progress() {
    let config = ConsensusConfig {
        query_sample_size: 5,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(5),
        confirmation_depth: 3,
    };

    let mut dag = DAGConsensus::with_config(config);
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(2);

    // Add vertices continuously
    let mut vertex_count = 0;
    while start_time.elapsed() < timeout {
        let id = format!("V{}", vertex_count);
        let parents = if vertex_count == 0 {
            vec![]
        } else {
            vec![format!("V{}", vertex_count - 1)]
        };

        let vertex = Vertex {
            id,
            parents,
            timestamp: vertex_count as u64,
            signature: vec![],
            payload: vec![],
        };

        dag.add_vertex(vertex).unwrap();
        vertex_count += 1;

        thread::sleep(Duration::from_millis(100));
    }

    // Verify system made progress
    assert!(vertex_count > 5, "System should process multiple vertices");

    // Verify earlier vertices reached finality
    assert_eq!(
        dag.get_confidence(&format!("V{}", 0)),
        Some(Confidence::Final)
    );
}

// Test concurrent progress - multiple paths can progress simultaneously
#[test]
fn test_concurrent_progress() {
    let config = ConsensusConfig {
        query_sample_size: 5,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(5),
        confirmation_depth: 3,
    };

    let mut dag = DAGConsensus::with_config(config);

    // Create two parallel chains
    let vertex_a = create_test_vertex("A", vec![], 0);
    let vertex_b1 = create_test_vertex("B1", vec!["A"], 1);
    let vertex_b2 = create_test_vertex("B2", vec!["A"], 1);
    let vertex_c1 = create_test_vertex("C1", vec!["B1"], 2);
    let vertex_c2 = create_test_vertex("C2", vec!["B2"], 2);

    // Add vertices
    dag.add_vertex(vertex_a).unwrap();
    dag.add_vertex(vertex_b1).unwrap();
    dag.add_vertex(vertex_b2).unwrap();
    dag.add_vertex(vertex_c1).unwrap();
    dag.add_vertex(vertex_c2).unwrap();

    // Wait for consensus
    thread::sleep(Duration::from_millis(200));

    // Verify both chains made progress
    assert_eq!(dag.get_confidence("C1"), Some(Confidence::Final));
    assert_eq!(dag.get_confidence("C2"), Some(Confidence::Final));
}

// Test no-deadlock property
#[test]
fn test_no_deadlock() {
    let config = ConsensusConfig {
        query_sample_size: 5,
        finality_threshold: 0.8,
        finality_timeout: Duration::from_secs(1),
        confirmation_depth: 3,
    };

    let mut dag = DAGConsensus::with_config(config);

    // Create complex DAG structure with multiple paths
    let vertices = vec![
        create_test_vertex("A", vec![], 0),
        create_test_vertex("B1", vec!["A"], 1),
        create_test_vertex("B2", vec!["A"], 1),
        create_test_vertex("C1", vec!["B1", "B2"], 2),
        create_test_vertex("C2", vec!["B1"], 2),
        create_test_vertex("D", vec!["C1", "C2"], 3),
    ];

    // Add all vertices
    for vertex in vertices {
        dag.add_vertex(vertex).unwrap();
    }

    // Wait for consensus
    thread::sleep(Duration::from_millis(300));

    // Verify system didn't deadlock
    assert_eq!(dag.get_confidence("D"), Some(Confidence::Final));

    // Verify we can still add new vertices
    let vertex_e = create_test_vertex("E", vec!["D"], 4);
    assert!(dag.add_vertex(vertex_e).is_ok());
}
