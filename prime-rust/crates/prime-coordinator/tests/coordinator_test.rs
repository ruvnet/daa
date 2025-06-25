use prime_coordinator::{Coordinator, CoordinatorConfig, GovernancePolicy, TaskAllocation};
use daa_ai::agents::Agent;
use prime_core::NodeIdentity;

#[tokio::test]
async fn test_coordinator_initialization() {
    let config = CoordinatorConfig {
        consensus_threshold: 0.66,
        min_nodes: 3,
        max_nodes: 100,
    };
    
    let coordinator = Coordinator::new(config).await.unwrap();
    assert_eq!(coordinator.consensus_threshold(), 0.66);
}

#[tokio::test]
async fn test_node_registration() {
    let config = CoordinatorConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    
    let node1 = NodeIdentity::new("node-001");
    let node2 = NodeIdentity::new("node-002");
    
    coordinator.register_node(node1).await.unwrap();
    coordinator.register_node(node2).await.unwrap();
    
    assert_eq!(coordinator.active_nodes().await, 2);
}

#[tokio::test]
async fn test_governance_policy_creation() {
    let policy = GovernancePolicy {
        voting_period: std::time::Duration::from_secs(3600),
        quorum: 0.5,
        approval_threshold: 0.66,
    };
    
    assert!(policy.validate().is_ok());
}

#[tokio::test]
async fn test_task_allocation() {
    let config = CoordinatorConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    
    // Register nodes
    for i in 0..5 {
        let node = NodeIdentity::new(&format!("node-{:03}", i));
        coordinator.register_node(node).await.unwrap();
    }
    
    let allocation = coordinator.allocate_training_task("model-001").await.unwrap();
    assert!(!allocation.node_assignments.is_empty());
    assert_eq!(allocation.task_id, "model-001");
}

#[tokio::test]
async fn test_consensus_mechanism() {
    let config = CoordinatorConfig {
        consensus_threshold: 0.66,
        min_nodes: 3,
        max_nodes: 100,
    };
    
    let mut coordinator = Coordinator::new(config).await.unwrap();
    
    // Register nodes
    for i in 0..5 {
        let node = NodeIdentity::new(&format!("node-{:03}", i));
        coordinator.register_node(node).await.unwrap();
    }
    
    // Submit votes
    let proposal_id = "proposal-001";
    coordinator.submit_vote(proposal_id, "node-000", true).await.unwrap();
    coordinator.submit_vote(proposal_id, "node-001", true).await.unwrap();
    coordinator.submit_vote(proposal_id, "node-002", true).await.unwrap();
    coordinator.submit_vote(proposal_id, "node-003", false).await.unwrap();
    
    let result = coordinator.tally_votes(proposal_id).await.unwrap();
    assert!(result.approved); // 3/4 votes = 75% > 66% threshold
}

#[tokio::test]
async fn test_daa_agent_integration() {
    let config = CoordinatorConfig::default();
    let coordinator = Coordinator::new(config).await.unwrap();
    
    let agent = coordinator.create_governance_agent("governance-001").await.unwrap();
    assert_eq!(agent.name(), "governance-001");
}