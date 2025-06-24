use qudag_protocol::{Coordinator, ProtocolConfig, ProtocolState};
use qudag_crypto::KeyPair;
use qudag_dag::QrDag;
use qudag_network::NetworkManager;
use qudag_cli::{CliConfig, NodeCommand, NodeRunner};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_cli_node_startup() {
    // Initialize CLI configuration
    let cli_config = CliConfig {
        network_port: 9001,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-9001".into(),
    };

    // Create and start node through CLI
    let node_runner = NodeRunner::new(cli_config).await.unwrap();
    node_runner.execute(NodeCommand::Start).await.unwrap();

    // Allow time for startup
    sleep(Duration::from_millis(100)).await;

    // Verify node is running
    assert_eq!(node_runner.state().await, ProtocolState::Running);

    // Clean shutdown
    node_runner.execute(NodeCommand::Stop).await.unwrap();
    assert_eq!(node_runner.state().await, ProtocolState::Stopped);
}

#[tokio::test]
async fn test_cli_message_broadcast() {
    // Initialize two CLI nodes
    let cli_config1 = CliConfig {
        network_port: 9002,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-9002".into(),
    };

    let cli_config2 = CliConfig {
        network_port: 9003,
        bootstrap_nodes: vec!["127.0.0.1:9002".to_string()],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-9003".into(),
    };

    let node_runner1 = NodeRunner::new(cli_config1).await.unwrap();
    let node_runner2 = NodeRunner::new(cli_config2).await.unwrap();

    // Start both nodes
    node_runner1.execute(NodeCommand::Start).await.unwrap();
    node_runner2.execute(NodeCommand::Start).await.unwrap();

    // Allow time for network formation
    sleep(Duration::from_millis(200)).await;

    // Broadcast message through CLI command
    let test_message = b"cli test message".to_vec();
    node_runner1.execute(NodeCommand::Broadcast(test_message.clone())).await.unwrap();

    // Allow time for message propagation
    sleep(Duration::from_millis(200)).await;

    // Verify message was received by both nodes
    assert!(node_runner1.dag_contains_message(&test_message).await.unwrap());
    assert!(node_runner2.dag_contains_message(&test_message).await.unwrap());

    // Clean shutdown
    node_runner1.execute(NodeCommand::Stop).await.unwrap();
    node_runner2.execute(NodeCommand::Stop).await.unwrap();
}

#[tokio::test]
async fn test_cli_crypto_operations() {
    // Initialize CLI node with crypto capabilities
    let cli_config = CliConfig {
        network_port: 9004,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-9004".into(),
    };

    let node_runner = NodeRunner::new(cli_config).await.unwrap();
    node_runner.execute(NodeCommand::Start).await.unwrap();

    // Test key generation through CLI
    let keypair = node_runner.execute(NodeCommand::GenerateKeypair).await.unwrap();
    
    // Create and sign message
    let message = b"crypto test message".to_vec();
    let signature = node_runner.execute(NodeCommand::SignMessage(message.clone())).await.unwrap();
    
    // Verify signature
    assert!(node_runner.execute(NodeCommand::VerifySignature(message, signature, keypair.public_key())).await.unwrap());

    node_runner.execute(NodeCommand::Stop).await.unwrap();
}

#[tokio::test]
async fn test_cli_network_management() {
    // Initialize network management test node
    let cli_config = CliConfig {
        network_port: 9005,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-9005".into(),
    };

    let node_runner = NodeRunner::new(cli_config).await.unwrap();
    node_runner.execute(NodeCommand::Start).await.unwrap();

    // Test peer connection commands
    let peer_addr = "127.0.0.1:9006";
    node_runner.execute(NodeCommand::Connect(peer_addr.to_string())).await.unwrap();
    
    // Verify peer list
    let peers = node_runner.execute(NodeCommand::ListPeers).await.unwrap();
    assert!(peers.contains(&peer_addr.to_string()));

    // Test peer disconnection
    node_runner.execute(NodeCommand::Disconnect(peer_addr.to_string())).await.unwrap();
    let peers = node_runner.execute(NodeCommand::ListPeers).await.unwrap();
    assert!(!peers.contains(&peer_addr.to_string()));

    node_runner.execute(NodeCommand::Stop).await.unwrap();
}

#[tokio::test]
async fn test_cli_dag_operations() {
    // Initialize DAG test node
    let cli_config = CliConfig {
        network_port: 9007,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-9007".into(),
    };

    let node_runner = NodeRunner::new(cli_config).await.unwrap();
    node_runner.execute(NodeCommand::Start).await.unwrap();

    // Submit multiple messages to create DAG structure
    let messages = vec![
        b"dag_message_1".to_vec(),
        b"dag_message_2".to_vec(),
        b"dag_message_3".to_vec(),
    ];

    for msg in &messages {
        node_runner.execute(NodeCommand::Broadcast(msg.clone())).await.unwrap();
        sleep(Duration::from_millis(50)).await;
    }

    // Test DAG traversal and verification
    let dag_info = node_runner.execute(NodeCommand::GetDagInfo).await.unwrap();
    assert_eq!(dag_info.message_count, messages.len());

    // Verify message ordering
    let ordered_messages = node_runner.execute(NodeCommand::GetOrderedMessages).await.unwrap();
    assert_eq!(ordered_messages.len(), messages.len());

    node_runner.execute(NodeCommand::Stop).await.unwrap();
}

#[tokio::test]
async fn test_cli_error_handling() {
    // Test invalid configuration
    let invalid_config = CliConfig {
        network_port: 0, // Invalid port
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-invalid".into(),
    };

    let result = NodeRunner::new(invalid_config).await;
    assert!(result.is_err());

    // Test operations on stopped node
    let valid_config = CliConfig {
        network_port: 9008,
        bootstrap_nodes: vec![],
        max_peers: 10,
        validation_timeout: 1000,
        data_dir: "/tmp/qudag-test-9008".into(),
    };

    let node_runner = NodeRunner::new(valid_config).await.unwrap();
    
    // Try operations before start
    let result = node_runner.execute(NodeCommand::Broadcast(vec![1,2,3])).await;
    assert!(result.is_err());

    // Try invalid peer connection
    node_runner.execute(NodeCommand::Start).await.unwrap();
    let result = node_runner.execute(NodeCommand::Connect("invalid:addr".to_string())).await;
    assert!(result.is_err());

    node_runner.execute(NodeCommand::Stop).await.unwrap();
}