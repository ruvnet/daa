use crate::consensus::ConsensusError;
use crate::vertex::VertexError;
use thiserror::Error;

/// Errors that can occur during DAG operations
#[derive(Error, Debug)]
pub enum DagError {
    /// Node already exists in the graph
    #[error("Node {0} already exists")]
    NodeExists(String),

    /// Node not found in the graph
    #[error("Node {0} not found")]
    NodeNotFound(String),

    /// Invalid edge - creates a cycle
    #[error("Edge would create cycle between {from} and {to}")]
    CycleDetected {
        /// Source node of the cycle-creating edge
        from: String,
        /// Target node of the cycle-creating edge
        to: String,
    },

    /// Parent node missing for edge
    #[error("Parent node {0} missing for edge")]
    MissingParent(String),

    /// Child node missing for edge
    #[error("Child node {0} missing for edge")]
    MissingChild(String),

    /// Invalid node state transition
    #[error("Invalid state transition for node {0}")]
    InvalidStateTransition(String),

    /// Consensus error
    #[error("Consensus error: {0}")]
    ConsensusError(String),

    /// Vertex error
    #[error("Vertex error: {0}")]
    VertexError(#[from] VertexError),
}

impl From<ConsensusError> for DagError {
    fn from(err: ConsensusError) -> Self {
        DagError::ConsensusError(err.to_string())
    }
}
