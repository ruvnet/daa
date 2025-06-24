//! Concurrent operations and race condition tests for DAG consensus
//!
//! This module tests the thread safety of DAG consensus operations, including
//! concurrent node additions, state transitions, tip selection, and finality
//! detection under high contention scenarios.

use blake3::Hash;
use qudag_dag::{
    consensus::{ConsensusEngine, QrAvalanche, Vote, VoteResult},
    error::{DagError, Result},
    node::{NodeId, NodeState},
    tip_selection::TipSelector,
    Graph, Node, QrDag,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Mutex, RwLock};
use tokio::time::sleep;

/// Test concurrent node additions to DAG
#[tokio::test]
async fn test_concurrent_dag_node_addition() {
    const NUM_THREADS: usize = 16;
    const NODES_PER_THREAD: usize = 50;

    let dag = Arc::new(QrDag::new());
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();

    // Create genesis node first
    let genesis_data = vec![0u8; 256];
    let genesis_node = Node::new(genesis_data, vec![]);
    let genesis_hash = *genesis_node.hash();
    dag.add_node(genesis_node).await.unwrap();

    for thread_id in 0..NUM_THREADS {
        let dag_clone = dag.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut successful_additions = 0;
            let mut failed_additions = 0;
            let mut node_hashes = Vec::new();

            for i in 0..NODES_PER_THREAD {
                // Create node data
                let node_data = format!("Thread {} Node {}", thread_id, i).into_bytes();

                // Reference genesis or previously created nodes
                let parents = if i == 0 {
                    vec![genesis_hash]
                } else if !node_hashes.is_empty() {
                    // Reference up to 2 previous nodes
                    let mut parents = vec![genesis_hash];
                    let reference_count = std::cmp::min(2, node_hashes.len());
                    if reference_count > 0 {
                        let mut selected: Vec<_> = node_hashes
                            .choose_multiple(&mut thread_rng(), reference_count)
                            .cloned()
                            .collect();
                        parents.append(&mut selected);
                    }
                    parents
                } else {
                    vec![genesis_hash]
                };

                let node = Node::new(node_data, parents);
                let node_hash = *node.hash();

                match dag_clone.add_node(node).await {
                    Ok(()) => {
                        successful_additions += 1;
                        node_hashes.push(node_hash);
                    }
                    Err(e) => {
                        failed_additions += 1;
                        eprintln!("Thread {} Node {}: Addition failed: {}", thread_id, i, e);
                    }
                }

                // Yield occasionally to allow other threads
                if i % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (
                thread_id,
                successful_additions,
                failed_additions,
                node_hashes,
            )
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_successful = 0;
    let mut total_failed = 0;
    let mut all_node_hashes = Vec::new();

    for handle in handles {
        let (thread_id, successful, failed, hashes) = handle.await.unwrap();
        println!(
            "Thread {}: {} successful, {} failed node additions",
            thread_id, successful, failed
        );
        total_successful += successful;
        total_failed += failed;
        all_node_hashes.extend(hashes);
    }

    let final_dag_size = dag.node_count().await;
    let expected_size = total_successful + 1; // +1 for genesis

    println!("Concurrent DAG node addition results:");
    println!("  Total successful: {}", total_successful);
    println!("  Total failed: {}", total_failed);
    println!("  Final DAG size: {}", final_dag_size);
    println!("  Expected size: {}", expected_size);

    assert_eq!(
        final_dag_size, expected_size,
        "DAG size should match successful additions plus genesis"
    );
    assert!(
        total_successful > 0,
        "Should have some successful additions"
    );

    // Verify all nodes are actually accessible
    let mut accessible_count = 0;
    for hash in &all_node_hashes {
        if dag.get_node(hash).await.is_some() {
            accessible_count += 1;
        }
    }
    assert_eq!(
        accessible_count,
        all_node_hashes.len(),
        "All successfully added nodes should be accessible"
    );
}

/// Test concurrent consensus voting operations
#[tokio::test]
async fn test_concurrent_consensus_voting() {
    const NUM_VOTERS: usize = 20;
    const NODES_TO_VOTE_ON: usize = 50;
    const VOTES_PER_VOTER: usize = 100;

    let dag = Arc::new(QrDag::new());
    let consensus = Arc::new(QrAvalanche::new());
    let barrier = Arc::new(Barrier::new(NUM_VOTERS));

    // Setup: Create nodes to vote on
    let genesis_node = Node::new(vec![0u8; 256], vec![]);
    let genesis_hash = *genesis_node.hash();
    dag.add_node(genesis_node).await.unwrap();

    let mut vote_targets = Vec::new();
    vote_targets.push(genesis_hash);

    for i in 1..NODES_TO_VOTE_ON {
        let node_data = format!("Vote target node {}", i).into_bytes();
        let node = Node::new(node_data, vec![genesis_hash]);
        let node_hash = *node.hash();
        dag.add_node(node).await.unwrap();
        vote_targets.push(node_hash);
    }

    let vote_targets = Arc::new(vote_targets);
    let mut handles = Vec::new();

    for voter_id in 0..NUM_VOTERS {
        let consensus_clone = consensus.clone();
        let targets_clone = vote_targets.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut votes_cast = 0;
            let mut vote_conflicts = 0;
            let mut vote_results = Vec::new();

            for i in 0..VOTES_PER_VOTER {
                // Select random target to vote on
                let target_index = thread_rng().gen_range(0..targets_clone.len());
                let target_hash = targets_clone[target_index];

                // Create vote (alternating between accept/reject to create conflicts)
                let vote_value = if i % 3 == 0 {
                    false // Reject vote to create conflicts
                } else {
                    true // Accept vote
                };

                let vote = Vote {
                    node_id: target_hash,
                    voter_id: format!("voter_{}", voter_id),
                    value: vote_value,
                    timestamp: Instant::now(),
                };

                match consensus_clone.cast_vote(vote).await {
                    Ok(result) => {
                        votes_cast += 1;
                        vote_results.push(result);

                        // Check for conflicts
                        if matches!(result, VoteResult::Conflict) {
                            vote_conflicts += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Voter {} vote {}: Failed to cast vote: {}", voter_id, i, e);
                    }
                }

                // Query consensus occasionally
                if i % 20 == 0 {
                    let _consensus_state = consensus_clone.get_consensus_state(&target_hash).await;
                }

                // Yield to allow other voters
                if i % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (voter_id, votes_cast, vote_conflicts, vote_results)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_votes = 0;
    let mut total_conflicts = 0;
    let mut all_results = Vec::new();

    for handle in handles {
        let (voter_id, votes, conflicts, results) = handle.await.unwrap();
        println!(
            "Voter {}: {} votes cast, {} conflicts",
            voter_id, votes, conflicts
        );
        total_votes += votes;
        total_conflicts += conflicts;
        all_results.extend(results);
    }

    println!("Concurrent consensus voting results:");
    println!("  Total votes cast: {}", total_votes);
    println!("  Total conflicts: {}", total_conflicts);
    println!(
        "  Conflict rate: {:.2}%",
        (total_conflicts as f64 / total_votes as f64) * 100.0
    );

    // Verify final consensus states
    let mut finalized_nodes = 0;
    let mut pending_nodes = 0;

    for target_hash in vote_targets.iter() {
        match consensus.get_consensus_state(target_hash).await {
            Ok(state) => {
                if state.is_finalized() {
                    finalized_nodes += 1;
                } else {
                    pending_nodes += 1;
                }
            }
            Err(e) => {
                eprintln!("Failed to get consensus state for {:?}: {}", target_hash, e);
            }
        }
    }

    println!("  Finalized nodes: {}", finalized_nodes);
    println!("  Pending nodes: {}", pending_nodes);

    assert!(total_votes > 0, "Should cast some votes");
    assert_eq!(
        total_votes,
        NUM_VOTERS * VOTES_PER_VOTER,
        "All votes should be cast successfully"
    );
}

/// Test race conditions in node state transitions
#[tokio::test]
async fn test_node_state_transition_races() {
    const NUM_THREADS: usize = 15;
    const STATE_TRANSITIONS_PER_THREAD: usize = 100;
    const NUM_NODES: usize = 50;

    let dag = Arc::new(QrDag::new());
    let state_consistency = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let barrier = Arc::new(Barrier::new(NUM_THREADS));

    // Setup: Create nodes for state transitions
    let genesis_node = Node::new(vec![0u8; 256], vec![]);
    let genesis_hash = *genesis_node.hash();
    dag.add_node(genesis_node).await.unwrap();

    let mut test_nodes = Vec::new();
    test_nodes.push(genesis_hash);

    for i in 1..NUM_NODES {
        let node_data = format!("State test node {}", i).into_bytes();
        let node = Node::new(node_data, vec![genesis_hash]);
        let node_hash = *node.hash();
        dag.add_node(node).await.unwrap();
        test_nodes.push(node_hash);
    }

    let test_nodes = Arc::new(test_nodes);
    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let dag_clone = dag.clone();
        let nodes_clone = test_nodes.clone();
        let consistency_clone = state_consistency.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut successful_transitions = 0;
            let mut failed_transitions = 0;
            let mut consistency_violations = 0;

            for i in 0..STATE_TRANSITIONS_PER_THREAD {
                // Select random node
                let node_index = thread_rng().gen_range(0..nodes_clone.len());
                let node_hash = nodes_clone[node_index];

                // Get current state
                let current_state = if let Some(node) = dag_clone.get_node(&node_hash).await {
                    node.state()
                } else {
                    continue;
                };

                // Determine next valid state transition
                let next_state = match current_state {
                    NodeState::Pending => {
                        if i % 3 == 0 {
                            NodeState::Verified
                        } else {
                            NodeState::Processing
                        }
                    }
                    NodeState::Processing => {
                        if i % 2 == 0 {
                            NodeState::Verified
                        } else {
                            NodeState::Rejected
                        }
                    }
                    NodeState::Verified => NodeState::Final,
                    NodeState::Final => continue, // No further transitions
                    NodeState::Rejected => continue, // No further transitions
                };

                // Attempt state transition
                match dag_clone.update_node_state(&node_hash, next_state).await {
                    Ok(()) => {
                        successful_transitions += 1;

                        // Verify transition took effect
                        if let Some(updated_node) = dag_clone.get_node(&node_hash).await {
                            if updated_node.state() != next_state {
                                consistency_violations += 1;
                                consistency_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                                eprintln!(
                                    "Thread {}: State transition inconsistency for node {:?}",
                                    thread_id, node_hash
                                );
                            }
                        } else {
                            consistency_violations += 1;
                            consistency_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                            eprintln!(
                                "Thread {}: Node disappeared after state transition",
                                thread_id
                            );
                        }
                    }
                    Err(e) => {
                        failed_transitions += 1;
                        // Some failures are expected due to invalid transitions or race conditions
                    }
                }

                // Yield to increase chance of race conditions
                if i % 15 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (
                thread_id,
                successful_transitions,
                failed_transitions,
                consistency_violations,
            )
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_successful = 0;
    let mut total_failed = 0;
    let mut total_violations = 0;

    for handle in handles {
        let (thread_id, successful, failed, violations) = handle.await.unwrap();
        println!(
            "Thread {}: {} successful, {} failed, {} violations",
            thread_id, successful, failed, violations
        );
        total_successful += successful;
        total_failed += failed;
        total_violations += violations;
    }

    let final_consistency = state_consistency.load(std::sync::atomic::Ordering::SeqCst);

    println!("Node state transition race test results:");
    println!("  Total successful transitions: {}", total_successful);
    println!("  Total failed transitions: {}", total_failed);
    println!("  Total consistency violations: {}", total_violations);
    println!("  Final consistency: {}", final_consistency);

    // Verify final state distribution
    let mut state_counts = HashMap::new();
    for node_hash in test_nodes.iter() {
        if let Some(node) = dag.get_node(node_hash).await {
            *state_counts.entry(node.state()).or_insert(0) += 1;
        }
    }

    println!("  Final state distribution:");
    for (state, count) in &state_counts {
        println!("    {:?}: {}", state, count);
    }

    assert!(
        total_successful > 0,
        "Should have some successful transitions"
    );
    assert_eq!(total_violations, 0, "Should have no consistency violations");
    assert!(final_consistency, "Should maintain state consistency");
}

/// Test concurrent tip selection operations
#[tokio::test]
async fn test_concurrent_tip_selection() {
    const NUM_SELECTORS: usize = 12;
    const SELECTIONS_PER_SELECTOR: usize = 200;
    const DAG_SIZE: usize = 100;

    let dag = Arc::new(QrDag::new());
    let tip_selector = Arc::new(TipSelector::new());
    let barrier = Arc::new(Barrier::new(NUM_SELECTORS));

    // Setup: Create a DAG structure
    let genesis_node = Node::new(vec![0u8; 256], vec![]);
    let genesis_hash = *genesis_node.hash();
    dag.add_node(genesis_node).await.unwrap();

    let mut all_nodes = vec![genesis_hash];

    // Create a branching DAG structure
    for i in 1..DAG_SIZE {
        let node_data = format!("Tip selection test node {}", i).into_bytes();

        // Reference 1-3 random previous nodes
        let reference_count = thread_rng().gen_range(1..=std::cmp::min(3, all_nodes.len()));
        let parents: Vec<_> = all_nodes
            .choose_multiple(&mut thread_rng(), reference_count)
            .cloned()
            .collect();

        let node = Node::new(node_data, parents);
        let node_hash = *node.hash();
        dag.add_node(node).await.unwrap();
        all_nodes.push(node_hash);
    }

    let mut handles = Vec::new();

    for selector_id in 0..NUM_SELECTORS {
        let dag_clone = dag.clone();
        let selector_clone = tip_selector.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            let mut successful_selections = 0;
            let mut failed_selections = 0;
            let mut unique_tips = HashSet::new();
            let mut tip_counts = HashMap::new();

            for i in 0..SELECTIONS_PER_SELECTOR {
                match selector_clone.select_tips(&dag_clone, 2).await {
                    Ok(tips) => {
                        successful_selections += 1;

                        // Track unique tips and their frequency
                        for tip in &tips {
                            unique_tips.insert(*tip);
                            *tip_counts.entry(*tip).or_insert(0) += 1;
                        }

                        // Verify tips are valid (exist in DAG)
                        for tip in &tips {
                            if dag_clone.get_node(tip).await.is_none() {
                                eprintln!(
                                    "Selector {}: Selected non-existent tip {:?}",
                                    selector_id, tip
                                );
                                failed_selections += 1;
                            }
                        }
                    }
                    Err(e) => {
                        failed_selections += 1;
                        eprintln!("Selector {} selection {}: Failed: {}", selector_id, i, e);
                    }
                }

                // Yield occasionally
                if i % 25 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (
                selector_id,
                successful_selections,
                failed_selections,
                unique_tips,
                tip_counts,
            )
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_successful = 0;
    let mut total_failed = 0;
    let mut all_unique_tips = HashSet::new();
    let mut global_tip_counts = HashMap::new();

    for handle in handles {
        let (selector_id, successful, failed, unique_tips, tip_counts) = handle.await.unwrap();
        println!(
            "Selector {}: {} successful, {} failed, {} unique tips",
            selector_id,
            successful,
            failed,
            unique_tips.len()
        );

        total_successful += successful;
        total_failed += failed;
        all_unique_tips.extend(unique_tips);

        for (tip, count) in tip_counts {
            *global_tip_counts.entry(tip).or_insert(0) += count;
        }
    }

    println!("Concurrent tip selection results:");
    println!("  Total successful selections: {}", total_successful);
    println!("  Total failed selections: {}", total_failed);
    println!("  Total unique tips selected: {}", all_unique_tips.len());

    // Analyze tip selection distribution
    let mut sorted_tips: Vec<_> = global_tip_counts.iter().collect();
    sorted_tips.sort_by(|a, b| b.1.cmp(a.1));

    println!("  Top 10 most selected tips:");
    for (i, (tip, count)) in sorted_tips.iter().take(10).enumerate() {
        println!("    {}: {:?} (selected {} times)", i + 1, tip, count);
    }

    assert!(
        total_successful > 0,
        "Should have successful tip selections"
    );
    assert!(all_unique_tips.len() > 1, "Should select diverse tips");
    assert_eq!(total_failed, 0, "All tip selections should succeed");

    // Verify all selected tips actually exist in the DAG
    for tip in &all_unique_tips {
        assert!(
            dag.get_node(tip).await.is_some(),
            "All selected tips should exist in DAG"
        );
    }
}

/// Test high-contention DAG operations stress test
#[tokio::test]
async fn test_dag_high_contention_stress() {
    const NUM_NODE_ADDERS: usize = 8;
    const NUM_STATE_UPDATERS: usize = 6;
    const NUM_READERS: usize = 10;
    const STRESS_DURATION_SECS: u64 = 15;

    let dag = Arc::new(QrDag::new());
    let consistency_checker = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let operation_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // Setup genesis
    let genesis_node = Node::new(vec![0u8; 256], vec![]);
    let genesis_hash = *genesis_node.hash();
    dag.add_node(genesis_node).await.unwrap();

    let shared_node_list = Arc::new(RwLock::new(vec![genesis_hash]));
    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(STRESS_DURATION_SECS);

    let mut handles = Vec::new();

    // Node adder tasks
    for adder_id in 0..NUM_NODE_ADDERS {
        let dag_clone = dag.clone();
        let node_list_clone = shared_node_list.clone();
        let counter_clone = operation_counter.clone();

        let handle = tokio::spawn(async move {
            let mut operations = 0;
            let mut added_nodes = 0;

            while Instant::now() < end_time {
                // Get random parents from existing nodes
                let parents = {
                    let nodes = node_list_clone.read().await;
                    if nodes.is_empty() {
                        vec![genesis_hash]
                    } else {
                        let count = std::cmp::min(2, nodes.len());
                        nodes
                            .choose_multiple(&mut thread_rng(), count)
                            .cloned()
                            .collect()
                    }
                };

                let node_data = format!("Adder {} Node {}", adder_id, operations).into_bytes();
                let node = Node::new(node_data, parents);
                let node_hash = *node.hash();

                if dag_clone.add_node(node).await.is_ok() {
                    added_nodes += 1;

                    // Add to shared list
                    {
                        let mut nodes = node_list_clone.write().await;
                        nodes.push(node_hash);
                    }
                }

                operations += 1;
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                if operations % 20 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (format!("Adder_{}", adder_id), operations, added_nodes)
        });

        handles.push(handle);
    }

    // State updater tasks
    for updater_id in 0..NUM_STATE_UPDATERS {
        let dag_clone = dag.clone();
        let node_list_clone = shared_node_list.clone();
        let counter_clone = operation_counter.clone();
        let checker_clone = consistency_checker.clone();

        let handle = tokio::spawn(async move {
            let mut operations = 0;
            let mut successful_updates = 0;

            while Instant::now() < end_time {
                // Get random node to update
                let target_hash = {
                    let nodes = node_list_clone.read().await;
                    if nodes.is_empty() {
                        continue;
                    }
                    *nodes.choose(&mut thread_rng()).unwrap()
                };

                // Get current state and determine next state
                if let Some(node) = dag_clone.get_node(&target_hash).await {
                    let current_state = node.state();
                    let next_state = match current_state {
                        NodeState::Pending => NodeState::Processing,
                        NodeState::Processing => {
                            if operations % 2 == 0 {
                                NodeState::Verified
                            } else {
                                NodeState::Rejected
                            }
                        }
                        NodeState::Verified => NodeState::Final,
                        _ => continue, // Can't transition further
                    };

                    if dag_clone
                        .update_node_state(&target_hash, next_state)
                        .await
                        .is_ok()
                    {
                        successful_updates += 1;

                        // Verify the update took effect
                        if let Some(updated_node) = dag_clone.get_node(&target_hash).await {
                            if updated_node.state() != next_state {
                                checker_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                    }
                }

                operations += 1;
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                if operations % 30 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (
                format!("Updater_{}", updater_id),
                operations,
                successful_updates,
            )
        });

        handles.push(handle);
    }

    // Reader tasks
    for reader_id in 0..NUM_READERS {
        let dag_clone = dag.clone();
        let node_list_clone = shared_node_list.clone();
        let counter_clone = operation_counter.clone();

        let handle = tokio::spawn(async move {
            let mut operations = 0;
            let mut successful_reads = 0;

            while Instant::now() < end_time {
                // Read random nodes
                let target_hash = {
                    let nodes = node_list_clone.read().await;
                    if nodes.is_empty() {
                        continue;
                    }
                    *nodes.choose(&mut thread_rng()).unwrap()
                };

                if dag_clone.get_node(&target_hash).await.is_some() {
                    successful_reads += 1;
                }

                // Also read DAG statistics
                let _size = dag_clone.node_count().await;

                operations += 1;
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                if operations % 50 == 0 {
                    tokio::task::yield_now().await;
                }
            }

            (
                format!("Reader_{}", reader_id),
                operations,
                successful_reads,
            )
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_operations = 0;
    let mut results_by_type = HashMap::new();

    for handle in handles {
        let result = handle.await.unwrap();
        let (task_name, ops, specific_metric) = result;
        let task_type = task_name.split('_').next().unwrap();

        println!(
            "{}: {} operations, {} specific metric",
            task_name, ops, specific_metric
        );
        total_operations += ops;
        *results_by_type.entry(task_type.to_string()).or_insert(0) += ops;
    }

    let elapsed = start_time.elapsed();
    let ops_per_second = total_operations as f64 / elapsed.as_secs_f64();
    let final_counter = operation_counter.load(std::sync::atomic::Ordering::SeqCst);
    let final_consistency = consistency_checker.load(std::sync::atomic::Ordering::SeqCst);
    let final_dag_size = dag.node_count().await;
    let final_node_list_size = shared_node_list.read().await.len();

    println!("\nDAG high contention stress test results:");
    println!("  Duration: {:?}", elapsed);
    println!("  Total operations: {}", total_operations);
    println!("  Counter value: {}", final_counter);
    println!("  Operations per second: {:.2}", ops_per_second);
    println!("  Final consistency: {}", final_consistency);
    println!("  Final DAG size: {}", final_dag_size);
    println!("  Final node list size: {}", final_node_list_size);

    for (task_type, ops) in &results_by_type {
        println!("  {}: {} operations", task_type, ops);
    }

    assert!(total_operations > 0, "Should complete operations");
    assert_eq!(
        final_counter, total_operations,
        "Counter should match operations"
    );
    assert!(final_consistency, "Should maintain consistency");
    assert_eq!(
        final_dag_size, final_node_list_size,
        "DAG size should match node list"
    );
    assert!(
        ops_per_second > 50.0,
        "Should achieve reasonable throughput"
    );
}

/// Test parallel DAG operations using rayon
#[test]
fn test_dag_parallel_operations() {
    use tokio::runtime::Runtime;

    const NUM_OPERATIONS: usize = 1000;
    const THREAD_POOL_SIZE: usize = 8;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let dag = Arc::new(QrDag::new());

        // Setup genesis
        let genesis_node = Node::new(vec![0u8; 256], vec![]);
        let genesis_hash = *genesis_node.hash();
        dag.add_node(genesis_node).await.unwrap();

        // Generate test data
        let test_data: Vec<Vec<u8>> = (0..NUM_OPERATIONS)
            .map(|i| format!("parallel_test_node_{}", i).into_bytes())
            .collect();

        // Create nodes in parallel (using a custom thread pool approach)
        let semaphore = Arc::new(tokio::sync::Semaphore::new(THREAD_POOL_SIZE));
        let mut handles = Vec::new();

        for (i, data) in test_data.iter().enumerate() {
            let dag_clone = dag.clone();
            let data_clone = data.clone();
            let semaphore_clone = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                let node = Node::new(data_clone, vec![genesis_hash]);
                let node_hash = *node.hash();

                match dag_clone.add_node(node).await {
                    Ok(()) => Some(node_hash),
                    Err(_) => None,
                }
            });

            handles.push(handle);
        }

        // Collect results
        let mut successful_additions = 0;
        let mut added_hashes = Vec::new();

        for handle in handles {
            if let Some(hash) = handle.await.unwrap() {
                successful_additions += 1;
                added_hashes.push(hash);
            }
        }

        println!("Parallel DAG operations results:");
        println!(
            "  Successful additions: {}/{}",
            successful_additions, NUM_OPERATIONS
        );
        println!("  Final DAG size: {}", dag.node_count().await);

        // Verify all nodes are accessible
        let mut accessible_count = 0;
        let verification_handles: Vec<_> = added_hashes
            .into_iter()
            .map(|hash| {
                let dag_clone = dag.clone();
                tokio::spawn(async move { dag_clone.get_node(&hash).await.is_some() })
            })
            .collect();

        for handle in verification_handles {
            if handle.await.unwrap() {
                accessible_count += 1;
            }
        }

        println!(
            "  Accessible nodes: {}/{}",
            accessible_count, successful_additions
        );

        assert_eq!(
            successful_additions, NUM_OPERATIONS,
            "All operations should succeed"
        );
        assert_eq!(
            accessible_count, successful_additions,
            "All nodes should be accessible"
        );
        assert_eq!(
            dag.node_count().await,
            successful_additions + 1,
            "DAG size should be correct"
        );
    });
}
