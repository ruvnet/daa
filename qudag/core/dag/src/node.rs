use blake3::Hash;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Serializable wrapper for blake3::Hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerializableHash(pub [u8; 32]);

impl From<Hash> for SerializableHash {
    fn from(hash: Hash) -> Self {
        SerializableHash(*hash.as_bytes())
    }
}

impl From<SerializableHash> for Hash {
    fn from(hash: SerializableHash) -> Self {
        Hash::from(hash.0)
    }
}

/// Represents the state of a node in the DAG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// Node has been created but not yet verified
    Pending,
    /// Node has been verified and is part of the DAG
    Verified,
    /// Node has achieved finality through consensus
    Final,
    /// Node has been rejected by consensus
    Rejected,
}

/// A node in the DAG containing a transaction or consensus message
///
/// # Examples
///
/// ```rust
/// use qudag_dag::{Node, NodeState};
///
/// // Create a new node with payload and no parents (genesis node)
/// let payload = b"Hello, DAG!".to_vec();
/// let node = Node::new(payload.clone(), vec![]);
///
/// assert_eq!(node.payload(), &payload);
/// assert_eq!(node.state(), NodeState::Pending);
/// assert!(node.parents().is_empty());
///
/// // Hash is computed deterministically
/// let hash = node.hash();
/// assert_eq!(hash.as_bytes().len(), 32);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier hash for this node
    hash: SerializableHash,
    /// Payload contained in this node
    payload: Vec<u8>,
    /// Current state of this node
    state: NodeState,
    /// Timestamp when node was created
    timestamp: SystemTime,
    /// Parent node hashes
    parents: Vec<SerializableHash>,
}

impl Node {
    /// Creates a new node with the given payload and parents
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qudag_dag::Node;
    ///
    /// // Create a genesis node (no parents)
    /// let payload = b"Genesis block".to_vec();
    /// let node = Node::new(payload, vec![]);
    ///
    /// // Create a child node
    /// let parent_hash = node.hash();
    /// let child_payload = b"Child block".to_vec();
    /// let child_node = Node::new(child_payload, vec![parent_hash]);
    /// ```
    pub fn new(payload: Vec<u8>, parents: Vec<Hash>) -> Self {
        let timestamp = SystemTime::now();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&payload);
        for parent in &parents {
            hasher.update(parent.as_bytes());
        }
        let hash = hasher.finalize();

        Self {
            hash: hash.into(),
            payload,
            state: NodeState::Pending,
            timestamp,
            parents: parents.into_iter().map(|h| h.into()).collect(),
        }
    }

    /// Returns the node's unique hash
    pub fn hash(&self) -> Hash {
        self.hash.clone().into()
    }

    /// Returns reference to node's payload
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns current state of the node
    pub fn state(&self) -> NodeState {
        self.state
    }

    /// Returns reference to parent hashes (converted)
    pub fn parents(&self) -> Vec<Hash> {
        self.parents.iter().map(|h| h.clone().into()).collect()
    }

    /// Updates node state if transition is valid
    pub fn update_state(&mut self, new_state: NodeState) -> crate::Result<()> {
        match (self.state, new_state) {
            // Valid transitions
            (NodeState::Pending, NodeState::Verified)
            | (NodeState::Verified, NodeState::Final)
            | (NodeState::Pending, NodeState::Rejected)
            | (NodeState::Verified, NodeState::Rejected) => {
                self.state = new_state;
                Ok(())
            }
            // Invalid transitions
            _ => Err(crate::DagError::InvalidStateTransition(format!(
                "{:?} -> {:?}",
                self.state, new_state
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let payload = vec![1, 2, 3];
        let parents = vec![blake3::hash(b"parent1"), blake3::hash(b"parent2")];
        let node = Node::new(payload.clone(), parents.clone());

        assert_eq!(node.state(), NodeState::Pending);
        assert_eq!(node.payload(), &payload);
        assert_eq!(node.parents(), &parents);
    }

    #[test]
    fn test_valid_state_transitions() {
        let mut node = Node::new(vec![1, 2, 3], vec![]);

        // Pending -> Verified
        assert!(node.update_state(NodeState::Verified).is_ok());
        assert_eq!(node.state(), NodeState::Verified);

        // Verified -> Final
        assert!(node.update_state(NodeState::Final).is_ok());
        assert_eq!(node.state(), NodeState::Final);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut node = Node::new(vec![1, 2, 3], vec![]);

        // Can't go from Pending to Final
        assert!(node.update_state(NodeState::Final).is_err());

        // Update to Verified
        assert!(node.update_state(NodeState::Verified).is_ok());

        // Can't go back to Pending
        assert!(node.update_state(NodeState::Pending).is_err());
    }
}
