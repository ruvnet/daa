//! Integration tests for the qudag-dag module

use qudag_dag::*;
use std::collections::HashSet;

#[test]
fn test_module_can_be_imported() {
    // Test that the module can be imported and key types are available
    let _vertex_id = VertexId::new();
    let _consensus = QRAvalanche::new();
    let _dag = QrDag::new();

    // Test that enums can be matched
    let status = ConsensusStatus::Pending;
    match status {
        ConsensusStatus::Pending => {}
        ConsensusStatus::Accepted => {}
        ConsensusStatus::Rejected => {}
        ConsensusStatus::Final => {}
    }

    // Test that error types can be used
    let _error = DagError::NodeExists("test".to_string());

    println!("Module import test passed");
}

#[test]
fn test_dag_basic_workflow() {
    let mut dag = QrDag::new();

    // Create some test vertices
    let vertex1_id = VertexId::new();
    let vertex1 = Vertex::new(vertex1_id.clone(), b"vertex1".to_vec(), HashSet::new());

    let vertex2_id = VertexId::new();
    let mut parents = HashSet::new();
    parents.insert(vertex1_id.clone());
    let vertex2 = Vertex::new(vertex2_id.clone(), b"vertex2".to_vec(), parents);

    // Add vertices to DAG
    assert!(dag.add_vertex(vertex1).is_ok());
    assert!(dag.add_vertex(vertex2).is_ok());

    // Test getting tips
    let tips = dag.get_tips();
    assert!(!tips.is_empty());

    // Test getting confidence
    let vertex1_str = String::from_utf8_lossy(vertex1_id.as_bytes()).to_string();
    let confidence = dag.get_confidence(&vertex1_str);
    assert!(confidence.is_some());

    println!("DAG basic workflow test passed");
}

#[test]
fn test_consensus_types() {
    let mut consensus = QRAvalanche::new();

    // Test adding participants
    let participant1 = VertexId::new();
    let participant2 = VertexId::new();
    consensus.add_participant(participant1.clone());
    consensus.add_participant(participant2.clone());

    // Test processing vertex
    let vertex_id = VertexId::new();
    let result = consensus.process_vertex(vertex_id.clone());
    assert!(result.is_ok());

    // Test getting confidence
    let confidence = consensus.get_confidence(&vertex_id);
    assert!(confidence.is_some());

    // Test getting metrics
    let metrics = consensus.get_metrics();
    assert!(metrics.total_vertices_processed > 0);

    println!("Consensus types test passed");
}

#[test]
fn test_graph_operations() {
    let graph = Graph::new();

    // Test that graph can be created with default config
    let config = StorageConfig::default();
    let _graph_with_config = Graph::with_config(config);

    // Test graph metrics
    let metrics = graph.metrics();
    assert_eq!(metrics.vertices_processed, 0); // New graph should have no processed vertices

    println!("Graph operations test passed");
}

#[test]
fn test_vertex_operations() {
    let vertex_id = VertexId::new();
    let vertex_id2 = VertexId::from_bytes(b"test_vertex".to_vec());

    // Test vertex ID operations
    assert_ne!(vertex_id, vertex_id2);
    assert_eq!(vertex_id2.as_bytes(), b"test_vertex");

    // Test vertex creation
    let payload = b"test payload".to_vec();
    let parents = HashSet::new();
    let vertex = Vertex::new(vertex_id.clone(), payload.clone(), parents.clone());

    assert_eq!(vertex.id, vertex_id);
    assert_eq!(vertex.payload, payload);
    assert_eq!(vertex.parents(), parents);
    assert!(vertex.timestamp > 0);

    println!("Vertex operations test passed");
}

#[test]
fn test_tip_selection() {
    let config = TipSelectionConfig::default();
    assert!(config.tip_count > 0);
    assert!(config.max_age > 0);
    assert!(config.min_confidence >= 0.0);
    assert!(config.mcmc_walk_length > 0);

    println!("Tip selection test passed");
}

#[test]
fn test_error_handling() {
    // Test that errors can be created and converted
    let vertex_error = VertexError::InvalidParent;
    let consensus_error = ConsensusError::InvalidVertex;
    let dag_error = DagError::NodeExists("test".to_string());

    // Test error display
    assert!(!vertex_error.to_string().is_empty());
    assert!(!consensus_error.to_string().is_empty());
    assert!(!dag_error.to_string().is_empty());

    // Test error conversion
    let _dag_error_from_vertex: DagError = vertex_error.into();
    let _dag_error_from_consensus: DagError = consensus_error.into();

    println!("Error handling test passed");
}

#[test]
fn test_configuration_types() {
    // Test DAG consensus config
    let dag_config = ConsensusConfig::default();
    assert!(dag_config.query_sample_size > 0);
    assert!(dag_config.finality_threshold > 0.0);
    assert!(dag_config.finality_threshold <= 1.0);

    // Test QR-Avalanche config variants
    let default_config = QRAvalancheConfig::default();
    let fast_config = QRAvalancheConfig::fast_finality();
    let secure_config = QRAvalancheConfig::high_security();

    // Verify config relationships
    assert!(fast_config.beta < default_config.beta);
    assert!(secure_config.beta > default_config.beta);

    // Test storage config
    let storage_config = StorageConfig::default();
    assert!(storage_config.max_vertices > 0);
    assert!(storage_config.max_edges > 0);

    println!("Configuration types test passed");
}
