//! Basic usage example for the qudag-dag module

use qudag_dag::*;
use std::collections::HashSet;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("QuDAG Core DAG Module - Basic Usage Example");

    // Create a new DAG consensus instance
    let mut dag = QrDag::new();
    println!("Created new DAG consensus instance");

    // Create some vertices to add to the DAG
    let genesis_id = VertexId::new();
    let genesis_vertex = Vertex::new(
        genesis_id.clone(),
        b"Genesis vertex".to_vec(),
        HashSet::new(), // No parents for genesis
    );

    // Add the genesis vertex
    dag.add_vertex(genesis_vertex)?;
    println!("Added genesis vertex to DAG");

    // Create a child vertex
    let child_id = VertexId::new();
    let mut parents = HashSet::new();
    parents.insert(genesis_id.clone());
    let child_vertex = Vertex::new(child_id.clone(), b"Child vertex".to_vec(), parents);

    // Add the child vertex
    dag.add_vertex(child_vertex)?;
    println!("Added child vertex to DAG");

    // Get and display current tips
    let tips = dag.get_tips();
    println!("Current DAG tips: {:?}", tips);

    // Check confidence/consensus status for vertices
    let genesis_str = String::from_utf8_lossy(genesis_id.as_bytes()).to_string();
    let child_str = String::from_utf8_lossy(child_id.as_bytes()).to_string();

    if let Some(genesis_status) = dag.get_confidence(&genesis_str) {
        println!("Genesis vertex status: {:?}", genesis_status);
    }

    if let Some(child_status) = dag.get_confidence(&child_str) {
        println!("Child vertex status: {:?}", child_status);
    }

    // Get the total order of vertices
    match dag.get_total_order() {
        Ok(order) => println!("Total order: {:?}", order),
        Err(e) => println!("Error getting total order: {}", e),
    }

    // Test message-based interface
    let message = b"Hello from message interface!".to_vec();
    dag.add_message(message.clone())?;
    println!("Added message via message interface");

    if dag.contains_message(&message) {
        println!("Message successfully stored in DAG");
    }

    // Demonstrate consensus configuration
    let config = ConsensusConfig {
        query_sample_size: 15,
        finality_threshold: 0.9,
        finality_timeout: std::time::Duration::from_secs(3),
        confirmation_depth: 5,
    };

    let _configured_dag = QrDag::with_config(config);
    println!("Created DAG with custom configuration");

    // Demonstrate different QR-Avalanche configurations
    let fast_config = QRAvalancheConfig::fast_finality();
    let secure_config = QRAvalancheConfig::high_security();

    println!(
        "Fast finality config - Beta: {}, Alpha: {}",
        fast_config.beta, fast_config.alpha
    );
    println!(
        "High security config - Beta: {}, Alpha: {}",
        secure_config.beta, secure_config.alpha
    );

    // Create a standalone consensus instance
    let mut consensus = QRAvalanche::with_config(fast_config);

    // Add participants to the consensus
    for i in 0..5 {
        let participant_id = VertexId::from_bytes(format!("participant_{}", i).into_bytes());
        consensus.add_participant(participant_id);
    }

    println!("Added 5 participants to consensus");

    // Process a vertex through consensus
    let test_vertex_id = VertexId::new();
    match consensus.process_vertex(test_vertex_id.clone()) {
        Ok(status) => println!("Vertex processed with status: {:?}", status),
        Err(e) => println!("Error processing vertex: {}", e),
    }

    // Get consensus metrics
    let metrics = consensus.get_metrics();
    println!(
        "Consensus metrics - Processed: {}, Throughput: {:.2} vertices/sec",
        metrics.total_vertices_processed, metrics.current_throughput
    );

    println!("Example completed successfully!");
    Ok(())
}
