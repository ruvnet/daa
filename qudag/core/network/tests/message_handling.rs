use qudag_network::{Message, MessageHandler, NetworkError, PeerId, Route};
use tokio_test::*;

#[tokio::test]
async fn test_message_throughput() {
    const MSG_COUNT: usize = 10_000;

    let handler = MessageHandler::new();
    let mut handles = vec![];

    // Spawn multiple senders
    for i in 0..4 {
        let handler = handler.clone();
        let handle = tokio::spawn(async move {
            for j in 0..MSG_COUNT {
                let msg = Message::new(
                    format!("test_msg_{}_{}", i, j).into(),
                    PeerId::random(),
                    Route::direct(),
                );
                handler.send(msg).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all messages to be sent
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify throughput
    let stats = handler.get_stats();
    assert!(
        stats.messages_per_second() >= 10_000.0,
        "Message throughput below target: {} msg/s",
        stats.messages_per_second()
    );
}

#[tokio::test]
async fn test_anonymous_routing() {
    let handler = MessageHandler::new();

    // Create a route with multiple hops
    let route = Route::new()
        .add_hop(PeerId::random())
        .add_hop(PeerId::random())
        .add_hop(PeerId::random());

    let msg = Message::new("test_anonymous_msg".into(), PeerId::random(), route.clone());

    // Send message through route
    handler.send(msg).await.unwrap();

    // Verify route privacy
    let routed_msg = handler.receive().await.unwrap();
    assert_eq!(routed_msg.route().next_hop(), route.next_hop());
    assert!(routed_msg.route().is_anonymous());
    assert!(!routed_msg.route().reveals_sender());
}

#[tokio::test]
async fn test_message_encryption() {
    let handler = MessageHandler::new();
    let content = b"secret message";

    let msg = Message::new(content.to_vec(), PeerId::random(), Route::direct()).encrypt();

    // Send encrypted message
    handler.send(msg).await.unwrap();

    // Receive and decrypt
    let received = handler.receive().await.unwrap();
    let decrypted = received.decrypt().unwrap();

    assert_eq!(decrypted.content(), content);
    assert!(received.is_encrypted());
}

#[tokio::test]
async fn test_error_handling() {
    let handler = MessageHandler::new();

    // Test invalid route
    let result = handler
        .send(Message::new(
            vec![],
            PeerId::random(),
            Route::new(), // Empty route
        ))
        .await;

    assert!(matches!(result, Err(NetworkError::InvalidRoute)));

    // Test message too large
    let large_msg = vec![0u8; 1024 * 1024 * 100]; // 100MB
    let result = handler
        .send(Message::new(large_msg, PeerId::random(), Route::direct()))
        .await;

    assert!(matches!(result, Err(NetworkError::MessageTooLarge)));
}
