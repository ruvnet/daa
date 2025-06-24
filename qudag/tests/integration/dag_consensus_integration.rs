//! Integration tests for DAG consensus and finality

use qudag_dag::{
    QrDag, Vertex, Edge, ConsensusEngine, TipSelection,
    consensus::{QrAvalanche, ConsensusState, VotingRound},
    graph::{DAGGraph, GraphMetrics},
    vertex::{VertexId, VertexData, VertexType},
};
use qudag_protocol::{Coordinator, ProtocolConfig};
use qudag_crypto::{
    ml_dsa::{MlDsa65, SigningKey, VerifyingKey},
    fingerprint::Fingerprint,
};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn, error};

#[tokio::test]
async fn test_dag_consensus_convergence() {
    // Test DAG consensus convergence with multiple nodes
    let node_count = 5;
    let mut coordinators = Vec::new();
    
    // Create and start coordinators
    for i in 0..node_count {
        let config = ProtocolConfig {
            network_port: 11000 + i as u16,
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![format!("127.0.0.1:{}", 11000)]
            },
            max_peers: 10,
            validation_timeout: 1000,
        };
        
        let coordinator = Coordinator::new(config).await.unwrap();
        coordinators.push(coordinator);
    }
    
    // Start all coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.start().await.unwrap();
    }
    
    // Allow network formation
    sleep(Duration::from_secs(2)).await;
    
    // Create test transactions
    let transactions = vec![
        b"Transaction A: Alice sends 10 tokens to Bob".to_vec(),
        b"Transaction B: Bob sends 5 tokens to Charlie".to_vec(),
        b"Transaction C: Charlie sends 3 tokens to Alice".to_vec(),
        b"Transaction D: Alice sends 2 tokens to Dave".to_vec(),
        b"Transaction E: Dave sends 1 token to Bob".to_vec(),
    ];
    
    // Submit transactions from different nodes
    for (i, tx) in transactions.iter().enumerate() {
        let node_idx = i % node_count;
        coordinators[node_idx].submit_transaction(tx.clone()).await.unwrap();
        
        // Small delay between transactions
        sleep(Duration::from_millis(100)).await;
    }
    
    // Allow consensus to complete
    sleep(Duration::from_secs(3)).await;
    
    // Verify all nodes have the same finalized DAG state
    let mut dag_states = Vec::new();
    for (i, coordinator) in coordinators.iter().enumerate() {
        let dag = coordinator.dag_manager().unwrap();
        let finalized_vertices = dag.get_finalized_vertices().await.unwrap();
        dag_states.push(finalized_vertices);
        
        info!("Node {} has {} finalized vertices", i, dag_states[i].len());
    }
    
    // All nodes should have the same number of finalized vertices
    let expected_count = dag_states[0].len();
    for (i, state) in dag_states.iter().enumerate() {
        assert_eq!(
            state.len(), expected_count,
            "Node {} has different finalized vertex count", i
        );
    }
    
    // Verify transaction finality
    for coordinator in &coordinators {
        let dag = coordinator.dag_manager().unwrap();
        for tx in &transactions {
            assert!(
                dag.is_transaction_finalized(tx).await.unwrap(),
                "Transaction should be finalized"
            );
        }
    }
    
    // Stop all coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_byzantine_fault_tolerance() {
    // Test Byzantine fault tolerance with up to f faulty nodes
    let total_nodes = 7; // Can tolerate up to 2 Byzantine nodes
    let byzantine_nodes = 2;
    let mut coordinators = Vec::new();
    
    // Create coordinators
    for i in 0..total_nodes {
        let config = ProtocolConfig {
            network_port: 11100 + i as u16,
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![format!("127.0.0.1:{}", 11100)]
            },
            max_peers: 10,
            validation_timeout: 2000,
        };
        
        let coordinator = Coordinator::new(config).await.unwrap();
        coordinators.push(coordinator);
    }
    
    // Start all coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(2)).await;
    
    // Configure first two nodes as Byzantine (malicious)
    for i in 0..byzantine_nodes {
        coordinators[i].enable_byzantine_behavior().await.unwrap();
    }
    
    // Submit valid transactions from honest nodes
    let honest_transactions = vec![
        b"Honest TX 1: Transfer 100 tokens".to_vec(),
        b"Honest TX 2: Transfer 50 tokens".to_vec(),
        b"Honest TX 3: Transfer 25 tokens".to_vec(),
    ];
    
    for (i, tx) in honest_transactions.iter().enumerate() {
        let node_idx = byzantine_nodes + (i % (total_nodes - byzantine_nodes));
        coordinators[node_idx].submit_transaction(tx.clone()).await.unwrap();
        sleep(Duration::from_millis(50)).await;
    }
    
    // Byzantine nodes attempt to submit conflicting/invalid transactions
    let malicious_transactions = vec![
        b"Malicious TX 1: Double spend attempt".to_vec(),
        b"Malicious TX 2: Invalid signature".to_vec(),
        b"Malicious TX 3: Conflicting state".to_vec(),
    ];
    
    for (i, tx) in malicious_transactions.iter().enumerate() {
        let result = coordinators[i % byzantine_nodes]
            .submit_malicious_transaction(tx.clone()).await;
        
        // Malicious transactions should be rejected or isolated
        match result {
            Ok(_) => info!("Malicious transaction {} submitted", i),
            Err(e) => info!("Malicious transaction {} rejected: {}", i, e),
        }
    }
    
    // Allow consensus despite Byzantine nodes
    sleep(Duration::from_secs(5)).await;
    
    // Verify honest nodes reached consensus on valid transactions
    let honest_node_indices: Vec<_> = (byzantine_nodes..total_nodes).collect();
    let mut consensus_states = Vec::new();
    
    for &i in &honest_node_indices {
        let dag = coordinators[i].dag_manager().unwrap();
        let finalized_txs = dag.get_finalized_transactions().await.unwrap();
        consensus_states.push(finalized_txs);
    }
    
    // All honest nodes should agree on the same set of valid transactions
    let reference_state = &consensus_states[0];
    for (i, state) in consensus_states.iter().enumerate() {
        assert_eq!(
            state.len(), reference_state.len(),
            "Honest node {} has different consensus state", honest_node_indices[i]
        );
        
        // Verify all honest transactions are included
        for tx in &honest_transactions {
            assert!(
                state.contains(tx),
                "Honest transaction missing from node {}", honest_node_indices[i]
            );
        }
        
        // Verify malicious transactions are excluded
        for tx in &malicious_transactions {
            assert!(
                !state.contains(tx),
                "Malicious transaction included in node {}", honest_node_indices[i]
            );
        }
    }
    
    info!("Byzantine fault tolerance test passed with {} Byzantine nodes", byzantine_nodes);
    
    // Stop all coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_dag_finality_performance() {
    // Test DAG finality performance under high load
    let node_count = 4;
    let mut coordinators = Vec::new();
    
    // Create coordinators with optimized settings
    for i in 0..node_count {
        let config = ProtocolConfig {
            network_port: 11200 + i as u16,
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![format!("127.0.0.1:{}", 11200)]
            },
            max_peers: 10,
            validation_timeout: 500,
        };
        
        let coordinator = Coordinator::new(config).await.unwrap();
        coordinators.push(coordinator);
    }
    
    // Start coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(1)).await;
    
    // Generate high transaction load
    let tx_count = 1000;
    let batch_size = 50;
    
    let start_time = Instant::now();
    
    for batch in 0..(tx_count / batch_size) {
        let mut batch_txs = Vec::new();
        
        for i in 0..batch_size {
            let tx_id = batch * batch_size + i;
            let tx = format!("High-load transaction {}: value {}", tx_id, tx_id * 10).into_bytes();
            batch_txs.push(tx);
        }
        
        // Submit batch from different nodes
        for (i, tx) in batch_txs.iter().enumerate() {
            let node_idx = i % node_count;
            coordinators[node_idx].submit_transaction(tx.clone()).await.unwrap();
        }
        
        // Brief pause between batches
        sleep(Duration::from_millis(10)).await;
    }
    
    let submission_time = start_time.elapsed();
    info!("Submitted {} transactions in {:?}", tx_count, submission_time);
    
    // Measure finality time
    let finality_start = Instant::now();
    
    // Wait for all transactions to be finalized
    loop {
        let mut all_finalized = true;
        
        for coordinator in &coordinators {
            let dag = coordinator.dag_manager().unwrap();
            let finalized_count = dag.get_finalized_transaction_count().await.unwrap();
            
            if finalized_count < tx_count {
                all_finalized = false;
                break;
            }
        }
        
        if all_finalized {
            break;
        }
        
        sleep(Duration::from_millis(100)).await;
        
        // Timeout check
        if finality_start.elapsed() > Duration::from_secs(30) {
            panic!("Finality timeout - not all transactions finalized");
        }
    }
    
    let finality_time = finality_start.elapsed();
    let total_time = start_time.elapsed();
    
    info!("Finality achieved in {:?} (total time: {:?})", finality_time, total_time);
    info!("Throughput: {:.2} tx/sec", tx_count as f64 / total_time.as_secs_f64());
    
    // Performance assertions
    assert!(
        finality_time < Duration::from_secs(10),
        "Finality should be achieved within 10 seconds"
    );
    
    let throughput = tx_count as f64 / total_time.as_secs_f64();
    assert!(
        throughput > 50.0,
        "Throughput should exceed 50 tx/sec, got {:.2}", throughput
    );
    
    // Verify consistency across all nodes
    let mut final_states = Vec::new();
    for (i, coordinator) in coordinators.iter().enumerate() {
        let dag = coordinator.dag_manager().unwrap();
        let finalized_txs = dag.get_finalized_transactions().await.unwrap();
        final_states.push(finalized_txs);
        
        info!("Node {} finalized {} transactions", i, final_states[i].len());
    }
    
    // All nodes should have finalized the same transactions
    let reference_count = final_states[0].len();
    for (i, state) in final_states.iter().enumerate() {
        assert_eq!(
            state.len(), reference_count,
            "Node {} has different finalized count", i
        );
    }
    
    // Stop coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_tip_selection_algorithm() {
    // Test tip selection algorithm under various conditions
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    coordinator.start().await.unwrap();
    
    let dag = coordinator.dag_manager().unwrap();
    
    // Create initial DAG structure
    let genesis_vertex = dag.create_genesis_vertex().await.unwrap();
    
    // Add several layers of vertices
    let mut previous_layer = vec![genesis_vertex];
    
    for layer in 1..=5 {
        let mut current_layer = Vec::new();
        
        for i in 0..3 {
            let vertex_data = VertexData::new(
                format!("Layer {} Vertex {}", layer, i).into_bytes(),
                Instant::now(),
            );
            
            // Reference vertices from previous layer
            let references = if previous_layer.len() >= 2 {
                vec![previous_layer[0].id(), previous_layer[1].id()]
            } else {
                vec![previous_layer[0].id()]
            };
            
            let vertex = dag.add_vertex(vertex_data, references).await.unwrap();
            current_layer.push(vertex);
        }
        
        previous_layer = current_layer;
        sleep(Duration::from_millis(50)).await;
    }
    
    // Test tip selection
    let tip_selector = dag.tip_selector();
    
    // Get current tips
    let tips = tip_selector.select_tips(2).await.unwrap();
    assert_eq!(tips.len(), 2, "Should select 2 tips");
    
    // Verify tips are actually tips (no children)
    for tip_id in &tips {
        let children = dag.get_vertex_children(*tip_id).await.unwrap();
        assert!(children.is_empty(), "Tip should have no children");
    }
    
    // Test weighted tip selection
    let weighted_tips = tip_selector.select_weighted_tips(3).await.unwrap();
    assert!(weighted_tips.len() <= 3, "Should select at most 3 weighted tips");
    
    // Test tip selection with constraints
    let constrained_tips = tip_selector.select_tips_with_constraints(
        2,
        |vertex_id| {
            // Only select tips from recent layers
            true // Simplified constraint for test
        }
    ).await.unwrap();
    
    assert!(!constrained_tips.is_empty(), "Should find constrained tips");
    
    // Add more vertices and test adaptive tip selection
    for i in 0..10 {
        let vertex_data = VertexData::new(
            format!("Additional vertex {}", i).into_bytes(),
            Instant::now(),
        );
        
        let selected_tips = tip_selector.select_tips(2).await.unwrap();
        dag.add_vertex(vertex_data, selected_tips).await.unwrap();
        
        sleep(Duration::from_millis(20)).await;
    }
    
    // Verify DAG properties after tip selection
    let dag_metrics = dag.get_metrics().await.unwrap();
    info!("DAG metrics: {:?}", dag_metrics);
    
    assert!(dag_metrics.vertex_count > 20, "Should have created multiple vertices");
    assert!(dag_metrics.edge_count > 15, "Should have sufficient edges");
    assert!(dag_metrics.max_depth >= 5, "Should have proper depth");
    
    coordinator.stop().await.unwrap();
}

#[tokio::test]
async fn test_conflicting_transaction_resolution() {
    // Test resolution of conflicting transactions in DAG
    let node_count = 3;
    let mut coordinators = Vec::new();
    
    // Create coordinators
    for i in 0..node_count {
        let config = ProtocolConfig {
            network_port: 11300 + i as u16,
            bootstrap_nodes: if i == 0 { 
                vec![] 
            } else { 
                vec![format!("127.0.0.1:{}", 11300)]
            },
            max_peers: 10,
            validation_timeout: 1000,
        };
        
        let coordinator = Coordinator::new(config).await.unwrap();
        coordinators.push(coordinator);
    }
    
    // Start coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.start().await.unwrap();
    }
    
    sleep(Duration::from_secs(1)).await;
    
    // Create conflicting transactions
    let alice_balance = 100;
    
    // Alice tries to spend the same 100 tokens in two different transactions
    let tx_a = format!("Alice sends {} tokens to Bob", alice_balance).into_bytes();
    let tx_b = format!("Alice sends {} tokens to Charlie", alice_balance).into_bytes();
    
    // Submit conflicting transactions simultaneously from different nodes
    let submit_a = coordinators[0].submit_transaction(tx_a.clone());
    let submit_b = coordinators[1].submit_transaction(tx_b.clone());
    
    let (result_a, result_b) = tokio::join!(submit_a, submit_b);
    
    // Both transactions should be initially accepted into the DAG
    assert!(result_a.is_ok(), "Transaction A should be initially accepted");
    assert!(result_b.is_ok(), "Transaction B should be initially accepted");
    
    // Allow conflict resolution
    sleep(Duration::from_secs(3)).await;
    
    // Verify conflict resolution - only one transaction should be finalized
    let mut finalized_a_count = 0;
    let mut finalized_b_count = 0;
    
    for (i, coordinator) in coordinators.iter().enumerate() {
        let dag = coordinator.dag_manager().unwrap();
        
        let is_a_finalized = dag.is_transaction_finalized(&tx_a).await.unwrap();
        let is_b_finalized = dag.is_transaction_finalized(&tx_b).await.unwrap();
        
        if is_a_finalized {
            finalized_a_count += 1;
        }
        if is_b_finalized {
            finalized_b_count += 1;
        }
        
        // Exactly one of the conflicting transactions should be finalized
        assert!(
            is_a_finalized ^ is_b_finalized,
            "Node {} should finalize exactly one conflicting transaction", i
        );
        
        info!(
            "Node {}: TX_A finalized: {}, TX_B finalized: {}",
            i, is_a_finalized, is_b_finalized
        );
    }
    
    // All nodes should agree on which transaction was finalized
    assert!(
        (finalized_a_count == node_count && finalized_b_count == 0) ||
        (finalized_a_count == 0 && finalized_b_count == node_count),
        "All nodes should agree on conflict resolution"
    );
    
    // Test additional non-conflicting transaction
    let tx_c = b"Bob sends 10 tokens to Dave".to_vec();
    coordinators[2].submit_transaction(tx_c.clone()).await.unwrap();
    
    sleep(Duration::from_secs(1)).await;
    
    // Non-conflicting transaction should be finalized on all nodes
    for coordinator in &coordinators {
        let dag = coordinator.dag_manager().unwrap();
        assert!(
            dag.is_transaction_finalized(&tx_c).await.unwrap(),
            "Non-conflicting transaction should be finalized"
        );
    }
    
    // Stop coordinators
    for coordinator in coordinators.iter_mut() {
        coordinator.stop().await.unwrap();
    }
}

#[tokio::test]
async fn test_dag_pruning_and_checkpointing() {
    // Test DAG pruning and checkpointing functionality
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    coordinator.start().await.unwrap();
    
    let dag = coordinator.dag_manager().unwrap();
    
    // Create a large DAG
    let transaction_count = 500;
    let checkpoint_interval = 100;
    
    for i in 0..transaction_count {
        let tx = format!("Transaction {} with data {}", i, i * 2).into_bytes();
        dag.submit_transaction(tx).await.unwrap();
        
        // Create checkpoint every 100 transactions
        if i % checkpoint_interval == 0 && i > 0 {
            dag.create_checkpoint().await.unwrap();
            info!("Created checkpoint at transaction {}", i);
        }
        
        if i % 50 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    // Wait for finalization
    sleep(Duration::from_secs(2)).await;
    
    // Get initial metrics
    let initial_metrics = dag.get_metrics().await.unwrap();
    info!("Initial DAG metrics: {:?}", initial_metrics);
    
    // Perform pruning
    let pruning_result = dag.prune_finalized_vertices().await.unwrap();
    info!("Pruning result: {:?}", pruning_result);
    
    // Get post-pruning metrics
    let pruned_metrics = dag.get_metrics().await.unwrap();
    info!("Post-pruning DAG metrics: {:?}", pruned_metrics);
    
    // Verify pruning effectiveness
    assert!(
        pruned_metrics.vertex_count < initial_metrics.vertex_count,
        "Pruning should reduce vertex count"
    );
    
    // Verify data integrity after pruning
    let recent_transactions: Vec<_> = (transaction_count - 50..transaction_count)
        .map(|i| format!("Transaction {} with data {}", i, i * 2).into_bytes())
        .collect();
    
    for tx in &recent_transactions {
        assert!(
            dag.is_transaction_finalized(tx).await.unwrap(),
            "Recent transaction should still be accessible after pruning"
        );
    }
    
    // Test checkpoint loading
    let checkpoint_data = dag.get_latest_checkpoint().await.unwrap();
    assert!(checkpoint_data.is_some(), "Should have checkpoint data");
    
    let checkpoint = checkpoint_data.unwrap();
    info!("Latest checkpoint: block height {}, {} vertices", 
          checkpoint.block_height, checkpoint.vertex_count);
    
    // Verify checkpoint can be used for state reconstruction
    let reconstructed_state = dag.reconstruct_from_checkpoint(&checkpoint).await.unwrap();
    assert!(
        reconstructed_state.vertex_count >= checkpoint.vertex_count,
        "Reconstructed state should include checkpoint data"
    );
    
    coordinator.stop().await.unwrap();
}