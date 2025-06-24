#!/usr/bin/env rust-script
//! Quick DAG benchmark runner to measure consensus performance
//! 
//! ```cargo
//! [dependencies]
//! qudag-dag = { path = "core/dag" }
//! tokio = { version = "1.45", features = ["full"] }
//! ```

use std::time::{Duration, Instant};
use qudag_dag::{Graph, Vertex, Edge, DagConsensus};

fn main() {
    println!("QuDAG Consensus Performance Benchmarks");
    println!("=====================================\n");
    
    // Test different network sizes
    let network_sizes = vec![10, 50, 100, 250, 500];
    
    for node_count in network_sizes {
        println!("Testing with {} nodes:", node_count);
        
        let mut finality_times = Vec::new();
        let mut throughputs = Vec::new();
        
        // Run 10 iterations
        for _ in 0..10 {
            let (graph, vertex_count) = create_test_graph(node_count);
            let mut consensus = DagConsensus::new();
            
            let start = Instant::now();
            let result = consensus.process_round(&graph);
            let elapsed = start.elapsed();
            
            finality_times.push(elapsed);
            
            let throughput = vertex_count as f64 / elapsed.as_secs_f64();
            throughputs.push(throughput);
        }
        
        // Calculate statistics
        finality_times.sort();
        let p50 = finality_times[finality_times.len() / 2];
        let p99 = finality_times[(finality_times.len() as f64 * 0.99) as usize];
        let avg_throughput: f64 = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
        
        println!("  - Finality time (p50): {:?}", p50);
        println!("  - Finality time (p99): {:?}", p99);
        println!("  - Average throughput: {:.2} vertices/sec", avg_throughput);
        println!("  - Sub-second finality: {}", if p99 < Duration::from_secs(1) { "✓ PASS" } else { "✗ FAIL" });
        println!();
    }
    
    // Test scalability with increasing DAG sizes
    println!("\nScalability Test (Fixed 100 nodes):");
    println!("===================================");
    
    let dag_sizes = vec![1000, 5000, 10000, 50000];
    
    for dag_size in dag_sizes {
        let (graph, _) = create_large_graph(100, dag_size);
        let mut consensus = DagConsensus::new();
        
        let start = Instant::now();
        let _ = consensus.process_round(&graph);
        let elapsed = start.elapsed();
        
        let vertices_per_sec = dag_size as f64 / elapsed.as_secs_f64();
        
        println!("  DAG size: {} vertices", dag_size);
        println!("  - Processing time: {:?}", elapsed);
        println!("  - Throughput: {:.2} vertices/sec", vertices_per_sec);
        println!("  - Memory estimate: ~{} MB", (dag_size * 320) / (1024 * 1024));
        println!();
    }
}

fn create_test_graph(node_count: usize) -> (Graph, usize) {
    let mut graph = Graph::new();
    let vertex_count = node_count * 2; // 2 vertices per node
    
    // Create vertices
    for i in 0..vertex_count {
        let vertex = Vertex::new(
            i as u64,
            format!("vertex_{}", i),
            vec![0u8; 256], // Simulate message payload
            (i % node_count) as u64, // Distribute across nodes
        );
        graph.add_vertex(vertex);
    }
    
    // Create edges (each vertex references 1-3 previous vertices)
    for i in 1..vertex_count {
        let ref_count = 1 + (i % 3);
        for j in 0..ref_count {
            if j < i {
                let edge = Edge::new(
                    j as u64,
                    i as u64,
                    1.0,
                );
                graph.add_edge(edge);
            }
        }
    }
    
    (graph, vertex_count)
}

fn create_large_graph(node_count: usize, vertex_count: usize) -> (Graph, usize) {
    let mut graph = Graph::new();
    
    // Create vertices
    for i in 0..vertex_count {
        let vertex = Vertex::new(
            i as u64,
            format!("vertex_{}", i),
            vec![0u8; 256],
            (i % node_count) as u64,
        );
        graph.add_vertex(vertex);
    }
    
    // Create edges with limited connectivity to avoid O(n²) complexity
    for i in 1..vertex_count {
        let max_refs = 3.min(i);
        for j in 0..max_refs {
            let parent = i - j - 1;
            let edge = Edge::new(
                parent as u64,
                i as u64,
                1.0,
            );
            graph.add_edge(edge);
        }
    }
    
    (graph, vertex_count)
}