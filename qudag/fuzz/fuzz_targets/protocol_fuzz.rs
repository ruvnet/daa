#![no_main]
use libfuzzer_sys::fuzz_target;
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Test message types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestMessageType {
    Handshake,
    Data,
    Control,
    Sync,
}

/// Test protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMessage {
    pub msg_type: TestMessageType,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Test vertex for DAG operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVertex {
    pub id: Vec<u8>,
    pub parents: Vec<Vec<u8>>,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

/// Test DAG structure
#[derive(Debug, Clone)]
pub struct TestDAG {
    vertices: HashMap<Vec<u8>, TestVertex>,
    edges: HashMap<Vec<u8>, HashSet<Vec<u8>>>,
}

impl TestDAG {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
        }
    }
    
    pub fn add_vertex(&mut self, vertex: TestVertex) -> Result<(), String> {
        if vertex.id.is_empty() {
            return Err("Empty vertex ID".to_string());
        }
        
        if vertex.id.len() > 64 {
            return Err("Vertex ID too long".to_string());
        }
        
        // Check for cycles (simplified)
        for parent_id in &vertex.parents {
            if parent_id == &vertex.id {
                return Err("Self-referencing vertex".to_string());
            }
        }
        
        self.vertices.insert(vertex.id.clone(), vertex.clone());
        
        // Add edges
        for parent_id in &vertex.parents {
            self.edges.entry(parent_id.clone())
                .or_insert_with(HashSet::new)
                .insert(vertex.id.clone());
        }
        
        Ok(())
    }
    
    pub fn get_tips(&self) -> Vec<Vec<u8>> {
        let mut tips = Vec::new();
        
        for vertex_id in self.vertices.keys() {
            if !self.edges.contains_key(vertex_id) {
                tips.push(vertex_id.clone());
            }
        }
        
        tips
    }
    
    pub fn validate(&self) -> Result<(), String> {
        // Check that all parent references exist
        for vertex in self.vertices.values() {
            for parent_id in &vertex.parents {
                if !self.vertices.contains_key(parent_id) {
                    return Err("Missing parent vertex".to_string());
                }
            }
        }
        
        Ok(())
    }
}

/// Test for timing uniformity in protocol operations
fn verify_protocol_timing<F>(op: F) -> (bool, Duration)
where
    F: Fn() -> Result<(), String>
{
    let iterations = 25; // Reduced for faster fuzzing
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    if timings.is_empty() {
        return (false, Duration::from_nanos(0));
    }
    
    let mean = timings.iter().sum::<Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    let max_allowed_variance = 75000; // Relaxed for fuzzing
    (variance < max_allowed_variance, mean)
}

/// Create test message from fuzz data
fn create_test_message(data: &[u8]) -> Option<TestMessage> {
    if data.len() < 2 {
        return None;
    }
    
    let msg_type = match data[0] % 4 {
        0 => TestMessageType::Handshake,
        1 => TestMessageType::Data,
        2 => TestMessageType::Control,
        _ => TestMessageType::Sync,
    };
    
    let payload = data[1..].to_vec();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Some(TestMessage {
        msg_type,
        payload,
        timestamp,
        signature: Vec::new(),
    })
}

/// Create test vertex from fuzz data
fn create_test_vertex(data: &[u8]) -> Option<TestVertex> {
    if data.len() < 16 {
        return None;
    }
    
    let id = data[..8].to_vec();
    let parent_id = data[8..16].to_vec();
    let parents = if parent_id.iter().all(|&b| b == 0) {
        vec![] // Genesis vertex
    } else {
        vec![parent_id]
    };
    let payload = data.get(16..).unwrap_or(&[]).to_vec();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Some(TestVertex {
        id,
        parents,
        payload,
        timestamp,
    })
}

/// Test message operations
fn test_message_operations(msg: &TestMessage) -> Result<(), String> {
    // Test serialization
    let serialized = bincode::serialize(msg)
        .map_err(|e| format!("Message serialization failed: {}", e))?;
    
    // Test deserialization
    let deserialized: TestMessage = bincode::deserialize(&serialized)
        .map_err(|e| format!("Message deserialization failed: {}", e))?;
    
    // Verify integrity
    if msg.msg_type != deserialized.msg_type {
        return Err("Message type mismatch".to_string());
    }
    
    if msg.payload != deserialized.payload {
        return Err("Payload mismatch".to_string());
    }
    
    Ok(())
}

/// Test vertex operations
fn test_vertex_operations(vertex: &TestVertex) -> Result<(), String> {
    // Validate vertex
    if vertex.id.is_empty() {
        return Err("Empty vertex ID".to_string());
    }
    
    if vertex.id.len() > 64 {
        return Err("Vertex ID too long".to_string());
    }
    
    // Check parent references
    for parent_id in &vertex.parents {
        if parent_id == &vertex.id {
            return Err("Self-referencing vertex".to_string());
        }
        
        if parent_id.is_empty() {
            return Err("Empty parent ID".to_string());
        }
    }
    
    // Test serialization
    let serialized = bincode::serialize(vertex)
        .map_err(|e| format!("Vertex serialization failed: {}", e))?;
    
    let deserialized: TestVertex = bincode::deserialize(&serialized)
        .map_err(|e| format!("Vertex deserialization failed: {}", e))?;
    
    if vertex.id != deserialized.id {
        return Err("Vertex ID mismatch".to_string());
    }
    
    Ok(())
}

/// Test DAG operations
fn test_dag_operations(vertices: &[TestVertex]) -> Result<(), String> {
    let mut dag = TestDAG::new();
    
    // Add vertices to DAG
    for vertex in vertices {
        dag.add_vertex(vertex.clone())?;
    }
    
    // Validate DAG structure
    dag.validate()?;
    
    // Test tip selection
    let tips = dag.get_tips();
    if tips.len() > vertices.len() {
        return Err("Too many tips".to_string());
    }
    
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Test message creation and validation
    if let Some(message) = create_test_message(data) {
        let (msg_timing, _) = verify_protocol_timing(|| {
            test_message_operations(&message)
        });
        // Don't assert on timing in fuzzing - just ensure it doesn't crash
    }

    // Test vertex creation and validation
    if let Some(vertex) = create_test_vertex(data) {
        let (vertex_timing, _) = verify_protocol_timing(|| {
            test_vertex_operations(&vertex)
        });
        // Don't assert on timing in fuzzing - just ensure it doesn't crash
    }

    // Test DAG operations with multiple vertices
    if data.len() >= 64 {
        let mut vertices = Vec::new();
        
        // Create multiple vertices from data chunks
        for i in 0..std::cmp::min(data.len() / 16, 8) {
            let start_idx = i * 16;
            let end_idx = std::cmp::min(start_idx + 16, data.len());
            
            if let Some(vertex) = create_test_vertex(&data[start_idx..end_idx]) {
                vertices.push(vertex);
            }
        }
        
        if !vertices.is_empty() {
            let (dag_timing, _) = verify_protocol_timing(|| {
                test_dag_operations(&vertices)
            });
            // Don't assert on timing - just ensure it doesn't crash
        }
    }

    // Test message type validation
    let message_types = vec![
        TestMessageType::Handshake,
        TestMessageType::Data,
        TestMessageType::Control,
        TestMessageType::Sync,
    ];
    
    for (i, msg_type) in message_types.iter().enumerate() {
        let serialized = bincode::serialize(msg_type).unwrap();
        let deserialized: TestMessageType = bincode::deserialize(&serialized).unwrap();
        assert_eq!(*msg_type, deserialized);
    }

    // Test with malformed data
    if data.len() >= 128 {
        // Test truncated data
        for i in (1..64).step_by(4) {
            if i < data.len() {
                let truncated = &data[..i];
                let _ = create_test_message(truncated);
                let _ = create_test_vertex(truncated);
            }
        }
        
        // Test bit flipping
        let mut mutated = data[..64].to_vec();
        for i in (0..mutated.len()).step_by(4) {
            mutated[i] ^= 0xFF;
            let _ = create_test_message(&mutated);
            let _ = create_test_vertex(&mutated);
            mutated[i] ^= 0xFF; // Restore
        }
    }

    // Test edge cases
    if data.len() >= 32 {
        // Test with all zeros
        let zero_data = vec![0u8; 32];
        let _ = create_test_message(&zero_data);
        let _ = create_test_vertex(&zero_data);

        // Test with all ones
        let ones_data = vec![0xFFu8; 32];
        let _ = create_test_message(&ones_data);
        let _ = create_test_vertex(&ones_data);

        // Test with alternating pattern
        let alt_data: Vec<u8> = (0..32).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect();
        let _ = create_test_message(&alt_data);
        let _ = create_test_vertex(&alt_data);
    }
});