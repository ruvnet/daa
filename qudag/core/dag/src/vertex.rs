//! DAG vertex implementation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during vertex operations.
#[derive(Debug, Error)]
pub enum VertexError {
    /// Invalid parent reference
    #[error("Invalid parent reference")]
    InvalidParent,

    /// Parent not found
    #[error("Parent not found")]
    ParentNotFound,

    /// Invalid payload format
    #[error("Invalid payload format")]
    InvalidPayload,

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Vertex creation failed
    #[error("Vertex creation failed")]
    CreationFailed,
}

/// Unique vertex identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(Vec<u8>);

impl Default for VertexId {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexId {
    /// Creates a new vertex ID with random bytes
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Self(timestamp.to_be_bytes().to_vec())
    }

    /// Creates a vertex ID from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Gets the raw bytes of the vertex ID
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// DAG vertex containing a message payload and references to parent vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    /// Unique vertex identifier
    pub id: VertexId,

    /// References to parent vertices
    pub parents: Vec<VertexId>,

    /// Message payload
    pub payload: Vec<u8>,

    /// Vertex timestamp
    pub timestamp: u64,

    /// Cryptographic signature
    pub signature: Vec<u8>,
}

impl Vertex {
    /// Creates a new vertex with the given parameters
    pub fn new(
        id: VertexId,
        payload: Vec<u8>,
        parents: std::collections::HashSet<VertexId>,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            parents: parents.into_iter().collect(),
            payload,
            timestamp,
            signature: Vec::new(), // Empty signature for now
        }
    }

    /// Gets the parent vertices as a set
    pub fn parents(&self) -> std::collections::HashSet<VertexId> {
        self.parents.iter().cloned().collect()
    }
}

/// Vertex trait defining the interface for creating and validating vertices.
pub trait VertexOps {
    /// Create a new vertex with the given payload and parent references.
    fn create(payload: Vec<u8>, parents: Vec<VertexId>) -> Result<Vertex, VertexError>;

    /// Validate a vertex's structure and signature.
    fn validate(&self) -> Result<bool, VertexError>;

    /// Get the vertex's score based on the DAG topology.
    fn score(&self) -> f64;

    /// Check if vertex is a tip (has no children).
    fn is_tip(&self) -> bool;
}
