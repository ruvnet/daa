use crate::node::SerializableHash;
use blake3::Hash;
use serde::{Deserialize, Serialize};

/// Represents a directed edge in the DAG between two nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Edge {
    /// Hash of the parent node
    from: SerializableHash,
    /// Hash of the child node
    to: SerializableHash,
    /// Edge weight for consensus
    weight: u32,
}

impl Edge {
    /// Creates a new edge between parent and child nodes
    pub fn new(from: Hash, to: Hash) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            weight: 1,
        }
    }

    /// Returns the parent node hash
    pub fn from(&self) -> Hash {
        self.from.clone().into()
    }

    /// Returns the child node hash
    pub fn to(&self) -> Hash {
        self.to.clone().into()
    }

    /// Returns the edge weight
    pub fn weight(&self) -> u32 {
        self.weight
    }

    /// Increases the edge weight by 1
    pub fn increment_weight(&mut self) {
        self.weight = self.weight.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let from = blake3::hash(b"parent");
        let to = blake3::hash(b"child");
        let edge = Edge::new(from, to);

        assert_eq!(edge.from(), from);
        assert_eq!(edge.to(), to);
        assert_eq!(edge.weight(), 1);
    }

    #[test]
    fn test_edge_weight() {
        let mut edge = Edge::new(blake3::hash(b"parent"), blake3::hash(b"child"));

        assert_eq!(edge.weight(), 1);
        edge.increment_weight();
        assert_eq!(edge.weight(), 2);

        // Test saturation
        for _ in 0..u32::MAX {
            edge.increment_weight();
        }
        assert_eq!(edge.weight(), u32::MAX);
    }
}
