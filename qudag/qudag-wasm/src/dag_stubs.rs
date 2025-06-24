//! DAG stub implementations for WASM
//!
//! These provide DAG functionality adapted for WASM environments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// DAG vertex for WASM
#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub struct Vertex {
    id: String,
    data: Vec<u8>,
    parents: Vec<String>,
    timestamp: f64,
    signature: Vec<u8>,
}

#[wasm_bindgen]
impl Vertex {
    /// Create new vertex
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>, parents: Vec<String>) -> Self {
        let id = blake3::hash(&data).to_hex().to_string();
        let timestamp = js_sys::Date::now();

        Self {
            id,
            data,
            parents,
            timestamp,
            signature: Vec::new(),
        }
    }

    /// Get vertex ID
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Get vertex data
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Get parent IDs
    pub fn parents(&self) -> Vec<String> {
        self.parents.clone()
    }

    /// Get timestamp
    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Sign the vertex
    pub fn sign(&mut self, signature: Vec<u8>) {
        self.signature = signature;
    }
}

/// DAG implementation for WASM
#[wasm_bindgen]
pub struct DAG {
    vertices: HashMap<String, Vertex>,
    tips: Vec<String>,
}

#[wasm_bindgen]
impl DAG {
    /// Create new DAG
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            tips: Vec::new(),
        }
    }

    /// Add vertex to DAG
    #[wasm_bindgen(js_name = "addVertex")]
    pub fn add_vertex(&mut self, vertex: &Vertex) -> Result<(), JsError> {
        // Validate parents exist
        for parent_id in &vertex.parents {
            if !self.vertices.contains_key(parent_id) {
                return Err(JsError::new(&format!(
                    "Parent vertex {} not found",
                    parent_id
                )));
            }
        }

        // Update tips
        self.tips.retain(|tip| !vertex.parents.contains(tip));
        self.tips.push(vertex.id.clone());

        // Add vertex
        self.vertices.insert(vertex.id.clone(), vertex.clone());

        web_sys::console::log_1(&format!("Added vertex {} to DAG", vertex.id).into());
        Ok(())
    }

    /// Get vertex by ID
    #[wasm_bindgen(js_name = "getVertex")]
    pub fn get_vertex(&self, id: &str) -> Option<Vertex> {
        self.vertices.get(id).cloned()
    }

    /// Get current tips
    #[wasm_bindgen(js_name = "getTips")]
    pub fn get_tips(&self) -> Vec<String> {
        self.tips.clone()
    }

    /// Get DAG statistics
    #[wasm_bindgen(js_name = "getStats")]
    pub fn get_stats(&self) -> Result<JsValue, JsError> {
        let stats = serde_json::json!({
            "total_vertices": self.vertices.len(),
            "tips_count": self.tips.len(),
            "tips": self.tips,
        });

        Ok(serde_wasm_bindgen::to_value(&stats)?)
    }

    /// Check if vertex exists
    #[wasm_bindgen(js_name = "hasVertex")]
    pub fn has_vertex(&self, id: &str) -> bool {
        self.vertices.contains_key(id)
    }
}

/// Consensus mechanism stub for WASM
#[wasm_bindgen]
pub struct Consensus {
    confidence_threshold: f64,
}

#[wasm_bindgen]
impl Consensus {
    /// Create new consensus instance
    #[wasm_bindgen(constructor)]
    pub fn new(confidence_threshold: f64) -> Self {
        Self {
            confidence_threshold,
        }
    }

    /// Calculate vertex confidence (stub)
    #[wasm_bindgen(js_name = "calculateConfidence")]
    pub fn calculate_confidence(&self, vertex_id: &str, dag: &DAG) -> f64 {
        // Simple stub: confidence based on depth from tips
        if dag.tips.contains(&vertex_id.to_string()) {
            1.0
        } else if dag.has_vertex(vertex_id) {
            0.8 // Simplified confidence for non-tip vertices
        } else {
            0.0
        }
    }

    /// Resolve conflict between vertices (stub)
    #[wasm_bindgen(js_name = "resolveConflict")]
    pub fn resolve_conflict(&self, vertex1: &str, vertex2: &str, dag: &DAG) -> String {
        let conf1 = self.calculate_confidence(vertex1, dag);
        let conf2 = self.calculate_confidence(vertex2, dag);

        if conf1 > conf2 {
            vertex1.to_string()
        } else {
            vertex2.to_string()
        }
    }
}

/// DAG traversal utilities
#[wasm_bindgen]
pub struct DAGTraversal;

#[wasm_bindgen]
impl DAGTraversal {
    /// Get ancestors of a vertex
    #[wasm_bindgen(js_name = "getAncestors")]
    pub fn get_ancestors(vertex_id: &str, dag: &DAG, max_depth: Option<u32>) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut to_visit = vec![(vertex_id.to_string(), 0u32)];
        let mut visited = std::collections::HashSet::new();

        while let Some((current_id, depth)) = to_visit.pop() {
            if let Some(max_d) = max_depth {
                if depth > max_d {
                    continue;
                }
            }

            if !visited.insert(current_id.clone()) {
                continue;
            }

            if let Some(vertex) = dag.get_vertex(&current_id) {
                ancestors.push(current_id);
                for parent in vertex.parents() {
                    to_visit.push((parent, depth + 1));
                }
            }
        }

        ancestors
    }

    /// Get descendants (not typically available in DAG, stub only)
    #[wasm_bindgen(js_name = "getDescendants")]
    pub fn get_descendants(_vertex_id: &str, _dag: &DAG) -> Vec<String> {
        // In a true DAG, we don't track descendants
        Vec::new()
    }
}

/// Export DAG-related utilities for internal use
pub mod internal {
    use super::*;

    /// Create genesis vertex
    pub fn create_genesis() -> Vertex {
        Vertex::new(b"GENESIS".to_vec(), Vec::new())
    }

    /// Create test DAG with sample data
    pub fn create_test_dag() -> DAG {
        let mut dag = DAG::new();
        let genesis = create_genesis();
        dag.add_vertex(&genesis).unwrap();
        dag
    }
}
