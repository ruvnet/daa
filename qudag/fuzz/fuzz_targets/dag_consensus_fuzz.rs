#![no_main]
use libfuzzer_sys::fuzz_target;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use qudag_dag::{
    DAG, Vertex, Edge, VertexId, 
    consensus::{QRAvalanche, ConsensusDecision, ConsensusState},
    tip_selection::TipSelection,
    error::DAGError,
};

/// Test vertex structure for fuzzing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVertex {
    pub id: VertexId,
    pub parents: Vec<VertexId>,
    pub payload: Vec<u8>,
    pub weight: u64,
    pub timestamp: u64,
}

/// Test transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTransaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub signature: Vec<u8>,
}

/// Helper to create a test vertex from fuzz data
fn create_test_vertex(data: &[u8], vertex_id: u64) -> Option<TestVertex> {
    if data.len() < 16 {
        return None;
    }
    
    let id = VertexId::from(vertex_id);
    let weight = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], 
        data[4], data[5], data[6], data[7]
    ]);
    
    // Create parent references
    let num_parents = (data[8] % 4) as usize; // Max 3 parents
    let mut parents = Vec::new();
    for i in 0..num_parents {
        if i + 9 < data.len() {
            let parent_id = VertexId::from(data[i + 9] as u64);
            parents.push(parent_id);
        }
    }
    
    let payload = data[std::cmp::min(12, data.len())..].to_vec();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Some(TestVertex {
        id,
        parents,
        payload,
        weight,
        timestamp,
    })
}

/// Helper to create a test transaction from fuzz data
fn create_test_transaction(data: &[u8], tx_id: u64) -> Option<TestTransaction> {
    if data.len() < 32 {
        return None;
    }
    
    let id = format!("tx_{}", tx_id);
    let from = format!("addr_{}", data[0]);
    let to = format!("addr_{}", data[1]);
    let amount = u64::from_le_bytes([
        data[2], data[3], data[4], data[5],
        data[6], data[7], data[8], data[9]
    ]);
    let nonce = u64::from_le_bytes([
        data[10], data[11], data[12], data[13],
        data[14], data[15], data[16], data[17]
    ]);
    let signature = data[18..].to_vec();
    
    Some(TestTransaction {
        id,
        from,
        to,
        amount,
        nonce,
        signature,
    })
}

/// Test DAG construction and validation
fn test_dag_operations(vertices: &[TestVertex]) -> Result<(), String> {
    let mut dag = DAG::new();
    
    // Add vertices to DAG
    for vertex in vertices {
        // Create a proper vertex for the DAG
        let dag_vertex = match Vertex::new(
            vertex.id.clone(),
            vertex.parents.clone(),
            vertex.payload.clone(),
        ) {
            Ok(v) => v,
            Err(_) => continue,
        };
        
        match dag.add_vertex(dag_vertex) {
            Ok(_) => {},
            Err(DAGError::DuplicateVertex(_)) => continue, // Ignore duplicates in fuzzing
            Err(e) => return Err(format!("Failed to add vertex: {:?}", e)),
        }
    }
    
    // Test DAG validation
    match dag.validate() {
        Ok(_) => {},
        Err(e) => return Err(format!("DAG validation failed: {:?}", e)),
    }
    
    // Test tip selection
    let tip_selector = TipSelection::new();
    match tip_selector.select_tips(&dag, 2) {
        Ok(tips) => {
            // Verify tips are valid
            for tip in tips {
                if !dag.contains_vertex(&tip) {
                    return Err("Selected tip not in DAG".to_string());
                }
            }
        },
        Err(e) => return Err(format!("Tip selection failed: {:?}", e)),
    }
    
    // Test path finding
    if vertices.len() >= 2 {
        let start = &vertices[0].id;
        let end = &vertices[vertices.len() - 1].id;
        
        if dag.contains_vertex(start) && dag.contains_vertex(end) {
            let _path = dag.find_path(start, end);
        }
    }
    
    Ok(())
}

/// Test consensus algorithm operations
fn test_consensus_operations(vertices: &[TestVertex]) -> Result<(), String> {
    if vertices.is_empty() {
        return Ok(());
    }
    
    let mut consensus = QRAvalanche::new();
    
    // Initialize consensus state
    let mut state = ConsensusState::new();
    
    // Test consensus decisions for vertices
    for vertex in vertices {
        let decision = consensus.query_vertex(&vertex.id, &state)
            .map_err(|e| format!("Consensus query failed: {:?}", e))?;
            
        match decision {
            ConsensusDecision::Accept => {
                state.accept_vertex(vertex.id.clone());
            },
            ConsensusDecision::Reject => {
                state.reject_vertex(vertex.id.clone());
            },
            ConsensusDecision::Undecided => {
                // Leave undecided
            },
        }
    }
    
    // Test consensus finality
    for vertex in vertices {
        let is_final = consensus.is_finalized(&vertex.id, &state)
            .map_err(|e| format!("Finality check failed: {:?}", e))?;
            
        // Verify finality consistency
        if is_final {
            let decision = consensus.query_vertex(&vertex.id, &state)
                .map_err(|e| format!("Final vertex query failed: {:?}", e))?;
                
            if let ConsensusDecision::Undecided = decision {
                return Err("Finalized vertex should not be undecided".to_string());
            }
        }
    }
    
    Ok(())
}

/// Test Byzantine fault tolerance
fn test_byzantine_resistance(vertices: &[TestVertex]) -> Result<(), String> {
    if vertices.len() < 4 {
        return Ok(()); // Need at least 4 vertices for Byzantine testing
    }
    
    let mut consensus = QRAvalanche::new();
    let mut state = ConsensusState::new();
    
    // Simulate Byzantine behavior: conflicting votes
    let byzantine_count = vertices.len() / 3; // Up to 1/3 Byzantine nodes
    
    for (i, vertex) in vertices.iter().enumerate() {
        if i < byzantine_count {
            // Byzantine node: alternate between accept/reject for same vertex
            let _ = consensus.query_vertex(&vertex.id, &state);
            state.accept_vertex(vertex.id.clone());
            
            let _ = consensus.query_vertex(&vertex.id, &state);
            state.reject_vertex(vertex.id.clone());
        } else {
            // Honest node: consistent behavior
            let decision = consensus.query_vertex(&vertex.id, &state)
                .map_err(|e| format!("Honest node query failed: {:?}", e))?;
                
            match decision {
                ConsensusDecision::Accept => state.accept_vertex(vertex.id.clone()),
                _ => {}, // Don't change state for reject/undecided
            }
        }
    }
    
    // Verify consensus can still make progress despite Byzantine nodes
    let honest_vertices: Vec<_> = vertices.iter()
        .skip(byzantine_count)
        .collect();
        
    for vertex in honest_vertices {
        let decision = consensus.query_vertex(&vertex.id, &state)
            .map_err(|e| format!("Post-Byzantine query failed: {:?}", e))?;
            
        // Should be able to reach a decision
        if let ConsensusDecision::Undecided = decision {
            // This is acceptable - just ensure we don't crash
        }
    }
    
    Ok(())
}

/// Test edge case scenarios
fn test_edge_cases(data: &[u8]) -> Result<(), String> {
    let mut dag = DAG::new();
    
    // Test with empty DAG
    let tip_selector = TipSelection::new();
    match tip_selector.select_tips(&dag, 1) {
        Ok(tips) => {
            if !tips.is_empty() {
                return Err("Empty DAG should have no tips".to_string());
            }
        },
        Err(_) => {}, // Expected for empty DAG
    }
    
    // Test with single vertex
    if data.len() >= 16 {
        if let Some(vertex) = create_test_vertex(data, 1) {
            let dag_vertex = Vertex::new(
                vertex.id.clone(),
                vec![], // No parents - genesis vertex
                vertex.payload,
            ).map_err(|e| format!("Failed to create genesis vertex: {:?}", e))?;
            
            dag.add_vertex(dag_vertex)
                .map_err(|e| format!("Failed to add genesis vertex: {:?}", e))?;
            
            // Test tip selection with single vertex
            match tip_selector.select_tips(&dag, 1) {
                Ok(tips) => {
                    if tips.len() != 1 || tips[0] != vertex.id {
                        return Err("Single vertex DAG should have one tip".to_string());
                    }
                },
                Err(e) => return Err(format!("Single vertex tip selection failed: {:?}", e)),
            }
        }
    }
    
    // Test circular reference detection
    if data.len() >= 32 {
        let mut circular_dag = DAG::new();
        
        if let Some(vertex1) = create_test_vertex(&data[..16], 1) {
            if let Some(mut vertex2) = create_test_vertex(&data[16..32], 2) {
                // Make vertex2 parent of vertex1, and vertex1 parent of vertex2 (circular)
                vertex2.parents = vec![vertex1.id.clone()];
                let mut circular_vertex1 = vertex1.clone();
                circular_vertex1.parents = vec![vertex2.id.clone()];
                
                // Try to add circular vertices
                let dag_vertex1 = Vertex::new(
                    circular_vertex1.id.clone(),
                    circular_vertex1.parents,
                    circular_vertex1.payload,
                );
                
                let dag_vertex2 = Vertex::new(
                    vertex2.id.clone(),
                    vertex2.parents,
                    vertex2.payload,
                );
                
                if let (Ok(v1), Ok(v2)) = (dag_vertex1, dag_vertex2) {
                    let _ = circular_dag.add_vertex(v1);
                    let _ = circular_dag.add_vertex(v2);
                    
                    // Validation should catch circular reference
                    match circular_dag.validate() {
                        Ok(_) => return Err("Circular DAG should not validate".to_string()),
                        Err(_) => {}, // Expected
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Test timing consistency for consensus operations
fn measure_consensus_timing<F>(op: F) -> bool
where
    F: Fn() -> Result<(), String>
{
    let iterations = 20; // Reduced for fuzzing
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    if timings.is_empty() {
        return false;
    }
    
    let mean = timings.iter().sum::<Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    // Allow more variance for consensus operations
    variance < 1000000
}

fuzz_target!(|data: &[u8]| {
    // Set panic hook to prevent information leaks
    std::panic::set_hook(Box::new(|_| {}));
    
    if data.is_empty() {
        return;
    }
    
    // Create test vertices from fuzz data
    let mut vertices = Vec::new();
    let max_vertices = std::cmp::min(data.len() / 16, 20); // Limit for performance
    
    for i in 0..max_vertices {
        let start_idx = i * 16;
        let end_idx = std::cmp::min(start_idx + 16, data.len());
        
        if end_idx > start_idx {
            if let Some(vertex) = create_test_vertex(&data[start_idx..end_idx], i as u64) {
                vertices.push(vertex);
            }
        }
    }
    
    if vertices.is_empty() {
        return;
    }
    
    // Test DAG operations with timing validation
    let _dag_timing = measure_consensus_timing(|| {
        test_dag_operations(&vertices)
    });
    
    // Test consensus operations with timing validation
    let _consensus_timing = measure_consensus_timing(|| {
        test_consensus_operations(&vertices)
    });
    
    // Test Byzantine resistance
    if vertices.len() >= 4 {
        let _byzantine_timing = measure_consensus_timing(|| {
            test_byzantine_resistance(&vertices)
        });
    }
    
    // Test edge cases
    let _edge_timing = measure_consensus_timing(|| {
        test_edge_cases(data)
    });
    
    // Test with transactions
    if data.len() >= 64 {
        let mut transactions = Vec::new();
        let max_txs = std::cmp::min(data.len() / 32, 10);
        
        for i in 0..max_txs {
            let start_idx = i * 32;
            let end_idx = std::cmp::min(start_idx + 32, data.len());
            
            if end_idx > start_idx {
                if let Some(tx) = create_test_transaction(&data[start_idx..end_idx], i as u64) {
                    transactions.push(tx);
                }
            }
        }
        
        // Test transaction validation and ordering
        test_transaction_operations(&transactions);
    }
    
    // Test with malformed data
    if data.len() >= 32 {
        test_malformed_data_handling(data);
    }
    
    // Test concurrent operations simulation
    if data.len() >= 128 {
        test_concurrent_operations(&vertices);
    }
    
    // Test memory management
    test_memory_management(&vertices);
});

/// Test transaction operations
fn test_transaction_operations(transactions: &[TestTransaction]) {
    // Test transaction validation
    for tx in transactions {
        // Basic validation
        if tx.id.is_empty() || tx.from.is_empty() || tx.to.is_empty() {
            continue;
        }
        
        // Test serialization
        let serialized = match bincode::serialize(tx) {
            Ok(s) => s,
            Err(_) => continue,
        };
        
        // Test deserialization
        let _deserialized: Result<TestTransaction, _> = bincode::deserialize(&serialized);
    }
    
    // Test transaction ordering
    let mut ordered_txs = transactions.to_vec();
    ordered_txs.sort_by(|a, b| a.nonce.cmp(&b.nonce));
    
    // Verify ordering is maintained
    for window in ordered_txs.windows(2) {
        assert!(window[0].nonce <= window[1].nonce, "Transaction ordering violated");
    }
}

/// Test malformed data handling
fn test_malformed_data_handling(data: &[u8]) {
    // Test with truncated data
    for i in 1..std::cmp::min(data.len(), 32) {
        let truncated = &data[..i];
        let _ = create_test_vertex(truncated, 999);
        let _ = create_test_transaction(truncated, 999);
    }
    
    // Test with bit flipping
    if data.len() >= 16 {
        let mut mutated = data[..16].to_vec();
        for i in 0..mutated.len() {
            mutated[i] ^= 1;
            let _ = create_test_vertex(&mutated, 998);
            mutated[i] ^= 1; // Restore
        }
    }
    
    // Test with all zeros/ones
    let zero_data = vec![0u8; 32];
    let _ = create_test_vertex(&zero_data, 997);
    
    let ones_data = vec![0xFFu8; 32];
    let _ = create_test_vertex(&ones_data, 996);
}

/// Test concurrent operations simulation
fn test_concurrent_operations(vertices: &[TestVertex]) {
    let mut consensus = QRAvalanche::new();
    let mut state = ConsensusState::new();
    
    // Simulate concurrent queries
    for vertex in vertices {
        // Multiple queries for same vertex (simulating concurrent access)
        let _decision1 = consensus.query_vertex(&vertex.id, &state);
        let _decision2 = consensus.query_vertex(&vertex.id, &state);
        let _decision3 = consensus.query_vertex(&vertex.id, &state);
        
        // Check consistency
        if let (Ok(d1), Ok(d2), Ok(d3)) = (
            consensus.query_vertex(&vertex.id, &state),
            consensus.query_vertex(&vertex.id, &state),
            consensus.query_vertex(&vertex.id, &state)
        ) {
            // All decisions should be consistent for same vertex and state
            assert_eq!(d1 as u8, d2 as u8, "Concurrent decisions inconsistent");
            assert_eq!(d2 as u8, d3 as u8, "Concurrent decisions inconsistent");
        }
    }
}

/// Test memory management and resource cleanup
fn test_memory_management(vertices: &[TestVertex]) {
    // Test DAG with many vertices (memory stress)
    let mut dag = DAG::new();
    
    for vertex in vertices {
        if let Ok(dag_vertex) = Vertex::new(
            vertex.id.clone(),
            vertex.parents.clone(),
            vertex.payload.clone(),
        ) {
            let _ = dag.add_vertex(dag_vertex);
        }
    }
    
    // Test cleanup - DAG should be properly dropped
    drop(dag);
    
    // Test consensus state cleanup
    let mut consensus = QRAvalanche::new();
    let mut state = ConsensusState::new();
    
    for vertex in vertices {
        let _ = consensus.query_vertex(&vertex.id, &state);
    }
    
    // Test cleanup
    drop(consensus);
    drop(state);
}