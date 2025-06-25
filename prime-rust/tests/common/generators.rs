//! Advanced test data generators using property-based testing

use proptest::prelude::*;
use arbitrary::{Arbitrary, Unstructured};
use quickcheck::{Arbitrary as QcArbitrary, Gen};
use std::collections::HashMap;

/// Generate valid consensus messages
#[derive(Debug, Clone, PartialEq)]
pub struct ConsensusMessage {
    pub msg_type: ConsensusMessageType,
    pub round: u64,
    pub proposer: String,
    pub value: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusMessageType {
    Propose,
    Vote,
    Commit,
    ViewChange,
}

impl Arbitrary for ConsensusMessage {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        Ok(ConsensusMessage {
            msg_type: u.arbitrary()?,
            round: u.arbitrary()?,
            proposer: format!("peer_{}", u.arbitrary::<u32>()?),
            value: u.arbitrary()?,
            signature: (0..64).map(|_| u.arbitrary().unwrap_or(0)).collect(),
        })
    }
}

impl Arbitrary for ConsensusMessageType {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0..=3)? {
            0 => ConsensusMessageType::Propose,
            1 => ConsensusMessageType::Vote,
            2 => ConsensusMessageType::Commit,
            _ => ConsensusMessageType::ViewChange,
        })
    }
}

/// Generate DHT operations
#[derive(Debug, Clone)]
pub enum DhtOperation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Get { key: Vec<u8> },
    FindNode { target: String },
    FindProviders { key: Vec<u8> },
    Provide { key: Vec<u8> },
}

impl QcArbitrary for DhtOperation {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&[0, 1, 2, 3, 4]).unwrap() {
            0 => DhtOperation::Put {
                key: Vec::<u8>::arbitrary(g),
                value: Vec::<u8>::arbitrary(g),
            },
            1 => DhtOperation::Get {
                key: Vec::<u8>::arbitrary(g),
            },
            2 => DhtOperation::FindNode {
                target: format!("peer_{}", u32::arbitrary(g)),
            },
            3 => DhtOperation::FindProviders {
                key: Vec::<u8>::arbitrary(g),
            },
            _ => DhtOperation::Provide {
                key: Vec::<u8>::arbitrary(g),
            },
        }
    }
}

/// Generate training scenarios
pub fn training_scenario() -> impl Strategy<Value = TrainingScenario> {
    (
        1usize..10usize,  // num_epochs
        prop::collection::vec(gradient_update(), 1..100),
        0.0f32..1.0f32,  // target_accuracy
        prop::option::of(failure_scenario()),
    ).prop_map(|(epochs, updates, accuracy, failure)| {
        TrainingScenario {
            num_epochs: epochs,
            gradient_updates: updates,
            target_accuracy: accuracy,
            failure_scenario: failure,
        }
    })
}

#[derive(Debug, Clone)]
pub struct TrainingScenario {
    pub num_epochs: usize,
    pub gradient_updates: Vec<GradientUpdate>,
    pub target_accuracy: f32,
    pub failure_scenario: Option<FailureScenario>,
}

#[derive(Debug, Clone)]
pub struct GradientUpdate {
    pub node_id: String,
    pub gradients: HashMap<String, Vec<f32>>,
    pub loss: f32,
    pub timestamp: u64,
}

fn gradient_update() -> impl Strategy<Value = GradientUpdate> {
    (
        "[a-zA-Z0-9]{10}",
        prop::collection::hash_map(
            "[a-zA-Z0-9_]{5,10}",
            prop::collection::vec(0.0f32..1.0f32, 10..100),
            1..5,
        ),
        0.0f32..10.0f32,
        0u64..1000000u64,
    ).prop_map(|(node_id, gradients, loss, timestamp)| {
        GradientUpdate {
            node_id,
            gradients,
            loss,
            timestamp,
        }
    })
}

#[derive(Debug, Clone)]
pub enum FailureScenario {
    NodeCrash { node_id: String, at_epoch: usize },
    NetworkPartition { duration_updates: usize },
    ByzantineNode { node_id: String, behavior: ByzantineBehavior },
    SlowNode { node_id: String, slowdown_factor: f32 },
}

#[derive(Debug, Clone)]
pub enum ByzantineBehavior {
    RandomGradients,
    InvertedGradients,
    StaleGradients,
    DropUpdates,
}

fn failure_scenario() -> impl Strategy<Value = FailureScenario> {
    prop_oneof![
        ("[a-zA-Z0-9]{10}", 0usize..10usize).prop_map(|(id, epoch)| {
            FailureScenario::NodeCrash { node_id: id, at_epoch: epoch }
        }),
        (1usize..100usize).prop_map(|duration| {
            FailureScenario::NetworkPartition { duration_updates: duration }
        }),
        ("[a-zA-Z0-9]{10}", byzantine_behavior()).prop_map(|(id, behavior)| {
            FailureScenario::ByzantineNode { node_id: id, behavior }
        }),
        ("[a-zA-Z0-9]{10}", 2.0f32..10.0f32).prop_map(|(id, factor)| {
            FailureScenario::SlowNode { node_id: id, slowdown_factor: factor }
        }),
    ]
}

fn byzantine_behavior() -> impl Strategy<Value = ByzantineBehavior> {
    prop_oneof![
        Just(ByzantineBehavior::RandomGradients),
        Just(ByzantineBehavior::InvertedGradients),
        Just(ByzantineBehavior::StaleGradients),
        Just(ByzantineBehavior::DropUpdates),
    ]
}

/// Generate network topology scenarios
pub fn network_topology_scenario() -> impl Strategy<Value = NetworkTopologyScenario> {
    (
        5usize..50usize,  // num_nodes
        topology_type(),
        prop::collection::vec(topology_change(), 0..10),
    ).prop_map(|(nodes, initial, changes)| {
        NetworkTopologyScenario {
            num_nodes: nodes,
            initial_topology: initial,
            topology_changes: changes,
        }
    })
}

#[derive(Debug, Clone)]
pub struct NetworkTopologyScenario {
    pub num_nodes: usize,
    pub initial_topology: TopologyType,
    pub topology_changes: Vec<TopologyChange>,
}

#[derive(Debug, Clone)]
pub enum TopologyType {
    FullMesh,
    Star,
    Ring,
    Tree { fanout: usize },
    Random { connectivity: f64 },
}

fn topology_type() -> impl Strategy<Value = TopologyType> {
    prop_oneof![
        Just(TopologyType::FullMesh),
        Just(TopologyType::Star),
        Just(TopologyType::Ring),
        (2usize..5usize).prop_map(|f| TopologyType::Tree { fanout: f }),
        (0.1f64..1.0f64).prop_map(|c| TopologyType::Random { connectivity: c }),
    ]
}

#[derive(Debug, Clone)]
pub enum TopologyChange {
    AddNode { node_id: String },
    RemoveNode { node_id: String },
    AddLink { from: String, to: String },
    RemoveLink { from: String, to: String },
    PartitionNetwork { groups: Vec<Vec<String>> },
    HealPartition,
}

fn topology_change() -> impl Strategy<Value = TopologyChange> {
    prop_oneof![
        "[a-zA-Z0-9]{10}".prop_map(|id| TopologyChange::AddNode { node_id: id }),
        "[a-zA-Z0-9]{10}".prop_map(|id| TopologyChange::RemoveNode { node_id: id }),
        ("[a-zA-Z0-9]{10}", "[a-zA-Z0-9]{10}").prop_map(|(from, to)| {
            TopologyChange::AddLink { from, to }
        }),
        ("[a-zA-Z0-9]{10}", "[a-zA-Z0-9]{10}").prop_map(|(from, to)| {
            TopologyChange::RemoveLink { from, to }
        }),
        Just(TopologyChange::HealPartition),
    ]
}

/// Generate complex system states for invariant testing
pub fn system_state() -> impl Strategy<Value = SystemState> {
    (
        prop::collection::hash_map(
            "[a-zA-Z0-9]{10}",
            node_state(),
            1..20,
        ),
        0u64..1000u64,
        prop::collection::vec(pending_transaction(), 0..50),
    ).prop_map(|(nodes, epoch, transactions)| {
        SystemState {
            nodes,
            current_epoch: epoch,
            pending_transactions: transactions,
        }
    })
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub nodes: HashMap<String, NodeState>,
    pub current_epoch: u64,
    pub pending_transactions: Vec<Transaction>,
}

#[derive(Debug, Clone)]
pub struct NodeState {
    pub is_active: bool,
    pub model_version: u64,
    pub training_progress: f32,
    pub connections: Vec<String>,
}

fn node_state() -> impl Strategy<Value = NodeState> {
    (
        prop::bool::ANY,
        0u64..100u64,
        0.0f32..1.0f32,
        prop::collection::vec("[a-zA-Z0-9]{10}", 0..10),
    ).prop_map(|(active, version, progress, connections)| {
        NodeState {
            is_active: active,
            model_version: version,
            training_progress: progress,
            connections,
        }
    })
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_type: TransactionType,
    pub sender: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum TransactionType {
    ModelUpdate,
    ParameterSync,
    RewardClaim,
    NodeRegistration,
}

fn pending_transaction() -> impl Strategy<Value = Transaction> {
    (
        transaction_type(),
        "[a-zA-Z0-9]{10}",
        prop::collection::vec(0u8..255u8, 0..1000),
    ).prop_map(|(tx_type, sender, data)| {
        Transaction {
            tx_type,
            sender,
            data,
        }
    })
}

fn transaction_type() -> impl Strategy<Value = TransactionType> {
    prop_oneof![
        Just(TransactionType::ModelUpdate),
        Just(TransactionType::ParameterSync),
        Just(TransactionType::RewardClaim),
        Just(TransactionType::NodeRegistration),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_consensus_message_generation(msg in any::<ConsensusMessage>()) {
            assert!(!msg.proposer.is_empty());
            assert_eq!(msg.signature.len(), 64);
        }

        #[test]
        fn test_training_scenario_generation(scenario in training_scenario()) {
            assert!(scenario.num_epochs > 0);
            assert!(!scenario.gradient_updates.is_empty());
            assert!(scenario.target_accuracy >= 0.0 && scenario.target_accuracy <= 1.0);
        }

        #[test]
        fn test_network_topology_generation(scenario in network_topology_scenario()) {
            assert!(scenario.num_nodes >= 5);
        }

        #[test]
        fn test_system_state_generation(state in system_state()) {
            assert!(!state.nodes.is_empty());
            for (_, node) in &state.nodes {
                assert!(node.training_progress >= 0.0 && node.training_progress <= 1.0);
            }
        }
    }

    quickcheck! {
        fn test_dht_operation_quickcheck(op: DhtOperation) -> bool {
            match op {
                DhtOperation::Put { key, value } => !key.is_empty(),
                DhtOperation::Get { key } => !key.is_empty(),
                DhtOperation::FindNode { target } => !target.is_empty(),
                DhtOperation::FindProviders { key } => !key.is_empty(),
                DhtOperation::Provide { key } => !key.is_empty(),
            }
        }
    }
}