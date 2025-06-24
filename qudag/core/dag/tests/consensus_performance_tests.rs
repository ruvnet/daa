use blake3::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::{ConsensusEvent, DagError, Graph, Node, NodeState, QrAvalanche};

const TEST_NODES: usize = 1000;
const MAX_PARENTS: usize = 5;
const CONCURRENT_VOTES: usize = 100;
const TARGET_FINALITY_MS: u64 = 1000; // 1 second target

async fn setup_test_dag(nodes: usize) -> (Arc<Graph>, Vec<Hash>) {
    let graph = Arc::new(Graph::new());
    let mut node_hashes = Vec::with_capacity(nodes);

    // Create root node
    let root = Node::new(vec![0], vec![]);
    let root_hash = *root.hash();
    graph.add_node(root).unwrap();
    node_hashes.push(root_hash);

    // Create remaining nodes with random parents
    for i in 1..nodes {
        let data = vec![i as u8];
        let mut parents = Vec::new();

        // Select random parents from existing nodes
        let parent_count = (i % MAX_PARENTS) + 1;
        for _ in 0..parent_count {
            let parent_idx = i % node_hashes.len();
            parents.push(node_hashes[parent_idx]);
        }

        let node = Node::new(data, parents);
        let node_hash = *node.hash();
        graph.add_node(node).unwrap();
        node_hashes.push(node_hash);
    }

    (graph, node_hashes)
}

async fn simulate_concurrent_voting(
    consensus: &QrAvalanche,
    node_hash: Hash,
    vote_count: usize,
) -> Result<(), DagError> {
    let mut handles = Vec::with_capacity(vote_count);

    for i in 0..vote_count {
        let voter_hash = blake3::hash(&[i as u8]);
        let consensus = consensus.clone();
        let node_hash = node_hash;

        let handle =
            tokio::spawn(async move { consensus.record_vote(node_hash, voter_hash, true).await });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap()?;
    }

    Ok(())
}

#[tokio::test]
async fn test_finality_latency() {
    let (graph, node_hashes) = setup_test_dag(TEST_NODES).await;
    let (consensus, mut events_rx) = QrAvalanche::new(graph.clone());

    let mut total_latency = Duration::ZERO;
    let mut finalized_count = 0;

    for &node_hash in &node_hashes {
        graph
            .update_node_state(&node_hash, NodeState::Verified)
            .unwrap();

        let start = Instant::now();
        consensus.process_node(node_hash).await.unwrap();

        // Simulate concurrent voting
        simulate_concurrent_voting(&consensus, node_hash, CONCURRENT_VOTES)
            .await
            .unwrap();

        // Wait for finalization event
        match events_rx.recv().await {
            Some(ConsensusEvent::NodeFinalized(hash)) => {
                assert_eq!(hash, node_hash);
                let latency = start.elapsed();
                total_latency += latency;
                finalized_count += 1;

                // Verify sub-second finality
                assert!(
                    latency.as_millis() < TARGET_FINALITY_MS as u128,
                    "Finality took {}ms, exceeding target of {}ms",
                    latency.as_millis(),
                    TARGET_FINALITY_MS
                );
            }
            _ => panic!("Expected finalization event"),
        }
    }

    let avg_latency = total_latency / finalized_count as u32;
    println!(
        "Average finality latency: {}ms over {} nodes",
        avg_latency.as_millis(),
        finalized_count
    );
}

#[tokio::test]
async fn test_concurrent_path_validation() {
    let (graph, node_hashes) = setup_test_dag(TEST_NODES).await;
    let (consensus, mut events_rx) = QrAvalanche::new(graph.clone());

    // Process nodes in parallel batches
    let batch_size = 10;
    let mut handles = Vec::new();

    for nodes in node_hashes.chunks(batch_size) {
        let mut batch_handles = Vec::new();

        for &node_hash in nodes {
            graph
                .update_node_state(&node_hash, NodeState::Verified)
                .unwrap();

            let consensus = consensus.clone();
            let handle = tokio::spawn(async move {
                consensus.process_node(node_hash).await.unwrap();
                simulate_concurrent_voting(&consensus, node_hash, CONCURRENT_VOTES)
                    .await
                    .unwrap();
                node_hash
            });

            batch_handles.push(handle);
        }

        handles.extend(batch_handles);
    }

    // Verify all nodes were finalized
    let mut finalized_nodes = std::collections::HashSet::new();

    for handle in handles {
        let node_hash = handle.await.unwrap();

        match events_rx.recv().await {
            Some(ConsensusEvent::NodeFinalized(hash)) => {
                assert_eq!(hash, node_hash);
                finalized_nodes.insert(hash);
            }
            _ => panic!("Expected finalization event"),
        }
    }

    assert_eq!(finalized_nodes.len(), TEST_NODES);
}

#[tokio::test]
async fn test_vertex_processing_performance() {
    let (graph, node_hashes) = setup_test_dag(TEST_NODES).await;
    let (consensus, mut events_rx) = QrAvalanche::new(graph);

    let start = Instant::now();
    let mut processed = 0;

    for &node_hash in &node_hashes {
        consensus.process_node(node_hash).await.unwrap();
        processed += 1;
    }

    let processing_time = start.elapsed();
    let vertices_per_second = processed as f64 / processing_time.as_secs_f64();

    println!(
        "Processed {} vertices in {}ms ({} vertices/second)",
        processed,
        processing_time.as_millis(),
        vertices_per_second as u64
    );

    assert!(
        vertices_per_second > 1000.0,
        "Vertex processing rate below target: {} vertices/second",
        vertices_per_second
    );
}
