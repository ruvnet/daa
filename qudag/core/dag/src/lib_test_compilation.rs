//! Compilation test to ensure all exported types are accessible
//! This is a compile-time test that ensures all public API is properly exposed

#![allow(dead_code, unused_imports)]

// Test that all core types can be imported
use crate::{
    AdvancedTipSelection,
    Confidence,

    // Consensus types
    Consensus,
    // Configuration types
    ConsensusConfig,

    ConsensusError,
    ConsensusMetrics,
    ConsensusStatus,
    Dag,
    DagError as DagModuleError,

    // Error types
    DagError,

    DagMessage,
    // Edge types
    Edge,

    // Graph types
    Graph,
    GraphMetrics,
    // Node types
    Node,
    NodeState,
    ParentSelectionAlgorithm,
    QRAvalanche,
    QRAvalancheConfig,
    // Main DAG types
    QrDag,
    // Result type
    Result,
    SerializableHash,

    StorageConfig,

    // Tip selection types
    TipSelection,
    TipSelectionConfig,
    TipSelectionError,
    // Vertex types
    Vertex,
    VertexError,
    VertexId,
    VertexOps,

    VertexWeight,

    VotingRecord,
};

/// Compile-time test function that uses all major types
/// This function is never called but ensures compilation succeeds
fn _compile_test() -> Result<()> {
    // Test basic type construction
    let _vertex_id: VertexId = VertexId::new();
    let _vertex: Vertex = Vertex::new(
        _vertex_id.clone(),
        vec![1, 2, 3],
        std::collections::HashSet::new(),
    );
    let _dag: QrDag = QrDag::new();
    let _consensus: QRAvalanche = QRAvalanche::new();
    let _graph: Graph = Graph::new();
    let _node: Node = Node::new(vec![1, 2, 3], vec![]);

    // Test configuration types
    let _dag_config: ConsensusConfig = ConsensusConfig::default();
    let _qr_config: QRAvalancheConfig = QRAvalancheConfig::default();
    let _storage_config: StorageConfig = StorageConfig::default();
    let _tip_config: TipSelectionConfig = TipSelectionConfig::default();

    // Test enum types
    let _consensus_status: ConsensusStatus = ConsensusStatus::Pending;
    let _node_state: NodeState = NodeState::Pending;

    // Test error types
    let _vertex_error: VertexError = VertexError::InvalidParent;
    let _consensus_error: ConsensusError = ConsensusError::InvalidVertex;
    let _dag_error: DagError = DagError::NodeExists("test".to_string());

    // Test result type
    let _result: Result<()> = Ok(());

    Ok(())
}

/// Test that trait methods are accessible
fn _trait_test() {
    // Test that traits can be used in generic contexts
    fn _use_consensus<T: Consensus>(_consensus: T) {}
    fn _use_tip_selection<T: TipSelection>(_tip_selection: T) {}
    fn _use_vertex_ops<T: VertexOps>(_vertex_ops: T) {}

    // Ensure traits are object-safe by testing dynamic dispatch
    let _consensus_trait: Box<dyn Consensus> = panic!("compile test only");
    // Note: TipSelection and VertexOps traits are not object-safe due to associated functions
    // let _tip_selection_trait: Box<dyn TipSelection> = panic!("compile test only");
    // let _vertex_ops_trait: Box<dyn VertexOps> = panic!("compile test only");
}

/// Test that all types implement expected standard traits
fn _standard_traits_test() {
    // Test Debug trait (required for most types)
    fn _test_debug<T: std::fmt::Debug>(_t: T) {}

    _test_debug(VertexId::new());
    _test_debug(ConsensusStatus::Pending);
    _test_debug(NodeState::Pending);
    _test_debug(VertexError::InvalidParent);
    _test_debug(ConsensusError::InvalidVertex);
    _test_debug(DagError::NodeExists("test".to_string()));

    // Test Clone trait (for types that should be cloneable)
    fn _test_clone<T: Clone>(_t: T) {}

    _test_clone(VertexId::new());
    _test_clone(ConsensusStatus::Pending);
    _test_clone(NodeState::Pending);
    _test_clone(ConsensusConfig::default());
    _test_clone(QRAvalancheConfig::default());
    _test_clone(StorageConfig::default());
}

// Ensure this file is only compiled during testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compilation_test() {
        // This test simply ensures the module compiles
        // All the actual testing happens at compile-time
        println!("All types compile successfully");
    }
}
