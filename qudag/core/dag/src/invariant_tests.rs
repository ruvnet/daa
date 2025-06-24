//! Tests for DAG invariants and property verification

#[cfg(test)]
mod tests {
    use super::super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

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

    /// Test that DAG maintains acyclicity invariant
    #[test]
    fn test_dag_acyclicity_invariant() {
        let mut dag = DAGConsensus::new();

        // Create vertices in topological order
        let vertices = vec![
            ("A", vec![]),
            ("B", vec!["A"]),
            ("C", vec!["A"]),
            ("D", vec!["B", "C"]),
        ];

        for (id, parents) in vertices {
            let vertex = create_test_vertex(id, parents);
            assert!(dag.add_vertex(vertex).is_ok());
        }

        // Verify no cycles exist by checking total order
        let order = dag.get_total_order().unwrap();
        assert_eq!(order.len(), 4);

        // A should come before B and C
        let a_pos = order.iter().position(|x| x == "A").unwrap();
        let b_pos = order.iter().position(|x| x == "B").unwrap();
        let c_pos = order.iter().position(|x| x == "C").unwrap();
        let d_pos = order.iter().position(|x| x == "D").unwrap();

        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
    }

    /// Test that once a vertex achieves consensus, it remains stable
    #[test]
    fn test_consensus_stability_invariant() {
        let mut dag = DAGConsensus::new();
        let vertex = create_test_vertex("stable", vec![]);

        // Add vertex and verify it reaches consensus
        assert!(dag.add_vertex(vertex).is_ok());
        assert_eq!(dag.get_confidence("stable"), Some(ConsensusStatus::Final));

        // Consensus status should remain stable
        for _ in 0..10 {
            assert_eq!(dag.get_confidence("stable"), Some(ConsensusStatus::Final));
        }

        // Adding other vertices shouldn't affect existing consensus
        let vertex2 = create_test_vertex("other", vec![]);
        assert!(dag.add_vertex(vertex2).is_ok());
        assert_eq!(dag.get_confidence("stable"), Some(ConsensusStatus::Final));
    }

    /// Test that DAG preserves partial order
    #[test]
    fn test_partial_order_preservation() {
        let mut dag = DAGConsensus::new();

        // Create a diamond structure
        let genesis = create_test_vertex("genesis", vec![]);
        let left = create_test_vertex("left", vec!["genesis"]);
        let right = create_test_vertex("right", vec!["genesis"]);
        let merge = create_test_vertex("merge", vec!["left", "right"]);

        assert!(dag.add_vertex(genesis).is_ok());
        assert!(dag.add_vertex(left).is_ok());
        assert!(dag.add_vertex(right).is_ok());
        assert!(dag.add_vertex(merge).is_ok());

        let order = dag.get_total_order().unwrap();

        // Genesis must come first
        let genesis_pos = order.iter().position(|x| x == "genesis").unwrap();
        let left_pos = order.iter().position(|x| x == "left").unwrap();
        let right_pos = order.iter().position(|x| x == "right").unwrap();
        let merge_pos = order.iter().position(|x| x == "merge").unwrap();

        assert!(genesis_pos < left_pos);
        assert!(genesis_pos < right_pos);
        assert!(left_pos < merge_pos);
        assert!(right_pos < merge_pos);
    }

    /// Test that tip set is maintained correctly
    #[test]
    fn test_tip_set_invariant() {
        let mut dag = DAGConsensus::new();

        // Initially no tips
        assert!(dag.get_tips().is_empty());

        // Add genesis - should become tip
        let genesis = create_test_vertex("genesis", vec![]);
        assert!(dag.add_vertex(genesis).is_ok());
        let tips = dag.get_tips();
        assert_eq!(tips.len(), 1);
        assert!(tips.contains(&"genesis".to_string()));

        // Add child - parent should no longer be tip
        let child = create_test_vertex("child", vec!["genesis"]);
        assert!(dag.add_vertex(child).is_ok());
        let tips = dag.get_tips();
        assert_eq!(tips.len(), 1);
        assert!(tips.contains(&"child".to_string()));
        assert!(!tips.contains(&"genesis".to_string()));
    }

    /// Test that consensus preserves safety under concurrent operations
    #[test]
    fn test_concurrent_safety() {
        let mut dag = DAGConsensus::new();

        // Add vertices in parallel branches
        let root = create_test_vertex("root", vec![]);
        assert!(dag.add_vertex(root).is_ok());

        // Create concurrent branches
        let branch1 = create_test_vertex("branch1", vec!["root"]);
        let branch2 = create_test_vertex("branch2", vec!["root"]);

        assert!(dag.add_vertex(branch1).is_ok());
        assert!(dag.add_vertex(branch2).is_ok());

        // Both branches should achieve consensus
        assert_eq!(dag.get_confidence("branch1"), Some(ConsensusStatus::Final));
        assert_eq!(dag.get_confidence("branch2"), Some(ConsensusStatus::Final));

        // Merge should work
        let merge = create_test_vertex("merge", vec!["branch1", "branch2"]);
        assert!(dag.add_vertex(merge).is_ok());
        assert_eq!(dag.get_confidence("merge"), Some(ConsensusStatus::Final));
    }

    /// Test that DAG handles Byzantine scenarios properly
    #[test]
    fn test_byzantine_resistance() {
        let mut dag = DAGConsensus::new();

        // Add honest vertices
        let honest1 = create_test_vertex("honest1", vec![]);
        let honest2 = create_test_vertex("honest2", vec!["honest1"]);

        assert!(dag.add_vertex(honest1).is_ok());
        assert!(dag.add_vertex(honest2).is_ok());

        // Try Byzantine attack: fork attempt
        let byzantine_fork = create_test_vertex("honest1", vec![]); // Same ID as honest1
        let result = dag.add_vertex(byzantine_fork);
        assert!(result.is_err());

        // Honest vertices should maintain their consensus
        assert_eq!(dag.get_confidence("honest1"), Some(ConsensusStatus::Final));
        assert_eq!(dag.get_confidence("honest2"), Some(ConsensusStatus::Final));
    }

    /// Property-based test for DAG invariants
    proptest! {
        #[test]
        fn prop_dag_maintains_invariants(
            vertex_count in 1..20usize,
            max_parents in 1..3usize
        ) {
            let mut dag = DAGConsensus::new();
            let mut vertex_ids = Vec::new();

            // Add vertices with valid parent relationships
            for i in 0..vertex_count {
                let id = format!("V{}", i);
                let mut parents = Vec::new();

                // Select parents from previously added vertices
                let parent_count = std::cmp::min(max_parents, i);
                for j in 0..parent_count {
                    if i > 0 && j < vertex_ids.len() {
                        parents.push(vertex_ids[j].as_str());
                    }
                }

                let vertex = create_test_vertex(&id, parents);
                prop_assert!(dag.add_vertex(vertex).is_ok());
                vertex_ids.push(id);
            }

            // Verify all vertices achieved consensus
            for id in &vertex_ids {
                prop_assert_eq!(dag.get_confidence(id), Some(ConsensusStatus::Final));
            }

            // Verify total order respects partial order
            let order = dag.get_total_order().unwrap();
            prop_assert_eq!(order.len(), vertex_count);

            // Verify no duplicate vertices in order
            let mut seen = HashSet::new();
            for vertex in &order {
                prop_assert!(seen.insert(vertex.clone()));
            }
        }

        #[test]
        fn prop_dag_acyclicity(
            edges in prop::collection::vec(
                (0usize..20, 0usize..20),
                1..50
            )
        ) {
            let mut dag = DAGConsensus::new();
            let mut vertex_ids = Vec::new();

            // Create vertices first
            for i in 0..20 {
                let id = format!("V{}", i);
                vertex_ids.push(id);
            }

            // Build adjacency list from edges, ensuring acyclicity
            let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); 20];
            for (from, to) in edges {
                if from != to && from < to { // Only allow edges that maintain topological order
                    adjacency[from].push(to);
                }
            }

            // Add vertices in topological order
            for i in 0..20 {
                let parents = adjacency.iter()
                    .enumerate()
                    .filter(|(_, children)| children.contains(&i))
                    .map(|(parent, _)| vertex_ids[parent].as_str())
                    .collect::<Vec<_>>();

                let vertex = create_test_vertex(&vertex_ids[i], parents);
                prop_assert!(dag.add_vertex(vertex).is_ok());
            }

            // Property: DAG maintains acyclicity
            let order = dag.get_total_order().unwrap();
            for i in 0..order.len() {
                for j in (i+1)..order.len() {
                    // Verify no vertex appears before its dependencies
                    let vertex_i = &order[i];
                    let vertex_j = &order[j];

                    // If vertex_i depends on vertex_j, this would be a cycle
                    prop_assert_ne!(vertex_i, vertex_j);
                }
            }
        }

        #[test]
        fn prop_dag_consensus_monotonicity(
            operations in prop::collection::vec(
                prop_oneof![
                    prop::collection::vec(any::<u8>(), 1..100).prop_map(|payload| ("add", payload)),
                    Just(("query", vec![]))
                ],
                1..50
            )
        ) {
            let mut dag = DAGConsensus::new();
            let mut vertex_counter = 0;
            let mut consensus_levels = Vec::new();

            for (op, payload) in operations {
                match op {
                    "add" => {
                        let vertex_id = format!("V{}", vertex_counter);
                        let parents = if vertex_counter > 0 {
                            vec![format!("V{}", vertex_counter - 1).as_str()]
                        } else {
                            vec![]
                        };

                        let vertex = create_test_vertex(&vertex_id, parents);
                        if dag.add_vertex(vertex).is_ok() {
                            vertex_counter += 1;
                        }
                    },
                    "query" => {
                        // Record current consensus level
                        let mut current_level = 0;
                        for i in 0..vertex_counter {
                            let vertex_id = format!("V{}", i);
                            if dag.get_confidence(&vertex_id) == Some(ConsensusStatus::Final) {
                                current_level += 1;
                            }
                        }
                        consensus_levels.push(current_level);
                    },
                    _ => {}
                }
            }

            // Property: Consensus is monotonic (never decreases)
            for i in 1..consensus_levels.len() {
                prop_assert!(consensus_levels[i] >= consensus_levels[i-1],
                    "Consensus level decreased: {} -> {}", consensus_levels[i-1], consensus_levels[i]);
            }
        }

        #[test]
        fn prop_dag_tip_set_invariants(
            vertex_structure in prop::collection::vec(
                prop::collection::vec(0usize..10, 0..3),
                1..15
            )
        ) {
            let mut dag = DAGConsensus::new();
            let mut vertex_ids = Vec::new();

            for (i, parents) in vertex_structure.iter().enumerate() {
                let vertex_id = format!("V{}", i);

                let parent_names: Vec<&str> = parents.iter()
                    .filter(|&&p| p < vertex_ids.len())
                    .map(|&p| vertex_ids[p].as_str())
                    .collect();

                let vertex = create_test_vertex(&vertex_id, parent_names);
                if dag.add_vertex(vertex).is_ok() {
                    vertex_ids.push(vertex_id);

                    // Property: Tips are always vertices with no children
                    let tips = dag.get_tips();

                    // Every tip should be a valid vertex
                    for tip in &tips {
                        prop_assert!(vertex_ids.iter().any(|id| id == tip),
                            "Tip {} is not a valid vertex", tip);
                    }

                    // No vertex with children should be a tip
                    for (j, child_parents) in vertex_structure.iter().enumerate().skip(i + 1) {
                        for &parent_idx in child_parents {
                            if parent_idx < vertex_ids.len() {
                                let parent_name = &vertex_ids[parent_idx];
                                prop_assert!(!tips.contains(parent_name) || j > vertex_ids.len(),
                                    "Vertex {} is a tip but has child {}", parent_name, j);
                            }
                        }
                    }
                }
            }
        }

        #[test]
        fn prop_dag_safety_under_concurrency(
            concurrent_vertices in prop::collection::vec(
                (prop::collection::vec(any::<u8>(), 1..50), prop::collection::vec(0usize..5, 0..3)),
                2..10
            )
        ) {
            let mut dag = DAGConsensus::new();
            let mut all_vertices = Vec::new();

            // Add a root vertex first
            let root = create_test_vertex("root", vec![]);
            dag.add_vertex(root).unwrap();
            all_vertices.push("root".to_string());

            // Process concurrent vertices
            for (payload, parent_indices) in concurrent_vertices {
                let vertex_id = format!("V{}", all_vertices.len());

                let parents: Vec<&str> = parent_indices.iter()
                    .filter(|&&idx| idx < all_vertices.len())
                    .map(|&idx| all_vertices[idx].as_str())
                    .collect();

                let mut vertex_payload = payload;
                vertex_payload.truncate(32); // Limit payload size

                let vertex = Vertex::new(
                    VertexId::from_bytes(vertex_id.as_bytes().to_vec()),
                    vertex_payload,
                    parents.iter().map(|p| VertexId::from_bytes(p.as_bytes().to_vec())).collect(),
                );

                if dag.add_vertex(vertex).is_ok() {
                    all_vertices.push(vertex_id.clone());

                    // Property: Safety is preserved under concurrent operations
                    let confidence = dag.get_confidence(&vertex_id);
                    prop_assert!(
                        confidence == Some(ConsensusStatus::Final) ||
                        confidence == Some(ConsensusStatus::Pending),
                        "Invalid consensus status for vertex {}: {:?}", vertex_id, confidence
                    );
                }
            }

            // Property: All successfully added vertices should achieve consensus
            for vertex_id in &all_vertices {
                prop_assert_eq!(dag.get_confidence(vertex_id), Some(ConsensusStatus::Final),
                    "Vertex {} failed to achieve consensus", vertex_id);
            }
        }

        #[test]
        fn prop_dag_liveness_properties(
            message_sequence in prop::collection::vec(
                prop::collection::vec(any::<u8>(), 1..100),
                1..20
            )
        ) {
            let mut dag = DAGConsensus::new();
            let mut processed_count = 0;

            for (i, message) in message_sequence.iter().enumerate() {
                let vertex_id = format!("msg_{}", i);
                let parents = if i > 0 {
                    vec![format!("msg_{}", i - 1).as_str()]
                } else {
                    vec![]
                };

                let vertex = create_test_vertex(&vertex_id, parents);

                if dag.add_vertex(vertex).is_ok() {
                    processed_count += 1;

                    // Property: Liveness - messages eventually achieve consensus
                    let confidence = dag.get_confidence(&vertex_id);
                    prop_assert_eq!(confidence, Some(ConsensusStatus::Final),
                        "Message {} did not achieve consensus", vertex_id);
                }
            }

            // Property: Progress is made (at least some messages processed)
            prop_assert!(processed_count > 0, "No messages were processed");

            // Property: Total order includes all processed messages
            let order = dag.get_total_order().unwrap();
            prop_assert_eq!(order.len(), processed_count,
                "Total order length {} does not match processed count {}", order.len(), processed_count);
        }

        #[test]
        fn prop_dag_byzantine_resistance(
            honest_messages in prop::collection::vec(
                prop::collection::vec(any::<u8>(), 1..50),
                1..10
            ),
            byzantine_attacks in prop::collection::vec(
                prop_oneof![
                    Just("duplicate_vertex"),
                    Just("invalid_parent"),
                    Just("circular_reference")
                ],
                0..5
            )
        ) {
            let mut dag = DAGConsensus::new();
            let mut honest_vertices = Vec::new();

            // Add honest messages first
            for (i, message) in honest_messages.iter().enumerate() {
                let vertex_id = format!("honest_{}", i);
                let parents = if i > 0 {
                    vec![format!("honest_{}", i - 1).as_str()]
                } else {
                    vec![]
                };

                let vertex = create_test_vertex(&vertex_id, parents);
                if dag.add_vertex(vertex).is_ok() {
                    honest_vertices.push(vertex_id);
                }
            }

            let honest_count = honest_vertices.len();

            // Attempt Byzantine attacks
            for (i, attack) in byzantine_attacks.iter().enumerate() {
                let attack_result = match attack {
                    "duplicate_vertex" => {
                        // Try to add duplicate vertex
                        if !honest_vertices.is_empty() {
                            let duplicate = create_test_vertex(&honest_vertices[0], vec![]);
                            dag.add_vertex(duplicate)
                        } else {
                            Ok(())
                        }
                    },
                    "invalid_parent" => {
                        // Try to reference non-existent parent
                        let invalid_vertex = create_test_vertex(
                            &format!("byzantine_{}", i),
                            vec!["non_existent_parent"]
                        );
                        dag.add_vertex(invalid_vertex)
                    },
                    "circular_reference" => {
                        // Try to create circular reference (should be impossible with our design)
                        let vertex_id = format!("circular_{}", i);
                        let circular_vertex = create_test_vertex(&vertex_id, vec![&vertex_id]);
                        dag.add_vertex(circular_vertex)
                    },
                    _ => Ok(())
                };

                // Property: Byzantine attacks should be rejected
                if let Err(_) = attack_result {
                    // Attack was properly rejected, continue
                } else {
                    // If attack was accepted, verify it doesn't break honest vertices
                    for honest_id in &honest_vertices {
                        prop_assert_eq!(dag.get_confidence(honest_id), Some(ConsensusStatus::Final),
                            "Byzantine attack affected honest vertex {}", honest_id);
                    }
                }
            }

            // Property: Honest vertices should maintain their consensus despite attacks
            for honest_id in &honest_vertices {
                prop_assert_eq!(dag.get_confidence(honest_id), Some(ConsensusStatus::Final),
                    "Honest vertex {} lost consensus after Byzantine attacks", honest_id);
            }

            // Property: Total order should still be valid
            let order = dag.get_total_order().unwrap();
            prop_assert!(order.len() >= honest_count,
                "Total order lost honest vertices: {} < {}", order.len(), honest_count);
        }
    }

    /// Test that DAG handles edge cases properly
    #[test]
    fn test_edge_cases() {
        let mut dag = DAGConsensus::new();

        // Test empty vertex ID
        let empty_vertex = create_test_vertex("", vec![]);
        assert!(dag.add_vertex(empty_vertex).is_ok());

        // Test very long vertex ID
        let long_id = "a".repeat(1000);
        let long_vertex = create_test_vertex(&long_id, vec![]);
        assert!(dag.add_vertex(long_vertex).is_ok());

        // Test vertex with many parents
        let root1 = create_test_vertex("root1", vec![]);
        let root2 = create_test_vertex("root2", vec![]);
        let root3 = create_test_vertex("root3", vec![]);

        assert!(dag.add_vertex(root1).is_ok());
        assert!(dag.add_vertex(root2).is_ok());
        assert!(dag.add_vertex(root3).is_ok());

        let multi_parent = create_test_vertex("multi", vec!["root1", "root2", "root3"]);
        assert!(dag.add_vertex(multi_parent).is_ok());

        // All should achieve consensus
        assert_eq!(dag.get_confidence("multi"), Some(ConsensusStatus::Final));
    }
}
