//! Tests to verify that all important types are properly exported from the module

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashSet;

    #[test]
    fn test_core_types_are_exported() {
        // Test that we can create and use key types

        // Vertex types
        let vertex_id = VertexId::new();
        let vertex = Vertex::new(vertex_id.clone(), vec![1, 2, 3], HashSet::new());
        assert!(vertex.id == vertex_id);

        // DAG types
        let dag = QrDag::new();
        assert!(dag.get_tips().is_empty()); // New DAG should have no tips initially

        // Consensus types
        let consensus = QRAvalanche::new();
        assert!(consensus.vertices.is_empty());

        // Configuration types
        let config = ConsensusConfig::default();
        assert!(config.query_sample_size > 0);
        assert!(config.finality_threshold > 0.0);
        assert!(config.finality_threshold <= 1.0);

        // Consensus status
        let status = ConsensusStatus::Pending;
        assert!(matches!(status, ConsensusStatus::Pending));

        // Error types should be available
        let _vertex_error: VertexError = VertexError::InvalidParent;
        let _consensus_error: ConsensusError = ConsensusError::InvalidVertex;
        let _dag_error: DagError = DagError::NodeExists("test".to_string());

        // Node types
        let _node = Node::new(vertex.payload.clone(), vec![]);

        // Graph and storage types
        let _storage_config = StorageConfig::default();
        let _graph = Graph::new();

        // Tip selection types
        let _tip_config = TipSelectionConfig::default();

        // QRAvalanche config
        let _qr_config = QRAvalancheConfig::default();

        println!("All core types are properly exported and functional");
    }

    #[test]
    fn test_dag_basic_operations() {
        let mut dag = QrDag::new();

        // Test adding a message
        let message = b"Hello, DAG!".to_vec();
        let result = dag.add_message(message.clone());
        assert!(result.is_ok(), "Should be able to add message to DAG");

        // Test checking if message exists
        assert!(
            dag.contains_message(&message),
            "DAG should contain the added message"
        );

        // Test getting tips
        let tips = dag.get_tips();
        assert!(
            !tips.is_empty(),
            "DAG should have at least one tip after adding a message"
        );

        println!("Basic DAG operations work correctly");
    }

    #[test]
    fn test_consensus_configuration() {
        // Test different consensus configurations
        let default_config = ConsensusConfig::default();
        let qr_config = QRAvalancheConfig::default();
        let fast_config = QRAvalancheConfig::fast_finality();
        let secure_config = QRAvalancheConfig::high_security();

        // Verify default values are reasonable
        assert!(default_config.query_sample_size >= 1);
        assert!(default_config.finality_threshold > 0.0);
        assert!(default_config.finality_threshold <= 1.0);

        // Verify QR-Avalanche configurations
        assert!(qr_config.beta > 0.5);
        assert!(qr_config.alpha > 0.0);
        assert!(qr_config.query_sample_size > 0);

        // Verify fast finality has appropriate settings
        assert!(fast_config.beta < qr_config.beta);
        assert!(fast_config.alpha < qr_config.alpha);
        assert!(fast_config.query_sample_size <= qr_config.query_sample_size);

        // Verify high security has appropriate settings
        assert!(secure_config.beta > qr_config.beta);
        assert!(secure_config.alpha > qr_config.alpha);
        assert!(secure_config.query_sample_size >= qr_config.query_sample_size);

        println!("Consensus configurations are properly structured");
    }

    #[test]
    fn test_error_types() {
        // Test that error types can be created and match properly

        let vertex_errors = vec![
            VertexError::InvalidParent,
            VertexError::ParentNotFound,
            VertexError::InvalidPayload,
            VertexError::InvalidSignature,
            VertexError::CreationFailed,
        ];

        for error in vertex_errors {
            assert!(!error.to_string().is_empty());
        }

        let consensus_errors = vec![
            ConsensusError::InvalidVertex,
            ConsensusError::ConflictingVertices,
            ConsensusError::ConsensusFailure,
            ConsensusError::InvalidState,
            ConsensusError::InsufficientVotes,
            ConsensusError::Timeout,
        ];

        for error in consensus_errors {
            assert!(!error.to_string().is_empty());
        }

        println!("Error types are properly defined and functional");
    }

    #[test]
    fn test_vertex_operations() {
        let vertex_id = VertexId::new();
        let payload = vec![1, 2, 3, 4, 5];
        let parents = HashSet::new();

        let vertex = Vertex::new(vertex_id.clone(), payload.clone(), parents.clone());

        assert_eq!(vertex.id, vertex_id);
        assert_eq!(vertex.payload, payload);
        assert_eq!(vertex.parents(), parents);
        assert!(vertex.timestamp > 0);

        // Test vertex ID operations
        let vertex_id2 = VertexId::from_bytes(b"test".to_vec());
        assert_ne!(vertex_id, vertex_id2);
        assert_eq!(vertex_id2.as_bytes(), b"test");

        println!("Vertex operations work correctly");
    }
}
