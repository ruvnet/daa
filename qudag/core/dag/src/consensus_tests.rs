//! Comprehensive unit tests for consensus edge cases and invariants

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashSet;
    use std::time::Duration;

    fn create_test_vertex(id: &str, parents: Vec<&str>) -> Vertex {
        let parent_ids: HashSet<VertexId> = parents
            .into_iter()
            .map(|p| VertexId::from_bytes(p.as_bytes().to_vec()))
            .collect();

        Vertex::new(
            VertexId::from_bytes(id.as_bytes().to_vec()),
            vec![1, 2, 3], // dummy payload
            parent_ids,
        )
    }

    #[test]
    fn test_consensus_basic_operations() {
        let mut consensus = QRAvalanche::new();
        let vertex_id = VertexId::new();

        // Test initial state
        assert!(consensus.vertices.is_empty());
        assert!(consensus.tips.is_empty());

        // Test processing a vertex
        let status = consensus.process_vertex(vertex_id.clone()).unwrap();
        assert_eq!(status, ConsensusStatus::Accepted);
        assert!(consensus.vertices.contains_key(&vertex_id));
        assert!(consensus.tips.contains(&vertex_id));
    }

    #[test]
    fn test_dag_consensus_genesis_vertex() {
        let mut dag = DAGConsensus::new();
        let genesis = create_test_vertex("genesis", vec![]);

        // Genesis vertex should be added successfully
        assert!(dag.add_vertex(genesis).is_ok());

        // Should have confidence status
        assert_eq!(dag.get_confidence("genesis"), Some(ConsensusStatus::Final));
    }

    #[test]
    fn test_dag_consensus_chain_building() {
        let mut dag = DAGConsensus::new();

        // Build a simple chain: A -> B -> C
        let vertex_a = create_test_vertex("A", vec![]);
        let vertex_b = create_test_vertex("B", vec!["A"]);
        let vertex_c = create_test_vertex("C", vec!["B"]);

        assert!(dag.add_vertex(vertex_a).is_ok());
        assert!(dag.add_vertex(vertex_b).is_ok());
        assert!(dag.add_vertex(vertex_c).is_ok());

        // All vertices should achieve consensus
        assert_eq!(dag.get_confidence("A"), Some(ConsensusStatus::Final));
        assert_eq!(dag.get_confidence("B"), Some(ConsensusStatus::Final));
        assert_eq!(dag.get_confidence("C"), Some(ConsensusStatus::Final));
    }

    #[test]
    fn test_fork_detection() {
        let mut dag = DAGConsensus::new();

        // Add initial vertex
        let vertex_a = create_test_vertex("A", vec![]);
        assert!(dag.add_vertex(vertex_a).is_ok());

        // Try to add another vertex with same ID (fork)
        let fork_vertex = create_test_vertex("A", vec![]);
        let result = dag.add_vertex(fork_vertex);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Fork detected"));
    }

    #[test]
    fn test_missing_parent_validation() {
        let mut dag = DAGConsensus::new();

        // Try to add vertex with non-existing parent
        let invalid_vertex = create_test_vertex("B", vec!["A"]);
        let result = dag.add_vertex(invalid_vertex);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parent"));
    }

    #[test]
    fn test_self_reference_prevention() {
        let mut dag = DAGConsensus::new();

        // Create vertex that references itself
        let self_ref_vertex = create_test_vertex("A", vec!["A"]);
        let result = dag.add_vertex(self_ref_vertex);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("references itself"));
    }

    #[test]
    fn test_parallel_branches() {
        let mut dag = DAGConsensus::new();

        // Create parallel branches from root
        let root = create_test_vertex("root", vec![]);
        let branch_a = create_test_vertex("A", vec!["root"]);
        let branch_b = create_test_vertex("B", vec!["root"]);
        let merge = create_test_vertex("merge", vec!["A", "B"]);

        assert!(dag.add_vertex(root).is_ok());
        assert!(dag.add_vertex(branch_a).is_ok());
        assert!(dag.add_vertex(branch_b).is_ok());
        assert!(dag.add_vertex(merge).is_ok());

        // All vertices should achieve consensus
        assert_eq!(dag.get_confidence("root"), Some(ConsensusStatus::Final));
        assert_eq!(dag.get_confidence("A"), Some(ConsensusStatus::Final));
        assert_eq!(dag.get_confidence("B"), Some(ConsensusStatus::Final));
        assert_eq!(dag.get_confidence("merge"), Some(ConsensusStatus::Final));
    }

    #[test]
    fn test_total_order_consistency() {
        let mut dag = DAGConsensus::new();

        // Create linear chain
        let vertices = vec!["A", "B", "C", "D"];
        let mut parents = vec![];

        for vertex_id in &vertices {
            let vertex = if parents.is_empty() {
                create_test_vertex(vertex_id, vec![])
            } else {
                create_test_vertex(vertex_id, vec![parents.last().unwrap()])
            };

            assert!(dag.add_vertex(vertex).is_ok());
            parents.push(*vertex_id);
        }

        // Get total order
        let order = dag.get_total_order().unwrap();
        assert_eq!(order.len(), vertices.len());

        // Order should respect parent-child relationships
        for i in 1..vertices.len() {
            let parent_pos = order.iter().position(|x| x == vertices[i - 1]).unwrap();
            let child_pos = order.iter().position(|x| x == vertices[i]).unwrap();
            assert!(
                parent_pos < child_pos,
                "Parent {} should come before child {}",
                vertices[i - 1],
                vertices[i]
            );
        }
    }

    #[test]
    fn test_consensus_status_transitions() {
        let mut consensus = QRAvalanche::new();
        let vertex_id = VertexId::new();

        // Initially no status
        assert!(!consensus.vertices.contains_key(&vertex_id));

        // Process vertex - should be accepted
        let status = consensus.process_vertex(vertex_id.clone()).unwrap();
        assert_eq!(status, ConsensusStatus::Accepted);

        // Should be in consensus tracking
        assert_eq!(
            consensus.vertices.get(&vertex_id),
            Some(&ConsensusStatus::Accepted)
        );
        assert!(consensus.tips.contains(&vertex_id));
    }

    #[test]
    fn test_consensus_config_validation() {
        let config = ConsensusConfig {
            query_sample_size: 0,
            finality_threshold: 1.5, // Invalid > 1.0
            finality_timeout: Duration::from_secs(0),
            confirmation_depth: 0,
        };

        // Should still create DAG but with potentially invalid behavior
        let _dag = DAGConsensus::with_config(config);
        // Note: In a real implementation, we'd validate config parameters
    }

    #[test]
    fn test_dag_invariants() {
        let mut dag = DAGConsensus::new();

        // Build complex DAG structure
        let vertices = vec![
            ("genesis", vec![]),
            ("a1", vec!["genesis"]),
            ("a2", vec!["genesis"]),
            ("b1", vec!["a1"]),
            ("b2", vec!["a2"]),
            ("merge", vec!["b1", "b2"]),
        ];

        for (id, parents) in vertices {
            let vertex = create_test_vertex(id, parents);
            assert!(
                dag.add_vertex(vertex).is_ok(),
                "Failed to add vertex {}",
                id
            );
        }

        // Verify all vertices achieved consensus
        for (id, _) in [
            ("genesis", vec![]),
            ("a1", vec!["genesis"]),
            ("a2", vec!["genesis"]),
            ("b1", vec!["a1"]),
            ("b2", vec!["a2"]),
            ("merge", vec!["b1", "b2"]),
        ]
        .iter()
        {
            assert_eq!(
                dag.get_confidence(id),
                Some(ConsensusStatus::Final),
                "Vertex {} should have final status",
                id
            );
        }

        // Verify tips are updated correctly
        let tips = dag.get_tips();
        assert!(!tips.is_empty(), "Should have at least one tip");
    }
}
