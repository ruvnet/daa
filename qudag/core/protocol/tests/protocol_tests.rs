use qudag_crypto::{KeyPair, PublicKey};
use qudag_dag::QrDag;
use qudag_network::NetworkManager;
use qudag_protocol::{Coordinator, ProtocolConfig, ProtocolState};

#[tokio::test]
async fn test_protocol_initialization() {
    let config = ProtocolConfig::default();
    let coordinator = Coordinator::new(config).await.unwrap();
    assert!(coordinator.is_initialized());
}

#[tokio::test]
async fn test_protocol_state_transitions() {
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();

    // Test state transitions
    assert_eq!(coordinator.state(), ProtocolState::Initialized);
    coordinator.start().await.unwrap();
    assert_eq!(coordinator.state(), ProtocolState::Running);
    coordinator.stop().await.unwrap();
    assert_eq!(coordinator.state(), ProtocolState::Stopped);
}

#[tokio::test]
async fn test_component_integration() {
    let config = ProtocolConfig::default();
    let coordinator = Coordinator::new(config).await.unwrap();

    // Verify all components are properly initialized
    assert!(coordinator.crypto_manager().is_some());
    assert!(coordinator.network_manager().is_some());
    assert!(coordinator.dag_manager().is_some());
}

#[tokio::test]
async fn test_message_propagation() {
    let config = ProtocolConfig::default();
    let mut coordinator = Coordinator::new(config).await.unwrap();
    coordinator.start().await.unwrap();

    // Test message propagation through the system
    let message = vec![1, 2, 3, 4];
    let result = coordinator.broadcast_message(message.clone()).await;
    assert!(result.is_ok());

    // Verify message was processed by DAG
    let dag = coordinator.dag_manager().unwrap();
    assert!(dag.contains_message(&message));
}

#[tokio::test]
async fn test_error_handling() {
    let mut config = ProtocolConfig::default();
    config.network_port = 0; // Invalid port to trigger error
    let result = Coordinator::new(config).await;
    assert!(result.is_err());
}

proptest::proptest! {
    #[test]
    fn test_protocol_with_random_messages(message in proptest::collection::vec(0u8..255, 1..1000)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = ProtocolConfig::default();
            let mut coordinator = Coordinator::new(config).await.unwrap();
            coordinator.start().await.unwrap();

            let result = coordinator.broadcast_message(message).await;
            assert!(result.is_ok());
        });
    }
}
